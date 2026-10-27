# Job 20261026-232420-ctt9: verify "Add produce_and_verify ordering characterization test (R0)"

## Request (verbatim)

> Priority: high
> Structure pass job R0 (docs/design/structure-pass.md, section 4). Add the one characterization test: a fake job through produce_and_verify asserting the order of ui.line output and the invocation numbers. Edits no other test. No behavior change. 
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run.rs, CHANGELOG.md

## Reading (accepted by the user)

Add one new test in crates/athena-core/src/run.rs that drives a fake job through produce_and_verify and asserts the order of ui.line output and the invocation sequence numbers. Add a CHANGELOG entry. No behavior change, no other test edited. The audience is the structure-pass owner, who needs R5-R8 guarded on ordering.

Assumptions:
- The test goes in the existing tests module of run.rs, reusing existing fake-job helpers where possible.
- The section 5 move-proof items hold vacuously since R0 moves nothing.

## Acceptance criteria

1. [behavior] Run cargo test --locked: a new test drives a fake job through produce_and_verify and passes; it asserts the exact order of ui.line output and the invocation numbers.
2. [content] The diff only adds one test (plus helpers it needs) in run.rs; no existing test and no non-test code is changed, and the earlier test list is unchanged except for the one new name.
3. [content] The test would fail if closing.baseline were set after the producer instead of before (the check in design doc section 5, R0 row); the report says how this was established.
4. [behavior] cargo fmt --all --check and cargo clippy --all-targets --locked -- -D warnings pass.
5. [content] CHANGELOG.md has an entry describing the new characterization test.
6. [content] Only the files run.rs and CHANGELOG.md change; nothing is moved, so the section 5 move proof (re-exports, caller checks) holds trivially and the report states that.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving or splitting any code (R1-R9)
- Editing existing tests
- Any behavior change

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test, with cargo test and clippy passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts ordering behavior, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One test, two files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs test, fmt, clippy.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 78d075ca5b74 → candidate f79f30e4996b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                  |   5 +
 crates/athena-core/src/run.rs | 279 ++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 284 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1396 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --all --check and cargo clippy --all-targets --locked -- -D warnings pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
