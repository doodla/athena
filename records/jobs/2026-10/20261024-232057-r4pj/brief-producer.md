# Job 20261024-232057-r4pj: Trim verifier and study producer prompts to free 100+ tokens each

## Request (verbatim)

> Free prompt-budget room before more instructions are added: the change verifier composes at 1298 of its 1300-token budget, the study verifier at 1296 and the study producer at 1276 (cargo test --locked -p athena-cli --test budgets -- --nocapture prints them). Remove text in those prompts and their shared frames that repeats what the briefs already say or that no current rule needs.
> Acceptance:
> - The budgets test passes and each of those three sets is at least 100 tokens under its budget.
> - No brief test or behavior criterion changes meaning.
> - docs/decisions.md lists what was removed and why.
> Files: crates/athena-context/prompts, crates/athena-cap-change/prompts, crates/athena-cap-study/prompts, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Cut repeated or rule-less text from the change verifier, study verifier and study producer prompts and the shared frames so each set sits at least 100 tokens under its 1300 budget, without changing any brief rule. Record the removals in docs/decisions.md and CHANGELOG.md, and refresh the budget lines in docs/system-facts.md that the facts test checks.

Assumptions:
- Goal: free prompt-budget room for later instructions; audience is maintainers adding rules and the verifier checking the diff.
- Code survey: read crates/athena-cli/tests/budgets.rs; it composes each role set from the prompt files (crates/athena-context/prompts, crates/athena-cap-change/prompts, crates/athena-cap-study/prompts) and calls check_budget; system_facts_budgets_match_computed compares docs/system-facts.md budget lines to computed sizes. No record or data file is added.
- The producer is the study producer (cap-study/prompts/producer.md at 1257 by the headroom table, 1276 per the request) plus frame producer; the 'at least 100 under' target is measured by the budgets test output, not by the table.
- Shared frames (athena-context producer.md and verifier.md) also feed the change producer and the planner/critic; trims there only shrink them, but docs/system-facts.md budget lines for producer, verifier, planner and critic must be updated to computed values or system_facts_budgets_match_computed fails.
- Contract.md is shared by every set including the reader; trimming it is allowed but not required, and the reader set must not grow.
- Existing tests asserting prompt phrases (brief tests) must still pass unmodified; the producer greps them before deleting any sentence.
- docs/system-facts.md is not named in the request but is required by the facts test; it is listed as a deliverable.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`: it passes, and the printed sizes for the change verifier, the study verifier and the study producer are each at most 1200.
2. [behavior] Run `cargo test --locked` for the workspace: all tests pass, including every prompt/brief test, with no test file modified in the diff, and `cargo clippy --locked --all-targets` is clean.
3. [content] The diff touches only prompt .md files under crates/athena-context/prompts, crates/athena-cap-change/prompts and crates/athena-cap-study/prompts, plus docs/decisions.md, CHANGELOG.md and docs/system-facts.md; no brief test, behavior criterion or rule is reworded in meaning (removed text is repetition of brief content or backs no current rule).
4. [content] docs/decisions.md has a new entry listing each removed passage (or group) and why it was removable, citing the file; a reader can match each bullet to the diff.
5. [content] CHANGELOG.md has an entry describing the prompt trim and the new headroom.
6. [content] docs/system-facts.md budget lines equal the computed sizes (system_facts_budgets_match_computed passes) and budgets are unchanged.
7. [content] Standing defaults: nothing secret or personal is committed; the change is one focused edit with text written for an outside reader.

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/contract.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)

## Out of scope

- Changing any budget value, test, or athena.toml/defaults.toml
- Adding new rules or instructions to the prompts
- Trimming the reader, planner or critic prompts beyond what shared frames imply

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the budgets test output and that rules remain, not that edits were made.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: decisions.md entry written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test and clippy.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt-trim change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The budgets test enforces the headroom.
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

- path: <state>/jobs/20261024-232057-r4pj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261024-232057-r4pj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`: it passes, and the printed sizes for the change verifier, the study verifier and the study producer are each at most 1200.
Run `cargo test --locked` for the workspace: all tests pass, including every prompt/brief test, with no test file modified in the diff, and `cargo clippy --locked --all-targets` is clean.
The diff touches only prompt .md files under crates/athena-context/prompts, crates/athena-cap-change/prompts and crates/athena-cap-study/prompts, plus docs/decisions.md, CHANGELOG.md and docs/system-facts.md; no brief test, behavior criterion or rule is reworded in meaning (removed text is repetition of brief content or backs no current rule).
docs/decisions.md has a new entry listing each removed passage (or group) and why it was removable, citing the file; a reader can match each bullet to the diff.
CHANGELOG.md has an entry describing the prompt trim and the new headroom.
docs/system-facts.md budget lines equal the computed sizes (system_facts_budgets_match_computed passes) and budgets are unchanged.
Standing defaults: nothing secret or personal is committed; the change is one focused edit with text written for an outside reader.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
