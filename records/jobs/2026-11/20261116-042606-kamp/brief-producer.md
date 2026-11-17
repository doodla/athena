# Job 20261116-042606-kamp: Give the planner the demonstration format rules in its brief

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The planner is told the plan's shape only in prose, and Athena's structural check (one demonstration
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: plan-breaks-mechanical-format-rule-found-only-in-last-planning-round (20261029-183026-7k8t)
> Cause: The planner is told the plan's shape only in prose, and Athena's structural check (one demonstration per criterion) is not shown to it as a rule. It is also not run as a cheap gate on every draft with its result fed back. A format defect ca...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261029-183010-e7ma (job 20261029-181835-9vvy); prevention it proposed: Run the plan's structural validation (demonstration count per criterion, and any other format rules) on every draft immediately after the planner writes it, before the critic. Return the exact findings to the planner as a mandatory fix, so ...
> Introspection 20261116-042544-t5h1 (job 20261116-041223-m3vc); prevention it proposed: Run the same code check that finds disallowed demonstration commands on each plan draft, and feed the exact offending command and the list of allowed demonstration forms back to the planner inside the mechanical-revision step. Also put the ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the planner brief says "one demonstration per criterion" only in the Report prose, and never says which command shapes the verifier's policy denies (redirection, pipes, chaining). Neither does the mechanical-revision brief, so a plan fails the same check again (jobs 20261029-181835-9vvy and 20261116-041223-m3vc). The fix is information in the planner brief, plus a revision rule and finding text that lists the allowed forms. It adds no refusal rule.

Assumptions:
- Goal and audience: the planner agent and Athena's planning stage. Fewer plans should stop on mechanical format findings, and the maintainers who read the stopped jobs should see that fewer do.
- Code survey: I read crates/athena-core/src/run/planning.rs (plan_round, planning_round, MAX_MECHANICAL_PASSES, the Mechanical round end and the PlanRevision it builds), crates/athena-core/src/admission.rs (plan_findings, the demonstrations count check and the verifier-command Deny check) and crates/athena-context/src/briefs/planner.rs (planner_brief, the rule constants, the Report section and the revision section). The entry point is planning_round: the planner runs, then plan_findings, then the critic. The records validation allow-list is untouched, because no record or data file is added.
- Existing code: the change extends crates/athena-context/src/briefs/planner.rs and its rule constants (REFUSAL_CORPUS_RULE, FIELD_MEANING_RULE, EXITS_RULE) and reuses plan_findings in crates/athena-core/src/admission.rs. The mechanical check already runs on every draft before the critic, and its findings already go back to the planner. The missing piece is that the planner is never told the rule up front.
- Job 20261029-181835-9vvy: its finding was 'Criterion 4 has 2 demonstrations; it needs exactly one'. It is addressed by putting the exactly-one rule and the merge instruction in the brief and in the revision brief. Its candidate changed no files, so none are kept. Its earlier scope (reading.rs refusal work) is dropped.
- Job 20261116-041223-m3vc: its finding was 'cites a command the verifier may not run' (output redirection). It is addressed by listing the denied command shapes in the brief. Its candidate changed no files, so none are kept. Its status-snapshot scope is dropped.
- Refusal-rule share: not applicable, because the change adds no refusal rule. The existing check already refuses these plans, so the share of newly refused past accepted readings is expected 0%.
- The prompt files stay unchanged: planner.md has 12 tokens of headroom, and the rule goes in the Rust brief instead.
- Governance files (athena.toml, defaults.toml and others) are not changed. Mechanical revision pass counts are unchanged.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/planner.rs has a new rule constant, with a doc comment that names the root cause. The rule states that each acceptance criterion gets exactly one demonstration, and that a behavior demonstration is one whole simple command from the verifier list. Pipes, chaining, output redirection and environment variables are not allowed. The planner brief's Report section includes the constant.
2. [content] The mechanical-revision brief (the revision section of planner_brief in crates/athena-context/src/briefs/planner.rs) tells the planner that Athena's own plan checks failed and must be fixed in this revision. It says to merge extra demonstrations into one and to replace a denied command with an allowed one.
3. [behavior] Run `cargo test -p athena-context`. A new unit test builds the planner brief and asserts that it contains the demonstration-format rule. A second test builds a revision brief whose findings include a 'needs exactly one' finding and a 'may not run' finding, and asserts that it carries the mandatory-fix text. Both tests fail on the base tree, where the brief has no such text, and pass with the change.
4. [behavior] Run `cargo test -p athena-cli --test job_plan`. A test with a fake planner that returns two demonstrations for one criterion in pass 1 checks that the pass 2 brief contains the rule and the exact finding, and that the job reaches the critic. The test fails without the change.
5. [behavior] Run `cargo test`. It passes. Also run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean.
6. [content] No refusal rule is added or tightened. plan_findings in crates/athena-core/src/admission.rs keeps the same checks and severities. At most, its finding text now lists the allowed forms. CHANGELOG.md records the change and names the root cause.

## Expected deliverables

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing prompt files (planner.md, contract.md)
- Changing MAX_MECHANICAL_PASSES, athena.toml, defaults.toml or workflows
- Changing the verifier's command grant or the policy evaluator
- The work of jobs 20261029-111202-2bpc and 20261114-194517-c16g (reading.rs refusal rules, status snapshot changes); neither is retried here
- records/ and owner files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and integration tests; fmt, clippy and test are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix gives the planner the rule in its brief rather than refusing more plans.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise the brief the planner receives and the mechanical revision path.
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

37 more qualifying classes are left out.

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

- path: <state>/jobs/20261116-042606-kamp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261116-042606-kamp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 2: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 3: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 4: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 5: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 6: a named file or diff line: cite the path, and the diff line, that shows it.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/src/briefs/planner.rs has a new rule constant, with a doc comment that names the root cause. The rule states that each acceptance criterion gets exactly one demonstration, and that a behavior demonstration is one whole simple command from the verifier list. Pipes, chaining, output redirection and environment variables are not allowed. The planner brief's Report section includes the constant.
The mechanical-revision brief (the revision section of planner_brief in crates/athena-context/src/briefs/planner.rs) tells the planner that Athena's own plan checks failed and must be fixed in this revision. It says to merge extra demonstrations into one and to replace a denied command with an allowed one.
Run `cargo test -p athena-context`. A new unit test builds the planner brief and asserts that it contains the demonstration-format rule. A second test builds a revision brief whose findings include a 'needs exactly one' finding and a 'may not run' finding, and asserts that it carries the mandatory-fix text. Both tests fail on the base tree, where the brief has no such text, and pass with the change.
Run `cargo test -p athena-cli --test job_plan`. A test with a fake planner that returns two demonstrations for one criterion in pass 1 checks that the pass 2 brief contains the rule and the exact finding, and that the job reaches the critic. The test fails without the change.
Run `cargo test`. It passes. Also run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean.
No refusal rule is added or tightened. plan_findings in crates/athena-core/src/admission.rs keeps the same checks and severities. At most, its finding text now lists the allowed forms. CHANGELOG.md records the change and names the root cause.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
