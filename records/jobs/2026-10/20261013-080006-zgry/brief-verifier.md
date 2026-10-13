# Job 20261013-080006-zgry: verify "Design a requirements map and roadmap in docs/design"

## Request (verbatim)

> Retry 20261013-074419-sczw.

## Reading (accepted by the user)

Retry of the earlier job. Write docs/design/requirements-map.md: a requirements map built from BRIEF.md and its addenda, with status, owning design and serving jobs per requirement. It also holds the living strategy view (A1) and a roadmap that sequences the queued work. Every citation must be real, and the earlier invented D46 quote and the off-by-one BRIEF.md line must not recur. Design doc only.

Assumptions:
- The previous attempt's file was not kept, so the doc is written afresh. docs/design/requirements-map.md is not in the repository.
- The verifier's two findings are corrections to apply: cite the athena-v0.md status line for O1, and use the correct BRIEF.md line numbers.
- The producer re-reads BRIEF.md, docs/decisions.md and the existing design docs to ground every citation.

## Acceptance criteria

1. [content] docs/design/requirements-map.md exists and has a requirements table with one row per BRIEF.md requirement, addenda included. Each row has a status, an owning design doc and the serving jobs.
2. [content] Every quote and line citation is real. Each quoted string appears verbatim at the cited file and line, and line numbers match (for example, the BRIEF.md permission to use GitHub features and the ban on pushing upstream are on adjacent lines and are not swapped).
3. [content] The O1 row's `built` status cites the status line of docs/design/athena-v0.md. It does not cite D46, which concerns per-job build directories.
4. [content] The doc contains a living strategy view (A1) that says how it is kept current.
5. [content] The doc contains a roadmap that sequences the queued work, with the order justified and the dependencies stated.
6. [content] The doc covers alternatives considered, per the designed default.
7. [content] Only design/doc files change. No code, governance files or records/ change.
8. [behavior] Run `git status` and `git diff --stat`. Only docs, and optionally CHANGELOG.md, are touched, and `cargo fmt --check` still passes.

## Expected deliverables

- docs/design/requirements-map.md (create)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Any code, tests or tooling that generates or validates the map
- Changes to athena.toml, defaults.toml, CI or other governance files
- Editing BRIEF.md or other existing design docs
- Implementing any queued work item

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every status and claim cites a verified file and line, and any number is measured or labeled as an estimate.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design doc lists the alternatives considered.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused doc, with no code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written so an outside reader can follow it.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The original request text and the verifier findings supply the missing detail.

## The change

Base a7556d1792fc → candidate cc775d518200. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                    |   4 +
 README.md                       |   1 +
 docs/design/requirements-map.md | 264 ++++++++++++++++++++++++++++++++++++++++
 3 files changed, 269 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (322 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
