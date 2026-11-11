# Job 20261029-233055-70he: Give OLY-9-12 a role-value and safe-retirement mechanism

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-9-12 a mechanism
> 
> Requirement OLY-9-12 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 6 is accepted when new roles and workflows show value against simpler alternatives and can be retired without losing institutional knowledge.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-12
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-12 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add crates/athena-core/src/role_value.rs, which judges a new role or workflow against a simpler alternative by reusing method_experiment::compare. It also allows retiring a role only if every piece of knowledge it holds is recorded elsewhere. Then record it as OLY-9-12's mechanism and measure in docs/spec.toml, following the standalone-module precedent of OLY-9-05.

Assumptions:
- Goal: the Athena maintainers and the Hestia spec review can check that new roles and workflows show value against simpler alternatives and can be retired safely. The spec shows OLY-9-12 with mechanism "none" and a placeholder measure.
- Code survey: read docs/spec.toml (OLY-9-01 to OLY-9-12), crates/athena-core/src/method_experiment.rs (Direction, MethodExperiment, compare, Outcome and Verdict, plus the refusals for an empty method, a missing arm and a zero bound), crates/athena-core/src/lib.rs (the `pub mod method_experiment` declaration), and the STATUS_MEASURE_LABELS list in crates/athena-core/src/status.rs. xtask/tests/spec.rs requires every active requirement to have a mechanism, an owner and a measure, and I read its mechanism tests by grep only. The new module is a pure library function and adds no record or data file, so check_record_file, AnyRecord and the round-trip test do not apply. Nothing in the module runs through an entry point or a probe order. The property is measured directly by unit tests on the verdict and retirement functions.
- Existing code: crates/athena-core/src/method_experiment.rs is reused for the comparison. The OLY-9-05 spec entry is the model for a mechanism that is a standalone module with a `cargo test -p athena-core <module>` measure.
- Following OLY-9-05, the module is not wired into `athena status`, so the measure uses `status: not shown` and names the test. No status label matches this requirement, and adding one is out of scope.
- This adds no refusal rule to the running pipeline; the refusals are inside the new library function only, so no replay refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) are unchanged; the only config-like file edited is docs/spec.toml, which the request names.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-9-12 has a mechanism naming crates/athena-core/src/role_value.rs (not "none"), keeps owner Athena, and a measure ending in `status: not shown` that names `cargo test -p athena-core role_value`. Check with Grep on docs/spec.toml.
2. [content] crates/athena-core/src/role_value.rs exists, is declared in crates/athena-core/src/lib.rs, and its value judgment calls method_experiment::compare instead of re-implementing the comparison (Grep for the call).
3. [behavior] Run `cargo test -p athena-core role_value`. Tests show: a role that beats its simpler alternative is judged worth keeping; one no better or worse is judged not worth keeping; an empty role name, a missing arm or a zero bound is refused. These tests would fail if the module were stubbed out.
4. [behavior] Run `cargo test -p athena-core role_value`. Tests show retirement is allowed only when every item of the role's institutional knowledge is recorded elsewhere. Retirement is refused, naming the unrecorded items, when any item is missing, and the case where a role holds no knowledge items is covered explicitly.
5. [behavior] Run `cargo test -p xtask --test spec`. The tests that require every active requirement to have a mechanism, an owner and a status-marked measure pass with OLY-9-12 filled in.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All three pass.

## Expected deliverables

- crates/athena-core/src/role_value.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Wiring role_value into athena status, the CLI display or the drain
- Adding or retiring any actual role in crates/athena-data/src/roles.rs
- Other requirements with mechanism none (OLY-9-06, OLY-9-11)
- Prompt files and athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new module has unit tests, and cargo test, clippy and fmt are criteria.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing method_experiment comparison instead of building a new one.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module, one lib.rs line and one spec entry.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the role_value tests and check the verdicts and refusals.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 18 cases in 18 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a criterion says a test must assert a property of rendered output, the producer's brief should require two things before the job is done. The test must run against the real rendered page or the real source data. And it must be shown to fail on the known bad input, here the current duplicated te...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

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

- path: <state>/jobs/20261029-233055-70he/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233055-70he/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml requirement OLY-9-12 has a mechanism naming crates/athena-core/src/role_value.rs (not "none"), keeps owner Athena, and a measure ending in `status: not shown` that names `cargo test -p athena-core role_value`. Check with Grep on docs/spec.toml.
crates/athena-core/src/role_value.rs exists, is declared in crates/athena-core/src/lib.rs, and its value judgment calls method_experiment::compare instead of re-implementing the comparison (Grep for the call).
Run `cargo test -p athena-core role_value`. Tests show: a role that beats its simpler alternative is judged worth keeping; one no better or worse is judged not worth keeping; an empty role name, a missing arm or a zero bound is refused. These tests would fail if the module were stubbed out.
Run `cargo test -p athena-core role_value`. Tests show retirement is allowed only when every item of the role's institutional knowledge is recorded elsewhere. Retirement is refused, naming the unrecorded items, when any item is missing, and the case where a role holds no knowledge items is covered explicitly.
Run `cargo test -p xtask --test spec`. The tests that require every active requirement to have a mechanism, an owner and a status-marked measure pass with OLY-9-12 filled in.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All three pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
