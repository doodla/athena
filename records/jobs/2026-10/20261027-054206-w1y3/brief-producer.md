# Job 20261027-054206-w1y3: Check every reading against the boundary words and repo paths

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading stage works from the request text and incident history, not from the repo's current stat
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-not-checked-against-repo-state-and-writing-rules (20261026-002202-n5bp)
> Cause: The reading stage works from the request text and incident history, not from the repo's current state. It does not see what has already landed (173d12e), which settings already have code defaults, or the boundary word list. Nothing cross-ch...
> Seen 4 time(s) in 4 jobs. The introspections that record it:
> Introspection 20261026-002145-374q (job 20261017-001109-qfax); prevention it proposed: Add a pre-acceptance check on every reading. (1) Scan the reading text with the same banned-word list the boundary test uses and send it back on a hit. (2) Test each deliverable and criterion against recent history on main, and flag any tha...
> Introspection 20261026-002953-bk4j (job 20261026-002911-8gp8); prevention it proposed: Add a pre-acceptance check on every reading. (1) Scan its text against the boundary word list and reject it on a hit. (2) Verify that each config deliverable names a setting without a code default, and that each deliverable is not already l...
> Introspection 20261026-150100-s7mw (job 20261026-145955-s7e2); prevention it proposed: Add a pre-acceptance check on the reading. (1) For each file or path a reading puts in scope or names as a reader, confirm by grep that the code really opens that path, and separate real readers from string mentions. (2) For each criterion ...
> Introspection 20261026-150740-cz1p (job 20261026-150607-85mv); prevention it proposed: Before a reading is accepted, run a check on a request that moves or reroots a path. It searches the code for every access to that path: loads, directory walks, and fully qualified or aliased calls. It compares the hits with the reading's d...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

The reading stage accepts a reading after checking only the prior-job part (check_prior_result_and_scope in crates/athena-core/src/reading.rs). It never checks the reading's own text against the boundary word list, or its deliverables against the repo's files at HEAD. I add one deterministic post-check, run before a reading is accepted, that sends the reading back with the findings. The goal is to stop the class behind jobs 20261017-001109-qfax, 20261026-002911-8gp8, 20261026-145955-s7e2 and 20261026-002202-n5bp; the audience is the operator and later readers.

Assumptions:
- Root cause: the reader's output is validated only for schema, the closed lists and the prior-result/scope rule, never against repo state or the boundary word list. Prompts cannot enforce this, and the reader prompt has 3 tokens of headroom, so the fix is a code check (mechanism default).
- The fix lives beside check_prior_result_and_scope, called from the same place in reading.rs (around line 397). A failure returns the findings as the refusal reason, as the prior-result refusal does now.
- Check 1: run athena_context::briefs::boundary_scan, the scan already used for request input, over the reading's title, summary, assumptions, criteria, out_of_scope and departures.
- Check 2: for each deliverable, 'modify' and 'delete' paths must exist in the tree at HEAD and 'create' paths must not. This catches a reading that re-plans work already landed or names a missing file. It uses the git handle the reader already has.
- Previous jobs 20261017-001109-qfax, 20261026-002911-8gp8 and 20261026-145955-s7e2 are not re-run. Their findings (hold-resume route, stale hold marker, the records reader in gh.rs) are feature-level and outside this root-cause fix; those jobs are addressed only by preventing readings that mis-scope them. 20261026-002911-8gp8 landed already.
- Introspection proposals about call sites for a moved path (grep-based reader checks) and about criteria are not mechanised here. They need semantic judgment, so they stay with the plan critic.

## Acceptance criteria

1. [content] The change description names the root cause: reading output is checked only for schema and prior-result rules, not against the boundary word list or the repo's files at HEAD. The check sits in crates/athena-core/src/reading.rs next to check_prior_result_and_scope, not in a prompt.
2. [behavior] A unit test in reading.rs fails without the change and passes with it: a reading whose summary holds a boundary-scan word is refused, and the refusal names the word. Run `cargo test -p athena-core reading`.
3. [behavior] A unit test shows a reading with a 'modify' or 'delete' deliverable for a path absent at HEAD is refused, and a 'create' deliverable for an existing path is refused. A reading with valid paths and clean text is accepted. Run `cargo test -p athena-core reading`.
4. [behavior] A test runs the reader path with the fake reader returning a bad reading. It shows the reading is not accepted and the reason reaches the caller, so the check is wired in and is not only a free function. Run `cargo test -p athena-core`.
5. [behavior] Existing reading tests, including the prior-result and scope tests, still pass. `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry that says readings are now checked against the boundary words and the repo's files, written for an outside reader.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Re-implementing the disk-space hold, the hold-resume route, or the records-branch reader move from the earlier jobs
- Editing prompt files, which have almost no token headroom
- Semantic checks such as finding every reader of a moved path
- Changing athena.toml, defaults.toml or the workflows
- Changing the boundary word list itself

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is added at the reading stage's validation point, where the class originates, not per job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by code and tests, not by prompt prose.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests, plus fmt, clippy and test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run refusal and acceptance cases.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in reading.rs.
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

The change description names the root cause: reading output is checked only for schema and prior-result rules, not against the boundary word list or the repo's files at HEAD. The check sits in crates/athena-core/src/reading.rs next to check_prior_result_and_scope, not in a prompt.
A unit test in reading.rs fails without the change and passes with it: a reading whose summary holds a boundary-scan word is refused, and the refusal names the word. Run `cargo test -p athena-core reading`.
A unit test shows a reading with a 'modify' or 'delete' deliverable for a path absent at HEAD is refused, and a 'create' deliverable for an existing path is refused. A reading with valid paths and clean text is accepted. Run `cargo test -p athena-core reading`.
A test runs the reader path with the fake reader returning a bad reading. It shows the reading is not accepted and the reason reaches the caller, so the check is wired in and is not only a free function. Run `cargo test -p athena-core`.
Existing reading tests, including the prior-result and scope tests, still pass. `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
CHANGELOG.md has an entry that says readings are now checked against the boundary words and the repo's files, written for an outside reader.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
