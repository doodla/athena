# Job 20261026-232440-0x6r: verify "Move take_baseline, baseline_checks, SnapExec to run/checks.rs"

## Request (verbatim)

> Priority: high
> Structure pass job R7 (docs/design/structure-pass.md, section 4). Split step 3: S6-S7 into run/checks.rs with take_baseline, baseline_checks and SnapExec. No behavior change. Depends on 20261026-232437-69k1.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/checks.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass R7 (split step 3): move SnapExec (with its Exec impl), take_baseline and baseline_checks verbatim from run/mod.rs into a new run/checks.rs, with no behavior change. It builds on the landed R6 job 20261026-232437-69k1; the audience is the maintainers and verifier checking the section 5 proof.

Assumptions:
- Prior job 20261026-232437-69k1 (R6) landed with verdict pass. Its findings are addressed as follows. (1) The CHANGELOG lacked old and new line numbers for the early-return pairing. That finding concerns R5 to R8 reporting, so this job's CHANGELOG entry pairs any early return with its Flow::Stop and gives old and new line numbers; this job probably moves none. (2) The R6 done-marker in structure-pass.md is not added here. Editing that doc is out of scope, as it was in R6, and a later job adds the R5 to R7 pointers. (3) The merged candidate_stage nit is a design departure that already landed and is left unchanged. The R6 candidate files (candidate.rs, mod.rs, CHANGELOG.md) are kept as landed state and built on, not redone.
- Code survey: read crates/athena-core/src/run/mod.rs. SnapExec is at about lines 994-1004 and its Exec impl at 1042-1068. check_seam, a cfg(test) module that SnapExec::exec reads, sits between them at 1010-1040. take_baseline and baseline_checks are in an impl Athena block at 1126-1193. Callers: producer.rs:53 calls take_baseline, and mod.rs:1348 constructs SnapExec. I also listed crates/athena-core/src/run/, where candidate.rs, close.rs, producer.rs, send_back.rs, pin.rs and planning.rs exist. check_seam stays in mod.rs so the test paths are unchanged. No record or data file is added, so there is no check_record_file or AnyRecord concern. No refusal rule changes, so there is no refusal-rate criterion.
- The moved methods stay in an `impl Athena` block in checks.rs, with pub(super) visibility where callers outside the file need it. SnapExec keeps its fields visible to mod.rs, where the struct literal at about line 1348 builds it. The struct is re-exported or imported so existing paths resolve.
- Only import lines change in existing tests. If a moved item's doc comment or a helper such as check_seam needs a different visibility, that is the only non-verbatim edit and it is reported.

## Acceptance criteria

1. [content] crates/athena-core/src/run/checks.rs exists and holds SnapExec with its Exec impl, take_baseline and baseline_checks. Their bodies are textually identical to the originals in run/mod.rs, apart from visibility and import lines, and mod.rs no longer defines them (Grep 'fn take_baseline|fn baseline_checks|struct SnapExec' finds them only in checks.rs).
2. [content] Public paths are unchanged. Any item reachable before the move, such as athena_core::run::..., is still reachable through a re-export or import in mod.rs. The git diff of test code shows only edited import lines, and no test is added, removed or renamed.
3. [content] CHANGELOG.md has an entry for R7 that lists each moved name with its caller check (producer.rs take_baseline call, mod.rs SnapExec construction) and states that no behavior changed. It pairs any early return with its Flow::Stop using old and new line numbers, or says there are none.
4. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. All pass, and the test count equals the pre-change count (1737 in the R6 landing).
5. [behavior] Run `cargo test --locked -- --list` on the change. Its test list has the same test names as before the change, apart from module paths that shift only if tests moved. None are expected to move.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)

## Out of scope

- Moving S8 and later stages or job R8
- Any behavior change, or test edits beyond import lines
- Editing docs/design/structure-pass.md, including the R5 to R7 done-markers
- Re-doing candidate.rs, producer.rs, send_back.rs or close.rs, which are kept as landed
- Editing athena.toml, defaults.toml, workflows, or callers in main.rs or drain.rs
- Rewriting produce_and_verify as a state machine

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require fmt, clippy and the full cargo test --locked to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check unchanged text, an unchanged test list and passing test runs, not that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move with no behavior change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier reruns fmt, clippy and test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The R6 findings are addressed by pairing early returns in the CHANGELOG, not just in the report.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 604a2667f76a → candidate e775db1460c4. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  14 ++++
 crates/athena-core/src/run/checks.rs | 120 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/mod.rs    | 111 +-------------------------------
 3 files changed, 136 insertions(+), 109 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1737 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. All pass, and the test count equals the pre-change count (1737 in the R6 landing).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
