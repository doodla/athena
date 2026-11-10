# Job 20261028-112841-hx35: Keep partial work and retry once on producer 401 login expiry

## Request (verbatim)

> Priority: high
> Treat an expired or failed model login as an environment stop. On 2026-10-28 a producer ran 30 minutes and then failed with 'API Error: 401 OAuth access token has expired'; the login refreshed by itself minutes later, but the job was recorded as failed and its partial work was not kept, so the retry started from scratch. Fix: recognise a 401 or authentication failure from the agent driver as an environment cause (like low disk and usage limits), keep the producer's partial work the way a timed-out producer's work is kept, and have the retry resume from it; retry the driver call once after a short wait before giving up.
> Acceptance:
> - A test with a fake driver returning the 401 text shows the job ends as an environment stop with the partial work kept, and athena retry resumes from it.
> - A test shows one retried driver call succeeds after a transient 401.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-agent/src/claude.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, CHANGELOG.md

## Reading (accepted by the user)

A producer that fails with API Error 401 (expired login) should end as an environment stop whose partial work is kept as a candidate, as a timed-out producer's is. `athena retry` should resume from that candidate, and the driver call should be retried once after a short wait. The goal is that a transient login expiry no longer costs a 30-minute producer's work. The audience is the owner running `athena retry`.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs. `api_error_status`, `StopCause::of`, `producer_stop_reason`, `is_environment_reason` and `resumable_stop` already treat a 401 (PRODUCER_AUTH) and a 429 as environment stops, and `resumable_stop` lets `athena retry` resume a Failed job with an environment reason.
- Code survey: read crates/athena-core/src/run/producer.rs (around lines 364-391). A TimedOut outcome goes to `keep_timed_out_work`, which builds a candidate commit and a progress note. Any other non-Success outcome, including a 401, goes to `self.failure(producer_stop_reason(..))`, which keeps nothing. This is the gap the request names.
- Code survey: read crates/athena-core/src/run/mod.rs. `keep_timed_out_work` (line ~938), `timeout_seed` (~881, which requires the job state TimedOut), `resume_seed` and `resume_candidate` (~533, ~647, which read a resume mark), `write_resume_mark` (~834), and `TimedOutKeep` and `timeout_note` (~1393-1435). The change reuses `keep_timed_out_work` for an auth stop. The job closes Failed with the PRODUCER_AUTH reason, so `resumable_stop` still holds, and the retry finds the kept candidate through the existing seed path. The job state is not TimedOut, so the producer must either add a Failed-state branch to `timeout_seed` or write the resume mark. This is the main design choice.
- Code survey: read crates/athena-agent/src/claude.rs. The driver spawns the CLI through `retry_busy` and builds the detail in `output_from_result` (`<subtype>: <result text>`). A single retry after a short wait on a 401 result goes here, and the wait must be injectable or short so the test is fast. crates/athena-agent/src/fake.rs holds the fake driver the tests use. Existing tests are in crates/athena-core/src/run/tests.rs and crates/athena-cli/tests/retry.rs. The test helpers `EXPIRED_LOGIN_TEXT`, `result_line` and `driver_detail` in disk.rs hold the exact 401 text.
- Records and data: no new record or data file is added, so `check_record_file`, `AnyRecord` and the round-trip test are not affected.
- The 'once after a short wait' retry lives in the agent driver (claude.rs). A second 401 after the retry falls through to the environment stop with kept work.
- Not a refusal rule: nothing newly refuses readings. A 401 job is already classified as an environment stop, so the change only adds kept work.
- Environment: no CI workflow, push or PR is involved. Tests use a fake driver in the local workspace only.
- Consumers: none found. No config deliverable. athena.toml and defaults.toml stay unchanged.

## Acceptance criteria

1. [behavior] A new test using a fake driver that returns the 401 text (`Failed to authenticate. API Error: 401 ... OAuth access token has expired`) for every call, after the producer has edited files, ends the job Failed with a reason starting `environment: producer login expired`. The test asserts that the closing holds a candidate commit with the producer's edits. Run `cargo test -p athena-core` and check that this test passes.
2. [behavior] A new test shows `athena retry <job>` on that stopped job resumes from the kept candidate (the retry's seed or resume names the earlier job, and the earlier edits are present) instead of starting from scratch. Run `cargo test` and check that this test passes.
3. [behavior] A new test shows a driver that returns a 401 once and then succeeds yields a successful invocation: exactly two driver calls were made and the job does not end as an environment stop. Run `cargo test -p athena-agent` and check that this test passes.
4. [behavior] A test shows two consecutive 401 results lead to only one retry (two calls in total, no more) before the environment stop. Run `cargo test` and check that this test passes.
5. [content] The diff in crates/athena-agent/src/claude.rs adds the single retry after a short wait on a 401. crates/athena-core/src/run/mod.rs (with run/producer.rs if needed) keeps partial work for a 401 stop by reusing `keep_timed_out_work`. crates/athena-core/src/disk.rs changes only if the classification or resume rules need it. All three are listed in `git diff`.
6. [content] The existing timeout path and the 429 and low-disk paths keep their behavior: the existing tests in crates/athena-core/src/run/tests.rs and crates/athena-cli/tests/retry.rs still pass under `cargo test`.
7. [content] CHANGELOG.md has a new entry describing the 401 environment stop, the kept partial work with resume, and the single driver retry.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)

## Out of scope

- Handling other API errors (5xx, 529 overloaded) with retries
- Changing how 429 usage limits are handled
- Automatic retry or resume of the job without the user running `athena retry`, beyond the existing drain behavior
- Refreshing the login or reading credentials

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The criteria require new tests plus cargo test, clippy -D warnings and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the 401 stop, kept work, resume and retry by running tests, not by checking that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix reuses the timeout keep-work path and the central StopCause classification instead of adding a special case.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and doc comments are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261028-112841-hx35/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-112841-hx35/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test using a fake driver that returns the 401 text (`Failed to authenticate. API Error: 401 ... OAuth access token has expired`) for every call, after the producer has edited files, ends the job Failed with a reason starting `environment: producer login expired`. The test asserts that the closing holds a candidate commit with the producer's edits. Run `cargo test -p athena-core` and check that this test passes.
A new test shows `athena retry <job>` on that stopped job resumes from the kept candidate (the retry's seed or resume names the earlier job, and the earlier edits are present) instead of starting from scratch. Run `cargo test` and check that this test passes.
A new test shows a driver that returns a 401 once and then succeeds yields a successful invocation: exactly two driver calls were made and the job does not end as an environment stop. Run `cargo test -p athena-agent` and check that this test passes.
A test shows two consecutive 401 results lead to only one retry (two calls in total, no more) before the environment stop. Run `cargo test` and check that this test passes.
The diff in crates/athena-agent/src/claude.rs adds the single retry after a short wait on a 401. crates/athena-core/src/run/mod.rs (with run/producer.rs if needed) keeps partial work for a 401 stop by reusing `keep_timed_out_work`. crates/athena-core/src/disk.rs changes only if the classification or resume rules need it. All three are listed in `git diff`.
The existing timeout path and the 429 and low-disk paths keep their behavior: the existing tests in crates/athena-core/src/run/tests.rs and crates/athena-cli/tests/retry.rs still pass under `cargo test`.
CHANGELOG.md has a new entry describing the 401 environment stop, the kept partial work with resume, and the single driver retry.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
