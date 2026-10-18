# Job 20261017-205600-a37m: Build docs/spec.toml, the first layer of the cohesive spec, with a test

## Request (verbatim)

> Build the first layer of the cohesive spec from Sri's words, as decision 3C of the 2026-10-17 decision sheet asks (BRIEF.md, addendum "2026-10-17: Olympus": "The brief has two layers. Sri's words, verbatim and append-only, are the record. One cohesive spec is built from them, and every requirement in it cites the words it came from. Agents read the spec."). Start with the governing layer: the Project Olympus part of BRIEF.md and the addenda dated 2026-10-17.
> Plan: (1) docs/spec.toml holds requirements; each has an id, a statement in plain cohesive wording, a kind (goal, requirement, constraint or acceptance), a status (active or superseded), one or more sources (a verbatim quote from BRIEF.md and the heading it sits under), optional supersedes ids, and open questions where Sri's words leave something undecided. (2) Cover every requirement-bearing sentence in the Olympus part (sections 1 to 10 and the governing philosophy) and in the 2026-10-17 addenda; merge duplicates into one requirement with several sources; record a conflict between two of Sri's statements as an open question, never by choosing. (3) Leave out statements about who runs Athena or where: they are the owner's side, and Athena's files must not name them (xtask/tests/policy.rs). (4) A test checks the file.
> Acceptance:
> - cargo test: a test parses docs/spec.toml and fails on a source quote that is not a verbatim substring of BRIEF.md (after collapsing whitespace), a duplicate id, a supersedes target that does not exist, or a requirement without a source; in-memory negative cases cover each.
> - docs/spec.toml has a requirement for each of Olympus's R1 to R8, its phases 1 to 6 with their acceptance, section 10's final acceptance, and the 2026-10-17 decisions 3C, 4A, 5A and 7A; a test counts these by their sources' headings.
> - At least one open question is recorded where Sri's words leave a choice open, with both quotes.
> - cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean; xtask's boundary test passes; CHANGELOG.md has an entry; docs/decisions.md records the spec's format and the choice of TOML with alternatives.
> Out of scope: the Athena brief's earlier parts (a later layer), the requirements-engineer role, and changing any reader or prompt to use the spec.

## Reading (accepted by the user)

Create docs/spec.toml: cohesive requirements from the Project Olympus part of BRIEF.md (sections 1–10, governing philosophy) and the 2026-10-17 addenda. Each requirement cites verbatim quotes with headings. A new xtask test checks the file. The audience is the agents that will read the spec and Sri, who approves it.

Assumptions:
- The checker test lives in xtask/tests/spec.rs, since xtask already has toml, serde and regex as dev-dependencies; no new crate is needed.
- Open questions are an array of tables inside or beside each requirement, each with a text and two or more quotes that are also checked as verbatim substrings.
- Sources are an array of {quote, heading} tables; the heading is checked against BRIEF.md headings, and the test counts R1–R8, phases, section 10 and the decisions by these headings.
- Owner-side items (2A, the operator-as-brain relay note, 6A) are left out; 1A is kept only as far as it says the Olympus brief governs and wins on conflict.
- Superseded status is allowed by the format but probably unused in this first layer.
- The user's wording that quotes sit under a heading means the nearest ### or ## heading, and for relay-note bullets the addendum heading.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test parses docs/spec.toml and passes. In-memory negative cases each fail the checker for (a) a quote not a verbatim substring of BRIEF.md after collapsing whitespace, (b) a duplicate id, (c) a supersedes target that does not exist, (d) a requirement with no source.
2. [behavior] Run `cargo test -p xtask`: a test counts requirements by their sources' headings and finds at least one for each of R1–R8, Phases 1–6 with their acceptance, section 10 (Final system acceptance), and the 2026-10-17 decisions 3C, 4A, 5A and 7A.
3. [content] Each requirement in docs/spec.toml has an id, a plain cohesive statement, a kind (goal, requirement, constraint or acceptance), a status (active or superseded), and one or more sources of a verbatim quote plus its heading. Duplicate statements across sections are merged into one requirement with several sources. Every requirement-bearing sentence in Olympus sections 1–10, the governing philosophy and the 2026-10-17 addenda is covered.
4. [content] docs/spec.toml has at least one open question where Sri's words leave a choice open or two statements conflict, carrying both verbatim quotes. The conflict is recorded, not resolved. Examples: the 'API surface not settled' remark against the settled-design review loop, or the Olympus-wins rule against the retained Athena brief.
5. [content] docs/spec.toml has no statement about who runs Athena or where (owner/operator-side items such as 2A and the operator-as-brain note). Run `cargo test -p xtask` to confirm xtask/tests/policy.rs still passes.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean. Run `cargo test` for the workspace: it passes.
7. [content] CHANGELOG.md has an entry for the spec. docs/decisions.md records the spec's format and the choice of TOML, with the alternatives considered (for example Markdown with ids, JSON, YAML). The Athena brief's earlier parts are not specified, and no reader or prompt is changed to use the spec.

## Expected deliverables

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/ontology.toml (modify)

## Out of scope

- The Athena brief's earlier parts (a later layer)
- The requirements-engineer role
- Changing any reader or prompt to use the spec
- Editing BRIEF.md (append-only)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The checker is a Rust test in xtask, using toml and serde.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Positive and in-memory negative tests; clippy and fmt required clean.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Spec statements and decisions.md entry are written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No owner/operator-side statements and no personal data in spec.toml.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the test rejects bad quotes, duplicates, dangling supersedes and sourceless requirements, not that the file exists.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test, clippy and fmt --check.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Existing toml and serde crates are used; no new parser is built.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the Olympus layer only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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
