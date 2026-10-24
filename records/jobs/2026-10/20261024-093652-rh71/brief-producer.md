# Job 20261024-093652-rh71: Add records importer with manifest and self-check

## Request (verbatim)

> Build job 2 of the records-history plan (docs/design/records-history.md sections 4 and 10, decision D111): the importer. Given a commit C0, it copies every file under records/ at C0 into one commit on the records branch at the same paths, using the writer in athena-git (records_branch) and the ImportManifest record type in athena-data (history.rs), and writes import-manifest.json listing each source path, its blob id at C0 and the SHA-256 of its contents. It then re-reads the new commit and fails if any path, blob or hash differs from C0 or the file counts differ. Tests on a scratch repository: a full import passes its self-check; a missing, changed or extra file fails it; importing twice is refused (records are append-only). Nothing runs it against this repository's main yet and the job pipeline is unchanged.

## Reading (accepted by the user)

Add a new athena-core module, which already depends on both athena-git and athena-data. It copies records/ at C0 to the athena/records branch in one commit with import-manifest.json, then re-reads the commit and fails on any path, blob, hash or count mismatch. It refuses a second import. The audience is the maintainer and the later cutover job (job 7). Nothing calls it from the pipeline or runs it against main.

Assumptions:
- The importer lives in athena-core, because athena-git and athena-data do not depend on each other and athena-core depends on both. I read the Cargo.toml files to establish this.
- The entry point takes a Git handle and a C0 commit id. It returns the new commit id or a typed error. It uses RecordsWriter::append with one Change::put per file plus import-manifest.json, in a single commit.
- import-manifest.json sits at the root of the records branch. The paths in the manifest are the source paths as they appear at C0, under records/. The imported files go at those same paths, as the request says.
- Importing twice is refused by checking that the branch already has a tip before writing. The writer's same-bytes no-op is not enough, because it would silently pass.
- Binary files are copied as bytes. The scratch repository tests use the existing athena-git testutil helper. The id and timestamp for the manifest come from RecordId::generate and the current time.
- An empty records/ at C0 is an error, because ImportManifest::validate rejects an empty paths list and an empty import must not count as success.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core records_import`. A test on a scratch repository with several nested files under records/ imports them. The importer's self-check passes. The new commit on athena/records has every records/ file at the same path with the same blob id. It also holds import-manifest.json.
2. [behavior] Run `cargo test -p athena-core records_import`. The manifest parses as HistoryRecord::ImportManifest and validates. It lists each source path with its blob id at C0 and the SHA-256 of its contents. The entry count equals the file count at C0.
3. [behavior] Run `cargo test -p athena-core records_import`. The self-check fails with a distinct error for each of three cases: a file missing from the imported commit, a file whose bytes changed, and an extra file not in C0. A count mismatch is also covered. A passing import is not what proves these.
4. [behavior] Run `cargo test -p athena-core records_import`. A second import into a repository that already has the records branch is refused, and the branch tip is unchanged.
5. [content] No existing caller, CLI command, job pipeline step or workflow invokes the importer, and nothing is run against this repository's main. The diff touches no athena.toml, defaults.toml or .github/workflows file. The importer module docs say the importer is not wired in.
6. [behavior] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check`. All pass, and the importer reuses RecordsWriter and ImportManifest rather than reimplementing them.
7. [content] The design doc's job-2 status text and the CHANGELOG say the importer exists and is not yet run against main, and a reader can see where the code lives.

## Expected deliverables

- crates/athena-core/src/records_import.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)

## Out of scope

- Running the importer against this repository's main or on the real records/ directory
- Wiring the importer into the job pipeline, the CLI or CI
- Cutover (job 7), host rules, and the other record writers

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Importer and tests are Rust.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses RecordsWriter, ImportManifest and the existing sha2 crate; builds no new tooling.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Scratch-repository tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the failing and passing import outcomes, not that types exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module plus the lib.rs registration and a doc note.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty records/ is an error, and the self-check compares counts.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module docs explain the import and the self-check for an outside reader.
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
