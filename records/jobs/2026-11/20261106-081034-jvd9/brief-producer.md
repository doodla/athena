# Job 20261106-081034-jvd9: Check hand-kept mirrors against their sources before handover

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> Introspection 20261102-102715-564y (job 20261031-172040-kq9v); prevention it proposed: Add a code check, run in the repo tests and before verification, that computes the athena binary's real input set (every file the build compiles in, including everything xtask input_files should list) and fails if the hashFiles globs in eve...
> Introspection 20261106-030930-hb84 (job 20261106-002344-jn5t); prevention it proposed: Add a fast code check that runs in the producer's loop, and again before the verifier. It fails when any test file in the repo is missing from xtask/shard-timings.txt, and its message names the file and the line to add. Also keep the real f...
> Introspection 20261106-074349-rag3 (job 20261106-062207-cbcc); prevention it proposed: Add a check in the decisions test and in the producer's local checks. It compares docs/decisions.md with its base version and fails unless the base is an exact byte prefix of the new file, so every change is added text after the last old li...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a file that summarizes or mirrors a governing source (ontology terms, CI hashFiles globs, shard-timings, decisions.md) has only structure tests, so content that contradicts the source passes until the final verdict. Add one registry of such mirrors in athena-core with content checks against each source, run in the repo tests and the producer's local checks. Audience: the producer and verifier of future jobs. Covers jobs 20261017-173508-0wtc, 20261025-214509-y179, 20261031-172040-kq9v and the two other introspection jobs.

Assumptions:
- Code survey: I read the list of crates/athena-core/src, and crates/athena-core/src/reading.rs (check_governance_coverage, check_criteria_evidence, check_code_survey) as the pattern for named pre-handover checks. Entry point for the new check is a pure function in a new crates/athena-core/src/mirrors.rs, called from a crates/athena-cli/tests test that reads the repo files. I did not read the producer's local-check runner; the producer must locate it (likely crates/athena-core/src/checks.rs) and hook the check in before handover.
- The new file is code, not a record or data file under crates/, so check_record_file, AnyRecord and the round-trip test do not apply. The property is measured directly: each mirror's check reads the mirror and its source and compares content, not file existence.
- Root cause named in the change: derived or summarizing files are kept in step with their source by hand, and tests check only their shape. Fix is a registry of mirrors (mirror path, source path, comparison rule) so every new mirror needs an entry; initial rules: every test file listed in xtask/shard-timings.txt, docs/decisions.md base is a byte prefix of new text (append-only), ci.yml hashFiles globs cover BUILD_PATHS from crates/athena-core/src/status.rs, and a term-name search of governing docs for ontology entries when docs/ontology.toml exists.
- Job 20261017-173508-0wtc (ontology; verifier found Hermes sense missing): the ontology work was rejected and is dropped, not re-done here; its finding is addressed by the generic term-name check, not by writing the ontology. Job 20261025-214509-y179 (test speed): its changes to secrets.rs and checks.rs are dropped; its findings are speed-goal specific and out of scope. Job 20261031-172040-kq9v (CI cache): its ci.yml, xtask/src/main.rs and xtask/tests/policy.rs changes are dropped; its stale-key finding is addressed by the hashFiles-versus-BUILD_PATHS rule, which will fail on the current key until fixed separately.
- This is a refusal-adding check on the producer's own work, not on readings, so no replay of past readings applies; expected 0% of newly refused readings.
- Existing code: crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs and crates/athena-core/src/checks.rs are extended or reused, not created.

## Acceptance criteria

1. [content] The change description names the root cause: hand-kept mirrors of a governing source are tested for structure only, not content against the source.
2. [content] crates/athena-core/src/mirrors.rs holds a registry listing each mirror, its source and its comparison rule, and is exported from crates/athena-core/src/lib.rs.
3. [behavior] Run `cargo test -p athena-core mirrors`: unit tests fail on the old code and pass with the change for each rule (test file missing from shard-timings, decisions.md rewritten rather than appended, hashFiles glob missing a BUILD_PATHS entry, term name found in a governing doc but sense unlisted), each with a message naming the file and fix.
4. [behavior] Run `cargo test -p athena-cli`: a repo-level test runs the registry over the real files and fails when a mirror drifts; add an entry to the registry and the test covers it with no other edit.
5. [content] The producer's local check path invokes the registry before handover (Grep shows the call site), and CHANGELOG.md and docs/decisions.md (appended only) record the change.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/mirrors.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/tests/mirrors.rs (create)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Writing docs/ontology.toml or its test (job 20261017-173508-0wtc candidate dropped)
- The secrets.rs memo and run_check poll changes and any test-speed goal (job 20261025-214509-y179 candidate dropped)
- Editing .github/workflows/ci.yml, xtask/src/main.rs or xtask/tests/policy.rs to fix the cache key (job 20261031-172040-kq9v candidate dropped)
- Changing athena.toml, defaults.toml or workflows
- Adding speed-target measure-first stages

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests; cargo test and clippy pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The drift rule is enforced by a check, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the shared mirror-drift cause, not one instance.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check drift being refused, not that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One registry with a few rules.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Messages and docs written for outside readers.
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

- path: <state>/jobs/20261106-081034-jvd9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-081034-jvd9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change description names the root cause: hand-kept mirrors of a governing source are tested for structure only, not content against the source.
crates/athena-core/src/mirrors.rs holds a registry listing each mirror, its source and its comparison rule, and is exported from crates/athena-core/src/lib.rs.
Run `cargo test -p athena-core mirrors`: unit tests fail on the old code and pass with the change for each rule (test file missing from shard-timings, decisions.md rewritten rather than appended, hashFiles glob missing a BUILD_PATHS entry, term name found in a governing doc but sense unlisted), each with a message naming the file and fix.
Run `cargo test -p athena-cli`: a repo-level test runs the registry over the real files and fails when a mirror drifts; add an entry to the registry and the test covers it with no other edit.
The producer's local check path invokes the registry before handover (Grep shows the call site), and CHANGELOG.md and docs/decisions.md (appended only) record the change.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
