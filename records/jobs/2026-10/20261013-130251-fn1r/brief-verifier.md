# Job 20261013-130251-fn1r: verify "Revise native-pr-landing design to answer review 2 (F1-F8)"

## Request (verbatim)

> Revise docs/design/native-pr-landing.md to answer review 2's findings (F1-F8). Revision only; the re-review is a later job.

## Reading (accepted by the user)

Edit docs/design/native-pr-landing.md only, so each of review 2's findings F1-F8 is answered in the text. Questions the review leaves to the user are resolved by stated assumptions and listed in §9. No re-review, no roadmap edit, no code.

Assumptions:
- Review 2's four user questions are not answered by the user, so the revision picks the smaller option for each and lists it as an open question: one PR per job; manual re-verify on request; Athena checks merged content against the verified fingerprint; no model Co-Authored-By in stealth.
- Evidence E1 and E2 are reported, not reproduced, and the revision labels them as such.
- The squash-chain design's previous_job gap status is read from squash-chain-landing.md during the revision.

## Acceptance criteria

1. [content] F1: the claim that trailers survive a host merge is removed. §5.1 step 5 and §5.3 state that a landed job is identified from records, never from trailers of the merged commit. The design names a test with a hand-built commit whose message ends in a Co-authored-by paragraph after the Athena trailers.
2. [content] F2: the design states a choice. Assumed: one pull request per job, and a user who squashes several jobs relies on the chain rule (M30) with its limits. It covers the squash naming one job and stealth bookkeeping of one commit to many jobs, and it notes the inherited previous_job gap.
3. [content] F3: the design says whether it supplies re-verification or per-job pull requests for the parallel pool. It states how a job whose base is no longer the target's tip proceeds. It says the roadmap order is to change separately, and the roadmap is not edited.
4. [content] F4: §5.4 'What is given up' records that two host-merged pull requests can produce unverified content and that stealth has no audit. §8 lists a pre-report check that the merged content equals the verified fingerprint.
5. [content] F5: stealth mode writes no Co-Authored-By for the model unless the user asks. The leak check distinguishes Athena's own line from one the host adds on squash.
6. [content] F6: §5.1 step 3 stores the pull request number in both modes, and step 5 looks up the merge by that number.
7. [content] F7: the cited line references (BRIEF.md 459-466, athena-v0-review-1 near line 626, athena-v0 near line 1447, enforcement.md row M23) are re-checked against the files and corrected, or marked unchecked.
8. [content] F8: §9 open questions are extended with the four user questions from review 2 §4, each with the assumption the revision took. The status line and any revision note show the design was revised after review 2.
9. [content] Only docs/design/native-pr-landing.md changes, plus at most a decisions or changelog entry if the repository convention requires one. No file under records/, reviews/, or roadmap.md changes.

## Expected deliverables

- docs/design/native-pr-landing.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing docs/design/roadmap.md
- Writing or editing code or tests
- Re-reviewing the design
- Editing the review file or other reviews
- Building the multi-job pull request design (option b)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Reference corrections cite files and lines, and unverified claims are labeled unchecked or estimate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The revised design is written for an outside reader, with no reliance on review chat.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file is revised, and no code or roadmap change is made.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Alternatives and chosen options stay written in the design.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: F1-F8 are read from docs/design/reviews/native-pr-landing-review-2.md.

## The change

Base 663cc51691d9 → candidate 1bbae3a6aac2. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |   4 +
 docs/decisions.md                |  18 +++++
 docs/design/native-pr-landing.md | 165 ++++++++++++++++++++++++++++++++++-----
 3 files changed, 167 insertions(+), 20 deletions(-)
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
