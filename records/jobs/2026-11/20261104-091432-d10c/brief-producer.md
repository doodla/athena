# Job 20261104-091432-d10c: Count recorded fails-without revert runs as commands seen running

## Request (verbatim)

> Count Athena's 'fails without the fix' runs as commands seen running. Since 64cb889 Athena runs each planned revert demonstration and shows it to the verifier under 'Checks Athena ran on this tree', but the run is not written to verification.json checks, so the verdict's seen-running rule (D43, crates/athena-core/src/verify.rs) does not count it. Job 20261104-081830-js1p was rejected only on this: the verifier quoted Athena's revert record fails-without-1 (both new tests failed on the base code, exit 101) and the criterion still came out unmet with commands_verified false. Record each demonstration run in the verification record's checks (name, command, exit code, output tail, marked as a revert run), and let a behavior criterion that cites that record's command, or its record name, count as seen running for the 'fails without the fix' part. Acceptance: a test where a criterion citing a recorded revert run is met with commands_verified true; a test where a criterion citing a command neither run nor recorded stays unseen.

## Reading (accepted by the user)

Retry of job 20261104-081830-js1p's follow-up need: Athena's revert demonstration (fails-without-N) is shown to the verifier but not recorded in verification.json checks, so judge() in verify.rs does not count it as seen running. Record each run as a check (name, command, exit code, output tail, marked as a revert run) and let a behavior criterion citing its command or record name count as seen running. The audience is the verifier verdict and the maintainers reading verification.json.

Assumptions:
- Prior job 20261104-081830-js1p (rejected only on its criterion 3 'cited but not seen running'): its easy-job-planning work is not addressed here and is treated as already handled elsewhere. The repository now has asks_for_fails_without_run in crates/athena-core/src/run/mod.rs, so its minor and nit findings no longer apply to this request. Its candidate files CHANGELOG.md, crates/athena-cli/tests/job_plan.rs, run/mod.rs, run/tests.rs, athena-data config.rs and docs/design/job-planning.md are not deliverables here, except run/tests.rs, which is kept as a deliverable for the new tests.
- Code survey: crates/athena-core/src/verify.rs judge() builds `ran` from commands_run plus passing checks, but only when candidate_tree equals checks_tree. It then marks a behavior criterion commands_verified only if every cited command is in `ran`. crates/athena-core/src/run/checks.rs (around lines 327-354) runs each demonstration as a CheckSpec named fails-without-N and returns only a CheckResultView for the brief. verify.rs demonstration_view and demonstration_entry (around lines 975-1035) build that view with command 'not a check'. crates/athena-data/src/records.rs CheckRun (line 785) is the record type in verification.json. It gets a new serde-default, skip-if-false field such as `revert: bool`, like `reused`. This is an optional field on an existing record, so no new record file is added. The records round-trip tests in records.rs and check_record_file cover CheckRun. All CheckRun literals (checks.rs, status.rs, introspect.rs, routing.rs, reading.rs, send_back.rs, gh/tests.rs, cli tests) need the new field.
- Seen-running measurement: a revert run is recorded with its demonstration command (the group's argv text), exit code and output tail, and marked as a revert run. Its `passed` means the test failed as the criterion expected, so judge() adds its command and its record name to `ran` only for a revert-marked record whose run happened, regardless of the exit code being nonzero. A recorded run that did not happen ('not run') is never counted.
- The change is judged only on the existing tree equality rule: the revert record belongs to the same candidate tree. A cited name such as `fails-without-1` is accepted as a cited command for a behavior criterion only when a revert record of that name exists.
- Existing wiring in run/verification.rs and run/checks.rs is reused; how demonstrations are chosen, run or shown to the verifier (64cb889) is unchanged except that the run is also returned as a CheckRun.

## Acceptance criteria

1. [behavior] A judge() test in crates/athena-core/src/verify.rs: with a recorded revert-marked CheckRun (name fails-without-1, exit code 101) for the candidate tree, a behavior criterion citing that run's command is met with commands_verified true. Run `cargo test --locked -p athena-core verify`.
2. [behavior] A judge() test where the criterion cites the record name fails-without-1 gets commands_verified true and met true, and no 'cited but not seen running' finding appears. Run `cargo test --locked -p athena-core verify`.
3. [behavior] A judge() test where a behavior criterion cites a command that was neither run nor recorded (including a revert-marked record that did not run, or one from a different tree) stays commands_verified false, is forced unmet, and gets the Major 'cited but not seen running' finding. Run `cargo test --locked -p athena-core verify`.
4. [behavior] A run-level test in crates/athena-core/src/run/tests.rs shows the demonstration run is written to the verification record's checks with name, command, exit code, output tail and the revert marker, in the same pass that shows it to the verifier. Run `cargo test --locked -p athena-core run::`.
5. [behavior] CheckRun with the new field round-trips, and an older verification.json without the field still loads, with the marker false. Run `cargo test --locked -p athena-data`.
6. [behavior] The whole workspace still passes. Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`.
7. [content] CHANGELOG.md has an entry, and a design doc (docs/design/job-planning.md or the one that covers D43) states in prose that recorded revert runs count as seen running for the 'fails without the fix' part only.
8. [content] Outcome check: a job like 20261104-081830-js1p, whose verifier cites the revert record fails-without-1, would get commands_verified true. This is shown by the test in the first criterion, which builds the same input shape (exit 101, failing on base).

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-cli/tests/status_durations.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Job 20261104-081830-js1p's candidate files crates/athena-cli/tests/job_plan.rs and crates/athena-data/src/config.rs: that work is dropped here, and so is its planning-gate change, which is not part of this request.
- Changing how demonstrations are chosen, run or shown to the verifier (landed in 64cb889).
- Counting any other non-command record as seen running, or loosening the D43 rule for criteria that do not cite a revert run.
- Changing the planner, critic, reader or verifier prompts.
- Governance files: athena.toml, defaults.toml, workflows.
- Adding a fails_without field to the reading's Criterion record.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus cargo test, clippy and fmt checks pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run judge() and the record round-trip tests, not wiring existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a verifier citing the recorded revert run gets commands_verified true.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix counts the recorded run in the verdict rule, not just the one rejected job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in judge() code with tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the record and the verdict rule.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for an outside reader.
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
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

26 more qualifying classes are left out.

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

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A judge() test in crates/athena-core/src/verify.rs: with a recorded revert-marked CheckRun (name fails-without-1, exit code 101) for the candidate tree, a behavior criterion citing that run's command is met with commands_verified true. Run `cargo test --locked -p athena-core verify`.
A judge() test where the criterion cites the record name fails-without-1 gets commands_verified true and met true, and no 'cited but not seen running' finding appears. Run `cargo test --locked -p athena-core verify`.
A judge() test where a behavior criterion cites a command that was neither run nor recorded (including a revert-marked record that did not run, or one from a different tree) stays commands_verified false, is forced unmet, and gets the Major 'cited but not seen running' finding. Run `cargo test --locked -p athena-core verify`.
A run-level test in crates/athena-core/src/run/tests.rs shows the demonstration run is written to the verification record's checks with name, command, exit code, output tail and the revert marker, in the same pass that shows it to the verifier. Run `cargo test --locked -p athena-core run::`.
CheckRun with the new field round-trips, and an older verification.json without the field still loads, with the marker false. Run `cargo test --locked -p athena-data`.
The whole workspace still passes. Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`.
CHANGELOG.md has an entry, and a design doc (docs/design/job-planning.md or the one that covers D43) states in prose that recorded revert runs count as seen running for the 'fails without the fix' part only.
Outcome check: a job like 20261104-081830-js1p, whose verifier cites the revert record fails-without-1, would get commands_verified true. This is shown by the test in the first criterion, which builds the same input shape (exit 101, failing on base).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
