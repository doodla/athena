# Job 20261104-103213-f1cw: Reuse a green CI result as the job baseline when the fingerprint matches

## Request (verbatim)

> Stop re-testing unchanged code before every job. Each job's baseline stage runs the full fmt, clippy and test suite on the base commit on Athena's machine before the writer starts: a mean of 20 minutes for each of the 30 jobs that landed from 1 to 4 Nov, about a third of a landed job's 60 minutes. The base's governed content was nearly always just tested: by the GitHub run of the candidate that landed, or by main's GitHub run. When a green GitHub result exists for the same governed content as the base (same fingerprint, the way candidate checks are already reused), reuse it as the baseline instead of running it locally; otherwise run it as today. Acceptance: a test where a green CI result for the base's governed content makes the baseline a reused record with no local run; a test where none exists and the baseline runs locally as before.

## Reading (accepted by the user)

The baseline stage (`take_baseline`) skips its local fmt, clippy and test run when a green GitHub result exists for the base's governed content (same fingerprint, resolved the way candidate checks are). It then records a reused baseline. Otherwise it runs locally as today. The audience is Athena's operator, who pays about 20 minutes per job for the local baseline.

Assumptions:
- Code survey: the entry point is `take_baseline` in crates/athena-core/src/run/checks.rs, called from `baseline_stage` in crates/athena-core/src/run/mod.rs (about line 1394) before the producer. Today it builds a `Baseline` and runs `baseline_checks`, which calls `crate::checks::baseline_runs` (crates/athena-core/src/checks.rs). The new order is: resolve a CI result for the base's fingerprint first (`resolve` in crates/athena-core/src/ci_result.rs, `Source::Ci`, the same path candidate checks use in run/mod.rs around lines 1499-1587 and `reuse_statement` in verify.rs). Only when that gives no green result does the baseline probe run locally.
- Code survey: the baseline record is `Baseline` in crates/athena-data/src/records.rs (`athena.baseline/1`, `deny_unknown_fields`, written at `<job>/baseline.json`). The change adds an optional reuse marker to it (for example a `reused` source note), defaulted so existing files still parse. The records validation accepts it through `check_record_file` (records.rs line 3050) and the `AnyRecord::Baseline` arm, which maps the path to `baseline.json` and the schema to `BASELINE_SCHEMA`. Because the schema tag stays the same, the allow-list needs no new path. The existing baseline round-trip tests in records.rs (around lines 4524-4598, using `check_record_file` on `{job}/baseline.json`) are extended to cover the new field.
- Code survey: the saving is measured directly in tests, not through a stand-in. The local-run path is observed by whether `baseline_runs` (the check executor) is called, as the existing `a_green_run_for_the_same_sha_is_reused_and_nothing_runs_locally` test in ci_result.rs and `ci_reuse_run` in crates/athena-core/src/run/tests.rs do for candidates. The recorded `Baseline.checks` then carry `reused: true` (the `CheckRun` field already exists) with no local run.
- The governed-content fingerprint of the base is the existing `fingerprint(&git, base, base)` used for `base_fingerprint` in run/candidate.rs. The base's CI result is looked up by that fingerprint, which may match another commit's green run, such as the landed candidate or main.
- A reused baseline keeps the `failed_before` and `failed_tests_before` semantics. A green CI result means all checks passed, so no check is excused as pre-existing. A CI lookup error, a red or pending run, or a missing run falls back to the local run with a note, and the job never fails because of the lookup.
- No governance file (athena.toml, defaults.toml, workflows) needs to change. A nested run keeps its existing skip note.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs (run with `cargo test -p athena-core baseline`) gives a green CI result for the base's fingerprint. `take_baseline` returns a baseline marked reused, whose checks are marked reused and pass, and the local check executor was never invoked.
2. [behavior] A new test (run with `cargo test -p athena-core baseline`) has no CI result, a red one, or a lookup error for the base's fingerprint. `take_baseline` runs the checks locally as before and returns the same records as before, with no reuse marker, and a red or error case also leaves a note.
3. [behavior] The `Baseline` round-trip tests in crates/athena-data/src/records.rs, via `check_record_file` and `AnyRecord::Baseline` on `{job}/baseline.json`, pass with and without the new reuse field. An older baseline file with no such field still parses (`cargo test -p athena-data`).
4. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass on the whole workspace.
5. [content] Reuse is keyed on the same governed-content fingerprint that candidate check reuse uses, through the existing `ci_result::resolve` path, not a new lookup. The reuse decision and its reason (reused from which run, or why it ran locally) are printed in the job's output like the existing `ci         checks reused from ...` line.
6. [content] Docs or CHANGELOG.md state that the baseline can now be reused from a green CI result, and what happens when none exists.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/verify.rs (modify)

## Out of scope

- Changing how candidate checks are reused, or the CI polling and wait logic.
- Changing athena.toml, defaults.toml or any workflow under .github/workflows/.
- Skipping the baseline for jobs with no green CI result.
- Changing the verdict rules for pre-existing failures.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover the reused and local paths, with cargo test and clippy passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that no local run happens on a hit and that the local run still happens on a miss.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the baseline record and local-run count, not the new wiring.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: It reuses the existing CI resolution and fingerprint code.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the baseline stage.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the named cargo tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the CHANGELOG entry are written for an outside reader.
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

A new test in crates/athena-core/src/run/tests.rs (run with `cargo test -p athena-core baseline`) gives a green CI result for the base's fingerprint. `take_baseline` returns a baseline marked reused, whose checks are marked reused and pass, and the local check executor was never invoked.
A new test (run with `cargo test -p athena-core baseline`) has no CI result, a red one, or a lookup error for the base's fingerprint. `take_baseline` runs the checks locally as before and returns the same records as before, with no reuse marker, and a red or error case also leaves a note.
The `Baseline` round-trip tests in crates/athena-data/src/records.rs, via `check_record_file` and `AnyRecord::Baseline` on `{job}/baseline.json`, pass with and without the new reuse field. An older baseline file with no such field still parses (`cargo test -p athena-data`).
`cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass on the whole workspace.
Reuse is keyed on the same governed-content fingerprint that candidate check reuse uses, through the existing `ci_result::resolve` path, not a new lookup. The reuse decision and its reason (reused from which run, or why it ran locally) are printed in the job's output like the existing `ci         checks reused from ...` line.
Docs or CHANGELOG.md state that the baseline can now be reused from a green CI result, and what happens when none exists.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
