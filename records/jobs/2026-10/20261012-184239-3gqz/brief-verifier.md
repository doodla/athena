# Job 20261012-184239-3gqz: verify "Answer review 1 in the squash-chain landing design"

## Request (verbatim)

> answer review 1 of the squash-chain landing design

## Reading (accepted by the user)

Revise docs/design/squash-chain-landing.md so it answers each finding of review 1 (3 major, 5 minor, 2 nit) and the review's three decisions, with a response appendix like the study-capability-m1 design. Documentation only; no code is built.

Assumptions:
- 'Answer review 1' means revising the design to respond to each finding of docs/design/reviews/squash-chain-landing-review-1.md, as was done for study-capability-m1 (response appendix in the design), not writing a new review.
- The user did not answer the review's three decisions, so the producer picks and records defaults: record explicit order (predecessor field) in candidate.json; refuse chains whose jobs change classification rules files; refuse chain landing on commits with more than one parent.
- The design stays proposed and no Rust is built.

## Acceptance criteria

1. [content] docs/design/squash-chain-landing.md is revised so every finding of review 1 (R1-M1, M2, M3, m1 to m5, n1, n2) is answered: fixed in the text, or declined with a stated reason.
2. [content] The design has an appendix mapping each review 1 finding id to the section that answers it, and states the answer to the review's three decisions for the user (chain order, rules-file changes, merge commits), each as a stated assumption.
3. [content] R1-M1: the chain order is defined unambiguously for reverting jobs and for jobs that change no governed content, either by a recorded predecessor or sequence field or by a stated refusal, with the cases added to the test list.
4. [content] R1-M2: §3 B and §4.4 no longer claim the final verifier saw the whole tree. They say the benefit over option E is catching accidental mismatches, not forgery.
5. [content] R1-M3: the design states which rules each recorded fingerprint uses and what happens when a job in the chain changes the path classification rules.
6. [content] The minor findings are covered: jobs::check and history in the follow-up list, the merge-commit case, one schema choice reconciled with D69, partly landed chains, and one refusal test per per-job check on a middle job.
7. [content] The change is documentation only: no Rust code, athena.toml, workflow, or records/ file is touched. A CHANGELOG line is added, and D69 in docs/decisions.md is amended only if the schema choice changes.
8. [behavior] Run `cargo test --locked` (including xtask policy wording scan) and `cargo fmt --check`; both pass.

## Expected deliverables

- docs/design/squash-chain-landing.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- docs/enforcement.md (modify)
- docs/design/reviews/squash-chain-landing-review-1.md (modify)

## Out of scope

- Implementing the chain-aware landing rule in Rust
- Changing Candidate schema in crates/athena-data
- Editing the review file itself
- Changing athena.toml, workflows, or records/

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Found the review file and design in the repository and inferred that answering means revising the design.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design is revised and then reviewed again before any build job.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each answer cites the code or doc it relies on, and untested claims are labeled.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused docs change; no code.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test and fmt check, which include the wording scan.

## The change

Base 73df85135fe4 → candidate 1174ac79d4ea. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |   5 +
 docs/decisions.md                   |  28 ++--
 docs/design/athena-v0.md            |   2 +-
 docs/design/squash-chain-landing.md | 246 +++++++++++++++++++++++++-----------
 docs/enforcement.md                 |   3 +-
 5 files changed, 199 insertions(+), 85 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (267 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
