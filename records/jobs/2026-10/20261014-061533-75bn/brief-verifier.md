# Job 20261014-061533-75bn: verify "Add 'athena review' to record class labels; show unreviewed in status"

## Request (verbatim)

> Add a way for a person to review an introspection's class, so 'athena score-jev' can ever have labels: today it counts only introspections a reviewer labeled, and no command records a review. Add 'athena review <introspection-id>' that shows the record (what happened, class, cause, prevention, and Jev's shadow answer) and records the person's verdict in one step: the class is right, or the right class is another existing or new class, with an optional one-line note. Show the number of unreviewed introspections in 'athena status' so it is easy to keep up. A review is a record and never edits the introspection itself. Verify with fixture records in a temporary directory.

## Reading (accepted by the user)

Add `athena review <introspection-id>` that prints the introspection (what happened, class, cause, prevention, Jev's shadow answer) and, in the same call, commits an `athena.class-label/1` record with the person's verdict: class right, another existing class, or a new class (a new `athena.failure-class/1` record), plus an optional one-line note. `athena status` gains an unreviewed-introspection count. The introspection record is never touched.

Assumptions:
- One-step means flags, not prompts: `athena review <id> --right`, `--class <class-id>`, or `--new-class <name> --description <line>`, with optional `--note <line>` and `--json`. The record is shown, then the verdict is committed in the same call. Exactly one verdict flag is required.
- `ClassLabel` has no note field, so add an optional `note: Option<String>` (serde default, skipped when absent) to `athena.class-label/1`. Old records still parse. The note must be a single line.
- `--right` copies the introspection's own failure_class into the label. If the introspection has no class, `--right` is refused with a hint to use `--class` or `--new-class`.
- A new class is written as a `FailureClass` record (status proposed, proposed_by = the introspection id) in the same commit as the label.
- `labeled_by` is `person`. Reviewing an already-reviewed introspection adds a newer label, which the existing 'newest wins' rule in labeled::build already handles. The command prints a notice that it replaces the earlier label.
- Unreviewed = introspections with no class-label record, the same rule as `labeled::build` (`Exclusions.unreviewed`). The status count reuses one shared helper, not a second definition.
- An unknown or ambiguous introspection id, or a class id not in the registry, is an error with nothing written.
- Writes go through `commit_records`, as other commands do. The `is_repeat` question is not asked.

## Acceptance criteria

1. [behavior] Run `cargo test`: tests using fixture records in a temporary directory show that after `review` with each verdict (right, existing class, new class, with and without a note) a class-label record exists with the expected class, note and labeled_by person, and the introspection file is byte-identical.
2. [behavior] Run `cargo test`: a test shows that after a review `labeled::build` no longer counts the introspection as unreviewed (`unreviewed` drops by one).
3. [behavior] Run `cargo test`: a test shows `athena status` (StatusView and its display) reports the number of unreviewed introspections and that the count falls after a review. When the count is above zero, `next` hints at `athena review`.
4. [behavior] Run `cargo test`: tests show errors write nothing: unknown introspection id, unknown class id, no verdict flag, two verdict flags, `--right` on an introspection with no class, and a multi-line note.
5. [behavior] `cargo clippy` (workspace, all targets) is clean and `cargo fmt --check` passes.
6. [content] `ClassLabel` in crates/athena-data/src/records.rs has an optional `note` field that is absent from the JSON when None. A round-trip test covers both with and without a note, and the existing label test still passes.
7. [content] The review output shows what happened, class (name and id), cause, prevention, and Jev's shadow class choice and probabilities, or says Jev was not asked or failed.
8. [content] README.md's command table lists `athena review` and notes the status count. CHANGELOG.md has an entry. The docs that describe `athena.class-label/1` (docs/design/defaults-loop.md section 10.2) mention the note and that the build now writes these records.

## Expected deliverables

- crates/athena-core/src/review.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/review.rs (create)
- crates/athena-cli/tests/labeled.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- docs/design/defaults-loop.md (modify)
- docs/records.md (modify)

## Out of scope

- Prompting interactively or asking the is-repeat question.
- Editing, deleting or re-committing any introspection record.
- Changing score-jev's scoring, split or verdict.
- Reviewing in bulk, a separate command that lists unreviewed ids, or auto-labeling with a stronger model.
- Governance files (athena.toml, defaults.toml, CI, lint config).

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust in the existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module and CLI behavior have passing tests using temporary-directory fixture records; clippy clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checked by running cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the review command, one optional record field, one status count.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: README, CHANGELOG and design docs updated for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is that no command writes class-label records; the fix adds the writer instead of seeding fixtures.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The flag design, the note field and the unreviewed definition were recovered from labeled.rs, records.rs and status.rs, not asked.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses existing record writing (`commit_records`), `RecordId` and the records parser. No new crates.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base d32bd35e6eda → candidate 5be1f5ca6ebe. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                       |  18 +-
 README.md                          |   1 +
 crates/athena-cli/src/display.rs   | 101 ++++++++
 crates/athena-cli/src/main.rs      |  48 ++++
 crates/athena-cli/tests/labeled.rs |   1 +
 crates/athena-cli/tests/review.rs  | 459 +++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/labeled.rs  |  43 +++-
 crates/athena-core/src/lib.rs      |   3 +
 crates/athena-core/src/review.rs   | 388 +++++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs   |  12 +
 crates/athena-data/src/records.rs  |  19 +-
 docs/design/defaults-loop.md       |  14 +-
 docs/records.md                    |   1 +
 13 files changed, 1097 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (527 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
