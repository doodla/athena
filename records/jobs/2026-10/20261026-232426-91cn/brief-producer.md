# Job 20261026-232426-91cn: Move Send-back row and left_to_athena_tests to run/send_back.rs

## Request (verbatim)

> Priority: high
> Structure pass job R2 (docs/design/structure-pass.md, section 4). Move the Send-back row with left_to_athena_tests. No behavior change. Depends on 20261026-232423-hvyb.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/send_back.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job R2: move unmet_criteria, left_to_athena, marked_left, is_athena_check_criterion, send_back_text, criterion_key and mod left_to_athena_tests from run/mod.rs into new run/send_back.rs, re-exported so paths like crate::run::send_back_text are unchanged. Pure move, no behavior change. Prior job 20261026-232423-hvyb (R1, run.rs to run/mod.rs) was closed as superseded because R1 landed via its retry; run/mod.rs already exists, so nothing from it is redone or kept.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs holds unmet_criteria (~1676), left_to_athena (~1695), marked_left, is_athena_check_criterion, send_back_text (~1803), criterion_key (~1836) and mod left_to_athena_tests (~3718); docs/design/structure-pass.md section 4 row R2 and section 5 proof define the move. No record or data files are added, so check_record_file/AnyRecord are not involved.
- Job 20261026-232423-hvyb (R1) is closed as superseded; its scope (delete run.rs, create run/mod.rs) is already landed, its candidate changed no files, so no finding remains and none of its files are touched here.
- Audience: the structure-pass reviewer/verifier who checks the section 5 proof.
- The 'report lists the caller check' item is satisfied by the verifier's own Grep of callers, not by the producer's report.
- No athena.toml, defaults.toml or workflow change is needed.

## Acceptance criteria

1. [content] crates/athena-core/src/run/send_back.rs exists and contains fn unmet_criteria, left_to_athena, marked_left, is_athena_check_criterion, send_back_text, criterion_key and mod left_to_athena_tests; Grep shows none of these definitions remain in run/mod.rs.
2. [content] run/mod.rs declares mod send_back and re-exports the pub(crate) names (unmet_criteria, left_to_athena, send_back_text) so Grep for run::(unmet_criteria|left_to_athena|send_back_text) across crates/ is unchanged and no caller path was edited.
3. [content] git diff shows moved function and test text identical (only visibility/import/use lines differ); no test other than imports is edited.
4. [behavior] Run cargo test --locked: passes, and the test list for left_to_athena_tests is unchanged in names (now under run::send_back).
5. [behavior] Run cargo fmt --all --check and cargo clippy --all-targets --locked -- -D warnings: both clean.
6. [content] CHANGELOG.md has a new entry describing the Send-back move as no behavior change.

## Expected deliverables

- crates/athena-core/src/run/send_back.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any other row of run/mod.rs (producer, verifier, etc.; later R jobs)
- Editing tests beyond import lines
- Changing athena.toml, defaults.toml or workflows
- Redoing R1 (run.rs to run/mod.rs), already landed
- Any behavior change or deduplicating test helpers

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust move.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single row move only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests and caller Grep verify unchanged behavior.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests run and Grep of callers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

18 more qualifying classes are left out.

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

crates/athena-core/src/run/send_back.rs exists and contains fn unmet_criteria, left_to_athena, marked_left, is_athena_check_criterion, send_back_text, criterion_key and mod left_to_athena_tests; Grep shows none of these definitions remain in run/mod.rs.
run/mod.rs declares mod send_back and re-exports the pub(crate) names (unmet_criteria, left_to_athena, send_back_text) so Grep for run::(unmet_criteria|left_to_athena|send_back_text) across crates/ is unchanged and no caller path was edited.
git diff shows moved function and test text identical (only visibility/import/use lines differ); no test other than imports is edited.
Run cargo test --locked: passes, and the test list for left_to_athena_tests is unchanged in names (now under run::send_back).
Run cargo fmt --all --check and cargo clippy --all-targets --locked -- -D warnings: both clean.
CHANGELOG.md has a new entry describing the Send-back move as no behavior change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
