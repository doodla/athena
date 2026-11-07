# Job 20261027-054225-ycks: Make planner and critic derive history rules from every commit shape

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The planning stage derives a commit-shape rule from the intended happy path instead of from the full
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-tested-only-against-job-commits-not-realistic-branch-history (20261024-140616-afw6)
> Cause: The planning stage derives a commit-shape rule from the intended happy path instead of from the full set of shapes the system produces. Here it missed the retry-after-block landing. The plan's demonstrations also use a fixture that does not...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261024-154443-51wt (job 20261024-153908-1rve); prevention it proposed: Change the plan template and critic checklist for any landing or history-check rule. The plan must include an enumerated table of every commit shape the real branch contains. The table must cover job commits, records-only commits, owner-pat...
> Introspection 20261024-161503-q0pp (job 20261024-154552-e65v); prevention it proposed: Before handover, run a check that runs every test in each crate that calls the changed history or classification functions. For this change that means `athena-core` reconcile and `athena-cli --test reconcile`, not only `land::`. The check s...
> Introspection 20261025-012736-2q6w (job 20261025-012657-hac3); prevention it proposed: Add a default to the reading stage. Any criterion on a landing, history or attribution check must name a test fixture that uses a realistic branch. That branch has at least one intervention commit, one records-only commit and the job landin...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the planning stage derives landing, history and attribution rules from the happy-path commit shape, and tests them on a fixture unlike real branch history. Fix it in the planner and critic prompts (crates/athena-context/prompts) by requiring an enumerated table of every real commit shape and a realistic fixture, with a test that pins this. Audience: Athena's planner and critic, and the users who see fewer repeated landing failures.

Assumptions:
- Code Survey / Code survey: I read crates/athena-context/prompts/planner.md and critic.md (the stage prompts, 334 and 398 tokens of headroom) and listed crates/athena-context/src/lib.rs, briefs.rs and outputs.rs, where the prompts are composed and where a test can read them. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- Code survey: the property is measured directly by a test in crates/athena-context/src/lib.rs that loads the composed planner and critic prompts and asserts each contains the commit-shape table requirement; it fails on the current prompts.
- Root cause: a commit-shape rule is derived from the intended happy path, not from the full set of shapes the system produces (job commits, records-only commits, owner-path commits, intervention commits, retry-after-block landings), and plans demonstrate it on a fixture lacking those shapes.
- Reader prompt (2000-token budget, 3 left) and the verifier/producer prompts are left unchanged; the fix goes in planner and critic, which have room.
- Job 20261024-153908-1rve: it ended with no candidate and no files changed; its pr_commits work is not repeated here. This request fixes the planning cause of its stop (a plan that missed the blocked-then-landed shape) rather than redoing that code.
- Job 20261024-154552-e65v: rejected; its candidate files (CHANGELOG.md, gh.rs, land.rs, invariant.rs, docs/design/records-history.md) are dropped, not kept. Its findings (reconcile fixtures with trailers only, owner-notes instead of the real operator path, and a missed landing-record writer in reconcile.rs) are the instances of the class, prevented by the new planner and critic rules rather than by patching code.
- Job 20261025-012657-hac3: rejected; its 11 changed files (commits.rs, manage.rs, athena-git lib.rs, gh.rs, status.rs, auditor.rs, athena-gate attribution.rs, history.rs, lib.rs, CHANGELOG.md, docs/decisions.md) are dropped, not kept. Its findings (a test asserting violations where the criterion required none, and untested fallbacks in status.rs and auditor.rs) are likewise covered by the rules, not patched here.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md requires that, for any landing, history-check or attribution rule, the plan includes an enumerated table of every commit shape the real branch contains (job commits, records-only, owner-path, intervention, retry-after-block landing) and states the rule's outcome for each; Grep for the table requirement finds it.
2. [content] crates/athena-context/prompts/planner.md requires demonstrations of such rules to use a fixture of a realistic branch with at least one intervention commit, one records-only commit and the job landing, and to say where the fixture differs from the real repository.
3. [content] crates/athena-context/prompts/critic.md has a checklist item that raises a major finding when a landing, history or attribution plan lacks the commit-shape table, lacks the realistic fixture, or omits the tests of every crate calling the changed classification functions.
4. [content] The change description names the root cause (rule derived from the happy path, not the full set of shapes the system produces) and the fix location (planner and critic prompts), and mentions the three introspections' class.
5. [content] The planner and critic prompt sets stay within their token budgets (1300 each), and the reader, producer and verifier prompt files are not modified.
6. [behavior] A new test in crates/athena-context/src/lib.rs asserts the planner and critic prompts contain the table and realistic-fixture requirements; run `cargo test -p athena-context`: it fails on the original prompts and passes with the change.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Code changes to land.rs, gh.rs, reconcile.rs, run.rs, invariant.rs or other landing and history code from jobs 20261024-153908-1rve, 20261024-154552-e65v and 20261025-012657-hac3
- Editing reader.md, producer.md or verifier.md (reader and verifier budgets are full)
- Changing athena.toml, defaults.toml, .gitattributes or CI workflows
- Adding a code-level check that rejects plans automatically

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the planner and critic prompts require the table and realistic fixture, backed by a test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test in athena-context; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the planning stage where the shape rule is derived, not in one job's code.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Prompt text is pinned by a test, so removing it fails the build.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Two prompt files and one test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

32 more qualifying classes are left out.

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

- path: <state>/jobs/20261027-054225-ycks/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054225-ycks/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/prompts/planner.md requires that, for any landing, history-check or attribution rule, the plan includes an enumerated table of every commit shape the real branch contains (job commits, records-only, owner-path, intervention, retry-after-block landing) and states the rule's outcome for each; Grep for the table requirement finds it.
crates/athena-context/prompts/planner.md requires demonstrations of such rules to use a fixture of a realistic branch with at least one intervention commit, one records-only commit and the job landing, and to say where the fixture differs from the real repository.
crates/athena-context/prompts/critic.md has a checklist item that raises a major finding when a landing, history or attribution plan lacks the commit-shape table, lacks the realistic fixture, or omits the tests of every crate calling the changed classification functions.
The change description names the root cause (rule derived from the happy path, not the full set of shapes the system produces) and the fix location (planner and critic prompts), and mentions the three introspections' class.
The planner and critic prompt sets stay within their token budgets (1300 each), and the reader, producer and verifier prompt files are not modified.
A new test in crates/athena-context/src/lib.rs asserts the planner and critic prompts contain the table and realistic-fixture requirements; run `cargo test -p athena-context`: it fails on the original prompts and passes with the change.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
