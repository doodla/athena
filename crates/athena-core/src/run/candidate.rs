//! The candidate stages of a job: building the candidate from the producer's change list and putting it
//! through admission (design §2.3, structure-pass §3.3, stages S4 and S5).

use super::*;

/// An admitted candidate (stage S5): the commit built in the user's repository and its fingerprint.
pub(super) struct Admitted {
    pub(super) commit: String,
    pub(super) fingerprint: String,
}

impl Athena {
    /// Stages S4 and S5: build the candidate from the snapshot's change list, then run admission on it.
    /// `Flow::Stop` means the job ended here (an idle producer, or a refused change) and `closing` says how.
    pub(super) fn candidate_stage(
        &self,
        ctx: &RunContext<'_>,
        closing: &mut Closing,
        report: Option<ProducerReport>,
        report_gaps: &[String],
        report_kept_in: &str,
        snapshot: &Snapshot,
    ) -> Result<Flow<Admitted>> {
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let p = ctx.prediction.clone();

        // The candidate: the snapshot's change list, applied to the base's full tree in the user's repository.
        let changes = snapshot.change_list()?;
        // The chain links (athena.candidate/2): the base's own fingerprint, under its own rules, and the
        // latest job landed in the base's history.
        let base_fingerprint = fingerprint(&self.git, &p, &p)?;
        let previous_job = latest_landed_job(&self.git, &p)?;
        let mut candidate = Candidate {
            job: id.clone(),
            base_commit: p.clone(),
            base_fingerprint: Some(base_fingerprint),
            previous_job,
            candidate_commit: None,
            candidate_tree: None,
            changes: Vec::new(),
            fingerprint: None,
            violations: Vec::new(),
            producer_report: report,
        };
        // The change list is against the base, so it is empty only when the candidate does nothing at all.
        // A retry seeded from a kept candidate already differs from the base: if its producer adds nothing
        // (the kept candidate did the work and only evidence was missing), the seeded candidate is the
        // candidate and goes on to admission, checks and the verifier. Only a run with no change against the
        // base fails as an idle producer.
        if changes.is_empty() {
            closing.candidate = Some(candidate);
            let reason = if report_gaps.is_empty() {
                "the producer made no change".to_string()
            } else {
                format!(
                    "the producer made no change, and its report still had problems when sent back: {}; the report is kept in {report_kept_in}",
                    report_gaps.join("; ")
                )
            };
            closing.happened = self.failure(ctx, &reason);
            return Ok(Flow::Stop);
        }
        let new_blobs: Vec<String> = changes
            .iter()
            .filter(|c| c.status != 'D')
            .map(|c| c.new_oid.clone())
            .collect();
        let sizes = snapshot.git().blob_sizes(&new_blobs)?;
        let mut edits = Vec::new();
        let mut changed = Vec::new();
        for change in &changes {
            let size = sizes.get(&change.new_oid).copied().unwrap_or(0);
            if change.status == 'D' {
                edits.push(IndexEdit::Remove {
                    path: change.path.clone(),
                });
            } else {
                if change.new_mode != "160000" {
                    let oid = self.git.hash_object_w(&snapshot.blob(&change.new_oid)?)?;
                    debug_assert_eq!(oid, change.new_oid, "blob ids carry over unchanged (E15)");
                }
                edits.push(IndexEdit::Put {
                    mode: change.new_mode.clone(),
                    oid: change.new_oid.clone(),
                    path: change.path.clone(),
                });
            }
            candidate.changes.push(ChangeEntry {
                path: change.path.clone(),
                status: change_status(change.status),
                mode: (change.status != 'D').then(|| change.new_mode.clone()),
                bytes: (change.status != 'D').then_some(size),
            });
            changed.push(Changed {
                path: change.path.clone(),
                status: change_status(change.status),
                mode: change.new_mode.clone(),
                size,
            });
        }
        closing.happened.changed_paths = changes
            .iter()
            .map(|c| c.path.clone())
            .collect::<BTreeSet<_>>();
        let index = TempIndex::from_tree(&self.git, &p)?;
        index.apply(&edits)?;
        let tree = index.write_tree()?;
        let c = self.git.commit_tree(
            &tree,
            &[&p],
            &format!(
                "Candidate for job {id}\n\nNot on any branch; landing builds its own commit.\n"
            ),
            false,
        )?;
        candidate.candidate_commit = Some(c.clone());
        candidate.candidate_tree = Some(tree.clone());

        // Admission: the core's rules, then the capability's.
        let mut violations: Vec<PolicyViolation> = admit(&AdmissionInput {
            git: &self.git,
            base: &p,
            candidate: &c,
            changes: &changed,
            reading: ctx.reading,
            max_file_bytes: jc.config.records.max_file_bytes,
            personal_data: &jc.config.personal_data,
            config: &jc.config,
        })?;
        violations.extend(ctx.capability.admit(
            &ChangeList {
                entries: candidate.changes.clone(),
            },
            jc,
        ));
        let f = fingerprint(&self.git, &c, &p)?;
        candidate.fingerprint = Some(f.clone());
        let stat: Vec<String> = candidate
            .changes
            .iter()
            .map(|e| format!("{} ({})", e.path, format!("{:?}", e.status).to_lowercase()))
            .collect();
        ctx.ui.line(&format!(
            "candidate  {} · fingerprint {}…",
            stat.join(", "),
            &f[..18.min(f.len())]
        ));
        if !violations.is_empty() {
            let summary = violations
                .iter()
                .map(|v| v.detail.clone())
                .collect::<Vec<_>>()
                .join("; ");
            candidate.violations = violations;
            closing.candidate = Some(candidate);
            let changed_paths = std::mem::take(&mut closing.happened.changed_paths);
            closing.happened =
                self.failure(ctx, &format!("admission refused the change: {summary}"));
            closing.happened.changed_paths = changed_paths;
            return Ok(Flow::Stop);
        }
        closing.candidate = Some(candidate);
        Ok(Flow::Go(Admitted {
            commit: c,
            fingerprint: f,
        }))
    }
}
