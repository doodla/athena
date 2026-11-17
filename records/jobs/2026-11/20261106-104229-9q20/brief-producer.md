# Job 20261106-104229-9q20: Add reconcile deliverables for decision-governed plan files

## Request (verbatim)

> Before the producer starts, map each file and behavior the plan names to the entries in docs/decisions.md that cover them. Use the same rule the decisions check uses. For each match, add a deliverable and a criterion: amend, supersede or confirm that decision in docs/decisions.md, and add the matching [[link]] in docs/decision-links.toml. Give the producer the same list, and have it run the decisions check locally before it reports. Add a test showing that a plan touching a covered file (for example crates/athena-git/src/snapshot.rs under D08) gets the reconcile deliverable, and that a plan touching no covered file gets none. Add a CHANGELOG entry.
> 
> Acceptance:
> - The change is made as described: Jobs edit files or behavior that a decision covers, and only the decisions check or the final verifier notices the missing or stale decision record
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: docs/decisions.md, docs/decision-links.toml., crates/athena-git/src/snapshot.rs

## Reading (accepted by the user)

When a plan or reading names a file a [[governs]] entry covers (e.g. crates/athena-git/src/snapshot.rs under D08), the plan's deliverables gain docs/decisions.md and docs/decision-links.toml plus a reconcile criterion, and the producer gets the same list and is told to run the decisions check locally. Tests cover a governed plan (gets the deliverables) and an ungoverned plan (gets none); CHANGELOG records it.

Assumptions:
- Goal and audience: Athena's own pipeline and the maintainers who judge jobs. A job that edits a governed file should add its reconcile deliverables on the first attempt, not at the decisions check or the final verifier.
- Code survey: crates/athena-data/src/decision_links.rs holds `governing(base_links, planned)`, which uses `Governs::covers` (an exact file, or a directory prefix ending in `/`), and `unreconciled`, which is the decisions check. crates/athena-core/src/run/checks.rs:816 `governing_decisions` calls it with the base list. crates/athena-core/src/run/mod.rs:1387 passes `planned_paths(plan, accepted)` (plan files plus reading deliverables) to `producer_prompt_with_governing`. crates/athena-context/src/briefs.rs:1605 `governing_decisions_section` already lists the governed paths and the reconcile rule in the producer prompt. The entry point is the run pipeline after plan_round. Order: reading accepted, plan round, planned_paths, governing, producer prompt. Not yet present: adding docs/decisions.md and docs/decision-links.toml to the deliverables and criteria, and telling the producer to run the decisions check locally before reporting. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly: the governing list is computed from the real list text and the plan paths, not through a stand-in.
- Existing code: reuse `athena_data::decision_links::governing`, `Governing` and `Governs::covers` (crates/athena-data/src/decision_links.rs), `Athena::governing_decisions` (crates/athena-core/src/run/checks.rs), `planned_paths` (crates/athena-core/src/run/mod.rs) and `governing_decisions_section` (crates/athena-context/src/briefs.rs). Extend them; do not write a second matcher, so the mapping uses the same rule as the decisions check.
- docs/decisions.md and docs/decision-links.toml are in the user's Files list, so the job appends one new decision with a Provenance bullet. It says "Amends D08 in prose only" (D08 governs crates/athena-git/src/snapshot.rs) or amends another decision. It adds a matching `amended` [[link]]. No existing entry is edited. The new entry's id is the next unused D<n>; the producer reads the log tail for it.
- The user's Files list names crates/athena-git/src/snapshot.rs, but nothing in the request changes snapshot behavior. I read it as the example governed path, and it changes only if the producer needs a test fixture there. If it is touched, D08 reconciliation applies to this very job.
- The test lives next to the code it imports: unit tests in crates/athena-core/src/run (checks.rs or tests.rs) or crates/athena-context/src/briefs.rs, built on a literal list text like the one in decision_links.rs tests. It must not read the live docs/decision-links.toml in a way that breaks when entries change.
- Environment: none. No CI workflow, push, PR, or records read is needed. athena.toml, defaults.toml and .github/workflows are not changed, and no config consumers are involved.
- No refusal rule is added or tightened. The decisions gate's refusals stay as they are, so no refusal-rate figure applies. The change moves the reconcile work earlier.

## Acceptance criteria

1. [behavior] A unit test builds a reading or plan whose planned paths include crates/athena-git/src/snapshot.rs, using a list text with D08 governing it. It asserts the resulting deliverables include docs/decisions.md and docs/decision-links.toml and the criteria include a reconcile criterion naming D08. Check by running `cargo test` (athena-core or athena-context) and finding the test by name in the diff.
2. [behavior] A second unit test uses a plan whose paths are covered by no [[governs]] entry (for example README.md). It asserts that no reconcile deliverable and no reconcile criterion is added and that the deliverables are unchanged. The first test fails if the addition is removed. Check by running `cargo test`.
3. [content] The mapping reuses `athena_data::decision_links::governing` (and `Governs::covers`) via the same base-list text. Grep the diff: no new path-matching function exists, and the diff calls `governing(` or `governing_decisions`.
4. [content] The producer prompt carries the same list of governed decisions, deliverables and criteria. It tells the producer to run the decisions check locally before it reports. Check the new prompt text in crates/athena-context/src/briefs.rs (or the producer prompt file), and the `governing_decisions_section` test updated to assert that sentence.
5. [content] docs/decisions.md gains one new appended entry with a Provenance bullet that amends, supersedes or confirms the decision(s) for this change, with no existing entry edited. Check with `git diff` on docs/decisions.md: only additions at the end.
6. [content] docs/decision-links.toml gains the matching [[link]] whose `older` is the governed decision and whose `newer` is the new entry's id, with status amended or superseded. Check with `git diff` on docs/decision-links.toml: additions only, and every existing [[link]] and [[governs]] entry is unchanged.
7. [content] CHANGELOG.md has a new entry under [Unreleased] describing that plans touching decision-governed files now get reconcile deliverables and criteria. Check with `git diff` on CHANGELOG.md.
8. [behavior] No unrelated behavior changes. Run `cargo test` and confirm the existing tests pass, including those in crates/athena-data/src/decision_links.rs and `governing_decisions_add_nothing_when_no_planned_file_is_governed`.
9. [behavior] Run `cargo fmt --check` and `cargo clippy`. Both are clean, or the passing record on the same tree is cited, and `cargo test` passes.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Changing the decisions gate's refusal logic or the [[governs]] entries themselves.
- Changing snapshot behavior in crates/athena-git/src/snapshot.rs.
- Editing any existing entry of docs/decisions.md or anything under records/.
- Changing athena.toml, defaults.toml or CI workflows.
- Prompt files under crates/*/prompts/, which have almost no token headroom.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The change adds two tests, and cargo test, clippy and fmt are criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests check that deliverables are added for a governed plan and not for an ungoverned one, not just that wiring exists.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The reconcile mapping runs in the run pipeline and reuses the gate's matcher.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that extends the existing governing path.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The new decision entry and the CHANGELOG entry are written for an outside reader.
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

- path: <state>/jobs/20261106-104229-9q20/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-104229-9q20/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 2: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 3: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 4: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 5: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 6: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 7: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 8: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 9: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A unit test builds a reading or plan whose planned paths include crates/athena-git/src/snapshot.rs, using a list text with D08 governing it. It asserts the resulting deliverables include docs/decisions.md and docs/decision-links.toml and the criteria include a reconcile criterion naming D08. Check by running `cargo test` (athena-core or athena-context) and finding the test by name in the diff.
A second unit test uses a plan whose paths are covered by no [[governs]] entry (for example README.md). It asserts that no reconcile deliverable and no reconcile criterion is added and that the deliverables are unchanged. The first test fails if the addition is removed. Check by running `cargo test`.
The mapping reuses `athena_data::decision_links::governing` (and `Governs::covers`) via the same base-list text. Grep the diff: no new path-matching function exists, and the diff calls `governing(` or `governing_decisions`.
The producer prompt carries the same list of governed decisions, deliverables and criteria. It tells the producer to run the decisions check locally before it reports. Check the new prompt text in crates/athena-context/src/briefs.rs (or the producer prompt file), and the `governing_decisions_section` test updated to assert that sentence.
docs/decisions.md gains one new appended entry with a Provenance bullet that amends, supersedes or confirms the decision(s) for this change, with no existing entry edited. Check with `git diff` on docs/decisions.md: only additions at the end.
docs/decision-links.toml gains the matching [[link]] whose `older` is the governed decision and whose `newer` is the new entry's id, with status amended or superseded. Check with `git diff` on docs/decision-links.toml: additions only, and every existing [[link]] and [[governs]] entry is unchanged.
CHANGELOG.md has a new entry under [Unreleased] describing that plans touching decision-governed files now get reconcile deliverables and criteria. Check with `git diff` on CHANGELOG.md.
No unrelated behavior changes. Run `cargo test` and confirm the existing tests pass, including those in crates/athena-data/src/decision_links.rs and `governing_decisions_add_nothing_when_no_planned_file_is_governed`.
Run `cargo fmt --check` and `cargo clippy`. Both are clean, or the passing record on the same tree is cited, and `cargo test` passes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
