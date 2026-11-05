# Job 20261105-093606-199q: Find and remove the cause of the longer checks stage

## Request (verbatim)

> Priority: high
> 
> Throughput slowdown: checks grew from 0.2 to 0.6 min per landed job
> 
> Athena's throughput review found that the median minutes per landed job rose from 464.5 to 616.3, more than 25%, since the previous throughput review. Find why the stage that grew most now takes longer and remove the cause.
> 
> Stage that grew most: checks, a median of 0.6 minutes per landed job now, 0.2 before.
> Stages whose median minutes grew, in minutes per landed job before to now: checks 0.2 to 0.6.
> 
> The medians are over the newest landed jobs that Athena measured. The status section of athena status prints the same stage medians.
> 
> Acceptance:
> - The cause of the longer checks stage is found and stated in the change or in a note under docs/design/
> - The change shortens that stage, and the next throughput review shows the median minutes per landed job falling
> 
> Files: crates/, docs/design/

## Reading (accepted by the user)

Athena's checks stage median rose from 0.2 to 0.6 min per landed job. Find the cause in the checks path (routing, CI result reuse, baseline lookup, executor), state it in a docs/design note, and fix it so the stage gets shorter. The audience is the maintainer reading the next throughput review.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs (SnapExec runs one routed check after a one-time disk gate; baseline lookup uses RefuseExec and WaitsWhileRunning, with a bounded CI read loop in crates/athena-core/src/ci_result.rs). Related, not yet read: crates/athena-core/src/routing.rs, crates/athena-core/src/status.rs (stage medians). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- The exact cause is not known from the request. The producer must find it by reading the checks path (what a check runs, CI result reuse versus local runs, baseline wait bound, snapshot restore after each check) and recent changes to it, and must measure it directly, not through a stand-in.
- The change is a single focused fix plus a short note in docs/design/ stating the cause with evidence; no refusal rule is added or tightened.
- The next throughput review cannot be run in the job; the verifier checks that the change removes work or waiting from the checks path, not the future median.
- Governance files (athena.toml, defaults.toml, workflows) are not expected to change.

## Acceptance criteria

1. [content] A note under docs/design/ (or the change's commit text/doc comment) states the cause of the longer checks stage, citing the specific crates/ code path and the evidence (measured or labeled estimate).
2. [content] The diff to crates/ removes or bounds the identified cause (extra check run, wait, repeated work or snapshot restore) in the checks path, rather than a symptom such as raising a limit.
3. [behavior] A new or updated test in crates/ exercises the fixed behavior, failing on the old behavior; run `cargo test` and it passes.
4. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] Existing checks behavior is preserved: the same routed checks still run and failures still hold or reject the job; existing tests in crates/athena-core/src/run/tests.rs still pass.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/checks-stage-slowdown.md (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing other stages (planning, producer, verification) that did not grow
- Changing athena.toml, defaults.toml or CI workflow files
- Changing how throughput medians are computed or reported

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fix is Rust code in crates/.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test and clippy to pass with a test for the fix.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause must be found and removed, not worked around.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test criterion checks the fixed behavior by running cargo test.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The goal is a shorter checks stage, so the cause is stated with evidence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused fix plus one note.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design note is written for an outside reader.
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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-093606-199q/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-093606-199q/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A note under docs/design/ (or the change's commit text/doc comment) states the cause of the longer checks stage, citing the specific crates/ code path and the evidence (measured or labeled estimate).
The diff to crates/ removes or bounds the identified cause (extra check run, wait, repeated work or snapshot restore) in the checks path, rather than a symptom such as raising a limit.
A new or updated test in crates/ exercises the fixed behavior, failing on the old behavior; run `cargo test` and it passes.
`cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
Existing checks behavior is preserved: the same routed checks still run and failures still hold or reject the job; existing tests in crates/athena-core/src/run/tests.rs still pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
