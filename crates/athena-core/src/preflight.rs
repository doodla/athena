//! The capability preflight (requirement OLY-B-RUN-01): before the producer starts, Athena lists the
//! programs a job's checks run, looks each up, records the result on the job, and turns any program that is
//! missing into a blocker with a workaround plan.
//!
//! What is a need. The first word of each check's command (`CheckSpec::run[0]`) in the capability's check
//! list, each program once with the names of the checks that run it. A bare name is looked up on PATH. A
//! name that holds a `/` is a path: it resolves against the check's working directory (a relative path may
//! name a file the producer creates) or a place Athena does not control, so it cannot be judged missing in
//! advance. It is listed as `not probed (path)`, by its last path component only, and never blocks.
//!
//! What is not a need, and why:
//! - `git`: Athena runs it itself before this check (reading the configuration, pinning, the prediction
//!   commit), so a missing git fails earlier and no record of the job exists yet to hold a blocker. A check
//!   that runs `git` is still a need, as a check program.
//! - `gh`: it never changes a job's outcome while the job runs (a failing `gh` is one warning), and landing
//!   is a later command.
//! - the agent binary: `ClaudeCodeDriver::new` runs `<bin> --version` and compares the pinned version when
//!   the driver is built, so a missing binary fails before any job is dispatched.
//!
//! How a program is found. [`SystemProbe`] searches Athena's own PATH and runs nothing, which is what
//! `Command::new` does. It has three limits. It reads Athena's own PATH, while a check runs with a cleared
//! environment rebuilt from `env_pass` and the capability's `env` templates; they agree when PATH is passed
//! and no template sets it, as in this repository. A relative or empty PATH entry resolves against the job
//! directory when a check runs, not against Athena's working directory, so such an entry is never searched
//! and a tool that only it could supply is reported `not probed (relative PATH)`, not missing. And "any
//! execute bit" is looser than the kernel's own check, so a file the user cannot run can still count as
//! present.
//!
//! The result is recorded in the job's `note` (one line per program, see [`Preflight::note`]) in the
//! prediction commit, before any agent runs. A blocker closes the job `failed` with an `environment:` reason
//! (see [`PREFLIGHT_BLOCKED`]) that names the program, the checks that need it and the workaround. A missing
//! bare-name check program used to land the job with a recorded pre-existing failure ("could not start");
//! it now stops the job, which is the intended stricter behavior.

use crate::capability::CheckSpec;
use crate::store::{JobState, Records};
use athena_data::JobId;
use std::ffi::OsString;
use std::path::Path;

/// The start of the closing reason of a job the preflight stopped. It starts with the environment prefix, so
/// the stop reads as the environment's and not as a failure of the work (`disk::StopCause::of`).
pub const PREFLIGHT_BLOCKED: &str = "environment: preflight blocked:";

/// Whether a closing reason is a preflight stop.
pub fn is_preflight_reason(reason: &str) -> bool {
    reason.starts_with(PREFLIGHT_BLOCKED)
}

/// What a probe found for one program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    Present,
    Missing,
    /// The program could not be judged, with the reason.
    NotProbed(&'static str),
}

/// Looks a program up. The production probe is [`SystemProbe`]; a test passes its own.
pub trait ToolProbe {
    fn probe(&self, tool: &str) -> Probe;
}

/// Looks a bare program name up on a PATH value. It runs nothing. See the module doc for its limits.
#[derive(Debug, Clone)]
pub struct SystemProbe {
    path: Option<OsString>,
}

impl SystemProbe {
    /// Athena's own PATH.
    pub fn from_env() -> SystemProbe {
        SystemProbe {
            path: std::env::var_os("PATH"),
        }
    }

    /// A given PATH value, for tests.
    pub fn with_path(path: impl Into<OsString>) -> SystemProbe {
        SystemProbe {
            path: Some(path.into()),
        }
    }
}

impl ToolProbe for SystemProbe {
    fn probe(&self, tool: &str) -> Probe {
        let Some(path) = &self.path else {
            return Probe::NotProbed("PATH is not set");
        };
        let mut relative = false;
        for entry in std::env::split_paths(path) {
            if !entry.is_absolute() {
                relative = true;
                continue;
            }
            if is_executable_file(&entry.join(tool)) {
                return Probe::Present;
            }
        }
        if relative {
            Probe::NotProbed("relative PATH")
        } else {
            Probe::Missing
        }
    }
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_file())
}

/// One program a job needs and the checks that run it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    /// The program as the check names it: a bare name, or a path.
    pub program: String,
    /// The names of the checks that run it, in check order.
    pub needed_by: Vec<String>,
}

impl Need {
    fn is_path(&self) -> bool {
        self.program.contains('/')
    }

    /// The name recorded: the bare name, or the last path component of a path (so no directory is stored).
    fn display(&self) -> &str {
        self.program
            .rsplit('/')
            .find(|part| !part.is_empty())
            .unwrap_or(&self.program)
    }
}

/// The programs a capability's checks run, each once.
pub fn needed_tools(checks: &[CheckSpec]) -> Vec<Need> {
    let mut needs: Vec<Need> = Vec::new();
    for check in checks {
        let Some(program) = check.run.first().filter(|p| !p.is_empty()) else {
            continue;
        };
        match needs.iter_mut().find(|n| &n.program == program) {
            Some(need) => {
                if !need.needed_by.contains(&check.name) {
                    need.needed_by.push(check.name.clone());
                }
            }
            None => needs.push(Need {
                program: program.clone(),
                needed_by: vec![check.name.clone()],
            }),
        }
    }
    needs
}

/// One program considered, and what was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    /// The recorded name (the last path component for a path).
    pub tool: String,
    pub needed_by: Vec<String>,
    pub found: Probe,
}

/// A missing program and the plan to get past it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    pub tool: String,
    pub needed_by: Vec<String>,
    /// Never empty.
    pub workaround: Vec<String>,
}

/// The preflight's result: every program considered, and a blocker for each one that is missing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Preflight {
    pub checked: Vec<Checked>,
    pub blockers: Vec<Blocker>,
}

/// Looks each need up through `probe`. A path is listed as not probed and never looked up.
pub fn preflight(needs: &[Need], probe: &dyn ToolProbe) -> Preflight {
    let mut result = Preflight::default();
    for need in needs {
        let found = if need.is_path() {
            Probe::NotProbed("path")
        } else {
            probe.probe(&need.program)
        };
        if found == Probe::Missing {
            result.blockers.push(Blocker {
                tool: need.display().to_string(),
                needed_by: need.needed_by.clone(),
                workaround: workaround(need.display(), &need.needed_by),
            });
        }
        result.checked.push(Checked {
            tool: need.display().to_string(),
            needed_by: need.needed_by.clone(),
            found,
        });
    }
    result
}

/// The production entry: the programs the checks run, looked up on Athena's own PATH.
pub fn check_tools(checks: &[CheckSpec]) -> Preflight {
    preflight(&needed_tools(checks), &SystemProbe::from_env())
}

fn workaround(tool: &str, needed_by: &[String]) -> Vec<String> {
    let install = if tool == "cargo" {
        "Install the Rust toolchain (rustup) so that `cargo` is on PATH for Athena and for its checks."
            .to_string()
    } else {
        format!("Install `{tool}` and put its directory on PATH for Athena and for its checks.")
    };
    vec![
        install,
        format!(
            "Or change or remove the check that needs it ({}) in athena.toml.",
            needed_by.join(", ")
        ),
        "Then run `athena retry` on this job again.".to_string(),
    ]
}

fn found_text(found: Probe) -> String {
    match found {
        Probe::Present => "present".to_string(),
        Probe::Missing => "missing".to_string(),
        Probe::NotProbed(why) => format!("not probed ({why})"),
    }
}

impl Preflight {
    /// One line per program considered: `<tool>: <present|missing|not probed (why)>; needed by <checks>`. With
    /// no program to probe, `no check programs to probe`. This is stored in the job record's `note`.
    pub fn note(&self) -> String {
        if self.checked.is_empty() {
            return "no check programs to probe".to_string();
        }
        self.checked
            .iter()
            .map(|c| {
                format!(
                    "{}: {}; needed by {}",
                    c.tool,
                    found_text(c.found),
                    c.needed_by.join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The closing reason when something is missing, `None` when nothing is. With `earlier`, it also says
    /// the kept work of that job was left alone and a retry of this job resumes it.
    pub fn stop_reason(&self, earlier: Option<&JobId>) -> Option<String> {
        if self.blockers.is_empty() {
            return None;
        }
        let blockers = self
            .blockers
            .iter()
            .map(|b| {
                format!(
                    "{} (needed by {}) is not on PATH; workaround: {}",
                    b.tool,
                    b.needed_by.join(", "),
                    b.workaround.join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        let mut reason = format!("{PREFLIGHT_BLOCKED} {blockers}");
        if let Some(earlier) = earlier {
            reason.push_str(&format!(
                "; the kept work of {earlier} was left alone; a retry of this job resumes it"
            ));
        }
        Some(reason)
    }
}

/// The job a retry of `id` should retry. A job the preflight stopped did no work, so when it was itself a
/// retry of another job, a retry of it retries that job: the stored request then reads `try again on` the job
/// whose work is meant, and every reader of that link (gate counts, evidence labels, the seeds of a resumed
/// retry) sees the real chain. For any other job it is `id`. It follows at most 16 links and stops at a cycle
/// or at a job it does not know.
pub fn retry_target(records: &Records, id: &JobId) -> JobId {
    let mut current = id.clone();
    let mut seen = vec![current.clone()];
    for _ in 0..16 {
        let Some(job) = records.jobs.get(current.as_str()) else {
            break;
        };
        let stopped_here = job.state()
            == JobState::Closed(athena_data::records::OutcomeState::Failed)
            && job
                .outcome
                .as_ref()
                .is_some_and(|o| is_preflight_reason(&o.reason));
        if !stopped_here {
            break;
        }
        let Some(request) = job.request.as_ref() else {
            break;
        };
        let Some(next) = crate::reading::retried_job(records, &request.text) else {
            break;
        };
        if seen.contains(&next) {
            break;
        }
        seen.push(next.clone());
        current = next;
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::config::CheckKind;

    /// Answers by name only: listed names are missing or relative, every other name is present.
    struct FakeProbe {
        missing: Vec<&'static str>,
        relative: Vec<&'static str>,
    }

    impl FakeProbe {
        fn all_present() -> FakeProbe {
            FakeProbe {
                missing: Vec::new(),
                relative: Vec::new(),
            }
        }
    }

    impl ToolProbe for FakeProbe {
        fn probe(&self, tool: &str) -> Probe {
            if self.missing.contains(&tool) {
                Probe::Missing
            } else if self.relative.contains(&tool) {
                Probe::NotProbed("relative PATH")
            } else {
                Probe::Present
            }
        }
    }

    fn spec(name: &str, run: &[&str]) -> CheckSpec {
        CheckSpec {
            name: name.to_string(),
            run: run.iter().map(|s| s.to_string()).collect(),
            kind: CheckKind::Command,
            skip_when_only: None,
        }
    }

    fn cargo_checks() -> Vec<CheckSpec> {
        vec![
            spec("fmt", &["cargo", "fmt", "--all", "--check"]),
            spec("clippy", &["cargo", "clippy", "--all-targets"]),
            spec("test", &["cargo", "test", "--locked"]),
        ]
    }

    #[test]
    fn capability_preflight_missing_tool_becomes_a_blocker_with_a_workaround_plan() {
        let needs = needed_tools(&[
            spec("fmt", &["cargo", "fmt"]),
            spec("test", &["cargo", "test"]),
        ]);
        let probe = FakeProbe {
            missing: vec!["cargo"],
            relative: Vec::new(),
        };
        let result = preflight(&needs, &probe);
        assert_eq!(result.blockers.len(), 1, "{result:?}");
        let blocker = &result.blockers[0];
        assert_eq!(blocker.tool, "cargo");
        assert_eq!(blocker.needed_by, vec!["fmt", "test"]);
        assert!(!blocker.workaround.is_empty());
        assert!(
            blocker
                .workaround
                .iter()
                .all(|step| !step.trim().is_empty())
        );
        assert!(
            result.note().contains("cargo: missing"),
            "{}",
            result.note()
        );
        let reason = result.stop_reason(None).expect("a blocker stops the job");
        assert!(
            reason.starts_with("environment: preflight blocked:"),
            "{reason}"
        );
        assert!(reason.contains("cargo"), "{reason}");
        assert!(reason.contains("workaround"), "{reason}");
        assert!(crate::disk::is_environment_reason(&reason));
        assert!(is_preflight_reason(&reason));
    }

    #[test]
    fn capability_preflight_all_tools_present_gives_no_blocker() {
        let mut checks = cargo_checks();
        checks.push(spec("docs", &["sh", "-c", "true"]));
        let result = preflight(&needed_tools(&checks), &FakeProbe::all_present());
        assert!(result.blockers.is_empty(), "{result:?}");
        assert_eq!(result.stop_reason(None), None);
        let tools: Vec<&str> = result.checked.iter().map(|c| c.tool.as_str()).collect();
        assert_eq!(tools, vec!["cargo", "sh"]);
        assert!(result.checked.iter().all(|c| c.found == Probe::Present));
        let note = result.note();
        assert!(
            note.contains("cargo: present; needed by fmt, clippy, test"),
            "{note}"
        );
        assert!(note.contains("sh: present; needed by docs"), "{note}");
    }

    #[test]
    fn capability_preflight_lists_path_programs_as_not_probed() {
        let checks = vec![
            spec("local", &["/some/home/dir/bin/mytool", "--flag"]),
            spec("rel", &["./script.sh"]),
            spec("empty", &[]),
            spec("blank", &[""]),
        ];
        let everything_missing = FakeProbe {
            missing: vec![
                "mytool",
                "script.sh",
                "/some/home/dir/bin/mytool",
                "./script.sh",
            ],
            relative: Vec::new(),
        };
        let result = preflight(&needed_tools(&checks), &everything_missing);
        assert!(result.blockers.is_empty(), "{result:?}");
        assert_eq!(result.checked.len(), 2, "an empty run is no need");
        assert!(
            result
                .checked
                .iter()
                .all(|c| c.found == Probe::NotProbed("path"))
        );
        let note = result.note();
        assert!(
            note.contains("mytool: not probed (path); needed by local"),
            "{note}"
        );
        assert!(
            note.contains("script.sh: not probed (path); needed by rel"),
            "{note}"
        );
        assert!(
            !note.contains("/some/home"),
            "no directory is recorded: {note}"
        );
        assert_eq!(
            preflight(&[], &everything_missing).note(),
            "no check programs to probe"
        );
    }

    #[test]
    fn capability_preflight_shared_tool_listed_once() {
        let needs = needed_tools(&cargo_checks());
        assert_eq!(needs.len(), 1);
        assert_eq!(needs[0].program, "cargo");
        assert_eq!(needs[0].needed_by, vec!["fmt", "clippy", "test"]);
        let again = needed_tools(&[spec("a", &["cargo"]), spec("a", &["cargo"])]);
        assert_eq!(again[0].needed_by, vec!["a"]);
    }

    #[test]
    fn capability_preflight_needs_come_from_the_capability_checks() {
        let programs: Vec<String> = needed_tools(&cargo_checks())
            .into_iter()
            .map(|n| n.program)
            .collect();
        assert_eq!(programs, vec!["cargo"]);
        let programs: Vec<String> = needed_tools(&[spec("log", &["git", "log"])])
            .into_iter()
            .map(|n| n.program)
            .collect();
        assert_eq!(programs, vec!["git"]);
        assert!(needed_tools(&[]).is_empty());
    }

    #[test]
    fn capability_preflight_git_alone_and_gh_are_not_needs() {
        // Checks that run neither git nor gh give no need for them: Athena's own use of git fails earlier,
        // and gh is not a need of the job.
        let needs = needed_tools(&cargo_checks());
        assert!(
            needs
                .iter()
                .all(|n| n.program != "git" && n.program != "gh")
        );
        assert!(needed_tools(&[]).is_empty());
        let result = preflight(&needs, &FakeProbe::all_present());
        assert!(
            result
                .checked
                .iter()
                .all(|c| c.tool != "git" && c.tool != "gh")
        );
    }

    #[test]
    fn capability_preflight_stop_reason_names_the_earlier_job() {
        let result = preflight(
            &needed_tools(&[spec("fmt", &["cargo", "fmt"])]),
            &FakeProbe {
                missing: vec!["cargo"],
                relative: Vec::new(),
            },
        );
        let earlier = JobId::parse("20261101-000000-abcd").unwrap();
        let reason = result.stop_reason(Some(&earlier)).expect("blocked");
        assert!(
            reason.contains("the kept work of 20261101-000000-abcd was left alone"),
            "{reason}"
        );
        assert!(
            reason.contains("a retry of this job resumes it"),
            "{reason}"
        );
        let plain = result.stop_reason(None).expect("blocked");
        assert!(!plain.contains("left alone"), "{plain}");
    }

    #[test]
    fn capability_preflight_relative_path_is_not_judged_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = std::env::join_paths([
            std::path::PathBuf::from("node_modules/.bin"),
            dir.path().to_path_buf(),
        ])
        .unwrap();
        let probe = SystemProbe::with_path(path);
        let result = preflight(
            &needed_tools(&[spec("lint", &["athena-preflight-absent-tool"])]),
            &probe,
        );
        assert!(result.blockers.is_empty(), "{result:?}");
        assert_eq!(result.checked[0].found, Probe::NotProbed("relative PATH"));
        assert!(
            result
                .note()
                .contains("athena-preflight-absent-tool: not probed (relative PATH)"),
            "{}",
            result.note()
        );
        // Without a relative entry the same tool is missing.
        let absolute_only = SystemProbe::with_path(dir.path().as_os_str());
        assert_eq!(
            absolute_only.probe("athena-preflight-absent-tool"),
            Probe::Missing
        );
    }

    #[cfg(unix)]
    #[test]
    fn capability_preflight_system_probe_finds_only_executable_files_on_path() {
        let dir = tempfile::tempdir().unwrap();
        let runnable = dir.path().join("runnable-tool");
        let plain = dir.path().join("plain-file");
        athena_testkit::write_executable(&runnable, "#!/bin/sh\n");
        // A file created with the default mode (0666 less the umask) never has an execute bit.
        std::fs::write(&plain, "text").unwrap();
        std::fs::create_dir(dir.path().join("a-directory")).unwrap();
        let probe = SystemProbe::with_path(dir.path().as_os_str());
        assert_eq!(probe.probe("runnable-tool"), Probe::Present);
        assert_eq!(probe.probe("plain-file"), Probe::Missing);
        assert_eq!(probe.probe("a-directory"), Probe::Missing);
        assert_eq!(probe.probe("nothing-here"), Probe::Missing);
    }

    #[test]
    fn capability_preflight_spec_entry_names_this_file_owner_and_test() {
        let requirements =
            crate::hestia::parse_spec(include_str!("../../../docs/spec.toml")).unwrap();
        let r = requirements
            .iter()
            .find(|r| r.id == "OLY-B-RUN-01")
            .expect("docs/spec.toml has OLY-B-RUN-01");
        assert!(r.active);
        assert_eq!(r.owner, "Athena");
        assert!(
            r.mechanism.contains("crates/athena-core/src/preflight.rs"),
            "{}",
            r.mechanism
        );
        assert!(
            r.measure.contains(
                "capability_preflight_missing_tool_becomes_a_blocker_with_a_workaround_plan"
            ),
            "{}",
            r.measure
        );
        assert!(r.measure.ends_with("status: not shown"), "{}", r.measure);
        assert_eq!(crate::hestia::measure_problem(&r.measure), None);
        let gaps = crate::hestia::spec_gaps(&requirements);
        assert!(gaps.iter().all(|g| g.id != "OLY-B-RUN-01"), "{gaps:?}");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(root.join("crates/athena-core/src/preflight.rs").is_file());
    }
}
