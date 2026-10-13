# Job 20261013-204535-djek: verify "Write second review of the revised parallel-jobs design"

## Request (verbatim)

> Review the revised docs/design/parallel-jobs.md as a second review, written to docs/design/reviews/parallel-jobs-review-2.md. Check each answer to review 1's findings F1 to F8 against the source code and docs/design/native-pr-landing.md, look for new problems, and end with a verdict: accept, accept with changes (listing them), or reject. Review only; do not change the design.

## Reading (accepted by the user)

Add docs/design/reviews/parallel-jobs-review-2.md. It checks each answer to review 1's F1-F8 against the source code and native-pr-landing.md, lists new problems, and ends with a verdict. The design is untouched. CHANGELOG.md gets an entry because past reviews have one.

Assumptions:
- Review 1 is docs/design/reviews/parallel-jobs-review-1.md and its findings are labelled F1-F8.
- The CHANGELOG.md entry is allowed because existing review files have entries, for example the squash-chain review 2 entry.
- Claims in the review cite the source code and docs; any number not measured is labelled as an estimate.
- The file follows the format of existing reviews, for example native-pr-landing-review-3.md.

## Acceptance criteria

1. [content] docs/design/reviews/parallel-jobs-review-2.md exists and has one assessment per finding F1-F8 of parallel-jobs-review-1.md, each with a status: resolved, partly resolved, or unresolved.
2. [content] Each F1-F8 assessment cites the source files and functions and the native-pr-landing.md sections it was checked against, and quotes or points to the revised design's answer.
3. [content] A new-problems section lists each problem with a severity and evidence (file or section), or states that none were found.
4. [content] The review ends with a verdict that is exactly one of: accept, accept with changes (changes listed), or reject.
5. [content] CHANGELOG.md has a short entry for the new review, in the style of the existing review entries.
6. [behavior] Run git status and git diff --stat: the only changes are the new review file and CHANGELOG.md. docs/design/parallel-jobs.md and every other file are unchanged.

## Expected deliverables

- docs/design/reviews/parallel-jobs-review-2.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Any change to docs/design/parallel-jobs.md
- Any change to source code, tests, or configuration
- Fixing the problems the review finds
- Changes to review 1 or other earlier reviews

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written so an outside reader can follow it without prior context.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each status cites a file, function, or section; unmeasured numbers are labelled as estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new review file plus a changelog entry.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verification is by git status and git diff on the file set, plus a read of the criteria.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The CHANGELOG convention and the review format were recovered from the repository.

## The change

Base c862a8a35c49 → candidate 2dfb54f640fe. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                  |   4 +
 docs/design/reviews/parallel-jobs-review-2.md | 230 ++++++++++++++++++++++++++
 2 files changed, 234 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (376 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
