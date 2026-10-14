# Job 20261014-053916-0tmz: verify "Add read-only labeled set and Jev shadow scoring command"

## Request (verbatim)

> Build step 6 of the build order in docs/design/defaults-loop.md section 12: the labeled set and its scoring (sections 10.2 and 10.3), as a read-only command that the user runs over the real records after landing (agents' copies leave out records/). It builds the labeled set from introspection records whose class a person has reviewed, scores Jev's recorded shadow answers against those labels with the scoring rule in section 10.3, and reports the score with its sample size and interval, plus the promotion verdict from section 10.4. With too few labeled cases it must say so and give no verdict, rather than a confident score. Verify with the fake Jev and fixture records in a temporary directory.

## Reading (accepted by the user)

Add a read-only `athena` command that builds the labeled set from introspection records whose failure class a person reviewed. It scores the stored Jev class and repeat answers by Brier skill against two references, with sample size, interval and the §10.4 verdict. Below the minimum labeled count it says so and gives no verdict. It is tested with FakeJev and fixture records in a temp directory.

Assumptions:
- No record of a person's class review exists in the code today (searched crates/ for review/label markers). I assume a person's review is a new record kind, e.g. athena.class-label/1 in athena-data records.rs, naming an introspection, the reviewed class (a registry id or none), who labeled it (person or stronger model spot-checked) and when. The new command only reads these. The writer of such records is out of scope.
- Only the failure-class (matching) and is-repeat answers are scored, since those are the only Jev answers recorded (IntrospectionJev). Set membership, signal match, cluster, confidence, prose and cost scoring from §10.3 are not built.
- Reference forecasters: always predict 'none', and a word-overlap matcher over the stored what_happened/cause text against the registry class names and descriptions. Skill is the Brier improvement over the better reference.
- Time split: records before a cut date calibrate, later ones test. The cut is a command flag with a default of the earliest two thirds by time. Only the test part is scored for the verdict.
- The interval is a normal-approximation 95% interval on mean per-record skill, given only when the test size n exceeds (1.96*sd/mean)^2 per §10.4 item 1. Otherwise the report says no bound was computed and shows the raw Brier scores and a per-record table.
- Verdict logic covers only the §10.4 items computable from these records: item 1 (skill), item 3 (false-match rate on 'none' labels under 10%) and the class-grouping check of item 2. Items 4, 5, 6 and 7 are reported as not evaluated. A minimum labeled-case count constant (about 10 test cases) triggers 'insufficient labeled cases' with no score or verdict.
- The command is a new Score-style subcommand in athena-cli main.rs (with a --json flag, like Propose) over a new athena-core module that never opens a file for writing.

## Acceptance criteria

1. [content] A new module in crates/athena-core builds the labeled set only from introspection records that have a matching person-review record, and keeps the labeler (person or stronger model) on each row.
2. [content] The scoring module computes the Brier score of Jev's class probability vector against the label, the two references (always 'none', word overlap), skill over the better reference, top-1 accuracy and the false-match rate on 'none' labels, split by time.
3. [content] The module and command never open a file for writing, and a test asserts it as proposals.rs does.
4. [content] docs/design/defaults-loop.md step 6 is updated to say what was built and what is not (membership, signal match, cluster, prose and cost scoring, and live shadow run), and CHANGELOG.md is updated.
5. [behavior] Run `cargo test`: tests with FakeJev answers and fixture records in a temp directory show that (a) enough labeled cases give a score, sample size and interval-or-'no bound computed' note plus a verdict, (b) too few labeled cases give an explicit 'too few labeled cases' message with no score and no verdict, (c) unreviewed introspections and records with no Jev answer are excluded and counted, (d) a Jev-error record is excluded and counted.
6. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean.
7. [behavior] Run `cargo run -p athena-cli -- --help`: the new command is listed, and its `--json` output on an empty temp store reports 'too few labeled cases' and exits cleanly.

## Expected deliverables

- crates/athena-core/src/labeled.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/tests/labeled.rs (create)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- README.md (modify)

## Out of scope

- Running the command over the real records/ (the user does this after landing).
- A command that writes review or label records.
- Scoring set membership, signal match, cluster assignment, confidence, prose faithfulness, cost and latency.
- Promotion itself, or any change to what `athena status` shows as live.
- Live Jev calls; only FakeJev is used.
- Changes to athena.toml, defaults.toml, CI, or lint configuration.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The command and scoring are Rust in the existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new module has passing tests, and cargo test and clippy are clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior is exercised with FakeJev and fixture records in a temp directory.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The report labels sample size, the interval and the estimates; the 10% false-match and sizing figures are marked as estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command and one module covering the matching and repeat scoring of §10.3 only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the design doc update are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The review is modeled as an explicit record, not inferred from the status of the introspection.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base cb7a6f385c45 → candidate 24a6878c45ec. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |   13 +
 README.md                            |    1 +
 crates/athena-cli/src/display.rs     |  114 ++++
 crates/athena-cli/src/main.rs        |   28 +
 crates/athena-cli/tests/labeled.rs   |  303 +++++++++
 crates/athena-core/src/introspect.rs |    2 +-
 crates/athena-core/src/labeled.rs    | 1241 ++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs        |    3 +
 crates/athena-core/src/store.rs      |    3 +
 crates/athena-data/src/records.rs    |   70 +-
 docs/design/defaults-loop.md         |   24 +
 11 files changed, 1799 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (516 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
