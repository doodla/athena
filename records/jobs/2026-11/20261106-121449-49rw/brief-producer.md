# Job 20261106-121449-49rw: Retry stale jobs on the new tip; append the decision at log end

## Request (verbatim)

> try again on 20261106-062207-cbcc

## Reading (accepted by the user)

Retry of job 20261106-062207-cbcc: redo the stale-job retry (`athena retry` merges the kept candidate onto the tip, re-verifies and lands with no producer call on a clean merge, producer retry on conflict), this time appending the reversal of D10's alternative (d) as a new decision after D156's last bullet, and pinning the log guard so a mid-file insert fails. Audience: users running parallel jobs, and the verifier.

Assumptions:
- Job 20261106-062207-cbcc is addressed as follows. Its candidate was discarded, so the producer redoes the whole behavior (all 13 files it changed are deliverables below, work kept in intent, not on disk). Major finding (D157 inserted inside D156's last Provenance bullet): fixed by appending the new decision after the final line of the base file and adding a guard that fails such inserts. Minor finding (reader brief always says the candidate will be merged although stale_seed can fall back to a retry from the base): fixed by wording the brief to say the candidate is merged when it is still usable and otherwise the retry starts from the base, with the test updated. Nit (stale shared CARGO_TARGET_DIR artifact): an environment artifact, not a candidate defect; no change.
- Code survey: the guard lives in crates/athena-cli/src/decisions.rs. check_log pins the first PINNED_LEN bytes (through D121) by SHA-256, so the insert inside D156 passed. The fix raises PINNED_LEN and PINNED_SHA256 to cover the base log through D156's last byte, changes the failure message to say 'append at end of file', and adds a check that a new entry's id is exactly one greater than the previous highest. The producer measures the base length and hash from docs/decisions.md as it stands, before appending. Stale-retry code: crates/athena-core/src/reading.rs (earlier_attempt_with, which refuses stale jobs today), crates/athena-core/src/run/mod.rs (stale_seed), run/close.rs, crates/athena-core/src/land.rs (Plan::Stale), crates/athena-git/src/lib.rs (merge_tree_conflicts), crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs (reader_brief). No new record or data file is added, so no records allow-list change is needed.
- The decisions guard is a test-time check in decisions.rs; 'the producer's local checks' means cargo test, which already runs it, so no athena.toml change is made. No new refusal rule is added for readings, so no refusal-rate criterion applies.
- The new decision gets the next id after the highest existing one, with a Provenance bullet, and cites D10 by id.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error.
2. [behavior] Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and no planner or producer call is recorded for the retry.
3. [behavior] Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer run seeded from the merged conflict, and nothing lands without a new verification.
4. [behavior] Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree, and the existing land.rs Plan::Stale tests still pass (a landing whose verified fingerprint differs is refused).
5. [behavior] Run `cargo test`: a decisions.rs test shows the log guard fails with a message containing 'append at end of file' when an entry is inserted before the end of the pinned base (including inside D156's last bullet) or when a new entry's id is not one greater than the previous highest, and passes on the real docs/decisions.md.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] Run `git diff -U0 -- docs/decisions.md`: a single hunk consisting only of added lines, located after the last line of the base file (after D156's final bullet); no existing line changed. The new decision states it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of D10, gives the reason that stale landings now recur with parallel jobs, and cites D10 by id.
8. [content] CHANGELOG.md has an entry describing only what the diff does; README.md text saying stale jobs are refused is corrected.
9. [content] The retry help text in crates/athena-cli/src/main.rs and the stale reader brief in crates/athena-context/src/briefs.rs no longer call a stale job simply rejected, and state that the retry starts from the base when the candidate cannot be merged; the brief test covers this.
10. [content] Only the listed files change; no secrets or personal data are committed.

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

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus docs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for clean-merge, conflict, and log guard; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The guard is fixed so misplaced decision entries fail in cargo test, not only at verification.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and git diff on the behavior.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier exercises the tests.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Append-only rule enforced by the pinned-prefix test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261106-062207-cbcc raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/decisions.md:4310-4353): D157 was inserted in the middle of D156's last bullet instead of appended at the end of the log, so D156's closing sentence now reads as part of D157 `git diff HEAD~1 -U0 -- docs/decisions.md` shows `@@ -4309,0 +4310,42 @@`. Base lines 4310-4311 ('  Not measured: how many future changes will touch the two D08 paths, and whether authors will add the link or' / '  work around it.') continue D156's Provenance bullet. In the candidate they sit at lines 4352-4353, after D157's own 'Not measured: how often stale landings occur…'. D156 is changed in meaning even though no line is edited, which the 'append, never edit' rule exists to prevent. The pinned-prefix test only covers D121 and earlier, so it passes anyway. Fix: move D157 after line 4311 of the base file.

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

- path: <state>/jobs/20261106-121449-49rw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-121449-49rw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error.
Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and no planner or producer call is recorded for the retry.
Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer run seeded from the merged conflict, and nothing lands without a new verification.
Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree, and the existing land.rs Plan::Stale tests still pass (a landing whose verified fingerprint differs is refused).
Run `cargo test`: a decisions.rs test shows the log guard fails with a message containing 'append at end of file' when an entry is inserted before the end of the pinned base (including inside D156's last bullet) or when a new entry's id is not one greater than the previous highest, and passes on the real docs/decisions.md.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
Run `git diff -U0 -- docs/decisions.md`: a single hunk consisting only of added lines, located after the last line of the base file (after D156's final bullet); no existing line changed. The new decision states it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of D10, gives the reason that stale landings now recur with parallel jobs, and cites D10 by id.
CHANGELOG.md has an entry describing only what the diff does; README.md text saying stale jobs are refused is corrected.
The retry help text in crates/athena-cli/src/main.rs and the stale reader brief in crates/athena-context/src/briefs.rs no longer call a stale job simply rejected, and state that the retry starts from the base when the candidate cannot be merged; the brief test covers this.
Only the listed files change; no secrets or personal data are committed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
