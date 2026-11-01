# Job 20261101-114004-358q: Count CI-reused passing checks as commands run in the verdict

## Request (verbatim)

> When the checks stage reused GitHub's CI result for the candidate commit, the verdict does not count those check commands as seen running, so a behavior criterion citing cargo fmt, clippy or test is marked unmet (D43) and a passing job is rejected. Count a passing check whose result was reused from a green CI run for the same candidate commit as a command run, as a passing local check on the same tree already counts (D115).
> - In crates/athena-core/src/verify.rs judge(): a reused passing check record for the candidate commit counts toward the seen commands; a failed, skipped or other-commit record still does not.
> - A test in verify.rs builds VerdictInputs with checks reused from CI (exit code absent, the reuse note set, same tree) and a behavior criterion citing cargo test, and shows the criterion is met; it fails on today's code.
> - A test shows a reused record for a different commit, or a failed one, still leaves the criterion unmet.
> - Mark reused records with a field rather than matching note text, if the CheckRun record needs one; keep old records loading.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion; the checks stage covers it.
> - CHANGELOG.md gets an entry.
> Priority: urgent

## Reading (accepted by the user)

When the checks stage reused a green CI run for the candidate commit, judge() in verify.rs should count those passing records as seen commands, so a behavior criterion citing cargo fmt, clippy or test is met. Goal: stop rejecting passing jobs; audience is the verdict and whoever reads its criteria results. Reused records get an explicit CheckRun field, with old records still loading.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (judge() at ~line 308 builds `ran` from commands_run, then adds passing records only when `candidate_tree == checks_tree` and `exit_code == Some(0)`; a reused CI record has no exit code, so it never counts), crates/athena-core/src/checks.rs (~line 230-254 builds reused records from a CiRecord with note 'result reused from the ... for {sha}'; that is the producer to mark), crates/athena-core/src/routing.rs (~267 runs_from_record, resolves reuse before local runs) and crates/athena-data/src/records.rs (CheckRun at ~751, `deny_unknown_fields`, existing optional fields use `#[serde(default, skip_serializing_if)]`). Records validation: `check_record_file` (records.rs ~2927) parses into `AnyRecord` (~2613), so a new `#[serde(default, skip_serializing_if = "...")] reused: bool` on CheckRun is accepted by deny_unknown_fields; old records without it load via default. The existing round-trip test `failing_test_names_round_trip_and_are_read_per_check` (records.rs ~4227) and the CheckRun literal at ~4188 are the pattern for covering it; the producer adds a similar old-record-still-loads test. Behavior is measured directly by calling judge() on VerdictInputs in verify.rs unit tests (existing helper judge_recorded ~816), not through a stand-in.
- Commit match: the record counts only when it is for the candidate commit. CheckRun has no commit field, so the producer decides how judge() knows the commit (for example the existing checks_tree/candidate_tree equality, or a recorded sha next to the reused flag); the test must show a different-commit record stays unmet.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/verify.rs builds VerdictInputs with passing checks reused from CI (exit_code None, reused field set, same tree and commit) and a behavior criterion citing `cargo test`, and asserts the criterion is met with commands_verified true. Run `cargo test -p athena-core verify`; the test passes, and it fails on the base code (judge() ignores records without exit code 0).
2. [behavior] Unit tests in verify.rs show a reused record for a different commit/tree, a failed reused record, and a skipped record each leave the `cargo test` behavior criterion unmet. Run `cargo test -p athena-core verify`.
3. [content] CheckRun in crates/athena-data/src/records.rs gains a serde-defaulted boolean field marking reused records (not note-text matching), crates/athena-core/src/checks.rs sets it where reused records are built, and judge() reads the field, not the note string (Grep for the note text in verify.rs finds no matching on it).
4. [behavior] A records.rs test deserializes a CheckRun JSON without the new field and it loads with the field false, and a reused record round-trips through check_record_file/AnyRecord. Run `cargo test -p athena-data`.
5. [content] CHANGELOG.md has an entry describing that checks reused from a green CI run for the candidate commit now count as commands run in the verdict (D43, D115).

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Changing how or when CI results are fetched or reused
- Changing the baseline or pre-existing failure logic
- Listing a repository-wide fmt, clippy or test run as its own acceptance criterion (the checks stage covers it)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests in verify.rs and records.rs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in judge()'s seen-command rule, not by loosening the criterion check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run judge() on reused-check inputs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is a passing job no longer rejected for unseen commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 38 cases in 38 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict step in code. Before the verifier runs, Athena runs the repo gate (fmt, clippy, test) itself and records the exact commands and exit codes. It also checks that every behavior criterion has a demonstration that names a command in that record. If the gate fails or a criterion has no ...
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

A new unit test in crates/athena-core/src/verify.rs builds VerdictInputs with passing checks reused from CI (exit_code None, reused field set, same tree and commit) and a behavior criterion citing `cargo test`, and asserts the criterion is met with commands_verified true. Run `cargo test -p athena-core verify`; the test passes, and it fails on the base code (judge() ignores records without exit code 0).
Unit tests in verify.rs show a reused record for a different commit/tree, a failed reused record, and a skipped record each leave the `cargo test` behavior criterion unmet. Run `cargo test -p athena-core verify`.
CheckRun in crates/athena-data/src/records.rs gains a serde-defaulted boolean field marking reused records (not note-text matching), crates/athena-core/src/checks.rs sets it where reused records are built, and judge() reads the field, not the note string (Grep for the note text in verify.rs finds no matching on it).
A records.rs test deserializes a CheckRun JSON without the new field and it loads with the field false, and a reused record round-trips through check_record_file/AnyRecord. Run `cargo test -p athena-data`.
CHANGELOG.md has an entry describing that checks reused from a green CI run for the candidate commit now count as commands run in the verdict (D43, D115).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
