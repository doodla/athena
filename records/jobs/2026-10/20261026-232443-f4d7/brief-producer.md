# Job 20261026-232443-f4d7: Move run stages S8-S10 into run/verification.rs

## Request (verbatim)

> Priority: high
> Structure pass job R8 (docs/design/structure-pass.md, section 4). Split step 4: S8-S10 into run/verification.rs with verifier_snapshot; produce_and_verify becomes a driver of about 80 lines. No behavior change. Depends on 20261026-232440-0x6r.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/verification.rs, CHANGELOG.md

## Reading (accepted by the user)

Job R8 of the structure pass: move stages S8-S10 and verifier_snapshot out of run/mod.rs into a new run/verification.rs, leaving produce_and_verify as a ~80-line driver. Behavior is unchanged. Audience: the maintainer and the verifier checking structure-pass §5. Follow-up to job 20261026-232440-0x6r (R7), whose findings are addressed in this entry.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs (produce_and_verify at ~line 1088, verifier_snapshot at ~1796, mod declarations at 53-59, test modules critic_soundness_tests and produce_and_verify_order_tests) and listed crates/athena-core/src/run/ (candidate, checks, close, pin, planning, producer, send_back). The entry point stays produce_and_verify; stage order is unchanged. No record or data file is added, so check_record_file and AnyRecord are not affected. Behavior is measured by the existing run:: tests passing unchanged.
- Job 20261026-232440-0x6r (R7) is kept as landed: checks.rs, its mod.rs wiring and its CHANGELOG entry are not redone. Its findings are addressed here: (a) the R8 CHANGELOG entry gives before and after file counts for each moved name's caller check; (b) the entry is split into short bullets rather than one dense paragraph; (c) the producer.rs nit does not apply, since pub(super) kept the call resolving without an edit, and R8 likewise should leave producer.rs alone unless needed. The R7 entry's missing counts are not retro-fixed, because editing it is out of scope.
- S8-S10 are the stage markers in docs/design/structure-pass.md section 4; the producer reads that section to find the exact boundaries. Moved items get pub(super) as needed; the text is otherwise identical apart from rustfmt wrapping.
- Test modules stay in mod.rs so test paths and the test list do not change. Only their import lines may be edited.

## Acceptance criteria

1. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all exit 0.
2. [behavior] Compare `cargo test --locked -- --list` before and after: the test list is identical, and the diff of existing test code shows only import-line edits.
3. [content] `git diff` shows S8-S10 and verifier_snapshot moved into crates/athena-core/src/run/verification.rs with the same text apart from visibility (pub(super)) and rustfmt wrapping; mod.rs adds `mod verification;` and any needed `use`.
4. [content] produce_and_verify in crates/athena-core/src/run/mod.rs is about 80 lines (roughly 60-100) and only drives the stage calls in the original order.
5. [content] Public paths are unchanged: no `pub` item path in athena-core changes, and re-exports are used where a moved name was public. Callers in main.rs, drain.rs, producer.rs and other run/ files need no edit beyond imports.
6. [content] CHANGELOG.md has an R8 entry as short bullets listing the Grep caller check for each moved name with the file count before and after (equal counts plus verification.rs), the visibility changes, and any early returns.
7. [content] No edits to docs/design/structure-pass.md, athena.toml, defaults.toml or workflows.

## Expected deliverables

- crates/athena-core/src/run/verification.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)

## Out of scope

- Moving other stages (S1-S7) or later jobs
- Any behavior change, or test edits beyond import lines
- Editing docs/design/structure-pass.md, including done-markers
- Redoing candidate.rs, send_back.rs, close.rs or the R7 work in checks.rs, which stay as landed
- Editing athena.toml, defaults.toml, workflows, or callers in main.rs or drain.rs
- Rewriting produce_and_verify as a state machine
- Retro-editing the R7 CHANGELOG entry

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and the full cargo test must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs fmt, clippy and tests and compares the test list.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move with no behavior change.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the tests pass unchanged and the moved text is identical, not just that the file exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is a behavior-preserving split, checked by an unchanged test list and diff.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry is written as short bullets for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
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

20 more qualifying classes are left out.

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

Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all exit 0.
Compare `cargo test --locked -- --list` before and after: the test list is identical, and the diff of existing test code shows only import-line edits.
`git diff` shows S8-S10 and verifier_snapshot moved into crates/athena-core/src/run/verification.rs with the same text apart from visibility (pub(super)) and rustfmt wrapping; mod.rs adds `mod verification;` and any needed `use`.
produce_and_verify in crates/athena-core/src/run/mod.rs is about 80 lines (roughly 60-100) and only drives the stage calls in the original order.
Public paths are unchanged: no `pub` item path in athena-core changes, and re-exports are used where a moved name was public. Callers in main.rs, drain.rs, producer.rs and other run/ files need no edit beyond imports.
CHANGELOG.md has an R8 entry as short bullets listing the Grep caller check for each moved name with the file count before and after (equal counts plus verification.rs), the visibility changes, and any early returns.
No edits to docs/design/structure-pass.md, athena.toml, defaults.toml or workflows.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
