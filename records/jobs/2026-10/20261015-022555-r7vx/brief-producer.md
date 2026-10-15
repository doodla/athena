# Job 20261015-022555-r7vx: Add an audience role that rates the live explainer page

## Request (verbatim)

> try again on 20261015-014012-akwq

## Reading (accepted by the user)

Add an "audience" role beside the steward. It reads the live firstmate explainer page as a newcomer and records a 1–5 rating with evidence-cited findings. The steward's brief and `athena status` show the latest rating, and a rating of 2 or lower raises a steward work item. This retry leaves athena.toml and policy.rs untouched, which caused the earlier admission failure.

Assumptions:
- Roles such as the steward are defined in code in roles.rs, not in athena.toml (grep of athena.toml found no steward role), so no athena.toml change is needed. This avoids the previous failure, which came from an unknown [audience] section.
- Audience is the steward and owner, who judge whether shipped outcomes work for their readers.
- The live explainer URL is a constant in the new audience module; no config schema extension is made.
- Earlier prevention (a reading-time governance scan) was already tried; this reading instead avoids governance files entirely.

## Acceptance criteria

1. [content] An `audience` role definition exists in crates/athena-data/src/roles.rs with a goal, a fixed prompt and a cadence. The prompt tells it to read the outcome as its audience would, give a 1–5 rating, and cite evidence for every finding. It passes RoleDefinition::validate and its path is records/roles/audience-<n>.json.
2. [content] A versioned, schema-tagged audience-rating record type holds the target (explainer page URL), the audience (newcomer), a rating (1–5 only), findings that each cite a source, and the time. Out-of-range ratings and findings without evidence are refused.
3. [behavior] Run `cargo test`. Tests cover the role definition, record validation and round-trip, and an end-to-end run with the fake agent over a stubbed page, with no network access. It produces a rating record and the steward sees it.
4. [behavior] Run `cargo test`. A test shows the steward's brief or `athena status` output includes the latest audience rating and findings, and that a rating of 2 or lower yields a steward work item. Run `cargo clippy --all-targets` and `cargo fmt --check`; both are clean.
5. [content] The role only reads and records; it changes no code or docs. The explainer URL default is defined in one place in code. No email address or personal data is added anywhere, and crates/athena-agent/src/policy.rs is unchanged.
6. [content] athena.toml is not modified, and still parses with the existing schema. No new top-level section such as [audience] is added. If a change to athena.toml proves necessary, it must use only sections the schema already accepts.
7. [content] docs/design/audience-role.md and a CHANGELOG entry describe the role, how to run it, where its records go, and how to add a target beyond the explainer.

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
- docs/design/audience-role.md (create)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- README.md (modify)

## Out of scope

- Changing the explainer page itself or any shipped outcome
- Editing crates/athena-agent/src/policy.rs or its email allow-list
- Extending the athena.toml schema
- Auditing targets other than the explainer, beyond documenting how to add one

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the role, record, and fake-agent end-to-end run; clippy and fmt clean.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email address or personal data in the record, code, or docs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Drops the earlier README, architecture, and athena.toml changes to keep one focused change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Record validation refuses bad ratings and findings without evidence.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Avoids the governance-file and policy.rs causes of the earlier failure.
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
