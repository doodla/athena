# Job 20261014-075345-x9cx: verify "Forbid verifiers from presenting snapshot runs as real-repo results"

## Request (verbatim)

> Verifiers have reported runs in their snapshot, which has no records/, as results on the real repository; that false claim reached main in PR #59. Stop that from happening.

## Reading (accepted by the user)

Read as a prompt fix: the verifier's snapshot lacks records/, history and private files, so any run there (cargo test, policy checks) describes only the snapshot. The verifier prompt must say this and require findings to label such runs as snapshot results and never claim anything about records/ or the real repository. A test pins the wording.

Assumptions:
- The false claim came from verifier prompt wording, not from code that fabricates results.
- Fix is a prompt rule plus a pinning test; no new gate that scans verdicts for such claims.
- PR #59 content itself is not rewritten; that is history.

## Acceptance criteria

1. [content] crates/athena-context/prompts/verifier.md states that the verifier's working directory is a snapshot without records/, owner files, or history, and that anything it ran there is a result about the snapshot only.
2. [content] The prompt tells the verifier to say so explicitly in evidence and findings whenever a command or its output depends on records/ or other absent paths, and never to report such a run as a result on the real repository.
3. [content] A unit test in athena-context asserts that VERIFIER_FRAME contains the new snapshot-scope wording, so removing it fails the test.
4. [behavior] Run `cargo test` from the repository root: it passes, including the prompt budget tests and the new test.
5. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean.

## Expected deliverables

- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Editing records/ or already-merged content from PR #59
- A mechanism that automatically detects false claims in verdicts
- Changing the snapshot contents

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the verifier prompt that lets snapshot runs be reported as real-repo results, not just the one claim.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a test pinning the new wording; cargo test and clippy pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt change and a test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces the prompt wording; the rule itself stays prose for the model.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 64bc795c7f75 → candidate 8cfa27f9ea79. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |  5 +++++
 crates/athena-context/prompts/verifier.md | 11 ++++++-----
 crates/athena-context/src/lib.rs          | 14 ++++++++++++++
 docs/decisions.md                         |  4 ++--
 4 files changed, 27 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (537 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
