# Job 20261026-232429-r147: Move Planning and Pin/reread rows into run/planning.rs and run/pin.rs

## Request (verbatim)

> Priority: high
> Structure pass job R3 (docs/design/structure-pass.md, section 4). Move the Planning, pin and reread rows. No behavior change. Depends on 20261026-232423-hvyb and 20261026-232426-91cn.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/pin.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass R3: move the Planning row (stop_planning, plan_round, planning_round, end_round, RoundEnd, findings_line, plan_record, redacted, planning_grant and planning_grant_tests) to run/planning.rs and the Pin and reread row (pin_repo, study_pin, reread_after_configuration_change) to run/pin.rs, with no behavior change. Follows landed R2 (20261026-232426-91cn, its work is kept) and R1 (20261026-232423-hvyb, abandoned but superseded by a retry that landed, so nothing from it is redone). Audience: the maintainer judging the structure-pass proof.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs: it has `mod send_back;` plus `pub(crate) use send_back::{...}` from R2; Planning items (RoundEnd ~1479, findings_line, plan_record, redacted, planning_grant ~3481, planning_grant_tests ~3490) sit in mod.rs. The row line ranges come from docs/design/structure-pass.md rows 63-64 and 112-113, which may be stale after R1/R2.
- Job 20261026-232426-91cn (R2) landed with a minor finding: the design doc contradicts itself on whether test modules move. This job follows R2's precedent: planning_grant_tests moves with planning_grant (as the design's Planning row lists it), so its test path becomes run::planning::planning_grant_tests::* with the same short names. The doc fix is out of scope. R2's files (send_back.rs, mod.rs, CHANGELOG.md) are kept as landed; mod.rs and CHANGELOG.md are modified again.
- Job 20261026-232423-hvyb (R1) was abandoned with no candidate and no verdict; its outcome was superseded by a retry that landed run/mod.rs, so no finding remains to address.
- Moved names keep their crate::run paths via `mod planning; mod pin;` and `pub(crate) use` re-exports in mod.rs; items used only inside the submodules are visible as pub(super) as needed. Only import lines in existing tests are edited.
- The caller check is done by Grep for each moved name over crates/, and the producer lists the results in its report.
- Governance files (athena.toml, defaults.toml, workflows) are not needed.

## Acceptance criteria

1. [content] crates/athena-core/src/run/planning.rs exists and contains stop_planning, plan_round, planning_round, end_round, RoundEnd, findings_line, plan_record, redacted, planning_grant and mod planning_grant_tests; Grep over run/mod.rs finds no definitions of them.
2. [content] crates/athena-core/src/run/pin.rs exists and contains pin_repo, study_pin and reread_after_configuration_change; Grep over run/mod.rs finds no definitions of them.
3. [content] The git diff shows the moved function and test bodies identical to the removed text (only visibility and import lines differ), and existing tests change only in their import lines.
4. [content] run/mod.rs declares the new modules and re-exports every moved name that was reachable by a path outside the module, so no caller file outside run/ is edited (git diff --stat lists only mod.rs, planning.rs, pin.rs and CHANGELOG.md).
5. [content] CHANGELOG.md has an Unreleased entry for the move that says there is no behavior change.
6. [behavior] Run `cargo test --locked`: it passes and the test list is the same as before apart from the module prefix of the moved planning_grant_tests (names unchanged). Also run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both are clean.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (create)
- crates/athena-core/src/run/pin.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any other row of run/mod.rs (producer, verifier, etc.; later R jobs)
- Editing tests beyond import lines
- Changing athena.toml, defaults.toml or workflows
- Redoing R1 or R2, already landed
- Correcting the docs/design/structure-pass.md contradiction about test modules
- Any behavior change or deduplicating test helpers

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the two rows move; one focused change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs cargo test to show unchanged behavior.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Passing existing tests with only import edits is the check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog entry and clear module layout.
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

crates/athena-core/src/run/planning.rs exists and contains stop_planning, plan_round, planning_round, end_round, RoundEnd, findings_line, plan_record, redacted, planning_grant and mod planning_grant_tests; Grep over run/mod.rs finds no definitions of them.
crates/athena-core/src/run/pin.rs exists and contains pin_repo, study_pin and reread_after_configuration_change; Grep over run/mod.rs finds no definitions of them.
The git diff shows the moved function and test bodies identical to the removed text (only visibility and import lines differ), and existing tests change only in their import lines.
run/mod.rs declares the new modules and re-exports every moved name that was reachable by a path outside the module, so no caller file outside run/ is edited (git diff --stat lists only mod.rs, planning.rs, pin.rs and CHANGELOG.md).
CHANGELOG.md has an Unreleased entry for the move that says there is no behavior change.
Run `cargo test --locked`: it passes and the test list is the same as before apart from the module prefix of the moved planning_grant_tests (names unchanged). Also run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both are clean.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
