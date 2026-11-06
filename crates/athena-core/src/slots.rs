//! The build-slot queue: a counting semaphore kept as lock files, so that every Athena command on one
//! repository (the drain, `athena go`, later a producer's shell) shares one count of cargo builds at once
//! (`docs/design/agent-and-build-slots.md` sections 3 and 4).
//!
//! A slot is an exclusive file lock on one of `slots` files, `slot-0.lock` to `slot-{N-1}.lock`, in the
//! directory [`crate::state::StateDir::slots_dir`] names. The operating system drops a file lock when its
//! holder's last handle closes, whether by [`Drop`], a panic or the death of the process. So a crashed job
//! frees its slot with no sweeper and no pid file.
//!
//! Order. A request that cannot be served at once waits as a ticket: a file under `queue/` whose name holds the
//! wall-clock time it was queued and its [`Class`], and which its owner keeps locked. Only the head ticket tries
//! the slot files, so the order is strict by class, first in first out inside a class. A request that has
//! waited `promote` moves up one class for each such interval, so a lower class is not starved behind a steady
//! stream of a higher one. A ticket is created under a temporary name, locked, and then renamed into place; a
//! ticket that is visible and whose lock is free therefore belongs to a dead process, and the next reader
//! removes it. A waiter polls every `poll` (250 ms) and needs no daemon.
//!
//! Timeout. A request waits at most `wait` ([`athena_data::config::PipelineConfig::build_wait_s`]). Past it
//! [`SlotQueue::acquire`] returns [`crate::disk::SlotWait`], which the run turns into a held job, not a failed
//! one. The caller's `stop` closure runs on every poll, so the kill switch still ends a long wait.
//!
//! Who asks. [`Class::Check`] is a check or rerun of a candidate. [`Class::Producer`] is a producer's `cargo
//! build`, `test`, `clippy` or `check`: the `cargo` wrapper first on the producer's `PATH`
//! ([`athena_agent::env::CargoShim`]) takes the slot from its own process and keeps it until cargo exits. A
//! wrapper that waits the limit out writes a wait note under the job's directory ([`write_wait_note`]), which
//! the run reads after the producer ends and turns into a held job. The landing rebuild is the one class still
//! unused (build job 7).

use crate::disk::SlotWait;
use anyhow::{Context, Result};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How often a waiting request looks at the queue again.
pub const DEFAULT_POLL: Duration = Duration::from_millis(250);

/// A request's priority class. A lower number is served first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// The landing rebuild: every ready landing waits behind it.
    Landing = 0,
    /// A check or rerun of a candidate: a job about to finish, which frees its worktree and directory.
    Check = 1,
    /// A producer's cargo command.
    Producer = 2,
}

impl Class {
    fn from_number(n: u8) -> Option<Class> {
        match n {
            0 => Some(Class::Landing),
            1 => Some(Class::Check),
            2 => Some(Class::Producer),
            _ => None,
        }
    }
}

/// The build slots of one repository. It holds only paths and limits, so any number of values over the same
/// directory (in one process or many) share the same slots.
#[derive(Debug, Clone)]
pub struct SlotQueue {
    dir: PathBuf,
    slots: usize,
    wait: Duration,
    promote: Duration,
    poll: Duration,
}

/// A held build slot. The slot is free again when this value is dropped, or when the process dies.
#[derive(Debug)]
pub struct Slot {
    _file: File,
    index: usize,
    queued: Duration,
    waited: bool,
}

impl Slot {
    /// Which slot this is, from 0: the `<n>` of `slot-<n>.lock` and of the slot's warm target directory
    /// ([`crate::state::StateDir::slot_target`]).
    pub fn index(&self) -> usize {
        self.index
    }

    /// How long the request took to get the slot, on the monotonic clock.
    pub fn queued(&self) -> Duration {
        self.queued
    }

    /// Whether the first look did not serve the request, so it waited behind others.
    pub fn waited(&self) -> bool {
        self.waited
    }
}

/// A queued request's ticket: a locked file that goes away when the request does.
#[derive(Debug)]
struct Ticket {
    name: String,
    path: PathBuf,
    _file: File,
}

impl Drop for Ticket {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// A live ticket as read from the queue directory.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Queued {
    name: String,
    queued_at: u64,
    class: Class,
}

static TICKETS: AtomicU64 = AtomicU64::new(0);

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
}

impl SlotQueue {
    /// A queue over `dir` with `slots` entries (at least 1), a longest wait of `wait` and an ageing interval of
    /// `promote`.
    pub fn new(dir: PathBuf, slots: usize, wait: Duration, promote: Duration) -> SlotQueue {
        SlotQueue {
            dir,
            slots: slots.max(1),
            wait,
            promote,
            poll: DEFAULT_POLL,
        }
    }

    /// The queue the `[drain.pipeline]` settings describe, over the state directory's slots.
    pub fn from_config(
        state: &crate::state::StateDir,
        pipeline: &athena_data::config::PipelineConfig,
    ) -> SlotQueue {
        SlotQueue::new(
            state.slots_dir(),
            pipeline.max_check_slots as usize,
            Duration::from_secs(pipeline.build_wait_s),
            Duration::from_secs(pipeline.build_age_promote_s),
        )
    }

    /// The same queue with another poll interval. Tests shorten it.
    pub fn with_poll(mut self, poll: Duration) -> SlotQueue {
        self.poll = poll;
        self
    }

    fn queue_dir(&self) -> PathBuf {
        self.dir.join("queue")
    }

    /// How many requests are waiting now. A ticket whose owner died is removed and not counted.
    pub fn waiting(&self) -> usize {
        self.live_tickets().len()
    }

    /// Takes a slot if one is free and nobody is waiting; never waits. `None` means all slots are held or a
    /// request is queued ahead.
    pub fn try_acquire(&self) -> Result<Option<Slot>> {
        if self.waiting() > 0 {
            return Ok(None);
        }
        let start = Instant::now();
        Ok(self.try_slots()?.map(|(file, index)| Slot {
            _file: file,
            index,
            queued: start.elapsed(),
            waited: false,
        }))
    }

    /// Takes a slot of the given class, waiting in the queue when none is free or others are ahead.
    ///
    /// `stop` runs before every attempt; an error from it ends the wait with that error (the kill switch).
    /// After the queue's longest wait the error is a [`SlotWait`]. The request leaves the queue on every
    /// return path.
    pub fn acquire(&self, class: Class, stop: &dyn Fn() -> Result<()>) -> Result<Slot> {
        let start = Instant::now();
        let ticket = self.enqueue(class)?;
        let mut waited = false;
        loop {
            stop()?;
            if self.is_head(&ticket.name)?
                && let Some((file, index)) = self.try_slots()?
            {
                drop(ticket);
                return Ok(Slot {
                    _file: file,
                    index,
                    queued: start.elapsed(),
                    waited,
                });
            }
            waited = true;
            let elapsed = start.elapsed();
            if elapsed >= self.wait {
                return Err(SlotWait {
                    waited_s: elapsed.as_secs(),
                    limit_s: self.wait.as_secs(),
                }
                .into());
            }
            std::thread::sleep(self.poll.min(self.wait - elapsed));
        }
    }

    /// The first slot file whose lock is free, locked, with its number.
    fn try_slots(&self) -> Result<Option<(File, usize)>> {
        std::fs::create_dir_all(&self.dir)
            .with_context(|| format!("creating {}", self.dir.display()))?;
        for n in 0..self.slots {
            let path = self.dir.join(format!("slot-{n}.lock"));
            let file = File::options()
                .create(true)
                .truncate(false)
                .write(true)
                .open(&path)
                .with_context(|| format!("opening {}", path.display()))?;
            match file.try_lock() {
                Ok(()) => return Ok(Some((file, n))),
                Err(std::fs::TryLockError::WouldBlock) => {}
                Err(std::fs::TryLockError::Error(e)) => {
                    return Err(
                        anyhow::Error::new(e).context(format!("locking {}", path.display()))
                    );
                }
            }
        }
        Ok(None)
    }

    /// Queues a request: the ticket is locked before it gets its visible name.
    fn enqueue(&self, class: Class) -> Result<Ticket> {
        let dir = self.queue_dir();
        std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        let name = format!(
            "{:020}-{}-{}-{}",
            now_nanos(),
            class as u8,
            std::process::id(),
            TICKETS.fetch_add(1, Ordering::Relaxed)
        );
        let temp = dir.join(format!(".{name}"));
        let path = dir.join(&name);
        let file = File::options()
            .create_new(true)
            .write(true)
            .open(&temp)
            .with_context(|| format!("creating {}", temp.display()))?;
        file.lock()
            .with_context(|| format!("locking {}", temp.display()))?;
        std::fs::rename(&temp, &path).with_context(|| format!("queueing {}", path.display()))?;
        Ok(Ticket {
            name,
            path,
            _file: file,
        })
    }

    /// The tickets whose owners are alive. A visible ticket with a free lock is dead and is removed.
    fn live_tickets(&self) -> Vec<Queued> {
        let Ok(entries) = std::fs::read_dir(self.queue_dir()) else {
            return Vec::new();
        };
        let mut live = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let Some(queued) = parse_ticket(&name) else {
                continue;
            };
            let Ok(file) = File::open(entry.path()) else {
                continue;
            };
            match file.try_lock_shared() {
                // Nobody holds it: the owner is gone.
                Ok(()) => {
                    let _ = std::fs::remove_file(entry.path());
                }
                Err(_) => live.push(queued),
            }
        }
        live
    }

    /// Whether the ticket `name` is first in the order: effective class, then queue time, then name.
    fn is_head(&self, name: &str) -> Result<bool> {
        let now = now_nanos();
        let promote = u64::try_from(self.promote.as_nanos())
            .unwrap_or(u64::MAX)
            .max(1);
        let mut tickets = self.live_tickets();
        tickets.sort_by_key(|t| {
            let promotions = now.saturating_sub(t.queued_at) / promote;
            let effective = (t.class as u64).saturating_sub(promotions);
            (effective, t.queued_at, t.name.clone())
        });
        match tickets.first() {
            Some(head) => Ok(head.name == name),
            None => anyhow::bail!(
                "the build-slot ticket {name} is gone from {}",
                self.queue_dir().display()
            ),
        }
    }
}

/// The time and class a ticket name holds: `<nanos>-<class>-<pid>-<n>`.
fn parse_ticket(name: &str) -> Option<Queued> {
    let mut parts = name.split('-');
    let queued_at = parts.next()?.parse().ok()?;
    let class = Class::from_number(parts.next()?.parse().ok()?)?;
    Some(Queued {
        name: name.to_string(),
        queued_at,
        class,
    })
}

/// Where the `cargo` wrapper leaves its wait note for the job whose directory is `job_dir`.
pub fn wait_note_path(job_dir: &Path) -> PathBuf {
    job_dir.join(athena_agent::env::WAIT_NOTE_FILE)
}

/// Writes the note a wrapper leaves when no build slot came free within its limit.
pub fn write_wait_note(note: &Path, wait: &SlotWait) -> Result<()> {
    let text = serde_json::to_string(wait).context("writing the build-slot wait note")?;
    std::fs::write(note, text).with_context(|| format!("writing {}", note.display()))
}

/// The wait the note at `note` records, or `None` when there is no note or it cannot be read.
pub fn read_wait_note(note: &Path) -> Option<SlotWait> {
    let text = std::fs::read_to_string(note).ok()?;
    serde_json::from_str(&text).ok()
}

/// Removes the note at `note`; no note is not an error.
pub fn clear_wait_note(note: &Path) {
    let _ = std::fs::remove_file(note);
}

/// Removes the workspace crates' output from a slot's target directory and keeps the third-party output.
///
/// It runs `cargo clean -p <member>` for every workspace member of the workspace at `workspace`, with
/// `target` as the target directory (`docs/design/agent-and-build-slots.md` section 5.3, step 3). Each job's
/// snapshot sits at another path, so workspace-crate fingerprints never match the next job's, and left in
/// place they would add one set per job to the slot. Returns the member names cleaned; a workspace with no
/// `Cargo.toml` has none and nothing runs. An error means the directory may still hold workspace output, and
/// the caller deletes it.
pub fn clean_workspace_output(workspace: &Path, target: &Path) -> Result<Vec<String>> {
    if !workspace.join("Cargo.toml").is_file() {
        return Ok(Vec::new());
    }
    let meta = std::process::Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--offline",
            "--format-version",
            "1",
        ])
        .current_dir(workspace)
        .output()
        .context("running `cargo metadata`")?;
    if !meta.status.success() {
        anyhow::bail!(
            "`cargo metadata` failed: {}",
            String::from_utf8_lossy(&meta.stderr).trim()
        );
    }
    let json: serde_json::Value =
        serde_json::from_slice(&meta.stdout).context("reading `cargo metadata`")?;
    let members: Vec<String> = json["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .filter_map(|p| p["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    if members.is_empty() {
        return Ok(members);
    }
    let mut clean = std::process::Command::new("cargo");
    clean
        .args(["clean", "--offline", "--target-dir"])
        .arg(target)
        .current_dir(workspace);
    for member in &members {
        clean.args(["-p", member]);
    }
    let out = clean.output().context("running `cargo clean`")?;
    if !out.status.success() {
        anyhow::bail!(
            "`cargo clean` failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use std::path::Path;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;

    const BOUND: Duration = Duration::from_secs(10);

    /// The names in a directory, for a test that wants to see nothing was cleaned up.
    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    fn queue(dir: &Path, slots: usize) -> SlotQueue {
        SlotQueue::new(
            dir.to_path_buf(),
            slots,
            Duration::from_secs(30),
            Duration::from_secs(600),
        )
        .with_poll(Duration::from_millis(10))
    }

    fn go() -> Result<()> {
        Ok(())
    }

    /// A slot taken without waiting, retried for a moment. Another test's child process, between its fork and
    /// its exec, can hold a copy of a lock file this test just released; the lock then frees within
    /// milliseconds, never later.
    fn take_soon(q: &SlotQueue) -> Slot {
        let start = Instant::now();
        loop {
            if let Some(slot) = q.try_acquire().unwrap() {
                return slot;
            }
            assert!(start.elapsed() < BOUND, "no slot became free");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Waits until `q` shows `n` waiting requests.
    fn wait_for_waiting(q: &SlotQueue, n: usize) {
        let start = Instant::now();
        while q.waiting() < n {
            assert!(start.elapsed() < BOUND, "no {n} waiters appeared");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn an_uncontended_request_is_served_at_once() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let slot = q.acquire(Class::Check, &go).unwrap();
        assert!(!slot.waited());
        assert!(slot.queued() < Duration::from_secs(5));
        assert_eq!(q.waiting(), 0, "the ticket is gone once the slot is held");
    }

    #[test]
    fn the_second_request_waits_then_runs() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let log = Mutex::new(Vec::new());
        let first = q.acquire(Class::Check, &go).unwrap();
        log.lock().unwrap().push("first-start");
        let second = std::thread::scope(|scope| {
            let waiter = scope.spawn(|| {
                let slot = q.acquire(Class::Check, &go).unwrap();
                log.lock().unwrap().push("second");
                slot
            });
            wait_for_waiting(&q, 1);
            // The second request is queued and has not run while the first holds the slot.
            assert_eq!(*log.lock().unwrap(), ["first-start"]);
            assert!(q.try_acquire().unwrap().is_none());
            log.lock().unwrap().push("first-end");
            drop(first);
            waiter.join().unwrap()
        });
        assert_eq!(*log.lock().unwrap(), ["first-start", "first-end", "second"]);
        assert!(second.waited());
        assert!(second.queued() > Duration::ZERO);
    }

    #[test]
    fn two_slots_admit_two_holders_and_the_third_waits() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 2);
        // Without a queue, the non-blocking call shows the count.
        let a = q.try_acquire().unwrap().expect("first slot");
        let b = q.try_acquire().unwrap().expect("second slot");
        assert!(q.try_acquire().unwrap().is_none(), "two held, none left");
        drop(a);
        let c = take_soon(&q);
        drop((b, c));

        let live = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let release = std::sync::atomic::AtomicBool::new(false);
        let waited: Vec<bool> = std::thread::scope(|scope| {
            let holders: Vec<_> = (0..3)
                .map(|_| {
                    scope.spawn(|| {
                        let slot = q.acquire(Class::Check, &go).unwrap();
                        let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                        peak.fetch_max(now, Ordering::SeqCst);
                        let waited = slot.waited();
                        let start = Instant::now();
                        while !release.load(Ordering::SeqCst) && waited_ok(start) {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        live.fetch_sub(1, Ordering::SeqCst);
                        drop(slot);
                        waited
                    })
                })
                .collect();
            // Two hold at once and the third waits in the queue.
            let start = Instant::now();
            while live.load(Ordering::SeqCst) < 2 || q.waiting() < 1 {
                assert!(start.elapsed() < BOUND, "two holders and a waiter expected");
                std::thread::sleep(Duration::from_millis(5));
            }
            assert_eq!(live.load(Ordering::SeqCst), 2);
            release.store(true, Ordering::SeqCst);
            holders.into_iter().map(|h| h.join().unwrap()).collect()
        });
        assert_eq!(peak.load(Ordering::SeqCst), 2, "never three at once");
        // The third request had to wait; one that queued behind a ticket still in line may have waited a poll
        // too, so this is a lower bound.
        assert!(waited.iter().any(|w| *w), "{waited:?}");
    }

    /// Bounds a holder's spin in the test above so a failed assertion elsewhere cannot hang the suite.
    fn waited_ok(start: Instant) -> bool {
        start.elapsed() < BOUND
    }

    /// The order in which waiters of the given classes are served, once the held slot is released.
    fn served_order(
        q: &SlotQueue,
        waiters: &[(&'static str, Class, Duration)],
    ) -> Vec<&'static str> {
        let log = Mutex::new(Vec::new());
        let held = q.acquire(Class::Check, &go).unwrap();
        std::thread::scope(|scope| {
            let mut joins = Vec::new();
            for (i, (label, class, pause)) in waiters.iter().enumerate() {
                let (log, label, class) = (&log, *label, *class);
                joins.push(scope.spawn(move || {
                    let slot = q.acquire(class, &go).unwrap();
                    log.lock().unwrap().push(label);
                    drop(slot);
                }));
                wait_for_waiting(q, i + 1);
                std::thread::sleep(*pause);
            }
            drop(held);
            for join in joins {
                join.join().unwrap();
            }
        });
        log.into_inner().unwrap()
    }

    #[test]
    fn a_later_check_overtakes_an_earlier_producer_command() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let none = Duration::ZERO;
        let order = served_order(
            &q,
            &[
                ("producer", Class::Producer, none),
                ("check", Class::Check, none),
            ],
        );
        assert_eq!(order, ["check", "producer"]);
    }

    #[test]
    fn requests_of_one_class_are_served_in_the_order_they_asked() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let pause = Duration::from_millis(5);
        let order = served_order(
            &q,
            &[
                ("one", Class::Check, pause),
                ("two", Class::Check, pause),
                ("three", Class::Check, pause),
            ],
        );
        assert_eq!(order, ["one", "two", "three"]);
    }

    #[test]
    fn a_request_that_waited_long_enough_moves_up_a_class() {
        let order_with = |promote: Duration| {
            let temp = tempfile::tempdir().unwrap();
            let q = SlotQueue::new(
                temp.path().to_path_buf(),
                1,
                Duration::from_secs(30),
                promote,
            )
            .with_poll(Duration::from_millis(10));
            // The producer command queues first and waits 200 ms before the check queues.
            served_order(
                &q,
                &[
                    ("producer", Class::Producer, Duration::from_millis(200)),
                    ("check", Class::Check, Duration::ZERO),
                ],
            )
        };
        assert_eq!(
            order_with(Duration::from_millis(50)),
            ["producer", "check"],
            "aged past a class, the earlier request goes first"
        );
        assert_eq!(
            order_with(Duration::from_secs(3600)),
            ["check", "producer"],
            "without ageing the class decides"
        );
    }

    #[test]
    fn a_ticket_left_by_a_dead_process_is_removed_and_blocks_nobody() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let dir = temp.path().join("queue");
        std::fs::create_dir_all(&dir).unwrap();
        // Older than any live ticket and not locked by anyone: what a killed waiter leaves.
        let dead = dir.join("00000000000000000001-1-4242-0");
        std::fs::write(&dead, b"").unwrap();
        assert_eq!(q.waiting(), 0);
        assert!(!dead.exists(), "the dead ticket is removed when read");
        std::fs::write(&dead, b"").unwrap();
        let slot = q.acquire(Class::Check, &go).unwrap();
        assert!(!slot.waited(), "a dead ticket ahead does not hold the line");
    }

    #[test]
    fn a_request_past_its_longest_wait_gets_a_slot_wait_and_leaves_the_queue() {
        let temp = tempfile::tempdir().unwrap();
        let q = SlotQueue::new(
            temp.path().to_path_buf(),
            1,
            Duration::from_millis(150),
            Duration::from_secs(600),
        )
        .with_poll(Duration::from_millis(10));
        let _held = q.acquire(Class::Check, &go).unwrap();
        let started = Instant::now();
        let error = q.acquire(Class::Check, &go).unwrap_err();
        assert!(started.elapsed() >= Duration::from_millis(150));
        let wait = error.downcast_ref::<SlotWait>().expect("a SlotWait");
        assert_eq!(wait.limit_s, 0, "150 ms is under a whole second");
        assert_eq!(q.waiting(), 0, "the timed-out request left the queue");
    }

    #[test]
    fn the_stop_closure_ends_a_wait() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let _held = q.acquire(Class::Check, &go).unwrap();
        let calls = AtomicUsize::new(0);
        let error = q
            .acquire(Class::Check, &|| {
                if calls.fetch_add(1, Ordering::SeqCst) >= 2 {
                    anyhow::bail!("stopped by the kill switch");
                }
                Ok(())
            })
            .unwrap_err();
        assert!(error.to_string().contains("kill switch"), "{error}");
        assert_eq!(q.waiting(), 0, "a stopped request leaves the queue");
    }

    /// Holds the only slot and leaves it held until the process is killed. It runs only as the child of
    /// `a_killed_holder_frees_its_slot`; in a normal test run it returns at once.
    #[test]
    fn slot_holder_child() {
        let Ok(dir) = std::env::var("ATHENA_SLOTS_TEST_DIR") else {
            return;
        };
        let q = SlotQueue::new(
            PathBuf::from(dir),
            1,
            Duration::from_secs(30),
            Duration::from_secs(600),
        );
        let _slot = q
            .acquire(Class::Check, &go)
            .expect("the child takes the slot");
        println!("SLOT-HELD-BY-CHILD");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::thread::sleep(Duration::from_secs(120));
    }

    #[test]
    fn a_killed_holder_frees_its_slot() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "slots::tests::slot_holder_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("ATHENA_SLOTS_TEST_DIR", temp.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut held = false;
        for line in std::io::BufReader::new(stdout)
            .lines()
            .map_while(|l| l.ok())
        {
            if line.contains("SLOT-HELD-BY-CHILD") {
                held = true;
                break;
            }
        }
        assert!(held, "the child never reported the slot held");
        // The child is alive and holds the slot: a second request cannot have it.
        assert!(q.try_acquire().unwrap().is_none());
        let before = files_in(temp.path());

        child.kill().unwrap();
        child.wait().unwrap();

        // No cleanup call and no file removed: the lock died with the process.
        assert_eq!(files_in(temp.path()), before);
        let slot = q
            .acquire(Class::Check, &go)
            .expect("the dead holder's slot is free");
        assert!(slot.queued() < BOUND);
    }

    #[test]
    fn a_panicking_holder_frees_its_slot() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _slot = q.acquire(Class::Check, &go).unwrap();
            assert!(q.try_acquire().unwrap().is_none(), "held while it runs");
            panic!("the holder panics with the slot held");
        }));
        assert!(outcome.is_err());
        take_soon(&q);
    }

    #[test]
    fn a_holder_that_returns_an_error_frees_its_slot() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 1);
        let run = || -> Result<()> {
            let _slot = q.acquire(Class::Check, &go)?;
            anyhow::bail!("the check failed to start");
        };
        assert!(run().is_err());
        take_soon(&q);
    }

    #[test]
    fn a_slot_knows_its_number_and_a_freed_number_is_taken_again() {
        let temp = tempfile::tempdir().unwrap();
        let q = queue(temp.path(), 2);
        let a = q.try_acquire().unwrap().unwrap();
        let b = q.try_acquire().unwrap().unwrap();
        assert_eq!((a.index(), b.index()), (0, 1));
        drop(a);
        let again = take_soon(&q);
        assert_eq!(
            again.index(),
            0,
            "the same slot, so the same warm directory"
        );
    }

    /// Every file path under `dir` whose name contains `needle`.
    fn files_named(dir: &Path, needle: &str) -> Vec<PathBuf> {
        let mut found = Vec::new();
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if entry.file_name().to_string_lossy().contains(needle) {
                found.push(path.clone());
            }
            if path.is_dir() {
                found.extend(files_named(&path, needle));
            }
        }
        found
    }

    #[test]
    fn cleaning_a_slot_drops_workspace_crate_output_and_keeps_third_party_output() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("ws");
        std::fs::create_dir_all(workspace.join("alpha/src")).unwrap();
        std::fs::write(
            workspace.join("Cargo.toml"),
            "[workspace]\nmembers = [\"alpha\"]\nresolver = \"2\"\n",
        )
        .unwrap();
        std::fs::write(
            workspace.join("alpha/Cargo.toml"),
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(workspace.join("alpha/src/main.rs"), "fn main() {}\n").unwrap();
        let target = temp.path().join("slot/target");

        // A real build of the workspace crate into the slot directory.
        let built = std::process::Command::new("cargo")
            .args(["build", "--offline"])
            .current_dir(&workspace)
            .env("CARGO_TARGET_DIR", &target)
            .env_remove("RUSTC_WRAPPER")
            .env_remove("CARGO_BUILD_RUSTC_WRAPPER")
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        // Output of a third-party crate, as an earlier build of a dependency left it.
        let deps = target.join("debug/deps");
        std::fs::write(deps.join("libserde-0123abcd.rlib"), b"compiled").unwrap();
        std::fs::create_dir_all(target.join("debug/.fingerprint/serde-0123abcd")).unwrap();
        std::fs::write(
            target.join("debug/.fingerprint/serde-0123abcd/lib-serde"),
            b"fingerprint",
        )
        .unwrap();
        assert!(
            !files_named(&target, "alpha").is_empty(),
            "the build left workspace-crate output"
        );

        let cleaned = clean_workspace_output(&workspace, &target).unwrap();
        assert_eq!(cleaned, ["alpha"]);
        assert_eq!(
            files_named(&target, "alpha"),
            Vec::<PathBuf>::new(),
            "no workspace-crate output is left"
        );
        assert!(deps.join("libserde-0123abcd.rlib").is_file());
        assert!(
            target
                .join("debug/.fingerprint/serde-0123abcd/lib-serde")
                .is_file()
        );
    }

    #[test]
    fn cleaning_a_directory_that_is_not_a_cargo_workspace_runs_nothing() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("kept"), b"x").unwrap();
        assert!(
            clean_workspace_output(temp.path(), &target)
                .unwrap()
                .is_empty()
        );
        assert!(target.join("kept").is_file());
    }

    #[test]
    fn a_wait_note_round_trips_and_is_cleared() {
        let temp = tempfile::tempdir().unwrap();
        let note = wait_note_path(temp.path());
        assert_eq!(note, temp.path().join("slot-wait.json"));
        assert_eq!(read_wait_note(&note), None, "no note yet");
        let wait = SlotWait {
            waited_s: 300,
            limit_s: 300,
        };
        write_wait_note(&note, &wait).unwrap();
        assert_eq!(read_wait_note(&note), Some(wait));
        clear_wait_note(&note);
        assert_eq!(read_wait_note(&note), None);
        clear_wait_note(&note);
        // A note that is not a wait reads as no note.
        std::fs::write(&note, b"not json").unwrap();
        assert_eq!(read_wait_note(&note), None);
    }

    #[test]
    fn the_queue_is_made_from_the_pipeline_settings() {
        let temp = tempfile::tempdir().unwrap();
        let state = crate::state::StateDir::new(temp.path().to_path_buf());
        let pipeline = athena_data::config::PipelineConfig {
            max_check_slots: 2,
            ..Default::default()
        };
        let q = SlotQueue::from_config(&state, &pipeline);
        let _a = q.try_acquire().unwrap().unwrap();
        let _b = q.try_acquire().unwrap().unwrap();
        assert!(q.try_acquire().unwrap().is_none());
        assert!(temp.path().join("slots").join("slot-1.lock").exists());
    }
}
