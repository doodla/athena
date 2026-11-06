//! The producer's `cargo` wrapper, run as a real process against a fake `cargo` and a real build-slot queue.
//!
//! The wrapper is installed by the same function the run uses ([`install_cargo_wrapper`]), so the link, the
//! `PATH` and the queue variables are the producer's own. The fake `cargo` logs its start and end and waits for a
//! release file, which lets a test look at the queue while a wrapped command is running.

use athena_agent::env::{CargoWrapper, install_cargo_wrapper};
use athena_core::disk::{BUILD_SLOT_WAIT, read_slot_hold};
use athena_core::slots::{Class, SlotQueue};
use athena_core::state::StateDir;
use athena_data::JobId;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const BOUND: Duration = Duration::from_secs(20);

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    state: StateDir,
    job: JobId,
    env: BTreeMap<String, String>,
    queue: SlotQueue,
}

fn fixture(wait_s: u64) -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    let state = StateDir::new(root.join("state"));
    let job = JobId::parse("20261026-001200-abcd").unwrap();
    let fake = root.join("fake");
    std::fs::create_dir_all(&fake).unwrap();
    // The fake cargo: log the start, wait for the release file, log the end.
    let script = "#!/bin/sh\n\
        echo \"start $FAKE_TAG $*\" >> \"$FAKE_LOG\"\n\
        while [ ! -e \"$FAKE_RELEASE\" ] && [ ! -e \"$FAKE_DONE\" ]; do sleep 0.02; done\n\
        echo \"end $FAKE_TAG $*\" >> \"$FAKE_LOG\"\n";
    let cargo = fake.join("cargo");
    std::fs::write(&cargo, script).unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let hold_file = state.hold_file(&job);
    let mut env = BTreeMap::from([(
        "PATH".to_string(),
        format!("{}:/usr/bin:/bin", fake.display()),
    )]);
    install_cargo_wrapper(
        &mut env,
        &CargoWrapper {
            athena_bin: Path::new(env!("CARGO_BIN_EXE_athena")),
            bin_dir: &root.join("bin"),
            slots_dir: &state.slots_dir(),
            slots: 1,
            wait_s,
            promote_s: 600,
            hold_file: &hold_file,
        },
    )
    .unwrap();
    let queue = SlotQueue::new(
        state.slots_dir(),
        1,
        Duration::from_secs(wait_s),
        Duration::from_secs(600),
    )
    .with_poll(Duration::from_millis(10));
    Fixture {
        _temp: temp,
        root,
        state,
        job,
        env,
        queue,
    }
}

impl Fixture {
    fn log(&self) -> String {
        std::fs::read_to_string(self.root.join("log")).unwrap_or_default()
    }

    fn release(&self, tag: &str) {
        std::fs::write(self.root.join(format!("release-{tag}")), b"").unwrap();
    }

    /// The wrapper, as the producer runs it: `cargo <args>` found on the producer's `PATH`.
    fn cargo(&self, tag: &str, args: &[&str]) -> Command {
        let mut command = Command::new(self.root.join("bin").join("cargo"));
        command
            .args(args)
            .env_clear()
            .envs(&self.env)
            .env("FAKE_TAG", tag)
            .env("FAKE_LOG", self.root.join("log"))
            .env("FAKE_DONE", self.root.join("done"))
            .env("FAKE_RELEASE", self.root.join(format!("release-{tag}")))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn wait_for_log(&self, needle: &str) {
        let start = Instant::now();
        while !self.log().contains(needle) {
            assert!(
                start.elapsed() < BOUND,
                "no {needle:?} in the log: {}",
                self.log()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Whether every slot is held now.
    fn slot_is_held(&self) -> bool {
        self.queue.try_acquire().unwrap().is_none()
    }

    /// Takes the slot, retrying briefly: a child between its fork and its exec can still hold a copy of the
    /// lock file for a few milliseconds.
    fn take_soon(&self) -> athena_core::slots::Slot {
        let start = Instant::now();
        loop {
            if let Some(slot) = self.queue.try_acquire().unwrap() {
                return slot;
            }
            assert!(start.elapsed() < BOUND, "the slot was never freed");
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for Fixture {
    /// Lets every fake cargo still waiting end, so a failed assertion leaves no process behind.
    fn drop(&mut self) {
        let _ = std::fs::write(self.root.join("done"), b"");
    }
}

fn finish(child: Child) -> std::process::Output {
    child.wait_with_output().unwrap()
}

#[test]
fn the_second_producer_waits_for_the_one_slot_then_runs() {
    let fx = fixture(30);
    fx.release("b");
    let first = fx.cargo("a", &["build"]).spawn().unwrap();
    fx.wait_for_log("start a build");
    assert!(fx.slot_is_held(), "the first command holds the slot");

    let second = fx.cargo("b", &["test"]).spawn().unwrap();
    let start = Instant::now();
    while fx.queue.waiting() < 1 {
        assert!(start.elapsed() < BOUND, "the second producer never queued");
        std::thread::sleep(Duration::from_millis(5));
    }
    // The second is queued; it has not started while the first holds the slot.
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(fx.log(), "start a build\n");

    fx.release("a");
    assert!(finish(first).status.success());
    assert!(finish(second).status.success());
    assert_eq!(
        fx.log(),
        "start a build\nend a build\nstart b test\nend b test\n",
        "the second ran only after the first finished"
    );
    drop(fx.take_soon());
}

#[test]
fn a_global_flag_before_the_subcommand_does_not_skip_the_queue() {
    for args in [vec!["-q", "build"], vec!["--locked", "test"]] {
        let fx = fixture(30);
        let child = fx.cargo("t", &args).spawn().unwrap();
        fx.wait_for_log(&format!("start t {}", args.join(" ")));
        assert!(
            fx.slot_is_held(),
            "`cargo {}` took no slot: the wrapper must read the subcommand after the global flags",
            args.join(" ")
        );
        fx.release("t");
        let out = finish(child);
        assert!(out.status.success());
        assert!(
            fx.log().contains(&format!("end t {}", args.join(" "))),
            "{}",
            fx.log()
        );
        drop(fx.take_soon());
    }
}

#[test]
fn clippy_and_check_take_a_slot_too() {
    for sub in ["clippy", "check"] {
        let fx = fixture(30);
        let child = fx.cargo("t", &["--offline", sub]).spawn().unwrap();
        fx.wait_for_log("start t");
        assert!(fx.slot_is_held(), "cargo {sub} holds a slot");
        fx.release("t");
        assert!(finish(child).status.success());
        drop(fx.take_soon());
    }
}

#[test]
fn no_slot_within_the_limit_ends_the_command_and_holds_the_job() {
    let fx = fixture(1);
    let held = fx.queue.acquire(Class::Check, &|| Ok(())).unwrap();
    // Released in advance, so a wrapper that wrongly started cargo would end and leave a log line.
    fx.release("t");
    let out = fx.cargo("t", &["--locked", "build"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(75), "{stderr}");
    assert!(
        stderr.contains("no build slot free for 1 s; try again"),
        "{stderr}"
    );
    assert_eq!(fx.log(), "", "the real cargo was never started");
    let hold = read_slot_hold(&fx.state, &fx.job).expect("the job is recorded as held");
    assert_eq!(hold.limit_s, 1);
    assert!(hold.text().starts_with(BUILD_SLOT_WAIT), "{}", hold.text());
    assert_eq!(
        fx.queue.waiting(),
        0,
        "the timed-out request left the queue"
    );
    drop(held);
}

#[test]
fn fmt_and_metadata_run_without_a_slot_even_when_every_slot_is_held() {
    let fx = fixture(1);
    let held = fx.queue.acquire(Class::Check, &|| Ok(())).unwrap();
    for (tag, args) in [("f", vec!["fmt", "--check"]), ("m", vec!["metadata"])] {
        fx.release(tag);
        let start = Instant::now();
        let out = fx.cargo(tag, &args).output().unwrap();
        assert!(
            out.status.success(),
            "cargo {} did not run: {}",
            args[0],
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            start.elapsed() < Duration::from_millis(900),
            "it waited for a slot"
        );
    }
    assert_eq!(
        fx.log(),
        "start f fmt --check\nend f fmt --check\nstart m metadata\nend m metadata\n"
    );
    assert!(read_slot_hold(&fx.state, &fx.job).is_none());
    assert!(
        fx.slot_is_held(),
        "the test's own slot is still the only holder"
    );
    drop(held);
}
