# Job 20261105-205932-ppfv: verify "Add drain pid file with status, stop-and-wait, restart, stale-build"

## Request (verbatim)

> Drain stop, wait, restart and resume are hand-written shell commands; self-matching kills and unverified resumes recur
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261105-180248-jjk8 (key drain-stop-restart-resume-done-by-hand-no-tracked-command), queued at 2026-11-05T18:02:48.184724588Z by the steward role.
> Rung of the requirements ladder: design-decisions.
> 
> ## Source
> 
> path records/failure-classes/2026-11/20261102-171949-nq13.json
> ```
> {
>   "schema": "athena.failure-class/1",
>   "id": "20261102-171949-nq13",
>   "at": "2026-11-02T17:19:49.727982741Z",
>   "name": "hand-wait-command-process-check-matches-itself-queue-idles",
>   "description": "A hand-written wait or watch command finds a running process by name in text that contains that name, so it matches itself or never fires, and the queue sits idle until someone notices.",
>   "status": "proposed",
>   "proposed_by": "20261102-171935-b1yf"
> }
> ```
> 
> ## Evidence
> 
> ### 1. path records/interventions/2026-11/20261102-171911-5dtp.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261102-171911-5dtp",
>   "at": "2026-11-02T17:19:11.464079254Z",
>   "what": "Mistake: Athena sat idle for about an hour: my wait-for-the-batch-to-end command matched its own text, then its helper script was never written, so neither the approved home page job nor the next batch started",
>   "why": "I check for running processes by name in commands whose own text contains that name; now the script is written separately and the check matches only the installed binary's path",
>   "class_of_work": "other",
>   "category": "mistake",
>   "job": null,
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 2. path records/interventions/2026-11/20261104-224617-2178.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261104-224617-2178",
>   "at": "2026-11-04T22:46:17.82257434Z",
>   "what": "Mistake: my pkill pattern matched its own command again (third time), killing my shell before the rebuild step, so a batch started on a build that lacked the just-landed pass-over fix. Stopped it before it dispatched anything, rebuilt, restarted.",
>   "why": "I used pkill -f with a pattern that appears in the command itself",
>   "class_of_work": "build",
>   "category": "mistake",
>   "job": null,
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 3. path records/interventions/2026-11/20261105-003746-vpcr.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261105-003746-vpcr",
>   "at": "2026-11-05T00:37:46.063970162Z",
>   "what": "Mistake: after the container restart my resume script had lost its execute permission, so starting it failed silently and nothing ran for 30 minutes; I then stopped the resumed job early because I misread its log as starting over. I did not confirm the process was running, and did not read the resume line before acting.",
>   "why": "I launched a detached script without checking it started, then acted on a misread log",
>   "class_of_work": "other",
>   "category": "mistake",
>   "job": null,
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 4. path records/introspections/2026-11/20261104-224624-kcx2.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261104-224624-kcx2",
>   "at": "2026-11-04T22:46:24.998009164Z",
>   "trigger": "intervention",
>   "job": null,
>   "intervention": "20261104-224617-2178",
>   "class_of_work": "build",
>   "category": "mistake",
>   "what_happened": "The person running Athena used `pkill -f` with a pattern that also appeared in the text of their own command line. The pattern matched and killed their own shell before the rebuild step ran. A queue batch then started on an old build that lacked the pass-over fix that had just landed. They stopped it before it dispatched any work, rebuilt, and restarted. This was the third time this self-matching mistake happened. No job was involved.",
>   "failure_class": "20261102-171949-nq13",
>   "new_class": false,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "Stopping, waiting on and restarting drain processes is done with ad-hoc shell commands that find processes by matching text, and that text also appears in the command doing the matching. Athena gives no tracked way to stop, wait for or restart a drain: no pid file, no stop command, no single restart command that rebuilds and then starts. Each person has to rewrite the process matching by hand, and the same self-match recurs. Nothing checks that a restarted drain runs the build that was just landed.",
>   "prevention_kind": "design-change",
>   "prevention": "Add an Athena command, for example `athena drain stop|restart`, that finds the drain through a recorded pid or lock file instead of matching command text. The restart should rebuild or reinstall first, then start the drain. At start, the drain should refuse to run if its binary is older than the landed code. This replaces hand-written pkill, pgrep and wait commands for the whole class, and it also stops a batch from starting on a stale build.",
>   "proposed_request": "Add a drain lifecycle command to Athena: `athena drain stop`, `athena drain status` and `athena drain restart`. The drain should write a pid/lock file under the state directory when it starts, and the commands should find and signal the drain only through that file, never by matching command-line text. `stop` should let the running job finish, or stop at a job boundary, and then exit. `restart` should run the setup/rebuild step first, wait for the old drain to exit, then start a new drain, and it should report the build it started. At start, the drain should refuse to dispatch if its binary is older than the current landed code, and print why. Acceptance: tests show that stop and status work with a command line that contains the drain's own name; a stale-binary start is refused with a clear message; restart rebuilds before starting; and a missing or stale pid file is handled without killing unrelated processes.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0934301,
>     "input_tokens": 2,
>     "output_tokens": 951
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 5. path records/introspections/2026-11/20261105-003804-5qdm.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261105-003804-5qdm",
>   "at": "2026-11-05T00:38:04.683411521Z",
>   "trigger": "intervention",
>   "job": null,
>   "intervention": "20261105-003746-vpcr",
>   "class_of_work": "other",
>   "category": "mistake",
>   "what_happened": "After a container restart, the person running Athena's resume script had lost its execute permission. They launched it detached and did not check it had started. It failed silently and nothing ran for about 30 minutes. They then stopped the resumed job early because they misread its log as starting over, without reading the resume line. No work was judged bad.",
>   "failure_class": "20261105-003814-52qd",
>   "new_class": true,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "Athena gives the person running Athena no tracked way to resume or launch a job that shows whether it is alive and whether it is resuming or restarting. The person relies on ad hoc detached scripts and raw logs. Nothing checks that the process started or says what a retry will do. Safe use depends on the person remembering to verify state before acting. This is the same cause as the other hand-command mistakes where Athena does not guard the person's view of state.",
>   "prevention_kind": "design-change",
>   "prevention": "Add an Athena resume command that runs the job as a tracked process. It should refuse to report success until a live process and a first progress record exist. It should print the mode (resume or rebuild) on its first line. It should warn when an approved or interrupted job has had no running process for more than a few minutes. Hand scripts would then not be needed. As a default, the person running Athena confirms a detached launch with a process check and reads the resume line before stopping anything.",
>   "proposed_request": "Add a tracked resume path for interrupted jobs. Outcome: after a container restart, one Athena command resumes an interrupted job without a hand-written script. Acceptance:\n- The command prints on its first line whether it will resume or rebuild, and the expected cost.\n- It exits non-zero with a clear message if the job process does not start or no progress record appears within a set time.\n- `athena status` shows a warning when a job is approved or interrupted and no process has run for more than 10 minutes.\n- A test starts the command with a process that fails to launch and checks that the failure is reported.\n- A test checks that a resumed job's log states that it is resuming.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.099634,
>     "input_tokens": 2,
>     "output_tokens": 967
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [
>     "20261102-171935-b1yf",
>     "20261102-005313-xm7b",
>     "20261101-102346-0dyj"
>   ],
>   "from_jev": false
> }
> ```
> 
> ### 6. path records/failure-classes/2026-11/20261105-003814-52qd.json
> ```
> {
>   "schema": "athena.failure-class/1",
>   "id": "20261105-003814-52qd",
>   "at": "2026-11-05T00:38:14.467815248Z",
>   "name": "detached-hand-launch-unverified-and-log-misread-no-athena-resume-command",
>   "description": "The person running Athena starts a detached resume or run script by hand, does not confirm the process is running, and acts on a misread log, because Athena offers no tracked resume command that reports its mode and liveness.",
>   "status": "proposed",
>   "proposed_by": "20261105-003804-5qdm"
> }
> ```
> 
> ### 7. path crates/athena-cli/src/drain.rs
> ```
> //! `athena drain`: works the queue itself (requests the steward queued and requests deferred with `athena ask --later`), within a budget the user sets. For each queued request
> //! it reads and runs the job, lands it through the pull-request path (`athena land --pr`), and only then
> //! rates the live page as its audience would. It stops at the first of the reasons in [`Stop`].
> //!
> //! - **Planning comes first.** The open requests are fleshed out and planned together
> //!   ([`athena_core::plan`]): goal, requirements-ladder rung, acceptance, open questions, dependencies and
> //!   touched paths. The loop works the items in plan order (a dependent after what it
> //!   needs, then priority from a `Priority:` line in the request, then higher rung, then age as the tiebreak),
> //!   not oldest first. An item whose plan lacks a goal, a rung or acceptance is
> //!   refused and reported with a reason ([`Report::refused`]). The plan also groups items that could run in
> //!   parallel ([`Report::plan_groups`]); [`drain_concurrent`] runs the items of one group side by side, up to
> //!   `[parallel] max_concurrency` (default 1, which is [`drain`]'s one job at a time). The pass is
> //!   deterministic code and calls no model of its own; nothing is defaulted into a plan.
> //! - **The drain follows the plan as it stands.** The drain does not keep the plan it built at its start. After
> //!   each item (each group, under [`drain_concurrent`]) it reads the queue again and builds the plan again
> //!   through the same function `athena plan` uses (`Drain::prepare`, which calls
> //!   [`athena_core::plan::Plan::build_with`]), so a priority record, a job end or a request queued meanwhile
> //!   changes what is worked next. [`batches_for`] only filters that plan's groups: it drops what this run has
> //!   already tried and never reorders. Memory kept across these plans: the jobs the drain approved, the
> //!   readings it made, and the items it refused to read, so nothing is approved, read or paid for twice.
> //!   *Passed-over log line.* An item that is planned but not worked gets one line, `plan       <id> passed
> //!   over: <reason>`, once per run: refused, held by a failed dependency, in a loop of dependencies, not
> //!   ready (needs accepting), held for disk or budget, skipped, over the budget, over the job limit, or not
> //!   reached because the run ended first (with the path it shares with the last group when that is why it
> //!   waited).
> //! - **A paused line is passed over, not failed.** [`Steps::paused`] says which queued items are on a line the
> //!   scope guard has paused. `Drain::prepare` asks once for the whole queue, before it reads anything: such an
> //!   item is not read, is held back with whatever depends on it (so `athena plan` lists it as `held`), and its
> //!   one passed-over line, which names the line and how to lift the pause, is printed as soon as the plan is
> //!   made. [`Drain::attempts`] asks again before each attempt ([`ItemEnd::Paused`]), and once more after the
> //!   last failed attempt, for a line the item's own failed attempt has just paused. Neither is a failed
> //!   attempt: an item whose second failure paused its line is passed over with the pause reason, and the
> //!   batch goes on to the next queued item.
> //! - **A flagged line is passed over after two failures.** A line the scope guard has only flagged (T1, T3 or
> //!   T5, not a pause) is still dispatched, so its item gets both attempts. [`Steps::flagged`] is asked only
> //!   after the last failed attempt, when no pause was found ([`ItemEnd::Flagged`]): the item is then passed
> //!   over for the rest of the batch with the flag signals as the reason, and the batch goes on to the next
> //!   queued item. Nothing is written.
> //! - **An item that fails twice is passed over, not a reason to stop.** An item whose line is neither flagged
> //!   nor paused and whose second attempt fails ([`ItemEnd::Failed`]) is passed over for the rest of the batch
> //!   with the reason `failed twice; left for the scope guard and Hestia: <last reason>`, and the batch goes
> //!   on to the next queued item. Both jobs stay in the records, so the scope guard (which pauses or flags a line that
> //!   keeps failing) and Hestia (which lists repeated failures) see the failure as before; the drain adds no
> //!   verdict of its own. The one exception is not about the item: when the closing reason of that second
> //!   failure says the environment is at fault ([`athena_core::disk::StopCause`]: the producer's login
> //!   expired, a usage limit was reached, or free disk space is low), every later item would fail the same
> //!   way, so the run also ends with [`Stop::Environment`]. Before this rule an item that failed twice with no
> //!   pause or flag ended the whole batch (a 4 Nov batch with $43 and 3 job slots left stopped on one such
> //!   item).
> //! - **Concurrency.** A group is worked by up to `max_concurrency` workers; the next group starts only when
> //!   the group before it is done, so an item that depends on another starts after it has finished. Landing
> //!   and rating go one at a time (a lock), and every model call's cap is held against the budget while it is
> //!   in flight, so jobs side by side cannot overspend it. The first reason to stop ends the run: no worker
> //!   starts another job, jobs already started are finished (a failure or a refusal stops no sibling that is
> //!   in flight), and the first reason found is the one reported. A worker that panics is not isolated: the
> //!   drain ends with the panic once the other workers have finished.
> //! - **Read-ahead.** When [`Steps::read_ahead`] is above 0, with one job at a time as with several,
> //!   [`drain_concurrent`] also reads, and for a hard job plans ([`Steps::plan_ahead`]), the next items up to
> //!   `read_ahead` of them ahead of the batch that is running (`docs/design/async-drain.md` section 4). It
> //!   reads an item ahead only when the paths the plan says it touches are disjoint from every job of the
> //!   runni
> [clipped: 243841 more characters]
> ```

## Reading (accepted by the user)

Replace hand-written pkill/pgrep/wait with tracked commands: `athena drain` writes a pid/lock file in the state directory, and `athena drain-status` and `athena drain-restart` find the drain only through it. The existing `drain-stop` gains a wait for exit. Drain refuses to start on a binary older than landed code. Audience: the person running Athena.

Assumptions:
- Code survey: crates/athena-cli/src/main.rs (Command::Drain at ~1182 removes a stale stop file, then runs drain_concurrent and follow_restart; Command::DrainStop at ~1234 only writes the stop file; warn_if_stale at 574 only prints a line from athena.stale_build()), crates/athena-cli/src/drain.rs (stop_file at ~970 is state_root/drain.stop; reinstall_note and Stop::RebuildFailed handle rebuild), crates/athena-cli/src/wiring.rs (installed_binary, open).
- Existing code: extends crates/athena-cli/src/drain.rs stop_file and the drain-stop command, and reuses Athena::stale_build and the wiring installed_binary / reexec path; no new record type or data file under crates/, so no check_record_file or AnyRecord change is needed.
- The pid file lives next to drain.stop in the state directory. It holds the pid and a process start marker, so a recycled pid is not signalled. Status and stop never match command-line text.
- Stale-binary refusal reuses the existing stale_build detection, promoted from a warning to a refusal at drain start, with a clear message. This new refusal affects only drain starts on a stale binary: expected 5% of newly refused readings.
- restart runs the setup.sh rebuild step first, waits for the old drain to exit, starts the new one, and prints the build it started. The resume-command proposal (class 20261105-003814-52qd) is a separate item and not done here.
- Existing `athena drain-stop` keeps its name and behavior (finish the job in progress) and gains an optional wait for the pid to exit; new subcommand names follow the existing hyphenated style.
- The producer must check whether the `athena --help` text or docs/ and README mention drain-stop and update them for the new commands; CHANGELOG.md gets an entry.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli`: a test starts a fake drain pid file, and `drain-status` and `drain-stop` find it correctly when the surrounding command line contains the word `drain`; they never match command text.
2. [behavior] Run `cargo test -p athena-cli`: a missing pid file, or one naming a dead or recycled pid, is reported as not running by status and stop, and no unrelated process is signalled.
3. [behavior] Run `cargo test -p athena-cli`: a drain start with a binary older than the landed code is refused with a message naming why, and dispatches no job.
4. [behavior] Run `cargo test -p athena-cli`: restart performs the rebuild step before starting the new drain, waits for the old drain to exit, and prints the build it started; a failed rebuild means no drain starts.
5. [behavior] Run `cargo test -p athena-cli`: a drain writes the pid file at start and removes it on exit; a second drain start with a live pid file is refused.
6. [behavior] The change refuses more drain starts (stale binary): expected 5% of newly refused readings.
7. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
8. [content] Grep of crates/athena-cli/src/main.rs shows the new drain-status and drain-restart subcommands with help text telling the person running Athena to use them instead of pkill/pgrep; no pgrep/pkill-style command-text matching is added in the code.
9. [content] CHANGELOG.md has an entry for the drain lifecycle commands and stale-binary refusal, and any docs naming drain-stop describe the new commands.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- The tracked `athena resume` command and the `athena status` no-process warning (failure class 20261105-003814-52qd)
- Changes to athena.toml, defaults.toml, setup.sh or workflow files
- Changes to records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each new command and the refusal; clippy and fmt criteria.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust in athena-cli.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests exercising stop/status with self-containing command lines, stale pid, stale binary.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a pid file and refusal in code, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Removes text-matching for the whole class, not one instance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Resume command left out as a separate change.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Help text and CHANGELOG written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in drain.rs's tests module over the production `drain_state`, `display::drain_status` and `stop_and_wait`, with real children and a tempdir state root. The 'drain' child is a `sleep` whose argv[0] is `athena drain --budget 5` (safe `CommandExt::arg0`). A decoy `sleep` also has `athena drain` in its argv and no pid file. The test writes `drain.pid` for the drain child through the production `write_record` with the marker from `RealProcs.start_marker(child.id())`. Run: `cargo test -p athena-cli --lib status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain`. Observes: `drain_state` returns Running with the drain child's pid and never the decoy. The printed status holds that pid, the build and 'running'. `stop_and_wait` with no wait returns Asked and creates `drain.stop`. With a wait and a child that exits once the stop file appears, it returns Exited. The decoy is still alive at the end. No code path reads command text.
  Fails without the behavior: A text-matching implementation reports the decoy (or none) as running. With no pid-file lookup, status says 'not running' for the live child and the Running assertion fails. With no wait, the Exited assertion fails.
  Test seam: Entry points `drain::drain_state`, `drain::stop_and_wait`, `display::drain_status`. The injection point is the `Procs` trait; the test passes `&RealProcs` over real child processes, so there is no fake. `Procs`, `RealProcs`, `write_record`, `drain_state`, `stop_and_wait` are added in plan steps 1-3, `display::drain_status` in step 6. `drain::stop_file` (drain.rs:970) exists.
- Criterion 2: Unit test of `drain_state` and `stop_and_wait` over: no file; unreadable JSON; a file naming a child killed and reaped (true marker saved while alive); a file naming a killed but unreaped child (zombie); a file naming a live unrelated `sleep` child with a wrong marker (recycled pid). Run: `cargo test -p athena-cli --lib a_missing_dead_or_recycled_pid_file_is_not_running_and_signals_nothing`. Observes: Each case returns NotRunning with its own reason (missing, unreadable, dead, recycled). The status printer says 'no drain is running' and the exit-code mapping function returns 2. `stop_and_wait` returns NotRunning and still writes `drain.stop`. The recycled-pid child is still alive afterward (`try_wait()` is None), so nothing signalled it.
  Fails without the behavior: A pid-only check (`process_alive`) reports the recycled and zombie cases as running. With no unreadable-file guard, the garbage file is treated as running or panics.
  Test seam: `drain::drain_state`, `drain::stop_and_wait`, `RealProcs`, real children, tempdir state root. Added in plan steps 1-3; no fake.
- Criterion 3: Unit test calling the production `drain::guarded(state_root, &RealProcs, build, &stale, || drain::drain(&stub, limits, &Kept::default()))`. `stale` is a closure returning `Some("stale build: this athena was built from aaaaaaaaaaaa, and 3 file(s) under crates/ ... changed by bbbbbbbbbbbb; run ./setup.sh")`. `stub` is the existing `Stub` steps in drain.rs's tests module (drain.rs:3968), which logs every call. A control case passes a closure returning None. Run: `cargo test -p athena-cli --lib a_stale_binary_start_is_refused_and_dispatches_no_job`. Observes: With the stale closure `guarded` returns Err containing 'refusing to start' and the stale line (both commit ids and 'crates/'). The `Stub` call log is empty (no read, run or land), `drain.pid` does not exist, and a pre-existing `drain.stop` is untouched. In the control case the closure runs, the log is non-empty and the pid file is gone afterward. The stale-build detection itself (`stale_build_line`) is unchanged and covered by tests/drain.rs `status_prints_the_stale_build_line_after_a_crates_change_and_none_after_docs_or_records_changes` (line 1993).
  Fails without the behavior: Without the refusal in `guarded`, the closure runs, the Stub log is non-empty and the result is Ok, so the Err and empty-log assertions fail.
  Test seam: `drain::guarded` is added in plan step 4; the stale answer is injected as a closure parameter, and only the yes/no answer is canned, the refusal logic is in `guarded`. `Stub`, `Kept` and `drain::drain` exist (drain.rs:3968, 4201, 2797). main.rs passes `&|| athena.stale_build()` in step 5.
- Criterion 4: Unit tests calling the production `drain::restart(state_root, &life, waiting, &Kept)` with a `Lifecycle` double `Rec`. `Rec.rebuild` appends 'rebuild' to a shared log and returns a scripted Ok/Err. `Rec.spawn` appends 'spawn', starts a real `sleep` child and writes its pid file through the production `write_record` with build 'athena 0.0.0 (commit abcdef123456)'. `Rec.installed_stale` returns Some(line) while a fake HEAD flag is ahead of the count of successful rebuilds. The old drain is a real `sh -c 'while [ ! -e <stop file> ]; do sleep 0.05; done'` child with a pid file, so it exits only when `restart` creates the stop file; a watcher thread reaps it, logs 'old exited', and (in the race case) advances the fake HEAD, standing in for a landing. Cases: (a) happy path; (b) failed first rebuild; (c) the old drain advances HEAD while restart waits; (d) the second rebuild fails; (e) the old drain never exits and `--timeout` is short; (f) no pid file; (g) the spawned child exits at once. Run: `cargo test -p athena-cli --lib restart_`. Observes: (a) log is exactly rebuild, old exited, spawn; the report names old pid, new pid and build and `display::drain_restart` prints the build. (b) rebuild Err: restart returns Err carrying the build output, the log is only 'rebuild', no stop file, the old child is alive, no spawn. (c) log is rebuild, old exited, rebuild, spawn: a second rebuild runs before spawn. (d) log is rebuild, old exited, rebuild with no spawn, and the Err says no drain is running and to run ./setup.sh. (e) Err says the stop is still asked for, no new drain was started, and to run `athena drain-restart` again; `drain.stop` still exists; log has no second rebuild and no spawn. (f) the Kept lines contain the notice about drains from builds without a pid file, no `drain.stop` is written, and spawn is logged. (g) Err names `drain.log`. Also a pure-function test of `commit_in_version` on `athena 0.1.0 (commit abc123def456)` and `(commit unknown, dirty)`.
  Fails without the behavior: If restart spawned before the old drain exited, 'spawn' would precede 'old exited'. If it skipped the rebuild, 'rebuild' would be missing. If a failed rebuild did not abort, 'spawn' would appear or the stop file would exist. Without the post-exit freshness check, case (c) has no second 'rebuild'. Without the timeout message, (e) fails its text assertions.
  Test seam: `drain::restart` and the `Lifecycle` trait are added in plan step 3 (`installed_stale` included). The double supplies only the rebuild result, the stale answer, and the process standing in for the new binary, and it uses the production pid-file writer. Ordering, waiting, the re-check, the timeout and verification are all in `restart`. `RealLifecycle` (which calls `run_setup` at drain.rs:3317, runs the installed `--version` and `wiring::spawn_detached`) is not used in this test; `spawn_detached` is added in step 5. `Kept` exists (drain.rs:4201).
- Criterion 5: Unit test of `PidGuard::acquire` and `drain::guarded` with a tempdir state root. Cases: (1) read `drain.pid` inside the closure; (2) a second `guarded` while a pid file names a live `sleep` child (real pid and marker), with a `drain.stop` present; (3) a file naming the test's own pid and marker (exec carry-over) with a `drain.stop` present; (4) a closure that panics under `catch_unwind`; (5) a leftover file naming a dead pid, with a `drain.stop` present; (6) no file, with a `drain.stop` present. Run: `cargo test -p athena-cli --lib the_pid_file_is_written_at_start_removed_on_exit_and_a_second_live_start_is_refused`. Observes: (1) During the closure the file exists with the test's pid and the build string; after `guarded` returns it is gone. (2) Err 'a drain is already running (pid N)', the closure did not run, the first drain's `drain.stop` still exists. (3) accepted, and `drain.stop` still exists inside the closure (the carried-over stop is kept). (4) the file is gone after the panic. (5) the leftover is replaced and `drain.stop` is removed. (6) `drain.stop` is removed.
  Fails without the behavior: Without the write, the in-closure read fails. Without the Drop removal, the after-exit check fails. Without the live check, the second start runs its closure. If `guarded` always removed the stop file, case (3) or (2) fails.
  Test seam: `drain::guarded`, `PidGuard::acquire` and `Drop` are added in plan steps 2 and 4. `RealProcs` with real children is injected as `&dyn Procs`.
- Criterion 6: The same unit test as criterion 3. Its control half injects a closure that returns None, as a docs-only or records-only change after the build gives (`stale_build_line`, status.rs:35-58, returns None unless BUILD_PATHS changed), and shows the guard runs the closure. The 5% figure is an estimate carried over from the reading and is not measured; the CHANGELOG entry says so. Run: `cargo test -p athena-cli --lib a_stale_binary_start_is_refused_and_dispatches_no_job`. Observes: A start with a stale answer is refused and a start with none is not, so the new refusals are limited to genuinely stale builds. The test does not show the 5% rate.
  Fails without the behavior: If the guard refused every start or ignored the closure's answer, the control assertion fails.
  Test seam: Same as criterion 3: `drain::guarded` (plan step 4) with the stale closure injected.
- Criterion 7: Run the workspace lint with warnings denied, then the format check. The workspace denies `unsafe_code` (athena-cli Cargo.toml:34-35), so new code uses only safe APIs (`CommandExt::process_group` and `arg0`, /proc reads, `ps` by pid, `<installed> --version`). Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Exit 0 with no warnings. The verifier then runs `cargo fmt --all --check`, which prints no diff and exits 0.
  Fails without the behavior: A new `unsafe` block, an unused helper (a `Procs` or `Lifecycle` method or `pid_file` with no caller), a needless clone, or unformatted code makes the command exit non-zero.
  Test seam: The whole workspace compiled under the workspace `[lints]` configuration; no fake or fixture. The seam exists today; the code from plan steps 1-6 and its tests are what is compiled.
- Criterion 8: Unit test in main.rs's tests module (`Cli::command().render_help()` is used at main.rs:1685). It renders help for the top level and for `drain-status`, `drain-restart` and `drain-stop`. A second assertion takes `include_str!` of `drain.rs`, `main.rs` and `wiring.rs`, keeps the text before `#[cfg(test)]`, and asserts none contains `Command::new("pgrep")`, `Command::new("pkill")`, `Command::new("pidof")` or a `/cmdline` read. The needles are built with `format!` so the test file does not contain them. It also asserts that every `Command::new("ps")` argument list contains `-p`. Run: `cargo test -p athena-cli --bin athena the_new_commands_are_in_the_help_and_the_code_does_no_text_matching`. Observes: The top-level help lists `drain-status` and `drain-restart`. Each command's help tells the person to use it instead of pkill or pgrep; `drain-stop --help` mentions `--wait`; `drain-restart --help` says the new drain runs in the background and logs to drain.log. No command-text matching appears in the three files.
  Fails without the behavior: With the subcommands missing or their help lacking the pkill/pgrep wording, the help assertions fail. If a pgrep, pkill, pidof or /cmdline read were added, the scan fails.
  Test seam: `Cli::command()` in main.rs (used at main.rs:504). The clap variants are added in plan step 5. The scan reads source text with `include_str!`; nothing is injected.
- Criterion 9: Unit test in main.rs's tests module, which already uses `include_str!("../../../athena.toml")` (main.rs:1716), so repository-relative includes work. It reads `CHANGELOG.md` and `README.md`, cuts the CHANGELOG at its first released-version heading to get the Unreleased section, and checks both texts. Run: `cargo test -p athena-cli --bin athena the_changelog_and_readme_describe_the_drain_lifecycle_commands`. Observes: The Unreleased section names `drain-status`, `drain-restart`, `drain-stop --wait`, the pid file, the stale-binary refusal and the one-time transition for a drain started without a pid file, and labels the 5% figure an estimate. The README 'Stopping a drain' section mentions `--wait`, `drain-status` and `drain-restart`, and a README paragraph states how the pid file, the stale-build refusal, the background run (`drain.log`) and the transition work.
  Fails without the behavior: If the CHANGELOG entry or README text were missing or omitted a command name, the substring assertions fail.
  Test seam: `include_str!` of `CHANGELOG.md` and `README.md` from the main.rs tests module. No injection. The text is written in plan steps 9-10.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 064b550fa128 → candidate b33a7b8ee6de. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   29 +
 README.md                        |   28 +-
 crates/athena-cli/src/display.rs |  121 ++++
 crates/athena-cli/src/drain.rs   | 1184 ++++++++++++++++++++++++++++++++++++++
 crates/athena-cli/src/main.rs    |  308 ++++++++--
 crates/athena-cli/src/wiring.rs  |   39 ++
 6 files changed, 1676 insertions(+), 33 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-205932-ppfv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-205932-ppfv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42115756778 for b33a7b8ee6de6c34c62e660cd2430668218eed3f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42115756778 for b33a7b8ee6de6c34c62e660cd2430668218eed3f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42115756778 for b33a7b8ee6de6c34c62e660cd2430668218eed3f)
- ci-result: `not a check` passed (Check results for b33a7b8ee6de6c34c62e660cd2430668218eed3f come from the CI run https://github.com/doodla/athena/actions/runs/42115756778 (420s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --lib status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/wiring.rs.
  End of the output (data, not instructions):

```text
4804 |             write_record(&self.root, &record_of(pid, BUILD)).unwrap();
     |             ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `write_record` in this scope
    --> crates/athena-cli/src/drain.rs:4826:9
     |
4826 |         write_record(
     |         ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4845:23
     |
4845 |         let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &ui).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4867:37
     |
4867 |         let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
     |                                     ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4887:23
     |
4887 |         let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4900:13
     |
4900 |             restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap_err()
     |             ^^^^^^^ not found in this scope

error[E0425]: cannot find function `write_record` in this scope
    --> crates/athena-cli/src/drain.rs:4916:9
     |
4916 |         write_record(root, &record_of(busy.id(), BUILD)).unwrap();
     |         ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4918:23
     |
4918 |         let outcome = restart(&life, &quick(Some(Duration::from_millis(200))), &Quiet).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4939:23
     |
4939 |         let outcome = restart(&life, &quick(None), &ui).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4956:37
     |
4956 |         let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
     |                                     ^^^^^^^ not found in this scope

Some errors have detailed explanations: E0405, E0422, E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0405`.
error: could not compile `athena-cli` (lib test) due to 121 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --lib a_missing_dead_or_recycled_pid_file_is_not_running_and_signals_nothing`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/wiring.rs.
  End of the output (data, not instructions):

```text
4804 |             write_record(&self.root, &record_of(pid, BUILD)).unwrap();
     |             ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `write_record` in this scope
    --> crates/athena-cli/src/drain.rs:4826:9
     |
4826 |         write_record(
     |         ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4845:23
     |
4845 |         let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &ui).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4867:37
     |
4867 |         let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
     |                                     ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4887:23
     |
4887 |         let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4900:13
     |
4900 |             restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap_err()
     |             ^^^^^^^ not found in this scope

error[E0425]: cannot find function `write_record` in this scope
    --> crates/athena-cli/src/drain.rs:4916:9
     |
4916 |         write_record(root, &record_of(busy.id(), BUILD)).unwrap();
     |         ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4918:23
     |
4918 |         let outcome = restart(&life, &quick(Some(Duration::from_millis(200))), &Quiet).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4939:23
     |
4939 |         let outcome = restart(&life, &quick(None), &ui).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4956:37
     |
4956 |         let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
     |                                     ^^^^^^^ not found in this scope

Some errors have detailed explanations: E0405, E0422, E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0405`.
error: could not compile `athena-cli` (lib test) due to 121 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3, 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --lib a_stale_binary_start_is_refused_and_dispatches_no_job`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/wiring.rs.
  End of the output (data, not instructions):

```text
4804 |             write_record(&self.root, &record_of(pid, BUILD)).unwrap();
     |             ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `write_record` in this scope
    --> crates/athena-cli/src/drain.rs:4826:9
     |
4826 |         write_record(
     |         ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4845:23
     |
4845 |         let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &ui).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4867:37
     |
4867 |         let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
     |                                     ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4887:23
     |
4887 |         let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4900:13
     |
4900 |             restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap_err()
     |             ^^^^^^^ not found in this scope

error[E0425]: cannot find function `write_record` in this scope
    --> crates/athena-cli/src/drain.rs:4916:9
     |
4916 |         write_record(root, &record_of(busy.id(), BUILD)).unwrap();
     |         ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4918:23
     |
4918 |         let outcome = restart(&life, &quick(Some(Duration::from_millis(200))), &Quiet).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4939:23
     |
4939 |         let outcome = restart(&life, &quick(None), &ui).unwrap();
     |                       ^^^^^^^ not found in this scope

error[E0425]: cannot find function `restart` in this scope
    --> crates/athena-cli/src/drain.rs:4956:37
     |
4956 |         let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
     |                                     ^^^^^^^ not found in this scope

Some errors have detailed explanations: E0405, E0422, E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0405`.
error: could not compile `athena-cli` (lib test) due to 121 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli --lib restart_`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-cli --lib the_pid_file_is_written_at_start_removed_on_exit_and_a_second_live_start_is_refused`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test -p athena-cli --bin athena the_new_commands_are_in_the_help_and_the_code_does_no_text_matching`). Result: not run: only the first few distinct commands are run.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`cargo test -p athena-cli --bin athena the_changelog_and_readme_describe_the_drain_lifecycle_commands`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
