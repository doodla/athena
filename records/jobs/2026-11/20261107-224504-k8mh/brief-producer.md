# Job 20261107-224504-k8mh: Add bug-evidence default and enforce root-cause criterion on bug

## Request (verbatim)

> try again on 20261105-234147-88tw

## Reading (accepted by the user)

Retry of job 20261105-234147-88tw (dispatched, never verified, no findings, no candidate files): add a "bug-evidence" default to defaults.toml for bug, wrong-output and regression requests, and make build_reading add a criterion that the change names the root cause, shows it from readable data and shows why the symptom follows, before any fix. Audience: the verifier who rejects data-less root-cause claims at the final verdict. The whole prior scope is kept.

Assumptions:
- Prior job 20261105-234147-88tw: it recorded no findings and no candidate changes, so nothing is carried over. This retry redoes the same work from the base and keeps every deliverable of that job's scope (defaults.toml, reading.rs, defaults.rs, CHANGELOG.md, athena.toml, briefs.rs, defaults-loop.md). Out-of-scope items are also unchanged.
- Code survey: crates/athena-core/src/reading.rs. build_reading (around line 685) builds the Reading from the structured output, then calls ensure_changelog (line 842) before validate(). ensure_changelog (line 281) is the model: idempotent, pushes a DefaultApplied entry only if its id is absent. The new step goes beside it, runs before validate(), and is idempotent. build_reading does not receive the request text, so the step detects bug-type work from the reading's own title, summary and criteria, or the producer threads the request through. Also read crates/athena-data/src/defaults.rs, where defaults.toml is parsed and rendered, and crates/athena-context/src/briefs.rs, where the defaults text reaches the reader prompt.
- Code survey: no new record or data file is created under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. Root-cause evidence is measured directly as a criterion in Reading.acceptance_criteria plus a defaults_applied entry with id bug-evidence, not through a stand-in.
- The reader prompt is at 1997 of 2000 tokens, and the new default renders into it, so the reader budget in athena.toml likely needs a raise. Only that budget value changes, and only if the budget check fails. Verifier prompt sets have 2 to 4 tokens of headroom and must not grow.
- The new step adds a criterion and refuses nothing, so it creates no new refusal rule and no replay refusal rate applies.
- Existing code: crates/athena-core/src/reading.rs (ensure_changelog pattern), crates/athena-data/src/defaults.rs, docs/design/defaults-loop.md. The new step extends the ensure_changelog pattern.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] with id bug-evidence for bug, wrong-output and regression requests. Its statement requires that the change names the root cause, shows it from data the job can read (records, code path, a failing case), and shows why the wrong output or metric change follows, before any fix. It says a guess labelled as an estimate does not meet it. Its source cites 20261104-054700-e6js, 20261105-051344-q9fv, 20261105-101121-qw9r and 20261105-205835-hk09. The existing root-cause default is not reworded.
2. [content] crates/athena-core/src/reading.rs has a step called next to ensure_changelog in build_reading that adds the root-cause-evidence criterion and a bug-evidence defaults_applied entry exactly once for bug, wrong-output, regression and slowdown readings, and nothing for other readings. A Grep for bug-evidence in that file shows it.
3. [behavior] Unit tests in crates/athena-core/src/reading.rs build a reading from a bug request and assert the criterion and defaults entry are present. They assert a non-bug reading gets neither, and that running the step twice adds them once. Run `cargo test -p athena-core`.
4. [behavior] A test in crates/athena-data/src/defaults.rs asserts bug-evidence parses from the repo defaults.toml and its first-sentence line renders. Run `cargo test -p athena-data`.
5. [behavior] `cargo fmt --check`, `cargo clippy --workspace` and `cargo test --workspace` pass. Existing tests keep their outcomes and the prompt token budget tests pass.
6. [content] Governance: defaults.toml is changed and named here. athena.toml changes only if the reader prompt budget check requires it, and then only that budget value. CHANGELOG.md has an entry that names the cause: bug jobs were rejected at the final verdict because no criterion demanded data-backed root-cause evidence. docs/design/defaults-loop.md mentions the new default.
7. [content] Standing defaults hold. outcome-check: a newcomer reading the new default and the criterion text can tell what evidence is required. root-cause: the change description states why bug jobs were rejected. tested: new tests pass. no-empty-success and thrash-stop apply unchanged. records/ is untouched.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/defaults.rs (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-context/src/briefs.rs (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Changing records/ or any owner file, including records/failure-classes/2026-11/20261104-054711-kb90.json
- Changing the verifier's final-verdict rejection logic
- Rewording the existing root-cause default
- Fixing the individual bug and slowdown jobs that were rejected

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires that the new default and criterion text make the required evidence clear to a newcomer.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The change description names why bug jobs were rejected: no criterion asked for data-backed root-cause evidence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Reading and defaults tests are added; fmt, clippy and test are required to pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a reading-pipeline step, not prose alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run cargo test to see the criterion added.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that extends the ensure_changelog pattern.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The default text and docs are written for an outside reader.
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

- path: <state>/jobs/20261107-224504-k8mh/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-224504-k8mh/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

defaults.toml has a new [[default]] with id bug-evidence for bug, wrong-output and regression requests. Its statement requires that the change names the root cause, shows it from data the job can read (records, code path, a failing case), and shows why the wrong output or metric change follows, before any fix. It says a guess labelled as an estimate does not meet it. Its source cites 20261104-054700-e6js, 20261105-051344-q9fv, 20261105-101121-qw9r and 20261105-205835-hk09. The existing root-cause default is not reworded.
crates/athena-core/src/reading.rs has a step called next to ensure_changelog in build_reading that adds the root-cause-evidence criterion and a bug-evidence defaults_applied entry exactly once for bug, wrong-output, regression and slowdown readings, and nothing for other readings. A Grep for bug-evidence in that file shows it.
Unit tests in crates/athena-core/src/reading.rs build a reading from a bug request and assert the criterion and defaults entry are present. They assert a non-bug reading gets neither, and that running the step twice adds them once. Run `cargo test -p athena-core`.
A test in crates/athena-data/src/defaults.rs asserts bug-evidence parses from the repo defaults.toml and its first-sentence line renders. Run `cargo test -p athena-data`.
`cargo fmt --check`, `cargo clippy --workspace` and `cargo test --workspace` pass. Existing tests keep their outcomes and the prompt token budget tests pass.
Governance: defaults.toml is changed and named here. athena.toml changes only if the reader prompt budget check requires it, and then only that budget value. CHANGELOG.md has an entry that names the cause: bug jobs were rejected at the final verdict because no criterion demanded data-backed root-cause evidence. docs/design/defaults-loop.md mentions the new default.
Standing defaults hold. outcome-check: a newcomer reading the new default and the criterion text can tell what evidence is required. root-cause: the change description states why bug jobs were rejected. tested: new tests pass. no-empty-success and thrash-stop apply unchanged. records/ is untouched.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
