# Job 20261026-232437-69k1: verify "Move candidate and admission stages S4-S5 into run/candidate.rs"

## Request (verbatim)

> Priority: high
> Structure pass job R6 (docs/design/structure-pass.md, section 4). Split step 2: S4-S5 into run/candidate.rs. No behavior change. Depends on 20261026-232434-6wfq.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/candidate.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job R6 (docs/design/structure-pass.md section 4): move stages S4 (candidate build from the snapshot's change list) and S5 (admission) of `produce_and_verify` from run/mod.rs into new run/candidate.rs as stage methods, bodies unchanged, early returns as Stop, with no behavior change. Audience is the maintainer and the verifier judging the section 5 proof. Prior job 20261026-232434-6wfq (R5, rejected on a `cargo test` failure plus minor findings) is addressed below; the landed producer.rs is a given, and R6 builds on it.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs read. `produce_and_verify` (about line 1193) now calls `baseline_stage`, `producer_snapshot` and `producer_loop` (from run/producer.rs, returning `Flow::Go(Reported{..})` or `Flow::Stop`). S4 is the code from `snapshot.change_list()` through the idle-producer failure (about 1222-1257). S5 is the blob copy, TempIndex, commit_tree and `admit` through the violations return (about 1258-1356). The tail (`write_resume_mark`, `check_and_verify`) stays. The entry point is `produce_and_verify`; no records or data files are added, so no check_record_file or AnyRecord change. Behavior is measured by the existing test suite via cargo test --locked.
- Job 20261026-232434-6wfq (R5): its candidate is not carried over. The reading assumes the R5 split is present in the tree (producer.rs exists, mod.rs uses Flow). Its findings: (a) the failed `cargo test --locked` is addressed by requiring cargo test to pass here, with no test edits beyond imports; (b) the minor note that the design doc says `Stage::Stop` while the code says `Flow::Stop` is out of scope (docs/design/structure-pass.md is not a deliverable), and R6 uses the existing `Flow` type; (c) the `&mut closing` to `closing` change and (d) seq by value are R5 choices already in place. R6 may make a similar clippy-forced change only if the CHANGELOG states it.
- The new stage methods are pub(super) on `Athena` in run/candidate.rs. Closing goes in as `&mut Closing`, and values flow out through a small struct, as in R5. The idle-producer failure and the admission refusal become `Flow::Stop`, and the driver returns `Ok(closing)` on Stop.
- Public paths stay unchanged, with re-exports in mod.rs if any moved item was public. The report lists the caller check (Grep) for each moved name. The design doc says R6 is high-risk because the idle-producer message depends on `report_gaps` and `report_kept_in`, so they must stay byte-identical in the message.

## Acceptance criteria

1. [content] crates/athena-core/src/run/candidate.rs exists and holds the S4 and S5 code (candidate construction from `snapshot.change_list()`, the idle-producer failure text, blob copy, TempIndex/commit_tree, `admit`, the violations failure). Moved statements are textually identical apart from the Stop returns and parameter plumbing, as `git diff` shows.
2. [content] In crates/athena-core/src/run/mod.rs, `produce_and_verify` calls the candidate stage method(s) and returns `Ok(closing)` on Stop, then continues unchanged to `write_resume_mark` and `check_and_verify`. Grep shows the S4/S5 bodies no longer in mod.rs.
3. [content] The idle-producer message strings ("the producer made no change", "its report still had problems when sent back", "the report is kept in") and the "admission refused the change:" string appear unchanged in candidate.rs, and the changed_paths take/restore order around `self.failure` is preserved.
4. [content] Section 5 proof: the diff to existing test code touches only import lines, the test list is unchanged (no test added, removed or renamed), and public paths are unchanged through re-exports. Grep confirms each moved name's callers, and the CHANGELOG entry lists the caller check per moved name.
5. [content] CHANGELOG.md has a new entry for R6 describing the move, stating no behavior change, and naming any statement altered beyond a name.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass, with the same number of tests as before the change.
7. [content] Only crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/candidate.rs and CHANGELOG.md change; `git status` shows no other path. No governance file (athena.toml, defaults.toml, workflows) changes.

## Expected deliverables

- crates/athena-core/src/run/candidate.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving S6 and later stages (checks, verification, assembly) or jobs R7-R8
- Any behavior change, or test edits beyond import lines
- Editing docs/design/structure-pass.md, including the `Stage::Stop` versus `Flow::Stop` naming from R5
- Re-doing or rewriting the R5 files run/producer.rs, run/send_back.rs or run/close.rs; the R5 candidate itself (CHANGELOG, mod.rs, producer.rs, structure-pass.md) is dropped as such and only its landed state is built on
- Editing athena.toml, defaults.toml, workflows, or callers in main.rs or drain.rs
- Rewriting `produce_and_verify` as a state machine

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Existing tests must pass, clippy must be clean, and no test lines change beyond imports.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move of S4-S5 into one file.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: fmt, clippy and test are run by the verifier.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the strings, the order and the passing test suite, not just that the files exist.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The R5 test failure is addressed by requiring a green cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and module docs are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base eae351d07f79 → candidate 8b60909e16d6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  12 +++
 crates/athena-core/src/run/candidate.rs | 169 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/mod.rs       | 151 +++-------------------------
 3 files changed, 196 insertions(+), 136 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1737 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass, with the same number of tests as before the change.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
