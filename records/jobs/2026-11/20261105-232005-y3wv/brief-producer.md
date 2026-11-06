# Job 20261105-232005-y3wv: Replay new refusal rules over past readings inside the producer loop

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Nothing requires a new refusal or classification rule to be measured against real past requests befo
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: new-refusal-rule-not-replayed-against-real-requests (20261028-034817-rhs5)
> Cause: Nothing requires a new refusal or classification rule to be measured against real past requests before it lands. Acceptance criteria and the verifier test that the rule exists and fires on its intended cases. Neither measures how often it w...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261028-034806-d8g0 (job 20261027-054211-a2n5); prevention it proposed: Add a check that runs for any change to an admission or refusal rule. It replays the new rule over the recorded past requests (accepted ones included) and reports the share it would newly refuse. The job fails if that share exceeds a set th...
> Introspection 20261105-231947-72g2 (job 20261105-093556-zyt3); prevention it proposed: Run the replay of past accepted readings as a check inside the producer's loop. It would give the producer the list of newly refused readings and the measured share. Cap the share with a numeric limit set in the request or config. Make the ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the replay of past accepted readings (`replay_stage` in run/mod.rs) runs only after the producer finishes, as a hold at verification. The producer never sees the newly refused list or the measured share, so a rule such as job 20261105-093556-zyt3's bug-fix check lands at 18% newly refused and is rejected after the work is done. Fix: run the replay as a check in the producer's loop and feed the list and share back to it. Audience: Athena's operator and the producer. Job 20261027-054211-a2n5 landed and is kept; job 20261105-093556-zyt3's candidate is dropped.

Assumptions:
- Code survey: I read crates/athena-core/src/run/mod.rs (`produce_and_verify`, `replay_stage`, `replay_run`, `replay_checkers`), crates/athena-core/src/reading.rs (`replay_cases`, `replay`, `REPLAY_HOLD_PERCENT`, `replay_declaration_of`, `check_governance_coverage`, `refusing_check`) and crates/athena-core/src/verify.rs (the replay blocker). Today the replay runs once after the checks pass and before verification, and it never fails the job on its own. No record or data file is added under crates/, so `check_record_file`, `AnyRecord` and the round-trip test are not affected. The share is measured directly: base and candidate `check-reading` builds are run over the newest accepted readings, and the newly refused ones are counted.
- Job 20261027-054211-a2n5 (landed, verdict pass): its governance-coverage work in reading.rs, admission.rs, CHANGELOG.md and docs/architecture.md is kept as it is. Its findings are minor: broad keywords, and a keyword table test that has its own phrasings. They are not the cause here, so this job does not address them.
- Job 20261105-093556-zyt3 (rejected): its candidate is dropped, including `check_root_cause_diagnosis`, the changes to briefs.rs, refusal_corpus.rs and run/mod.rs, and its CHANGELOG and architecture edits. Its findings are addressed by the generic mechanism: the producer now sees the newly refused list and the measured share during its own loop, so false positives such as 'the fix' in 'fails without the fix' show up before verification. That bug-fix check is not re-landed here.
- The numeric limit is the existing one: `REPLAY_HOLD_PERCENT` (10) or the share the criteria declare plus 5 points. No new config key, so athena.toml stays unchanged.
- The in-loop replay reuses `replay_cases`, `replay` and `replay_checkers`, and runs after the producer's checks pass. When it exceeds the limit, the list and share go back to the producer as a send-back, within the existing retry budget. When the replay cannot run, it says `no replay: <reason>` and does not count as a pass.
- This job adds no new refusal rule, so the 'expected N% of newly refused readings' criterion does not apply.

## Acceptance criteria

1. [content] The root cause is named in the change description and CHANGELOG.md: nothing makes the producer measure a new or tightened refusal rule against past accepted readings while it works. The replay (`replay_stage` in crates/athena-core/src/run/mod.rs) runs once after the producer finishes, as a verification hold. Checked by reading the diff.
2. [behavior] The test(s) added in crates/athena-core/src/run/tests.rs (the producer-loop replay tests) fail on the base tree and pass with the change. They use a fake candidate checker that newly refuses more than the limit of a corpus of recorded accepted readings. They assert that the producer is sent back, that its brief names the newly refused job ids and the measured percent, and that the loop ends once the share is under the limit. Run `cargo test -p athena-core`.
3. [behavior] A test in crates/athena-core/src/run/tests.rs or crates/athena-core/src/reading.rs shows that the in-loop replay respects the declared share and the 10% default (`REPLAY_HOLD_PERCENT`): exactly the limit passes, one over is sent back. It also shows that an unrunnable replay reports `no replay: <reason>` and is not treated as a pass. Run `cargo test -p athena-core`.
4. [behavior] A test shows that a job whose reading adds no refusal rule, or whose replay has no history, is not sent back, and that an empty corpus is reported as `no replay` and not as success.
5. [content] docs/architecture.md and CHANGELOG.md describe the replay as a check inside the producer loop, with its limit, and say the post-hoc verification hold stays as the backstop.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass from the repository root.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)
- crates/athena-core/src/run/send_back.rs (modify)

## Out of scope

- Re-landing `check_root_cause_diagnosis` or any other new refusal rule from job 20261105-093556-zyt3: its candidate files (crates/athena-core/tests/refusal_corpus.rs and its other edits) are dropped
- Reworking job 20261027-054211-a2n5's governance keyword table or its crates/athena-core/src/admission.rs changes: that work is kept as it is
- Editing athena.toml, defaults.toml, workflows or other governance files
- Editing prompt files (the reader and verifier prompts have no token headroom)
- Changing the replay hold thresholds or the verifier's blocker in crates/athena-core/src/verify.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests, plus fmt, clippy and test passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is where the producer's loop lacks the replay, not in one job's rule.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the producer is sent back with the list and share.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty corpus or an unrunnable replay reports `no replay`, never a pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The limit is enforced in the producer loop by code.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs cargo build, test, clippy and fmt --check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG written for an outside reader.
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

29 more qualifying classes are left out.

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

- path: <state>/jobs/20261105-232005-y3wv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-232005-y3wv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The root cause is named in the change description and CHANGELOG.md: nothing makes the producer measure a new or tightened refusal rule against past accepted readings while it works. The replay (`replay_stage` in crates/athena-core/src/run/mod.rs) runs once after the producer finishes, as a verification hold. Checked by reading the diff.
The test(s) added in crates/athena-core/src/run/tests.rs (the producer-loop replay tests) fail on the base tree and pass with the change. They use a fake candidate checker that newly refuses more than the limit of a corpus of recorded accepted readings. They assert that the producer is sent back, that its brief names the newly refused job ids and the measured percent, and that the loop ends once the share is under the limit. Run `cargo test -p athena-core`.
A test in crates/athena-core/src/run/tests.rs or crates/athena-core/src/reading.rs shows that the in-loop replay respects the declared share and the 10% default (`REPLAY_HOLD_PERCENT`): exactly the limit passes, one over is sent back. It also shows that an unrunnable replay reports `no replay: <reason>` and is not treated as a pass. Run `cargo test -p athena-core`.
A test shows that a job whose reading adds no refusal rule, or whose replay has no history, is not sent back, and that an empty corpus is reported as `no replay` and not as success.
docs/architecture.md and CHANGELOG.md describe the replay as a check inside the producer loop, with its limit, and say the post-hoc verification hold stays as the backstop.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass from the repository root.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
