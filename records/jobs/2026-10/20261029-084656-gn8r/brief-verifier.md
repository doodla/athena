# Job 20261029-084656-gn8r: verify "Retry R5: move produce_and_verify S1-S3 into run/producer.rs"

## Request (verbatim)

> try again on 20261026-232434-6wfq

## Reading (accepted by the user)

Retry of job 20261026-232434-6wfq: move the baseline, producer snapshot and send-back loop (S1-S3) of `produce_and_verify` into stage methods in new run/producer.rs, bodies unchanged, early returns becoming a Stop value. Audience: maintainers and the verifier. The earlier candidate's moved work is kept as the starting point, and its full test run is checked against the unchanged base.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs (produce_and_verify; baseline ui line ~1135, producer-loop early `return Ok(closing)` at ~1223, 1236, 1246, 1256, `producer   sent back:` ~1326, S4 idle-producer return ~1364), plus sibling modules run/send_back.rs, run/close.rs, run/pin.rs, run/planning.rs. Entry point is `produce_and_verify`; the order is baseline, snapshot, producer loop, then S4. No record or data file is added, so check_record_file/AnyRecord are not affected. The property (no behavior change) is measured by cargo test, the test list and a moved-text diff.
- Job 20261026-232434-6wfq: its candidate was rejected because `cargo test --locked` failed (591 tests ran, the failing test was not named in the stored log tail), plus three findings. Its four changed files are kept as deliverables and redone from the base. Findings: (a) design doc named `Stage::Stop` while code used `Flow::Stop` - fixed by making doc and code agree on one name (docs at structure-pass.md lines 125 and ~250 edited); (b) `&mut closing` became `closing` - fixed by keeping the moved text verbatim where possible, or listing the single change with its reason in the CHANGELOG; (c) `seq` by value rather than `&mut u32` - accepted as a nit if documented, or switched to match the design doc.
- The test failure is not assumed to be caused by the move or to pre-exist. The producer first runs `cargo test --locked` on the unchanged base and notes any failing tests by name, then reruns after the move. If the move caused a failure, it is fixed inside the move's scope. If a test already fails on the base, it is reported in the CHANGELOG/report and not fixed here.
- Prior jobs 20261026-232426-91cn (R2), 20261026-232431-dpem (R4) and 20261026-232420-ctt9 (R0) are landed and are not touched.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`. It passes in full, with no failing tests. If any test fails, the recorded output names it, and the CHANGELOG states whether it also fails at the base commit.
2. [behavior] Run `cargo test --locked -p athena-core -- --list`. The test names and count equal those of the base commit, and `produce_and_verify_order_tests::a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order` passes.
3. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both pass, and `git diff` shows no added `#[allow]`.
4. [content] `git diff --color-moved` shows the S1-S3 body text in crates/athena-core/src/run/producer.rs as the same text as the lines removed from run/mod.rs. Lines not marked moved are limited to mod/use/visibility lines, stage method signatures, the Stop/Reported types, local-to-field renames and rustfmt rewraps, each listed with its reason in the CHANGELOG entry.
5. [content] Each early `return Ok(closing)` in S1-S3 (the producer could not run, timed out, did not complete, returned no valid report) is paired in the CHANGELOG with the Stop that replaced it, with old and new line numbers. The driver in run/mod.rs returns `closing` when it sees Stop. The S4 idle-producer `return Ok(closing)` is unchanged.
6. [content] Grep over crates/athena-core/src/run shows the `baseline   ` ui line and the `producer   sent back:` line only in producer.rs, and one definition of each stage method. `produce_and_verify` calls them in the order baseline, snapshot, producer loop.
7. [content] The stop value is named the same in code and in docs/design/structure-pass.md (sections 3.3 and 5); Grep finds no `Stage::Stop` left if the code uses `Flow::Stop`, or the reverse.
8. [content] The CHANGELOG entry lists every local that crossed the S1-S3 seam and the field or parameter that carries it now. For each moved or new name it gives the Grep over crates/ with file counts before and after, and the counts are equal.
9. [content] `git diff --stat` lists only crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, CHANGELOG.md and docs/design/structure-pass.md. No test line changes except `use` lines. CHANGELOG.md has an R5 entry under Unreleased saying there is no behavior change, in the style of the R2 and R4 entries.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving S4 and later stages (candidate, admission, checks, verification, assembly); jobs R6-R8
- Any behavior change, or test edits beyond import lines
- Fixing a test that already fails on the base commit (report it only)
- Fixing the R2 design-doc contradiction about test modules, deduplicating R0 test helpers, or fixing the R0 comment
- Reverting or rewriting landed files run/send_back.rs, run/close.rs or the R0 test, or their CHANGELOG entries
- Editing athena.toml, defaults.toml, workflows, gh/ files, or callers in main.rs or drain.rs
- Rewriting produce_and_verify as a state machine
- Building the code-check that runs baseline tests at admission (the introspection's prevention proposal)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check test results, the test list and the moved-text diff, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move limited to S1-S3.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test failure is classified against the base before any fix.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry is written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 058a721f149e → candidate 04eaa2722db0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  40 +++++
 crates/athena-core/src/run/mod.rs      | 220 ++----------------------
 crates/athena-core/src/run/producer.rs | 304 +++++++++++++++++++++++++++++++++
 docs/design/structure-pass.md          |   9 +-
 4 files changed, 366 insertions(+), 207 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked`. It passes in full, with no failing tests. If any test fails, the recorded output names it, and the CHANGELOG states whether it also fails at the base commit.; Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both pass, and `git diff` shows no added `#[allow]`.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
