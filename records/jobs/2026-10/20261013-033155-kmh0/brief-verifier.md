# Job 20261013-033155-kmh0: verify "Study firstmate's work handling as a validated study"

## Request (verbatim)

> Study firstmate's work handling (M1).

## Reading (accepted by the user)

Read as a study job: model how firstmate (pinned from the configured repo, kunchenguid/firstmate) handles work, as cited typed records plus a work-handling model that the athena-cap-study validator accepts. It is the "study firstmate's work handling" first use of the capability named in BRIEF.md; no Athena behavior changes.

Assumptions:
- The study is stored under studies/firstmate-work-handling/ as JSON records; the exact layout follows the athena-cap-study schemas and may differ.
- 'M1' means the first milestone in BRIEF.md (one mechanism, work handling, on firstmate), not a code milestone.
- Scope is firstmate only; Hermes and Athena itself are separate studies.
- I did not read pinned-source/ or the record schemas in crates/athena-cap-study, so the file names are guesses.

## Acceptance criteria

1. [content] A study directory (assumed studies/firstmate-work-handling/) holds typed records: pin (resolved SHA), claims, rationale cards, and a work-handling model, matching the study.*/1 schemas in docs/design/study-capability-m1.md.
2. [content] Every claim about firstmate cites a file and line range (or run) in pinned-source/ with a quote; numbers are measured or labeled as estimates.
3. [content] The model covers how work enters, is assigned to the crew, progresses, is reviewed, and finishes or fails, with guards and each element evidenced.
4. [content] Any divergence between what firstmate's docs say and what it does is recorded as a contradicts-linked finding, and enforcement of each rule is marked as mechanism or prose-only.
5. [behavior] Run cargo test --locked and cargo clippy --all-targets --locked -- -D warnings; both pass, and the study validation (every section 5.4 rule) reports pass with no offending ids.
6. [behavior] Run cargo fmt --all --check; it passes.

## Expected deliverables

- studies/firstmate-work-handling/pin.json (create)
- studies/firstmate-work-handling/claims.json (create)
- studies/firstmate-work-handling/model.json (create)
- studies/firstmate-work-handling/validation.json (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Studying Hermes Agent or Athena itself
- Rendering the explainer page (Pages)
- Changing the study capability crate or schemas
- Changing governance files (athena.toml, CI, lints)
- Running firstmate

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim cites the pinned source; numbers are measured or labeled as estimates.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered 'M1' and the scope from BRIEF.md and docs/design/study-capability-m1.md.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Quotes from firstmate are public repo text only; no secrets or personal data.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier reruns validation and checks citations against the pin.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One study on one mechanism.

## The change

Base aba0eb74fc18 → candidate f6de745c0ac7. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                    |   5 +
 docs/decisions.md                               |  23 ++++
 docs/non-rust-files.md                          |   1 +
 studies/firstmate-work-handling/claims.json     | 148 ++++++++++++++++++++++++
 studies/firstmate-work-handling/findings.json   |  14 +++
 studies/firstmate-work-handling/model.json      | 140 ++++++++++++++++++++++
 studies/firstmate-work-handling/pin.json        |  10 ++
 studies/firstmate-work-handling/rationales.json |  47 ++++++++
 studies/firstmate-work-handling/validation.json | 114 ++++++++++++++++++
 9 files changed, 502 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (302 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
