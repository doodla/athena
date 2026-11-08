# Job 20261027-054311-dr5k: Make the reader state CI cache branch scoping before a cache criterion

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading stage works from the request text and the local repo view. It has no step that asks, for
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-ignores-ci-cache-branch-scoping (20261025-050615-8gj7)
> Cause: The reading stage works from the request text and the local repo view. It has no step that asks, for each CI cache or stored artifact, which branch saves it, which runs can restore it, and which runs may write shared entries. The cache scop...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261025-050603-wrv7 (job 20261025-034731-2am0); prevention it proposed: Add an environment-facts step to the reading stage. For every criterion that assigns an action, a stored artifact or a cache to CI or to a producer, the reading must list: which ref runs it, what the token can write, which refs are fetched,...
> Introspection 20261025-065849-n99a (job 20261025-055626-er96); prevention it proposed: Add a short, checked table of CI cache scoping rules to the place the reading stage consults, such as ci-routing section 4.1. It should say: which ref scope and event type saves an entry; which runs can restore it (the same pull request's m...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (jobs 20261025-034731-2am0 and 20261025-055626-er96, class reading-ignores-ci-cache-branch-scoping): check_environment_reach makes the reader state site, permissions and data but never asks which ref saves each CI cache or artifact, which runs restore it, and which may write shared entries. Extend that check so a CI workflow deliverable or a cache/artifact behavior criterion needs an Environment: assumption stating save scope, restore scope and write rights; the audience is the reader stage and the verifier replay.

Assumptions:
- Code survey: read crates/athena-core/src/reading/surveys.rs (check_environment_reach, ENV_* word lists, environment_assumptions, scan_segments), crates/athena-core/src/reading.rs (tests module, check list) and crates/athena-core/tests/refusal_corpus.rs (acceptance_check_names, doc-comment test). Entry is the reading acceptance checks run in order after the reading is built. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by calling the check on constructed Readings.
- Existing code: extend crates/athena-core/src/reading/surveys.rs check_environment_reach and its ENV_* lists rather than adding a new check; reuse environment_assumptions and plain_words. No file under crates/ is created.
- Environment: the check runs locally in the reader stage over the in-memory Reading with read-only access to the request and snapshot; it reaches no refs, origin or CI, so the no origin case does not apply. The rule it enforces concerns CI (GitHub Actions) caches: a cache is saved by the ref and event that ran it, restored by runs of the same ref or its base, and shared entries are written only by the default branch.
- Previous jobs: 20261025-034731-2am0 (planned edits to ci.yml, xtask, gh.rs, docs/decisions.md; failed at the 1800s producer deadline, no candidate) and 20261025-055626-er96 (planning stopped over GhEarly warnings in CLI tests; no candidate). Neither candidate changed any file, so no work is kept and their files stay untouched. The earlier findings concern those jobs' own feature work (the CI result cache, the early push) and no longer apply: this job fixes only the reading-stage cause that let both plans ignore cache scoping.
- The trigger is lexical like the existing check: the words cache or artifact in a behavior criterion, or a workflow deliverable. The required Environment: text must contain a cache or artifact word, a save word, a restore word, and a scope word (branch, ref, main, pull request, default); wording lists are constants beside the ENV_* lists.
- Expected refusal share 3% is an estimate not measured, since records are absent from the snapshot.

## Acceptance criteria

1. [content] The doc comment of check_environment_reach in crates/athena-core/src/reading/surveys.rs names the root cause (the reading has no step asking, for each CI cache or stored artifact, which ref saves it, which runs restore it, and which runs may write shared entries), cites jobs 20261025-034731-2am0 and 20261025-055626-er96 and introspections 20261025-050603-wrv7 and 20261025-065849-n99a, and says: This check refuses more readings; the higher refusal rate is intended, expected 3% of newly refused readings (an estimate, not measured).
2. [content] CHANGELOG.md Unreleased has an item naming the cause and the cache-scope requirement, with the sentence 'expected 3% of newly refused readings'.
3. [behavior] Run cargo test -p athena-core: a new unit test in crates/athena-core/src/reading.rs shows a reading with a created .github/workflows/ci.yml deliverable or a behavior criterion that keys a CI cache or artifact is refused when its Environment: assumption lacks cache save scope, restore scope or write rights, with an error naming the missing part, and is accepted when the assumption states them (for example saved by the main branch push run, restored by pull request runs of the same branch and by main, written only by main). The test fails when the new check is removed.
4. [behavior] Run cargo test -p athena-core: a test covers the class, not one job. It uses wording from both referenced jobs (a fingerprint-keyed result cache; an early push whose CI run is reused at landing), plus a reading with no cache or artifact and no workflow, which stays accepted unchanged.
5. [behavior] Run cargo test -p athena-core: crates/athena-core/tests/refusal_corpus.rs includes the new check in the acceptance-check name list and the replay declaration test passes with the declared share of 3%.
6. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing .github/workflows/ci.yml, xtask or the fingerprint
- Implementing the CI result cache or the early push of jobs 20261025-034731-2am0 and 20261025-055626-er96
- Changing athena.toml, defaults.toml or prompt files
- Editing gh.rs, run.rs, land.rs, pr.rs or test_support.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests plus fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the reading check where the missing step originates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the new check on constructed readings.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a refusing check, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check extended.
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
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

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

- path: <state>/jobs/20261027-054311-dr5k/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054311-dr5k/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The doc comment of check_environment_reach in crates/athena-core/src/reading/surveys.rs names the root cause (the reading has no step asking, for each CI cache or stored artifact, which ref saves it, which runs restore it, and which runs may write shared entries), cites jobs 20261025-034731-2am0 and 20261025-055626-er96 and introspections 20261025-050603-wrv7 and 20261025-065849-n99a, and says: This check refuses more readings; the higher refusal rate is intended, expected 3% of newly refused readings (an estimate, not measured).
CHANGELOG.md Unreleased has an item naming the cause and the cache-scope requirement, with the sentence 'expected 3% of newly refused readings'.
Run cargo test -p athena-core: a new unit test in crates/athena-core/src/reading.rs shows a reading with a created .github/workflows/ci.yml deliverable or a behavior criterion that keys a CI cache or artifact is refused when its Environment: assumption lacks cache save scope, restore scope or write rights, with an error naming the missing part, and is accepted when the assumption states them (for example saved by the main branch push run, restored by pull request runs of the same branch and by main, written only by main). The test fails when the new check is removed.
Run cargo test -p athena-core: a test covers the class, not one job. It uses wording from both referenced jobs (a fingerprint-keyed result cache; an early push whose CI run is reused at landing), plus a reading with no cache or artifact and no workflow, which stays accepted unchanged.
Run cargo test -p athena-core: crates/athena-core/tests/refusal_corpus.rs includes the new check in the acceptance-check name list and the replay declaration test passes with the declared share of 3%.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
