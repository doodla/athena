# Job 20261015-014012-akwq: Add an audience-check role that rates the live explainer page

## Request (verbatim)

> Athena can't see whether what it ships works for the people it's for: the firstmate explainer has been rated 2 of 5 by independent readers three times, and those ratings exist only outside Athena. Add a role that checks a shipped outcome against its audience, starting with the live explainer page as a newcomer would read it, and records the rating and findings so the steward can see them.

## Reading (accepted by the user)

Add a second recorded role, "audience", beside the steward. It reads the live firstmate explainer page as a newcomer and writes a 1–5 rating with findings as a record. The steward's brief and `athena status` then show that record. The audience is the steward and owner, who judge whether shipped outcomes work for their readers.

Assumptions:
- The explainer's live URL comes from the existing Pages deployment (`xtask/src/site.rs`, `.github/workflows/pages.yml`, README line ~134). It is configurable and not hard-coded to an owner name.
- The role runs through the existing agent and role machinery, like the steward. Its fetch of the page goes through the agent's allowed read tools. Tests use the fake agent and a stub page, never the real network.
- A new role needs no change to athena.toml, defaults.toml or workflows. I assumed cadence and target can live in the role definition and code.
- The earlier ratings of 2/5 from outside Athena are not imported. Only new ratings are recorded.
- The role may be triggered by a CLI command and by cadence. Choosing between the two is left to the producer, who keeps the change small.

## Acceptance criteria

1. [content] An `audience` role definition exists in roles.rs, with a goal, a short stable prompt and a cadence. The prompt tells the role to read the outcome as its audience would, give a 1–5 rating, and cite evidence for every finding. It validates under RoleDefinition::validate, and its path is records/roles/audience-<n>.json.
2. [content] A versioned, schema-tagged record type holds the audited target (the explainer page URL), the audience (newcomer), the rating (1–5 only), findings that each cite a source, and the time. Out-of-range ratings and findings without evidence are refused.
3. [behavior] Run `cargo test`. Tests pass for the role definition, the record's validation and round-trip, and an end-to-end run with the fake agent. That run produces a rating record from a stubbed page without network access and shows it to the steward. Run `cargo clippy --all-targets` and `cargo fmt --check`; both are clean.
4. [behavior] Run `cargo test`. A test shows the steward's brief, or `athena status` output, includes the latest audience rating and findings. A rating at or below 2 produces a steward work item, so the owner can see the 2/5 result inside Athena.
5. [content] The role reads and records only; it changes no code or docs. The record contains no secrets or personal data. The default target is the live explainer URL, configured in one place and not hard-coded in several.
6. [content] A docs page and a CHANGELOG entry describe the role, how to run it, where its records go, and how to add a target beyond the explainer.

## Expected deliverables

- crates/athena-data/src/roles.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/audience.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/tests/audience.rs (create)
- crates/athena-context/src/lib.rs (modify)
- docs/records.md (modify)
- docs/architecture.md (modify)
- docs/design/audience-role.md (create)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- README.md (modify)

## Out of scope

- Fixing the explainer page's content or design
- Auditing outcomes other than the explainer, beyond a target field that allows them later
- Importing the three earlier external ratings
- Changes to the scoring rules, ladder or steward's limits
- Changes to deploy workflows or Pages settings

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new role, record and wiring get unit and integration tests. Clippy and fmt stay clean.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust, in the existing crates.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rating range and evidence requirements are enforced by record validation, not by prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by running cargo test, which exercises the role end to end with the fake agent.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The docs page and CHANGELOG entry are written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Records contain only the page URL, ratings and cited findings.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused role with one target; further targets are deferred.
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
