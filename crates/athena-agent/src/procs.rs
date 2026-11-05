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

/// The environment variable that marks every process of one measured build. A compiler that an `sccache`
/// server runs gets the environment of the client that asked for it, so it carries the mark even though it
/// is not a descendant of the build's root process.
pub const BUILD_MARK_ENV: &str = "ATHENA_BUILD_MARK";

/// One process as the sampler saw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcSnapshot {
    pub pid: i32,
    pub ppid: i32,
    /// Resident memory in bytes.
    pub rss_bytes: u64,
    /// Whether the process carries this build's [`BUILD_MARK_ENV`].
    pub marked: bool,
}

/// The peaks of a build's processes over the sampled moments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BuildPeak {
    /// The highest resident memory summed over all build processes at one moment.
    pub memory_bytes: u64,
    /// The highest number of build processes alive at one moment.
    pub processes: u32,
    /// The highest resident memory of any single process: kept to show how far the total is from it.
    pub largest_process_bytes: u64,
}

impl BuildPeak {
    fn merge(self, other: BuildPeak) -> BuildPeak {
        BuildPeak {
            memory_bytes: self.memory_bytes.max(other.memory_bytes),
            processes: self.processes.max(other.processes),
            largest_process_bytes: self.largest_process_bytes.max(other.largest_process_bytes),
        }
    }
}

/// The build's processes at one moment: `root`, every descendant of it, every process carrying the build's
/// mark (compilers an `sccache` server runs, or anything else that detached) and their descendants. Returns
/// the summed resident memory, the process count and the largest single process. Zero when `root` and all
/// marked processes are gone.
pub fn build_totals(table: &[ProcSnapshot], root: i32) -> BuildPeak {
    use std::collections::HashSet;
    let mut members: HashSet<i32> = table
        .iter()
        .filter(|p| p.pid == root || p.marked)
        .map(|p| p.pid)
        .collect();
    loop {
        let before = members.len();
        for p in table {
            if members.contains(&p.ppid) {
                members.insert(p.pid);
            }
        }
        if members.len() == before {
            break;
        }
    }
    let mut total = BuildPeak::default();
    for p in table.iter().filter(|p| members.contains(&p.pid)) {
        total.memory_bytes += p.rss_bytes;
        total.processes += 1;
        total.largest_process_bytes = total.largest_process_bytes.max(p.rss_bytes);
    }
    total
}

/// Reads every live process from `/proc` (Linux); empty where `/proc` is absent. Zombies are left out.
/// `mark` is the value of [`BUILD_MARK_ENV`] that makes a process part of the build.
pub fn read_proc_table(mark: &str) -> Vec<ProcSnapshot> {
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let wanted = format!("{BUILD_MARK_ENV}={mark}");
    let mut table = Vec::new();
    for entry in dir.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|n| n.parse::<i32>().ok())
        else {
            continue;
        };
        let Ok(status) = std::fs::read_to_string(entry.path().join("status")) else {
            continue;
        };
        let field = |name: &str| {
            status
                .lines()
                .find_map(|l| l.strip_prefix(name)?.strip_prefix(':'))
                .map(str::trim)
        };
        if field("State").is_some_and(|s| s.starts_with('Z')) {
            continue;
        }
        let Some(ppid) = field("PPid").and_then(|v| v.parse().ok()) else {
            continue;
        };
        // Kernel threads have no VmRSS line; they hold no user memory.
        let rss_kib: u64 = field("VmRSS")
            .and_then(|v| v.split_whitespace().next()?.parse().ok())
            .unwrap_or(0);
        let marked = std::fs::read(entry.path().join("environ"))
            .map(|env| env.split(|b| *b == 0).any(|var| var == wanted.as_bytes()))
            .unwrap_or(false);
        table.push(ProcSnapshot {
            pid,
            ppid,
            rss_bytes: rss_kib * 1024,
            marked,
        });
    }
    table
}

/// How often [`output_measured`] samples the process list.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(25);

/// Runs `cmd` to completion like [`Command::output`] (stdout and stderr captured, stdin closed) while a
/// thread samples the whole build every [`SAMPLE_INTERVAL`] and keeps the highest total. The peak is the
/// highest sum of resident memory of all build processes at one sampled moment, and the highest count of
/// them, not the largest single process; a process that lives for less than one interval can be missed, so
/// both are lower bounds. Resident memory counts a page shared between processes once for each, so the sum
/// can run above the memory the build really takes. The peak is `None` where `/proc` cannot be read.
pub fn output_measured(
    cmd: &mut Command,
) -> std::io::Result<(std::process::Output, Option<BuildPeak>)> {
    use std::io::Read;
    use std::process::Stdio;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mark = format!("{}-{nanos}", std::process::id());
    cmd.env(BUILD_MARK_ENV, &mark)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let root = child.id() as i32;
    let drain = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_end(&mut bytes);
            }
            bytes
        })
    };
    let out = drain(child.stdout.take().map(|p| Box::new(p) as _));
    let err = drain(child.stderr.take().map(|p| Box::new(p) as _));

    let done = Arc::new(AtomicBool::new(false));
    let sampler = {
        let done = Arc::clone(&done);
        std::thread::spawn(move || {
            let mut peak: Option<BuildPeak> = None;
            loop {
                let table = read_proc_table(&mark);
                let now = build_totals(&table, root);
                if now.processes > 0 {
                    peak = Some(peak.map_or(now, |p| p.merge(now)));
                }
                if done.load(Ordering::Relaxed) {
                    return peak;
                }
                std::thread::sleep(SAMPLE_INTERVAL);
            }
        })
    };
    let status = child.wait()?;
    done.store(true, Ordering::Relaxed);
    let peak = sampler.join().ok().flatten();
    let stdout = out.join().unwrap_or_default();
    let stderr = err.join().unwrap_or_default();
    Ok((
        std::process::Output {
            status,
            stdout,
            stderr,
        },
        peak,
    ))
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

    fn snap(pid: i32, ppid: i32, mib: u64, marked: bool) -> ProcSnapshot {
        ProcSnapshot {
            pid,
            ppid,
            rss_bytes: mib << 20,
            marked,
        }
    }

    #[test]
    fn totals_sum_the_tree_and_marked_processes_that_left_it_and_not_others() {
        // 10 is the build root with children 11 and 12 (12 has child 13); 50 was detached to init but
        // carries the mark, and has child 51; 99 is some other process.
        let table = [
            snap(1, 0, 5, false),
            snap(10, 1, 10, false),
            snap(11, 10, 100, false),
            snap(12, 10, 200, false),
            snap(13, 12, 300, false),
            snap(50, 1, 400, true),
            snap(51, 50, 50, false),
            snap(99, 1, 9000, false),
        ];
        let all = build_totals(&table, 10);
        assert_eq!(all.memory_bytes, (10 + 100 + 200 + 300 + 400 + 50) << 20);
        assert_eq!(all.processes, 6);
        assert_eq!(all.largest_process_bytes, 400 << 20);
        // Without the mark the detached compiler and its child are lost, which is the old wait4 gap.
        let unmarked: Vec<_> = table
            .iter()
            .map(|p| ProcSnapshot {
                marked: false,
                ..*p
            })
            .collect();
        let tree = build_totals(&unmarked, 10);
        assert_eq!(tree.memory_bytes, (10 + 100 + 200 + 300) << 20);
        assert_eq!(tree.processes, 4);
        // A per-process maximum is far below the total.
        assert!(all.largest_process_bytes < all.memory_bytes / 2);
        assert_eq!(build_totals(&table, 4242).processes, 2);
    }

    /// Not a test on its own: a process that holds `ATHENA_TEST_HOLD_MIB` of touched memory for a while.
    #[test]
    #[ignore = "a helper process for a_measured_build_totals_all_concurrent_processes_including_detached"]
    fn helper_hold_memory() {
        let Ok(mib) = std::env::var("ATHENA_TEST_HOLD_MIB") else {
            return;
        };
        let held = vec![1u8; mib.parse::<usize>().unwrap() << 20];
        std::thread::sleep(Duration::from_secs(4));
        std::hint::black_box(&held);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_measured_build_totals_all_concurrent_processes_including_detached() {
        const HOLD_MIB: u64 = 64;
        // The root shell starts one holder through a double fork, so it is reparented to init and is no
        // descendant of the root (as a compiler run by an sccache server is), then two plain holders.
        // Concurrent: the root shell and three holders.
        let script = r#"
            ( "$0" --exact procs::tests::helper_hold_memory --ignored >/dev/null 2>&1 & )
            sleep 0.5
            "$0" --exact procs::tests::helper_hold_memory --ignored >/dev/null 2>&1 &
            "$0" --exact procs::tests::helper_hold_memory --ignored >/dev/null 2>&1 &
            wait
        "#;
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(script)
            .arg(std::env::current_exe().unwrap())
            .env("ATHENA_TEST_HOLD_MIB", HOLD_MIB.to_string());
        let (out, peak) = output_measured(&mut cmd).unwrap();
        assert!(out.status.success());
        let peak = peak.expect("/proc is readable on Linux");

        let known_total = (3 * HOLD_MIB) << 20;
        assert!(
            peak.memory_bytes >= known_total,
            "peak {} MiB is below the {} MiB held at once",
            peak.memory_bytes >> 20,
            known_total >> 20
        );
        assert_eq!(peak.processes, 4, "the root shell and three holders");
        // The old figure, the largest single process, would fall short of the total by far.
        assert!(peak.largest_process_bytes >= HOLD_MIB << 20);
        assert!(
            peak.largest_process_bytes < known_total,
            "largest single process {} MiB",
            peak.largest_process_bytes >> 20
        );
        // Without the detached holder the build would show three processes and about two holders' memory,
        // short of the known total: the total above can only be reached with the detached one included.
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
