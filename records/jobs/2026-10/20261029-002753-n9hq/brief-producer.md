# Job 20261029-002753-n9hq: Treat a weekly/session usage-limit reply as an environment stop

## Request (verbatim)

> Priority: high
> First small slice of handling model usage limits. When the agent driver's reply says the weekly or session usage limit was hit (text such as "You've hit your weekly limit" with a reset time), classify the invocation as an environment stop using the existing environment-reason path in disk.rs, so the job is not counted as a failure, and have athena drain stop with the reason 'usage limit' and the reset time. Nothing else: no status display, no new record kind, no changes to retry or resume.
> Acceptance:
> - A test with a fake driver reply carrying the limit text shows the job ends as an environment stop, not a failure.
> - A drain test shows the stop reason 'usage limit' with the reset time.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-agent/src/claude.rs, crates/athena-core/src/disk.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Teach the driver and disk.rs's environment-reason path to recognise "You've hit your weekly/session limit … resets <time>" replies (not only "API Error: 429") as a usage-limit environment stop, keeping the reset time. Make athena drain stop at once with reason "usage limit" and that reset time. Audience: the athena operator reading drain output and the verifier running tests.

Assumptions:
- Code survey: read crates/athena-agent/src/claude.rs (api_error_status reads only 'API Error: NNN' details; output_from_result builds the detail as '<subtype>: <first 300 chars>'), crates/athena-core/src/disk.rs (USAGE_LIMIT, StopCause::of, producer_stop_reason, is_environment_reason), crates/athena-core/src/run/producer.rs (a non-success invocation goes through producer_stop_reason, then self.failure for usage limits, closing failed with an environment reason), crates/athena-core/src/scope_guard.rs (environment reasons are not counted as attempts) and crates/athena-cli/src/drain.rs (Stop enum; ItemEnd::Failed sets Stop::Environment only after an item fails twice).
- No record file or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched. The Stop::label and Stop::meaning tables, and any test that iterates Stop::ALL, may need updating if a variant is added.
- Existing code: the usage-limit path already exists for 'API Error: 429' (disk::USAGE_LIMIT, StopCause::UsageLimit, producer_stop_reason). This job extends it with text detection and a reset time; it does not add a parallel path.
- Assumption: the limit text looks like "You've hit your weekly limit · resets Nov 12, 3pm" (or 'session limit'). The detector matches 'hit your weekly limit' / 'hit your session limit' case-insensitively, and takes any text after 'resets' as the reset time. A reply without a reset time still counts as a usage limit, with no time shown.
- Assumption: 'drain stop with the reason usage limit' means a new Stop variant labelled 'usage limit' (not the generic 'environment'). It ends the run on the first such job, since every later item would hit the same limit. The reset time goes in the drain's stop output. The existing 'fails twice' Environment rule for logins and disk is unchanged.
- Reset time is carried inside the existing closing-reason string (USAGE_LIMIT prefix plus detail), so no new record kind is needed. The drain reads it back from the last reason.
- No governance files (athena.toml, defaults.toml, workflows) change; the only non-source deliverable is CHANGELOG.md.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-agent/src/claude.rs (fake driver script emitting an is_error result with 'You've hit your weekly limit · resets <time>') passes under `cargo test -p athena-agent`, and shows the classifier reports a usage limit with the reset time. A 'session limit' reply is covered too.
2. [behavior] A test in crates/athena-core/src/disk.rs shows producer_stop_reason on the limit text gives a reason starting with disk::USAGE_LIMIT that includes the reset time, and that is_environment_reason is true and StopCause::of gives UsageLimit. Run `cargo test -p athena-core`.
3. [behavior] A run-level test with a fake driver reply carrying the limit text (beside the existing usage-limit test in crates/athena-core/src/run/tests.rs, or an equivalent in a file listed as a deliverable) shows the job closes as an environment stop, not a counted failure, and keeps nothing. Run `cargo test -p athena-core`.
4. [behavior] A drain test in crates/athena-cli (drain.rs unit test or tests/drain.rs) shows the run stops after the first job whose reply says the limit was hit, with the stop reason 'usage limit' and the reset time in the output. Run `cargo test -p athena-cli`.
5. [behavior] Controls: text that only mentions a limit in a model's own prose (under a subtype other than success/error) and a normal success reply are not classified as usage limits. Existing 429 and 401 tests still pass.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has a new entry, in the file's existing format, describing the usage-limit environment stop and the drain stop with reset time. `git diff` shows it.
8. [content] The diff limits itself to the deliverables: no status display, no new record kind and no change to retry or resume logic (the resumable_stop and keeps_unfinished_work behaviour is unchanged). Checked with `git diff --stat`.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)

## Out of scope

- Status display of the usage limit or reset time
- A new record kind
- Changes to retry or resume (resumable_stop, keeps_unfinished_work)
- Waiting until the reset time or automatic restart of the drain
- Changes to the existing login (401) or low-disk handling

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus cargo test, clippy -D warnings and fmt --check to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name tests that show the job closing as an environment stop and the drain stopping with the reason and reset time.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the four named files, plus tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The classifier is extended in the shared disk.rs path that the scope guard and drain already read, not patched in one caller.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261029-002753-n9hq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-002753-n9hq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-agent/src/claude.rs (fake driver script emitting an is_error result with 'You've hit your weekly limit · resets <time>') passes under `cargo test -p athena-agent`, and shows the classifier reports a usage limit with the reset time. A 'session limit' reply is covered too.
A test in crates/athena-core/src/disk.rs shows producer_stop_reason on the limit text gives a reason starting with disk::USAGE_LIMIT that includes the reset time, and that is_environment_reason is true and StopCause::of gives UsageLimit. Run `cargo test -p athena-core`.
A run-level test with a fake driver reply carrying the limit text (beside the existing usage-limit test in crates/athena-core/src/run/tests.rs, or an equivalent in a file listed as a deliverable) shows the job closes as an environment stop, not a counted failure, and keeps nothing. Run `cargo test -p athena-core`.
A drain test in crates/athena-cli (drain.rs unit test or tests/drain.rs) shows the run stops after the first job whose reply says the limit was hit, with the stop reason 'usage limit' and the reset time in the output. Run `cargo test -p athena-cli`.
Controls: text that only mentions a limit in a model's own prose (under a subtype other than success/error) and a normal success reply are not classified as usage limits. Existing 429 and 401 tests still pass.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has a new entry, in the file's existing format, describing the usage-limit environment stop and the drain stop with reset time. `git diff` shows it.
The diff limits itself to the deliverables: no status display, no new record kind and no change to retry or resume logic (the resumable_stop and keeps_unfinished_work behaviour is unchanged). Checked with `git diff --stat`.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
