# Job 20261012-183133-ghjf: verify "Add independent review of the squash-chain landing design"

## Request (verbatim)

> review the squash-chain landing design independently

## Reading (accepted by the user)

Write an independent critical review of docs/design/squash-chain-landing.md. Check it against the gate code, landing tests and decisions.md. Save it in docs/design/reviews/ following the existing naming, and change no code.

Assumptions:
- Naming follows the existing <design>-review-N.md convention, so the file is squash-chain-landing-review-1.md.
- The review is read-only with respect to code, config and records.
- The paths jobs.rs, invariant.rs, landing_tests.rs and docs/decisions.md are as in the earlier reading; I did not verify them.

## Acceptance criteria

1. [content] A new file docs/design/reviews/squash-chain-landing-review-1.md exists, following the naming of the other files in docs/design/reviews/, written for an outside reader.
2. [content] Each finding cites a specific section of docs/design/squash-chain-landing.md plus the related code (crates/athena-gate/src/jobs.rs, invariant.rs, landing_tests.rs) or a docs/decisions.md entry.
3. [content] Each finding has a severity, evidence, and a recommended change. The review ends with a verdict: approve, approve with changes, or reject.
4. [content] The review checks whether the design's claims match what the code and landing_tests.rs do, and lists untested failure modes.
5. [behavior] Run `git status` and `git diff`: only files under docs/ changed and the only new file is the review. Run `cargo test` to confirm nothing broke.

## Expected deliverables

- docs/design/reviews/squash-chain-landing-review-1.md (create)

## Out of scope

- Changing the design document itself
- Changing code, tests, config or records
- Fixing the issues the review finds

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every finding cites the file and section it rests on.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new docs file only.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier checks that only docs changed and that cargo test passes.

## The change

Base fdded9df71e1 → candidate aac82e4f89f4. In your working directory, `git diff HEAD~1` shows it.

```text
 .../reviews/squash-chain-landing-review-1.md       | 272 +++++++++++++++++++++
 1 file changed, 272 insertions(+)
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
