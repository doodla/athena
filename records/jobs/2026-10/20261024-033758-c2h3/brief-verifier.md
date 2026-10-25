# Job 20261024-033758-c2h3: verify "Add six records-history record types with validation to athena-data"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Six unit tests: build a valid value, serialize to pretty JSON, assert the text has the schema tag, parse with HistoryRecord::parse, assert equality. Run: `cargo test -p athena-data history`. Observes: All six round-trip tests pass.
  Fails without the behavior: Without the schema tag or serde derives, the text lacks the tag or the parsed value differs or fails to parse.
- Criterion 2: Rejection tests per type: empty required field, unknown JSON field, sha uppercase/short/non-hex, wrong schema tag, and an athena.intervention/1 body; valid examples return Ok. Run: `cargo test -p athena-data history`. Observes: Err for each malformed case, Ok for each valid example.
  Fails without the behavior: Without validate(), empty fields and bad shas are accepted; without deny_unknown_fields, the unknown field parses.
- Criterion 3: Run cargo test --workspace, cargo clippy --workspace --all-targets and cargo fmt --check; use git diff to confirm existing test code is unchanged. Run: `cargo test --workspace`. Observes: All three pass, and the existing records and history-check tests run and pass unmodified.
  Fails without the behavior: If the change broke the existing records or history-check behaviour (for example by adding AnyRecord variants or editing shared helpers), those existing tests would fail. A lint, format or compile error in the new code would fail clippy, fmt or the test build.
- Criterion 4: Read the diff of history.rs. Run: `git diff`. Observes: Every pub item has a doc comment citing docs/design/records-history.md (§5.3, §4 or §8) and D111, and the field sets match the design list.
- Criterion 5: Inspect the changed-file list and check history.rs for git or branch code. Run: `git diff --stat`. Observes: Only the deliverable files change; there is no writer or host-check code; nothing under athena-core, athena-cli, athena-gate or records/ changed.
- Criterion 6: Read the CHANGELOG diff. Run: `git diff CHANGELOG.md`. Observes: A new entry names landing-intent, landing, intervention, checkpoint, legacy-exempt and import-manifest in athena-data.

Planned files:

- crates/athena-data/src/history.rs (create)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/Cargo.toml (modify)

## The change

Base e344daf41641 → candidate edddb743c635. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   8 +
 crates/athena-data/src/history.rs | 633 ++++++++++++++++++++++++++++++++++++++
 crates/athena-data/src/lib.rs     |   2 +
 crates/athena-data/src/records.rs |   2 +
 4 files changed, 645 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1020 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the recorded check results: acceptance criteria not met or not reported: "Run cargo test -p athena-data: validation rejects malformed examples for the new types: an empty required field, an unknown field, a malformed sha (not 40 lowercase hex), and a wrong schema tag. Valid examples pass."; "Run cargo test --workspace, cargo clippy --workspace --all-targets and cargo fmt --check: all pass. Existing records and history-check tests are unchanged and still pass."; "Public new items carry doc comments citing docs/design/records-history.md (§5.3, §4 or §8 as relevant) and D111. Fields follow the design: landing-intent has PR number, head sha, head tree sha, expected base, job ids; landing has PR number, head sha, host commit, merge method, ordered commits (sha, tree) with job and verification, and the intent it fulfills; intervention has declarer, reason and commit sha; import-manifest has C0 and per-path blob id and SHA-256; legacy-exempt has entries with sha and reason."; "No writer, no host-rules check, and no git-branch code is added. No code in athena-core, athena-cli or athena-gate uses the new types. records/ on main and the history check have no diff."; "CHANGELOG.md has an entry naming the six new record types in athena-data.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
