# Job 20261105-180320-62j6: Record peak memory and cores for cold builds; show in status

## Request (verbatim)

> Measure agent memory, build memory, CPU and the slot-relevant sizes (step 1). Build step from docs/design/agent-and-build-slots.md section 8; follow that section for the files touched and how the effect is measured.
> 
> Acceptance:
> - the build-measurement record gains peak resident memory and cores used for a cold build; old records still read
> - athena status prints the new figures
> - tests cover the new fields and reading an old record

## Reading (accepted by the user)

Build step 1 of the slots design: each cold build in `athena measure-build` also records peak resident memory and cores used, stored as optional fields on `BuildMeasurement`, so old records still parse. The `build` line of `athena status` prints them for the maintainer sizing slots.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs (BuildRun, CargoBuilder, CargoBuild::build runs cargo via Command::output and times it; Athena::measure_build builds two cold worktrees and commits the record), crates/athena-data/src/records.rs (BuildMeasurement with deny_unknown_fields; validate; AnyRecord::BuildMeasurement; round-trip test a_build_measurement_has_a_path_a_schema_and_round_trips), crates/athena-core/src/status.rs (build_measurement_line, tests in build_measurement_tests), crates/athena-cli/tests/status_build_measurement.rs. Entry point is `athena measure-build`; status reads the latest record.
- Records validation: the schema tag athena.build-measurement/1 stays; new fields are Option with #[serde(default, skip_serializing_if = "Option::is_none")] so old records parse under deny_unknown_fields and the AnyRecord round-trip stays valid; validate() rejects non-finite or negative values. No new record or data file is added.
- Peak resident memory and cores are measured directly from the cargo process tree (e.g. wait4 rusage maxrss of the child plus CPU time divided by wall time as cores used), not estimated; the exact mechanism is the producer's choice, preferring an existing maintained crate over hand-rolled code. Peak memory and cores are recorded per cold build (first and second), or for the first at minimum.
- Audience is the maintainer sizing agent and build slots (section 7 of the design). The one-off sccache run in the design text uses code that already exists and needs no change. Agent memory measurement is read as out of this step's record-field scope beyond the build record, since the acceptance names only the build record.
- Changes to disk.rs fake builders in tests (FakeBuilder) and BuildMeasurement literals in hestia.rs, status.rs and the CLI test are needed to compile.

## Acceptance criteria

1. [content] BuildMeasurement in crates/athena-data/src/records.rs gains optional peak-resident-memory and cores-used fields with serde defaults, and validate() rejects negative or non-finite values.
2. [content] BuildRun and CargoBuild::build in crates/athena-core/src/disk.rs capture peak resident memory and cores used of each cold build from the cargo process, and measure_build stores them in the record.
3. [behavior] A test parses a JSON build-measurement record written without the new fields and it deserializes with those fields None; a test round-trips a record with them set. Run `cargo test -p athena-data`.
4. [behavior] build_measurement_line prints the peak memory and cores when present and still prints old records without them; tests in status.rs and crates/athena-cli/tests/status_build_measurement.rs check the printed text. Run `cargo test -p athena-core -p athena-cli`.
5. [behavior] A disk.rs test with a fake builder checks measure_build writes the builder's memory and core figures into the committed record. Run `cargo test -p athena-core`.
6. [behavior] `cargo test` passes for the workspace and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
7. [content] The design doc docs/design/agent-and-build-slots.md or CHANGELOG.md notes the new fields briefly, if the repository convention records such changes.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-cli/tests/status_build_measurement.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)

## Out of scope

- Running sccache as a separate one-off measurement (existing code already does it)
- Build steps 2 to 8 of the slots design (landing stage, slot queue, shared target dirs)
- Changes to athena.toml, defaults.toml or workflows
- Editing existing records under records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for new fields, old-record reading and status output; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check printed status text and deserialized values by running tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is memory and cores visible in the status build line.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Prefer an existing maintained crate or std/libc rusage for resource measurement.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the build-measurement path.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-180320-62j6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-180320-62j6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

BuildMeasurement in crates/athena-data/src/records.rs gains optional peak-resident-memory and cores-used fields with serde defaults, and validate() rejects negative or non-finite values.
BuildRun and CargoBuild::build in crates/athena-core/src/disk.rs capture peak resident memory and cores used of each cold build from the cargo process, and measure_build stores them in the record.
A test parses a JSON build-measurement record written without the new fields and it deserializes with those fields None; a test round-trips a record with them set. Run `cargo test -p athena-data`.
build_measurement_line prints the peak memory and cores when present and still prints old records without them; tests in status.rs and crates/athena-cli/tests/status_build_measurement.rs check the printed text. Run `cargo test -p athena-core -p athena-cli`.
A disk.rs test with a fake builder checks measure_build writes the builder's memory and core figures into the committed record. Run `cargo test -p athena-core`.
`cargo test` passes for the workspace and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
The design doc docs/design/agent-and-build-slots.md or CHANGELOG.md notes the new fields briefly, if the repository convention records such changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
