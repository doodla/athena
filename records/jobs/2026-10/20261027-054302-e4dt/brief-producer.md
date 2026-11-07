# Job 20261027-054302-e4dt: Check finished diffs for lost structure before verification

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Nothing in the job compares the finished diff against what was intended. A prompt file is edited as
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: edit-side-effect-and-unverified-claim-caught-only-at-final-verdict (20261015-141828-4prp)
> Cause: Nothing in the job compares the finished diff against what was intended. A prompt file is edited as free text, and no check confirms that only the intended lines changed or that the structure survived (each instruction still its own bullet)...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261015-141817-npx3 (job 20261015-095214-y8kr); prevention it proposed: Add a pre-verification check that runs on every job touching prompts or docs. (1) For prompt files, show the diff and fail if any changed hunk modifies lines outside the intended edit, or if the bullet or line structure changes where no cha...
> Introspection 20261025-095304-sr9e (job 20261025-034452-wqp8); prevention it proposed: Add a code check that runs before handover and again in the repo tests. It parses docs/decisions.md into entries and requires three things. (1) Each entry has the required fields, including a Provenance bullet. (2) Entry ids are unique and ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the pre-verification mirror checks (crates/athena-core/src/mirrors.rs, run via mirror_drift_in at the producer step) never compare the finished diff of a prompt file or docs/decisions.md with the intended edit, so a joined bullet (job 20261015-095214-y8kr) or a decision inserted inside another (job 20261025-034452-wqp8) reached the final verdict. Add structure rules to the mirror registry so both are found at the producer step, for the whole class of prompt and decision-log files.

Assumptions:
- Code survey: crates/athena-core/src/mirrors.rs (REGISTRY, Rule, Source, check_entry, check_all, append_only_problem), crates/athena-core/src/run/producer.rs (mirror_drift_in at ~line 357 runs on the producer's work tree and sends the change back), crates/athena-core/src/run/mod.rs (decision gate at ~1869, before PR, checks and verifier), crates/athena-core/src/run/checks.rs (decision_gate, unreconciled_decisions). The entry point is the producer loop, then the candidate gate; the mirror check runs before the verifier.
- This change adds no record or data file under crates/, so check_record_file, AnyRecord and the records round-trip test are not involved.
- The property is measured directly: the new rules run over base and new file text (prompt files under crates/*/prompts/*.md and docs/decisions.md) and return Drift findings; unit tests feed them the actual failing shapes from both jobs.
- Prompt rule (job 20261015-095214-y8kr): a base bullet line that no longer starts its own line in the new text, or a line with a second '- word:' bullet marker mid-line, is a finding; this fixes finding 1 (joined bullet) at its origin. Findings 2 and 3 (untrue CHANGELOG claim, unexplained trimming) are not machine-checkable and are out of scope; the old candidate's files are dropped, not kept.
- Decision-log rule (job 20261025-034452-wqp8): every '## D<n>' entry has a Provenance bullet, ids are unique, and an entry's Provenance sits inside that entry; this fixes the major D122-inside-D121 finding at its origin. Its minor findings (untested urgent_reason parse path, unlisted fixture files) belong to the priority feature, which this change does not redo; that candidate's 18 files are dropped.
- Audience: Athena's producer and verifier. Prompt files are not edited, because reader (3 left) and verifier (2 left) have no token headroom.
- Whether existing docs/decisions.md entries all satisfy the new rule is unknown; the rule should judge only entries the diff touches, so legacy entries do not newly fail.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core mirrors`: a unit test feeds the reader.md shape from job 20261015-095214-y8kr (two bullets joined as '...suffice.- deliverables: ...') and gets a Drift naming the file and the bullet; the same test fails when the new rule is removed from check_entry.
2. [behavior] Run `cargo test -p athena-core mirrors`: a unit test feeds a docs/decisions.md where D122 is inserted inside D121, leaving D121 without its Provenance bullet, and gets a finding naming D121; a well-formed appended entry gives none.
3. [behavior] Run `cargo test -p athena-core mirrors`: a test shows an edit that changes only the intended line of a prompt file, and an unchanged-structure edit of a decision entry, produce no findings (no false positives), and legacy entries untouched by the diff are not reported.
4. [content] The new rules are entries of REGISTRY in crates/athena-core/src/mirrors.rs (new Rule variants, covering the crates/*/prompts/*.md glob and docs/decisions.md), so mirror_drift_in in run/producer.rs and the candidate gate run them with no new call site; Grep of REGISTRY shows both entries.
5. [content] CHANGELOG.md (Unreleased) names the root cause (no pre-verification comparison of the finished diff against the intended structure) and cites jobs 20261015-095214-y8kr and 20261025-034452-wqp8, and makes no claim the diff does not implement.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] Per outcome-check, the check is the outcome: a prompt or decision-log edit that breaks structure is sent back at the producer step, not found at the final verdict; the tests assert that finding text, not that types or wiring exist.

## Expected deliverables

- crates/athena-core/src/mirrors.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- Re-implementing the urgent priority, athena priority command or reading default of jobs 20261025-034452-wqp8 and 20261015-095214-y8kr; those candidates' files (reader.md, defaults.toml, plan.rs, records.rs, and the rest) are dropped
- Editing prompt files, athena.toml, defaults.toml or records/
- Rewriting existing docs/decisions.md entries; the log stays append-only
- Checking the truth of CHANGELOG claims by machine

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule goes into the shared mirror registry, not into a single job's file.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests on the failing shapes from both jobs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a structural break is caught at the producer step.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover the new rules; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by the mirror gate, not by prose.
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

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261027-054302-e4dt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054302-e4dt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core mirrors`: a unit test feeds the reader.md shape from job 20261015-095214-y8kr (two bullets joined as '...suffice.- deliverables: ...') and gets a Drift naming the file and the bullet; the same test fails when the new rule is removed from check_entry.
Run `cargo test -p athena-core mirrors`: a unit test feeds a docs/decisions.md where D122 is inserted inside D121, leaving D121 without its Provenance bullet, and gets a finding naming D121; a well-formed appended entry gives none.
Run `cargo test -p athena-core mirrors`: a test shows an edit that changes only the intended line of a prompt file, and an unchanged-structure edit of a decision entry, produce no findings (no false positives), and legacy entries untouched by the diff are not reported.
The new rules are entries of REGISTRY in crates/athena-core/src/mirrors.rs (new Rule variants, covering the crates/*/prompts/*.md glob and docs/decisions.md), so mirror_drift_in in run/producer.rs and the candidate gate run them with no new call site; Grep of REGISTRY shows both entries.
CHANGELOG.md (Unreleased) names the root cause (no pre-verification comparison of the finished diff against the intended structure) and cites jobs 20261015-095214-y8kr and 20261025-034452-wqp8, and makes no claim the diff does not implement.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
Per outcome-check, the check is the outcome: a prompt or decision-log edit that breaks structure is sent back at the producer step, not found at the final verdict; the tests assert that finding text, not that types or wiring exist.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
