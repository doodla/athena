# Job 20261015-033631-x3a1: verify "Correct and complete the firstmate explainer page, citing source"

## Request (verbatim)

> try again on 20261015-031159-xx7n

## Reading (accepted by the user)

Retry of the rejected firstmate explainer job: fix what the page (built by `xtask site` from studies/firstmate-work-handling) gets wrong or omits, for newcomers who read only the page. The earlier rejection came from the verifier mutating its snapshot, which is not a repository problem, so this reading repeats the page fix and also applies the earlier minor findings.

Assumptions:
- The audience is newcomers reading only the page; the independent readers who checked the source are the judges.
- 'Try again' means redo the page fix from the base, not the verifier-hardening proposal in the introspection.
- The pinned firstmate source is readable to the producer and needs a fresh read to find facts that c1–c22 do not cover.
- Claims are added only where the pinned source supports them.

## Acceptance criteria

1. [content] Every changed firstmate fact in intro.json, claims.json and model.json is backed by a claim whose quote appears in the pinned firstmate source at the pinned commit. The CHANGELOG entry lists each correction and addition with the firstmate source path and lines.
2. [content] The Start here section and transition cards state the failure and limit behaviour already in the claims: lock contention exits 1 without waiting, an unreadable capacity exits 1, relaunch failure restores the record, no retry or relaunch bound was found, and a missing session start leaves a close unreplayed. Merge authority is worded as the configured setting, and the 'ship' glossary entry is corrected.
3. [content] The glossary entry 'exit 1' says only what claims c4 and c7 support (the lock and unreadable-capacity cases) and does not imply every refusal exits 1. 'captain' is not described with words no claim states.
4. [content] Placeholder values in the model (guard 'none' with evaluator 'agent' on t5, t7, t8) are shown on the page as 'no guard applies', not as an agent-evaluated guard.
5. [behavior] Run `cargo run -p xtask -- site --out <fresh dir inside the working directory>`. It exits 0 and writes index.html. Every cited claim id resolves and no intro element, glossary term or transition summary is uncited.
6. [behavior] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check`. All pass.
7. [content] No governance file (athena.toml, defaults.toml, workflows) changes. If the intro structure changes, docs/design/explainer-site.md describes it.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/rationales.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- xtask/src/site.rs (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing Athena's verifier, its snapshot handling or hashing (not in this repository's deliverables)
- Changes to records/, governance files or other studies

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each page fact cites a claim with a quote from the pinned source.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the study content, touching code only if the page build needs it.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The site build and the workspace tests are run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The earlier minor findings are fixed generally, not only for the instances they named.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Workspace tests, clippy and fmt must pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 94ef4cf64b72 → candidate 77317651bd27. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                  | 25 +++++++++++++++++++++
 docs/design/explainer-site.md                 |  7 +++++-
 studies/firstmate-work-handling/findings.json |  2 +-
 studies/firstmate-work-handling/intro.json    | 31 +++++++++++++++-----------
 studies/firstmate-work-handling/model.json    |  8 +++----
 xtask/src/site.rs                             | 32 ++++++++++++++++++++++-----
 6 files changed, 80 insertions(+), 25 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (630 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
