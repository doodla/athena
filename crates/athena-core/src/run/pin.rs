//! Pinning the studied repository and rereading after a configuration change (design §3.4, D38).

use super::*;

impl Athena {
    /// For a capability that pins, resolves the commit to read and checks its tree out under the job's
    /// directory. Nothing is pinned for other capabilities.
    ///
    /// The rule: an existing study's `studies/<id>/pin.json` overrides `repo_ref`. When the committed head holds
    /// a study whose `repository_url` equals the configured `repo_url`, the job pins exactly that study's
    /// `sha`; athena.toml's `repo_ref` (a branch such as `main`) is used only for a new study, one with no
    /// pin.json for the repository. With several studies for the repository, the one whose `study_id` appears
    /// in the request text is used; if none or more than one is named, the job stops rather than guess.
    /// A pin.json whose `sha` is not a full 40-hex id, or a pin that does not resolve to that same sha, stops
    /// the job before dispatch with a message naming the study; it never falls back to `repo_ref`.
    ///
    /// The cause this fixes: job 20261015-201650-cyvb mounted fc8972cd (the tip of `main`) while the study's
    /// pin.json said 6407052f, so the producer cited lines of newer code under the older label.
    ///
    /// A pinning capability whose configuration names no repository, or a ref that does not resolve, also
    /// stops the job before dispatch.
    pub(super) fn pin_repo(
        &self,
        capability: &dyn Capability,
        jc: &JobConfig,
        id: &JobId,
        head: &str,
        request: &str,
    ) -> Result<Option<(PinnedRepo, PathBuf)>> {
        if !capability.pins_repo() {
            return Ok(None);
        }
        let section = jc.config.capability(capability.id());
        let (Some(url), Some(git_ref)) = (
            section.and_then(|c| c.repo_url.clone()),
            section.and_then(|c| c.repo_ref.clone()),
        ) else {
            bail!(
                "capability {} studies a repository, but athena.toml [capabilities.{}] sets no repo_url and repo_ref to pin",
                capability.id(),
                capability.id()
            );
        };
        if let Some((_, rest)) = url.split_once("://")
            && rest.split('/').next().is_some_and(|a| a.contains('@'))
        {
            bail!("repo_url must not carry credentials; it is recorded in the job record");
        }
        let dir = self.state.job_dir(id).join("pin");
        crate::pin::remove_tree(&dir)?;
        if let Some((study, wanted)) = self.study_pin(head, &url, request)? {
            let sha = self
                .pinner
                .pin(&url, &wanted, &dir)
                .with_context(|| format!("pinning study {study}'s commit {wanted} of {url}"))?;
            if sha != wanted {
                bail!(
                    "study {study}: its pin.json names {wanted}, but the pin resolved to {sha}; the job would read a different commit"
                );
            }
            return Ok(Some((
                PinnedRepo {
                    url,
                    git_ref: wanted,
                    sha,
                },
                dir,
            )));
        }
        let sha = self
            .pinner
            .pin(&url, &git_ref, &dir)
            .with_context(|| format!("pinning {git_ref} of {url}"))?;
        if !is_full_sha(&sha) {
            bail!("the pinner returned {sha:?}, which is not a full 40-hex commit id");
        }
        Ok(Some((PinnedRepo { url, git_ref, sha }, dir)))
    }

    /// The `(study id, sha)` an existing study of the repository at `url` pins, read from `studies/*/pin.json`
    /// in the commit `head`; `None` when no study has a pin for that repository. See [`Self::pin_repo`].
    fn study_pin(&self, head: &str, url: &str, request: &str) -> Result<Option<(String, String)>> {
        let mut found: Vec<(String, String)> = Vec::new();
        for entry in self.git.ls_tree_under(head, "studies/")? {
            let Some(dir) = entry
                .path
                .strip_prefix("studies/")
                .and_then(|p| p.strip_suffix("/pin.json"))
                .filter(|d| !d.contains('/'))
            else {
                continue;
            };
            let bytes = self
                .git
                .show_file(head, &entry.path)?
                .with_context(|| format!("{} vanished", entry.path))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)
                .with_context(|| format!("study {dir}: {} is not valid JSON", entry.path))?;
            if value["repository_url"].as_str() != Some(url) {
                continue;
            }
            let study = value["study_id"].as_str().unwrap_or(dir).to_string();
            let sha = value["sha"].as_str().unwrap_or_default();
            if !is_full_sha(sha) {
                bail!(
                    "study {study}: pin.json sha {sha:?} is not a full 40-hex commit id, so the job cannot pin it; fix the pin"
                );
            }
            found.push((study, sha.to_string()));
        }
        match found.len() {
            0 => Ok(None),
            1 => Ok(found.pop()),
            _ => {
                let mut named: Vec<_> = found
                    .into_iter()
                    .filter(|(study, _)| request.contains(study.as_str()))
                    .collect();
                if named.len() == 1 {
                    Ok(named.pop())
                } else {
                    bail!(
                        "{url} has several studies and the request names {}; name exactly one study id so the job reads that study's pinned commit",
                        if named.is_empty() {
                            "none of them".to_string()
                        } else {
                            "more than one of them".to_string()
                        }
                    )
                }
            }
        }
    }

    /// The reading-validity rule (D38): a reading is valid only for the configuration inputs it was made
    /// under, which are the parsed `athena.toml` settings that bear on a reading
    /// ([`athena_data::config::Config::reading_inputs`]; `[parallel]` and `[landing]` do not) and the bytes
    /// of `defaults.toml`. When they differ, the request is read again here, and [`Athena::dispatch`]
    /// accepts the new reading for a flag-accepted caller when its scope is unchanged.
    pub(super) fn reread_after_configuration_change(
        &self,
        jc: &JobConfig,
        records: &Records,
        job: &JobRecords,
        ui: &dyn Ui,
    ) -> Result<Proposal> {
        let id = job.id().clone();
        let n = job.readings.iter().map(|r| r.n).max().unwrap_or(0);
        if n >= jc.config.budget.max_readings {
            bail!("job {id} has {n} readings, the most a job may have; close it and ask again");
        }
        let redactor = self.redactor(&jc.config, Some(&id))?;
        let request = job.request.clone().context("the job has no request")?;
        let history: Vec<(Reading, Option<String>)> =
            job.readings.iter().map(|r| (r.clone(), None)).collect();
        let spent = job.spent_usd();
        let attempt = crate::reading::earlier_attempt(records, &request.text, &redactor)?;
        let (reading, invocations, error) = self.read_for(
            jc,
            records,
            &id,
            &request,
            attempt.as_ref(),
            &history,
            n + 1,
            job.next_seq(),
            spent,
            &redactor,
            ui,
        )?;
        let invocation = invocations.last().cloned().expect("a read makes one");
        let mut files = crate::reading::invocation_files(&invocations);
        if let Some(r) = &reading {
            files.push(NewFile::record(&AnyRecord::Reading(Box::new(r.clone()))));
        }
        let commit = self.commit_records(
            &files,
            &message(
                "Read again: the configuration changed before dispatch",
                "A reading is valid only for the configuration it was made under (D38).",
                &[("Athena-Job", id.as_str().to_string())],
            ),
        )?;
        Ok(Proposal {
            job: id,
            reading,
            reader_error: error,
            spent_usd: spent + crate::reading::total_cost(&invocations),
            invocation: Some(invocation),
            commit,
        })
    }
}
