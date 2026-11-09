# Job 20261109-035221-jj8g: Rewrite agent prompts leaner and rebase the prompt token budgets

## Request (verbatim)

> Hmm. I kinda want to rethink that budget and instructions. What does the model actually need to know? Knowledge can be in skills or reference docs or whatever as well.
> 
>  Those instructions look kinda mid

## Reading (accepted by the user)

Reading "that budget and instructions" as the fixed prompt token budgets (crates/athena-context/src/lib.rs) and the always-loaded role prompts. The job rewrites the prompts to hold only what each model needs to act, moves reference knowledge into a docs/ file agents can read on demand, and resets the budgets to match. The audience is the maintainer, who finds the prompts mediocre.

Assumptions:
- Term ladder: none used; 'budget' means the fixed estimated-token budget per always-loaded prompt set (BudgetSet::budget in crates/athena-context/src/lib.rs), not the [budget] USD caps in athena.toml.
- Goal: prompts that give each role only what it needs to act correctly, with lookup-style knowledge moved to a reference doc. Audience: the maintainer, plus the agents who read the prompts.
- Code survey: crates/athena-context/src/lib.rs defines BudgetSet budgets (contract 350, reader 2000, producer/verifier 1300, others 600), compose() and check_budget(). crates/athena-context/src/briefs.rs has tests that quote headroom rows such as '1298 of 1300'. Prompts live in crates/athena-context/prompts/*.md and crates/athena-cap-change/prompts/ and crates/athena-cap-study/prompts/. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. Token use is measured directly with estimate_tokens on the composed text through check_budget.
- Existing code: crates/athena-context/src/lib.rs, crates/athena-context/prompts/, crates/athena-cap-change/prompts/, crates/athena-cap-study/prompts/ are extended, not recreated.
- Consumers: none found for athena.toml changes; the budget values live in lib.rs, so athena.toml and defaults.toml stay unchanged.
- The rewrite keeps every rule that a check enforces; prose-only guidance may be shortened or moved. The reference doc is read by agents with the Read tool.
- Only enabled roles (reader, planner, critic, producer, verifier) get new content.

## Acceptance criteria

1. [behavior] `cargo test --locked` passes, including the budget tests in crates/athena-context (every set within its budget) and any updated headroom-row tests in crates/athena-context/src/briefs.rs.
2. [behavior] `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` are clean.
3. [content] `git diff` shows the always-loaded prompt files under crates/athena-context/prompts/ and the capability prompts rewritten. For each role, the diff drops text that restates defaults, catalog or facts already in the brief, and keeps the role's job, output contract and hard rules.
4. [content] A reference doc under docs/ holds the knowledge moved out of the prompts, and each rewritten prompt that depends on it names its path. The doc exists and the pointer path matches it.
5. [content] The BudgetSet::budget values in crates/athena-context/src/lib.rs are changed only where the rewrite justifies it; the diff gives a reason, and no budget rises without a matching cut elsewhere.
6. [content] Every rule the old prompts stated that a check enforces (for example the reader's Code survey and criterion rules) is still stated in the new prompts or in the referenced doc. Grep of the old and new text shows each is present.
7. [content] Outcome check: the diff includes a measured before-and-after token count per prompt set, taken from check_budget or the headroom rows. The sets are leaner, or the extra headroom is stated.
8. [content] CHANGELOG.md has an entry for the prompt and budget change.

## Expected deliverables

- crates/athena-context/prompts/contract.md (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/agent-reference.md (create)
- CHANGELOG.md (modify)

## Out of scope

- The USD [budget] caps in athena.toml and defaults.toml
- Changing which roles exist or the pipeline's logic
- Study-capability behavior beyond prompt wording
- records/ and owner files

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires measured before/after token counts and the retention of enforced rules.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test and clippy to pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to prompts, budget constants and one reference doc.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Read 'that budget and instructions' from the repo as the prompt token budgets and role prompts.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The reference doc and the prompts are written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Budgets stay enforced by check_budget; prose-only guidance may move to the doc.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
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

- path: <state>/jobs/20261109-035221-jj8g/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-035221-jj8g/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

`cargo test --locked` passes, including the budget tests in crates/athena-context (every set within its budget) and any updated headroom-row tests in crates/athena-context/src/briefs.rs.
`cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` are clean.
`git diff` shows the always-loaded prompt files under crates/athena-context/prompts/ and the capability prompts rewritten. For each role, the diff drops text that restates defaults, catalog or facts already in the brief, and keeps the role's job, output contract and hard rules.
A reference doc under docs/ holds the knowledge moved out of the prompts, and each rewritten prompt that depends on it names its path. The doc exists and the pointer path matches it.
The BudgetSet::budget values in crates/athena-context/src/lib.rs are changed only where the rewrite justifies it; the diff gives a reason, and no budget rises without a matching cut elsewhere.
Every rule the old prompts stated that a check enforces (for example the reader's Code survey and criterion rules) is still stated in the new prompts or in the referenced doc. Grep of the old and new text shows each is present.
Outcome check: the diff includes a measured before-and-after token count per prompt set, taken from check_budget or the headroom rows. The sets are leaner, or the extra headroom is stated.
CHANGELOG.md has an entry for the prompt and budget change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
