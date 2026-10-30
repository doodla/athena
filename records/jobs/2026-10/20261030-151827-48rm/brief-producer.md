# Job 20261030-151827-48rm: Make drain follow athena plan's order and log passed-over items

## Request (verbatim)

> try again on 20261030-033434-3v30

## Reading (accepted by the user)

Retry of job 20261030-033434-3v30: find why drain works items in a different order than `athena plan` (priority record ignored, ready item e5sg skipped silently) and fix it so drain uses the Plan order and logs one line (item, reason) per planned item not worked. Audience: the owner running drain. The fix is kept small so planning fits the job budget.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Steps::queue at line 440, Plan::build_with call at line 1481 in the plan building that also forces refusals, drain at 1892, drain_concurrent at 1903, entry_of at 2360, RealSteps::queue at 2440) and crates/athena-core/src/plan.rs (Plan::build and Plan::build_with, JobEnd). Resolution order: the queue is loaded, entries are built via entry_of, underspecified items are read, then Plan::build_with orders them. No new record or data file is added, so check_record_file and AnyRecord are not touched. Order is measured directly by comparing the sequence of dispatched ids with Plan::build_with(...).order over the same entries and history.
- Prior job 20261030-033434-3v30 failed in planning (budget $5.50 left vs. $5.75 stage caps), before any code; it has no candidate, so nothing is kept or dropped. Its five findings are addressed: (1) root cause must not be asserted from a guessed timeline; the producer weighs a stale plan mid-run, a drain on an older build than `athena plan` (commits 9f3946d and d72e755 changed the dependency rule on 2026-10-30), and a run ending before reaching a waiting item, and says which evidence each explains; (2) one log line per planned item not worked, once per run, covering refused, held, cyclic, not ready, clash, over budget, job limit, run ended; (3) re-plan only when something changed (records, priority records, job ends) or justify the cost; (4) tests cover a priority record present before drain() and one added mid-run; (5) the budget shortfall itself is not fixed here.
- The typed request, retry the drain-order job, takes precedence over the introspection's proposed budget preflight, which is a separate follow-up and out of scope. athena.toml is not changed; if the budget is still too small the job stops again and the owner must raise [budget] separately.
- Test case for e5sg: a ready group-1 item in the first queue citing another job id without 'Depends on' must be worked first; its expected result on base is stated by the producer from running it, not assumed.
- reading.rs is listed only as a possible change if the dependency rule needs touching; it is kept unchanged unless the cause lies there.

## Acceptance criteria

1. [behavior] A new test through drain() or drain_concurrent() in crates/athena-cli/src/drain.rs has a priority record lowering an urgent-text request to low, present before the drain starts, and shows it is worked after a higher-priority item; run `cargo test -p athena-cli drain`.
2. [behavior] A second new test adds the priority record mid-run and shows the drain re-plans and works the lowered item after the higher-priority one; run `cargo test -p athena-cli drain`.
3. [behavior] A new test shows the drain's dispatch order equals Plan::build_with(entries, history).order for the same queue, including a ready group-1 item that cites another job id without 'Depends on'; run `cargo test -p athena-cli drain`.
4. [behavior] A new test shows every planned item the drain does not work (refused, held, cyclic, not ready, clash, over budget, job limit, run ended) gets exactly one log line naming the item id and the reason, once per run; run `cargo test -p athena-cli drain` and check the captured log.
5. [content] The diff removes the divergence at its root: drain ordering uses the same Plan code as `athena plan`, not a second copy of the rule, and re-planning happens only when records, priority records or job ends changed (or the cost is justified in a comment). The cause is stated with the evidence it explains and does not explain.
6. [behavior] Existing drain tests with max_in_flight = 1 pass unchanged; run `cargo test`.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
8. [content] CHANGELOG.md has a new entry under Fixed saying the drain now follows athena plan's order and logs passed-over items with a reason.
9. [content] athena.toml, defaults.toml, .github/workflows and records/ are unchanged.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Budget preflight, held-for-budget state, or any change to athena.toml [budget] (separate follow-up)
- Retrying or fixing jobs 20261029-142858-5c0p, 20261029-233021-e5sg or 20261029-142902-5x4h; named as evidence only
- Changes to athena.toml, defaults.toml, .github/workflows or records/
- Wiring read-ahead or raising max_in_flight
- Changing the priority semantics (urgent/high/normal/low) beyond making drain and plan agree

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain tests plus cargo test and clippy -D warnings criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check dispatch order and log lines by running tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Requires removing the shared-rule divergence and stating the cause with evidence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in drain.rs, with plan.rs and reading.rs only if needed.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Order agreement enforced by a test comparing drain order with Plan order.
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

20 more qualifying classes are left out.

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

A new test through drain() or drain_concurrent() in crates/athena-cli/src/drain.rs has a priority record lowering an urgent-text request to low, present before the drain starts, and shows it is worked after a higher-priority item; run `cargo test -p athena-cli drain`.
A second new test adds the priority record mid-run and shows the drain re-plans and works the lowered item after the higher-priority one; run `cargo test -p athena-cli drain`.
A new test shows the drain's dispatch order equals Plan::build_with(entries, history).order for the same queue, including a ready group-1 item that cites another job id without 'Depends on'; run `cargo test -p athena-cli drain`.
A new test shows every planned item the drain does not work (refused, held, cyclic, not ready, clash, over budget, job limit, run ended) gets exactly one log line naming the item id and the reason, once per run; run `cargo test -p athena-cli drain` and check the captured log.
The diff removes the divergence at its root: drain ordering uses the same Plan code as `athena plan`, not a second copy of the rule, and re-planning happens only when records, priority records or job ends changed (or the cost is justified in a comment). The cause is stated with the evidence it explains and does not explain.
Existing drain tests with max_in_flight = 1 pass unchanged; run `cargo test`.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has a new entry under Fixed saying the drain now follows athena plan's order and logs passed-over items with a reason.
athena.toml, defaults.toml, .github/workflows and records/ are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
