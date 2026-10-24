# Job 20261024-033758-c2h3: Add six records-history record types with validation to athena-data

## Request (verbatim)

> Build job 1 of the records-history plan (docs/design/records-history.md section 10, decision D111): the schema and the writer. Add the new record types (landing-intent, landing, intervention, checkpoint, legacy-exempt, import-manifest) to athena-data with their schemas and validation, and a writer that commits records to the branch athena/records (its own root, never merged into main) with compare-and-swap and bounded retry as section 8.2 describes. Tests: two simultaneous writers both land; a rejected update retries and then succeeds; a writer never rewrites or deletes an existing record path. Also add a read-only check that reports whether the host forbids force-pushes and deletion on athena/records and requires branches to be up to date before merging on main; it reports, it changes no host setting. Nothing reads or writes the new branch in the job pipeline yet; records/ on main and the history check are unchanged.

## Reading (accepted by the user)

Narrowed job 1 of records-history (§10, D111) to the schema only: add landing-intent, landing, intervention, checkpoint, legacy-exempt and import-manifest record types to athena-data, with validation, round-trip and rejection tests, doc comments citing the design, and a CHANGELOG entry. The writer and host-rules check are excluded. Audience: the maintainer and the verifier running cargo test.

Assumptions:
- Existing records.rs conventions (serde deny_unknown_fields, schema tag, pretty JSON) are followed; new types may live in records.rs or a new module.
- The new 'intervention' record is distinct from the existing InterventionCategory enum; naming must avoid clashing with it.
- The writer and host-rules check are later jobs and out of scope per the user's reply.
- No new crate dependency is needed; if one is added it is justified under buy-first.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-data: each of the six new record types (landing-intent, landing, intervention, checkpoint, legacy-exempt, import-manifest) serializes to JSON with a schema tag and deserializes back to an equal value.
2. [behavior] Run cargo test -p athena-data: validation rejects malformed examples for the new types: an empty required field, an unknown field, a malformed sha (not 40 lowercase hex), and a wrong schema tag. Valid examples pass.
3. [behavior] Run cargo test --workspace, cargo clippy --workspace --all-targets and cargo fmt --check: all pass. Existing records and history-check tests are unchanged and still pass.
4. [content] Public new items carry doc comments citing docs/design/records-history.md (§5.3, §4 or §8 as relevant) and D111. Fields follow the design: landing-intent has PR number, head sha, head tree sha, expected base, job ids; landing has PR number, head sha, host commit, merge method, ordered commits (sha, tree) with job and verification, and the intent it fulfills; intervention has declarer, reason and commit sha; import-manifest has C0 and per-path blob id and SHA-256; legacy-exempt has entries with sha and reason.
5. [content] No writer, no host-rules check, and no git-branch code is added. No code in athena-core, athena-cli or athena-gate uses the new types. records/ on main and the history check have no diff.
6. [content] CHANGELOG.md has an entry naming the six new record types in athena-data.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/history.rs (create)
- crates/athena-data/src/lib.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/Cargo.toml (modify)

## Out of scope

- The compare-and-swap writer and bounded retry for athena/records
- The read-only host branch-rules check
- Any change to records/ on main or the history check
- Wiring the types into athena-core, athena-cli or athena-gate

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Round-trip and rejection tests; cargo test and clippy pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Schema only, per the user's narrowing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise round-trip and rejection.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite the design for outside readers.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
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
