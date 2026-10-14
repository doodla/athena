# Job 20261014-032524-cbxt: Add introspection record, class registry, and status review (step 1)

## Request (verbatim)

> Build step 1 of the build order in docs/design/defaults-loop.md section 12: the introspection record, the class registry, the records-reading code (section 3.3), and a person-visible review of the output, using a language model for the prose and no Jev. Introspection is triggered by the event itself (section 3.1): every rejected or failed job, every correction and every intervention. Show the record in 'athena status <job>' so the user sees it without opening files. Reuse what 'athena backtest' already has (the introspection schema, the records reader, citation validation, the evidence-after-decision rule) instead of a second copy. Verify with fixture records in a temporary directory; agents' copies leave out records/, so the run over real records comes after landing.

## Reading (accepted by the user)

Implements build-order step 1 of defaults-loop.md §12: an event-triggered athena.introspection/1 record (rejected or failed job, correction, intervention), a class registry record store, and code that reads records/ per §3.3 and passes a filtered summary to a language model for the prose. The record shows in `athena status <job>`. It reuses backtest's schema, records reader, citation validation and evidence-after-decision rule by moving them to shared code, with no Jev. Tests use fixture records in a temp directory.

Assumptions:
- Without Jev, failure_class and prevention_kind are chosen by code or the language model from the registry, with `new` plus a proposed name when nothing fits; the choice is marked as not from Jev.
- The language model call goes through the existing athena-agent abstraction, with a fake agent in tests.
- A failed introspection is recorded as failed, never blocks the job, and is not itself introspected.
- Host-error and landing triggers (§3.1 rows 5 and 6) and retry-from-record (§3.5) are step 2 or later and are left out.
- Rejected proposals and plans have no record yet, so they are not triggers here.
- The 'person-visible review' is the introspection block in `athena status <job>` plus its --json form; no separate review command.
- Shared reader and introspection types move from athena-cli backtest.rs into a lower crate (athena-data or athena-core) that both backtest and the new code use; backtest output is unchanged.

## Acceptance criteria

1. [content] The athena.introspection/1 schema, the records reader and the citation validation exist once, in shared code used by both `athena backtest` and the new introspection path; backtest.rs keeps no duplicate definition.
2. [content] A class registry record store exists. New classes are added as `proposed` with a code-assigned id, and the repository files are never edited.
3. [content] The §3.3 reading code reads only the listed stored fields (never transcripts, diffs or other jobs' files). It narrows by class_of_work and category, caps earlier cases at 5, and passes only those fields to the model.
4. [content] On return, any earlier_cases or decisions_matched id not on the list passed in is dropped and logged. A cause equal to what_happened after normalising is flagged and the record rated failed.
5. [content] Introspection is triggered automatically for a rejected job, a failed job, a correction and any other intervention. It is not triggered for a failed introspection.
6. [content] Evidence is checked to be after the decision it is cited against, using the existing backtest rule.
7. [content] `athena status <job>` prints the introspection (what happened, class, earlier cases, cause, prevention, proposed request, status) and includes it in --json.
8. [behavior] Run `cargo test`: fixture-record tests in a temp directory pass. They cover each trigger, class matching and new-class creation, id dropping, cause-restates-event flagging, the failed-introspection path, and the status output. The existing backtest tests still pass unchanged.
9. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean.
10. [content] A docs note, and a CHANGELOG entry, say that step 1 is built and that the run over real records follows landing.

## Expected deliverables

- crates/athena-core/src/introspect.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/tests/introspection.rs (create)
- crates/athena-core/src/fake.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-data/src/paths.rs (modify)

## Out of scope

- Jev client or any call to POST /v1/evaluate
- Retry-from-record (step 2)
- Proposal records, `athena status` surfacing of proposals, apply-as-a-job (step 3)
- Host-error and landing triggers, and a record for rejected proposals or plans
- Running over the real records/ directory
- Changes to defaults.toml, athena.toml, CI or lint configuration

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse backtest's reader, schema and validation, and existing crates (serde, jiff); build nothing a crate already covers.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture tests in a temp directory; cargo test passes and clippy is clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Status output is exercised by a test running the CLI path.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the CHANGELOG are written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixtures use synthetic data, with no personal data.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Step 1 only; steps 2 to 6 are excluded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The shared code is extracted rather than copied, so there is one reader and one schema.
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
