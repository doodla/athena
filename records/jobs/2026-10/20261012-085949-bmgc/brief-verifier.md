# Job 20261012-085949-bmgc: verify "Write review 2 of the revised study design, passing policy test"

## Request (verbatim)

> try again on 20261012-084057-0eaq

## Reading (accepted by the user)

Re-check the revised study-capability M1 design (docs/design/study-capability-m1.md) against review 1's findings and write it up as review 2. The prior attempt failed only because review 2 used a word the repository's boundary test bans ("operators"); reword so cargo test passes.

Assumptions:
- Review 2 does not exist in the current tree; the earlier attempt's file was discarded, so it must be written again.
- 'Re-check' means a written design review per the design's section 9 gate, not code changes.
- The banned-word list in policy.rs is not to be edited.

## Acceptance criteria

1. [content] docs/design/reviews/study-capability-m1-review-2.md exists, checks each review-1 finding and decision against the revised design, and gives a verdict (approve, or approve with named changes), citing sections.
2. [content] The review contains none of the words banned by the boundary scan in xtask/tests/policy.rs (e.g. 'operators'); it never refers to who runs Athena or where.
3. [content] If the design's status or review-gate section is updated to record review 2, links resolve and no governance file is changed.
4. [behavior] Run `cargo test --locked`: all tests pass, including athena_refers_to_no_one_who_runs_it_or_where.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --locked`: both clean.

## Expected deliverables

- docs/design/reviews/study-capability-m1-review-2.md (create)
- docs/design/study-capability-m1.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Building the study capability
- Editing policy.rs or any governance file
- Changing records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Review findings cite design sections and review-1 items.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review written for an outside reader.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test must pass, including the boundary policy test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One review document, minimal edits elsewhere.

## The change

Base e89340ae2c84 → candidate 93e0b3efadbe. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                       |   5 +
 .../design/reviews/study-capability-m1-review-2.md | 131 +++++++++++++++++++++
 docs/design/study-capability-m1.md                 |  13 +-
 3 files changed, 144 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (232 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
