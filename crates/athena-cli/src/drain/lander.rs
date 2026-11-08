//! How a verified job is landed, and the checks of a study's model and files that run beside it.

use super::*;

// ---------------------------------------------------------------------------------------------------------
// The real world

/// How a verified job is landed.
pub trait Lander {
    fn land(&self, athena: &Athena, job: &str, ui: &dyn Ui) -> Landing;
}

/// Lands through the pull-request path of `athena land --pr`: push the job's branch, open the pull request,
/// wait for every check run to finish and be green, merge only when `[landing] merge_via_rest` is on. The wait
/// and the merge are one call: checks that turn green during the poll are merged in that same call, in the
/// drain's locked step. Only a merged, verified pull request is a landing; one whose checks are still
/// unfinished when the poll ends is [`Landing::Waiting`].
#[derive(Debug)]
pub struct PrLander;

impl Lander for PrLander {
    fn land(&self, athena: &Athena, job: &str, ui: &dyn Ui) -> Landing {
        let report = athena.land_pr(
            job,
            &GhCli,
            crate::wiring::checks_poll(&std::thread::sleep),
            ui,
        );
        landing_from_report(report)
    }
}

/// What the drain makes of the result of `athena land --pr` for one job. A merged pull request is a landing,
/// including one whose checks turned green during the poll: `land_pr` merges those in the same call, so the
/// drain sees them merged. An open one whose checks are still pending when the poll ends (including a merge the
/// host refused for its repository rules) is [`Landing::Waiting`], not a failure. Anything else is
/// [`Landing::NotLanded`].
pub(super) fn landing_from_report(report: anyhow::Result<PrReport>) -> Landing {
    match report {
        // The paths it changed are read from the job's candidate record by `RealSteps::land`.
        Ok(report) if report.link.state == LinkState::Landed => Landing::Landed {
            changed: Vec::new(),
        },
        Ok(report)
            if report.link.state == LinkState::Open && report.checks == Some(Checks::Pending) =>
        {
            Landing::Waiting(format!(
                "pull request {} is open and its checks have not finished; it merges once they have",
                report.link.pr_number
            ))
        }
        Ok(report) => Landing::NotLanded(format!(
            "pull request {} is {:?}, checks {:?}; it is landed once it is merged",
            report.link.pr_number, report.link.state, report.checks
        )),
        Err(error) => Landing::NotLanded(format!("{error:#}")),
    }
}

/// Where a study model contradicts itself about its counters, as one sentence each. A retry edge that says a
/// counter changes (a text in `retry.changes` that names the counter) means the counter is taken by one
/// transition and freed by each event the text names; the events are its alternatives, split on " or ". The
/// counter's `changed_by` must then list at least one transition for taking and one for each such event. The
/// check reads only the model, so it needs no model call and no network. It is a floor, not a proof: the
/// pinned source is still the authority, and the study's own validation rules still apply.
pub fn model_contradictions(model: &serde_json::Value) -> Vec<String> {
    let list = |v: &serde_json::Value, key: &str| -> Vec<serde_json::Value> {
        v.get(key)
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    for counter in list(model, "counters") {
        let Some(id) = counter.get("id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let changed_by = list(&counter, "changed_by").len();
        for t in list(model, "transitions") {
            let tid = t
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("?");
            let Some(retry) = t.get("retry").filter(|r| !r.is_null()) else {
                continue;
            };
            for text in list(retry, "changes")
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter(|text| text.contains(id))
            {
                let needed = 1 + text.split(" or ").count();
                if changed_by < needed {
                    out.push(format!(
                        "counter {id} lists {changed_by} transition(s) in changed_by, but {tid}'s retry says \"{text}\", which needs {needed}: one that takes a place and one for each way it is freed"
                    ));
                }
            }
        }
    }
    out
}

/// One study directory's files: each file's path relative to the directory, with its bytes.
pub type StudyFiles = Vec<(String, Vec<u8>)>;

/// The files of each study directory under `studies/` at `commit`, with the study's name. Studies are in name
/// order.
pub fn study_files(
    git: &athena_git::Git,
    commit: &str,
) -> anyhow::Result<Vec<(String, StudyFiles)>> {
    let mut studies: std::collections::BTreeMap<String, StudyFiles> =
        std::collections::BTreeMap::new();
    for entry in git.ls_tree_under(commit, "studies")? {
        let Some((name, file)) = entry
            .path
            .strip_prefix("studies/")
            .and_then(|rest| rest.split_once('/'))
        else {
            continue;
        };
        if let Some(bytes) = git.show_file(commit, &entry.path)? {
            studies
                .entry(name.to_string())
                .or_default()
                .push((file.to_string(), bytes));
        }
    }
    Ok(studies.into_iter().collect())
}

/// A study's page text and pin, from its files. Every file but those in `athena_core::pin::STRUCTURAL_FILES`
/// is page text, so a text file added to a study is checked without anyone listing it. Errs, saying why, when
/// the study has no usable pin or a page file is not text.
pub fn study_text(
    name: &str,
    files: &[(String, Vec<u8>)],
) -> Result<athena_core::pin::StudyText, String> {
    let pin = files
        .iter()
        .find(|(f, _)| f == "pin.json")
        .ok_or("it has no pin.json, so there is no source to check against")?;
    let pin: serde_json::Value =
        serde_json::from_slice(&pin.1).map_err(|e| format!("pin.json is not valid JSON: {e}"))?;
    let field = |key: &str| {
        pin.get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or(format!("pin.json has no {key}"))
    };
    let sha = field("sha")?;
    if !athena_core::pin::is_full_sha(&sha) {
        return Err(format!("pin.json's sha {sha:?} is not a full commit id"));
    }
    let mut text = Vec::new();
    for (file, bytes) in files
        .iter()
        .filter(|(f, _)| athena_core::pin::is_page_text(f))
    {
        let body = String::from_utf8(bytes.clone())
            .map_err(|_| format!("{file} is not text, so it cannot be checked"))?;
        text.push(athena_core::pin::TextFile {
            name: file.clone(),
            text: body,
        });
    }
    Ok(athena_core::pin::StudyText {
        name: name.to_string(),
        repository_url: field("repository_url")?,
        sha,
        files: text,
    })
}

/// Builds and installs Athena from the checkout at `dir` by running its `setup.sh`, which is the one command
/// that does it (`./setup.sh`). A checkout with no `setup.sh` is not Athena's own source: nothing is built.
/// A failure is the exit status followed by what the script printed.
pub fn run_setup(dir: &std::path::Path) -> Result<Rebuild, String> {
    let script = dir.join("setup.sh");
    if !script.is_file() {
        return Ok(Rebuild::NotApplicable);
    }
    let out = std::process::Command::new("sh")
        .arg(&script)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("could not run {}: {e}", script.display()))?;
    if out.status.success() {
        return Ok(Rebuild::Installed);
    }
    Err(format!(
        "{} failed ({})\n{}{}",
        script.display(),
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}
