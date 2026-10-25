# Job 20261012-073105-mj6d: verify "Revise study-capability M1 design to answer review 1 (retry)"

## Request (verbatim)

> try again on 20261012-071907-p09e

## Reading (accepted by the user)

Redo the revision of the study-capability M1 design doc to answer review 1. Fix the earlier nits (a stable BRIEF.md reference, the §10.2 redaction citation, the deferred-order note). Write behavior criteria the verifier can check with git diff and git status, which it runs itself.

Assumptions:
- The earlier rejection was caused by the unverifiable evidence citation (criterion 8), not by the design content; the content is largely right.
- The brief's 'modify' listing for the review file contradicts its own out-of-scope list, so the review file stays untouched.
- The design doc is not yet revised at this commit, since the earlier job was rejected, so the full revision is redone.
- Templates are deferred to the Pages design, so the 'Inferred shown as stated' test is marked as dependent on it.

## Acceptance criteria

1. [content] docs/design/study-capability-m1.md answers every finding in docs/design/reviews/study-capability-m1-review-1.md, with a response table naming each finding and its resolution.
2. [content] Each BRIEF.md reference names the BRIEF.md commit or a stated snapshot, so a reader can find it without job records (fixes the earlier n2 nit).
3. [content] The §10.2 response table cites BRIEF.md:299 directly on the redaction row.
4. [content] §5.4 and §8.1 state that the 'Inferred shown as stated' rule's planted-fault test is deferred until the Pages template design exists, and the re-review is asked to confirm that order.
5. [content] BRIEF.md and the review file are unchanged; the diff touches only docs/design/study-capability-m1.md, and possibly CHANGELOG.md.
6. [behavior] Run `git diff --stat` and `git status`: only the design doc, and optionally CHANGELOG.md, appear as changed. The verifier runs these commands itself, so the evidence cites commands it saw run.
7. [behavior] Run `git diff` and `git status` and confirm that no governance file, BRIEF.md or review file was modified.

## Expected deliverables

- docs/design/study-capability-m1.md (modify)
- CHANGELOG.md (modify)
- docs/design/reviews/study-capability-m1-review-1.md (modify)

## Out of scope

- Editing BRIEF.md
- Editing the review file
- Building the Pages templates or the study code
- Governance files

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every design claim cites BRIEF.md lines or review findings; the response table shows its sources.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the design doc changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria use git diff and git status, which the verifier can run itself.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the earlier job's request and findings from the brief.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: References do not depend on private job records.

## The change

Base 9102a2bb2e90 → candidate 33c09ee76c58. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                       |   4 +
 docs/design/study-capability-m1.md | 293 +++++++++++++++++++++++++++----------
 2 files changed, 221 insertions(+), 76 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (221 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
