# Job 20261030-165704-qfqs: Make drain follow athena plan's order and log passed-over items

## Request (verbatim)

> Priority: urgent
> Re-ask of 20261030-033434-3v30, which stopped twice in planning at the old $8 job cap (no code written; the cap is now $12).
> athena drain does not work items in the order athena plan prints. Evidence from 2026-10-30:
> - After 20261029-142858-5c0p was set to low priority with athena priority, drain-23 dispatched it next, ahead of urgent items.
> - athena plan lists 20261029-233021-e5sg first (group 1, no unmet dependency), yet drain-24 dispatched 20261029-142902-5x4h, then 20261029-180911-7q3e, then 20261029-210827-18fc, and never read or mentioned e5sg; its log has no line naming it.
> Find the cause in the drain's ordering (crates/athena-cli/src/drain.rs, plan building in crates/athena-core/src/plan.rs) and fix it so drain works items in the same order athena plan prints, and when drain passes over a planned item it logs one line naming the item and the reason.
> Acceptance:
> - A test through the drain entry point with a priority record that lowers an urgent-text request shows the drain works it after a higher-priority item.
> - A test shows the drain's order equals athena plan's order for the same queue, and that a passed-over item gets a log line with its reason.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

Re-ask of 3v30: find why drain's dispatch order differs from athena plan (priority records and e5sg's omission), fix it so drain works the planned order, and log one line per passed-over planned item with the reason. Audience: the person running athena drain, who compares its dispatch with athena plan.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs (entry point drain() at ~1966, plan() at ~2175, Steps::queue trait at ~450, RealSteps::entry_of ~2434 and RealSteps::queue ~2514) and the test fakes in crates/athena-cli/tests/drain.rs (fake queue() impls). Plan building lives in crates/athena-core/src/plan.rs. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. The order is measured directly: the sequence of jobs the drain starts, compared with the entries athena plan returns for the same queue.
- Job 20261030-033434-3v30 (previous attempt) stopped in planning with no candidate and no files changed. Its round-2 critique findings are addressed here: (a) do not assume a timeline; weigh three causes, a plan that goes stale mid-run, a drain built before commits 9f3946d and d72e755 (dependency rules), and a run that ended before reaching a waiting item; (b) log every planned item left unworked, with its reason; (c) re-plan only when records, priority records or job ends change; (d) test a priority record present before drain() starts as well as one added mid-run. Its allowed files (drain.rs, plan.rs, CHANGELOG.md) are kept as deliverables; reading.rs is added only as a low-probability deliverable.
- Job 20261029-142858-5c0p (A4, now low priority): not retried here. Its candidate changed nothing, so no work is kept or dropped. It is only the evidence that a lowered job was dispatched ahead of urgent ones.
- Job 20261029-233021-e5sg: not retried here. Its rejected candidate changes to crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-git/src/lib.rs and its CHANGELOG entry are not touched or reapplied; its findings are left to its own follow-up. It is only evidence of an item that plan listed first but drain never logged.
- Job 20261029-142902-5x4h, 20261029-180911-7q3e and 20261029-210827-18fc are cited only as the observed dispatch order; they are not changed.
- Assumption: priority semantics (urgent, high, normal, low) stay as they are. The fix makes drain use the same plan-building function and the same ordering as athena plan, and re-plans after each item only when the queue inputs changed.
- CHANGELOG.md is the only file that gets an entry; athena.toml, defaults.toml and workflows need no change.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-cli: a test calling drain() with a priority record that lowers an urgent-text request shows that item is started after a higher-priority item. It covers both a record present before drain() starts and one added while an earlier item runs.
2. [behavior] Run cargo test: a test builds one queue and shows the sequence of jobs drain() starts equals the entries athena plan's plan() returns for the same queue, including a group-1 item that has a cited-but-not-'Depends on' job id.
3. [behavior] Run cargo test -p athena-cli: a test shows each planned item that is not worked (held, refused, cyclic, not ready, clash with a running batch, over budget, job limit, run ended first) gets exactly one UI log line with its job id and a reason, once per run.
4. [behavior] Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test: all pass, and the existing drain tests pass unchanged.
5. [content] Grep of crates/athena-cli/src/drain.rs and crates/athena-core/src/plan.rs shows drain takes its order from the same plan-building code as plan(), with no separate ordering in drain.
6. [content] CHANGELOG.md has a new entry that names the cause found, the order fix and the passed-over log line.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Retrying or fixing jobs 20261029-142858-5c0p, 20261029-233021-e5sg (status.rs, store.rs, athena-git changes) or 20261029-142902-5x4h; their candidates are not touched
- Changes to athena.toml, defaults.toml, .github/workflows or records/
- Wiring read-ahead or raising max_in_flight
- Changing the priority rules themselves (urgent/high/normal/low semantics) beyond making drain and plan agree

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new drain tests plus cargo test and clippy -D warnings.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check dispatch order and log lines by running tests, not by wiring existing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is weighed among three explanations and the fix is applied to ordering in general.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands the verifier runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to drain ordering and logging.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that drain order equals plan order, not that code was edited.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

22 more qualifying classes are left out.

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

Run cargo test -p athena-cli: a test calling drain() with a priority record that lowers an urgent-text request shows that item is started after a higher-priority item. It covers both a record present before drain() starts and one added while an earlier item runs.
Run cargo test: a test builds one queue and shows the sequence of jobs drain() starts equals the entries athena plan's plan() returns for the same queue, including a group-1 item that has a cited-but-not-'Depends on' job id.
Run cargo test -p athena-cli: a test shows each planned item that is not worked (held, refused, cyclic, not ready, clash with a running batch, over budget, job limit, run ended first) gets exactly one UI log line with its job id and a reason, once per run.
Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test: all pass, and the existing drain tests pass unchanged.
Grep of crates/athena-cli/src/drain.rs and crates/athena-core/src/plan.rs shows drain takes its order from the same plan-building code as plan(), with no separate ordering in drain.
CHANGELOG.md has a new entry that names the cause found, the order fix and the passed-over log line.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
