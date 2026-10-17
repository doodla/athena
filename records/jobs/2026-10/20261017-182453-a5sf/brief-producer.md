# Job 20261017-182453-a5sf: Add docs/ontology.toml with sourced terms, validating test, decision

## Request (verbatim)

> try again on 20261017-173508-0wtc

## Reading (accepted by the user)

Retry of job 20261017-173508-0wtc: add a small ontology file with the seven seeded misread terms, each with per-term sources, a Rust structural test, and a decision entry. Hermes now covers both senses (Hermes Agent and the Olympus Orchestration god), and relations match Olympus. No per-occurrence BRIEF.md cross-check, per the user's reply.

Assumptions:
- Audience: agents that read the file, checks that parse it, and Sri judging it.
- The system is Olympus; Athena is its strategy and learning role; Sri is the owner (Zeus) of Olympus.
- Test lives in crates/athena-cli/tests/ontology.rs and parses docs/ontology.toml with the toml crate already available or added; negative cases run on in-memory input.
- The Hermes term was searched for in BRIEF.md by the producer to find its senses (Pantheon table, Part 2 S1), but this search is not recorded or tested.
- docs/decision-links.toml needs an entry for the new decision if the repo requires one.

## Acceptance criteria

1. [content] docs/ontology.toml defines the seven terms (planning, job, Athena, Hermes, product versus system, landed, Sri). Each has a definition, the misread that caused it, a sources field naming paths or decision ids that exist, and relations whose targets are all defined terms.
2. [content] The Hermes entry names both senses: Hermes Agent (second studied system, Part 2 S1, not part of Athena) and Hermes as the Orchestration god in the Olympus Pantheon (BRIEF.md). It says which is part of Olympus and cites a source for each. Every 'not part of' claim has a source.
3. [content] Relations match Olympus: the system is Olympus, Athena is the strategy and learning role within it, Sri owns Olympus (Zeus). No relation makes Athena the whole system.
4. [content] The file states a term is added only when it causes a misread and says how a check reads it (parse path and fields). Landed means merged on main through a verified pull request. The Sri term says use the name or 'they', never he or she, and cites job 20261017-173504-91ys.
5. [content] docs/decisions.md has a new entry keeping the ontology, citing the Olympus 2026-10-17 decision sheet (decision 5), listing alternatives (no ontology; per-study glossaries only), and stating studies keep their own concept model so firstmate, Hermes Agent and Athena are compared in the same terms. docs/decision-links.toml is updated if required, and CHANGELOG.md has an entry.
6. [behavior] Run `cargo test`: the new ontology test passes on docs/ontology.toml, and in-memory unit tests show failure for a missing sources field, a non-existent source, a dangling relation target, and a duplicate term. `cargo clippy` is clean and `cargo fmt --check` passes.
7. [content] No secrets or personal data, including the user's email address, appear in any new or changed file.

## Expected deliverables

- docs/ontology.toml (create)
- crates/athena-cli/tests/ontology.rs (create)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)

## Out of scope

- Recording or testing every occurrence of a term name in BRIEF.md
- Changing studies or their concept models
- Changing athena.toml, defaults.toml or workflows
- Separate docs/ontology.md or xtask test

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New Rust test with negative unit cases; cargo test, clippy, fmt --check pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test run shows the real file passes and bad in-memory inputs fail.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email or personal data in new files.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; dropped per-occurrence cross-check and the optional docs/ontology.md and xtask test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Terms defined for outside readers with sources.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every term and every 'not part of' claim cites a source.
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
