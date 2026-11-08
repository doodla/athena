//! The drain's life cycle: one pid file, and the commands that read it. Split out of `drain.rs`; every public
//! item is re-exported from [`super`], so callers use the `drain::` paths.
//!
//! Root cause of the hand-written `pkill`, `pgrep` and wait loops that matched themselves: Athena kept no record
//! of its own drain, so a person found it by matching text in the process list, and the command doing the
//! matching contains that text. A drain now writes `drain.pid` in the state directory when it starts and removes
//! it when it ends. Every command here finds the drain only through that file. Nothing reads or matches command
//! text, and nothing sends a signal: a stop is the stop file, which the drain reads at a job boundary.

use super::{Limits, Rebuild, reexec_args, run_setup, stop_file};
use athena_core::{Athena, Ui};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The file that names the running drain, inside the repository's state directory, next to [`stop_file`].
pub fn pid_file(state_root: &Path) -> PathBuf {
    state_root.join("drain.pid")
}

/// Where a drain started by `athena drain-restart` writes what it prints.
pub fn log_file(state_root: &Path) -> PathBuf {
    state_root.join("drain.log")
}

/// What `drain.pid` holds: who the drain is, and when and as which build it started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PidRecord {
    /// The drain's process id.
    pub pid: u32,
    /// The process start marker taken when the file was written (see [`Procs::start_marker`]). A pid that
    /// another process has since been given has a different marker, so it is not mistaken for the drain.
    pub started: String,
    /// The build that runs the drain, as `athena --version` shows it.
    pub build: String,
    /// When the drain started (UTC).
    pub at: String,
}

/// The one way code asks whether a process is alive: by its pid, never by its name or command line.
pub trait Procs {
    /// A string that is the same for as long as process `pid` lives (and across an exec), and different for any
    /// later process that is given the same pid. `None` when there is no such process, or it has already
    /// exited and is only waiting to be reaped.
    fn start_marker(&self, pid: u32) -> Option<String>;
}

/// The real processes of this machine.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealProcs;

impl Procs for RealProcs {
    fn start_marker(&self, pid: u32) -> Option<String> {
        if cfg!(target_os = "linux") {
            linux_marker(&std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?)
        } else {
            let out = std::process::Command::new("ps")
                .args(["-o", "lstart=,stat=", "-p", &pid.to_string()])
                .stdin(std::process::Stdio::null())
                .output()
                .ok()?;
            if !out.status.success() {
                return None;
            }
            ps_marker(&String::from_utf8_lossy(&out.stdout))
        }
    }
}

/// The start marker in the text of a Linux `/proc/<pid>/stat`: field 22, the start time in clock ticks since
/// boot. The command name (field 2) is in parentheses and may hold spaces and parentheses, so the fields are
/// counted after the last `)`. A zombie or dead process has no marker.
pub(super) fn linux_marker(stat: &str) -> Option<String> {
    let rest = &stat[stat.rfind(')')? + 1..];
    let mut fields = rest.split_whitespace();
    let state = fields.next()?;
    if state.starts_with('Z') || state.starts_with('X') {
        return None;
    }
    // Field 3 is the state; field 22 is the 19th after it.
    fields.nth(18).map(str::to_string)
}

/// The start marker in the output of `ps -o lstart=,stat= -p <pid>` (macOS and other systems without
/// `/proc`): the start time and the state, such as `Thu Nov  5 20:59:32 2026 S`. A zombie has no marker.
pub(super) fn ps_marker(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let (state, start) = words.split_last()?;
    if start.is_empty() || state.starts_with('Z') {
        return None;
    }
    Some(start.join(" "))
}

/// Why no drain counts as running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotRunning {
    /// There is no pid file.
    Missing,
    /// The pid file could not be read or is not a pid record.
    Unreadable(String),
    /// The pid file names a process that no longer exists.
    Dead(u32),
    /// The pid file names a pid that another process now holds.
    Recycled(u32),
}

impl NotRunning {
    /// The reason in words.
    pub fn reason(&self) -> String {
        match self {
            NotRunning::Missing => {
                "there is no pid file, so no drain started by this build is running".into()
            }
            NotRunning::Unreadable(why) => {
                format!("the pid file cannot be used ({why}), so it names no drain")
            }
            NotRunning::Dead(pid) => format!(
                "the pid file names process {pid}, which has exited (the drain ended without removing it)"
            ),
            NotRunning::Recycled(pid) => format!(
                "the pid file names process {pid}, but that pid now belongs to a different process that started later; it is not the drain"
            ),
        }
    }
}

/// Whether a drain is running, from the pid file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainState {
    Running(PidRecord),
    NotRunning(NotRunning),
}

impl DrainState {
    /// The exit code `athena drain-status` gives: 0 when a drain is running, 2 when none is.
    pub fn exit_code(&self) -> u8 {
        match self {
            DrainState::Running(_) => 0,
            DrainState::NotRunning(_) => 2,
        }
    }
}

/// Reads the pid file and checks the process it names by pid and start marker. A missing, unreadable, dead or
/// recycled entry is reported as not running, and no process is touched.
pub fn drain_state(state_root: &Path, procs: &dyn Procs) -> DrainState {
    let text = match std::fs::read_to_string(pid_file(state_root)) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return DrainState::NotRunning(NotRunning::Missing);
        }
        Err(e) => return DrainState::NotRunning(NotRunning::Unreadable(e.to_string())),
    };
    let record: PidRecord = match serde_json::from_str(&text) {
        Ok(record) => record,
        Err(e) => return DrainState::NotRunning(NotRunning::Unreadable(e.to_string())),
    };
    match procs.start_marker(record.pid) {
        None => DrainState::NotRunning(NotRunning::Dead(record.pid)),
        Some(marker) if marker != record.started => {
            DrainState::NotRunning(NotRunning::Recycled(record.pid))
        }
        Some(_) => DrainState::Running(record),
    }
}

/// Writes `record` as the pid file, replacing any file there. The new file is complete when it appears.
pub fn write_record(state_root: &Path, record: &PidRecord) -> std::io::Result<()> {
    std::fs::create_dir_all(state_root)?;
    let temp = temp_pid_file(state_root);
    std::fs::write(&temp, serde_json::to_vec_pretty(record)?)?;
    std::fs::rename(&temp, pid_file(state_root))
}

fn temp_pid_file(state_root: &Path) -> PathBuf {
    state_root.join(format!("drain.pid.{}.tmp", std::process::id()))
}

/// Creates the pid file only if there is none. The file is complete when it appears, and of two starts at the
/// same moment only one succeeds (`AlreadyExists` for the other).
fn publish_record(state_root: &Path, record: &PidRecord) -> std::io::Result<()> {
    std::fs::create_dir_all(state_root)?;
    let temp = temp_pid_file(state_root);
    std::fs::write(&temp, serde_json::to_vec_pretty(record)?)?;
    let linked = std::fs::hard_link(&temp, pid_file(state_root));
    let _ = std::fs::remove_file(&temp);
    linked
}

/// The pid file while this drain runs. Dropping it removes the file, if the file still names this process.
#[derive(Debug)]
pub struct PidGuard {
    state_root: PathBuf,
    pid: u32,
}

impl PidGuard {
    /// Takes the pid file for process `pid`. A live drain already named there refuses the start. A file that is
    /// missing, unreadable, dead or recycled is replaced. A file that names `pid` itself is the drain that
    /// replaced its own image after a rebuild (`exec` keeps the pid): it is replaced too, and the second value
    /// is `true`.
    pub fn acquire(
        state_root: &Path,
        procs: &dyn Procs,
        pid: u32,
        build: &str,
    ) -> Result<(PidGuard, bool), String> {
        let started = procs
            .start_marker(pid)
            .ok_or_else(|| format!("cannot read the start time of process {pid}"))?;
        let record = PidRecord {
            pid,
            started,
            build: build.to_string(),
            at: jiff::Timestamp::now().to_string(),
        };
        let guard = || PidGuard {
            state_root: state_root.to_path_buf(),
            pid,
        };
        for _ in 0..3 {
            match publish_record(state_root, &record) {
                Ok(()) => return Ok((guard(), false)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => {
                    return Err(format!(
                        "cannot write {}: {e}",
                        pid_file(state_root).display()
                    ));
                }
            }
            let before = std::fs::read_to_string(pid_file(state_root)).ok();
            match drain_state(state_root, procs) {
                DrainState::Running(old) if old.pid == pid => {
                    return write_record(state_root, &record)
                        .map(|()| (guard(), true))
                        .map_err(|e| {
                            format!("cannot write {}: {e}", pid_file(state_root).display())
                        });
                }
                DrainState::Running(old) => {
                    return Err(format!(
                        "a drain is already running (pid {}, {}, started {}); it is named in {}. Run \
                         `athena drain-status` to see it, or `athena drain-stop --wait` to stop it first",
                        old.pid,
                        old.build,
                        old.at,
                        pid_file(state_root).display()
                    ));
                }
                DrainState::NotRunning(_) => {
                    // Remove it only if it is still the file judged stale.
                    if std::fs::read_to_string(pid_file(state_root)).ok() == before {
                        let _ = std::fs::remove_file(pid_file(state_root));
                    }
                }
            }
        }
        Err(format!(
            "could not take {}: it keeps changing; run `athena drain-status`",
            pid_file(state_root).display()
        ))
    }
}

impl Drop for PidGuard {
    fn drop(&mut self) {
        let file = pid_file(&self.state_root);
        let ours = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| serde_json::from_str::<PidRecord>(&text).ok())
            .is_some_and(|record| record.pid == self.pid);
        if ours {
            let _ = std::fs::remove_file(file);
        }
    }
}

/// Runs a drain (`run`) as the one drain of this state directory. It refuses, and runs nothing, when `stale`
/// says the running build is older than the landed code (a drain on an old build lacks fixes that have
/// landed). It then takes the pid file (a second live drain is refused) and, unless this is a drain carrying on
/// after its own rebuild, removes a stop file left by an earlier run. The pid file is removed when `run` ends,
/// however it ends.
pub fn guarded<T>(
    state_root: &Path,
    procs: &dyn Procs,
    build: &str,
    stale: &dyn Fn() -> Option<String>,
    run: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    if let Some(line) = stale() {
        anyhow::bail!("refusing to start the drain, so nothing was dispatched: {line}");
    }
    let (guard, carried_over) = PidGuard::acquire(state_root, procs, std::process::id(), build)
        .map_err(anyhow::Error::msg)?;
    if !carried_over {
        // A stop asked for before this run started belongs to an earlier run. After an exec into the rebuilt
        // binary the stop belongs to this run, so it stays.
        let _ = std::fs::remove_file(stop_file(state_root));
    }
    let result = run();
    drop(guard);
    result
}

/// How long the stop and restart commands wait, and how often they look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopWait {
    /// How long to wait for the old drain to exit; `None` waits as long as it takes (a job can run for hours).
    pub limit: Option<Duration>,
    /// How often the pid file is looked at.
    pub poll: Duration,
    /// How often a progress line is printed while waiting.
    pub notice: Duration,
    /// How long a new drain has to show itself in the pid file.
    pub start: Duration,
}

impl StopWait {
    pub fn new(limit: Option<Duration>) -> StopWait {
        StopWait {
            limit,
            poll: Duration::from_millis(500),
            notice: Duration::from_secs(30),
            start: Duration::from_secs(60),
        }
    }
}

/// What `athena drain-stop` found and did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopOutcome {
    /// The stop is asked for and a drain is running; no wait was asked for.
    Asked(PidRecord),
    /// The stop is asked for and the drain has exited.
    Exited(PidRecord),
    /// The stop is asked for and the drain was still running when the wait ended.
    TimedOut(PidRecord, Duration),
    /// No drain is running (the stop file is left; the next drain removes it).
    NotRunning(NotRunning),
}

/// Asks the drain to stop at the next job boundary by creating the stop file (always, so a drain started by
/// an older build that wrote no pid file still stops), and, when `wait` is given, waits until the drain named
/// in the pid file has exited. No signal is sent to any process.
pub fn stop_and_wait(
    state_root: &Path,
    procs: &dyn Procs,
    wait: Option<&StopWait>,
    ui: &dyn Ui,
) -> std::io::Result<StopOutcome> {
    std::fs::create_dir_all(state_root)?;
    std::fs::write(stop_file(state_root), "stop at the next job boundary\n")?;
    let old = match drain_state(state_root, procs) {
        DrainState::NotRunning(why) => return Ok(StopOutcome::NotRunning(why)),
        DrainState::Running(old) => old,
    };
    let Some(wait) = wait else {
        return Ok(StopOutcome::Asked(old));
    };
    let began = std::time::Instant::now();
    let mut told = began;
    loop {
        std::thread::sleep(wait.poll);
        match drain_state(state_root, procs) {
            DrainState::Running(now) if now.pid == old.pid && now.started == old.started => {}
            _ => return Ok(StopOutcome::Exited(old)),
        }
        let waited = began.elapsed();
        if wait.limit.is_some_and(|limit| waited >= limit) {
            return Ok(StopOutcome::TimedOut(old, waited));
        }
        if told.elapsed() >= wait.notice {
            told = std::time::Instant::now();
            ui.line(&format!(
                "still waiting: drain pid {} is finishing its job ({}s so far)",
                old.pid,
                waited.as_secs()
            ));
        }
    }
}

/// What `athena drain-restart` needs from the world, so the order of its steps can be tested.
pub trait Lifecycle {
    /// The state directory.
    fn state_root(&self) -> &Path;
    /// How processes are looked up.
    fn procs(&self) -> &dyn Procs;
    /// Builds and installs Athena from the checkout (`./setup.sh`).
    fn rebuild(&self) -> Result<Rebuild, String>;
    /// The stale-build line when the INSTALLED binary is older than the checkout, else `None`.
    fn installed_stale(&self) -> Option<String>;
    /// Starts the installed binary as a drain in the background.
    fn spawn(&self) -> Result<Spawned, String>;
}

/// A drain process that was started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spawned {
    pub pid: u32,
    /// The file its output goes to.
    pub log: PathBuf,
}

/// The new drain `athena drain-restart` started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestartReport {
    /// The drain that was stopped, if one was running.
    pub old: Option<PidRecord>,
    /// The new drain, as its pid file records it: its pid and the build it runs.
    pub new: PidRecord,
    pub log: PathBuf,
}

/// What `athena drain-restart` ended with, short of an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartOutcome {
    Started(RestartReport),
    /// The old drain was still on its job when the wait ended. The stop stays asked for and no new drain was
    /// started.
    StillStopping(PidRecord, Duration),
}

/// Restarts the drain on a fresh build, in this order: rebuild (a failure ends it, with the running drain left
/// alone); ask the running drain to stop and wait for it to exit; check that the INSTALLED binary is not behind
/// the checkout (a landing by the old drain while this waited can leave it so) and rebuild again, now that
/// nothing lands; start the new drain; wait until its pid file shows it. A drain that cannot be seen (no pid
/// file) is not stopped: this says so and goes on.
pub fn restart(
    life: &dyn Lifecycle,
    waiting: &StopWait,
    ui: &dyn Ui,
) -> anyhow::Result<RestartOutcome> {
    use anyhow::anyhow;
    let root = life.state_root();
    let procs = life.procs();
    let first = life.rebuild().map_err(|e| {
        anyhow!("the rebuild failed, so the running drain (if any) was left alone and nothing was started:\n{e}")
    })?;
    ui.line("rebuilt and installed");
    let old = match drain_state(root, procs) {
        DrainState::Running(old) => {
            ui.line(&format!(
                "stop asked for; waiting for drain pid {} to finish its job and exit",
                old.pid
            ));
            if let StopOutcome::TimedOut(_, waited) = stop_and_wait(root, procs, Some(waiting), ui)?
            {
                return Ok(RestartOutcome::StillStopping(old, waited));
            }
            Some(old)
        }
        DrainState::NotRunning(why) => {
            ui.line(&format!("no drain to stop: {}", why.reason()));
            if why == NotRunning::Missing {
                ui.line("a drain started by a build before the pid file existed cannot be seen from here; if one is running, run `athena drain-stop` and let it finish before the next restart");
            }
            None
        }
    };
    if first == Rebuild::Installed
        && let Some(line) = life.installed_stale()
    {
        ui.line(&format!(
            "the installed build is behind the checkout ({line}); building again now that nothing lands"
        ));
        life.rebuild().map_err(|e| {
            anyhow!("the second rebuild failed, so no drain is running and none was started; fix the build, then run ./setup.sh and `athena drain`:\n{e}")
        })?;
        if let Some(line) = life.installed_stale() {
            anyhow::bail!(
                "the installed build is still behind the checkout after a second rebuild ({line}), so no drain is running and none was started; run ./setup.sh, then `athena drain`"
            );
        }
    }
    let spawned = life
        .spawn()
        .map_err(|e| anyhow!("could not start the drain, so no drain is running: {e}"))?;
    let began = std::time::Instant::now();
    loop {
        if let DrainState::Running(new) = drain_state(root, procs)
            && new.pid == spawned.pid
        {
            return Ok(RestartOutcome::Started(RestartReport {
                old,
                new,
                log: spawned.log,
            }));
        }
        if procs.start_marker(spawned.pid).is_none() {
            anyhow::bail!(
                "the new drain (pid {}) exited before it took the pid file, so no drain is running; its output in {}:\n{}",
                spawned.pid,
                spawned.log.display(),
                tail_lines(&spawned.log, 10)
            );
        }
        if began.elapsed() >= waiting.start {
            anyhow::bail!(
                "the new drain (pid {}) is running but has not taken the pid file within {}s; see {}:\n{}",
                spawned.pid,
                waiting.start.as_secs(),
                spawned.log.display(),
                tail_lines(&spawned.log, 10)
            );
        }
        std::thread::sleep(waiting.poll);
    }
}

/// The last `count` lines of a text file, or a note that it cannot be read.
fn tail_lines(file: &Path, count: usize) -> String {
    match std::fs::read_to_string(file) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            lines[lines.len().saturating_sub(count)..].join("\n")
        }
        Err(e) => format!("(cannot read {}: {e})", file.display()),
    }
}

/// The commit in the text `athena --version` prints (`athena 0.1.0 (commit abc123, dirty)`), or `None` when
/// it names none (`unknown`) or the text has no such part.
pub fn commit_in_version(text: &str) -> Option<String> {
    let rest = &text[text.find("(commit ")? + "(commit ".len()..];
    let commit = rest.split([',', ')']).next()?.trim();
    (!commit.is_empty() && commit != "unknown").then(|| commit.to_string())
}

/// The restart against the real repository: `./setup.sh`, the installed binary, and a detached drain.
#[derive(Debug)]
pub struct RealLifecycle<'a> {
    pub athena: &'a Athena,
    /// What the new drain is given: the same limits and `--json` as `athena drain`.
    pub limits: Limits,
    pub json: bool,
}

impl Lifecycle for RealLifecycle<'_> {
    fn state_root(&self) -> &Path {
        self.athena.state.root()
    }

    fn procs(&self) -> &dyn Procs {
        &RealProcs
    }

    fn rebuild(&self) -> Result<Rebuild, String> {
        run_setup(self.athena.git.dir())
    }

    fn installed_stale(&self) -> Option<String> {
        let out = std::process::Command::new(crate::wiring::installed_binary())
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let commit = commit_in_version(&String::from_utf8_lossy(&out.stdout))?;
        athena_core::status::stale_build_line(&self.athena.git, &commit, &self.athena.head().ok()?)
    }

    fn spawn(&self) -> Result<Spawned, String> {
        let log = log_file(self.state_root());
        let pid = crate::wiring::spawn_detached(
            &crate::wiring::installed_binary(),
            &reexec_args(&self.limits, self.json),
            self.athena.git.dir(),
            &log,
        )
        .map_err(|e| {
            format!(
                "could not run {}: {e}",
                crate::wiring::installed_binary().display()
            )
        })?;
        Ok(Spawned { pid, log })
    }
}
