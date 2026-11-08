# Job 20261027-054242-08bg: Tell the reader which pipeline roles are enabled; target briefs to them

## Request (verbatim)

> Priority: normal
> 
> Give the reader the list of currently enabled pipeline roles, taken from the effective config (for example [job_plan] enabled), and add a reading-brief rule that brief content targets only enabled roles. Acceptance: a test shows the reading brief names the enabled roles from the config.
> 
> Introspections that record the repeated cause: 20261026-202058-73tz, 20261026-210555-xpn0
> 
> Acceptance:
> - The change is made as described: Reading puts new content into planner/critic briefs although job planning is off in athena.toml; user redirects it to the producer by hand
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261026-202109-ckhx.json

## Reading (accepted by the user)

The reader brief gains an "Enabled roles" section built from the effective config (reader, producer, verifier always; planner and critic only when [job_plan] enabled is true), plus a tenth reading rule that brief content targets only enabled roles. Goal: stop readings from putting new content into planner/critic briefs when job planning is off; the audience is the reader agent and the maintainers who redirect it by hand.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (ReaderInputs at ~2214, READING_RULES at ~2247 as a [&str; 9], reader_brief at ~2276, tests near ~4511-4590 that assert READING_RULES.len() == 9) and crates/athena-core/src/reading.rs (read_once builds ReaderInputs at ~567 from jc.config) and crates/athena-data/src/config.rs (JobPlanConfig.enabled, default false; job_plan is an Option). Entry point: read_once -> reader_brief. The role list is computed from jc.config.job_plan in read_once and passed as a new ReaderInputs field. No record or data file is added under crates/, so the records allow-list is not touched.
- Enabled roles: reader, producer and verifier always run; planner and critic are listed only when config.job_plan is Some and enabled is true. 'hard' mode still counts as enabled; the section may note the mode. The effective config is the already-merged jc.config.
- The new rule is prose only (no check_* refusal): it adds no refusal, so no refused-readings percentage applies. It is added to READING_RULES with the count constant and existing count tests updated.
- The reader prompt token budget (3 left) is unaffected: changes are in Rust brief text rendered at run time, not in the prompt files.
- athena.toml and defaults.toml are not changed; only read for the current [job_plan] enabled = true.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-context/src/briefs.rs builds a reader brief with job planning disabled and asserts the brief names reader, producer and verifier as enabled roles and does not list planner or critic as enabled; run `cargo test -p athena-context`.
2. [behavior] A second test builds the brief with job planning enabled and asserts planner and critic appear in the enabled-roles section; run `cargo test -p athena-context`.
3. [behavior] A test in crates/athena-core (e.g. reading.rs tests) shows read_once/reader input construction derives the role list from the config's [job_plan] enabled value, not a hard-coded list; run `cargo test -p athena-core`.
4. [content] crates/athena-context/src/briefs.rs READING_RULES contains a new rule stating that brief content targets only enabled roles, marked `(prose only)`, and the array length and existing length assertions are updated consistently.
5. [content] reader_brief output has a heading section listing the currently enabled roles, checkable by Grep of the heading constant in briefs.rs.
6. [behavior] No unrelated behavior changes: `cargo test` passes with all existing tests, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## Out of scope

- Changing planner, critic, producer or verifier briefs or prompts
- Making the rule a refusal check (check_*) in athena-core
- Changing athena.toml, defaults.toml or the [job_plan] settings
- Editing records/ failure-class files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the brief output names the enabled roles, not just that fields exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the brief builder and its caller.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Gives the reader the config fact so it stops guessing roles, rather than patching one reading.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule marked prose only; the role list itself is derived from config by code.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered brief for enabled and disabled roles.
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

- path: <state>/jobs/20261027-054242-08bg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054242-08bg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-context/src/briefs.rs builds a reader brief with job planning disabled and asserts the brief names reader, producer and verifier as enabled roles and does not list planner or critic as enabled; run `cargo test -p athena-context`.
A second test builds the brief with job planning enabled and asserts planner and critic appear in the enabled-roles section; run `cargo test -p athena-context`.
A test in crates/athena-core (e.g. reading.rs tests) shows read_once/reader input construction derives the role list from the config's [job_plan] enabled value, not a hard-coded list; run `cargo test -p athena-core`.
crates/athena-context/src/briefs.rs READING_RULES contains a new rule stating that brief content targets only enabled roles, marked `(prose only)`, and the array length and existing length assertions are updated consistently.
reader_brief output has a heading section listing the currently enabled roles, checkable by Grep of the heading constant in briefs.rs.
No unrelated behavior changes: `cargo test` passes with all existing tests, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
