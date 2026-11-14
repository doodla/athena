# Job 20261112-005306-pa2j: Fix flaky drain test that leaves CI shard 3 unfinished

## Request (verbatim)

> Diagnose and fix the test parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn in crates/athena-cli/tests/drain.rs. In jobs 20261029-204942-2zqw and 20261111-213010-8jgz, its CI test shard (xtask test-shard 3 of 6) was recorded as 'did not finish' and the job was rejected. Neither job changed drain code, and both verifiers reran the test alone and saw it pass. Job 20261029-233047-ab43 also saw it fail in a full parallel run and pass alone. Steps: (1) Run the test alone many times, and again under parallel load, as in the full suite. Measure how long it takes. Find the cause, for example the drain_concurrent barrier waiting out its deadline with a bound of 1, or a wait that depends on timing. State the cause from those measured runs. (2) Fix that cause in the test or in its helpers, so the test passes in 20 repeated runs under load and finishes well within the shard limit. Do not just raise a timeout or skip the test. (3) Add a CHANGELOG.md entry. Run cargo fmt --check, cargo clippy --all-targets -D warnings and the full cargo test.
> 
> Acceptance:
> - The change is made as described: One drain test, a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn, leaves its CI shard unfinished and rejects jobs that never touch the drain
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/tests/drain.rs., CHANGELOG.md

## Reading (accepted by the user)

Retries/follows up jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43: measure parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn alone and under load, find the timing-dependent cause, and fix it in crates/athena-cli/tests/drain.rs (or its helpers) without raising timeouts or skipping. Add a CHANGELOG.md entry. Audience: CI and verifiers who reject jobs on this flake.

Assumptions:
- Code survey: read crates/athena-cli/tests/drain.rs mod parallel (line 8205 on): Shared stub implements Steps; run() logs start/end, and has 5-second spin barriers for `meet` and `wait_for`; the target test (line 8523) uses neither, with run_ms 0, calling drain::drain_concurrent with bounds 3, 1 and 3. The entry point is drain_concurrent; the cause must be found by running the test, not assumed. The records validation allow-list and record files are not touched.
- Cause is unproven: I could not run anything. The producer must measure alone and under load (e.g. cargo test -p athena-cli --test drain repeated, plus parallel load) and state the cause from the timings. Likely candidates: the drain worker threads under CPU load, a deadline wait in the shared stub, or an order assumption that depends on thread scheduling, in assertions like started[..3] contains s1 or the run <= 2 streak.
- Job 20261029-204942-2zqw: its candidate changes (docs/spec.toml, briefs.rs, planner.rs, CHANGELOG.md for OLY-8-05) are dropped, not kept; its only finding that applies here is the flaky drain test, which this job fixes. Its other findings concern a different change and no longer apply.
- Job 20261111-213010-8jgz: its candidate changes (introspection.rs, status.rs, display.rs, status_build_measurement.rs, docs/spec.toml, CHANGELOG.md) are dropped; its flake finding is fixed here, and its other findings do not apply to this job.
- Job 20261029-233047-ab43: landed, so its work (learning.rs, lib.rs, status.rs, display.rs, docs/spec.toml, CHANGELOG.md) is already in the tree and not touched. Only its observed flake (failed in a full parallel run, passed alone) is addressed.
- Environment: the CI shard is run by xtask test-shard 3 of 6 on GitHub-hosted runners; this job only runs local cargo commands and cannot read CI or push. Shard time limit is unknown to me; the producer should record the measured time of the test.

## Acceptance criteria

1. [behavior] Running `cargo test -p athena-cli --test drain a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn` repeatedly (20 times) passes every time, including while the rest of the drain test binary runs in parallel, and each run finishes in seconds (well under the shard limit).
2. [content] The diff in crates/athena-cli/tests/drain.rs removes the cause of the flake (a timing-dependent wait or scheduling-dependent assertion) and does not merely raise a timeout, add #[ignore], or delete the test; its assertions on start order (s1 among first three, at most two non-Steward starts in a row, 8 starts, improvement_turns equal to s1, s2) are kept.
3. [content] CHANGELOG.md has a new entry naming the test and the measured cause, and mentioning jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (recorded Athena checks, D115).
5. [content] No file under crates/ other than crates/athena-cli/tests/drain.rs changes, and no drain production code behavior changes, so the other tests are unaffected.

## Expected deliverables

- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing drain production code in crates/athena-core or elsewhere unless the measured cause is there
- Changes to athena.toml, defaults.toml, xtask or CI workflows, including the shard layout or time limits
- docs/spec.toml, OLY-8-05, OLY-10-03 and OLY-9-09 work from the earlier jobs; these candidates are dropped
- Changing other drain tests except shared helpers needed for the fix

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and full cargo test criteria
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the measured cause, not raise a timeout or skip
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require repeated runs under load
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to one test file plus CHANGELOG
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

35 more qualifying classes are left out.

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

- path: <state>/jobs/20261112-005306-pa2j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-005306-pa2j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Running `cargo test -p athena-cli --test drain a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn` repeatedly (20 times) passes every time, including while the rest of the drain test binary runs in parallel, and each run finishes in seconds (well under the shard limit).
The diff in crates/athena-cli/tests/drain.rs removes the cause of the flake (a timing-dependent wait or scheduling-dependent assertion) and does not merely raise a timeout, add #[ignore], or delete the test; its assertions on start order (s1 among first three, at most two non-Steward starts in a row, 8 starts, improvement_turns equal to s1, s2) are kept.
CHANGELOG.md has a new entry naming the test and the measured cause, and mentioning jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (recorded Athena checks, D115).
No file under crates/ other than crates/athena-cli/tests/drain.rs changes, and no drain production code behavior changes, so the other tests are unaffected.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
