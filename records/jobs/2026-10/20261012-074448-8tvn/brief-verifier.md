# Job 20261012-074448-8tvn: verify "Revise M1 study design doc: fix BRIEF pin and verification evidence"

## Request (verbatim)

> try again on 20261012-073105-mj6d

## Reading (accepted by the user)

Retry of the rejected job: the design doc docs/design/study-capability-m1.md must pin BRIEF.md line references without citing a commit that exists only in Athena's rebuilt repo. Behavior criteria must be checkable with the allowed commands, and the final report must cite only commands that actually ran.

Assumptions:
- The job id in the request has no readable record here, so the earlier request (an M1 study-capability design doc) and its findings define the scope.
- The current doc already says line numbers refer to BRIEF.md 'as of this change'; I assume the earlier faults (unreal commit hash, nonexistent §8.1 reference) may still need fixing or re-verifying.
- Real-repo-independent pinning is acceptable, e.g. the BRIEF.md content description or a statement relative to the change, with no hash from a rebuilt history.

## Acceptance criteria

1. [content] docs/design/study-capability-m1.md contains no commit hash that is absent from the real repository; the BRIEF.md pin names no synthetic base commit (grep for c6004fd returns nothing).
2. [content] Every cross-reference in the doc resolves to an existing heading; no reference to a nonexistent subsection such as §8.1 remains.
3. [content] BRIEF.md line citations in the doc match the BRIEF.md in the working tree, and the doc states how a reader can confirm the version they cite.
4. [behavior] Run `git status` and `git diff --stat`: only the design doc (and optionally CHANGELOG.md) changed, and BRIEF.md, records/ and governance files are untouched.
5. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass, since no Rust code changed.

## Expected deliverables

- docs/design/study-capability-m1.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the study capability crate
- Editing BRIEF.md
- Changing governance files or CI
- Changes to the design's substance beyond fixing pins and references

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The pin must work for a reader of the real repository, not only Athena's rebuilt history.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Line citations must be verified against the current BRIEF.md.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria use only commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One doc fix only.

## The change

Base 81ec0c5d5cde → candidate b210511de99a. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                       | 4 ++++
 docs/design/study-capability-m1.md | 7 ++++++-
 2 files changed, 10 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (222 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
