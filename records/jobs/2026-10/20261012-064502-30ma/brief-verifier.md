# Job 20261012-064502-30ma: verify "Design the studying capability for firstmate M1 (design doc only)"

## Request (verbatim)

> study firstmate's work handling (M1), first milestone

## Reading (accepted by the user)

Per the latest reply, I read this as design only: write docs/design/study-capability-m1.md for the capability the firstmate M1 (work handling) study needs. No crate and no code. It makes no claims about firstmate itself, and an independent review comes before anything is built.

Assumptions:
- Design only: no crate, no Cargo.toml change, no code or tests.
- The doc follows the style of docs/design/athena-v0.md, including its structure for alternatives and reversal cost.
- Describing the planned crate (athena-cap-study) and its data model in prose is in scope; creating it is not.
- docs/architecture.md stays unchanged until the design is reviewed and built.
- The doc makes no claims about how firstmate behaves; the SHA is pinned at study time, not now.

## Acceptance criteria

1. [content] docs/design/study-capability-m1.md exists and covers: goal, the M1 method (pin SHA, read, run, model, publish), the data model, at least two considered alternatives with tradeoffs, reversal cost, and what is out of scope.
2. [content] The doc states the evidence rules: every claim cites a source (path, line, pinned SHA) and is marked read or observed; rationale cards are marked stated, evidenced or inferred.
3. [content] The doc describes a work-handling model of states, transitions with trigger and guard, failure and retry edges, and concurrency, and the validation rules (uncited claims, unreachable states, dangling targets, inferred rationale shown as stated).
4. [content] The doc makes no claim about firstmate's behaviour; any examples are labeled synthetic.
5. [content] The doc says it is design-only, that nothing is built until an independent review is done, and what that review must check.
6. [content] No files other than the design doc change; no Cargo.toml, crate, governance, or records files are touched.

## Expected deliverables

- docs/design/study-capability-m1.md (create)
- docs/architecture.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Creating the athena-cap-study crate or any Rust code
- Reading, running, or modeling firstmate
- Publishing to Pages
- Studying Hermes or other milestones M2-M9
- Changing governance files or Cargo.toml

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Significant design is written down with alternatives and left for independent review before anything is built.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The doc cites BRIEF.md lines for requirements and labels any numbers as estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The doc is written so an outside reader can follow it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused doc; the build is deferred.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Hidden requirements, such as the evidence rules and the M1 scope, are recovered from BRIEF.md.

## The change

Base f36b35a7cff3 → candidate 0d85b62a14a4. In your working directory, `git diff HEAD~1` shows it.

```text
 docs/design/study-capability-m1.md | 306 +++++++++++++++++++++++++++++++++++++
 1 file changed, 306 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (218 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
