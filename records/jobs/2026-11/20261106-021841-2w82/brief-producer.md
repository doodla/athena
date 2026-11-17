# Job 20261106-021841-2w82: Add stale-text check that fails when old values or claims remain

## Request (verbatim)

> Add a check that runs before the final verdict. When a job's reading or diff changes a value or a stated behaviour (a constant, a limit, a rule that a decision describes), search the repo's code, CLI help strings and docs, including docs/decisions.md, for the old value or the old claim. Fail the check and name each place that still says the old thing, unless that place is in the diff, is marked 'amended by' or 'superseded by', or is in CHANGELOG.md. Give the producer this list inside its own loop so it can fix the text before verification. Where a help string restates a limit, build it from the constant that enforces the limit. Acceptance: (1) a test where a constant changes from 8 to 16 but a help string still says 8 fails the check and names that line; (2) a test where a decision in docs/decisions.md says the reverse of a changed rule and is not marked amended fails the check; (3) both tests pass once the text is updated; (4) existing tests still pass; (5) a CHANGELOG entry states only what the diff does.
> 
> Acceptance:
> - The change is made as described: A behaviour change leaves old text (decision record, help string) contradicting it; only the final verdict catches it, and each case needs a follow-up job
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: docs/decisions.md

## Reading (accepted by the user)

Add a pre-verdict check in the run pipeline: when the diff changes a constant, limit or stated rule, search code, CLI help and docs (including docs/decisions.md) for the old value or claim and fail naming each stale place, except text in the diff, marked 'amended by'/'superseded by', or in CHANGELOG.md. The producer loop gets the list as a send-back so it fixes text before verification; help strings that restate a limit are built from the enforcing constant. Audience: Athena maintainers and producers.

Assumptions:
- Goal: stop jobs that change a value or rule from leaving old text (decision records, help strings) contradicting it, so no follow-up job is needed. Audience: Athena maintainers and producers.
- Code survey: read crates/athena-core/src/run/producer.rs (producer_loop at line 298, send-back loop with PRODUCER_SEND_BACKS), crates/athena-core/src/run/checks.rs (decision_gate, run_loop_checks, loop_check_failures), crates/athena-cli/src/decisions.rs (log guard, ConfirmedLinks, link status from docs/decision-links.toml), crates/athena-data/src/decision_links.rs. The new check hooks beside the existing loop checks, so the producer sees it in-loop and the same function runs before verdict. No record or data file is added under crates/, so check_record_file/AnyRecord are not touched.
- Existing code: reuse crates/athena-core/src/run/checks.rs loop-check plumbing, crates/athena-core/src/run/send_back.rs, and crates/athena-cli/src/decisions.rs / docs/decision-links.toml for 'amended by'/'superseded by' status; a criterion extends the send-back path.
- Field meaning: no existing field, argument or key is reused for a new purpose.
- Term: 'marked amended by / superseded by' means the decision entry text contains that phrase or docs/decision-links.toml confirms the link.
- Detection is heuristic: old values come from removed diff lines (numeric/string literals in constants) and from decision-style claims in the reading; the producer chooses the exact module name stale_text.rs, listed here as a created file.
- The check refuses more jobs than before; the higher refusal rate is intended: expected 10% of newly refused readings.
- Existing paths not in the snapshot: none are assumed; docs/decisions.md and docs/decision-links.toml exist. Help-string constants are located by the producer by search.
- Environment: none; no CI workflow, push, PR or history read.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/stale_text.rs builds a diff where a constant changes from 8 to 16 while a CLI help string still says 8; the check fails and its finding names that file and line. Verified with `cargo test -p athena-core stale_text`.
2. [behavior] A new test uses a decision log text where a decision states the reverse of a changed rule and has no 'amended by' or 'superseded by' mark; the check fails and names that decision's id and line. Run with `cargo test -p athena-core stale_text`.
3. [behavior] A new test shows both fixtures pass once the help text and decision text are updated, and that matches in the diff, in a marked entry, or in CHANGELOG.md are not reported. Run with `cargo test -p athena-core stale_text`.
4. [content] crates/athena-core/src/run/producer.rs (producer_loop) feeds the check's list into the producer's send-back before the candidate reaches verification, and the check runs before the final verdict; a diff in those files shows the call.
5. [content] Each CLI help string that restates a limit is built from the constant enforcing it (format! or concat from the constant), shown in the diff of the file where each such constant and string live; no literal copy of the limit remains in help text.
6. [content] The change is recorded in a new appended entry in docs/decisions.md with a Provenance bullet; no existing entry is edited. docs/decision-links.toml is changed only if the new entry amends or supersedes an earlier one.
7. [content] CHANGELOG.md gains one entry stating only what the diff does (the check, the send-back, help strings built from constants), with no other claims.
8. [behavior] Existing tests still pass and the build is clean: `cargo fmt --check`, `cargo clippy` and `cargo test` (Athena's recorded checks on the final tree count).
9. [content] Scope limit: apart from the deliverables and CHANGELOG.md, no other files change and no existing behavior changes.

## Expected deliverables

- crates/athena-core/src/run/stale_text.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing or annotating existing entries of docs/decisions.md
- Changing athena.toml, defaults.toml or CI workflows
- Semantic (non-textual) detection of contradictions beyond the old value or claim search

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt pass with new tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the two fixture tests and show failures naming lines.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a check in the run pipeline, not prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry and CHANGELOG written for outside readers.
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

- path: <state>/jobs/20261106-021841-2w82/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-021841-2w82/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-core/src/run/stale_text.rs builds a diff where a constant changes from 8 to 16 while a CLI help string still says 8; the check fails and its finding names that file and line. Verified with `cargo test -p athena-core stale_text`.
A new test uses a decision log text where a decision states the reverse of a changed rule and has no 'amended by' or 'superseded by' mark; the check fails and names that decision's id and line. Run with `cargo test -p athena-core stale_text`.
A new test shows both fixtures pass once the help text and decision text are updated, and that matches in the diff, in a marked entry, or in CHANGELOG.md are not reported. Run with `cargo test -p athena-core stale_text`.
crates/athena-core/src/run/producer.rs (producer_loop) feeds the check's list into the producer's send-back before the candidate reaches verification, and the check runs before the final verdict; a diff in those files shows the call.
Each CLI help string that restates a limit is built from the constant enforcing it (format! or concat from the constant), shown in the diff of the file where each such constant and string live; no literal copy of the limit remains in help text.
The change is recorded in a new appended entry in docs/decisions.md with a Provenance bullet; no existing entry is edited. docs/decision-links.toml is changed only if the new entry amends or supersedes an earlier one.
CHANGELOG.md gains one entry stating only what the diff does (the check, the send-back, help strings built from constants), with no other claims.
Existing tests still pass and the build is clean: `cargo fmt --check`, `cargo clippy` and `cargo test` (Athena's recorded checks on the final tree count).
Scope limit: apart from the deliverables and CHANGELOG.md, no other files change and no existing behavior changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
