# Job 20261025-031609-k0tt: verify "Add shared records-history cutover fixture for core and cli tests"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A cli integration test builds the fixture, then runs the real `athena` binary with `check --history` and `check --attribution` in the fixture's work repo (env ATHENA_STATE_DIR set to a scratch dir, `CI` removed). It asserts exit code 0 and no violation lines in stdout for both. Run: `cargo test --locked -p athena-cli --test cutover_fixture`. Observes: Both checks exit 0 and print pass/no violations on the fixture's main.
  Fails without the behavior: If the cutover import, the landing record or the legacy-exempt record were missing or hand-shaped wrongly, `check --history` or `--attribution` would exit 2 with a violation (for example `legacy-exempt record missing` or an unattributed commit), and the test would fail.
- Criterion 2: The shape test reads the old-way job's landing commit on main with `git show`/`Git::run`. It asserts the trailers `Athena-Job`, `Athena-Verified-Content` and the others the writer emits are present, and that the commit's parent chain starts at the bootstrap commit. It loads the records at main and asserts the job has a reading, a dispatch (candidate), a passing verification and a landed outcome. The job was made by `ask` and `go`, not by commit text. Run: `cargo test --locked -p athena-cli --test cutover_fixture`. Observes: The landing commit message carries the trailers, and the job's records show read, dispatched, verified and landed.
  Fails without the behavior: If the fixture hand-built the commit or skipped `go`, the trailers or the candidate, verification or outcome records would be absent and the assertions would fail.
- Criterion 3: The shape test fetches the bare remote's `athena/records` tip. It asserts `attribution::cutover_commit_at(git, tip, cutover_job)` returns the squash commit on main (C0), that `records_import::verify_cutover_import(git, c0, tip)` is Ok, and that the import commit holds `import-manifest.json` naming C0 and `legacy-exempt.json` (`LEGACY_EXEMPT_PATH`) parsed as a `LegacyExempt` record. It also asserts that the import commit's parent is the cutover job's landing record commit. Run: `cargo test --locked -p athena-cli --test cutover_fixture`. Observes: C0 is resolved from the landing record, the manifest self-check passes, and the import and exempt record exist on the records branch.
  Fails without the behavior: If the cutover hook did not run, `cutover_commit_at` would find no landing, or the manifest and legacy-exempt files would be missing, and the asserts would fail.
- Criterion 4: The shape test asserts the remote is a bare repo (`rev-parse --is-bare-repository`), that main on it equals the work repo's main, that the first commit is the bootstrap commit (trailer `Athena-Job: bootstrap`), and that `cutover_job_for(git, main)` equals the cutover job id. Then the three repository checks run. Run: `cargo test --locked`. Observes: The tests pass. `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` are clean.
  Fails without the behavior: A fixture with no bare remote or no `cutover_job` setting fails the shape asserts.
- Criterion 5: Review the doc comments on `test_support` and `CutoverFixture` (the shape, a numbered history, a how-to-reuse section for the trailer job and job 7 part 2, from core and from cli tests) and the CHANGELOG `[Unreleased]` entry. `git diff` shows them. Run: `git diff -- CHANGELOG.md crates/athena-core/src/test_support.rs docs/decisions.md`. Observes: The doc comments and the CHANGELOG entry name both later jobs and say how to call the helper from each crate.
- Criterion 6: `git diff --stat` shows no edits to run.rs, gh.rs, land.rs or gate crates, and `lib.rs` changes only by the cfg-gated `pub mod`. The default build is unchanged: `cargo build` compiles without the test-support feature, and `cargo clippy --all-targets` still passes. Run: `git diff --stat`. Observes: Only test_support.rs, lib.rs (one gated line plus module doc), the core Cargo.toml feature, the cli dev-dependency line, common/mod.rs, cutover_fixture.rs, CHANGELOG.md and docs/decisions.md changed.

Planned files:

- crates/athena-core/src/test_support.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/cutover_fixture.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 495bec2f5d52 → candidate ca4d1fdd5866. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  11 +
 crates/athena-cli/Cargo.toml               |   1 +
 crates/athena-cli/tests/common/mod.rs      |  91 ++++++
 crates/athena-cli/tests/cutover_fixture.rs | 218 +++++++++++++
 crates/athena-core/Cargo.toml              |   4 +
 crates/athena-core/src/lib.rs              |   4 +
 crates/athena-core/src/test_support.rs     | 492 +++++++++++++++++++++++++++++
 docs/decisions.md                          |  23 ++
 8 files changed, 844 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1184 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
