# Job 20261114-054109-erbs: Fix flaky urgent-group drain test without weakening its assertions

## Request (verbatim)

> try again on 20261112-005306-pa2j

## Reading (accepted by the user)

Retry of job 20261112-005306-pa2j (rejected): reproduce the flake of parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn under load, state its cause from the code, and fix it in drain.rs tests so the test can still fail on a wrong drain order. Audience: CI and verifiers rejecting jobs on this flake. Also follows up 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43.

Assumptions:
- Code survey: read crates/athena-cli/tests/drain.rs (the test and its stub), crates/athena-cli/src/drain.rs (run_batches at 2855 starts workers over one VecDeque; take_until_done at 2770 pops an item and runs it, logging start, before popping again) and crates/athena-core/src/plan.rs (groups_in_order at 583 gives one group u1..u6,s1,s2 for disjoint items). Consequence: with 3 workers, s1 is popped only after six u's, so s1 can appear at log index 4 or 5 but never in the first four starts; the base flake is log-order skew between popping and logging. No records or data files are added, so check_record_file/AnyRecord do not apply.
- Previous job 20261112-005306-pa2j: finding 1 (stub made to log starts in expected order, so the assertion tested the stub) is addressed by dropping that stub gate and instead asserting on the drain's deterministic hand-out order (report.plan_groups.concat() == u1..u6,s1,s2) plus a race-free log check (s1 start index >= 4 with 3 workers). Finding 2 (changelog failure example impossible) is addressed by rewriting the entry with a failure the drain and stub can produce (s1 at index 4 or 5 of the log, or whatever the new measurement shows), taken from a fresh reproduction. Finding 3 (CI 'did not finish' unexplained) is addressed by the entry saying plainly that a hang was not reproduced and the open question remains. Its candidate's drain.rs and CHANGELOG.md changes are dropped and redone from base.
- Producer must first reproduce the flake on the base tree by running the test repeatedly in parallel under load, measure runtimes, and state the cause from that output. The original first assertion (s1 among the first three starts) is racy only if it is the failing one; if the measured cause differs, the producer keeps the intent (s1 not ahead of the improvement turn) in a form the drain can fail.
- Mutation check: the producer deliberately breaks the drain's order (temporarily, not committed) or the test's input and shows the repaired assertions fail; this is reported but only the committed diff counts.
- Governance: no change to athena.toml, defaults.toml, xtask or CI workflows; shard layout and limits untouched.

## Acceptance criteria

1. [behavior] Running `cargo test -p athena-cli --test drain a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn` passes, and the full `cargo test --locked` (which runs the drain tests in parallel) passes, each in seconds for this test.
2. [content] The diff in crates/athena-cli/tests/drain.rs does not make the stub log starts in an expected order (no gate that holds a start entry until earlier entries exist), does not raise a timeout, add #[ignore] or delete the test.
3. [content] The test in crates/athena-cli/tests/drain.rs still asserts 8 starts, at most two non-Steward starts in a row, improvement_turns equal to s1, s2, and an order check that fails if s1 is misplaced: either report.plan_groups.concat() equals u1..u6,s1,s2, or s1's start index is at least 4 with 3 workers (verifiable by reading the assertions against take_until_done in crates/athena-cli/src/drain.rs).
4. [content] CHANGELOG.md has a new Fixed entry naming the test, a cause stated in terms of drain.rs worker pop-then-log behavior, a failure example the drain and stub can produce (s1 not in the first four log entries is impossible; example shows s1 at index 4 or 5 versus the old assertion), the jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43, and a plain statement that a never-finishing run was not reproduced.
5. [content] No file other than crates/athena-cli/tests/drain.rs and CHANGELOG.md changes; no production code under crates/ changes behavior (checked with git diff).
6. [behavior] Recorded Athena checks `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (D115).

## Expected deliverables

- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing drain production code in crates/athena-core or crates/athena-cli/src unless the measured cause is there
- Changes to athena.toml, defaults.toml, xtask or CI workflows, including shard layout or time limits
- docs/spec.toml, OLY-8-05, OLY-10-03 and OLY-9-09 work from earlier jobs; those candidates are dropped
- Changing other drain tests except shared helpers needed for the fix
- Adding a flaky-test step to the Athena job flow (the introspection's prevention); not part of this fix

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix the measured cause, not the symptom
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria check that start order is still verified by the drain
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: behavior criteria name cargo commands the verifier runs
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261112-005306-pa2j

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> The diff in crates/athena-cli/tests/drain.rs removes the cause of the flake (a timing-dependent wait or scheduling-dependent assertion) and does not merely raise a timeout, add #[ignore], or delete the test; its assertions on start order (s1 among first three, at most two non-Steward starts in a row, 8 starts, improvement_turns equal to s1, s2) are kept.: The fix adds no timeout and no #[ignore], deletes nothing, and keeps the assertions word for word. But it does not make the scheduling-dependent assertion sound. It makes the stub write `start` entries in the order the test expects. In `Shared::run`, a job at index k of `start_order` spins until any k `start ` entries are logged; the gate counts entries and does not check which jobs they are. In the plain case (bound 3, all eight items in one plan group, per groups_in_order in plan.rs:583), suppose the drain dispatched s1 first. s1 would hold one worker until six other starts are logged, and the other two workers would log u1..u6. So `first_six == [u1..u6]` would still pass. That assertion can no longer catch the regression it exists to catch: the log order now comes from the stub, not from the drain.

## Findings you must address

The verifier of job 20261112-005306-pa2j raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-cli/tests/drain.rs:8311-8326 and 8546): The fix makes the stub log starts in the expected order, so the first assertion can no longer fail when the drain's order is wrong. The gate holds the job at index k of `start_order` until any k `start ` entries exist; it counts entries and does not check which jobs they are. With bound 3, a misplaced s1 would hold one worker while the other two logged u1..u6, and `first_six == [u1..u6]` would still pass. The test now checks the stub, not the drain. Sounder options: (a) assert on the order the drain itself decides, which is deterministic: `report.plan_groups.concat() == [u1..u6, s1, s2]`; or (b) keep a log check that the worker model makes race-free. A worker logs its item's start before it takes the next item, so with 3 workers s1 can never be among the first 4 starts. Asserting that s1's start index is at least 4 would still catch a misplaced s1.
> major (CHANGELOG.md (new Fixed entry)): The CHANGELOG's failure example cannot happen with this drain and stub, which casts doubt on the measurement behind the stated cause. The entry gives `["s1", "u1", "u3", "u4", "u5", "u6"]` as a measured failure. All eight items are disjoint, so the plan makes one group ordered u1..u6, s1, s2 (groups_in_order, plan.rs:583). With read_ahead 0, run_batches starts 3 workers on one VecDeque. Each worker in take_until_done (drain.rs:2770) pops an item and runs it (which logs `start`) before it pops again. So s1 is popped only after six u's were popped; at most two of those are still held by the other workers, and at least four u's have already logged a start. s1 cannot be first. The real race in the base gives s1 at index 4 or 5. The 24-thread harness that produced the 7200-run numbers likely did not exercise the test as written, for example by sharing state across threads. So the 2.8% / 0-failure numbers are not reliable evidence.

All of the verification's findings:

> major (crates/athena-cli/tests/drain.rs:8311-8326 and 8546): The fix makes the stub log starts in the expected order, so the first assertion can no longer fail when the drain's order is wrong. The gate holds the job at index k of `start_order` until any k `start ` entries exist; it counts entries and does not check which jobs they are. With bound 3, a misplaced s1 would hold one worker while the other two logged u1..u6, and `first_six == [u1..u6]` would still pass. The test now checks the stub, not the drain. Sounder options: (a) assert on the order the drain itself decides, which is deterministic: `report.plan_groups.concat() == [u1..u6, s1, s2]`; or (b) keep a log check that the worker model makes race-free. A worker logs its item's start before it takes the next item, so with 3 workers s1 can never be among the first 4 starts. Asserting that s1's start index is at least 4 would still catch a misplaced s1.
> major (CHANGELOG.md (new Fixed entry)): The CHANGELOG's failure example cannot happen with this drain and stub, which casts doubt on the measurement behind the stated cause. The entry gives `["s1", "u1", "u3", "u4", "u5", "u6"]` as a measured failure. All eight items are disjoint, so the plan makes one group ordered u1..u6, s1, s2 (groups_in_order, plan.rs:583). With read_ahead 0, run_batches starts 3 workers on one VecDeque. Each worker in take_until_done (drain.rs:2770) pops an item and runs it (which logs `start`) before it pops again. So s1 is popped only after six u's were popped; at most two of those are still held by the other workers, and at least four u's have already logged a start. s1 cannot be first. The real race in the base gives s1 at index 4 or 5. The 24-thread harness that produced the 7200-run numbers likely did not exercise the test as written, for example by sharing state across threads. So the 2.8% / 0-failure numbers are not reliable evidence.
> minor (CHANGELOG.md): The CI failure mode named in the request, a shard that did not finish, is still unexplained. The entry itself says it 'did not reproduce a run that never finished'. A failing assertion reports as failed, not as unfinished, so the job that prompted this request may still be open. The entry says so plainly; it should be followed up rather than treated as closed.

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

The work starts from the rejected candidate of job 20261112-005306-pa2j: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-054109-erbs/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-054109-erbs/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Running `cargo test -p athena-cli --test drain a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn` passes, and the full `cargo test --locked` (which runs the drain tests in parallel) passes, each in seconds for this test.
The diff in crates/athena-cli/tests/drain.rs does not make the stub log starts in an expected order (no gate that holds a start entry until earlier entries exist), does not raise a timeout, add #[ignore] or delete the test.
The test in crates/athena-cli/tests/drain.rs still asserts 8 starts, at most two non-Steward starts in a row, improvement_turns equal to s1, s2, and an order check that fails if s1 is misplaced: either report.plan_groups.concat() equals u1..u6,s1,s2, or s1's start index is at least 4 with 3 workers (verifiable by reading the assertions against take_until_done in crates/athena-cli/src/drain.rs).
CHANGELOG.md has a new Fixed entry naming the test, a cause stated in terms of drain.rs worker pop-then-log behavior, a failure example the drain and stub can produce (s1 not in the first four log entries is impossible; example shows s1 at index 4 or 5 versus the old assertion), the jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43, and a plain statement that a never-finishing run was not reproduced.
No file other than crates/athena-cli/tests/drain.rs and CHANGELOG.md changes; no production code under crates/ changes behavior (checked with git diff).
Recorded Athena checks `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (D115).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
