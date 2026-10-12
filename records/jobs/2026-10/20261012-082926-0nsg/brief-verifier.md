# Job 20261012-082926-0nsg: verify "Answer review 1 in the study-capability M1 design"

## Request (verbatim)

> answer review 1 of the study design

## Reading (accepted by the user)

Revise docs/design/study-capability-m1.md to answer every finding in review 1 (7 major, 8 minor, 2 nit). Add a finding-by-finding response appendix. The eight user decisions are settled in the design, using BRIEF.md where it speaks, and none is left open. The status line says a re-check is required before building.

Assumptions:
- The user's reply means: decide all eight review decisions myself, using BRIEF.md where it settles them, and record them as decided, not open.
- Only the design doc changes; the review file and records/ are untouched.
- Where BRIEF.md does not settle a decision, choose the more conservative option and give the reason.
- I did not read the design or review files; their contents come from the earlier reading.

## Acceptance criteria

1. [content] docs/design/study-capability-m1.md answers every finding in review 1 (M1–M7, m1–m8, n1–n2): each is either changed in the design text or has a recorded reasoned decision not to change.
2. [content] A response appendix, modelled on athena-v0.md Appendix D, maps each finding id to a disposition (accepted, changed in §x, deferred with reason, declined with reason); it is linked from the header and contents.
3. [content] The review's eight user decisions (waivers, containment, run caps, judgment rules, model scope, observability, Pages design timing, alternatives) are each decided in the design with a stated reason, and none is left open or phrased as a question.
4. [content] Evidence rules are non-waivable (M1); §5.4 covers commit, PR and run-id citations and requires a file source and a run source for `both`; `study.run/1` gains a sha field and the pin a tag field (M2).
5. [content] The four judgment rules in §5.4 become structural checks or are relabeled verifier duties (M3); §5.1 states handling of or limits for multi-agent, fan-out, model-decided guards, unhandled failures and retry changes (M4).
6. [content] The design states run-step containment, redaction before output is stored, which meter enforces the per-run cap, and run-record capture streams and the replay step (M5–M7).
7. [content] The synthetic example is consistent on attempts versus requeues and the declared counter, and the retry-edge definition no longer relies on an undefined state order (m2). No claim about firstmate is added.
8. [content] The status line no longer says findings are unanswered and states a re-check of the revised design is required before building under §9; the review file is unchanged.
9. [behavior] Run `cargo test` and `cargo fmt --check` from the repository root; both pass, including xtask policy tests that may check docs links and anchors.

## Expected deliverables

- docs/design/study-capability-m1.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing the review file or anything under records/
- Building any of the study capability code
- Adding claims about firstmate
- Changing governance files such as athena.toml or defaults.toml
- Re-reviewing the revised design

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the target from the repository and the user's reply instead of asking again.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design is revised to answer its review and needs re-check before building.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The response appendix cites finding ids and design sections.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused doc revision; no code changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Docs-checking tests run through cargo test and cargo fmt --check.

## The change

Base ca73bb4041e8 → candidate a0f043423ab9. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                       |   5 +
 docs/decisions.md                  |  28 +++
 docs/design/study-capability-m1.md | 360 +++++++++++++++++++++++++++----------
 3 files changed, 303 insertions(+), 90 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (229 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
