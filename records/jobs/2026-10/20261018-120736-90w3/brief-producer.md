# Job 20261018-120736-90w3: Build the Olympus spec layer in docs/spec.toml with a checking test

## Request (verbatim)

> try again on 20261017-205600-a37m

## Reading (accepted by the user)

Retry: build docs/spec.toml, the cohesive requirements layer from the Project Olympus part of BRIEF.md and the 2026-10-17 addenda, with verbatim cited sources and open questions, plus an xtask test that checks it. The audience is the agents that will read the spec and Sri, who approves it. The reading is unchanged from the accepted one.

Assumptions:
- Retry of rejected job 20261017-205600-a37m with the same scope; the earlier accepted reading stands.
- Audience: agents that will read the spec, and Sri who approves it.
- The earlier rejection reason is unrecorded; likely causes are quote mismatches against BRIEF.md, incomplete coverage, or a policy.rs leak, so the producer should verify each quote and run the whole workspace tests.
- The checker lives in xtask/tests/spec.rs and may need a toml dependency (buy-first) in xtask/Cargo.toml.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test parses docs/spec.toml and passes; in-memory negative cases each fail the checker for (a) a quote not a verbatim substring of BRIEF.md after collapsing whitespace, (b) a duplicate id, (c) a supersedes target that does not exist, (d) a requirement with no source.
2. [behavior] Run `cargo test -p xtask`: a test counts requirements by their sources' headings and finds at least one for each of R1–R8, Phases 1–6 with their acceptance, section 10 (Final system acceptance), and 2026-10-17 decisions 3C, 4A, 5A and 7A.
3. [content] Each requirement in docs/spec.toml has an id, a plain statement, a kind (goal, requirement, constraint, acceptance), a status (active or superseded), and one or more sources (verbatim quote plus heading). Duplicates across sections are merged into one requirement with several sources. Requirement-bearing sentences in Olympus sections 1–10, the governing philosophy and the 2026-10-17 addenda are covered.
4. [content] docs/spec.toml records at least one open question with both verbatim quotes where Sri's words conflict or leave a choice open; it is recorded, not resolved.
5. [content] docs/spec.toml has no statement about who runs Athena or where; `cargo test -p xtask` shows xtask/tests/policy.rs still passes.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.
7. [content] CHANGELOG.md has an entry for the spec; docs/decisions.md records the spec format and the choice of TOML with alternatives (Markdown with ids, JSON, YAML). No earlier Athena brief parts are specified; no reader or prompt is changed.

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
- Owner/operator-side statements about who runs Athena or where

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: xtask test with negative cases; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check quotes, counts and negative cases, not just file existence.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use a maintained toml crate for parsing rather than hand-rolling.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Spec and decision record written for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No owner-side or personal data in the spec.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the Olympus layer; no reader or prompt changes.
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
