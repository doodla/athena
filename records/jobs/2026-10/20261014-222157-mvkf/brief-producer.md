# Job 20261014-222157-mvkf: Check that athena.toml and defaults.toml parse during admission

## Request (verbatim)

> Run the gate's 'configuration parses' check (athena.toml and defaults.toml against the config schema) as a check during verification, so a candidate with an unknown config field fails with a repairable finding before the landing commit is made. Add a test with an unknown field.

## Reading (accepted by the user)

Today the gate's "configuration parses" check (content.rs) only runs at landing. The reading adds the same check to candidate admission in athena-core, where secrets are already scanned. A candidate whose athena.toml or defaults.toml has an unknown field then fails with a repairable finding before the landing commit is built. A test covers an unknown field. The audience is the Athena operator and the producer agent that reads the finding.

Assumptions:
- "During verification" means candidate admission in crates/athena-core/src/admission.rs, which run.rs calls before fingerprinting and the verifier checks. Admission already scans for secrets there "so a secret fails the job here instead of at landing". I did not take it to mean a configured [capabilities.change] check command, because that would mean editing athena.toml, a governance file.
- The gate's own check in athena-gate/src/content.rs stays as the landing-time backstop. Admission reuses the same parsers rather than a new schema.
- The finding goes through PolicyViolation (rule, path, detail), which has no separate repair field. The repair text goes into the detail, for example "fix the file so it parses".
- Only a changed athena.toml or defaults.toml is parsed, using the candidate's blob. Whether the check also repeats the exempt-commit rule is left to the producer. Parsing alone meets the request.
- The test uses Git::isolated with a temporary repository, as other athena-core tests do.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/admission.rs builds a base commit and a candidate commit whose athena.toml contains an unknown field. admit() must return a violation with rule "configuration", the path athena.toml, and a detail that names the parse error and says how to repair it. Run `cargo test -p athena-core admission`.
2. [behavior] A second test covers an unknown field in defaults.toml: admit() returns a configuration violation for the path defaults.toml. A third test shows that a candidate whose athena.toml and defaults.toml both parse, and a candidate that touches neither file, add no configuration violation. Run `cargo test -p athena-core admission`.
3. [content] admit() parses the candidate's athena.toml and defaults.toml only when they changed, using Config::parse and DefaultsModel::parse from athena-data (the same parsers the gate uses). It does not duplicate the schema. A deleted file is not parsed. The violation reaches the existing candidate.violations path in run.rs, so the job is rejected with the finding before landing.
4. [content] The module doc comment in admission.rs lists the new rule. A parse failure is reported as an admission violation (the finding), not as an anyhow error that aborts the job.
5. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass, including the existing athena-gate content tests for the configuration rule.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/content.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the gate's landing-time checks or the Rule enum names
- Editing athena.toml or defaults.toml, including adding a check entry to [capabilities.change]
- Changing the config schema or adding fields
- The exempt-commit history validation in content.rs::new_exemptions
- Validating other files, such as roles.toml or Cargo.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and its tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new tests cover an unknown field in each of the two files; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in admission.rs, with at most a changelog line.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The candidate is checked at admission using the real parsers, rather than catching one unknown-field case by hand.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: It reuses Config::parse and DefaultsModel::parse, so no new parser or schema is built.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands the verifier can run.
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
