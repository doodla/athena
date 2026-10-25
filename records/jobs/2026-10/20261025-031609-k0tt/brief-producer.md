# Job 20261025-031609-k0tt: Add shared records-history cutover fixture for core and cli tests

## Request (verbatim)

> Priority: high
> Add a reusable test fixture for the records-history cutover (decision D111): a helper, shared by athena-core and athena-cli tests, that builds a scratch repository with a bare remote and a realistic history made through Athena's real writers: a bootstrap commit, one job read, dispatched, verified and landed the old way (with trailers, through the real landing path), and then the cutover job named by [history] cutover_job, landed with its landing record, import commit and legacy-exempt record on athena/records. Job 20261025-022144-emm8 and its two earlier attempts were rejected because their hand-built fixtures did not match real history.
> Acceptance:
> - athena check --history and athena check --attribution both pass on the fixture's main with no violations.
> - A test uses the helper and asserts the fixture's shape: the old-way landing carries its trailers, C0 resolves from the cutover job's landing record, and the import manifest's self-check passes.
> - The helper is documented for reuse by the trailer job and job 7 part 2.
> Files: crates/athena-core/src/test_support.rs or tests/common, crates/athena-cli/tests/common/mod.rs, CHANGELOG.md

## Reading (accepted by the user)

Build a reusable test helper that creates a scratch repo with a bare remote and real-writer history: bootstrap, one old-way landed job with trailers, then the cutover job with landing record, import commit and legacy-exempt record on athena/records. Add a shape-asserting test, document reuse for the trailer job and job 7 part 2, and add a CHANGELOG entry. Audience: maintainers and verifiers of later history jobs.

Assumptions:
- The helper may live in athena-core behind a test-support feature or cfg(test) plus a shared path, with athena-cli tests/common/mod.rs reusing it; the exact sharing mechanism is the producer's choice.
- History is built only via existing real writers (land, records import, legacy-exempt writer); if they are not callable, the producer reports rather than hand-building commits.
- I did not read the writers' code; deliverable paths are estimates.

## Acceptance criteria

1. [behavior] A test using the helper runs `athena check --history` and `athena check --attribution` on the fixture's main and both report no violations; run `cargo test` and see it pass.
2. [behavior] A test asserts the old-way landing commit carries its Athena trailers, produced through the real landing path (not hand-built commit text), and that the job was read, dispatched, verified and landed.
3. [behavior] A test asserts C0 resolves from the cutover job's landing record on athena/records, and that the import commit and legacy-exempt record exist there; the import manifest self-check passes.
4. [behavior] The fixture has a bare remote, a bootstrap commit, and [history] cutover_job names the cutover job; `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass.
5. [content] The helper has doc comments (and a CHANGELOG.md entry) explaining its shape and how the trailer job and job 7 part 2 should reuse it from both athena-core and athena-cli tests.
6. [content] No production behavior changes: no edits to run.rs, gh.rs, land.rs, or gate code; the fixture is test-only and compiled only for tests.

## Expected deliverables

- crates/athena-core/src/test_support.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/cutover_fixture.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing production landing, retry_landing, or gate behavior
- Implementing the trailer job or job 7 part 2
- Changing athena.toml or defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Helper and tests in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Shape test plus cargo test/clippy/fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the check commands on the fixture rather than checking existence.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Test-only change, no production edits.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test.
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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A test using the helper runs `athena check --history` and `athena check --attribution` on the fixture's main and both report no violations; run `cargo test` and see it pass.
A test asserts the old-way landing commit carries its Athena trailers, produced through the real landing path (not hand-built commit text), and that the job was read, dispatched, verified and landed.
A test asserts C0 resolves from the cutover job's landing record on athena/records, and that the import commit and legacy-exempt record exist there; the import manifest self-check passes.
The fixture has a bare remote, a bootstrap commit, and [history] cutover_job names the cutover job; `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass.
The helper has doc comments (and a CHANGELOG.md entry) explaining its shape and how the trailer job and job 7 part 2 should reuse it from both athena-core and athena-cli tests.
No production behavior changes: no edits to run.rs, gh.rs, land.rs, or gate code; the fixture is test-only and compiled only for tests.

## Report

End with the producer report as your structured output.
