# Job 20261017-173508-0wtc: Add a seeded ontology file with a structure test and decision record

## Request (verbatim)

> From Sri's decision sheet of 2026-10-17 (relay note in BRIEF.md, addendum "2026-10-17: Olympus"), decision 5, worded as on the sheet: Should Athena keep an ontology? A small shared vocabulary, grown from misreads. Defined terms and how they relate, each with its source, in one file that agents read and checks can use. A term is added when it causes a misread. The study keeps its own concept model, so firstmate, Hermes Agent and Athena are compared in the same terms.

## Reading (accepted by the user)

Answer yes to decision 5: add docs/ontology.toml, a small vocabulary of defined terms with sources and relations, seeded with the seven misreads Sri listed, plus a Rust test that validates it and a decision entry. Audience: agents that read it, checks that parse it, and Sri judging it.

Assumptions:
- The file is machine-readable TOML (docs/ontology.toml) so checks can parse it; the repo already uses docs/decision-links.toml.
- The new decision gets the next free id after D108 (D109); the producer checks decisions.md for the current highest id and for any link-table requirement in docs/decision-links.toml.
- The test lives in an existing crate's tests dir (athena-cli/tests) and uses crates already in the workspace (toml/serde) rather than adding a new dependency.
- Term sources are repo paths or decision ids that exist; BRIEF.md addenda are cited by path plus addendum heading and the test checks the file and heading text exist.
- Seven terms: planning, job, Athena, Hermes, product versus system, landed, Sri. Each records the misread, definition, source and relations; relation targets must be defined terms, so supporting terms (e.g. Olympus, firstmate, Hermes Agent, study) are added only if needed as relation targets and each carries a misread or is avoided.
- Sri's pronoun rule (refer by name or as they) is a term definition; the file itself uses they/Sri only.

## Acceptance criteria

1. [content] docs/ontology.toml exists with the seven seeded terms (planning, job, Athena, Hermes, product versus system, landed, Sri). Each has a definition, the misread that caused it, a source naming a path or decision id that exists in the repo (planning: BRIEF.md addendum 2026-10-16 'Speed, planning first' and D105; job and Athena: addendum 2026-10-17: Olympus; Hermes: Part 2 S1; product versus system: the Olympus relay note), and relations whose targets are all defined terms.
2. [content] The file states that a term is added only when it causes a misread, and says how a check reads it (the parse path and fields). The landed term says merged on main through a verified pull request. The Sri term says to use the name or 'they', never he or she, and cites job 20261017-173504-91ys.
3. [content] docs/decisions.md has a new entry keeping the ontology, citing the Olympus 2026-10-17 decision sheet (decision 5), listing alternatives (no ontology; per-study glossaries only), and stating that studies keep their own concept model so firstmate, Hermes Agent and Athena are compared in the same terms. Any decision-link or index file the repo requires for new decisions is updated.
4. [behavior] Run `cargo test`: a new test parses docs/ontology.toml and passes, and the test is written to fail on a missing or non-existent source, a dangling relation target, or a duplicate term (negative cases covered by unit tests on in-memory input). `cargo clippy` is clean and `cargo fmt --check` passes.
5. [content] No secrets or personal data, including the user's email address, appear in any new or changed file.

## Expected deliverables

- docs/ontology.toml (create)
- crates/athena-cli/tests/ontology.rs (create)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)
- crates/athena-cli/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/ontology.md (create)
- xtask/tests/ontology.rs (create)

## Out of scope

- Changing any study's own concept model or files under studies/
- Wiring the ontology into agent contexts or token budgets beyond the file and its test
- Adding terms not caused by a misread
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The structure test is Rust; no scripts.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test, cargo test, clippy and fmt --check required.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Terms defined for an outside reader with cited sources.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every term carries a verifiable source.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or email address in new files.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Decision entry records alternatives.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Test enforces sources, relations and uniqueness.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test must fail on bad input, not just check the file exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file, one test, one decision entry.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
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
