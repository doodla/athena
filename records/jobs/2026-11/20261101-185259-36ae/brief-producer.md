# Job 20261101-185259-36ae: Wait for pending checks before merging; treat rule-violation refusal as

## Request (verbatim)

> The drain asked GitHub to merge the rebase-before-landing pull request while its history check was still running. GitHub refused with 'Repository rule violations found', the drain counted the landed job as failed and stopped the run. A later 'athena land --pr' on the same job merged it cleanly once every check had finished.
> - Before requesting a merge, wait while any check run on the pull request head is still queued or in progress; only a failed required check is a failure, and a failed optional check stays a note.
> - A refused merge with 'Repository rule violations' while checks are pending is a wait, not a failure: the drain reports the job as waiting to merge and does not stop the run.
> - A gh test: a pull request with the required check green and another check in progress is reported as pending, and no merge call is made.
> - A drain test: a merge refused for rule violations is reported as waiting, and the run goes on to the next job.
> - CHANGELOG.md gets an entry.
> Priority: urgent

## Reading (accepted by the user)

The drain asked GitHub to merge while a non-required check run was still in progress, was refused with 'Repository rule violations', and stopped the run. The change makes check classification report pending while any check run on the head is queued or in progress, and makes a rule-violation merge refusal a wait: the drain reports the job as waiting to merge and moves to the next job. The audience is the person running the drain.

Assumptions:
- Code survey: crates/athena-core/src/gh/client.rs (classify_checks decides only on the required CI_AGGREGATE_JOB run, so an in-progress optional run reads Green); crates/athena-core/src/gh/early.rs (wait_for_checks polls check_report until not Pending); crates/athena-core/src/gh/landing.rs (land_pr calls host.merge only when merge_via_rest && checks == Green, and a merge error propagates as Err); crates/athena-cli/src/drain.rs (PrLander::land maps Err to Landing::NotLanded, which the drain counts as a failed landing). Property is measured directly via classify_checks output and via the Landing value PrLander returns.
- No record or data file under crates/ is added, so records validation (check_record_file, AnyRecord) is not touched.
- Rule-violation detection matches the error text 'Repository rule violations' from the host merge call; it is converted into a pending report (merge not confirmed) in land_pr, and PrLander/drain reports a waiting-to-merge outcome instead of a failure. The exact enum shape (a new Landing variant or a flag on PrReport) is the producer's choice, consistent with how the drain handles other not-landed outcomes.
- Check runs with status queued or in_progress count as pending; completed runs with failure conclusions stay: required failure is Red, optional failure is a note only.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [behavior] A gh test in crates/athena-core/src/gh/tests.rs: check runs with the required CI_AGGREGATE_JOB completed green plus another check run in_progress classify as Checks::Pending, and a land_pr run against the fake host makes no merge call (merge_requested false). Run `cargo test -p athena-core gh::`.
2. [behavior] A classify test shows a failed optional check run (completed, failure) with the required check green is still Green with a note, and a failed required check is Red. Run `cargo test -p athena-core gh::`.
3. [behavior] A gh test: when the fake host's merge returns an error containing 'Repository rule violations' while checks are not all complete, land_pr returns a pending/waiting result rather than Err. Run `cargo test -p athena-core gh::`.
4. [behavior] A drain test in crates/athena-cli/src/drain.rs: a merge refused for rule violations is reported as waiting to merge (not a failed landing), the run does not stop, and the next job is worked. Run `cargo test -p athena-cli drain`.
5. [content] CHANGELOG.md has a new entry describing waiting on pending check runs and treating a rule-violation merge refusal as waiting.
6. [behavior] `cargo test` passes and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
7. [content] Doc comments on classify_checks and wait_for_checks are updated to state that any queued or in-progress check run makes the verdict pending.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Reading required checks from branch protection instead of the fixed CI_AGGREGATE_JOB name.
- Changing the re-run logic for red required checks.
- Changes to athena.toml, defaults.toml or workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New gh and drain tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing pending classification, no merge call, and the drain continuing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes both the premature merge request (pending runs ignored) and the drain's handling of the refusal.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across gh and drain.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier runs the named tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 14 cases in 12 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make disk a managed budget. Before a job is held, or when free disk drops below the need, Athena reclaims what it can on its own. That means pruning the build folders of held jobs and the job's own writer build once a recoverable form exists, such as the committed diff or a patch saved in the job's ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

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

A gh test in crates/athena-core/src/gh/tests.rs: check runs with the required CI_AGGREGATE_JOB completed green plus another check run in_progress classify as Checks::Pending, and a land_pr run against the fake host makes no merge call (merge_requested false). Run `cargo test -p athena-core gh::`.
A classify test shows a failed optional check run (completed, failure) with the required check green is still Green with a note, and a failed required check is Red. Run `cargo test -p athena-core gh::`.
A gh test: when the fake host's merge returns an error containing 'Repository rule violations' while checks are not all complete, land_pr returns a pending/waiting result rather than Err. Run `cargo test -p athena-core gh::`.
A drain test in crates/athena-cli/src/drain.rs: a merge refused for rule violations is reported as waiting to merge (not a failed landing), the run does not stop, and the next job is worked. Run `cargo test -p athena-cli drain`.
CHANGELOG.md has a new entry describing waiting on pending check runs and treating a rule-violation merge refusal as waiting.
`cargo test` passes and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
Doc comments on classify_checks and wait_for_checks are updated to state that any queued or in-progress check run makes the verdict pending.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
