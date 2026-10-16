//! Pinning: the core resolves a study's repository ref to one commit, and mounts that tree read-only.
//!
//! A study reads a system "at one pinned commit". The core, not an agent, picks that commit: [`Pinner`]
//! turns the URL and ref from the capability's configuration into a full 40-hex commit id and checks the
//! tree out. [`mount`] then copies the tree into an agent's working directory as read-only files.
//!
//! Read-only is enforced by file permissions (every file and directory loses its write bits, so nothing in
//! the tree can be edited, created, or removed by a process that honors them) plus the agents' own policy;
//! it is not an operating-system mount. The mount directory is listed in the working directory's git
//! `info/exclude`, so it never appears in a candidate's change list.
//!
//! The same pinned tree backs the source check ([`crate::Athena::check_study_text`]): a model-backed read of a
//! study's page text against the source it was written from, used by `athena drain`. Every file of a study
//! directory is page text except those in [`STRUCTURAL_FILES`].

use anyhow::{Context, Result, anyhow, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The directory, inside an agent's working directory, that holds the pinned tree.
pub const MOUNT_DIR: &str = "pinned-source";

/// Resolves a repository ref to a commit and checks that commit's tree out.
pub trait Pinner: Send + Sync {
    /// Resolves `git_ref` of the repository at `url` to a full 40-hex commit id, and writes that commit's
    /// tree (without any `.git`) into `dest`, which must not exist yet. Returns the commit id.
    fn pin(&self, url: &str, git_ref: &str, dest: &Path) -> Result<String>;
}

/// The real pinner: `git ls-remote` to resolve, then a shallow fetch of that commit and a checkout.
#[derive(Debug, Default, Clone, Copy)]
pub struct GitPinner;

/// Whether `text` is a full lowercase 40-hex commit id.
pub fn is_full_sha(text: &str) -> bool {
    text.len() == 40 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn git(dir: Option<&Path>, args: &[&str]) -> Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    let output = command
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .context("running git")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Picks the commit `git_ref` names from `git ls-remote` output (annotated tags use their peeled commit).
fn pick(listing: &str, git_ref: &str) -> Option<String> {
    let wanted = [
        git_ref.to_string(),
        format!("refs/heads/{git_ref}"),
        format!("refs/tags/{git_ref}^{{}}"),
        format!("refs/tags/{git_ref}"),
    ];
    let lines: Vec<(&str, &str)> = listing.lines().filter_map(|l| l.split_once('\t')).collect();
    wanted.iter().find_map(|name| {
        lines
            .iter()
            .find(|(sha, n)| n == name && is_full_sha(sha))
            .map(|(sha, _)| sha.to_string())
    })
}

impl Pinner for GitPinner {
    fn pin(&self, url: &str, git_ref: &str, dest: &Path) -> Result<String> {
        for (what, text) in [("repository URL", url), ("ref", git_ref)] {
            if text.trim().is_empty() || text.starts_with('-') {
                bail!("the {what} {text:?} is not usable");
            }
        }
        if dest.exists() {
            bail!("pin destination {} already exists", dest.display());
        }
        let sha = if is_full_sha(git_ref) {
            git_ref.to_string()
        } else {
            let listing = git(None, &["ls-remote", url, git_ref])?;
            pick(&listing, git_ref)
                .ok_or_else(|| anyhow!("the ref {git_ref:?} was not found in {url}"))?
        };
        let scratch = tempfile::tempdir().context("creating a scratch repository")?;
        let repo = scratch.path();
        git(Some(repo), &["init", "-q"])?;
        git(
            Some(repo),
            &["fetch", "-q", "--depth", "1", "--no-tags", url, &sha],
        )
        .with_context(|| format!("fetching {sha} from {url}"))?;
        // An annotated tag resolves to its own object; the pin is the commit it points at.
        let fetched = git(Some(repo), &["rev-parse", "FETCH_HEAD^{commit}"])?
            .trim()
            .to_string();
        if !is_full_sha(&fetched) || (is_full_sha(git_ref) && fetched != sha) {
            bail!("fetched {fetched:?} where {git_ref} was expected");
        }
        let sha = fetched;
        std::fs::create_dir_all(dest)?;
        let prefix = format!("--prefix={}/", dest.display());
        git(Some(repo), &["read-tree", "FETCH_HEAD"])
            .and_then(|_| git(Some(repo), &["checkout-index", "-a", &prefix]))
            .with_context(|| format!("checking out {sha}"))?;
        Ok(sha)
    }
}

/// Copies the pinned tree at `src` into `workdir/MOUNT_DIR`, hides it from the working directory's git,
/// and makes it read-only. Returns the mount path.
pub fn mount(src: &Path, workdir: &Path) -> Result<PathBuf> {
    let dest = workdir.join(MOUNT_DIR);
    if dest.exists() {
        bail!("{} already exists", dest.display());
    }
    copy_tree(src, &dest)?;
    let info = workdir.join(".git").join("info");
    if info.is_dir() {
        let exclude = info.join("exclude");
        let mut text = std::fs::read_to_string(&exclude).unwrap_or_default();
        text.push_str(&format!("/{MOUNT_DIR}/\n"));
        std::fs::write(&exclude, text)?;
    }
    make_read_only(&dest)?;
    Ok(dest)
}

fn copy_tree(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&from, &to)?;
        } else if kind.is_symlink() {
            copy_symlink(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(from: &Path, to: &Path) -> Result<()> {
    std::os::unix::fs::symlink(std::fs::read_link(from)?, to)?;
    Ok(())
}

#[cfg(not(unix))]
fn copy_symlink(_from: &Path, _to: &Path) -> Result<()> {
    Ok(())
}

/// Removes the write bits of every file and directory under `path`, and of `path` itself.
pub fn make_read_only(path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    if meta.is_dir() {
        for entry in std::fs::read_dir(path)? {
            make_read_only(&entry?.path())?;
        }
    }
    set_writable(path, false)
}

#[cfg(unix)]
fn set_writable(path: &Path, writable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)?.permissions();
    let mode = permissions.mode();
    permissions.set_mode(if writable {
        mode | 0o200
    } else {
        mode & !0o222
    });
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_writable(path: &Path, writable: bool) -> Result<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(!writable);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn make_writable(path: &Path) {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    let _ = set_writable(path, true);
    if meta.is_dir()
        && let Ok(entries) = std::fs::read_dir(path)
    {
        for entry in entries.flatten() {
            make_writable(&entry.path());
        }
    }
}

/// Removes a directory tree that may hold read-only mounts. Missing paths are fine.
pub fn remove_tree(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    // A locked verifier snapshot is immutable even for root until unlocked.
    let _ = athena_git::snapshot::unlock_dir(path);
    make_writable(path);
    std::fs::remove_dir_all(path)
}

// ---------------------------------------------------------------------------------------------------------
// The source check: the text of a study's page against the pinned source

/// The directory, inside the checker's workspace, that holds a study's page text.
pub const PAGE_TEXT_DIR: &str = "page-text";

/// The files of a study directory that say nothing about the source, so the source check does not read them:
/// `pin.json` is the pin itself (the reference the check is made against) and `validation.json` holds rule
/// names with pass flags. Every other file in a study directory is page text and is checked, so a file added
/// later is covered until someone makes the case here that it is not.
pub const STRUCTURAL_FILES: [&str; 2] = ["pin.json", "validation.json"];

/// The most page text one check reads, in bytes. A study over it fails the check; the text is never cut.
pub const MAX_STUDY_TEXT_BYTES: usize = 400_000;

/// Whether the source check reads `file` (a path relative to the study directory).
pub fn is_page_text(file: &str) -> bool {
    !STRUCTURAL_FILES.contains(&file)
}

/// The checker's stored prompt.
pub const SOURCE_CHECK_PROMPT: &str = "You check the text of an explainer page against the source it explains. \
You only read and report. A statement is supported only if the source says or shows it. You report what the \
source contradicts and the facts the text leaves out; you do not judge style, and you report nothing you \
cannot support from a path and lines of the source. Quotes and single statements are not enough. The page \
also carries a model (`model.json`: states, transitions, counters, rules and their waivers). Trace every \
state, transition, counter and rule of that model through the pinned code. A model that behaves \
differently from the code is a behavior error: a transition that ends in a different state than the code \
leaves it in (a reopened item that the code returns to its queue, shown as finished), a counter that misses \
an event that changes it (a failed close that already freed the place), or a rule or waiver the code does not \
implement as written. Also compare the rules and statements of the page with each other: a rule that \
contradicts a waiver or another statement on the same page is an inconsistency, even if each is true alone.";

/// One file of page text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFile {
    /// The path relative to the study directory.
    pub name: String,
    pub text: String,
}

/// One study's page text and the pin it was written against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyText {
    pub name: String,
    pub repository_url: String,
    pub sha: String,
    pub files: Vec<TextFile>,
}

/// What a source check found. No problems means the text passed; a check that could not be made is a problem.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourceReport {
    pub problems: Vec<String>,
    /// What the model call cost, even when the check failed.
    pub cost_usd: f64,
}

/// The checker's report: the files it read and what it found.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckerOutput {
    files_checked: Vec<String>,
    problems: Vec<CheckerProblem>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckerProblem {
    file: String,
    kind: String,
    statement: String,
    evidence: String,
}

fn checker_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["files_checked", "problems"],
        "properties": {
            "files_checked": { "type": "array", "items": { "type": "string" } },
            "problems": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["file", "kind", "statement", "evidence"],
                    "properties": {
                        "file": { "type": "string" },
                        "kind": { "type": "string", "enum": ["contradicted", "omitted", "behavior"] },
                        "statement": { "type": "string" },
                        "evidence": { "type": "string" }
                    }
                }
            }
        }
    })
}

/// The checker's user prompt.
pub fn source_check_brief(study: &StudyText) -> String {
    let files: Vec<String> = study
        .files
        .iter()
        .map(|f| format!("- `{PAGE_TEXT_DIR}/{}`", f.name))
        .collect();
    format!(
        "# Source check: {name}\n\nThe source is `{url}` at commit {sha}, mounted read-only in `{MOUNT_DIR}/`. \
         The text of the page is in `{PAGE_TEXT_DIR}/`, one file per input:\n{files}\n\n\
         Read every one of those files in full with Read, and search the source with Grep and Glob. Nothing \
         may be changed. For each statement that says what the source does, decide whether the source \
         supports it.\n\n## Your report\n\n`problems` lists each of:\n\
         - kind `contradicted`: a statement the source contradicts; `evidence` says what the source says \
         instead, with its path and lines;\n\
         - kind `omitted`: a fact the source states that the text needs to be true and leaves out; \
         `statement` names the fact and `evidence` cites the source;\n\
         - kind `behavior`: the model, or the page against itself, is wrong about behavior. Trace each state, \
         each transition (where it ends, including a reopen or a failed step), each counter (every event that \
         raises or lowers it, including a failed close that frees the place) and each rule with its waivers \
         from `{PAGE_TEXT_DIR}/model.json` through the source. Report a model that behaves differently from the \
         code, and a rule that contradicts a waiver or another statement on the page. `statement` quotes the \
         model entry or the conflicting statements, and `evidence` says what the code does, with path and lines.\n\
         If the page has a `model.json`, you must read it and list it in `files_checked`.\n\
         Every problem names its `file` (as listed above) and quotes the `statement`. `files_checked` lists \
         every file you read in full, by the names above. An empty `problems` means you found none.\n",
        name = study.name,
        url = study.repository_url,
        sha = study.sha,
        files = files.join("\n"),
    )
}

/// Removes a check's directory in the state directory when it ends, however it ends. The mount is read-only,
/// so it is made writable first.
struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = remove_tree(&self.0);
    }
}

impl crate::Athena {
    /// Checks one study's page text against its pinned source with one model call, on the verifier's model and
    /// cap. The core pins the source; the checker reads it from a read-only mount and may not change anything.
    /// The check passes only when the model reports no problem and says it read every file. A pin that does not
    /// resolve to the study's sha, an unusable report, or a text over [`MAX_STUDY_TEXT_BYTES`] fails it.
    /// Nothing is committed: the call is no job, and its cost is returned for the caller to count.
    pub fn check_study_text(&self, study: &StudyText) -> SourceReport {
        let mut report = SourceReport::default();
        match self.check_study_text_inner(study, &mut report.cost_usd) {
            Ok(problems) => report.problems = problems,
            Err(error) => report.problems.push(format!(
                "{}: the text could not be checked against the source: {error:#}",
                study.name
            )),
        }
        report
    }

    fn check_study_text_inner(&self, study: &StudyText, cost: &mut f64) -> Result<Vec<String>> {
        use crate::capability::{JobConfig, ToolGrant};
        use athena_agent::env::Side;
        use athena_context::{Component, compose};
        use athena_data::records::{InvocationOutcome, Role};
        use athena_data::{JobId, RecordId};
        use std::collections::BTreeMap;

        if self.nested.is_some() {
            bail!("a nested run makes no model call");
        }
        self.refuse_if_stopped()?;
        if study.files.is_empty() {
            bail!("the study has no page text to check");
        }
        let bytes: usize = study.files.iter().map(|f| f.text.len()).sum();
        if bytes > MAX_STUDY_TEXT_BYTES {
            bail!(
                "the page text is {bytes} bytes, over the {MAX_STUDY_TEXT_BYTES} the check reads"
            );
        }
        let head = self.head()?;
        let jc = JobConfig::at(&self.git, &head)?;
        let job = JobId::Id(RecordId::generate(crate::now()));
        let job_dir = self.state.job_dir(&job);
        let _cleanup = Cleanup(job_dir.clone());
        let pinned = job_dir.join("pin");
        remove_tree(&pinned)?;
        let sha = self
            .pinner
            .pin(&study.repository_url, &study.sha, &pinned)
            .with_context(|| format!("pinning {} of {}", study.sha, study.repository_url))?;
        if sha != study.sha {
            bail!("the pin resolved to {sha}, not the study's {}", study.sha);
        }
        let workspace = job_dir.join("workspace");
        std::fs::create_dir_all(&workspace)?;
        for file in &study.files {
            let path = workspace.join(PAGE_TEXT_DIR).join(&file.name);
            std::fs::create_dir_all(path.parent().expect("a file has a parent"))?;
            std::fs::write(path, &file.text)?;
        }
        mount(&pinned, &workspace)?;

        let composed = compose(&[
            athena_context::CONTRACT.into(),
            Component::new("source-check.prompt", 2, SOURCE_CHECK_PROMPT),
        ]);
        let grant = ToolGrant {
            tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
            commands: BTreeMap::new(),
            writes: false,
            permission_mode: "dontAsk".into(),
        };
        // The checker is a verifier: it borrows the verifier's model, cap and timeout.
        let spec = self.agent_spec(
            &jc,
            &job,
            1,
            Role::Verifier,
            &jc.config.models.verifier,
            &composed,
            source_check_brief(study),
            &grant,
            &workspace,
            Side::Read,
            &BTreeMap::new(),
            jc.config.budget.verifier_usd,
            jc.config.budget.verifier_timeout_s,
            checker_schema(),
        )?;
        let facts = crate::invocation::InvocationFacts {
            job: &job,
            spec: &spec,
            composed: &composed,
            config: &jc.config,
            fake: self.is_fake(),
        };
        let redactor = self.redactor(&jc.config, Some(&job))?;
        let (output, invocation) = match self.invoke_and_record(&facts, &redactor)? {
            Ok(done) => done,
            Err(error) => bail!("the checker could not run: {error}"),
        };
        *cost = invocation.cost.best_lower();
        if output.outcome != InvocationOutcome::Success {
            bail!(
                "the checker did not complete: {}",
                invocation.detail.as_deref().unwrap_or("no detail")
            );
        }
        let found: CheckerOutput =
            serde_json::from_value(output.structured.clone().unwrap_or_default())
                .context("the checker returned no valid report")?;
        let mut problems: Vec<String> = found
            .problems
            .iter()
            .map(|p| {
                format!(
                    "{}/{}: {} statement \"{}\" ({})",
                    study.name, p.file, p.kind, p.statement, p.evidence
                )
            })
            .collect();
        for file in &study.files {
            if !found.files_checked.contains(&file.name) {
                problems.push(format!(
                    "{}/{}: the checker did not report reading this file",
                    study.name, file.name
                ));
            }
        }
        Ok(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_pin_and_the_rule_results_are_exempt_from_the_source_check() {
        assert!(!is_page_text("pin.json") && !is_page_text("validation.json"));
        for file in [
            "claims.json",
            "intro.json",
            "model.json",
            "rationales.json",
            "findings.json",
            "notes.md",
            "extra/more.json",
        ] {
            assert!(is_page_text(file), "{file} is checked");
        }
    }

    #[test]
    fn the_brief_names_every_file_and_the_pin() {
        let study = StudyText {
            name: "demo".into(),
            repository_url: "https://example.com/r.git".into(),
            sha: "a".repeat(40),
            files: vec![
                TextFile {
                    name: "claims.json".into(),
                    text: "[]".into(),
                },
                TextFile {
                    name: "intro.json".into(),
                    text: "{}".into(),
                },
            ],
        };
        let brief = source_check_brief(&study);
        for needle in [
            "page-text/claims.json",
            "page-text/intro.json",
            "pinned-source",
            &"a".repeat(40),
        ] {
            assert!(brief.contains(needle), "{needle}: {brief}");
        }
    }

    fn run(dir: &Path, args: &[&str]) -> String {
        git(Some(dir), args).unwrap()
    }

    /// A local repository with two commits on `main` and an annotated tag on the first; returns both ids.
    fn origin() -> (tempfile::TempDir, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        run(p, &["init", "-q", "-b", "main"]);
        for (k, v) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("tag.gpgsign", "false"),
        ] {
            run(p, &["config", k, v]);
        }
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::write(p.join("src/lib.rs"), "// one\n").unwrap();
        run(p, &["add", "-A"]);
        run(p, &["commit", "-q", "-m", "one"]);
        let first = run(p, &["rev-parse", "HEAD"]).trim().to_string();
        run(p, &["tag", "-a", "-m", "release", "v1"]);
        std::fs::write(p.join("src/lib.rs"), "// two\n").unwrap();
        run(p, &["commit", "-q", "-am", "two"]);
        let second = run(p, &["rev-parse", "HEAD"]).trim().to_string();
        (dir, first, second)
    }

    fn url(dir: &tempfile::TempDir) -> String {
        format!("file://{}", dir.path().display())
    }

    #[test]
    fn a_branch_a_tag_and_a_commit_resolve_to_their_commits() {
        let (dir, first, second) = origin();
        let out = tempfile::tempdir().unwrap();
        for (git_ref, expected, text) in [
            ("main", &second, "// two\n"),
            ("v1", &first, "// one\n"),
            (first.as_str(), &first, "// one\n"),
        ] {
            let dest = out.path().join(format!("t-{}", &expected[..8]));
            let _ = remove_tree(&dest);
            let sha = GitPinner.pin(&url(&dir), git_ref, &dest).unwrap();
            assert_eq!(&sha, expected, "ref {git_ref}");
            assert!(is_full_sha(&sha));
            assert_eq!(
                std::fs::read_to_string(dest.join("src/lib.rs")).unwrap(),
                text
            );
            assert!(!dest.join(".git").exists(), "the tree carries no history");
        }
    }

    #[test]
    fn a_missing_ref_is_an_error() {
        let (dir, _, _) = origin();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("tree");
        let error = GitPinner
            .pin(&url(&dir), "no-such-branch", &dest)
            .unwrap_err();
        assert!(error.to_string().contains("not found"), "{error:#}");
        assert!(!dest.exists());
        assert!(GitPinner.pin(&url(&dir), "--upload-pack=x", &dest).is_err());
    }

    #[test]
    fn the_mount_is_read_only_and_invisible_to_git() {
        let (dir, _, _) = origin();
        let out = tempfile::tempdir().unwrap();
        let src = out.path().join("pin");
        GitPinner.pin(&url(&dir), "main", &src).unwrap();
        let work = out.path().join("work");
        std::fs::create_dir_all(work.join(".git/info")).unwrap();
        let mounted = mount(&src, &work).unwrap();
        assert_eq!(mounted, work.join(MOUNT_DIR));
        assert_eq!(
            std::fs::read_to_string(mounted.join("src/lib.rs")).unwrap(),
            "// two\n"
        );
        for path in [
            mounted.clone(),
            mounted.join("src"),
            mounted.join("src/lib.rs"),
        ] {
            assert!(
                std::fs::metadata(&path).unwrap().permissions().readonly(),
                "{} is read-only",
                path.display()
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in [mounted.join("src"), mounted.join("src/lib.rs")] {
                let mode = std::fs::metadata(&path).unwrap().permissions().mode();
                assert_eq!(mode & 0o222, 0, "{} has no write bit", path.display());
            }
        }
        let exclude = std::fs::read_to_string(work.join(".git/info/exclude")).unwrap();
        assert!(exclude.contains(&format!("/{MOUNT_DIR}/")));
        remove_tree(&work).unwrap();
        assert!(!work.exists(), "a read-only mount can still be cleaned up");
    }
}
