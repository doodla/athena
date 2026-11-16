# Job 20261104-013524-85wt: Make the drain continue past every item that stops on itself

## Request (verbatim)

> Make it the drain's default to continue past any item that stops on itself, whatever state the item ends in (paused by the scope guard, flagged after repeated failures, failed, held). The drain records each skipped item's state and reason in its summary and moves to the next runnable item while budget and slots remain. Keep one list in the code of every state that can stop an item, and add a test that fails when a new state is added without being handled there. Add tests that drive an item into each state and show the drain continues. Keep this as one change: no failure-cause sorting. The in-flight 'pass over an item that fails twice while flagged' job covers only one of these states.
> 
> Acceptance:
> - The change is made as described: Batch keeps stopping on failing items; each fix by hand covers one stop path (first too wide, then too narrow)
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-11/20261103-231453-wy5j.json

## Reading (accepted by the user)

The drain should pass over any item that ends in a self-stopping state (paused, flagged, awaiting diagnosis, failed, held, skipped, needs-accepting, waiting). It should record each item's state and reason in the summary and run the next runnable item while budget and slots remain. One code list names every item-ending state, a test fails when a state is added without being handled, and one test per state drives an item into it and shows the drain continuing.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs and crates/athena-cli/tests/drain.rs. The ItemEnd enum (~line 1423) lists Landed, Failed, Budget, JobLimit, Stop, Idle, NeedsAccepting, Held, Skipped, Waiting, Paused(reason), Flagged(reason) and Awaiting(reason). Drain::finish (~line 2684) maps each to pass_over, and take_until_done (~line 2783) loops until the queue is empty or the run has stopped. Entry point: the drain loop over Steps in drain.rs. The Stop enum (~line 1047) has an ALL-style list. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched.
- Goal and audience: the operator running `athena drain` for a batch, who today sees it stop on one failing item. The result is checked by the verifier and by drain tests. Existing tests at crates/athena-cli/tests/drain.rs lines ~787-1134 and 1650-1720 already cover paused, flagged and double-failure pass-over, so the new work extends them to every state.
- Term state: an item-ending state is a variant of ItemEnd that is not Landed and that ends one item without ending the run. Budget, JobLimit, Stop and Idle are run-ending and listed separately in the same list, as are the run-ending Stop reasons.
- Reading: the one list is a const or fn in drain.rs that returns, for each ItemEnd variant, whether it continues or ends the run. The exhaustiveness test uses a match with no wildcard arm, so a new variant fails to compile, plus a runtime check that every listed state has a test. The summary entry is the existing pass_over line, which carries the job and the reason. If the summary lacks the state name, add it.
- No failure-cause sorting: StopCause (ProducerAuth, LowDisk) stays as it is, so an environment cause still ends the run through Stop::Environment. That is the existing run-ending rule, not a new one. The in-flight flagged-twice job is not duplicated: its test stays, and the new tests cover the other states.
- Environment: none. Local cargo only, with the stub Steps in tests. No CI, push or records reads.
- Defaults applied: rust, tested, verified, behavior-evidence, small. The change is one commit with no refusal rule added, so no refusal-rate criterion applies.

## Acceptance criteria

1. [content] crates/athena-cli/src/drain.rs holds one list covering every ItemEnd variant, marking each as continues-the-batch or ends-the-run. The diff shows no wildcard arm in the match over it, and Drain::finish uses it or agrees with it.
2. [content] crates/athena-cli/tests/drain.rs has a test that fails to compile or fails at run time when an ItemEnd variant is missing from the list. The diff shows the exhaustive match.
3. [behavior] For each continuing state (paused, flagged, awaiting diagnosis, failed twice, held, needs-accepting, skipped, waiting), a test in crates/athena-cli/tests/drain.rs drives a first item into that state with the stub Steps. It asserts that the next runnable item ran and landed, and that the summary or ui output has one pass-over line with the job, the state and the reason. Verified by `cargo test`.
4. [behavior] The drain continues only while budget and slots remain: a test shows that after a skipped item, a budget or job-limit stop still ends the run with its Stop reason. Verified by `cargo test`.
5. [behavior] The new continue tests fail if the pass-over for that state is made to stop the run. The producer shows that the test asserts a later item landed, not just that a line was printed. Verified by reading the test diff and `cargo test`.
6. [content] No unrelated behavior changes. The diff leaves the StopCause environment stop (Stop::Environment) and the existing tests unchanged apart from additions. CHANGELOG.md has an entry for the change.
7. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree, as recorded by Athena.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Sorting failures by cause, or any change to StopCause handling
- Changes to athena.toml, defaults.toml or workflows
- Edits to records/ or to existing entries in docs/decisions.md
- The in-flight job that passes over an item that fails twice while flagged

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-state tests and an exhaustiveness test, with fmt, clippy and test as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the next item landed, not that wiring exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change in the drain and its tests.
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

- path: <state>/jobs/20261104-013524-85wt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-013524-85wt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-cli/src/drain.rs holds one list covering every ItemEnd variant, marking each as continues-the-batch or ends-the-run. The diff shows no wildcard arm in the match over it, and Drain::finish uses it or agrees with it.
crates/athena-cli/tests/drain.rs has a test that fails to compile or fails at run time when an ItemEnd variant is missing from the list. The diff shows the exhaustive match.
For each continuing state (paused, flagged, awaiting diagnosis, failed twice, held, needs-accepting, skipped, waiting), a test in crates/athena-cli/tests/drain.rs drives a first item into that state with the stub Steps. It asserts that the next runnable item ran and landed, and that the summary or ui output has one pass-over line with the job, the state and the reason. Verified by `cargo test`.
The drain continues only while budget and slots remain: a test shows that after a skipped item, a budget or job-limit stop still ends the run with its Stop reason. Verified by `cargo test`.
The new continue tests fail if the pass-over for that state is made to stop the run. The producer shows that the test asserts a later item landed, not just that a line was printed. Verified by reading the test diff and `cargo test`.
No unrelated behavior changes. The diff leaves the StopCause environment stop (Stop::Environment) and the existing tests unchanged apart from additions. CHANGELOG.md has an entry for the change.
`cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree, as recorded by Athena.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
