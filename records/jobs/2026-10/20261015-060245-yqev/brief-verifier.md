# Job 20261015-060245-yqev: verify "Retry: correct and complete the firstmate explainer page"

## Request (verbatim)

> try again on 20261015-033631-x3a1

## Reading (accepted by the user)

Retry of rejected job 20261015-033631-x3a1: fix what the firstmate explainer page (built by `xtask site` from studies/firstmate-work-handling) gets wrong or omits, for newcomers who read only the page, and also fix the two earlier nits. The earlier rejection was a verifier snapshot mutation, not a repository fault. Changing the verifier is not part of this request.

Assumptions:
- The typed request, 'try again', takes precedence over the introspection's proposed verifier-hardening request, so verification infrastructure is not changed here.
- The audience is newcomers who read only the page; the goal is a rating of 4 or more out of 5 and no wrong or missing firstmate facts.
- The earlier reading and its criteria remain valid, as the user accepted them.
- Fixes are limited to facts already backed by claims, or new claims whose quotes appear in the pinned source.
- The page text 'only by a session start' must be reworded to what c14 and c15 say, or backed by a claim that says it directly.
- The CHANGELOG must say the source-quote test runs only when pinned-source/ is mounted, not that it always checks the pinned commit.

## Acceptance criteria

1. [content] Every changed firstmate fact in intro.json, claims.json and model.json is backed by a claim whose quote appears in the pinned firstmate source. The CHANGELOG entry lists each correction and addition with the firstmate source path and lines.
2. [content] The Start here section and transition cards state the failure and limit behaviour found in the claims. Lock contention exits 1 without waiting. An unreadable capacity exits 1. A relaunch failure restores the record. No retry or relaunch bound was found. A missing session start leaves a close unreplayed. Merge authority is worded as the configured setting, and the 'ship' glossary entry is corrected.
3. [content] The glossary entry 'exit 1' says only what claims c4 and c7 support (the lock and unreadable-capacity cases). 'captain' is not described with words no claim states. Placeholder values (guard 'none' with evaluator 'agent' on t5, t7, t8) appear on the page as 'no guard applies'.
4. [content] The wording 'only by a session start' in intro.json (what[6] and summary t5) matches what c14 and c15 state, or a claim says it directly. The CHANGELOG states that the source-quote test runs only when pinned-source/ is mounted.
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

- Changing how the verifier is run (read-only snapshot, fingerprinting, rerun on mutation)
- Changes to athena.toml, defaults.toml or workflows
- Changing the pinned firstmate commit
- Studying parts of firstmate beyond work handling

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page and CHANGELOG are written for newcomers who read only the page.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each corrected fact cites a claim whose quote is in the pinned source, and the CHANGELOG gives source paths and lines.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The site build is run to check that every citation resolves.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The wording is fixed in the claims and model, not only the page text.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The goal and criteria were recovered from the earlier job instead of asking.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused docs change in the firstmate study; no infrastructure work.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 132ebf594560 → candidate 08379ad6f3e2. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                  | 24 ++++++++++++
 docs/design/explainer-site.md                 |  5 +++
 studies/firstmate-work-handling/findings.json |  2 +-
 studies/firstmate-work-handling/intro.json    | 27 ++++++++-----
 studies/firstmate-work-handling/model.json    | 10 ++---
 xtask/src/site.rs                             | 56 ++++++++++++++++++++++-----
 6 files changed, 100 insertions(+), 24 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (641 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
