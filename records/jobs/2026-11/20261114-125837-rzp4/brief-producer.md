# Job 20261114-125837-rzp4: Retry 20261114-045625-aymy: run checks before verifier, send failures

## Request (verbatim)

> try again on 20261114-045625-aymy

## Reading (accepted by the user)

Retry of job 20261114-045625-aymy, which died in planning at the 600s deadline with no code written. Same fix, planned as one narrow change: before the verifier, Athena runs the configured checks on the producer's tree, and a failure goes into the existing single send-back brief, so the producer sees it before the verdict.

Assumptions:
- Goal and audience: the goal is that a producer sees a failing recorded check (fmt, clippy, test) before the verifier does, so one repair pass replaces a whole-job rejection. The audience is Athena maintainers and the verifier. The typed request 'try again on 20261114-045625-aymy' takes precedence over the introspection's proposed planner-deadline request, which is a separate job and out of scope.
- Previous job 20261114-045625-aymy: it failed in planning (planner deadline of 600s) with 5 outcome reasons and no candidate, so no files are kept and all its deliverables are re-planned from the base. Its plan-critique findings are addressed as follows: (1) athena-cli tests whose test check is `test -f README.md && ...` and that delete README.md (retry.rs, introspection.rs, steward.rs, code_health_hooks.rs, lifecycle.rs:588) get one extra producer pass; the producer must run the full `cargo test -p athena-cli` and fix any assertion on producer counts or report order that breaks, without weakening it. (2) The in-loop check run sets GIT_INDEX_FILE to a temporary index built with `git add -A`, as Snapshot::work_tree_tree does, so `git ls-files` sees new files. (3) The producer.md change is net-zero bytes, and the producer gives the exact final text of each changed line.
- To keep the plan inside the 600s planner deadline the change is deliberately narrow: no new loop or limit, one in-loop check run reusing the existing check runner, one new appended decision, no edits to reader or verifier prompts.
- Code survey: read crates/athena-core/src/run/send_back.rs (send_back_text, send_back_text_scoped, unmet criteria from criteria_check at lines 11-50 and 92-166), crates/athena-core/src/run/producer.rs (send-back loop near lines 292, 573 and 614), crates/athena-core/src/run/mod.rs:1591 (PRODUCER_SEND_BACKS = 1) and crates/athena-core/src/run/tests.rs (fake producer tests). The entry point is the producer loop in producer.rs. The checks currently run in crates/athena-core/src/run/checks.rs only at verification. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by driving the real producer loop with a fake producer, not through a stand-in.
- Field meaning: no new field, CLI argument, record key or config key is added. The stored invocation detail reuses the existing send-back problem text.
- Existing code: the change extends crates/athena-core/src/run/checks.rs (run_check, which clears the environment and sets ctx.env), crates/athena-core/src/run/send_back.rs and crates/athena-context/src/briefs.rs, and follows docs/decisions.md D115 (recorded checks on the same tree count as evidence).
- No refusal rule is added; the change only adds information to the existing send-back. Expected 0% of newly refused readings.
- Environment: none; local cargo only, with no CI workflow, push or records access.

## Acceptance criteria

1. [content] crates/athena-core/src/run/producer.rs runs the configured checks (via the existing runner in crates/athena-core/src/run/checks.rs) on the producer's tree before the verifier, with GIT_INDEX_FILE set to a temporary index of the work tree. A failure's command, exit status and output tail are added to the existing send-back problems and rendered by send_back_text in crates/athena-core/src/run/send_back.rs. The stored invocation detail names the check failure.
2. [content] The send-back stays bounded by PRODUCER_SEND_BACKS in crates/athena-core/src/run/mod.rs (diff shows no new loop and no changed limit). A check still failing after the send-back is passed to the verifier as flagged, not hidden.
3. [behavior] A new test in crates/athena-core/src/run/tests.rs drives the real producer loop with a fake producer whose tree fails a configured check on the first pass and passes after the send-back. Run `cargo test -p athena-core`: it passes, and the brief of the second producer call contains the failed command and its output tail. On the base tree it fails because no send-back happens.
4. [behavior] A second test in crates/athena-core/src/run/tests.rs shows that when all checks pass on the first pass there is exactly one producer call and no send-back, and a test shows that a file added by the producer and listed only by `git ls-files` is seen by the in-loop check. Run `cargo test -p athena-core`.
5. [behavior] The athena-cli tests that delete README.md and expect Rejected (retry.rs, introspection.rs, steward.rs, code_health_hooks.rs, lifecycle.rs:588) still pass with the extra producer pass. Run `cargo test -p athena-cli`.
6. [content] crates/athena-context/prompts/producer.md and the producer brief in crates/athena-context/src/briefs.rs tell the producer that Athena runs the configured checks before the verifier and sends a failure back once. A test in briefs.rs asserts that text. The producer.md edit is net-zero bytes and the prompt budgets in crates/athena-cli/tests/budgets.rs still pass.
7. [content] docs/decisions.md has a new appended entry with a Provenance bullet that names the root cause: Athena's recorded checks ran only at verification, so the producer could not see a failing gate before the verdict. It also names the temporary-index choice. No existing entry is edited. CHANGELOG.md records the change, and docs/decision-links.toml gets a link only if the entry amends or supersedes an earlier one.
8. [behavior] No reading is newly refused (expected 0% of newly refused readings). Run `cargo test` for the workspace.
9. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass on the final tree (Athena's recorded checks on the same tree, D115).

## Expected deliverables

- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)

## Out of scope

- Adding a new repair loop or raising PRODUCER_SEND_BACKS
- Editing the reader, verifier prompts or the reader budget, or the stale 1,800 doc mentions from jobs 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn (that work is dropped, not kept)
- Changing athena.toml, defaults.toml or workflow files
- Edits to records/ or past decision entries
- Adding any rule that refuses a reading
- Making the planner survive its deadline (the introspection's proposed request); it is a separate job

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check the producer seeing a failed check before the verdict, through a test on the real producer loop.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo fmt --check, clippy and test pass on the final tree.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix goes in the producer loop, where the missing information originates, not at one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The behavior is enforced by code and tests, with no prose-only rule.
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

- path: <state>/jobs/20261114-125837-rzp4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-125837-rzp4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/run/producer.rs runs the configured checks (via the existing runner in crates/athena-core/src/run/checks.rs) on the producer's tree before the verifier, with GIT_INDEX_FILE set to a temporary index of the work tree. A failure's command, exit status and output tail are added to the existing send-back problems and rendered by send_back_text in crates/athena-core/src/run/send_back.rs. The stored invocation detail names the check failure.
The send-back stays bounded by PRODUCER_SEND_BACKS in crates/athena-core/src/run/mod.rs (diff shows no new loop and no changed limit). A check still failing after the send-back is passed to the verifier as flagged, not hidden.
A new test in crates/athena-core/src/run/tests.rs drives the real producer loop with a fake producer whose tree fails a configured check on the first pass and passes after the send-back. Run `cargo test -p athena-core`: it passes, and the brief of the second producer call contains the failed command and its output tail. On the base tree it fails because no send-back happens.
A second test in crates/athena-core/src/run/tests.rs shows that when all checks pass on the first pass there is exactly one producer call and no send-back, and a test shows that a file added by the producer and listed only by `git ls-files` is seen by the in-loop check. Run `cargo test -p athena-core`.
The athena-cli tests that delete README.md and expect Rejected (retry.rs, introspection.rs, steward.rs, code_health_hooks.rs, lifecycle.rs:588) still pass with the extra producer pass. Run `cargo test -p athena-cli`.
crates/athena-context/prompts/producer.md and the producer brief in crates/athena-context/src/briefs.rs tell the producer that Athena runs the configured checks before the verifier and sends a failure back once. A test in briefs.rs asserts that text. The producer.md edit is net-zero bytes and the prompt budgets in crates/athena-cli/tests/budgets.rs still pass.
docs/decisions.md has a new appended entry with a Provenance bullet that names the root cause: Athena's recorded checks ran only at verification, so the producer could not see a failing gate before the verdict. It also names the temporary-index choice. No existing entry is edited. CHANGELOG.md records the change, and docs/decision-links.toml gets a link only if the entry amends or supersedes an earlier one.
No reading is newly refused (expected 0% of newly refused readings). Run `cargo test` for the workspace.
cargo fmt --check, cargo clippy --all-targets and cargo test pass on the final tree (Athena's recorded checks on the same tree, D115).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
