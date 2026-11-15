# Job 20261114-194517-c16g: Show per-stage durations and the main reason for each in status

## Request (verbatim)

> From Sri (the owner), word for word: "Drop the time limits for now. Let's just measure how long things take and why"
> 
> Background (not Sri's own words): the first half landed as 20261114-184053-050s (time limits raised to 24 hours). Its reading said the second half would be queued as its own request, but nothing queues it, so this is that request. What is wanted: for each stage of each job (reading, each planning round, producer, checks, verifier, landing, waiting in the queue, waiting for disk), record and show how long it took and the main reason it took that long (model and effort, tokens, tool calls, build and test time, retries, waits), so slow stages can be explained.

## Reading (accepted by the user)

Part 2 of Sri's request (follow-up to 20261114-184053-050s, which landed the 24-hour limits): extend the existing throughput figures in `athena status` so each job stage (reading, each planning round, producer, checks, verifier, landing, queue wait, disk wait) shows its duration and a main reason (model and effort, tokens, tool calls, check build/test time, retries, waits). The audience is Sri, who needs to see why slow stages were slow.

Assumptions:
- Previous job 20261114-184053-050s (landed, verdict pass) is addressed so: its part 1 (athena.toml limits at 86400 s, check_timeout_s 14400 s, and edits to config.rs, run/tests.rs, CHANGELOG.md) is kept unchanged and not touched here. Its two findings do not apply to this job's goal: the check_reading_size side effect (the guard now almost never fires) and the stale 1800 figure for check_timeout_s in docs/design/agent-and-build-slots.md and the config.rs comment are out of scope; this job does not edit athena.toml, config.rs or run/tests.rs, so the changed files athena.toml, config.rs, run/tests.rs are excluded here. CHANGELOG.md gets a new entry.
- Goal and audience: Sri, the owner, reads `athena status` output to explain slow stages; the reading is judged by whether a slow stage's reason is visible there.
- Code survey: crates/athena-core/src/status/throughput.rs already derives stage durations (landing and rebuild, check_durations) from invocation, check and landing records; crates/athena-core/src/status.rs and crates/athena-cli/src/display.rs render it; crates/athena-cli/tests/status_snapshot.rs exercises `status --snapshot`. Entry point is the status command in crates/athena-cli/src/commands.rs and main.rs. Records are read, not changed, so no new record file is added and the records validation allow-list (check_record_file, AnyRecord) is not touched; if an existing record lacks a field (tokens, tool calls, queue or disk wait), the figure shows it as unrecorded rather than guessing. Durations are measured directly from record timestamps, not a stand-in.
- Existing code: crates/athena-core/src/status/throughput.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, crates/athena-core/src/checks.rs and crates/athena-core/src/slots.rs and disk.rs (wait data) are extended, not replaced.
- Environment: the verifier reads git history of the repository snapshot with `git log` (read-only, local) and the committed evidence directory used by `status --snapshot`; no push, no network, no records branch writes. Real records are those in that snapshot.
- Consumers: none found for a config key; no config is changed, so athena.toml and defaults.toml are not deliverables.
- This does not add or tighten any refusal rule, so no refusal-rate criterion applies. Landing time is a records-computed measure: the existing landing figure may be touched, so a real-records check is included.
- Reasons are derived only from data already recorded; where a stage has no recorded reason the output says so.

## Acceptance criteria

1. [content] crates/athena-core/src/status/throughput.rs (or a new sibling module under crates/athena-core/src/status/) defines a per-stage breakdown covering reading, each planning round, producer, checks, verifier, landing, queue wait and disk wait, each with a duration and a reason field (model and effort, tokens, tool calls, build/test time, retries, waits), visible in the diff.
2. [content] crates/athena-cli/src/display.rs (or status.rs rendering) prints each stage's duration and its main reason, and prints an explicit 'not recorded' marker when a reason or wait is absent instead of a zero.
3. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records in the committed snapshot: the output lists per-stage durations with reasons for recent jobs, and the landing time it prints lies within the expected range taken from commit times in `git log` (run as its own simple command) for the same landings, i.e. a few minutes per landing, not 0.1 or hundreds of minutes; the range comes from git commit times, which the change does not compute.
4. [behavior] Run `cargo test -p athena-core` and `cargo test -p athena-cli`: new unit tests on fixture records show a slow planning round and a slow check each reported with their reason (e.g. tokens or retries; build versus test time), and tests cover a job missing a reason field.
5. [behavior] Athena's recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test), with no warnings.
6. [content] CHANGELOG.md has a new entry for per-stage timing and reasons, and docs/decisions.md gains a new appended decision with a Provenance bullet noting that time limits are dropped (24 h) and timing is measured instead; no existing decisions entry is edited.

## Expected deliverables

- crates/athena-core/src/status/throughput.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing time limits, athena.toml, config.rs or run/tests.rs (done by 20261114-184053-050s and kept as landed)
- Fixing the check_reading_size side effect or stale 1800 mentions in docs/design/agent-and-build-slots.md
- Adding new record types or changing what records store
- Prompt files
- records/ and owner files
- bash_default_timeout_ms and bash_max_timeout_ms

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the printed durations and reasons on real records against git commit times.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus recorded fmt, clippy and test checks.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Status output is run on real records, not only fixtures.
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

- path: <state>/jobs/20261114-194517-c16g/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-194517-c16g/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/status/throughput.rs (or a new sibling module under crates/athena-core/src/status/) defines a per-stage breakdown covering reading, each planning round, producer, checks, verifier, landing, queue wait and disk wait, each with a duration and a reason field (model and effort, tokens, tool calls, build/test time, retries, waits), visible in the diff.
crates/athena-cli/src/display.rs (or status.rs rendering) prints each stage's duration and its main reason, and prints an explicit 'not recorded' marker when a reason or wait is absent instead of a zero.
Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records in the committed snapshot: the output lists per-stage durations with reasons for recent jobs, and the landing time it prints lies within the expected range taken from commit times in `git log` (run as its own simple command) for the same landings, i.e. a few minutes per landing, not 0.1 or hundreds of minutes; the range comes from git commit times, which the change does not compute.
Run `cargo test -p athena-core` and `cargo test -p athena-cli`: new unit tests on fixture records show a slow planning round and a slow check each reported with their reason (e.g. tokens or retries; build versus test time), and tests cover a job missing a reason field.
Athena's recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test), with no warnings.
CHANGELOG.md has a new entry for per-stage timing and reasons, and docs/decisions.md gains a new appended decision with a Provenance bullet noting that time limits are dropped (24 h) and timing is measured instead; no existing decisions entry is edited.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
