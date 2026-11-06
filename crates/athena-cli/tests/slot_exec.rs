//! The producer's `cargo` wrapper (`athena_cli::slot_exec`), run as the real `athena` binary linked as `cargo`.
//!
//! Each test builds the producer's environment the way the run does ([`CargoShim::apply`]), puts a fake `cargo`
//! script after the shim directory, and uses a real [`SlotQueue`] over a temporary directory. The fake holds no
//! slot logic: it records its arguments and its `PATH`, says it started, and (when asked) waits for a release
//! file. Whether a slot was held while it ran is read from the queue by the test.

use athena_agent::env::CargoShim;
use athena_core::disk::{BUILD_SLOT_WAIT, SlotWait};
use athena_core::slots::{self, Slot, SlotQueue};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const BOUND: Duration = Duration::from_secs(10);

const FAKE_CARGO: &str = r#"#!/bin/sh
printf '%s\n' "$@" > "$FAKE_MARKS/argv-$FAKE_TAG"
printf '%s\n' "$PATH" > "$FAKE_MARKS/path-$FAKE_TAG"
echo started > "$FAKE_MARKS/started-$FAKE_TAG"
if [ -n "$FAKE_HOLD" ]; then
    while [ ! -e "$FAKE_MARKS/release-$FAKE_TAG" ]; do sleep 0.02; done
fi
exit 0
"#;

struct Fixture {
    _temp: tempfile::TempDir,
    shim: CargoShim,
    fake_dir: PathBuf,
    marks: PathBuf,
}

/// One build slot, a wrapper that waits `wait_s`, the `athena` binary linked as `cargo`, and a fake cargo.
fn fixture(wait_s: u64) -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let job_dir = temp.path().join("jobs/j");
    let shim = CargoShim {
        dir: job_dir.join("shim"),
        slots_dir: temp.path().join("slots"),
        slots: 1,
        wait_s,
        promote_s: 600,
        note: slots::wait_note_path(&job_dir),
    };
    std::fs::create_dir_all(&job_dir).unwrap();
    shim.install(Path::new(env!("CARGO_BIN_EXE_athena")))
        .unwrap();
    let fake_dir = temp.path().join("fake");
    std::fs::create_dir_all(&fake_dir).unwrap();
    let fake = fake_dir.join("cargo");
    std::fs::write(&fake, FAKE_CARGO).unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    let marks = temp.path().join("marks");
    std::fs::create_dir_all(&marks).unwrap();
    Fixture {
        _temp: temp,
        shim,
        fake_dir,
        marks,
    }
}

impl Fixture {
    fn queue(&self) -> SlotQueue {
        SlotQueue::new(
            self.shim.slots_dir.clone(),
            1,
            Duration::from_secs(30),
            Duration::from_secs(600),
        )
        .with_poll(Duration::from_millis(10))
    }

    /// The producer's `cargo <args>`: the shim's link, in the environment the run gives the producer.
    fn cargo(&self, tag: &str, hold: bool, args: &[&str]) -> Command {
        let mut env = std::collections::BTreeMap::from([(
            "PATH".to_string(),
            format!("{}:/usr/bin:/bin", self.fake_dir.display()),
        )]);
        self.shim.apply(&mut env);
        env.insert("FAKE_TAG".into(), tag.into());
        env.insert("FAKE_MARKS".into(), self.marks.display().to_string());
        if hold {
            env.insert("FAKE_HOLD".into(), "1".into());
        }
        let mut command = Command::new(self.shim.dir.join("cargo"));
        command.args(args).env_clear().envs(env);
        command
    }

    fn mark(&self, kind: &str, tag: &str) -> PathBuf {
        self.marks.join(format!("{kind}-{tag}"))
    }

    fn release(&self, tag: &str) {
        std::fs::write(self.mark("release", tag), b"").unwrap();
    }

    fn argv(&self, tag: &str) -> Vec<String> {
        std::fs::read_to_string(self.mark("argv", tag))
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

fn wait_for_file(path: &Path) {
    let start = Instant::now();
    while !path.exists() {
        assert!(start.elapsed() < BOUND, "{} never appeared", path.display());
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn wait_for_waiting(queue: &SlotQueue, n: usize) {
    let start = Instant::now();
    while queue.waiting() < n {
        assert!(start.elapsed() < BOUND, "no {n} request queued");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A slot taken without waiting, retried for a moment: another test's child process, between its fork and its
/// exec, can hold a copy of a lock file that was just released, and the lock frees within milliseconds.
fn take_soon(queue: &SlotQueue) -> Slot {
    let start = Instant::now();
    loop {
        if let Some(slot) = queue.try_acquire().unwrap() {
            return slot;
        }
        assert!(start.elapsed() < BOUND, "the slot was never freed");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn spawn(mut command: Command) -> Child {
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

#[test]
fn a_second_producer_cargo_waits_for_the_slot_then_runs() {
    let fx = fixture(30);
    let queue = fx.queue();
    let mut first = spawn(fx.cargo("A", true, &["build"]));
    wait_for_file(&fx.mark("started", "A"));
    assert!(
        queue.try_acquire().unwrap().is_none(),
        "the first cargo holds the only slot while it runs"
    );

    let mut second = spawn(fx.cargo("B", true, &["build"]));
    wait_for_waiting(&queue, 1);
    assert!(
        !fx.mark("started", "B").exists(),
        "the second cargo has not run while the first holds the slot"
    );

    fx.release("A");
    assert!(first.wait().unwrap().success());
    // The slot passes to the second, which now runs.
    wait_for_file(&fx.mark("started", "B"));
    assert!(queue.try_acquire().unwrap().is_none(), "B holds the slot");
    fx.release("B");
    assert!(second.wait().unwrap().success());
    assert_eq!(queue.waiting(), 0);
    drop(take_soon(&queue));
}

#[test]
fn global_flags_before_the_subcommand_still_hold_a_slot() {
    let fx = fixture(30);
    let queue = fx.queue();
    for (tag, args) in [
        ("q", ["-q", "build"].as_slice()),
        ("locked", ["--locked", "test"].as_slice()),
        ("vv", ["-vv", "check", "-p", "x"].as_slice()),
        (
            "clippy",
            ["--offline", "clippy", "--all-targets"].as_slice(),
        ),
    ] {
        let mut child = spawn(fx.cargo(tag, true, args));
        wait_for_file(&fx.mark("started", tag));
        assert!(
            queue.try_acquire().unwrap().is_none(),
            "`cargo {}` holds a slot while it runs",
            args.join(" ")
        );
        // The real cargo gets the arguments unchanged and a PATH without the shim directory.
        assert_eq!(fx.argv(tag), args);
        let path = std::fs::read_to_string(fx.mark("path", tag)).unwrap();
        assert!(!path.contains(&fx.shim.dir.display().to_string()), "{path}");
        assert!(
            path.starts_with(&fx.fake_dir.display().to_string()),
            "{path}"
        );
        fx.release(tag);
        assert!(child.wait().unwrap().success());
        drop(take_soon(&queue));
    }
}

#[test]
fn a_wait_past_the_limit_exits_75_and_never_starts_cargo() {
    let fx = fixture(1);
    let queue = fx.queue();
    let holder = queue.try_acquire().unwrap().expect("the slot is free");

    let out = fx.cargo("W", false, &["build"]).output().unwrap();
    assert_eq!(out.status.code(), Some(75), "a non-failure exit");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "no build slot free for 1 s; try again\n"
    );
    assert!(out.stdout.is_empty());
    assert!(
        !fx.mark("started", "W").exists() && !fx.mark("argv", "W").exists(),
        "the real cargo was never started"
    );
    assert_eq!(queue.waiting(), 0, "the gave-up request left the queue");
    // The note the run reads: the job is held as a build slot wait, not failed.
    let wait: SlotWait = slots::read_wait_note(&fx.shim.note).expect("a wait note");
    assert_eq!(wait.limit_s, 1);
    assert!(wait.waited_s >= 1, "{wait:?}");
    assert!(wait.text().starts_with(BUILD_SLOT_WAIT), "{}", wait.text());
    assert_eq!(BUILD_SLOT_WAIT, "environment: build slot wait");

    // The producer tries again once a slot is free: cargo runs, and the note is gone, so nothing is held.
    drop(holder);
    let again = fx.cargo("W2", false, &["build"]).output().unwrap();
    assert!(again.status.success());
    assert!(fx.mark("started", "W2").exists());
    assert_eq!(slots::read_wait_note(&fx.shim.note), None);
    drop(take_soon(&queue));
}

#[test]
fn fmt_and_metadata_take_no_slot_even_when_every_slot_is_held() {
    let fx = fixture(1);
    let queue = fx.queue();
    let holder = queue.try_acquire().unwrap().expect("the slot is free");
    for (tag, args) in [
        ("fmt", ["fmt", "--check"].as_slice()),
        ("meta", ["metadata", "--no-deps"].as_slice()),
        ("tree", ["--locked", "tree"].as_slice()),
    ] {
        let out = fx.cargo(tag, false, args).output().unwrap();
        assert!(
            out.status.success(),
            "`cargo {}` ran: {:?} {}",
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(fx.argv(tag), args, "the real cargo ran with its arguments");
        assert_eq!(queue.waiting(), 0, "it did not queue for a slot");
        assert_eq!(slots::read_wait_note(&fx.shim.note), None, "no wait note");
    }
    assert!(
        queue.try_acquire().unwrap().is_none(),
        "the test's slot was held throughout"
    );
    drop(holder);
}

#[test]
fn a_wrapper_without_settings_says_so_and_runs_nothing() {
    let fx = fixture(1);
    // The link run with no shim variables: it neither guesses nor looks itself up on PATH.
    let out = Command::new(fx.shim.dir.join("cargo"))
        .arg("build")
        .env_clear()
        .env("PATH", format!("{}:/usr/bin:/bin", fx.fake_dir.display()))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(70));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("ATHENA_CARGO_SHIM_DIR"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
