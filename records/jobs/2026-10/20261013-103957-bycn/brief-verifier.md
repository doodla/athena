# Job 20261013-103957-bycn: verify "Review the parallel-jobs design and record findings (RM-0)"

## Request (verbatim)

> Review docs/design/parallel-jobs.md independently (RM-0). Evidence from today: hand-run parallel branches collided on CHANGELOG.md and on docs/decisions.md numbering (both D81), and the landing check refuses any hand-merged squash, so a stale candidate can't land without re-verification.

## Reading (accepted by the user)

Write an independent review of docs/design/parallel-jobs.md as docs/design/reviews/parallel-jobs-review-1.md, as build-plan item 1 specifies. It reads the cited code to decide the seven open questions. It also weighs today's evidence: parallel branches collided on CHANGELOG.md and on D81 numbering, and hand-merged squashes are refused. The review then marks the design's status as reviewed.

Assumptions:
- 'Independently' means a fresh reviewer reading the design and the cited source, not the design's author.
- The review file path is the one the design names in section 10 item 1.
- The review decides the open questions but builds nothing; implementation stays out of scope.
- Today's collisions are evidence for the review (shared append-only files, stale candidates), not a request to change the landing code.
- The review also updates the design's status and adds a response section, as section 10 says.

## Acceptance criteria

1. [content] docs/design/reviews/parallel-jobs-review-1.md exists, is written for an outside reader, and gives a verdict (accept, accept with changes, or reject) for the design.
2. [content] The review answers each of the seven open questions in section 11 with a decision. Each cites the file and lines it read, or labels the claim as unchecked.
3. [content] The review checks the design's (read) claims against the cited source: locks in state.rs, run.rs land_or_block, plan in land.rs, and manage.rs and commits.rs lock order. It reports any claim that does not hold.
4. [content] The review addresses today's evidence. Parallel branches that each edit CHANGELOG.md and take the next number in docs/decisions.md (both D81) will always conflict. The landing check refuses a hand-merged squash, so a stale candidate needs re-verification. The review says whether the design handles this: section 7 treats it as stale, and the docs job in section 10 is the shared-file hotspot. It recommends a concrete change or marks the gap as a finding.
5. [content] The review lists findings with severity, and says which can be fixed in the design and which belong to later jobs.
6. [content] docs/design/parallel-jobs.md has its pending-review status and the build-plan item 1 updated to point to the review, and has a short response to the review added. Nothing else in the design changes without a stated reason.
7. [content] No file outside docs/ changes. records/, athena.toml, defaults.toml and CI files are untouched. No secrets or personal data appear.

## Expected deliverables

- docs/design/reviews/parallel-jobs-review-1.md (create)
- docs/design/parallel-jobs.md (modify)
- docs/design/requirements-map.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Implementing the worker pool, --jobs or --all
- Changing land.rs, run.rs or the landing check
- Adding README, CHANGELOG or docs/decisions.md entries
- Changes to athena.toml, defaults.toml, CI or any governance file
- Building native pull-request landing

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each review claim cites the file and lines read, or is labeled unchecked or an estimate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written so an outside reader can follow it without session context.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One review file and a minimal status update to the design.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The review path, scope and open questions are taken from section 10 and 11 of the design.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No emails, secrets or personal data appear in the review.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A verifier checks the review's citations against the source.

## The change

Base a704fa32091b → candidate 8702914a346d. In your working directory, `git diff HEAD~1` shows it.

```text
 docs/design/parallel-jobs.md                  |  25 +++-
 docs/design/requirements-map.md               |   4 +-
 docs/design/reviews/parallel-jobs-review-1.md | 208 ++++++++++++++++++++++++++
 docs/design/roadmap.md                        |   5 +-
 4 files changed, 232 insertions(+), 10 deletions(-)
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
