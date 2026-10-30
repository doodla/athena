# Job 20261026-232443-f4d7: verify "Move run stages S8-S10 into run/verification.rs"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the three Athena check commands as the verifier would. No new fixture is needed. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The `low_disk_before_the_verifier` test exercises the moved early `return Ok(closing)`. The replay tests exercise S8-S10 through the verdict and `Closing.verification`.
  Fails without the behavior: A wrong move shows up as a compile error. Examples: a `verification_stage` or `Checked` field without `pub(super)`, a `closing` used after the move, or a leftover `mut`. It also shows up as an unused-variable or unused-import lint under `-D warnings`, or as a failing run-level test. Those tests drive the production `Athena::go` into `produce_and_verify`, `check_and_verify` and the moved stage with `FakeAgentDriver` and the per-thread `check_seam`. Both seams exist today in `run/mod.rs`. They are `replay_holds_over_threshold_rejects_the_job_and_records_the_list`, `replay_within_threshold_records_the_list_and_lands`, `a_check_killed_at_its_deadline_closes_as_an_environment_stop`, `low_disk_before_the_verifier_resumes_without_planner_or_producer`, `interrupted_job_with_a_candidate_resumes_without_planner_or_producer`, `a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order`, and `run_pin_tests.rs`.
- Criterion 2: The producer records the test count and a hash of the sorted names from `cargo test --locked -- --list` at the base, before editing, and again after, and writes both in the CHANGELOG entry. The verifier reruns the list after, counts the names, and compares with the recorded base count. The verifier also reads `git diff -U0` of `run/mod.rs`, which cannot be rerun at the base since no checkout command is allowed. Every hunk must lie before the first `#[cfg(test)]` (the `mod`/`use` lines near the top and the cut in `check_and_verify`). Run: `cargo test --locked -- --list && git diff -U0 -- crates/athena-core/src/run/mod.rs`. Observes: The count equals the recorded base count and no test name contains `verification`. No hunk touches the two test modules or `check_seam`, and `git diff --stat` shows only the three paths.
  Fails without the behavior: If a test were moved, renamed or edited, the list would differ and a hunk in the test modules (from `mod critic_soundness_tests` at about line 1908 to the end) would show in the `-U0` diff. If the verification module accidentally held a `#[test]`, the list would gain names under `run::verification`.
- Criterion 3: Read the diff. In the new file, the S8-S10 block and the `verifier_snapshot` body appear as moved lines. The unmoved lines should be: the module doc, `use super::*;`, the `Checked` struct, the `verification_stage` signature, the rebinding lines (`id`, `jc`, `p`, `c`, `f`, `seq`, `report_gaps`, `left_criteria`, `usable_baseline`, the destructure of `checked`), `pub(super)` on `verifier_snapshot`, and rustfmt wrapping. In `mod.rs` the only added lines are `mod verification;`, `use verification::Checked;`, the `Checked { .. }` construction, the call, the dropped bindings and the removal of `mut`. Run: `git diff -M --color-moved=zebra --color-moved-ws=ignore-all-space -- crates/athena-core/src/run/mod.rs crates/athena-core/src/run/verification.rs`. Observes: Every moved line is marked moved apart from that list, and the producer's CHANGELOG entry or report names each exception.
- Criterion 4: The `sed` range counts the lines of `produce_and_verify` after the change. The verifier compares them with 57 at the base (lines 1088-1144) and checks by reading that the body is unchanged: `baseline_stage`, `producer_snapshot`, `producer_loop`, `candidate_stage`, `remove_tree`, `Built`, `write_resume_mark`, `check_and_verify`, in that order. The body is not edited and a 2-3 line doc comment is added above it, so the count is 57 for the body, or about 60 with the doc. `check_and_verify` is not the driver the brief names, and the verifier should read that as the reason for the number. Run: `git show HEAD:crates/athena-core/src/run/mod.rs | grep -c '' ; grep -n 'fn produce_and_verify' crates/athena-core/src/run/mod.rs; sed -n '/fn produce_and_verify/,/^    }$/p' crates/athena-core/src/run/mod.rs | wc -l`. Observes: A count of 57 to 60 and the stage calls in the original order. The brief's 60-100 range is met only at its lower edge; see open_questions.
- Criterion 5: Build the workspace and the test targets, then read `git diff --stat`. Search for `pub ` lines added or removed in the diff. Run: `cargo build --locked && cargo test --locked --no-run && git diff --stat`. Observes: The build succeeds. The diff stat lists only `mod.rs`, `verification.rs` and `CHANGELOG.md`, and no added or removed line has a plain `pub` item. Everything moved is private or `pub(super)`, since nothing moved was public, so no re-export is needed.
  Fails without the behavior: A moved name that stopped resolving would break the build or the test build. A changed `pub` path would break the `athena-cli` build, which uses `athena_core` items from `main.rs` and `drain.rs`.
- Criterion 6: Read the new entry. It sits first under `### Changed` and is split into short bullets. Run: `git diff -- CHANGELOG.md`. Observes: Bullets cover `verifier_snapshot`, `verification_stage` and `Checked`. Each has its Grep command and file counts before and after (equal, plus `verification.rs`; `verifier_snapshot` also names the unrelated fn of that name in `athena-git/src/snapshot.rs` tests). The entry also gives the visibility changes (`pub(super)` on the moved method, `verifier_snapshot`, `Checked` and its fields) and the early-return pairing: old 1446 `return Ok(closing)` became the same line in `verification.rs` at a line number the producer records, returning to a caller that returns it unchanged. It also states the test count before and after, and says the R7 entry was not edited.
- Criterion 7: List the changed paths. Run: `git status && git diff --stat`. Observes: None of `docs/design/structure-pass.md`, `athena.toml`, `defaults.toml` or `.github/workflows/*` appears, nor do `producer.rs`, `checks.rs`, `candidate.rs`, `main.rs` or `drain.rs`.

Planned files:

- crates/athena-core/src/run/verification.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f74d18ddef0e → candidate 8cc06b60f1ac. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  32 +++
 crates/athena-core/src/run/mod.rs          | 376 +++------------------------
 crates/athena-core/src/run/verification.rs | 395 +++++++++++++++++++++++++++++
 3 files changed, 457 insertions(+), 346 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1737 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all exit 0.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
