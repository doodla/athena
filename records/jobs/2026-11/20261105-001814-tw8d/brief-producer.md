# Job 20261105-001814-tw8d: Let the batch run deferred requests whose proposed reading passes checks

## Request (verbatim)

> try again on 20261104-215123-fpg0

## Reading (accepted by the user)

Retry of job 20261104-215123-fpg0 (dispatched, never verified, no findings, no candidate, so nothing to fix or keep): `athena drain` should run a request queued by `ask --later`, the Steward or Hestia whose latest reading is proposed and passes every reading check, in priority order; a changed-scope re-read after a settings change still waits for a person. Audience: the operator relying on unattended batches.

Assumptions:
- Prior job 20261104-215123-fpg0: it recorded no findings and no candidate, so there is no work to keep or drop. This retry takes the same reading and the same deliverables, and out_of_scope is unchanged. Its four files are all listed as deliverables here.
- Code survey: crates/athena-core/src/steward.rs `queued_work` (line ~356) already lists Queued and Proposed jobs on DRAINED_CHANNELS (steward, cli) with `read` set for Proposed. crates/athena-cli/src/drain.rs `run_job` (~1461) treats `item.read` as pre-read and skips the reader, and the outcome branch ~1580 handles `needs_accepting`. Only `athena go` (crates/athena-core/src/run/mod.rs ~376) dispatches a Proposed job with a capability. The producer must first find where a proposed, check-passing reading is actually passed over (for example the plan or readiness filtering in drain.rs around 3470-3500, or a reading check or a run-level acceptance gate for proposed jobs) and fix that root cause. If the base already lists these, it must show why the 42 are skipped.
- Existing tests in crates/athena-cli/tests/drain.rs use helpers like `planned`, `limits`, `waiting_proposal` and `passing`. New tests extend them, and the `needs_accepting` path is reused for the changed-scope case.
- No new record or data file is added under crates/, so the records allow-list is untouched. The change adds no refusal rule; it admits more readings to the batch, so there is no refusal-rate criterion.
- Running the 42 existing proposed requests is not part of the change; it only makes them eligible.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs has a deferred request (channel from `ask --later`) in state proposed whose latest reading passes the reading checks. The drain runs it. Check with `cargo test -p athena-cli --test drain`.
2. [behavior] A new test in crates/athena-cli/tests/drain.rs has a request re-read with a changed scope after a settings change. The drain does not run it, reports outcome `needs accepting`, counts no failure, and does not stop the batch. Check with `cargo test -p athena-cli --test drain`.
3. [behavior] A test shows a proposed deferred request whose latest reading fails a reading check is not run by the batch, and a mixed run of proposed and queued items goes in priority order, with the urgent one before the lower priority one. Check with `cargo test -p athena-cli`.
4. [content] `git diff` touches only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/steward.rs (only if selection lives there) and CHANGELOG.md. It shows no change to athena.toml, defaults.toml or .github/workflows, and no change to the reading checks themselves.
5. [content] The module docs in crates/athena-cli/src/drain.rs (the bullet 'A read request stays queued') state the new rule, and CHANGELOG.md has an entry for it.
6. [behavior] `cargo test`, `cargo clippy` and `cargo fmt --check` pass for the workspace.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the reading checks themselves
- Changing `athena go` or `athena ask` behavior
- Changing the rule that a changed-scope re-read needs a person
- Running the 42 existing proposed requests
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy are criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the drain running or leaving items, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The change is limited to drain item selection and its tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer must find where proposed readings are actually passed over rather than patching the symptom.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

A new test in crates/athena-cli/tests/drain.rs has a deferred request (channel from `ask --later`) in state proposed whose latest reading passes the reading checks. The drain runs it. Check with `cargo test -p athena-cli --test drain`.
A new test in crates/athena-cli/tests/drain.rs has a request re-read with a changed scope after a settings change. The drain does not run it, reports outcome `needs accepting`, counts no failure, and does not stop the batch. Check with `cargo test -p athena-cli --test drain`.
A test shows a proposed deferred request whose latest reading fails a reading check is not run by the batch, and a mixed run of proposed and queued items goes in priority order, with the urgent one before the lower priority one. Check with `cargo test -p athena-cli`.
`git diff` touches only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/steward.rs (only if selection lives there) and CHANGELOG.md. It shows no change to athena.toml, defaults.toml or .github/workflows, and no change to the reading checks themselves.
The module docs in crates/athena-cli/src/drain.rs (the bullet 'A read request stays queued') state the new rule, and CHANGELOG.md has an entry for it.
`cargo test`, `cargo clippy` and `cargo fmt --check` pass for the workspace.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
