# Job 20261025-001238-mkmh: Remove records/ from main and empty [history] exempt

## Request (verbatim)

> Build job 8 of the records-history plan (docs/design/records-history.md section 4, decision D111): remove records/ from main's tree and empty athena.toml's [history] exempt. Depends on 20261025-001236-brde and 20261025-001234-2hax, and must not run before 14 days after 20261025-001232-vesh landed (BRIEF.md addenda of 2026-10-23).
> Acceptance:
> - athena check --attribution and the history check pass on main with no records/.
> - Legacy commits are still judged by the legacy rule.
> Files: athena.toml, records/, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Job 8 of the records-history plan (D111): delete records/ from main's tree and set athena.toml [history] exempt to an empty list, so the attribution and history checks pass on main without records/ while pre-C0 commits keep the legacy rule. Prior jobs 20261025-001236-brde and 20261025-001234-2hax failed unverified and are dependencies, not retried here. Job 20261025-001232-vesh landed and its work is kept.

Assumptions:
- Goal and audience: the maintainers and CI who judge main; done means main has no records/ and the checks pass.
- Prior jobs: 20261025-001236-brde (readers move to the athena/records branch) and 20261025-001234-2hax (stop writing trailers) are dependencies that ended failed and unverified. Their work is neither kept nor retried by this job. If they have not landed, this job must not run, and the producer must stop and report that instead of removing records/. Their findings (scope, site generator, scratch-test design) do not apply to this job, which touches no code.
- Prior job 20261025-001232-vesh landed with 12 changed files (ci.yml, CHANGELOG.md, athena.toml, cli main.rs, gh.rs, manage.rs, reconcile.rs, records_import.rs, config.rs, attribution.rs, history.rs, decisions.md). All are kept and none are modified here except athena.toml, CHANGELOG.md and docs/decisions.md. Its findings do not apply to this job.
- Timing: the job must not run before 14 days after vesh landed (BRIEF.md addenda of 2026-10-23); today is 2026-11-09. The producer should check the landing date in the records and stop if fewer than 14 days have passed.
- Code survey: I read athena.toml lines 151-174, where [history] has cutover_job = "20261025-001232-vesh" and exempt lists seven commits. I grepped crates/ for 'exempt': crates/athena-gate/src/history.rs (72 hits), legacy.rs (45), crates/athena-data/src/config.rs and history.rs, and crates/athena-cli/tests/cutover_fixture.rs. The legacy-exempt record on athena/records now carries the exempt list, so an empty config list should not change how pre-C0 commits are judged. The producer must confirm this in history.rs and legacy.rs before emptying it. Entry points: athena check --history, which calls check_after_cutover once the cutover is Shown, and athena check --attribution. No record or data file is added under crates/.
- Existing code: this job reuses crates/athena-gate/src/history.rs, crates/athena-gate/src/legacy.rs, crates/athena-gate/src/attribution.rs and crates/athena-data/src/config.rs unchanged, and edits no code.
- Environment: local checkout of main plus the athena/records branch (read access via git). No CI workflow change is planned. The required CI job 'the full history' already fetches athena/records. If the CI history job reads records/ from the tree, ci.yml becomes a deliverable (p about 0.15).
- Consumers: athena.toml [history] exempt is read by crates/athena-data/src/config.rs and crates/athena-gate/src/history.rs and legacy.rs. Any test fixture that reads the repo's own athena.toml exempt list is a consumer to check.
- Deleting records/ is a large tree deletion; the deliverable path is recorded as one directory entry.
- Whether cargo test passes with records/ gone depends on tests that read the repo's records/. Those readers moved in 20261025-001236-brde, which is a dependency. If a test still reads records/ from the tree, the producer reports it as a blocker and does not patch code here.

## Acceptance criteria

1. [content] git diff shows the records/ directory deleted from main's tree and no files left under records/ (ls records fails).
2. [content] athena.toml [history] has exempt = [] and keeps cutover_job = "20261025-001232-vesh"; Grep for the seven old commit shas in athena.toml returns nothing.
3. [content] docs/decisions.md records this step (D111 job 8): records/ removed from main, exempt emptied because the legacy-exempt record on athena/records now carries the list, and the 14-day wait was met. CHANGELOG.md has a matching entry.
4. [behavior] Run `cargo run -- check --attribution` and `cargo run -- check --history` on the result: both pass with no records/ in the tree.
5. [behavior] Legacy commits are still judged by the legacy rule: `cargo test -p athena-gate` and `cargo test -p athena-cli --test cutover_fixture` pass, and the exempt-commit sha 02837e0 and the six other legacy shas are still accepted by the history check through the athena/records legacy-exempt record.
6. [behavior] cargo test, cargo clippy and cargo fmt --check pass with records/ absent. The change does not add refusal rules; expected 0% of newly refused readings.

## Expected deliverables

- records/ (delete)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Moving readers of records/ to the athena/records branch (20261025-001236-brde)
- Ending commit-message trailers (20261025-001234-2hax)
- Changing the cutover, importer, or the history and attribution checks
- Rewriting any commit history or the athena/records branch contents
- Changing code under crates/ or xtask/

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria run athena check --attribution and --history on the result.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands the verifier runs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision and changelog entries are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 13 cases in 13 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a reading's deliverable states a limit, exception or "only N" claim about Athena's behavior, require the reading and plan to derive the list from the enforcing source (the admission path classification code and config) and cite it. Add a criterion that each listed item maps to that source and t...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

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

- path: <state>/jobs/20261025-001238-mkmh/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261025-001238-mkmh/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

git diff shows the records/ directory deleted from main's tree and no files left under records/ (ls records fails).
athena.toml [history] has exempt = [] and keeps cutover_job = "20261025-001232-vesh"; Grep for the seven old commit shas in athena.toml returns nothing.
docs/decisions.md records this step (D111 job 8): records/ removed from main, exempt emptied because the legacy-exempt record on athena/records now carries the list, and the 14-day wait was met. CHANGELOG.md has a matching entry.
Run `cargo run -- check --attribution` and `cargo run -- check --history` on the result: both pass with no records/ in the tree.
Legacy commits are still judged by the legacy rule: `cargo test -p athena-gate` and `cargo test -p athena-cli --test cutover_fixture` pass, and the exempt-commit sha 02837e0 and the six other legacy shas are still accepted by the history check through the athena/records legacy-exempt record.
cargo test, cargo clippy and cargo fmt --check pass with records/ absent. The change does not add refusal rules; expected 0% of newly refused readings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
