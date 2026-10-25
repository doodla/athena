# Job 20261012-071907-p09e: verify "Revise study capability M1 design to answer review 1"

## Request (verbatim)

> revise the study design to answer review 1; the brief settles its questions, decide the rest yourself

## Reading (accepted by the user)

Revise docs/design/study-capability-m1.md so it answers every finding in study-capability-m1-review-1.md (7 major, 8 minor, 2 nits). The review's eight 'decisions for the user' are settled from BRIEF.md where it speaks and decided by the author where it doesn't. Design only: no code, and the review file is untouched.

Assumptions:
- 'Review 1' means docs/design/reviews/study-capability-m1-review-1.md, and 'the study design' means docs/design/study-capability-m1.md. This is the only design with a review that is still unanswered.
- 'The brief settles its questions' means the review's eight 'Decisions for the user' are resolved from BRIEF.md wherever it speaks, and the rest are my call. Likely outcomes, to be confirmed by the producer: containment is built into the capability (BRIEF.md:313 and v0 §16.2), all BRIEF.md:298 streams are required, Pages publishing becomes a named required design for M1 (BRIEF.md:395), and the evidence rules cannot be waived.
- The response lives inside the design doc (a response section plus revised sections). The review file is a historical record and is not edited.
- This is a revision of the design only. The design's gate requires that majors are answered before any build, so no build is started.
- A re-review is not part of this job. The design's status line says it is awaiting re-review.

## Acceptance criteria

1. [content] docs/design/study-capability-m1.md answers every finding in study-capability-m1-review-1.md (M1–M7, m1–m8, n1–n2) by id, in a response table or section, each marked accepted (with the change made), or declined (with a reason). None is left unanswered.
2. [content] Each of the eight 'Decisions for the user' is settled in the design with a stated basis. Where BRIEF.md settles it, the line is cited: BRIEF.md:298 for all observability streams (M7); 110-111 for the recorded cap, kill switch and downgrade blocker (m6); 299 for redaction (M6); 313 for trust and prompt injection (M5, m7); 306 for the M4 model scope; 395 for Pages being part of M1 (m1). Any decision the brief does not settle is made by the author, with the reason recorded.
3. [content] §5.4 is revised: the six evidence rules are non-waivable (M1); citation rules cover commit, pull-request and run ids, `study.run/1` carries the SHA, and `both` requires a file source and a run source (M2); the four judgment rules are rewritten as structural checks (`contradicts` link, `quote` field, typed Quantity, template-only provenance) or relabeled as verifier duties (M3). Every rule that remains labeled mechanical can be tested with one planted fault and one clean case.
4. [content] §5.1 and §5.3 are extended or bounded for M4: parallel agents, fan-out/join, an evaluated-by field on guards, unhandled failures, what a retry changes, and a timeout for an unanswered person. The synthetic example is corrected so the bound, the counter and the 'requeued' wording agree (m2).
5. [content] The run step (§2, `study.run/1`) names containment and credential handling, a redaction step before output is stored, which meter enforces the per-run cap and what happens when it cannot be observed, the capture streams, and a replay step (M5, M6, M7, m6). The design also covers fresh-context isolation and the capability-gap log (m8), and the prompt-injection limit (m7).
6. [content] Remaining minors and nits are handled: a rationale-card completeness rule (m3), observed-transition recording and scenario independence (m4), A2-C, A4-B and A1-C reworded fairly with a 'typed records now, export later' option (m5), A3 choice moved into the table (n1), and the BRIEF commit named (n2). The §7 reversal costs are updated for any new decision.
7. [content] The status header no longer says the findings are unanswered, and the design remains design-only. No crate, Rust code, or governance file is changed. The review file is untouched. The design makes no claim about firstmate, and its examples stay labeled synthetic.
8. [behavior] Run `cargo fmt --check`, `cargo test` and `cargo clippy` from the repo root. They pass, including the xtask policy tests, which may check doc links and anchors. Run `git diff --stat` and confirm only docs/ files changed.

## Expected deliverables

- docs/design/study-capability-m1.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/reviews/study-capability-m1-review-1.md (modify)

## Out of scope

- Creating the athena-cap-study crate or any Rust code, tests, or Cargo.toml changes
- Writing the Pages publishing design (named as a required follow-up only)
- Reading, running or modeling firstmate, or making any claim about it
- Editing BRIEF.md or the review file
- Writing a second review of the revised design
- Changing governance files such as athena.toml or defaults.toml

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The revision keeps alternatives and reversal costs for every changed decision. The design is returned for independent re-review before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each answered finding and each settled decision cites BRIEF.md lines or review sections. Cost and reversal figures stay labeled as estimates.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The brief and the review's decision list supply the user's answers, so no question is asked.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file is revised. No code is written and no governance file changes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The response is written so an outside reader can follow it without the review's context.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rules stay mechanical with planted-fault tests, or are explicitly relabeled as verifier duties (prose-only).
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Run-output redaction and credential handling are designed in, and nothing secret is written.

## The change

Base 4b02526fa00c → candidate c61f5126f109. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                       |   6 +
 docs/decisions.md                  |  33 +++
 docs/design/study-capability-m1.md | 582 ++++++++++++++++++++++++++++++-------
 3 files changed, 508 insertions(+), 113 deletions(-)
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
