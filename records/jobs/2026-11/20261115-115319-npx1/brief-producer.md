# Job 20261115-115319-npx1: Queue jobs for a single build-folder slot so two can run at once

## Request (verbatim)

> From Sri (the owner), word for word: "send both" (this is part 2 of 2).
> 
> Background (not Sri's own words; added so the request can be read on its own). Part 1 sets max_concurrency = 1 as a stopgap because two jobs each keep a 15-24 GB build folder on a disk that fits one (see the part 1 request).
> Part 2, the lasting fix: let two jobs run at once again, but queue for disk, not only for running builds. Only one job at a time may hold a build folder; a second job may read and plan, and waits for the folder to be freed before its producer starts building, instead of being held or discarded for low disk. When this lands and is shown to keep free space above the low-disk gate with two jobs, set max_concurrency back to 2.

## Reading (accepted by the user)

Add a disk-slot gate so only one job at a time holds a build folder. A second job may read and plan, and waits for the slot before its producer builds, instead of being held or discarded for low disk. With the gate in place, max_concurrency goes back to 2. The audience is the owner running the drain on a disk that fits one build folder.

Assumptions:
- Code survey: read crates/athena-core/src/admission.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/state.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/commands.rs and crates/athena-data/src/config.rs. Grep shows max_concurrency and build-slot handling in these files. The entry point is the drain in crates/athena-cli/src/drain.rs. Admission and the disk gate run before a job is started. The producer build step is where the new wait goes. Direct measurement is the count of jobs holding a build folder at the same moment, not a proxy. The producer should confirm the exact functions before editing. Any new record or data file under crates/ must be covered by check_record_file, AnyRecord or the round-trip test; prefer no new record type.
- Term ladder: not used. Build-folder slot means a single exclusive hold on the right to create or keep a per-job build directory (target folder). It is taken before the producer builds and freed when the job lands, is rejected or is discarded.
- Reading of the request: planning, reading and verification need no slot. Only the producer build phase waits. A job waiting for the slot is neither discarded nor held for low disk. If free space is below the low-disk gate for a reason other than another job's folder, existing behavior stays.
- Field meaning: max_concurrency in athena.toml (line 205, live value 2 in this copy) bounds concurrent jobs. Its meaning is unchanged. Part 1 may already have set it to 1. This job ends with it at 2. Consumers: crates/athena-data/src/config.rs, crates/athena-cli/src/drain.rs, crates/athena-core/src/admission.rs.
- The slot count is one, fixed by design, and the reason is that the disk fits one 15-24 GB build folder. Whether a config key for the slot count is added is left to the planner, with default 1.
- Environment: none. No CI workflow, push or records read is needed. Verification uses cargo test, clippy and fmt on the working copy.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (or crates/athena-core/src/run/tests.rs) drains two jobs with max_concurrency = 2. It asserts that both jobs reach planning together, that the second producer starts only after the first job frees its build folder, and that the peak count of build-folder holders is 1. Both jobs land, and neither is discarded or held for low disk. Run with cargo test; the test fails if the slot gate is removed.
2. [behavior] A test shows a job waiting for the slot does not count as a low-disk hold or discard. With a simulated free-space value below the gate while another job holds the folder, the waiting job stays queued and is later started. Run with cargo test.
3. [behavior] A test shows the slot is freed on every exit path (landed, rejected, failed or discarded job). A second job that was waiting then proceeds. Run with cargo test.
4. [content] The slot gate is enforced in code, in crates/athena-core/src/admission.rs, crates/athena-core/src/disk.rs or crates/athena-cli/src/drain.rs, and is not prose only. A Grep shows it called from the producer-start path.
5. [content] athena.toml has max_concurrency = 2 at line 205 in the final tree (git diff shows it unchanged or restored from 1 to 2). The CHANGELOG.md entry says the stopgap is lifted.
6. [content] A new appended entry in docs/decisions.md, with a Provenance bullet, records the design: one build-folder slot, the alternatives considered (a global max_concurrency of 1, a free-disk check only), and why. docs/design/parallel-jobs.md or docs/design/agent-and-build-slots.md is updated to describe the disk slot. No existing decisions entry is edited.
7. [behavior] cargo test, cargo clippy and cargo fmt --check pass on the final tree, with no new clippy warnings.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/design/agent-and-build-slots.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the low-disk gate threshold itself.
- Running a live drain on the owner's machine to prove free space; tests simulate it.
- Adding more than one build-folder slot or a cleanup of old build folders.
- Edits to records/ or existing docs/decisions.md entries.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for the slot gate; cargo test and clippy criteria.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Gate enforced in code, not prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain with two jobs and check peak slot holders.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: A new decision entry records alternatives.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Gate on disk use directly rather than limiting job count.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Time budget

Each producer run, the first and any send-back, is stopped after 86400 seconds of wall-clock time. A stopped run sends no report, so what is on disk when it stops must already be a usable change.

Pace the work by its order. Do the smallest change that meets every acceptance criterion first, one file at a time, and leave extras for afterwards; report as soon as the criteria are met. Leave no half-edited file and no broken build between steps. Use targeted tests while you work (`cargo test -p <crate> <filter>`). The full suite runs at most once, and Athena runs it again on your result (see "Checks before the verifier" below), so a second full run only spends this limit.

You have no clock of your own. If the commands Bash may run (below) include `date`, note the time when you start and read it between steps; without it, count steps against the order above.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

36 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261115-115319-npx1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261115-115319-npx1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-cli/tests/drain.rs (or crates/athena-core/src/run/tests.rs) drains two jobs with max_concurrency = 2. It asserts that both jobs reach planning together, that the second producer starts only after the first job frees its build folder, and that the peak count of build-folder holders is 1. Both jobs land, and neither is discarded or held for low disk. Run with cargo test; the test fails if the slot gate is removed.
A test shows a job waiting for the slot does not count as a low-disk hold or discard. With a simulated free-space value below the gate while another job holds the folder, the waiting job stays queued and is later started. Run with cargo test.
A test shows the slot is freed on every exit path (landed, rejected, failed or discarded job). A second job that was waiting then proceeds. Run with cargo test.
The slot gate is enforced in code, in crates/athena-core/src/admission.rs, crates/athena-core/src/disk.rs or crates/athena-cli/src/drain.rs, and is not prose only. A Grep shows it called from the producer-start path.
athena.toml has max_concurrency = 2 at line 205 in the final tree (git diff shows it unchanged or restored from 1 to 2). The CHANGELOG.md entry says the stopgap is lifted.
A new appended entry in docs/decisions.md, with a Provenance bullet, records the design: one build-folder slot, the alternatives considered (a global max_concurrency of 1, a free-disk check only), and why. docs/design/parallel-jobs.md or docs/design/agent-and-build-slots.md is updated to describe the disk slot. No existing decisions entry is edited.
cargo test, cargo clippy and cargo fmt --check pass on the final tree, with no new clippy warnings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
