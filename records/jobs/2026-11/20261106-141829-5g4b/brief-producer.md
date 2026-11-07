# Job 20261106-141829-5g4b: Retry stale jobs on the branch tip, keeping the candidate

## Request (verbatim)

> try again on 20261106-121449-49rw

## Reading (accepted by the user)

Retry of job 20261106-121449-49rw, which was dispatched but never verified and recorded no findings or candidate: redo the work from the base. `athena retry` on a stale job merges the kept candidate onto the tip. A clean merge is re-checked, re-verified and landed with no producer call; a conflict goes to a producer retry. The D10 alternative (d) reversal is appended as a new decision after D156, and the log guard is pinned. Audience: users running parallel jobs, and the verifier.

Assumptions:
- Job 20261106-121449-49rw has no candidate, no findings and no verdict. Nothing of its work is kept, so the work restarts from the base with the same scope as the earlier accepted reading.
- Code survey: I listed Stale occurrences across crates/athena-core/src/land.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/close.rs, crates/athena-cli/src/drain.rs and crates/athena-gate/src/landing_tests.rs. The entry point is the athena retry command in crates/athena-cli/src/main.rs, which reaches reading.rs and run/mod.rs. Landing is decided by Plan::Stale in land.rs plan(), where the verified fingerprint must equal the landed one. Re-verification on the merged tree must produce a fresh fingerprint. The change adds no record or data file, so check_record_file and AnyRecord are untouched. The properties are measured directly by cargo test cases that check the recorded calls, the landed fingerprint and the decisions.md guard in crates/athena-cli/src/decisions.rs.
- docs/decisions.md D10 (line 276) lists the left-out alternative (d); D156 (line 4269) is the last decision, so the new decision is appended after its last bullet.
- No refusal rule is tightened for readings. The log guard refuses mid-file inserts into decisions.md, which is not a reading refusal; expected 0% of newly refused readings.
- The outcome-check, behavior-evidence and verified defaults apply.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal rather than an error.
2. [behavior] Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and no planner or producer call is recorded for the retry.
3. [behavior] Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer run seeded from the candidate, and nothing lands without a new verification.
4. [behavior] Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree, and the existing land.rs Plan::Stale tests still pass (a landing whose verified fingerprint differs is refused).
5. [behavior] Run `cargo test`: a decisions.rs test shows the log guard fails with a message containing 'append at end of file' when an entry is inserted before the end of the pinned base, or when a new entry's id is not one greater than the previous highest. It passes on the real docs/decisions.md.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] `git diff -U0 -- docs/decisions.md` shows a single hunk of added lines after the last line of the base file, with no existing line changed. The new decision cites D10 by id, says it reverses the left-out alternative (d) 'Automatic re-verification on the new base', and gives the reason that stale landings now recur with parallel jobs.
8. [content] CHANGELOG.md has an entry describing only what the diff does, and README.md text saying stale jobs are refused is corrected.
9. [content] The retry help text in crates/athena-cli/src/main.rs and the stale reader brief in crates/athena-context/src/briefs.rs no longer call a stale job simply rejected. They state that the retry starts from the base when the candidate cannot be merged, and the brief test covers this.
10. [content] Only the listed files change, and no secrets or personal data are committed.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Automatic (drain-triggered) re-verification of stale jobs without an explicit athena retry
- Changing the staleness rules in land.rs plan() (what makes a job stale)
- Retrying other non-retryable states (landed, dropped, abandoned)
- Editing or annotating any existing line of docs/decisions.md or docs/decision-links.toml
- Prompt files and governance configuration (athena.toml, defaults.toml, workflows)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that stale jobs re-verify and land, or go to a producer retry, rather than that code exists.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Each behavior criterion names a cargo command and the observable result.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The scope was recovered from the earlier accepted reading and the repository.
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

32 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-141829-5g4b/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-141829-5g4b/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal rather than an error.
Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and no planner or producer call is recorded for the retry.
Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer run seeded from the candidate, and nothing lands without a new verification.
Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree, and the existing land.rs Plan::Stale tests still pass (a landing whose verified fingerprint differs is refused).
Run `cargo test`: a decisions.rs test shows the log guard fails with a message containing 'append at end of file' when an entry is inserted before the end of the pinned base, or when a new entry's id is not one greater than the previous highest. It passes on the real docs/decisions.md.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
`git diff -U0 -- docs/decisions.md` shows a single hunk of added lines after the last line of the base file, with no existing line changed. The new decision cites D10 by id, says it reverses the left-out alternative (d) 'Automatic re-verification on the new base', and gives the reason that stale landings now recur with parallel jobs.
CHANGELOG.md has an entry describing only what the diff does, and README.md text saying stale jobs are refused is corrected.
The retry help text in crates/athena-cli/src/main.rs and the stale reader brief in crates/athena-context/src/briefs.rs no longer call a stale job simply rejected. They state that the retry starts from the base when the candidate cannot be merged, and the brief test covers this.
Only the listed files change, and no secrets or personal data are committed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
