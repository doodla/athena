//! Process groups, the registry, and the kill switch (design §10.4, M13).
//!
//! Every agent and check runs in a process group of its own, with the parent-death signal set on Linux,
//! and is recorded in the state directory's registry while it runs. `athena stop` creates the `STOP` file
//! and terminates every registered group, including groups whose `athena` parent has died. Termination
//! is graceful: SIGTERM to the group, then SIGKILL after a grace period.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The grace period between SIGTERM and SIGKILL.
pub const GRACE: Duration = Duration::from_secs(5);

/// The kill-switch file inside a repository's state directory.
pub const STOP_FILE: &str = "STOP";

/// The reason recorded when an agent's own deadline of `secs` seconds passes.
pub fn deadline_reason(secs: u64) -> String {
    format!("deadline of {secs}s reached")
}

/// The shortest stall window the driver uses, so a short timeout is never cut by a stall check first.
pub const MIN_STALL_WINDOW: Duration = Duration::from_secs(60);

/// How long an agent may show no activity before the driver ends it: a third of its timeout, but never
/// less than `floor`. A window not shorter than the timeout can never fire before the deadline does.
pub fn stall_window(timeout: Duration, floor: Duration) -> Duration {
    (timeout / 3).max(floor)
}

/// The reason recorded when an agent showed no activity for `secs` seconds.
pub fn stall_reason(secs: u64) -> String {
    format!("no progress for {secs}s (stalled)")
}

#[allow(unsafe_code)]
mod sys {
    //! The only unsafe code in Athena: three libc calls, each with its contract stated.

    /// Sends `signal` to every process in group `pgid`. Returns false if no such group exists.
    pub fn signal_group(pgid: i32, signal: i32) -> bool {
        if pgid <= 1 {
            return false;
        }
        // SAFETY: kill(2) takes plain integers and has no memory-safety preconditions. A negative pid
        // addresses the process group; pgid > 1 excludes "every process" (-1) and our own group (0).
        unsafe { libc::kill(-pgid, signal) == 0 }
    }

    /// Whether a process exists (signal 0 checks without sending anything).
    pub fn process_alive(pid: i32) -> bool {
        if pid <= 0 {
            return false;
        }
        // SAFETY: as above; signal 0 performs only the existence and permission check.
        unsafe {
            libc::kill(pid, 0) == 0
                || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
        }
    }

    /// Makes the child receive SIGTERM when the thread that spawned it exits (Linux only).
    #[cfg(target_os = "linux")]
    pub fn set_parent_death_signal(cmd: &mut std::process::Command) {
        use std::os::unix::process::CommandExt;
        // SAFETY: the closure runs in the child between fork and exec, where only async-signal-safe
        // functions may be called. prctl(2) with PR_SET_PDEATHSIG is async-signal-safe and touches no
        // memory of the parent.
        unsafe {
            cmd.pre_exec(|| {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM, 0, 0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn set_parent_death_signal(_cmd: &mut std::process::Command) {}
}

pub use sys::{process_alive, signal_group};

/// Puts a command in its own process group and ties it to its parent's life.
pub fn isolate(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
    sys::set_parent_death_signal(cmd);
}

/// Whether any process of group `pgid` is alive.
pub fn group_alive(pgid: i32) -> bool {
    signal_group(pgid, 0)
}

/// Terminates a process group: SIGTERM, then SIGKILL after `grace` if any member survives.
pub fn kill_group(pgid: i32, grace: Duration) {
    if !signal_group(pgid, libc::SIGTERM) {
        return;
    }
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if !group_alive(pgid) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    signal_group(pgid, libc::SIGKILL);
}

/// One running group, as recorded in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub pgid: i32,
    /// The `athena` process that started the group.
    pub owner_pid: i32,
    pub job: String,
    pub what: String,
}

/// The process-group registry of one repository's state directory.
#[derive(Debug, Clone)]
pub struct Registry {
    dir: PathBuf,
    stop_file: PathBuf,
}

impl Registry {
    /// The registry under a repository's state directory.
    pub fn new(state_dir: &Path) -> Registry {
        Registry {
            dir: state_dir.join("procs"),
            stop_file: state_dir.join(STOP_FILE),
        }
    }

    pub fn stop_file(&self) -> &Path {
        &self.stop_file
    }

    /// Whether the kill switch is on.
    pub fn stopped(&self) -> bool {
        self.stop_file.exists()
    }

    pub fn register(&self, pgid: i32, job: &str, what: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let entry = Entry {
            pgid,
            owner_pid: std::process::id() as i32,
            job: job.to_string(),
            what: what.to_string(),
        };
        let text = serde_json::to_string(&entry).map_err(std::io::Error::other)?;
        std::fs::write(self.dir.join(format!("{pgid}.json")), text)
    }

    pub fn unregister(&self, pgid: i32) {
        let _ = std::fs::remove_file(self.dir.join(format!("{pgid}.json")));
    }

    pub fn entries(&self) -> Vec<Entry> {
        let Ok(dir) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        dir.filter_map(Result::ok)
            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
            .filter_map(|text| serde_json::from_str(&text).ok())
            .collect()
    }

    /// Removes entries whose group is gone, and terminates groups whose owner died (orphans).
    /// Returns the orphans it terminated.
    pub fn reap(&self) -> Vec<Entry> {
        let mut orphans = Vec::new();
        for entry in self.entries() {
            if !group_alive(entry.pgid) {
                self.unregister(entry.pgid);
            } else if !process_alive(entry.owner_pid) {
                kill_group(entry.pgid, GRACE);
                self.unregister(entry.pgid);
                orphans.push(entry);
            }
        }
        orphans
    }

    /// The kill switch: creates `STOP` and terminates every registered group.
    pub fn stop_all(&self) -> std::io::Result<Vec<Entry>> {
        if let Some(parent) = self.stop_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &self.stop_file,
            "stopped by `athena stop`; `athena start` removes this file\n",
        )?;
        let entries = self.entries();
        for entry in &entries {
            signal_group(entry.pgid, libc::SIGTERM);
        }
        let deadline = Instant::now() + GRACE;
        while Instant::now() < deadline && entries.iter().any(|e| group_alive(e.pgid)) {
            std::thread::sleep(Duration::from_millis(100));
        }
        for entry in &entries {
            if group_alive(entry.pgid) {
                signal_group(entry.pgid, libc::SIGKILL);
            }
            self.unregister(entry.pgid);
        }
        Ok(entries)
    }

    /// Turns the kill switch off.
    pub fn start(&self) -> std::io::Result<bool> {
        match std::fs::remove_file(&self.stop_file) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sleeper() -> std::process::Child {
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        isolate(&mut cmd);
        cmd.spawn().unwrap()
    }

    #[test]
    fn stop_terminates_every_registered_group_and_blocks_new_work() {
        // M13 (blocks): `stop` terminates a running agent; a stop file blocks new invocations.
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let mut child = sleeper();
        let pgid = child.id() as i32;
        registry.register(pgid, "job", "fake agent").unwrap();
        let stopped = registry.stop_all().unwrap();
        assert_eq!(stopped.len(), 1);
        assert!(registry.stopped());
        let status = child.wait().unwrap();
        assert!(!status.success());
        assert!(registry.entries().is_empty());
        assert!(registry.start().unwrap());
        assert!(!registry.stopped());
    }

    #[test]
    fn a_timeout_kills_the_whole_group() {
        // M13 (blocks): a timeout kills the group.
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 30 & sleep 30"]);
        isolate(&mut cmd);
        let mut child = cmd.spawn().unwrap();
        let pgid = child.id() as i32;
        std::thread::sleep(Duration::from_millis(200));
        kill_group(pgid, GRACE);
        child.wait().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        assert!(!group_alive(pgid), "the background sleep is gone too");
    }

    /// Not a test on its own: the parent process for `killing_the_parent_kills_its_agent`. It spawns an
    /// isolated sleeper, prints its pid, and waits to be killed.
    #[test]
    #[ignore = "a helper process for killing_the_parent_kills_its_agent"]
    fn helper_parent_of_an_isolated_agent() {
        if std::env::var("ATHENA_TEST_PARENT").as_deref() != Ok("1") {
            return;
        }
        let mut child = sleeper();
        println!("agent-pid {}", child.id());
        std::thread::sleep(Duration::from_secs(60));
        let _ = child.wait();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn killing_the_parent_kills_its_agent() {
        // M13 (blocks): killing `athena go` mid-invocation kills its agent (the parent-death signal).
        use std::io::BufRead;
        let mut parent = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "procs::tests::helper_parent_of_an_isolated_agent",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("ATHENA_TEST_PARENT", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let stdout = parent.stdout.take().unwrap();
        let mut pid = None;
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap();
            if let Some(rest) = line.split("agent-pid ").nth(1) {
                pid = rest.trim().parse::<i32>().ok();
                break;
            }
        }
        let pid = pid.expect("the helper reported its agent");
        assert!(process_alive(pid));
        parent.kill().unwrap();
        parent.wait().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while process_alive(pid) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(!group_alive(pid), "the agent died with its parent");
    }

    #[test]
    fn reaping_removes_finished_groups() {
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let mut child = Command::new("true").spawn().unwrap();
        let pgid = child.id() as i32;
        child.wait().unwrap();
        registry.register(pgid, "job", "done").unwrap();
        registry.reap();
        assert!(registry.entries().is_empty());
    }
}
