# Job 20261106-062207-cbcc: Let athena retry re-verify a stale job's kept candidate on the new tip

## Request (verbatim)

> try again on 20261106-041052-wzy7

## Reading (accepted by the user)

Retry of job 20261106-041052-wzy7: `athena retry` accepts a stale job, merges its kept candidate onto the tip, and on a clean merge re-checks, replays, re-verifies and lands it with no producer call; a conflict starts a producer retry from the candidate. Audience: users running parallel jobs, and the verifier. The reversal of alternative (d) is recorded as a NEW appended decision, not an edit to an existing line.

Assumptions:
- Code survey: entry is `Athena::retry` in crates/athena-core/src/reading.rs (~line 1103), then dispatch and resume_candidate in crates/athena-core/src/run/mod.rs, with close in run/close.rs. Staleness and the fingerprint-equality bail are in `plan()` in crates/athena-core/src/land.rs; the merge uses crates/athena-git/src/lib.rs. Retry CLI help is in crates/athena-cli/src/main.rs. Existing retry tests are in crates/athena-cli/tests/lifecycle.rs and crates/athena-cli/tests/retry.rs; unit tests in crates/athena-core/src/run/tests.rs. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The producer-call count is measured directly from the fake agent's recorded calls.
- Job 20261106-041052-wzy7 findings addressed: (1) major, in-place annotation of docs/decisions.md line 297: dropped. The log is append-only and hash-pinned through D121 by crates/athena-cli/src/decisions.rs, so the reversal goes in a new appended decision that cites D10 by id; no existing line is edited. (2) README.md change: kept and now a listed deliverable. (3) reader brief still says 'rejected' for a stale retry: fixed in crates/athena-context/src/briefs.rs (the earlier crates/athena-core/src/briefs.rs path does not exist) and the pinned reading.rs test is updated. (4) retry help first line in main.rs and CHANGELOG README wording: fixed so the help covers stale jobs and CHANGELOG states only what the diff does.
- The prior candidate's work is kept as the starting point: CHANGELOG.md, README.md, main.rs, lifecycle.rs, land.rs, reading.rs, run/mod.rs, athena-git lib.rs, docs/decisions.md. A new retry starts from the base, so the producer redoes these with the fixes above.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error.
2. [behavior] Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and zero producer calls are recorded for the retry.
3. [behavior] Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer retry seeded from the kept candidate, and nothing lands without a new verification.
4. [behavior] Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree, and a landing whose verified fingerprint differs from the landed content is still refused (existing land.rs Plan::Stale tests still pass).
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass, including the decisions test the_decision_log_is_well_formed_and_only_appended_to.
6. [content] Grep docs/decisions.md: a NEW decision appended at the end states it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision (D10), gives the reason that stale landings now recur with parallel jobs, and cites D10 by id. `git diff` shows no existing line of docs/decisions.md changed, only additions.
7. [content] CHANGELOG.md has an entry describing only what the diff does: retry accepts a stale job, merges its candidate onto the tip, re-verifies and lands on a clean merge without a producer run, and runs the producer from the candidate on conflict. README.md text saying stale jobs are refused is corrected.
8. [content] The retry help text in crates/athena-cli/src/main.rs and the reader brief for a stale retry (crates/athena-context/src/briefs.rs) no longer describe a stale job as simply 'rejected' or as starting from the base; Grep shows the updated wording and the updated test.
9. [content] Only the listed files change; no secrets or personal data are committed.

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
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Automatic (drain-triggered) re-verification of stale jobs without an explicit athena retry
- Changing the staleness rules in land.rs plan() (what makes a job stale)
- Retrying other non-retryable states (landed, dropped, abandoned)
- Editing or annotating any existing line of docs/decisions.md or docs/decision-links.toml
- Prompt files and governance configuration

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the retry landing or retrying behavior by running cargo test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert zero producer calls and the verified fingerprint.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The decision is appended to the log rather than editing the pinned line.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, CHANGELOG and README are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261106-041052-wzy7 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/decisions.md:297; crates/athena-cli/src/decisions.rs:200): Criterion 6's clause 'the original line 297 is annotated as reversed' is not met. It cannot be met without breaking the append-only check on the decision log. docs/decisions.md:297 is unchanged. crates/athena-cli/src/decisions.rs:200-204 pins the length and SHA-256 of every byte through D121, and `the_decision_log_is_well_formed_and_only_appended_to` enforces it. Editing D10 would fail `cargo test` (criterion 5). D157 instead records the reversal and tells readers to read D10's lines with it. The candidate needs no change; the criterion should drop the in-place annotation, or allow a forward reference or a decision-links.toml `amended` link (D10 by D157) instead.
> major: The reading does not match the request, so passing criteria do not show the request was met Criterion 6 adds a requirement the request did not make: annotate D10's line 297 in place. The repository forbids that edit by a test: the decision log is append-only, and the bytes through D121, which include D10, are hash-pinned. So criterion 6's annotation clause and criterion 5's 'cargo test passes' cannot both hold. The request only asks that docs/decisions.md record the change as reversing alternative (d), with the reason, and D157 does that. The planned demonstration repeated the impossible 'Reversed by D157' edit. The criterion should be amended, for example to a forward reference in D157 or an `amended` link in docs/decision-links.toml.

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

31 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-062207-cbcc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-062207-cbcc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error.
Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and zero producer calls are recorded for the retry.
Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer retry seeded from the kept candidate, and nothing lands without a new verification.
Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree, and a landing whose verified fingerprint differs from the landed content is still refused (existing land.rs Plan::Stale tests still pass).
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass, including the decisions test the_decision_log_is_well_formed_and_only_appended_to.
Grep docs/decisions.md: a NEW decision appended at the end states it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision (D10), gives the reason that stale landings now recur with parallel jobs, and cites D10 by id. `git diff` shows no existing line of docs/decisions.md changed, only additions.
CHANGELOG.md has an entry describing only what the diff does: retry accepts a stale job, merges its candidate onto the tip, re-verifies and lands on a clean merge without a producer run, and runs the producer from the candidate on conflict. README.md text saying stale jobs are refused is corrected.
The retry help text in crates/athena-cli/src/main.rs and the reader brief for a stale retry (crates/athena-context/src/briefs.rs) no longer describe a stale job as simply 'rejected' or as starting from the base; Grep shows the updated wording and the updated test.
Only the listed files change; no secrets or personal data are committed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
