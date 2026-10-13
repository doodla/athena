# Job 20261013-074419-sczw: verify "Design a requirements map, strategy view and roadmap from BRIEF.md"

## Request (verbatim)

> Requests keep landing as one-off jobs. Design a requirements map from BRIEF.md (addenda included) with status, owning design and serving jobs per requirement, plus the living strategy view (A1) and a roadmap that sequences the queued work. Design doc only.

## Reading (accepted by the user)

One new design doc, docs/design/requirements-map.md. It maps every BRIEF.md requirement, addenda included, to a status, an owning design and serving jobs. It also specifies the living strategy view (A1) and a roadmap that orders the queued work. Design only: no code, no schema implementation, and BRIEF.md is untouched.

Assumptions:
- Queued work means the proposed designs under docs/design/ (parallel-jobs, native-pr-landing, squash-chain-landing, study-capability-m1) plus items implied by the addenda. Job ids can only be cited from files visible in the repo; records/ is absent, so serving jobs may be listed by design doc or request.
- The map is a Markdown design doc that specifies a structured data form (fields and generated views) without implementing it.
- The strategy view is specified here as a design only. Its live instance comes later through Athena.

## Acceptance criteria

1. [content] docs/design/requirements-map.md exists and states its status as proposed, with the independent review pending, in the style of the existing docs/design/*.md files.
2. [content] It has a requirements table with one row for every ID in BRIEF.md: O1-O5, A1-A13, S1-S7, D1-D3, and the 2026-10-10 to 2026-10-13 addenda (plugin, taste probes, project tracker, native PR handling, brownfield and stealth, brief as structured contract, tired user and artifact home, dashboard, chunking and parallelism, job-state reconstruction and retros, acceleration). Each row has a status from a defined vocabulary, an owning design doc and the serving jobs.
3. [content] Every owning-design cell names a file that exists under docs/ or says 'none yet'. Serving jobs are referenced by job id or by the queued request, and no id is invented.
4. [content] It specifies the living strategy view (A1) with goals, bets, risks, open questions and next steps, and says how each part is sourced from the map and kept current. It also covers pushback and the recorded exchange. It names the data shape (structured fields) and how it is regenerated.
5. [content] It has a roadmap that sequences the queued work, with each item's dependencies and rationale. The queued work includes the parallel-jobs, native-pr-landing, squash-chain-landing and study-capability-m1 designs and the brief-derived items. The roadmap honors the BRIEF.md order (M1 on firstmate before the study broadens) and the addenda on moving the study up.
6. [content] It lists alternatives considered (for example a hand-written table against a structured file with generated views, and where Jev is used cheaply for tracing) and the tradeoffs. It states what is out of scope, and notes that BRIEF.md is never edited.
7. [content] Claims are marked read or design, and any count or status is cited to a file or labeled as an estimate.
8. [behavior] Run `git status` and `git diff --stat`: only the listed docs paths changed, with no Rust, Cargo, CI or governance file touched and BRIEF.md unmodified. Run `cargo fmt --check` and `cargo test` to confirm nothing else broke.

## Expected deliverables

- docs/design/requirements-map.md (create)
- README.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any Rust code, schema files, or generators for the map or strategy view
- Editing BRIEF.md or records/
- Building the dashboard, project tracker, taste probes or any queued design
- Changes to athena.toml, defaults.toml, CI or lint configuration
- Reviewing or accepting the other designs

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design doc, no code.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Alternatives, tradeoffs and a pending independent review are included.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each status and claim cites a file, or is labeled design or estimate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written so an outside reader can follow it without the brief.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Read the brief, addenda and existing designs to recover the request's scope.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data.

## The change

Base ae2ff7e4cd5e → candidate b6ce01604944. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                    |   4 +
 README.md                       |   1 +
 docs/decisions.md               |  22 +++
 docs/design/requirements-map.md | 353 ++++++++++++++++++++++++++++++++++++++++
 4 files changed, 380 insertions(+)
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
