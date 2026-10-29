# Job 20261029-084656-gn8r: Retry R5: move produce_and_verify S1-S3 into run/producer.rs

## Request (verbatim)

> try again on 20261026-232434-6wfq

## Reading (accepted by the user)

Retry of job 20261026-232434-6wfq: move the baseline, producer snapshot and send-back loop (S1-S3) of `produce_and_verify` into stage methods in new run/producer.rs, bodies unchanged, early returns becoming a Stop value. Audience: maintainers and the verifier. The earlier candidate's moved work is kept as the starting point, and its full test run is checked against the unchanged base.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs (produce_and_verify; baseline ui line ~1135, producer-loop early `return Ok(closing)` at ~1223, 1236, 1246, 1256, `producer   sent back:` ~1326, S4 idle-producer return ~1364), plus sibling modules run/send_back.rs, run/close.rs, run/pin.rs, run/planning.rs. Entry point is `produce_and_verify`; the order is baseline, snapshot, producer loop, then S4. No record or data file is added, so check_record_file/AnyRecord are not affected. The property (no behavior change) is measured by cargo test, the test list and a moved-text diff.
- Job 20261026-232434-6wfq: its candidate was rejected because `cargo test --locked` failed (591 tests ran, the failing test was not named in the stored log tail), plus three findings. Its four changed files are kept as deliverables and redone from the base. Findings: (a) design doc named `Stage::Stop` while code used `Flow::Stop` - fixed by making doc and code agree on one name (docs at structure-pass.md lines 125 and ~250 edited); (b) `&mut closing` became `closing` - fixed by keeping the moved text verbatim where possible, or listing the single change with its reason in the CHANGELOG; (c) `seq` by value rather than `&mut u32` - accepted as a nit if documented, or switched to match the design doc.
- The test failure is not assumed to be caused by the move or to pre-exist. The producer first runs `cargo test --locked` on the unchanged base and notes any failing tests by name, then reruns after the move. If the move caused a failure, it is fixed inside the move's scope. If a test already fails on the base, it is reported in the CHANGELOG/report and not fixed here.
- Prior jobs 20261026-232426-91cn (R2), 20261026-232431-dpem (R4) and 20261026-232420-ctt9 (R0) are landed and are not touched.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`. It passes in full, with no failing tests. If any test fails, the recorded output names it, and the CHANGELOG states whether it also fails at the base commit.
2. [behavior] Run `cargo test --locked -p athena-core -- --list`. The test names and count equal those of the base commit, and `produce_and_verify_order_tests::a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order` passes.
3. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both pass, and `git diff` shows no added `#[allow]`.
4. [content] `git diff --color-moved` shows the S1-S3 body text in crates/athena-core/src/run/producer.rs as the same text as the lines removed from run/mod.rs. Lines not marked moved are limited to mod/use/visibility lines, stage method signatures, the Stop/Reported types, local-to-field renames and rustfmt rewraps, each listed with its reason in the CHANGELOG entry.
5. [content] Each early `return Ok(closing)` in S1-S3 (the producer could not run, timed out, did not complete, returned no valid report) is paired in the CHANGELOG with the Stop that replaced it, with old and new line numbers. The driver in run/mod.rs returns `closing` when it sees Stop. The S4 idle-producer `return Ok(closing)` is unchanged.
6. [content] Grep over crates/athena-core/src/run shows the `baseline   ` ui line and the `producer   sent back:` line only in producer.rs, and one definition of each stage method. `produce_and_verify` calls them in the order baseline, snapshot, producer loop.
7. [content] The stop value is named the same in code and in docs/design/structure-pass.md (sections 3.3 and 5); Grep finds no `Stage::Stop` left if the code uses `Flow::Stop`, or the reverse.
8. [content] The CHANGELOG entry lists every local that crossed the S1-S3 seam and the field or parameter that carries it now. For each moved or new name it gives the Grep over crates/ with file counts before and after, and the counts are equal.
9. [content] `git diff --stat` lists only crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, CHANGELOG.md and docs/design/structure-pass.md. No test line changes except `use` lines. CHANGELOG.md has an R5 entry under Unreleased saying there is no behavior change, in the style of the R2 and R4 entries.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving S4 and later stages (candidate, admission, checks, verification, assembly); jobs R6-R8
- Any behavior change, or test edits beyond import lines
- Fixing a test that already fails on the base commit (report it only)
- Fixing the R2 design-doc contradiction about test modules, deduplicating R0 test helpers, or fixing the R0 comment
- Reverting or rewriting landed files run/send_back.rs, run/close.rs or the R0 test, or their CHANGELOG entries
- Editing athena.toml, defaults.toml, workflows, gh/ files, or callers in main.rs or drain.rs
- Rewriting produce_and_verify as a state machine
- Building the code-check that runs baseline tests at admission (the introspection's prevention proposal)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check test results, the test list and the moved-text diff, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move limited to S1-S3.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test failure is classified against the base before any fix.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry is written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

19 more qualifying classes are left out.

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

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked`. It passes in full, with no failing tests. If any test fails, the recorded output names it, and the CHANGELOG states whether it also fails at the base commit.
Run `cargo test --locked -p athena-core -- --list`. The test names and count equal those of the base commit, and `produce_and_verify_order_tests::a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order` passes.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both pass, and `git diff` shows no added `#[allow]`.
`git diff --color-moved` shows the S1-S3 body text in crates/athena-core/src/run/producer.rs as the same text as the lines removed from run/mod.rs. Lines not marked moved are limited to mod/use/visibility lines, stage method signatures, the Stop/Reported types, local-to-field renames and rustfmt rewraps, each listed with its reason in the CHANGELOG entry.
Each early `return Ok(closing)` in S1-S3 (the producer could not run, timed out, did not complete, returned no valid report) is paired in the CHANGELOG with the Stop that replaced it, with old and new line numbers. The driver in run/mod.rs returns `closing` when it sees Stop. The S4 idle-producer `return Ok(closing)` is unchanged.
Grep over crates/athena-core/src/run shows the `baseline   ` ui line and the `producer   sent back:` line only in producer.rs, and one definition of each stage method. `produce_and_verify` calls them in the order baseline, snapshot, producer loop.
The stop value is named the same in code and in docs/design/structure-pass.md (sections 3.3 and 5); Grep finds no `Stage::Stop` left if the code uses `Flow::Stop`, or the reverse.
The CHANGELOG entry lists every local that crossed the S1-S3 seam and the field or parameter that carries it now. For each moved or new name it gives the Grep over crates/ with file counts before and after, and the counts are equal.
`git diff --stat` lists only crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, CHANGELOG.md and docs/design/structure-pass.md. No test line changes except `use` lines. CHANGELOG.md has an R5 entry under Unreleased saying there is no behavior change, in the style of the R2 and R4 entries.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
