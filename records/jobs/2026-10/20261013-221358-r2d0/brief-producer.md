# Job 20261013-221358-r2d0: Reject self-contradicting readings; make the changelog entry a default

## Request (verbatim)

> Fix the root cause of four failures and corrections today (jobs 8qkc, m1bs and two hand corrections): a reading's acceptance criteria say 'only these files change' while its deliverables list names other files, or the criteria exclude CHANGELOG.md although this repository adds a CHANGELOG.md entry to every change (BRIEF.md asks for a changelog). The producer follows one, the verifier enforces the other, and the job fails. Make a reading self-consistent before it is shown: a file-scope criterion must agree with the deliverables list, and a contradiction is caught in code when the reading is parsed or admitted, not left to the verifier. Make the changelog entry a default the reader includes rather than something each request must remember.

## Reading (accepted by the user)

Add a code check, run when a reading is parsed (and again at admission), that a file-scope criterion ("only these files change") agrees with the deliverables list. Add a `changelog` default to defaults.toml and the reader prompt so CHANGELOG.md is listed as a deliverable and never excluded for code, docs, config and build changes.

Assumptions:
- The check is a heuristic over criterion text: a criterion that says only/exclusively/no other files change, and names paths, must name every deliverable path, and must not name a file that is outside the deliverables list as allowed. It is not a full natural-language parse.
- CHANGELOG.md is a deliverable for every reading whose class is code, tests, docs, config, build or prompts. Design and other readings are exempt, as are readings with no capability.
- A contradiction in build_reading is returned as an error with a message naming the file, so the reader is re-prompted or fails before the user sees the reading. Admission re-checks the accepted reading as a backstop.
- Adding a default changes defaults.toml, which is governance. It is therefore a listed deliverable and is named in a criterion. The defaults version is bumped if the repository's convention requires it for added entries.

## Acceptance criteria

1. [content] A function in crates/athena-core (reading consistency check) returns an error naming the file when a criterion that restricts the change to named files omits a deliverable path, or when a deliverables list contains CHANGELOG.md while a criterion excludes it.
2. [content] build_reading in crates/athena-core/src/reading.rs calls the check after the deliverables are normalized, so a contradictory reading is rejected before it is recorded or shown; admission.rs also calls it on the accepted reading.
3. [content] defaults.toml gains a `changelog` default, applying to code, tests, docs, config, build and prompts, which says every change adds a CHANGELOG.md entry; defaults.toml is named in this criterion as an intended change.
4. [content] crates/athena-context/prompts/reader.md tells the reader to list CHANGELOG.md as a deliverable and to keep file-scope criteria consistent with the deliverables list.
5. [content] CHANGELOG.md gains an entry for this change.
6. [behavior] Run `cargo test` and confirm the new unit tests pass: consistent scope accepted, scope omitting a deliverable rejected, scope excluding CHANGELOG.md while it is listed rejected, and a reading with no scope criterion accepted.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`; both are clean.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-context/prompts/reader.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the verifier's behavior or prompts.
- Rewriting past jobs' records or the four failed jobs.
- Parsing arbitrary natural-language criteria beyond the file-scope pattern.
- Changing athena.toml or CI workflows.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a code check at the point readings are built, plus a default, rather than another prompt reminder.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Consistency is enforced in code at parse and admission, not left to the verifier.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests cover the consistent and contradictory cases; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: one consistency check, one default.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check is Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The check's error messages and the default text are plain and self-explanatory.

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
