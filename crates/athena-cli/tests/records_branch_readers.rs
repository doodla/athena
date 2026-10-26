//! Readers of `records/` and the `athena/records` worktree (decision D111, job 7 part 2).
//!
//! Every reader that `athena-cli` can reach prefers the records worktree when the repository has one and
//! reads the in-tree `records/` (or the commit it was given) when it has none. Each reader here has a
//! `worktree_<reader>` test and a `fallback_<reader>` test. The repository holds job `T` in its tree. In the
//! worktree tests the worktree holds only job `W`, so the job ids a reader returns say which records it read.
//!
//! The worktree lies under a temporary state directory: the tests point the state root there, for the
//! calling thread (`athena_data::paths::with_state_root`) or, for the built binary, with `ATHENA_STATE_DIR`.
//!
//! Two readers are tested elsewhere, because their fixtures are private to those crates:
//! - the wall-time reader of `xtask`: `worktree_walltime`, `fallback_walltime` and `linked_worktree_walltime`
//!   in `xtask/src/walltime.rs`;
//! - `land_pr`: `worktree_land_pr` and `fallback_land_pr` in `crates/athena-core/src/gh.rs`.

mod common;

use athena_core::Quiet;
use athena_core::introspect::{read_records_current, records_root};
use athena_core::store::Records;
use athena_data::paths;
use athena_gate::records::RecordSet;
use athena_git::Git;
use common::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn queue_one(fx: &Fixture, text: &str) -> String {
    let athena = fx.athena(Box::new(|_| unreachable!("queueing calls no model")));
    athena.queue(text).unwrap().0.as_str().to_string()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn month_of(id: &str) -> String {
    format!("{}-{}", &id[0..4], &id[4..6])
}

/// A repository holding job `t` in its tree and, with a worktree, a records worktree holding only job `w`.
struct World {
    fx: Fixture,
    t: String,
    w: String,
    /// Where the records worktree is, when there is one.
    worktree: Option<PathBuf>,
}

impl World {
    fn new(with_worktree: bool) -> World {
        let fx = Fixture::new();
        let t = queue_one(&fx, "the request that is only in the tree");
        let (other, w) = loop {
            let other = Fixture::new();
            let w = queue_one(&other, "the request that is only in the worktree");
            if w != t {
                break (other, w);
            }
        };
        World::from_parts(fx, t, &other, w, with_worktree)
    }

    /// `fx` holds job `t` in its tree; the worktree (when asked for) holds the records of `other`, whose job
    /// is `w`.
    fn from_parts(
        fx: Fixture,
        t: String,
        other: &Fixture,
        w: String,
        with_worktree: bool,
    ) -> World {
        let mut world = World {
            fx,
            t,
            w,
            worktree: None,
        };
        if with_worktree {
            let dir = world.in_state(|| {
                paths::records_worktree_dir(&paths::repo_state_dir(
                    &world.fx.git.common_dir().unwrap(),
                ))
            });
            // The worktree is under the temporary state directory, never the real one.
            assert!(dir.starts_with(world.fx.state.path()), "{dir:?}");
            world
                .fx
                .git
                .run([
                    "worktree",
                    "add",
                    "-q",
                    "-b",
                    "athena/records",
                    dir.to_str().unwrap(),
                ])
                .unwrap();
            let wt = Git::at(&dir);
            wt.run(["rm", "-r", "-q", "records"]).unwrap();
            copy_dir(&other.root().join("records"), &dir.join("records"));
            wt.run(["add", "-A"]).unwrap();
            wt.run(["commit", "-q", "-m", "records of the other job"])
                .unwrap();
            world.worktree = Some(dir);
        }
        world
    }

    /// Runs `f` with the state root under this world's temporary state directory.
    fn in_state<R>(&self, f: impl FnOnce() -> R) -> R {
        paths::with_state_root(self.fx.state.path(), f)
    }

    /// The id the reader should return: the worktree's when there is one, else the tree's.
    fn expected(&self) -> BTreeSet<String> {
        BTreeSet::from([if self.worktree.is_some() {
            self.w.clone()
        } else {
            self.t.clone()
        }])
    }

    fn head(&self) -> String {
        self.fx.head()
    }

    /// The first commit, before any record: a head unlike the tip of the worktree and the one with records.
    fn first_commit(&self) -> String {
        self.fx
            .git
            .run(["rev-list", "--max-parents=0", "HEAD"])
            .unwrap()
            .trim()
            .to_string()
    }

    /// An outcome file for the job under `root`, as `read_records` reads it.
    fn plant_outcome(root: &Path, id: &str) {
        let dir = root.join("records/jobs").join(month_of(id)).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("outcome.json"),
            format!("{{\"job\": \"{id}\", \"state\": \"landed\"}}"),
        )
        .unwrap();
    }

    /// Outcome files for `t` in the tree and, with a worktree, for `w` in the worktree.
    fn plant_outcomes(&self) {
        World::plant_outcome(self.fx.root(), &self.t);
        if let Some(dir) = &self.worktree {
            World::plant_outcome(dir, &self.w);
        }
    }
}

fn job_ids(records: &Records) -> BTreeSet<String> {
    records.jobs.keys().cloned().collect()
}

/// Generates `worktree_<name>` and `fallback_<name>`: the observation is the set of job ids the reader
/// returns. With the worktree it is `{W}`; without, `{T}`.
macro_rules! reader {
    ($name:ident, $worktree:ident, $fallback:ident, $observe:expr) => {
        #[test]
        fn $worktree() {
            let world = World::new(true);
            let seen: BTreeSet<String> = world.in_state(|| $observe(&world));
            assert_eq!(
                seen,
                world.expected(),
                "the worktree's job W, not the tree's T"
            );
            assert!(seen.contains(&world.w) && !seen.contains(&world.t));
        }

        #[test]
        fn $fallback() {
            let world = World::new(false);
            let seen: BTreeSet<String> = world.in_state(|| $observe(&world));
            assert_eq!(seen, world.expected(), "the tree's job T");
            assert!(seen.contains(&world.t) && !seen.contains(&world.w));
        }
    };
}

/// The ids among `w` and `t` that `try_job` accepts, for readers that resolve a job reference.
fn accepted(world: &World, mut try_job: impl FnMut(&str) -> bool) -> BTreeSet<String> {
    [&world.w, &world.t]
        .into_iter()
        .filter(|id| try_job(id))
        .cloned()
        .collect()
}

// Records::load_current
reader!(
    records_load_current,
    worktree_records_load_current,
    fallback_records_load_current,
    |world: &World| job_ids(&Records::load_current(&world.fx.git, &world.head()).unwrap())
);

// RecordSet::load_current
reader!(
    recordset_load_current,
    worktree_recordset_load_current,
    fallback_recordset_load_current,
    |world: &World| {
        RecordSet::load_current(&world.fx.git, &world.head())
            .unwrap()
            .files
            .iter()
            .filter_map(|f| f.path.strip_prefix("records/jobs/"))
            .filter_map(|rest| rest.split('/').nth(1).map(str::to_string))
            .collect()
    }
);

// introspect::read_records_current: the outcome walk over `records/jobs`
reader!(
    read_records,
    worktree_read_records,
    fallback_read_records,
    |world: &World| {
        world.plant_outcomes();
        read_records_current(world.fx.root())
            .unwrap()
            .jobs
            .into_iter()
            .collect()
    }
);

// introspect::records_root: the directory every walk of `records/` starts from
reader!(
    introspect_walks,
    worktree_introspect_walks,
    fallback_introspect_walks,
    |world: &World| {
        let root = records_root(world.fx.root());
        std::fs::read_dir(root.join("records/jobs"))
            .unwrap()
            .flat_map(|month| std::fs::read_dir(month.unwrap().path()).unwrap())
            .map(|job| job.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }
);

// Scoring and the closing commit read `Records::load_current` at the head they are building on.
reader!(
    scoring,
    worktree_scoring,
    fallback_scoring,
    |world: &World| job_ids(&Records::load_current(&world.fx.git, &world.head()).unwrap())
);

// reading.rs: a request is given a priority only when its job is found among the current records.
reader!(
    forecasting,
    worktree_forecasting,
    fallback_forecasting,
    |world: &World| {
        let athena = world.fx.athena(Box::new(|_| unreachable!("no model")));
        accepted(world, |id| athena.set_priority(id, "high").is_ok())
    }
);

// reading.rs: the records a retry carries from are those of `job_records`.
reader!(
    retry_carry,
    worktree_retry_carry,
    fallback_retry_carry,
    |world: &World| {
        let athena = world.fx.athena(Box::new(|_| unreachable!("no model")));
        accepted(world, |id| {
            athena_core::reading::job_records(&athena, id).is_ok()
        })
    }
);

// auditor.rs: the audit resolves its job among the current records. A job without an outcome gets as far as
// "nothing to audit"; an unknown one is "no job matches".
reader!(
    auditor,
    worktree_auditor,
    fallback_auditor,
    |world: &World| {
        let athena = world.fx.athena(Box::new(|_| unreachable!("no model")));
        accepted(world, |id| match athena.audit(id) {
            Err(e) => !e.to_string().contains("no job matches"),
            Ok(_) => true,
        })
    }
);

// proposals.rs: `propose` reads the current records (`read_records_current` and `load_current`); it commits
// nothing when no class repeats, so the observation is the walk it makes plus its success.
reader!(
    proposals,
    worktree_proposals,
    fallback_proposals,
    |world: &World| {
        world.plant_outcomes();
        let athena = world.fx.athena(Box::new(|_| unreachable!("no model")));
        let generated = athena.propose(&Quiet);
        assert!(generated.is_ok(), "{generated:?}");
        read_records_current(athena.git.dir())
            .unwrap()
            .jobs
            .into_iter()
            .collect()
    }
);

// backtest.rs: `backtest::run` reads the records through the re-exported `read_records_current`.
reader!(
    backtest,
    worktree_backtest,
    fallback_backtest,
    |world: &World| {
        world.plant_outcomes();
        athena_cli::backtest::read_records_current(world.fx.root())
            .unwrap()
            .jobs
            .into_iter()
            .collect()
    }
);

/// Scoring uses the worktree whatever head it is given (decisions.md, D111): a head from before any record
/// still gives the worktree's job, and not the tree's.
#[test]
fn scoring_ignores_passed_head() {
    let world = World::new(true);
    let unlike = world.first_commit();
    assert_ne!(unlike, world.head());
    let seen = world.in_state(|| job_ids(&Records::load_current(&world.fx.git, &unlike).unwrap()));
    assert_eq!(seen, world.expected());
}

/// Without a worktree the passed head is honoured exactly as `Records::load` reads it.
#[test]
fn fallback_scoring_honours_passed_head() {
    let world = World::new(false);
    let unlike = world.first_commit();
    let (current, loaded) = world.in_state(|| {
        (
            job_ids(&Records::load_current(&world.fx.git, &unlike).unwrap()),
            job_ids(&Records::load(&world.fx.git, &unlike).unwrap()),
        )
    });
    assert_eq!(current, loaded);
    assert!(current.is_empty(), "no record exists at the first commit");
}

/// The fallback equals the pre-change result: `load_current` is `load` when there is no worktree.
#[test]
fn fallback_equals_the_unchanged_loaders() {
    let world = World::new(false);
    world.in_state(|| {
        let head = world.head();
        let now = Records::load_current(&world.fx.git, &head).unwrap();
        let before = Records::load(&world.fx.git, &head).unwrap();
        assert_eq!(job_ids(&now), job_ids(&before));
        let set_now: Vec<_> = RecordSet::load_current(&world.fx.git, &head)
            .unwrap()
            .files
            .into_iter()
            .map(|f| (f.path, f.bytes))
            .collect();
        let set_before: Vec<_> = RecordSet::load(&world.fx.git, &head)
            .unwrap()
            .files
            .into_iter()
            .map(|f| (f.path, f.bytes))
            .collect();
        assert_eq!(set_now, set_before);
    });
}

/// A linked checkout of the repository finds the worktree through git's common directory.
#[test]
fn linked_worktree_backtest() {
    let world = World::new(true);
    let linked = world.fx.state.path().join("linked-checkout");
    world
        .fx
        .git
        .run([
            "worktree",
            "add",
            "-q",
            "-b",
            "linked",
            linked.to_str().unwrap(),
        ])
        .unwrap();
    assert!(
        linked.join(".git").is_file(),
        ".git is a file in a linked checkout"
    );
    world.in_state(|| {
        assert_eq!(records_root(&linked), world.worktree.clone().unwrap());
        World::plant_outcome(world.worktree.as_ref().unwrap(), &world.w);
        let jobs: BTreeSet<String> = athena_cli::backtest::read_records_current(&linked)
            .unwrap()
            .jobs
            .into_iter()
            .collect();
        assert_eq!(jobs, world.expected());
    });
}

fn athena_bin(world: &World, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_athena"))
        .current_dir(world.fx.root())
        .env("ATHENA_STATE_DIR", world.fx.state.path())
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR")
        .args(args)
        .output()
        .unwrap()
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A stand-in `claude` program that answers every invocation with `structured` as its result, so the built
/// binary can run an agent role offline. Returns an install directory (`ATHENA_HOME`) whose config names it.
fn stand_in_home(
    world: &World,
    name: &str,
    structured: &serde_json::Value,
    model_of: impl Fn(&athena_data::config::Models) -> String,
) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let config = athena_gate::config_at(&world.fx.git, Some(&world.fx.head())).unwrap();
    let version = config.agents.claude_version.clone();
    let model = model_of(&config.models);
    let home = world.fx.state.path().join(format!("home-{name}"));
    std::fs::create_dir_all(&home).unwrap();
    let result = serde_json::json!({
        "type": "result", "subtype": "success", "is_error": false, "num_turns": 1,
        "duration_api_ms": 10, "total_cost_usd": 0.001,
        "usage": { "input_tokens": 5, "cache_creation_input_tokens": 1, "cache_read_input_tokens": 1, "output_tokens": 5 },
        "modelUsage": { model: { "inputTokens": 5, "outputTokens": 5, "cacheReadInputTokens": 1, "cacheCreationInputTokens": 1, "costUSD": 0.001 } },
        "permission_denials": [],
        "structured_output": structured,
    });
    std::fs::write(home.join("result.json"), format!("{result}\n")).unwrap();
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo \"{version} (Claude Code)\"; exit 0; fi\n\
         echo '{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"0c32611b-5c0a-43eb-bacb-20a90569b268\"}}'\n\
         while IFS= read -r line; do printf '%s\\n' \"$line\"; done < \"${{0%/*}}/result.json\"\n"
    );
    let bin = home.join("claude");
    std::fs::write(&bin, script).unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(
        home.join("config.toml"),
        format!("claude_bin = {:?}\n", bin.display().to_string()),
    )
    .unwrap();
    home
}

/// The built `athena` on a scratch checkout whose working tree has no `records/`. The tree's records hold a
/// queued job `T`; the worktree holds a landed job `W`. `status`, `audit W` and `ask` (a forecast, read by a
/// stand-in model) all succeed, and the first two name `W` and never `T`; `audit T` finds no such job.
#[test]
fn scratch_checkout() {
    use athena_core::run::GoResult;
    use athena_data::records::{AcceptedBy, OutcomeState};

    const BRIEF: (&str, &str) = ("BRIEF.md", "# The brief\n\nKeep it simple.\n");
    let fx = Fixture::new();
    write(fx.root(), &[BRIEF]);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run([
            "commit",
            "-q",
            "-m",
            "Add the brief\n\nAthena-Job: bootstrap",
        ])
        .unwrap();

    // A clone of the repository lands a job, so its records hold a real outcome. The clone's history is
    // pulled back here, so the audit can read the files the job changed; the pulled records are then
    // dropped from the tree and the tree is given its own job `T`.
    let other = Fixture {
        dir: tempfile::tempdir().unwrap(),
        state: tempfile::tempdir().unwrap(),
        git: Git::at(Path::new("/nonexistent")),
    };
    let other = Fixture {
        git: {
            fx.git
                .run([
                    "clone",
                    "-q",
                    fx.root().to_str().unwrap(),
                    other.root().to_str().unwrap(),
                ])
                .unwrap();
            let git = Git::at(other.root());
            for (key, value) in [
                ("user.name", "Test User"),
                ("user.email", "noreply@example.com"),
                ("commit.gpgsign", "false"),
                ("core.hooksPath", "/dev/null"),
            ] {
                git.run(["config", key, value]).unwrap();
            }
            git
        },
        ..other
    };
    let ui = Lines::default();
    let athena = other.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting to the readme", &ui).unwrap();
    let w = proposal.job.as_str().to_string();
    let GoResult::Finished(done) = athena.go(&w, AcceptedBy::User, &ui).unwrap() else {
        panic!("expected the job to finish");
    };
    assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
    fx.git
        .run([
            "pull",
            "-q",
            "--ff-only",
            other.root().to_str().unwrap(),
            "main",
        ])
        .unwrap();
    fx.git.run(["rm", "-r", "-q", "records"]).unwrap();
    fx.git
        .run([
            "commit",
            "-q",
            "-m",
            "Drop the pulled records from the tree",
        ])
        .unwrap();
    let t = queue_one(&fx, "the request that is only in the tree");
    assert_ne!(w, t);

    let world = World::from_parts(fx, t, &other, w, true);
    std::fs::remove_dir_all(world.fx.root().join("records")).unwrap();
    assert!(!world.fx.root().join("records").exists());
    let reader_home = stand_in_home(&world, "reader", &default_reading(), |m| m.reader.clone());
    let auditor_home = stand_in_home(
        &world,
        "auditor",
        &serde_json::json!({
            "summary": "The outcome serves the goal.",
            "findings": [{ "text": "The greeting is there.", "evidence": "README.md line 3" }]
        }),
        |m| m.verifier.clone(),
    );
    // A secret scanner that finds nothing: gitleaks writes no report, which Athena reads as no findings.
    let gitleaks = world.fx.state.path().join("gitleaks");
    std::fs::write(&gitleaks, "#!/bin/sh\nexit 0\n").unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&gitleaks, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let with_home = |home: &Path, args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_athena"))
            .env("ATHENA_GITLEAKS", &gitleaks)
            .current_dir(world.fx.root())
            .env("ATHENA_STATE_DIR", world.fx.state.path())
            .env("ATHENA_HOME", home)
            .env_remove("ATHENA_NESTED")
            .env_remove("ATHENA_SCRATCH_DIR")
            .args(args)
            .output()
            .unwrap()
    };

    let status = athena_bin(&world, &["status"]);
    assert!(status.status.success(), "{}", stderr(&status));
    let text = stdout(&status);
    assert!(text.contains(&world.w), "status names W:\n{text}");
    assert!(!text.contains(&world.t), "status does not name T:\n{text}");

    let audit_w = with_home(&auditor_home, &["audit", &world.w]);
    assert!(audit_w.status.success(), "{}", stderr(&audit_w));
    assert!(
        stdout(&audit_w).contains(&world.w),
        "audit names W:\n{}",
        stdout(&audit_w)
    );
    assert!(
        !stdout(&audit_w).contains(&world.t),
        "audit does not name T:\n{}",
        stdout(&audit_w)
    );
    let audit_t = with_home(&auditor_home, &["audit", &world.t]);
    assert!(!audit_t.status.success());
    assert!(
        stderr(&audit_t).contains("no job matches"),
        "{}",
        stderr(&audit_t)
    );

    // A forecast: `ask` reads a new request with the stand-in reader and prints its predictions, including
    // the reference forecast computed from the records the worktree holds.
    let ask = with_home(&reader_home, &["ask", "add a greeting to the readme"]);
    assert!(ask.status.success(), "{}", stderr(&ask));
    assert!(
        stdout(&ask).contains("predicts") && stdout(&ask).contains("reference"),
        "{}",
        stdout(&ask)
    );
    assert!(!stdout(&ask).contains(&world.t));
}

/// The same binary on a checkout with no worktree reads the tree: `status` names T and not W.
#[test]
fn scratch_checkout_without_a_worktree_reads_the_tree() {
    let world = World::new(false);
    let status = athena_bin(&world, &["status"]);
    assert!(status.status.success(), "{}", stderr(&status));
    let text = stdout(&status);
    assert!(text.contains(&world.t), "{text}");
    assert!(!text.contains(&world.w), "{text}");
}
