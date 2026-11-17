# Job 20261104-062100-0mm4: Add --queue to approve-work and retry so they return at once

## Request (verbatim)

> Stop a caller's command time limit from killing a job partway through. Today `athena approve-work --yes` and job retries run the whole job in the foreground (see the ApproveWork arm in crates/athena-cli/src/main.rs). Twice, the shell's time limit killed one mid-planning and left it interrupted (interventions 20261026-104730-j4g8 and 20261102-005305-rm74). Add a way to start these commands that returns at once, such as a flag that queues the job for the drain or runs it detached and tracked, and print where to follow its progress. Document it in the CLI help. Acceptance: (1) a test shows the queue-only path returns before any stage runs and leaves the job queued, not interrupted; (2) the drain then picks up and runs the queued job; (3) the help text for approve-work and retry names the time-limit risk and the non-blocking option.
> 
> Acceptance:
> - The change is made as described: Jobs run in the foreground under a caller's command time limit get killed mid-planning and left interrupted
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/src/main.rs).

## Reading (accepted by the user)

Add a `--queue` flag to `athena approve-work` and `athena retry`. The command records the job and leaves it queued for `athena drain`, prints the job id and the follow-up commands, and returns without running any stage. Both commands' help names the command time-limit risk and the option. The audience is operators who run these commands under a caller's time limit.

Assumptions:
- Code survey: read crates/athena-cli/src/main.rs (Command::Retry at line 52, Command::ApproveWork at line 130, show_proposal at line 1014, run dispatch at lines 1057-1079, go_and_report), crates/athena-cli/src/commands.rs (ask --later calls athena.queue and prints display::queued; retry calls athena.retry then show_proposal), crates/athena-core/src/steward.rs (queued_work takes jobs in JobState::Queued or Proposed, so the drain already runs both), crates/athena-core/src/store.rs (JobState). No record or data file is added, so no records validation change is needed (no AnyRecord or check_record_file work).
- Existing code: the change reuses crates/athena-cli/src/main.rs show_proposal and the drain's queued_work in crates/athena-core/src/steward.rs, plus display::queued in crates/athena-cli/src/display.rs. The new flag extends the `--later` behavior of `ask`.
- Assumption: `--queue` still builds the reading as today (the reading step runs, as `ask --later` is the no-model path but approve-work and retry need a proposal). It then stops before the run (go_and_report), so the job stays Proposed, which the drain treats as queued, and nothing is marked interrupted. The flag conflicts with --yes. The output names the job id plus `athena drain` and `athena status` to follow progress.
- Field meaning: no existing field is reused; `queue` is a new clap flag on Retry and ApproveWork. The producer may name it `--later` for consistency with `ask --later` (doc at main.rs:40, 'Only record the request, to be read and run later'); the criteria then apply to that name.
- Environment: none; no CI, push or records-branch access is needed. athena.toml, defaults.toml and workflows are unchanged because the flag needs no config. Consumers: none found.
- Detaching a background process is out of scope; the queue-only path is the chosen option.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli (run with `cargo test`) drives the queue-only path of retry or approve-work against a test repository. It asserts the command returns before any stage runs (no dispatch or stage record is written) and that the job's state is Queued or Proposed, not Interrupted. It fails if the flag falls through to go_and_report.
2. [behavior] A test run with `cargo test` shows that after a job is left by the queue-only path, steward::queued_work lists it, and a drain run on it dispatches and runs it. The test fails if the job is not listed.
3. [content] In crates/athena-cli/src/main.rs the doc comments of Command::Retry and Command::ApproveWork each state that a caller's command time limit can kill the foreground run mid-job and leave it interrupted, and name the non-blocking option (--queue) and `athena drain`. The `--queue` flag has its own doc comment, and the command's output prints the job id and how to follow progress (`athena status`, `athena drain`).
4. [content] `--queue` conflicts with `--yes` through clap (conflicts_with), and the `--json` output for the queued path includes the job id and state. The diff shows no behavior change when the flag is absent.
5. [behavior] `cargo test`, `cargo clippy` and `cargo fmt --check` pass on the final tree, with the existing tests unchanged. This can be shown by the check records Athena made on that tree.
6. [content] CHANGELOG.md records the change; no file other than the deliverables and CHANGELOG.md changes.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Running the job detached as a tracked background process
- Changing how `athena drain` or `athena go` run jobs
- Changes to athena.toml, defaults.toml or workflows
- Recovering the two already-interrupted interventions

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Added tests for the queue-only path and the drain pickup; fmt, clippy and test are required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the job's state and that no stage ran, not that the flag exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: a single flag on two commands, reusing existing queue and drain code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The help text is written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

- path: <state>/jobs/20261104-062100-0mm4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-062100-0mm4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-cli (run with `cargo test`) drives the queue-only path of retry or approve-work against a test repository. It asserts the command returns before any stage runs (no dispatch or stage record is written) and that the job's state is Queued or Proposed, not Interrupted. It fails if the flag falls through to go_and_report.
A test run with `cargo test` shows that after a job is left by the queue-only path, steward::queued_work lists it, and a drain run on it dispatches and runs it. The test fails if the job is not listed.
In crates/athena-cli/src/main.rs the doc comments of Command::Retry and Command::ApproveWork each state that a caller's command time limit can kill the foreground run mid-job and leave it interrupted, and name the non-blocking option (--queue) and `athena drain`. The `--queue` flag has its own doc comment, and the command's output prints the job id and how to follow progress (`athena status`, `athena drain`).
`--queue` conflicts with `--yes` through clap (conflicts_with), and the `--json` output for the queued path includes the job id and state. The diff shows no behavior change when the flag is absent.
`cargo test`, `cargo clippy` and `cargo fmt --check` pass on the final tree, with the existing tests unchanged. This can be shown by the check records Athena made on that tree.
CHANGELOG.md records the change; no file other than the deliverables and CHANGELOG.md changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
