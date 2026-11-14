# Job 20261114-045647-w91y: Tell the producer its deadline and pacing in its brief

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The producer runs against a fixed wall-clock deadline. Nothing in the job tracks progress, checks in
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: producer-deadline-exceeded-on-small-job (20261017-032151-9ymk)
> Cause: The producer runs against a fixed wall-clock deadline. Nothing in the job tracks progress, checks in partway, or tells a stalled producer from a slow one. A short prompt-edit job therefore burns the full 1800 seconds before anyone learns it...
> Seen 11 time(s) in 10 jobs. The introspections that record it:
> Introspection 20261017-032141-7xvh (job 20261010-154006-vc5k); prevention it proposed: Add a producer progress watchdog. It would require a visible step, such as a file written or a commit made, within a short window (about 10 minutes). If none appears, it would stop the producer early and record its last activity and partial...
> Introspection 20261017-044251-1804 (job 20261016-052605-qw38); prevention it proposed: Change the producer stage so a deadline does not discard its work. (1) The producer commits checkpoints and writes a progress note as it goes. (2) On deadline, the job keeps the partial result and is marked "timed out, resumable", which is ...
> Introspection 20261018-073427-vw9w (job 20261018-070117-qmxv); prevention it proposed: Make the producer stage resumable and size-aware. (1) Before the producer runs, a planning step must split any job with more than one separable deliverable into sub-jobs, each of which fits one deadline. (2) The producer must commit partial...
> Introspection 20261023-234113-5z2s (job 20261019-013309-nvbb); prevention it proposed: Give the producer stage a progress signal and an early-stop rule. It would write a checkpoint (files touched, criteria addressed) at intervals. If no checkpoint or diff appears within a fraction of the deadline, the stage aborts early and r...
> Introspection 20261023-234230-55v8 (job 20261023-234149-jcd2); prevention it proposed: Put the test budget in the harness, not in the prompt. Record the repo's scoped test command per crate and the typical full-suite duration. Run producer test commands through a wrapper that allows the full suite once, or twice at most, and ...
> Introspection 20261024-002507-470t (job 20261023-234149-jcd2); prevention it proposed: Give the producer stage a progress signal and an early stop. It should write a checkpoint of its partial diff and a short status note at set intervals. If it makes no progress for a set period, it should stop early and report what it found,...
> Introspection 20261024-145615-7s8b (job 20261024-140624-ewbs); prevention it proposed: Make the producer's deadline recoverable. (1) The producer commits checkpoints to the job branch and writes a short progress note as it goes. (2) On deadline, Athena keeps the partial work and the last note, marks the job "timed out with pa...
> Introspection 20261025-055532-4grg (job 20261025-034731-2am0); prevention it proposed: Make the producer stage report progress and keep partial work. It should write a checkpoint (files changed, last command, last test run) at intervals. On deadline, it should record that checkpoint as a timeout result separate from a real fa...
> Introspection 20261025-150430-h29z (job 20261017-044405-4zcq); prevention it proposed: Make a producer timeout a resumable state and not a bare failure. On deadline, Athena would snapshot the producer's working tree and its last activity, record the job as timed out, and let a retry start from that snapshot with a note of wha...
> Introspection 20261031-043209-pa0f (job 20261031-043018-kjy2); prevention it proposed: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena ove...
> Introspection 20261102-072959-zhys (job 20261031-230917-g47x); prevention it proposed: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer runs against a fixed wall-clock deadline (1800s) that its brief never states, so it can't pace itself or stop to report. Add a "Time budget" section to the producer brief with the real deadline, an instruction to finish the smallest complete change first, to run the full test suite at most once, and to leave the tree in a reportable state. Jobs 20261010-154006-vc5k, 20261016-052605-qw38 and 20261018-070117-qmxv failed only on this deadline and had no candidate; their requests are not redone here.

Assumptions:
- Goal and audience: the producer agent is the audience of the brief change, and Athena's operator judges whether deadline failures drop. The failure class is 'producer deadline exceeded on a small job'.
- Code survey: crates/athena-core/src/run/producer.rs (producer_snapshot and the producer loop, Ended::Deadline at line 395), crates/athena-core/src/run/mod.rs (line 999 formats 'the producer did not complete: deadline of Ns reached'; line 1010 handles Ended::Deadline; lines 1195-1213 build the resume RetrySeed), crates/athena-agent/src/procs.rs (the deadline reason text), and crates/athena-context/src/briefs.rs (producer_brief_with_evidence, retry_section, the resume note). A timed-out job already keeps its work and resumes with a progress note. No code puts the deadline in the brief (grep for deadline in athena-context finds only resume text). No record or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by reading the rendered brief text in a test.
- Existing code: the change extends crates/athena-context/src/briefs.rs (ProducerInputs, producer_brief_with_evidence) and the producer-input construction in crates/athena-core/src/run/; the existing resume and progress-note mechanism is reused unchanged.
- Field meaning: the new ProducerInputs field carries the producer deadline in seconds, read from the same config value that crates/athena-agent/src/procs.rs uses to enforce the deadline. The implementer should confirm the config key name in athena.toml.
- Governance: athena.toml and defaults.toml are not changed. The existing deadline value is only read. Consumers: none found for a new config key because none is added.
- Prompt files under crates/athena-context/prompts/ are not edited, because the reader, producer and verifier prompt sets are near their token budgets. The text lives in the generated brief, which has no token budget in the list above.
- The introspections' other proposals are alternatives not taken here: a progress watchdog and resumable checkpoints (resume already exists), and a refusal of identical-scope retries (a refusal rule, allowed only if giving the information cannot prevent the failure). They are left for follow-up.

## Acceptance criteria

1. [content] The producer brief in crates/athena-context/src/briefs.rs (producer_brief_with_evidence) has a new 'Time budget' section giving the producer's deadline in seconds, taken from the configured producer deadline and not a constant, and telling it to finish the smallest complete change first, run the full test suite at most once, and keep the tree reportable. Verify by reading the diff.
2. [behavior] A new unit test in crates/athena-context/src/briefs.rs fails on the base and passes with the change. It builds producer briefs for two different deadlines and asserts each brief states its own deadline value, and that the section appears for a fresh job, a retry and a resume. Run: cargo test -p athena-context.
3. [behavior] A test in crates/athena-core/src/run/tests.rs shows the deadline configured for the producer invocation is the number written into the stored brief-producer.md (the same value, not a copy). Run: cargo test -p athena-core.
4. [content] Root cause named in CHANGELOG.md and in the doc comment of the new section: the stage lacked its own deadline and pacing information. Jobs 20261010-154006-vc5k, 20261016-052605-qw38 and 20261018-070117-qmxv are addressed by this: they recorded no candidate, so nothing is kept from them. Their original requests (verifier snapshot note, auditor and warden roles, baseline checks) are not part of this change.
5. [content] No refusal rule is added: nothing in the diff refuses, holds or rejects a reading or job. Expected 0% of newly refused readings.
6. [behavior] cargo fmt --check, cargo clippy and cargo test pass, as recorded by Athena's checks on the final tree.
7. [behavior] Outcome check for the producer-deadline-exceeded-on-small-job class: running cargo test shows the brief of a small job (short deadline and one deliverable) carries the budget text, so the producer is told its limit before it starts.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding a progress watchdog, early-stop rule or checkpoint commits to the producer
- Making retry or auto-requeue refuse identical-scope attempts
- Redoing the requests of jobs 20261010-154006-vc5k (verifier snapshot note: crates/athena-context/prompts/verifier.md), 20261016-052605-qw38 (auditor and warden roles: roles.rs, warden.rs, auditor.rs, lib.rs, main.rs, drain.rs, store.rs, records.rs, design doc, decisions.md) and 20261018-070117-qmxv (baseline checks: checks.rs, routing.rs, display.rs, outputs.rs, verifier.md, lifecycle.rs, docs/design.md); their candidates changed no files, so no work is kept or dropped
- Changing the deadline value, athena.toml, defaults.toml or CI workflows
- Editing prompt files under crates/athena-context/prompts/

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: brief text plus the plumbing for the deadline value.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the stage's inputs, not a per-job workaround.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the rendered brief text for different deadlines.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

36 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261114-045647-w91y/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045647-w91y/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The producer brief in crates/athena-context/src/briefs.rs (producer_brief_with_evidence) has a new 'Time budget' section giving the producer's deadline in seconds, taken from the configured producer deadline and not a constant, and telling it to finish the smallest complete change first, run the full test suite at most once, and keep the tree reportable. Verify by reading the diff.
A new unit test in crates/athena-context/src/briefs.rs fails on the base and passes with the change. It builds producer briefs for two different deadlines and asserts each brief states its own deadline value, and that the section appears for a fresh job, a retry and a resume. Run: cargo test -p athena-context.
A test in crates/athena-core/src/run/tests.rs shows the deadline configured for the producer invocation is the number written into the stored brief-producer.md (the same value, not a copy). Run: cargo test -p athena-core.
Root cause named in CHANGELOG.md and in the doc comment of the new section: the stage lacked its own deadline and pacing information. Jobs 20261010-154006-vc5k, 20261016-052605-qw38 and 20261018-070117-qmxv are addressed by this: they recorded no candidate, so nothing is kept from them. Their original requests (verifier snapshot note, auditor and warden roles, baseline checks) are not part of this change.
No refusal rule is added: nothing in the diff refuses, holds or rejects a reading or job. Expected 0% of newly refused readings.
cargo fmt --check, cargo clippy and cargo test pass, as recorded by Athena's checks on the final tree.
Outcome check for the producer-deadline-exceeded-on-small-job class: running cargo test shows the brief of a small job (short deadline and one deliverable) carries the budget text, so the producer is told its limit before it starts.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
