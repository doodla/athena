# Job 20261030-143609-fj7k: Count only 'Depends on' ids and retry links as plan dependencies

## Request (verbatim)

> try again on 20261030-125332-sz3z

## Reading (accepted by the user)

Retry of job 20261030-125332-sz3z (failed on low disk, never verified): make plan_entry take dependencies only from 'Depends on' lines and the retry link of a 'try again on' request, so cited job ids no longer hold items. The earlier candidate is dropped and the change is redone from base.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (plan_entry at ~line 2125 builds entry.depends_on from job_ids_in(text), the cause of the bug) and grepped crates/athena-cli/tests/drain.rs and crates/athena-cli/src/drain.rs for plan_entry; plan.rs holds the hold logic and reason text, which stay unchanged. No record or data file is added, so check_record_file/AnyRecord are not touched.
- Retry of job 20261030-125332-sz3z: the user wants the same fix redone. The failure was environmental (low disk), not a verifier finding, so the original request's goal and acceptance stand. The introspection's proposal for disk handling is a separate change and is out of scope here.
- Goal: stop evidence-citing requests (including the steward's root-cause requests) from being held forever; the audience is the people waiting on the 29 held items.
- A 'Depends on' line is matched case-insensitively with optional '>', list marker or numbering; only full job ids count. The retry link of a 'try again on <id>' request remains a dependency.
- This tightens dependency detection, so it newly holds nothing and releases held items; no refusal rate increase.

## Acceptance criteria

1. [behavior] A new test through Plan::build_with (plan_entry built from request text) shows a request quoting a rejected job id in its text with no 'Depends on' line is planned and not in Plan::refused. Run: cargo test -p athena-core.
2. [behavior] A new test shows a 'Depends on <id>' line naming a rejected job with no landed retry is still held; its Refusal reason contains 'depends on <id>' and 'rejected'. Run: cargo test -p athena-core.
3. [behavior] A new test shows a 'try again on <id>' request still depends on that id, and that an id on a 'Depends on' line is a dependency while the same id elsewhere in the text is not. Run: cargo test -p athena-core.
4. [behavior] A drain test in crates/athena-cli/tests/drain.rs shows an item that only cites a rejected job id is planned and run, not held. Run: cargo test -p athena-cli.
5. [behavior] Existing reading tests about job ids (reading.rs near line 4164) and drain tests that used 'after <id>' are updated to the new rule; cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
6. [content] CHANGELOG.md has an entry saying only 'Depends on' ids and retry links are dependencies and that cited job ids no longer hold an item.
7. [content] The doc comment of plan_entry in crates/athena-core/src/reading.rs states the new rule in place of 'depends on every other job whose full id it names'.
8. [content] Job 20261030-125332-sz3z failed only on low disk (10.0 GB free, 10.0 GB required) before verification; it had no code findings. Its candidate (reading.rs, plan.rs, drain.rs tests, CHANGELOG.md) is dropped; this retry starts from base and redoes the same change.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changing how rejected or failed dependencies are held, or the held reason text
- Releasing or re-reading the held items by hand
- Changing athena.toml, defaults.toml or workflow files
- Changing the 'referenced job not among the records' open question beyond what the new rule needs
- Low-disk handling (cleanup, waiting-for-disk hold) proposed by the introspection; a separate request

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for planned, held and retry cases; fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix dependency extraction in plan_entry, not the held items one by one.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing items planned or held.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is cited-id requests no longer held, checked by tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to dependency extraction.
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

A new test through Plan::build_with (plan_entry built from request text) shows a request quoting a rejected job id in its text with no 'Depends on' line is planned and not in Plan::refused. Run: cargo test -p athena-core.
A new test shows a 'Depends on <id>' line naming a rejected job with no landed retry is still held; its Refusal reason contains 'depends on <id>' and 'rejected'. Run: cargo test -p athena-core.
A new test shows a 'try again on <id>' request still depends on that id, and that an id on a 'Depends on' line is a dependency while the same id elsewhere in the text is not. Run: cargo test -p athena-core.
A drain test in crates/athena-cli/tests/drain.rs shows an item that only cites a rejected job id is planned and run, not held. Run: cargo test -p athena-cli.
Existing reading tests about job ids (reading.rs near line 4164) and drain tests that used 'after <id>' are updated to the new rule; cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
CHANGELOG.md has an entry saying only 'Depends on' ids and retry links are dependencies and that cited job ids no longer hold an item.
The doc comment of plan_entry in crates/athena-core/src/reading.rs states the new rule in place of 'depends on every other job whose full id it names'.
Job 20261030-125332-sz3z failed only on low disk (10.0 GB free, 10.0 GB required) before verification; it had no code findings. Its candidate (reading.rs, plan.rs, drain.rs tests, CHANGELOG.md) is dropped; this retry starts from base and redoes the same change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
