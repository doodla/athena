# Job 20261114-045727-c8gn: Show per-prompt-file token room in the reader brief

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has le
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: reading-edits-prompt-without-checking-token-budget (20261024-002642-bm4c)
> Cause: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has left. The shared verifier prompt is composed into several sets, so an addition there costs every set that includes it, and one set had 4 token...
> Seen 7 time(s) in 7 jobs. The introspections that record it:
> Introspection 20261024-002725-eqs1 (job 20261024-002644-1as8); prevention it proposed: Add a pre-dispatch check for any job whose reading touches prompt files. It computes the token headroom of every prompt set that composes each target file, the same figure the budgets test prints, and adds it to the reading. If the planned ...
> Introspection 20261024-104248-8p9c (job 20261024-093833-vava); prevention it proposed: Before a reading is accepted, run a check that computes the remaining token room for every composed prompt (producer per capability, verifier, and so on) and puts the table into the reading stage's context. Add a check that any reading or p...
> Introspection 20261025-090447-2zm3 (job 20261025-034452-wqp8); prevention it proposed: Add a check at reading admission. It takes each prompt file the reading or plan says it will grow, and compares the planned growth with the remaining budget the budgets test computes. It rejects the reading, naming the file and the room lef...
> Introspection 20261027-170109-ycpz (job 20261027-054203-wkp5); prevention it proposed: Before dispatch, a code check computes the remaining token room for each prompt set a reading or plan will edit. It compares that room with the estimated addition and puts the result in the reading. Where room is short, the reading must eit...
> Introspection 20261108-004800-fka0 (job 20261024-232057-r4pj); prevention it proposed: Add a check that runs before the verifier on any job whose diff touches prompt files. It runs the prompt, brief and budget tests (the athena-context lib tests and the budgets tests) and reports the failing test name and assertion. It also f...
> Introspection 20261108-013638-a1y0 (job 20261025-154650-h9ea); prevention it proposed: Before dispatch, compute the remaining token room for any prompt a request edits and put the figure in the reading. After the producer edits, run a code check on the prompt diff. It must confirm that every old bullet is still present as its...
> Introspection 20261109-052112-m8x3 (job 20261109-035221-jj8g); prevention it proposed: Add a code check that runs before verification on any job editing files under crates/athena-context/prompts/. It extracts the rule statements (bullets and must/never lines) from the old and new version of each prompt. It then requires every...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader's "Prompt token headroom" lists sets by component id (e.g. frame.verifier), so a reading must map a file path to every set composing it and take the tightest room itself; 7 jobs (e.g. 20261024-002644-1as8) planned prompt growth past a set's room. Fix it at the source by adding to the reader brief a per-file table: each prompt file path, the sets that compose it, and the smallest room left. No refusal rule is added.

Assumptions:
- Code survey: read crates/athena-context/src/lib.rs (HeadroomRow, headroom_rows, component constants with include_str! of prompts/*.md), crates/athena-context/src/briefs/reader.rs (headroom_section, ReaderInputs.headroom), crates/athena-core/src/reading.rs (reader_headroom feeds the brief at reading.rs:629), crates/athena-cli/tests/budgets.rs (headroom rows equal budget figures), crates/athena-core/src/run_pin_tests.rs:1582. No record or data file is added, so no check_record_file/AnyRecord work.
- The brief already gives per-set headroom, but by component id (frame.verifier, capability.change.verifier), not by file path; the reader must infer which edited file feeds which sets and take the minimum. That missing mapping is the cause; the table is derived from the same HeadroomRow figures as the budgets test.
- Goal: the reader stage (audience: the reader agent and the verifier judging readings) sees, per prompt file path, the tightest room, so it picks a target file with room. Past jobs 20261024-002644-1as8 and the other six introspected jobs are the failure instances; the earlier retried jobs (vava, wqp8) are context, not retried here.
- Capability sections map by id: capability.<id>.producer/verifier to crates/athena-cap-<id>/prompts/producer.md / verifier.md; defaults and catalog are generated, not files, and are left out of the per-file table.
- The reader prompt (reader.md, 2 tokens left) is not edited; the new text lives in the brief, which has no token budget.

## Acceptance criteria

1. [content] crates/athena-context/src/lib.rs has a function that maps each prompt component (contract, frame.*, capability.<id>.producer/verifier) to its repository path (e.g. crates/athena-context/prompts/verifier.md, crates/athena-cap-change/prompts/verifier.md) and, from the HeadroomRow list, returns for each file the sets that compose it and the smallest tokens-left among them.
2. [content] headroom_section in crates/athena-context/src/briefs/reader.rs renders that per-file table under '# Prompt token headroom', as lines naming the file path, its sets and the tightest room, in addition to the existing per-set rows.
3. [behavior] A unit test in crates/athena-context builds rows where shared frame.verifier is composed by verifier (change) with 59 left and verifier (study) with 36 left, and asserts the per-file entry for crates/athena-context/prompts/verifier.md lists both sets and says 36 left; it fails without the change. Run: cargo test -p athena-context headroom.
4. [behavior] A reader-brief test asserts the rendered brief contains the line for crates/athena-context/prompts/contract.md with every set composing it and the minimum room, and that no per-file section appears when headroom is empty. Run: cargo test -p athena-context reader_brief.
5. [behavior] A test in athena-core (reading brief path, next to the_reading_brief_carries_the_token_headroom_of_the_reader_and_verifier_sets in run_pin_tests.rs) asserts the real reading brief includes a per-file line for crates/athena-cap-change/prompts/verifier.md. Run: cargo test -p athena-core token_headroom.
6. [content] CHANGELOG.md has an [Unreleased] entry naming the root cause (the brief gave per-set rows by component id, not per-file room); a new appended docs/decisions.md decision with a Provenance bullet may record it. Existing entries are not edited.
7. [behavior] The recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test).
8. [content] No refusal rule is added; expected 0% of newly refused readings.

## Expected deliverables

- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Adding any admission or refusal check on readings or plans
- Editing any prompt file under crates/*/prompts/
- Post-producer diff checks of prompt rules (other introspection proposals)
- Changing token budgets or athena.toml, defaults.toml
- Editing records/ or existing docs/decisions.md entries

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix at the brief that lacks per-file room
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: tests fail without the change
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: criteria check the brief text rendered
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

- path: <state>/jobs/20261114-045727-c8gn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045727-c8gn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/src/lib.rs has a function that maps each prompt component (contract, frame.*, capability.<id>.producer/verifier) to its repository path (e.g. crates/athena-context/prompts/verifier.md, crates/athena-cap-change/prompts/verifier.md) and, from the HeadroomRow list, returns for each file the sets that compose it and the smallest tokens-left among them.
headroom_section in crates/athena-context/src/briefs/reader.rs renders that per-file table under '# Prompt token headroom', as lines naming the file path, its sets and the tightest room, in addition to the existing per-set rows.
A unit test in crates/athena-context builds rows where shared frame.verifier is composed by verifier (change) with 59 left and verifier (study) with 36 left, and asserts the per-file entry for crates/athena-context/prompts/verifier.md lists both sets and says 36 left; it fails without the change. Run: cargo test -p athena-context headroom.
A reader-brief test asserts the rendered brief contains the line for crates/athena-context/prompts/contract.md with every set composing it and the minimum room, and that no per-file section appears when headroom is empty. Run: cargo test -p athena-context reader_brief.
A test in athena-core (reading brief path, next to the_reading_brief_carries_the_token_headroom_of_the_reader_and_verifier_sets in run_pin_tests.rs) asserts the real reading brief includes a per-file line for crates/athena-cap-change/prompts/verifier.md. Run: cargo test -p athena-core token_headroom.
CHANGELOG.md has an [Unreleased] entry naming the root cause (the brief gave per-set rows by component id, not per-file room); a new appended docs/decisions.md decision with a Provenance bullet may record it. Existing entries are not edited.
The recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test).
No refusal rule is added; expected 0% of newly refused readings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
