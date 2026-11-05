# Job 20261105-001940-nvyd: Let the drain run proposed deferred requests whose reading passes checks

## Request (verbatim)

> try again on 20261105-001814-tw8d

## Reading (accepted by the user)

Retry of job 20261105-001814-tw8d (dispatched, never verified, no findings, no candidate, so nothing to fix or keep): `athena drain` should run a request queued by `ask --later`, the Steward or Hestia whose latest reading is proposed and passes every reading check, in priority order. A changed-scope re-read after a settings change still waits for a person. Audience: the operator relying on unattended batches.

Assumptions:
- Job 20261105-001814-tw8d left no candidate and no findings, so its work is neither kept nor dropped. Its scope is reused: drain.rs, tests/drain.rs, steward.rs, CHANGELOG.md. Its out_of_scope items are carried over unchanged.
- Code survey: I read crates/athena-cli/src/drain.rs and crates/athena-core/src/steward.rs. In steward.rs, `queued_work` already lists Queued and Proposed jobs on DRAINED_CHANNELS (steward and cli) and marks proposed ones `read`. In drain.rs, `queue()` builds items from `approved_work`, `queued_work` and `interrupted_work`. `run_job` skips the second reading for a pre-read item. `run.needs_accepting` maps to outcome `needs accepting`, which is not a failure. `athena-core/src/run/mod.rs` `go` takes `AcceptedBy::Flag` from the drain, so a re-read with the same scope is accepted at once and a changed scope is not. The entry point is `drain()` in drain.rs. Selection and ordering run through the plan, then `pick_ahead`. No records or data files are added, so `check_record_file`, `AnyRecord` and the round-trip test are not involved. The measurement is the drain's own report lines in crates/athena-cli/tests/drain.rs.
- The listing code already appears to include proposed requests, so the producer must first find where a proposed, deferred, checks-passing request is actually skipped. Candidates are plan selection, the reading-check gate before the run, and priority ordering of read items. It then fixes that root cause, not the listing.
- This change does not add a refusal rule. It runs more items and refuses none, so no newly-refused rate applies.
- Changes to athena.toml, defaults.toml or .github/workflows are not needed. They stay untouched and are listed in out_of_scope.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs has a deferred request (channel from `ask --later`) in state proposed whose latest reading passes the reading checks. The drain runs it and reports it as passed. Check with `cargo test -p athena-cli --test drain`.
2. [behavior] A new test in crates/athena-cli/tests/drain.rs has a request re-read with a changed scope after a settings change. The drain does not run it, reports outcome `needs accepting`, counts no failure, and does not stop the batch. Check with `cargo test -p athena-cli --test drain`.
3. [behavior] A test shows a proposed deferred request whose latest reading fails a reading check is not run by the batch. Another test shows a mixed run of proposed and queued items goes in priority order, with the urgent one before the lower priority one. Check with `cargo test -p athena-cli --test drain`.
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

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain tests, and cargo test, clippy and fmt pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the drain running or leaving the item, through tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer finds where proposed items are skipped and fixes it there.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain selection.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the CHANGELOG entry are written for an outside reader.
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

A new test in crates/athena-cli/tests/drain.rs has a deferred request (channel from `ask --later`) in state proposed whose latest reading passes the reading checks. The drain runs it and reports it as passed. Check with `cargo test -p athena-cli --test drain`.
A new test in crates/athena-cli/tests/drain.rs has a request re-read with a changed scope after a settings change. The drain does not run it, reports outcome `needs accepting`, counts no failure, and does not stop the batch. Check with `cargo test -p athena-cli --test drain`.
A test shows a proposed deferred request whose latest reading fails a reading check is not run by the batch. Another test shows a mixed run of proposed and queued items goes in priority order, with the urgent one before the lower priority one. Check with `cargo test -p athena-cli --test drain`.
`git diff` touches only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/steward.rs (only if selection lives there) and CHANGELOG.md. It shows no change to athena.toml, defaults.toml or .github/workflows, and no change to the reading checks themselves.
The module docs in crates/athena-cli/src/drain.rs (the bullet 'A read request stays queued') state the new rule, and CHANGELOG.md has an entry for it.
`cargo test`, `cargo clippy` and `cargo fmt --check` pass for the workspace.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
