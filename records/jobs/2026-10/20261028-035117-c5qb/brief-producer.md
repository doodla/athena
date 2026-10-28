# Job 20261028-035117-c5qb: Plan hard jobs only; count later-caused failures in keep rule

## Request (verbatim)

> Priority: urgent
> Turn job planning back on for hard jobs only, and fix the keep rule's measure (Sri, 2026-10-28). Planning was turned off on 2026-10-26 because the keep rule measured landed share and cost; the next day a job with no planner or critic (20261027-054211-a2n5) landed a crude word-matching refusal rule that stopped three drains, which landed share counts as a success. Change: (1) in athena.toml [job_plan], planning runs when the reading marks a job hard (the same marking that already escalates the producer), and is skipped otherwise; a job is hard when it adds or changes a rule that can refuse or hold work, or changes landing, the history check, records, the run pipeline or drain; (2) the keep rule (docs/design/job-planning.md section 9) also counts, per planned and unplanned job, landings that later caused a failure, a revert or a fix job citing them, and reports them in athena status beside the landed share.
> Acceptance:
> - A test shows a reading marked hard runs the planner and critic and an ordinary reading does not, with athena.toml set to planning for hard jobs only.
> - A test shows status reports, for planned and unplanned landings, how many were later cited as the cause of a failure, revert or fix.
> - docs/design/job-planning.md section 9 and docs/decisions.md record the change and why.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: athena.toml, crates/athena-data/src/config.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, docs/design/job-planning.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Replace the boolean job_plan switch with a hard-jobs-only mode in athena.toml, so the run pipeline plans and critiques only when the reading is marked hard. Extend the keep rule (job-planning.md section 9) and athena status to count, per planned and unplanned landing, later failures, reverts or fix jobs citing them. This follows job 20261027-054211-a2n5, whose landed refusal rule showed landed share is the wrong measure. Audience: Sri and the operators who read athena status.

Assumptions:
- Job 20261027-054211-a2n5 is cited as the motivating case. Its two findings (broad governance keywords, separate keyword test table) concern reading.rs governance matching and are not part of this request. They are out of scope and remain unaddressed here.
- The config gains a mode setting (for example mode = "hard") in [job_plan] in place of or alongside enabled; the existing enabled=false stays a full off switch.
- Hardness comes from the existing hard marking in the reading, which already escalates the producer. reading.rs only gets a hardness definition (rule that can refuse or hold work, landing, history check, records, run pipeline, drain) if the marking lacks it.
- A later cause is read from existing records: failure introspections, revert records and fix jobs citing a landed job id. No new record kind is added unless none exists.
- athena.toml is a governance file and is a named deliverable here.

## Acceptance criteria

1. [behavior] Run cargo test: a test loads athena.toml (planning for hard jobs only), and shows a reading marked hard runs the planner and critic while an ordinary reading runs neither.
2. [behavior] Run cargo test: a test shows athena status reports, separately for planned and unplanned landings, how many were later cited as the cause of a failure, revert or fix job, beside the landed share.
3. [content] athena.toml [job_plan] is set so planning runs for hard jobs only (git diff shows the change), and crates/athena-data/src/config.rs parses and validates the setting.
4. [content] docs/design/job-planning.md section 9 and docs/decisions.md record the new keep-rule measure, the hard-only planning, and why, citing job 20261027-054211-a2n5.
5. [content] The hard definition in code or docs covers rules that refuse or hold work, landing, the history check, records, the run pipeline and drain.
6. [behavior] cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.
7. [content] CHANGELOG.md has an entry for the change.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- docs/design/job-planning.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Fixing the two findings of job 20261027-054211-a2n5 (broad governance keywords, KEYWORD_REQUESTS table)
- Changing prompt files
- Changing defaults.toml, workflows or other governance files besides athena.toml
- Changing the planner or critic prompts, caps or max_rounds
- Reverting or reworking the refusal rule job 20261027-054211-a2n5 landed

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for planning dispatch and status counts; cargo test, clippy -D warnings and fmt pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise hard versus ordinary readings and the status output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The keep rule now measures later harm, not only landed share.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Hard-only planning is enforced by config and run code, tested.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decisions explain the change and why for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the listed files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

16 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test: a test loads athena.toml (planning for hard jobs only), and shows a reading marked hard runs the planner and critic while an ordinary reading runs neither.
Run cargo test: a test shows athena status reports, separately for planned and unplanned landings, how many were later cited as the cause of a failure, revert or fix job, beside the landed share.
athena.toml [job_plan] is set so planning runs for hard jobs only (git diff shows the change), and crates/athena-data/src/config.rs parses and validates the setting.
docs/design/job-planning.md section 9 and docs/decisions.md record the new keep-rule measure, the hard-only planning, and why, citing job 20261027-054211-a2n5.
The hard definition in code or docs covers rules that refuse or hold work, landing, the history check, records, the run pipeline and drain.
cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.
CHANGELOG.md has an entry for the change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
