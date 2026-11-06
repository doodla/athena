# Job 20261106-041052-wzy7: Let athena retry re-verify a stale job's kept candidate on the new tip

## Request (verbatim)

> Let a stale job be tried again on the current branch tip, keeping its candidate.
> 
> With two jobs running at once, a job that passed checks, replay and verification can go stale because the other job landed first on content it governs. Today a stale job can only be asked again from scratch, which repeats the whole producer run. docs/decisions.md says automatic re-verification on the new base was left out until stale landings recur; they now recur with parallel jobs.
> 
> athena retry accepts a stale job. The retry merges the kept candidate onto the current branch tip. If the merge is clean, it re-runs checks, replay and verification on the merged result and lands it as usual; it does not re-run the producer. If the merge conflicts, the retry runs the producer from the kept candidate as a normal retry does.
> 
> Acceptance:
> - athena retry on a stale job no longer errors
> - a test: a candidate that merges cleanly onto a moved branch is re-checked and re-verified on the merged tree and lands, with no producer call recorded
> - a test: a candidate that conflicts with the moved branch goes to a producer retry
> - the landing still requires the verified fingerprint to match what lands (re-verification produces the new fingerprint)
> - docs/decisions.md records the change as reversing the left-out alternative (d), with the reason
> - CHANGELOG states only what the diff does
> 
> Files changed: the retry command, the landing path for stale jobs, tests, docs/decisions.md, CHANGELOG.md.

## Reading (accepted by the user)

Make `athena retry` accept a stale job. It merges the kept candidate onto the current branch tip. A clean merge re-runs checks, replay and verification on the merged tree and lands it with the new fingerprint, with no producer call. A conflict starts a producer retry seeded from the kept candidate. Audience: users running parallel jobs, and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (Athena::retry at ~1103 calls ask with 'try again on <id>'; qualifies_for_tree_retry and tree_retry_seed at ~1645-1758 only accept Rejected jobs, so a Stale job is not seeded; resumable_job/retries_by_job treat Stale as an unrun state), crates/athena-core/src/land.rs (plan() returns Plan::Stale/Tree/OtherBranch; landing requires P ancestor of H, same classification, and Verified fingerprint match), and crates/athena-core/src/run/close.rs (Stale outcome written at ~289-312; retry_landing at ~451 re-runs plan on a blocked job and bails on Stale). Behavior is measured by existing run tests (crates/athena-core/src/run/tests.rs) and CLI tests (crates/athena-cli/tests/lifecycle.rs, drain.rs) using a fake producer whose calls are counted.
- Existing code: the records for a stale job already hold its candidate commit and base; the new path reuses plan(), Verified::from_record, fingerprint, the tree-retry seeding in reading.rs and the checks/replay/verifier stages in crates/athena-core/src/run/. The merge uses the git layer's merge-tree in crates/athena-git; no new crate is added (buy-first).
- Where ask refuses non-retryable states, the Stale state is added to the accepted ones. The merge-clean/conflict decision is made once on the current tip, before any producer call; the merged tree gets a fresh candidate and a fresh verification record, and the fingerprint is recomputed, so the landing check still compares verified to landed content.
- No new record type or schema change is expected; existing candidate, verification and outcome records are reused (so the records allow-list and AnyRecord need no change). If one is needed, add it to AnyRecord with a round-trip test.
- This adds no refusal rule; it removes one (stale retry no longer errors). No replay refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) are not changed.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error (the stale-state refusal in ask/retry is gone).
2. [behavior] Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and the producer call count recorded for the retry is zero.
3. [behavior] Run `cargo test`: a test where the candidate conflicts with the moved branch shows the retry goes to a producer retry seeded from the kept candidate (one producer call recorded, nothing lands without a new verification).
4. [behavior] Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree (not the original candidate's), and a landing whose verified fingerprint differs from the landed content is still refused.
5. [behavior] Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including existing land.rs tests for Plan::Stale.
6. [content] docs/decisions.md records a new decision (or a revision) stating that it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision (the one at ~line 292), with the reason that stale landings now recur with parallel jobs, and the original line 297 is annotated as reversed.
7. [content] CHANGELOG.md has an entry describing only what the diff does: athena retry accepts a stale job, merges its candidate onto the tip, re-verifies and lands on a clean merge without a producer run, and runs the producer from the candidate on conflict.
8. [content] Only the retry command, the stale landing path, tests, docs/decisions.md and CHANGELOG.md change; no secrets or personal data are committed.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Automatic (drain-triggered) re-verification of stale jobs without an explicit athena retry
- Changing the staleness rules in land.rs plan() (what makes a job stale)
- Retrying other non-retryable states (landed, dropped, abandoned)
- Prompt files and governance configuration

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for clean-merge landing and conflict retry; cargo test and clippy pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test as the way to exercise it.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the producer call count, the landing and the fingerprint, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The stale retry path is changed in the retry and landing code, not by a workaround.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the listed files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a stale job lands re-verified without a producer run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The fingerprint-match landing rule stays enforced by code and is tested.
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
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

30 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-041052-wzy7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-041052-wzy7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error (the stale-state refusal in ask/retry is gone).
Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and the producer call count recorded for the retry is zero.
Run `cargo test`: a test where the candidate conflicts with the moved branch shows the retry goes to a producer retry seeded from the kept candidate (one producer call recorded, nothing lands without a new verification).
Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree (not the original candidate's), and a landing whose verified fingerprint differs from the landed content is still refused.
Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including existing land.rs tests for Plan::Stale.
docs/decisions.md records a new decision (or a revision) stating that it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision (the one at ~line 292), with the reason that stale landings now recur with parallel jobs, and the original line 297 is annotated as reversed.
CHANGELOG.md has an entry describing only what the diff does: athena retry accepts a stale job, merges its candidate onto the tip, re-verifies and lands on a clean merge without a producer run, and runs the producer from the candidate on conflict.
Only the retry command, the stale landing path, tests, docs/decisions.md and CHANGELOG.md change; no secrets or personal data are committed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
