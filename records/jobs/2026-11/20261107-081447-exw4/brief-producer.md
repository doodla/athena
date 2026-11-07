# Job 20261107-081447-exw4: Retry: make planner and critic derive history rules from all commit

## Request (verbatim)

> try again on 20261027-054225-ycks

## Reading (accepted by the user)

Retry of job 20261027-054225-ycks: fix the root cause (landing, history and attribution rules derived from the happy-path commit shape and tested on an unrealistic fixture) in the planner and critic prompts, with a pinning test. Audience: Athena's planner and critic, and users who see fewer repeated landing failures. The earlier candidate is kept in substance and redone, plus the docs budget file it had to touch.

Assumptions:
- Job 20261027-054225-ycks was rejected on 'check test failed'. The recorded failure was the flaky drain test athena_cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain plus an unfinished xtask shard. The verifier said it passes locally and is unrelated to the change, so it is treated as CI noise and retried unchanged, not fixed here. If it recurs, that is a separate job.
- The earlier candidate's five files are all kept as deliverables: CHANGELOG.md, critic.md, planner.md, lib.rs, and docs/system-facts.md. The last one is added because the budget lines (planner 966, critic 902) must be updated when the prompts grow; the verifier called it a necessary consequence, so it is now an expected deliverable.
- Introspection 20261107-081426-m469 proposes first improving the failed-check output extraction. The user's typed request takes precedence, so that is out of scope here; the producer should write the change description and the test names so a failing test is easy to see.
- Code survey: crates/athena-context/src/lib.rs has prompt-set tests, including a frame-size bound (estimate_tokens < 700, around line 404) that the earlier attempt raised to 900, and the system-facts budget test system_facts_budgets_match_computed. crates/athena-context/prompts/planner.md and critic.md have 334 and 398 tokens of headroom against 1300. No new record or data file is added, so the records allow-list is not touched.
- Keep the planner and critic additions compact so each stays under 1300 tokens; the earlier attempt reached 1175 and 992. Prefer not raising the 700 frame bound if the new text can fit, otherwise raise it minimally and say why.
- Root cause named in the CHANGELOG entry: a rule derived from the happy path, not from the full set of shapes the system produces. The guard is prose plus a phrase-pinning test, marked prose-only, per the mechanism default.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md requires that, for any landing, history-check or attribution rule, the plan include an enumerated table of every commit shape the real branch contains (job commits, records-only, owner-path, intervention, retry-after-block landing) and state the rule's outcome for each; Grep finds the requirement.
2. [content] planner.md requires demonstrations of such rules to use a realistic-branch fixture with at least one intervention commit, one records-only commit and the job landing, to say where the fixture differs from the real repository, and to run the tests of every crate that calls the changed classification functions.
3. [content] crates/athena-context/prompts/critic.md has a checklist item that raises a major finding when a landing, history or attribution plan lacks the commit-shape table, lacks the realistic fixture, or omits those crates' tests.
4. [content] The CHANGELOG.md entry names the root cause (rule derived from the happy path, not the full set of shapes), the fix location (planner and critic prompts), the class seen in introspections 20261024-154443-51wt, 20261024-161503-q0pp and 20261025-012736-2q6w, and that the guard is prose-only.
5. [content] docs/system-facts.md budget lines match the computed planner and critic token counts, both at most 1300; reader.md, producer.md, verifier.md, athena.toml, defaults.toml and the workflows are unchanged.
6. [behavior] A new test in crates/athena-context/src/lib.rs asserts that the planner and critic prompts contain the commit-shape table and realistic-fixture requirements. Run `cargo test -p athena-context`: it fails when the prompt edits are reverted and passes with them, and system_facts_budgets_match_computed passes.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` pass; the unrelated drain test is not modified.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Code changes to land.rs, gh.rs, reconcile.rs, run.rs, invariant.rs or other landing and history code
- Editing reader.md, producer.md or verifier.md (reader and verifier budgets are full)
- Changing athena.toml, defaults.toml, .gitattributes or CI workflows
- Adding a code-level check that rejects plans automatically
- Fixing the flaky drain test in crates/athena-cli/src/drain.rs
- Improving failed-check output extraction (introspection 20261107-081426-m469's proposal)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the planner and critic prompts require the table and fixture, and a test fails without the change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the planner and critic prompts, where the happy-path derivation originates.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A phrase-pinning test is added, and the guard is marked prose-only in the CHANGELOG.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New athena-context test; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test must fail on the reverted prompts.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt change, plus the budget doc it forces.
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

- path: <state>/jobs/20261107-081447-exw4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-081447-exw4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/prompts/planner.md requires that, for any landing, history-check or attribution rule, the plan include an enumerated table of every commit shape the real branch contains (job commits, records-only, owner-path, intervention, retry-after-block landing) and state the rule's outcome for each; Grep finds the requirement.
planner.md requires demonstrations of such rules to use a realistic-branch fixture with at least one intervention commit, one records-only commit and the job landing, to say where the fixture differs from the real repository, and to run the tests of every crate that calls the changed classification functions.
crates/athena-context/prompts/critic.md has a checklist item that raises a major finding when a landing, history or attribution plan lacks the commit-shape table, lacks the realistic fixture, or omits those crates' tests.
The CHANGELOG.md entry names the root cause (rule derived from the happy path, not the full set of shapes), the fix location (planner and critic prompts), the class seen in introspections 20261024-154443-51wt, 20261024-161503-q0pp and 20261025-012736-2q6w, and that the guard is prose-only.
docs/system-facts.md budget lines match the computed planner and critic token counts, both at most 1300; reader.md, producer.md, verifier.md, athena.toml, defaults.toml and the workflows are unchanged.
A new test in crates/athena-context/src/lib.rs asserts that the planner and critic prompts contain the commit-shape table and realistic-fixture requirements. Run `cargo test -p athena-context`: it fails when the prompt edits are reverted and passes with them, and system_facts_budgets_match_computed passes.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` pass; the unrelated drain test is not modified.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
