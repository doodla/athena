# Job 20261013-034916-s3wg: verify "Carry the resolved pin in study briefs; exempt it from the hash rule"

## Request (verbatim)

> Study briefs must carry the pin Athena resolved (url, ref, SHA), and the 'never cite a commit hash' brief rule must not apply to the pinned instance: job kmh0 was rejected for a placeholder SHA.

## Reading (accepted by the user)

Studies' producer and verifier briefs get a "Pinned source" section with the pin Athena resolved (url, ref, full SHA), taken from the job record's PinnedRepo. The "never pin or cite a commit hash" rule is reworded so it still bars verification-made hashes but explicitly allows the pinned instance's SHA.

Assumptions:
- 'Study briefs' means the producer and verifier briefs of a job whose capability pins a repository (job.pin is Some); reader briefs are unchanged.
- The exemption applies only to the SHA Athena resolved and put in the brief; hashes made during verification remain forbidden.
- The pin is rendered from the existing PinnedRepo record; no new pinning logic.

## Acceptance criteria

1. [content] ProducerInputs and VerifierInputs in crates/athena-context/src/briefs.rs gain an optional pin (athena_data PinnedRepo: url, ref, sha); producer_brief and verifier_brief render a 'Pinned source' section with all three values only when it is set, and omit it otherwise.
2. [content] crates/athena-core/src/run.rs passes the pin resolved by pin_repo into the producer brief (and into the verifier inputs), so the brief carries the real resolved SHA, never a placeholder.
3. [content] The VERIFIER_VISIBILITY text no longer forbids citing the pinned SHA: it still bars hashes made during verification, but states that the pinned instance's url, ref and SHA from the brief must be cited as given.
4. [behavior] New unit tests in briefs.rs show a brief with a pin contains the url, ref and full SHA and the exemption wording, and a brief without a pin has no pin section. Run: cargo test -p athena-context.
5. [behavior] Existing tests (including the one asserting the hash rule wording) are updated and pass, and the core's study pin tests still pass. Run: cargo test, cargo clippy --all-targets, cargo fmt --check.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/design/study-capability-m1.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)

## Out of scope

- Changing how Athena resolves or validates pins (pin.rs).
- Reader briefs.
- Studies already committed under studies/.
- Governance files such as athena.toml.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests for the pin section and the reworded rule; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to brief rendering and its call site.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the target (VERIFIER_VISIBILITY and pin_repo) from the repository.

## The change

Base 6f161553cb9f → candidate c43db1595afb. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  4 ++
 crates/athena-context/src/briefs.rs     | 97 ++++++++++++++++++++++++++++++++-
 crates/athena-context/src/lib.rs        |  2 +
 crates/athena-core/src/run.rs           |  2 +
 crates/athena-core/src/run_pin_tests.rs |  7 +++
 docs/design/study-capability-m1.md      |  6 ++
 6 files changed, 115 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (309 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
