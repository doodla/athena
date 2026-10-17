# Job 20261017-014911-5vnx: verify "Drain: accept unchanged-scope rereadings; ignore irrelevant config chang"

## Request (verbatim)

> Drain at 01:48 UTC landed nothing: after an athena.toml change (max_concurrency 1 to 2), every proposed item is read again, the new reading 'needs accepting', drain records 'no outcome', then tries to retry a proposed job ('only a rejected or failed job can be tried again') and stops on 'repeated failure'. Drain should accept a re-reading whose scope did not change and work it, and a config change irrelevant to a reading should not invalidate it.

## Reading (accepted by the user)

When athena.toml changes after a job was read, drain currently rereads each proposed item, gets a reading "needing accepting", records "no outcome", then wrongly retries a still-proposed job and stops on "repeated failure". Fix so that a config change not touching reading inputs (e.g. max_concurrency) doesn't invalidate readings, and a reread whose scope is unchanged is accepted and run by drain; the retry must only follow a rejected or failed job. Audience: the operator running unattended drains.

Assumptions:
- Scope equality means same capability, class_of_work and the same deliverable paths (action) in the new reading versus the one it replaces; probabilities and wording may differ.
- Config relevance is decided by comparing a projection of the parsed config (excluding fields such as max_concurrency) rather than the raw file hash; defaults.toml changes still invalidate.
- Acceptance by drain uses the existing AcceptedBy::Flag, as drain already passes it.
- No change to athena.toml or defaults.toml is needed; the config change in the report was the trigger, not the fix.

## Acceptance criteria

1. [behavior] A test in crates/athena-cli/tests/drain.rs changes only max_concurrency in athena.toml after items are proposed/read; the drain lands them with no reread and no 'needs accepting' or 'no outcome' line. Run: cargo test -p athena-cli --test drain.
2. [behavior] A test changes a reading-relevant config value (e.g. a budget cap or capability definition), so drain rereads; when the new reading has the same capability and deliverables as the old, drain accepts it and runs the job to a landed outcome in the same drain. Run: cargo test.
3. [behavior] A test where the reread scope differs (different capability or deliverables) does not run silently: the drain leaves the item with a clear reason that the reading needs accepting, and does not stop with 'repeated failure'. Run: cargo test.
4. [behavior] Drain never calls retry on a job that is still proposed: when a run ends with no outcome, the drain's next attempt is a run/accept of that job, not 'only a rejected or failed job can be tried again'. A regression test reproduces the 01:48 sequence (max_concurrency 1 to 2) and asserts the stop reason is not 'repeated failure'. Run: cargo test -p athena-cli.
5. [behavior] Reading validity compares only the configuration inputs that bear on a reading; the existing test that a relevant configuration or defaults.toml change forces a reread still passes. Run: cargo test -p athena-core.
6. [behavior] cargo test passes, cargo clippy --all-targets is clean, and cargo fmt --check passes.
7. [content] Doc comments on GoResult::Reread and the reading-validity rule (D38) say what counts as a relevant config change and that a same-scope reread is accepted by drain; CHANGELOG notes the fix.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Changing the retry rule that only rejected or failed jobs can be tried again
- Changing interactive `athena go` acceptance behavior for readings whose scope changed
- Editing athena.toml or defaults.toml
- Changing drain budget or concurrency semantics

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fix and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Regression tests; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix both the over-broad invalidation and the drain's retry of a proposed job, not only the instance.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain scenario and check landing.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in core run/capability and drain.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by tests rather than prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 9a4b10fcb60a → candidate a18c6cbd6ee2. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  12 ++
 crates/athena-cli/src/drain.rs       |  86 +++++++++++--
 crates/athena-cli/tests/drain.rs     | 240 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/capability.rs |   5 +-
 crates/athena-core/src/reading.rs    |  19 +++
 crates/athena-core/src/run.rs        |  54 +++++++-
 crates/athena-data/src/config.rs     |  28 ++++
 7 files changed, 424 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (792 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "Reading validity compares only the configuration inputs that bear on a reading; the existing test that a relevant configuration or defaults.toml change forces a reread still passes. Run: cargo test -p athena-core."; "cargo test passes, cargo clippy --all-targets is clean, and cargo fmt --check passes."; "Doc comments on GoResult::Reread and the reading-validity rule (D38) say what counts as a relevant config change and that a same-scope reread is accepted by drain; CHANGELOG notes the fix.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
