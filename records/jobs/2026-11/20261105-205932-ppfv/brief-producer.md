# Job 20261105-205932-ppfv: Add drain pid file with status, stop-and-wait, restart, stale-build

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-205932-ppfv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-205932-ppfv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-cli`: a test starts a fake drain pid file, and `drain-status` and `drain-stop` find it correctly when the surrounding command line contains the word `drain`; they never match command text.
Run `cargo test -p athena-cli`: a missing pid file, or one naming a dead or recycled pid, is reported as not running by status and stop, and no unrelated process is signalled.
Run `cargo test -p athena-cli`: a drain start with a binary older than the landed code is refused with a message naming why, and dispatches no job.
Run `cargo test -p athena-cli`: restart performs the rebuild step before starting the new drain, waits for the old drain to exit, and prints the build it started; a failed rebuild means no drain starts.
Run `cargo test -p athena-cli`: a drain writes the pid file at start and removes it on exit; a second drain start with a live pid file is refused.
The change refuses more drain starts (stale binary): expected 5% of newly refused readings.
`cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
Grep of crates/athena-cli/src/main.rs shows the new drain-status and drain-restart subcommands with help text telling the person running Athena to use them instead of pkill/pgrep; no pgrep/pkill-style command-text matching is added in the code.
CHANGELOG.md has an entry for the drain lifecycle commands and stale-binary refusal, and any docs naming drain-stop describe the new commands.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
