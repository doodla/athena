# Job 20261105-180653-398t: Detect and finish landed jobs whose PR never merged to main

## Request (verbatim)

> Find out why the pull requests for jobs 20261105-001940-nvyd and 20261104-124034-ysc4 (GitHub PRs 399 and 397) were never merged to main after Athena recorded both jobs as landed. Each has one failed pull_request CI run and no later run, and other jobs' PRs merged after them. Then make Athena detect this: athena status and the drain should list any landed job whose PR is open and whose landing commit is not on main, and should re-trigger or finish the merge. Add a test for a landed job whose PR merge never happened. Get the two missing changes onto main.
> 
> Acceptance:
> - The change is made as described: Jobs recorded as landed on the person running Athena/next5 whose pull requests stayed open with failed CI, so their changes never reached main
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/run/close.rs

## Reading (accepted by the user)

Jobs 20261105-001940-nvyd (PR 399) and 20261104-124034-ysc4 (PR 397) were recorded landed while their PRs stayed open after one failed pull_request CI run. I will add detection to `athena status` and the drain, which list each landed job whose PR is open and whose landing commit is not on main, then re-trigger or finish the merge. A test covers a landed job whose merge never happened. The audience is the operator running Athena.

Assumptions:
- Environment: the producer and verifier run in a sandbox copy with no network, no GitHub credentials and no push rights, so they run only cargo and read git. Athena runs on the owner's machine through the `gh` client in crates/athena-core/src/gh/client.rs, with the owner's repository token. Reads: PR state, check runs and main's first-parent commits. Writes: re-running a failed pull_request workflow, enabling or performing the merge of the open PR, and landing records. It cannot reach other repositories. With no origin or a non-GitHub origin the detector reports 'cannot check: no GitHub origin', changes nothing and does not fail status or drain. With a GitHub origin it checks and acts. Tests use the fake hosts in crates/athena-core/src/test_support.rs, not the network.
- Code survey: I read crates/athena-core/src/run/close.rs (landing paths, `landed` outcome and landed_commit at lines ~330-600), crates/athena-core/src/reconcile.rs (reconcile walk, `status_with_reconcile`, CommitHost, Finding), and grepped crates/athena-core/src/gh/client.rs (PrHost: open_pr, merge, merged_commit), crates/athena-core/src/status.rs (StatusView, landed_job_ids) and test_support.rs (fake hosts). Entry points: `athena status` goes through status_with_reconcile; the drain is crates/athena-cli/src/drain.rs. Detection must run after reconcile and read the host PR state directly, never through a stand-in such as the job's end state. No new record or data file under crates/ is planned, so the AnyRecord/check_record_file round-trip is not touched; if one is needed it must be covered by that test.
- Root cause to establish first (read-only investigation from the code and recorded PR data): why the merge did not follow the failed CI run. Likely the landing is recorded as soon as the PR is opened or auto-merge is requested, and a failed CI run with no later run leaves the PR open with nothing that re-triggers it. The producer records the finding in CHANGELOG.md and the code docs. Only the owner can run `gh` against PRs 397 and 399.
- Getting the two changes onto main: this candidate cannot push. The new finish-the-merge path is how PRs 397 and 399 reach main when the owner next runs `athena status` or `athena drain`. The two jobs' own diffs are not re-implemented here; they stay on their PRs.
- Previous results: job 20261105-001940-nvyd findings (CHANGELOG hedge about the 42 requests, no multi-item batch test, held requests less visible in queue listings) are not addressed because they concern the proposed-request selection, not the landing. Job 20261104-124034-ysc4 findings (extra README.md and job_plan.rs changes; stale spec.toml text on OLY-8-09; changed `--json` shape) are likewise not addressed. All their changed files are excluded in out_of_scope as work to be kept as landed on their own PRs.
- The request names crates/athena-core/src/run/close.rs as the file. The detection may need reconcile.rs, status.rs, gh/client.rs and drain.rs, listed with lower probability.

## Acceptance criteria

1. [content] A new function in crates/athena-core (reconcile.rs or close.rs) finds landed jobs whose PR is open and whose landing commit is not on main's first-parent history. Check the diff.
2. [content] The status view and `athena status` output list each such job with its id and PR number. The drain also lists them. Check the diff in status.rs and crates/athena-cli/src/drain.rs, and the CHANGELOG.md entry that gives the root cause.
3. [behavior] A test uses the fake host to build a landed job whose PR is open with one failed pull_request run and a landing commit not on main. It asserts the job is listed, and that the re-trigger or merge is called once. A second run on a merged PR does nothing. Run `cargo test`. The test must fail when the detection is removed.
4. [behavior] A test covers no origin or a non-GitHub origin: it reports 'cannot check', writes nothing and does not fail status or the drain. Run `cargo test`.
5. [behavior] A drain test shows an unmerged landed job does not stop the batch, and the next queued item still runs. Run `cargo test`.
6. [content] No unrelated behavior changes. The diff touches only the landing, status and drain code, their tests and CHANGELOG.md, and no file under athena.toml, defaults.toml or .github/workflows/.
7. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`. All pass, and the Athena-recorded passing results on the final tree count as evidence.

## Expected deliverables

- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Re-implementing or editing the two jobs' own changes, which stay on their PRs 397 and 399: crates/athena-cli/src/drain.rs behavior for item failures (job ysc4), crates/athena-core/src/steward.rs (job nvyd), crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, README.md, and crates/athena-cli/tests/job_plan.rs. Any edit to drain.rs or tests/drain.rs here is limited to the unmerged-landing listing.
- Fixing the findings of jobs 20261105-001940-nvyd and 20261104-124034-ysc4, including docs/spec.toml OLY-8-09.
- Pushing, merging or running gh from the producer or verifier.
- Changes to athena.toml, defaults.toml, rust-toolchain.toml or .github/workflows/ (CI configuration).
- Editing records/ or docs/decisions.md.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The root cause of the unmerged PRs is investigated and fixed generally, not just for these two PRs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail without the detection.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty unmerged list is reported as 'none found' and a check that could not run is reported separately.
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

- path: <state>/jobs/20261105-180653-398t/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-180653-398t/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 2: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 3: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 4: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 5: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 6: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 7: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new function in crates/athena-core (reconcile.rs or close.rs) finds landed jobs whose PR is open and whose landing commit is not on main's first-parent history. Check the diff.
The status view and `athena status` output list each such job with its id and PR number. The drain also lists them. Check the diff in status.rs and crates/athena-cli/src/drain.rs, and the CHANGELOG.md entry that gives the root cause.
A test uses the fake host to build a landed job whose PR is open with one failed pull_request run and a landing commit not on main. It asserts the job is listed, and that the re-trigger or merge is called once. A second run on a merged PR does nothing. Run `cargo test`. The test must fail when the detection is removed.
A test covers no origin or a non-GitHub origin: it reports 'cannot check', writes nothing and does not fail status or the drain. Run `cargo test`.
A drain test shows an unmerged landed job does not stop the batch, and the next queued item still runs. Run `cargo test`.
No unrelated behavior changes. The diff touches only the landing, status and drain code, their tests and CHANGELOG.md, and no file under athena.toml, defaults.toml or .github/workflows/.
Run `cargo fmt --check`, `cargo clippy` and `cargo test`. All pass, and the Athena-recorded passing results on the final tree count as evidence.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
