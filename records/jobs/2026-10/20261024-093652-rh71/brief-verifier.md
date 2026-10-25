# Job 20261024-093652-rh71: verify "Add records importer with manifest and self-check"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Test `full_import_passes_self_check_and_copies_every_file`. It builds a scratch repo whose C0 has several nested files under records/ (including a non-UTF-8 binary file), plus a file outside records/. It calls import_records, then lists the new commit with ls_tree. Run: `cargo test -p athena-core records_import`. Observes: import_records returns Ok. athena/records points at the returned commit. The tree holds exactly the records/ paths with blob ids equal to C0's, plus import-manifest.json, and nothing from outside records/.
  Fails without the behavior: If the copy or the self-check were missing or wrong, the tree listing or blob-id assertions would fail, or the import would return an error.
- Criterion 2: Test `manifest_lists_each_path_with_blob_and_sha256`. It reads import-manifest.json from the new commit and parses it with HistoryRecord::parse. It checks the variant is ImportManifest and that validate() is Ok. It compares the entries with C0's ls_tree output and an independent sha256_hex of each file's bytes. Run: `cargo test -p athena-core records_import`. Observes: The manifest c0 equals the C0 id. The entry count equals the number of records/ files at C0. Each entry's path and blob_id match C0, and each sha256 matches the content hash.
  Fails without the behavior: If the manifest were missing an entry or carried wrong hashes or blob ids, the equality assertions would fail.
- Criterion 3: Tests `self_check_fails_on_missing_file`, `..._changed_file`, `..._extra_file`, `..._count_mismatch` (plus a manifest-hash-mismatch case). Each uses the import_with tamper hook to commit a bad import on a fresh scratch repo, then asserts the specific ImportError variant. A control test asserts the untampered import is Ok. Run: `cargo test -p athena-core records_import`. Observes: The variants are Missing{path}, BlobMismatch/HashMismatch{path}, Extra{path} and CountMismatch{..}, each distinct, using matches!. The control import passes, so the failures come from the tampering.
  Fails without the behavior: A self-check that only compared counts, or that was skipped, would return Ok or the wrong variant for at least one tampering case.
- Criterion 4: Test `second_import_is_refused_and_tip_unchanged`. It imports once and records the tip. It imports again into the same repo and asserts AlreadyImported, then asserts the tip and the commit count are unchanged. A second test shows an unrelated pre-existing athena/records branch also blocks the import. Run: `cargo test -p athena-core records_import`. Observes: The second call returns ImportError::AlreadyImported. rev-parse of athena/records is identical before and after, and the rev-list count is 1.
  Fails without the behavior: Without the tip check, the second call would either add a commit or fail with a different writer error, and the variant or tip assertion would fail.
- Criterion 5: Inspect the diff and search for callers. Confirm the module docs state the importer is not wired in. Run: `git diff --stat`. Observes: The diff lists only the deliverable files, with no athena.toml, defaults.toml, .github/workflows or CLI file. Grep for `records_import` and `import_records` shows hits only in records_import.rs, lib.rs (the `pub mod` line), the docs and the CHANGELOG. The module //! text says it is not wired into the pipeline, the CLI or CI, and has not been run against main.
- Criterion 6: Run the full gates. Review confirms the module calls RecordsWriter::append and ImportManifest, with no copy of the writer's logic. Run: `cargo test --workspace`. Observes: All tests pass. `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean. A grep of records_import.rs shows RecordsWriter and ImportManifest in use, and no update-ref, commit_tree or hash-object calls.
  Fails without the behavior: A compile error, a failing test, a clippy warning or a formatting diff would fail the respective command.
- Criterion 7: Read the changed text. The design doc gets a 'Built (job 2)' note under §10 item 2 or §4 that names crates/athena-core/src/records_import.rs and says the importer exists and has not yet been run against main. The CHANGELOG [Unreleased] > Added gets a matching entry. Run: `git diff docs/design/records-history.md CHANGELOG.md`. Observes: Both diffs name `records_import` / `crates/athena-core/src/records_import.rs`, say the importer exists, and say it has not yet been run against main and is not wired into the pipeline.

Planned files:

- crates/athena-core/src/records_import.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## The change

Base e284a5a0a06e → candidate 68bb47e7cee7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   6 +
 Cargo.lock                               |   1 +
 crates/athena-core/Cargo.toml            |   1 +
 crates/athena-core/src/lib.rs            |   3 +
 crates/athena-core/src/records_import.rs | 570 +++++++++++++++++++++++++++++++
 crates/athena-git/src/lib.rs             |   3 +-
 crates/athena-git/src/records_branch.rs  |   3 +-
 docs/design/records-history.md           |   9 +-
 8 files changed, 593 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1053 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
