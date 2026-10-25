# Job 20261013-074419-sczw: Design a requirements map, strategy view and roadmap from BRIEF.md

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

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
