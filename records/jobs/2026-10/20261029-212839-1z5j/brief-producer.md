# Job 20261029-212839-1z5j: Check plan claims against code before the critic; send back open test

## Request (verbatim)

> Change the planner's brief, and add a deterministic plan check where one is possible, so that before a plan goes to the critic every claim about existing behavior is checked against the code. That covers what a named function does on the failure or retry path, which existing tests outside the deliverables the change will break, and which environment, paths and tools the sandboxed checks provide. Any open question that the code can settle must be settled, with the file and line cited. Add a test with a fake planner. It should show that a plan which leaves existing-test breakage as an open question is sent back without using up a critic round.
> 
> Acceptance:
> - The change is made as described: Plans rely on wrong claims about existing functions, tests or the check environment; the critic catches them only in the last round and the job fails before any code
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261024-115027-s1ww.json

## Reading (accepted by the user)

Tighten the planner brief so each claim about existing behavior (a named function's failure and retry path, existing tests the change breaks outside its deliverables, the environment, paths and tools the verifier's checks provide) is checked in code with file and line. Extend the deterministic plan check in admission.rs so a plan that leaves existing-test breakage as an open question is sent back mechanically, and add a fake-planner test showing this uses no critic round. Audience: planners and the critic, who now catch these errors only in the last round.

Assumptions:
- Goal: planners stop relying on unverified claims about existing code; audience is the planner and critic roles and whoever reads failed planning jobs.
- Code survey: read crates/athena-core/src/admission.rs (plan_findings at line 175, open_questions_findings at 524, the User:/Host: prefix rule at 549), crates/athena-core/src/run/planning.rs (lines 211-247: major findings return RoundEnd::Mechanical before critic_round, bounded by MAX_MECHANICAL_PASSES), crates/athena-context/prompts/planner.md, crates/athena-agent/src/fake.rs (fake planner) and crates/athena-cli/tests/job_plan.rs (mechanical-revision tests at lines 821 and 872). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly: the critic call count and round number after a plan with such an open question.
- Existing code: extends crates/athena-core/src/admission.rs open_questions_findings and plan_findings, crates/athena-context/prompts/planner.md, and the mechanical-revision path in crates/athena-core/src/run/planning.rs; none is created by this job.
- The refusal rule extension is narrow: it refuses a plan whose open question (even one prefixed User:/Host:) asks whether existing tests break or what a named function does, since those are searchable. Expected 3% of newly refused readings (it applies to plans, not readings, so the replay of readings is unaffected); the higher refusal rate is intended because those plans spent critic rounds on impact the planner can find by searching.
- Prompt budget: planner prompt has 95 tokens left, so the planner.md edit adds one compact bullet (under about 90 tokens) and replaces wording where possible. Only the planner role gets new brief content; the critic is not changed.
- The deterministic check can only cover the open-question form and the existing 'producer to check' deferral; whether a function claim is true cannot be checked by code, so that part stays in the brief and the critic.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md has a bullet requiring each claim about existing behavior (a named function's failure and retry path, existing tests outside the deliverables that the change breaks, the environment, paths and tools the verifier's checks provide) to be checked in code and cited with file and line, and open questions the code can settle to be settled; the planner prompt stays within its token budget (git diff shows the added lines).
2. [content] crates/athena-core/src/admission.rs plan_findings or open_questions_findings gives a major finding for an open question about whether existing tests break, even with a User: or Host: prefix, and its doc comment lists the rule (git diff shows it).
3. [behavior] A test in crates/athena-cli/tests/job_plan.rs using the fake planner and fake critic shows a first plan leaving existing-test breakage as an open question is sent back with no critic call and round 1 not used, then the revised plan reaches the critic as round 1; run `cargo test -p athena-cli --test job_plan`.
4. [behavior] Unit tests in admission.rs cover the new rule: a test-breakage question is refused and a genuine User: decision question still passes; run `cargo test -p athena-core admission`.
5. [behavior] No unrelated behavior changes: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass, including the existing mechanical-revision tests.
6. [content] CHANGELOG.md has an entry for the new plan check and brief change, and no files other than the listed deliverables change (git diff --stat).

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)

## Out of scope

- Changing the critic prompt, the reader or the producer
- Changing MAX_MECHANICAL_PASSES or critic round limits
- Editing records/ files, including records/failure-classes/2026-10/20261024-115027-s1ww.json
- Proving a claim about a function true by static analysis

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests in admission.rs and a fake-planner test; cargo test, clippy and fmt --check must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs the fake-planner test and checks that no critic call was made.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the brief and the plan check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The test-breakage rule is enforced by plan_findings, not only prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 12 cases in 12 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required survey step to every design job before the note is drafted. The producer writes an Existing code inventory: for each proposed change, the file and function that does it today, or a statement that none exists. For each rule attributed to a doc, it gives the quoted line. For the scoring...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

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

- path: <state>/jobs/20261029-212839-1z5j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-212839-1z5j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/prompts/planner.md has a bullet requiring each claim about existing behavior (a named function's failure and retry path, existing tests outside the deliverables that the change breaks, the environment, paths and tools the verifier's checks provide) to be checked in code and cited with file and line, and open questions the code can settle to be settled; the planner prompt stays within its token budget (git diff shows the added lines).
crates/athena-core/src/admission.rs plan_findings or open_questions_findings gives a major finding for an open question about whether existing tests break, even with a User: or Host: prefix, and its doc comment lists the rule (git diff shows it).
A test in crates/athena-cli/tests/job_plan.rs using the fake planner and fake critic shows a first plan leaving existing-test breakage as an open question is sent back with no critic call and round 1 not used, then the revised plan reaches the critic as round 1; run `cargo test -p athena-cli --test job_plan`.
Unit tests in admission.rs cover the new rule: a test-breakage question is refused and a genuine User: decision question still passes; run `cargo test -p athena-core admission`.
No unrelated behavior changes: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass, including the existing mechanical-revision tests.
CHANGELOG.md has an entry for the new plan check and brief change, and no files other than the listed deliverables change (git diff --stat).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
