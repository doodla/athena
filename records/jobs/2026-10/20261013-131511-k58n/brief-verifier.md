# Job 20261013-131511-k58n: verify "Write review 3 of the native PR landing design"

## Request (verbatim)

> Re-review docs/design/native-pr-landing.md independently after its revision for review 2. Review only.

## Reading (accepted by the user)

Write an independent third review of the revised docs/design/native-pr-landing.md as docs/design/reviews/native-pr-landing-review-3.md, in review 2's format. It checks whether review 2's findings are resolved and flags new ones; the design is not edited.

Assumptions:
- The new review file is docs/design/reviews/native-pr-landing-review-3.md, per the user's reply.
- Review 3 is independent of review 2 in verdict but checks review 2's findings against the revision.
- No commands are run to reproduce behavior; evidence is by reading.

## Acceptance criteria

1. [content] docs/design/reviews/native-pr-landing-review-3.md exists, mirrors review 2's structure (header, method, verdict, severity scale, evidence table, numbered findings), and states a clear verdict on the revised design.
2. [content] The review states whether each of review 2's findings (F1–F5) is resolved in the revised design, citing file and line numbers read, and labels claims read/design/reported/unchecked/estimate.
3. [content] The review lists any new findings with severity, and states it was written independently.
4. [content] No file other than the new review is changed; docs/design/native-pr-landing.md is untouched.
5. [content] The review contains no secrets or personal data such as email addresses.

## Expected deliverables

- docs/design/reviews/native-pr-landing-review-3.md (create)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing the design document
- Fixing findings
- Changing records/, code, or governance files

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim in the review cites file and lines read or is labeled unchecked/estimate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review is written for an outside reader, like review 2.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file only.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No personal data in the review.

## The change

Base 79b9fa5527b0 → candidate 1cd22486776d. In your working directory, `git diff HEAD~1` shows it.

```text
 docs/design/reviews/native-pr-landing-review-3.md | 193 ++++++++++++++++++++++
 1 file changed, 193 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (327 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
