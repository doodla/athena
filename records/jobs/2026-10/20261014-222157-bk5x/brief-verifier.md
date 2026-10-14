# Job 20261014-222157-bk5x: verify "Require producer to run full cargo test --locked; return not-run"

## Request (verbatim)

> Change the producer prompt (line 8 says the producer 'may' run the checks) so the producer must run the full `cargo test --locked` before it finishes, even for docs, TOML or data-only changes. Policy and config tests read tracked non-Rust files: boundary words, unlisted file kinds, defaults.toml, and study model.json. A candidate whose full test run is 'not-run' should be sent back to the producer instead of going on to verification.

## Reading (accepted by the user)

Replace line 8 of the producer frame prompt with a must-run rule for `cargo test --locked` on every change, including docs, TOML and data-only ones. Add a gate in the job flow: if the producer's report shows that test run as `not-run`, re-invoke the producer once rather than passing the candidate to verification. Audience: the Athena maintainer, who judges whether docs-only candidates stop reaching verification with an unrun test suite.

Assumptions:
- Today no code path re-invokes the producer: produce_and_verify in crates/athena-core/src/run.rs calls it once, and the candidate then goes straight to checks and verification. 'Sent back to the producer' therefore needs a small bounded retry in that function, which I assume the user wants.
- The retry is limited to one extra producer invocation, to bound cost. If it is still not-run, the job fails with a clear reason and does not reach verification.
- The gate keys on the producer's reported checks_run: an entry whose command is `cargo test --locked` with result not-run, or no such entry. Athena's own check runner is unchanged, though its skip_when_only patterns for docs and *.md remain; athena.toml line 79 already says the test check never skips.
- The producer's Bash grant must allow `cargo test --locked`. athena.toml already lists cargo test as allowed, so no governance file changes. If the grant needs a change, that is a separate decision.
- Only the shared producer frame (crates/athena-context/prompts/producer.md) changes. The capability-specific producer.md files are left alone.

## Acceptance criteria

1. [content] crates/athena-context/prompts/producer.md no longer says the producer 'may' run the checks. It requires the producer to run the full `cargo test --locked` before finishing, for every change including docs, TOML and data-only ones. It names the reason: policy and config tests read tracked non-Rust files (boundary words, unlisted file kinds, defaults.toml, study model.json).
2. [content] The prompt says a `not-run` full test result is not an acceptable finish, and that the producer must report the real result in checks_run.
3. [behavior] A candidate whose producer report lists the full `cargo test --locked` as `not-run` (or lists no such check) is not sent to the verifier. The producer is re-invoked with a message explaining why, and a bounded retry count applies. Run `cargo test` and check that the new test for this gate passes.
4. [behavior] A candidate whose report shows the full test run as pass or fail continues to verification as before. Run `cargo test` and check that the existing lifecycle and run tests still pass.
5. [behavior] `cargo fmt --check` and `cargo clippy` are clean, and prompt budget tests (crates/athena-cli/tests/budgets.rs) still pass with the longer prompt.
6. [content] Docs that describe the producer's checks (docs/design/athena-v0.md) match the new rule and the send-back behavior. The CHANGELOG records the change.

## Expected deliverables

- crates/athena-context/prompts/producer.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing Athena's own check runner or the skip_when_only patterns in athena.toml
- Changing the verifier or reader prompts
- Changing the study capability's producer prompt
- Making the producer run clippy or fmt as a hard requirement

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new gate gets a test, and cargo test and clippy must stay clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The prompt rule and the gate that enforces it are both changed, so the rule is not left as an unenforced request.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The not-run rule is enforced in the job flow, not only stated in prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One retry, no new config keys, and no changes to the check runner.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The prompt text states the reason for the rule, so an outside reader understands why docs-only changes still need the full test run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 94b873c17ddc → candidate 57c9f15a0b23. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |   8 ++
 crates/athena-agent/src/fake.rs           |   2 +-
 crates/athena-cli/tests/common/mod.rs     |  21 +++-
 crates/athena-cli/tests/lifecycle.rs      | 107 +++++++++++++++++++-
 crates/athena-context/prompts/producer.md |   3 +-
 crates/athena-context/src/lib.rs          |   2 +-
 crates/athena-core/src/run.rs             | 156 +++++++++++++++++++-----------
 docs/decisions.md                         |  22 +++++
 docs/design/athena-v0.md                  |  13 ++-
 9 files changed, 268 insertions(+), 66 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (587 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
