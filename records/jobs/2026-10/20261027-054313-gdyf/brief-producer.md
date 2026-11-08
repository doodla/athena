# Job 20261027-054313-gdyf: Refuse scope limits that omit default-written and dependent files

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading wrote a scope limit from the files the request named, without searching the repo for eve
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: scope-criterion-too-narrow-misses-dependent-tests (20261018-045551-4h0h)
> Cause: The reading wrote a scope limit from the files the request named, without searching the repo for every consumer of the changed config. Nothing in the reading step finds all the tests that load the real athena.toml. The strict criterion woul...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261018-045542-tem7 (job 20261018-045437-7z81); prevention it proposed: When a reading restricts edits for a config-changing job, a check should first list every file in the repo that loads the changed config file (for example by searching for athena.toml reads in tests and code). The reading should then allow ...
> Introspection 20261023-052812-187k (job 20261023-052543-9bw7); prevention it proposed: Add a check at reading acceptance. For any criterion that limits which files may change, the check builds the set of files that Athena's defaults always write, such as CHANGELOG.md, and compares it with the allowed list. If a default file i...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (class scope-criterion-too-narrow-misses-dependent-tests, jobs 20261018-045437-7z81 and 20261023-052543-9bw7): reading acceptance never checks a "only these files may change" limit against files Athena always writes (CHANGELOG.md) or the consumers of a changed config. Fix it in reading acceptance with one new step in `refusing_check`, so the reader is refused and repairs the reading before a producer runs.

Assumptions:
- Goal and audience: stop readings from writing a file-scope limit too narrow; the audience is the reader role, whose reading is judged by the reading gate, and then verifiers who fail a job that edits a default or dependent file.
- Previous jobs: 20261018-045437-7z81 landed and verified (pass); its only findings were a nit on the triplicated [job_plan]-stripping test helper (not touched, still allowed by its criterion 4) and a minor note that the producer output was not visible; neither needs a fix here. Its changed files (athena.toml, the three test files, docs/decisions.md, docs/design/job-planning.md, CHANGELOG.md) are kept as landed and are all out_of_scope for this job. Job 20261023-052543-9bw7 was abandoned with no candidate (its content could not land because the base held a reverted spec); its design work is not redone here, and docs/design/records-history.md is out_of_scope. Its scope limit omitted CHANGELOG.md, which Athena always writes; that is the second instance this job fixes.
- Code survey: read crates/athena-core/src/reading/checks.rs (check_reading_acceptance and refusing_check run checks as `step!` lines in order; check_governance_coverage at line 607 and GOVERNANCE_BY_DELIVERABLE at 559 map a deliverable to a governance file), and crates/athena-core/src/reading.rs and crates/athena-core/tests/refusal_corpus.rs (each step! needs a corpus case in which that check refuses). The change adds no record or data file under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly by calling refusing_check on a Reading and asserting the returned check name.
- Existing code: crates/athena-core/src/reading/checks.rs (refusing_check, check_governance_coverage, governance_is_covered, GOVERNANCE_BY_DELIVERABLE) and crates/athena-core/src/admission.rs (is_governance); the new check extends these step and table conventions and reuses the Reading fields, rather than adding a second mechanism.
- Design of the new check, named check_scope_limit: a criterion or out_of_scope item is a scope limit when it says only/no other/nothing else files may change. When a reading has one, it must list CHANGELOG.md as a deliverable (Athena's default write) or name it in the limit; and when athena.toml or defaults.toml is a modify deliverable, it must list a crates/ test deliverable or carry an assumption starting `Consumers:` naming the search of the repo for files that load that config. The error names the missing file and why.
- The reader prompt has 3 tokens left, so the prompt text is not changed; the check list in crates/athena-context/src/briefs.rs may need one short line, and the producer must stay inside the prompt token budget (or place it in a set with room).
- Term ladder: not used; no structural term from the request needs defining.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/checks.rs defines `check_scope_limit` with a doc comment that names the root cause (a scope limit written from the named files without listing default-written files and config consumers) and cites jobs 20261018-045437-7z81 and 20261023-052543-9bw7, and `refusing_check` calls it as a single `step!(check_scope_limit(..))` line.
2. [behavior] Run `cargo test -p athena-core`: a unit test in crates/athena-core/src/reading.rs builds a reading with a scope-limit criterion and no CHANGELOG.md deliverable and asserts refusing_check returns "check_scope_limit"; the same reading with CHANGELOG.md listed is accepted. The test fails when the step! line is removed.
3. [behavior] Run `cargo test -p athena-core`: a test builds a reading with a modify deliverable athena.toml, a scope limit, and neither a crates/ test deliverable nor a `Consumers:` assumption, and asserts it is refused by check_scope_limit; adding either makes it pass. A reading without a scope limit and a reading with no config deliverable are unaffected.
4. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a corpus case in crates/athena-core/tests/refusal_corpus.rs expects Refuse("check_scope_limit") using the shape of job 20261023-052543-9bw7's scope, and the corpus's rule that every step! has a refusing case passes.
5. [behavior] Run `cargo test`: the whole workspace passes, so existing accepted readings in the corpus are not newly refused beyond the intended cases: expected 3% of newly refused readings, and the higher refusal rate is intended because a scope limit that omits files Athena always writes would fail verification.
6. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass.
7. [content] CHANGELOG.md has an entry for the new check, and docs/decisions.md records the call (rule enforced by mechanism, not prose) naming the failure class scope-criterion-too-narrow-misses-dependent-tests.
8. [content] git diff shows no change to athena.toml, defaults.toml, workflows, records/, or the files kept from job 20261018-045437-7z81 beyond what is listed as deliverables.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Files changed by job 20261018-045437-7z81 (athena.toml, crates/athena-cli/tests/common/mod.rs, crates/athena-data/src/config.rs, crates/athena-core/src/run_pin_tests.rs, docs/design/job-planning.md): kept as landed, not touched
- Merging the three duplicated [job_plan]-stripping test helpers (a nit in job 20261018-045437-7z81)
- docs/design/records-history.md and the records-history design of job 20261023-052543-9bw7
- Changes to athena.toml, defaults.toml, workflows, [history] exempt entries, or records/
- Changing prompt text of the reader role (3 tokens left)
- Scanning file contents of the repo for consumers of a config file

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check and its tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and a corpus case; cargo test, clippy and fmt required.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is a refusing step in reading acceptance, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in reading acceptance, covering all scope limits, not the two jobs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail without the step.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that a too-narrow scope limit is refused before a producer runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new check and its tests.
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

- path: <state>/jobs/20261027-054313-gdyf/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054313-gdyf/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/checks.rs defines `check_scope_limit` with a doc comment that names the root cause (a scope limit written from the named files without listing default-written files and config consumers) and cites jobs 20261018-045437-7z81 and 20261023-052543-9bw7, and `refusing_check` calls it as a single `step!(check_scope_limit(..))` line.
Run `cargo test -p athena-core`: a unit test in crates/athena-core/src/reading.rs builds a reading with a scope-limit criterion and no CHANGELOG.md deliverable and asserts refusing_check returns "check_scope_limit"; the same reading with CHANGELOG.md listed is accepted. The test fails when the step! line is removed.
Run `cargo test -p athena-core`: a test builds a reading with a modify deliverable athena.toml, a scope limit, and neither a crates/ test deliverable nor a `Consumers:` assumption, and asserts it is refused by check_scope_limit; adding either makes it pass. A reading without a scope limit and a reading with no config deliverable are unaffected.
Run `cargo test -p athena-core --test refusal_corpus`: a corpus case in crates/athena-core/tests/refusal_corpus.rs expects Refuse("check_scope_limit") using the shape of job 20261023-052543-9bw7's scope, and the corpus's rule that every step! has a refusing case passes.
Run `cargo test`: the whole workspace passes, so existing accepted readings in the corpus are not newly refused beyond the intended cases: expected 3% of newly refused readings, and the higher refusal rate is intended because a scope limit that omits files Athena always writes would fail verification.
Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass.
CHANGELOG.md has an entry for the new check, and docs/decisions.md records the call (rule enforced by mechanism, not prose) naming the failure class scope-criterion-too-narrow-misses-dependent-tests.
git diff shows no change to athena.toml, defaults.toml, workflows, records/, or the files kept from job 20261018-045437-7z81 beyond what is listed as deliverables.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
