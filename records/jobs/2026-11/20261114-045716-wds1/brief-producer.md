# Job 20261114-045716-wds1: Give the planner a code-claims rule so callers and producers are listed

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The planner designs from the reading and from the code paths it first finds. It does not enumerate e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: plan-leaves-code-answerable-questions-open-and-skips-adjacent-existing-path-fails-planning-rounds (20261031-030110-hnnx)
> Cause: The planner designs from the reading and from the code paths it first finds. It does not enumerate every existing caller and mode that reaches the function it changes, such as the `[parallel] max_concurrency` path into landing. It also may ...
> Seen 8 time(s) in 8 jobs. The introspections that record it:
> Introspection 20261031-030054-6wkt (job 20261030-013555-f5qm); prevention it proposed: Add a plan-structure check that runs before each critic round. (a) A plan may not list an open question that is marked answerable by reading code. The planner must settle it by reading the code and cite the file and line. (b) Any plan that ...
> Introspection 20261031-031632-fdjf (job 20261031-030119-tw3v); prevention it proposed: Add a required step before the critic in planning for any change to a core flow such as landing. The planner must list every state the change touches, including records and job state read from the landing branch, and every existing caller a...
> Introspection 20261103-092900-v31e (job 20261103-075205-wbtm); prevention it proposed: Add a planning rule, enforced by code, that every open question and every 'rejected alternative' resting on a guess is checked against the snapshot before the critic's first round. Questions about a value or rule that existing code or recor...
> Introspection 20261103-135735-32k5 (job 20261103-135304-bwqp); prevention it proposed: Add a planning step that checks the reading's state claims before the plan is built. Every claim about what a job's state does next (stays queued, is retried, is picked up) must name the function that consumes that state, and the plan must ...
> Introspection 20261103-163901-xve1 (job 20261103-162416-m9th); prevention it proposed: Make the planning stage, for any plan that classifies errors or states into categories, enumerate the real producers of each class from the code (every bail!/Err site feeding the arm) and list them in the plan with their classification. Hav...
> Introspection 20261106-002309-v2xt (job 20261105-180330-tb41); prevention it proposed: When a plan adds a wrapper, rule or parser for commands that an existing rule already accepts in several forms, the planner should have to do four things in the plan. First, list the real accepted forms from the existing code: global flags ...
> Introspection 20261107-045737-r0sh (job 20261107-031917-r96w); prevention it proposed: Give planning a code-answerable check before the critic rounds. For a change to a core flow, the planner must list each claim about existing behavior (what counts as an outcome, which records the gate reconciles, which states the scope guar...
> Introspection 20261107-220018-3qj3 (job 20261107-212343-z26k); prevention it proposed: Add a repair step to planning for code-answerable findings. When the last round still has only findings that name a specific file and function, run one extra bounded round in which the planner must settle each finding by reading that code. ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the planner brief (crates/athena-context/src/briefs/planner.rs) asks for settling open questions and listing callers, but never requires every claim about existing behavior (state consumers, error producers, accepted forms, outcome/reconcile rules) to be backed by a read function and file:line, nor requires revisions to settle each code-named finding by reading. I add that information to the brief, as prose with no refusal rule, and address jobs 20261030-013555-f5qm, 20261031-030119-tw3v and 20261103-075205-wbtm as the evidence of the class.

Assumptions:
- Code survey: read crates/athena-context/src/briefs/planner.rs (planner_brief, OPEN_QUESTION_RULE, REFUSAL_CORPUS_RULE, FIELD_MEANING_RULE) and crates/athena-context/src/briefs.rs tests around line 1818; entry point is planner_brief(PlannerInputs); no record or data file is added, so no check_record_file/AnyRecord change.
- Existing code: crates/athena-context/src/briefs/planner.rs OPEN_QUESTION_RULE already asks to settle open questions and list callers; the new rule extends it with claims, producers and revision-finding handling, and the existing test is extended or paralleled.
- Goal and audience: the planner role (and the critic, who judges its plans) get the missing information in the brief; Athena maintainers judge the result.
- Previous jobs: 20261030-013555-f5qm, 20261031-030119-tw3v and 20261103-075205-wbtm failed at planning with no candidate; their code is not kept or revived, their original work is not redone here, and their findings (missed max_concurrency path, record loss from drop, role limit guessed) are the class examples the new rule targets.
- Environment: no CI, push or records read needed.
- Defaults applied: small, tested, root-cause, behavior-evidence.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/planner.rs has a new const (doc comment names the root cause and jobs 20261030-013555-f5qm, 20261031-030119-tw3v, 20261103-075205-wbtm) appended to the Report section of planner_brief. It requires each claim about existing behavior (what consumes a state, which sites produce each error or state class, which input forms an existing rule accepts, which records a gate reconciles, a role or config limit) to name the function and file:line the planner read, and a plan that classifies errors or states to list every producer site with its class.
2. [content] The same Report text tells a revising planner to settle each finding that names a file and function by reading that code and citing file:line, rather than leaving it open or answering by assertion; a finding about an unlisted adjacent path (for example the max_concurrency path, role limits, records read from the branch) is answered by listing that path in dependents.
3. [behavior] A new unit test in crates/athena-context/src/briefs.rs builds planner_brief for a first-round and a revision input and asserts the Report section contains the new wording in both; run `cargo test -p athena-context planner_brief`. It fails without the change because the wording is absent.
4. [content] The change adds no refusal or admission rule, so no refused-reading share applies; it adds only brief text. The planner frame prompt (frame.planner) is not edited, since it has 95 tokens of headroom.
5. [content] CHANGELOG.md has an entry naming the root cause (planner not told to cite the code behind claims about existing callers, producers and states).
6. [content] Recorded fmt, clippy and test checks pass on the final tree.

## Expected deliverables

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding any plan refusal or pre-critic admission rule
- Editing frame.planner or other prompt frames
- Computing caller lists in code
- Redoing the work of jobs 20261030-013555-f5qm, 20261031-030119-tw3v or 20261103-075205-wbtm
- athena.toml, defaults.toml and workflows

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix in the planner brief, where the information is missing
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one brief const and test
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Time budget

Each producer run, the first and any send-back, is stopped after 86400 seconds of wall-clock time. A stopped run sends no report, so what is on disk when it stops must already be a usable change.

Pace the work by its order. Do the smallest change that meets every acceptance criterion first, one file at a time, and leave extras for afterwards; report as soon as the criteria are met. Leave no half-edited file and no broken build between steps. Use targeted tests while you work (`cargo test -p <crate> <filter>`). The full suite runs at most once, and Athena runs it again on your result (see "Checks before the verifier" below), so a second full run only spends this limit.

You have no clock of your own. If the commands Bash may run (below) include `date`, note the time when you start and read it between steps; without it, count steps against the order above.

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

- path: <state>/jobs/20261114-045716-wds1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045716-wds1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/src/briefs/planner.rs has a new const (doc comment names the root cause and jobs 20261030-013555-f5qm, 20261031-030119-tw3v, 20261103-075205-wbtm) appended to the Report section of planner_brief. It requires each claim about existing behavior (what consumes a state, which sites produce each error or state class, which input forms an existing rule accepts, which records a gate reconciles, a role or config limit) to name the function and file:line the planner read, and a plan that classifies errors or states to list every producer site with its class.
The same Report text tells a revising planner to settle each finding that names a file and function by reading that code and citing file:line, rather than leaving it open or answering by assertion; a finding about an unlisted adjacent path (for example the max_concurrency path, role limits, records read from the branch) is answered by listing that path in dependents.
A new unit test in crates/athena-context/src/briefs.rs builds planner_brief for a first-round and a revision input and asserts the Report section contains the new wording in both; run `cargo test -p athena-context planner_brief`. It fails without the change because the wording is absent.
The change adds no refusal or admission rule, so no refused-reading share applies; it adds only brief text. The planner frame prompt (frame.planner) is not edited, since it has 95 tokens of headroom.
CHANGELOG.md has an entry naming the root cause (planner not told to cite the code behind claims about existing callers, producers and states).
Recorded fmt, clippy and test checks pass on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
