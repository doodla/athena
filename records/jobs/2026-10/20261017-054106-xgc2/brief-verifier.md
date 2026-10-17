# Job 20261017-054106-xgc2: verify "Route checks by job kind per ci-routing 3.2, with athena.toml"

## Request (verbatim)

> try again on 20261017-044557-3htd

## Reading (accepted by the user)

Add a routing function in athena-core mapping a plan's class_of_work to which checks run, which are skipped, and where the result comes from (reused CI result via item 4 `resolve`, or local), following the section 3.2 table. Unlike the failed attempt, athena.toml is listed as a deliverable and named in a criterion, because the per-kind routing rules are expressed there. Audience: the Athena maintainer and ci-routing.md reviewers.

Assumptions:
- Section 3.2 is the source of truth: code/tests run fmt, clippy, test; docs/design/study skip fmt and clippy but always run test; config never skips; build is as code plus the workflow's own run and never reuses a prior result.
- Missing or unknown kind takes the config row (all three checks, none skipped), per section 7.
- athena.toml is edited to carry the per-kind routing (for example a per-kind skip rule beside the existing [[capabilities.change.checks]] entries), and CheckSpec in capability.rs is extended only if the schema needs a new field.
- Routing consults item 4's `resolve` with the candidate SHA: a CI result for another SHA is ignored and falls back to local; a red CI result fails without a local rerun.
- No speedup is measured; the doc update says so.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core routing`: for each kind (code, tests, docs, design, config, build, study, other) the routed checks and skips match the section 3.2 table (docs, design and study skip fmt/clippy but always run test; config never skips; build adds the workflow's own run and never reuses a prior result).
2. [behavior] Run `cargo test -p athena-core routing`: a missing or unrecognised job kind gets the config row (fmt, clippy, test, none skipped), per section 7.
3. [behavior] Run `cargo test -p athena-core`: a test shows the routed path calls `resolve` with the candidate SHA; a CI result for another SHA is ignored and falls back to a local run; a red CI result fails and does not rerun locally.
4. [behavior] Run `cargo test` (including a test that loads the committed athena.toml with the schema in the tree): athena.toml parses, and representative jobs of each kind run the checks section 3.2 assigns to that kind.
5. [content] athena.toml is modified to express the per-kind routing, and `git diff` shows the change; if CheckSpec needed new fields, the schema change is in the same commit.
6. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both pass; `cargo test` passes in full.
7. [content] docs/design/ci-routing.md section 10 item 5 states what was built and what is not done (for example no speedup measured), without claiming a measured improvement; CHANGELOG.md has an entry.
8. [content] The routing is called from the job run or verifier input path (run.rs / verify.rs), not only defined; a reader can find the call site that selects checks by the plan's kind.

## Expected deliverables

- crates/athena-core/src/routing.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/design/ci-routing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Measuring speedup or running the section 9 baseline
- Editing .github/workflows beyond what already exists
- Widening the verifier's allowed commands
- Changing defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Routing tests plus cargo test, clippy, fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise each kind and the SHA-mismatch and red-CI paths.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the checks that run per kind, not that the file exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on routing only.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Routing rules live in athena.toml and code, enforced by tests.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Doc update claims no measured speedup.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 07cb1ed04ae5 → candidate 5de36fe05cec. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   8 +
 athena.toml                          |  11 +
 crates/athena-cap-change/src/lib.rs  |   1 +
 crates/athena-cap-study/src/lib.rs   |   1 +
 crates/athena-core/src/capability.rs |   2 +
 crates/athena-core/src/checks.rs     |   5 +
 crates/athena-core/src/lib.rs        |   3 +
 crates/athena-core/src/routing.rs    | 471 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/run.rs        |  50 +++-
 crates/athena-data/src/config.rs     |  18 ++
 docs/design/ci-routing.md            |  15 ++
 11 files changed, 583 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (822 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "Run `cargo test -p athena-core routing`: a missing or unrecognised job kind gets the config row (fmt, clippy, test, none skipped), per section 7."; "Run `cargo test -p athena-core`: a test shows the routed path calls `resolve` with the candidate SHA; a CI result for another SHA is ignored and falls back to a local run; a red CI result fails and does not rerun locally."; "Run `cargo test` (including a test that loads the committed athena.toml with the schema in the tree): athena.toml parses, and representative jobs of each kind run the checks section 3.2 assigns to that kind."; "athena.toml is modified to express the per-kind routing, and `git diff` shows the change; if CheckSpec needed new fields, the schema change is in the same commit."; "Run `cargo clippy --all-targets` and `cargo fmt --check`: both pass; `cargo test` passes in full."; "docs/design/ci-routing.md section 10 item 5 states what was built and what is not done (for example no speedup measured), without claiming a measured improvement; CHANGELOG.md has an entry."; "The routing is called from the job run or verifier input path (run.rs / verify.rs), not only defined; a reader can find the call site that selects checks by the plan's kind.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
