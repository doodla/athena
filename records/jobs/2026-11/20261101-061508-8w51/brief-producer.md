# Job 20261101-061508-8w51: Make readings state where each action runs and what data it reaches

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading stage works from the request text and the local repo view. It has no step that asks, for
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-ignores-runtime-permissions-and-data-reachability (20261024-220330-z0nm)
> Cause: The reading stage works from the request text and the local repo view. It has no step that asks, for each action or measurement the reading assigns, who performs it and where, what permissions that place has, and what data it can reach. The...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261024-220314-yjtg (job 20261024-220048-engm); prevention it proposed: Add an environment-capabilities step to the reading stage. A short checked-in table would give, for each execution site (CI job, producer working copy, local the person running Athena run), its write permissions and the refs and data it can see. The reading...
> Introspection 20261025-070613-4vfa (job 20261025-070538-8ra0); prevention it proposed: Add a required step to the reading stage for any new action that calls an external host or depends on repo configuration. The reading must list the environment cases the action can meet: no origin, a non-GitHub origin, a GitHub origin with ...
> Introspection 20261026-011842-889s (job 20261025-034247-nm0k); prevention it proposed: Add a check at reading acceptance. When a reading cites job records, history or the records branch as a source, it must either quote the needed figures inline or list them in a reachable file. The check compares each cited source against th...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (jobs 20261024-220048-engm, 20261025-070538-8ra0, 20261025-034247-nm0k): the reading stage never asks who performs an action or measurement, where, with what permissions and what data it can reach, so plans assumed CI writes, a missing GitHub origin, or unreachable records. Fix it at the reading gate with a new `check_environment_reach` step in refusing_check (reading.rs) that refuses readings that need an external site or unreachable source but lack an `Environment:` assumption.

Assumptions:
- Goal and audience: the reader stage and its gate (Athena maintainers) must stop readings that assign an action or measurement to a place without saying who runs it, its permissions and the data it reaches. The fix is in the gate, not a patch for one job.
- Code survey: read crates/athena-core/src/reading.rs. `refusing_check` (line ~3226) runs `step!` checks in order (prior result, criteria evidence, code survey, design existing code, governance, outcome state, boundary, references); a new check is added as one `step!` line there. `check_code_survey` (line ~3910) and its helpers `code_surveys` and `names_crates_path` are the model: label-prefixed assumption, class and criterion gating. The property is measured directly by calling the new check on Reading values and through `refusing_check`, which returns the refusing check's name. crates/athena-core/tests/refusal_corpus.rs fails if a `step!` has no corpus case in which it refuses, so a refuse case and an accept case are added there. No record or data file is added, so `check_record_file` and `AnyRecord` are not involved.
- Previous jobs: 20261024-220048-engm (planning stopped; no candidate, nothing kept) is addressed by requiring the CI-write versus read-only and records-reachability statement up front; 20261025-070538-8ra0 (admission refused, email in gh.rs; its candidate files run.rs, gh.rs, pr.rs, run_pin_tests.rs, CHANGELOG.md are not part of this change and are dropped, out of scope) is addressed by requiring the no-origin, non-GitHub-origin and GitHub-origin cases; 20261025-034247-nm0k (landed docs; findings were a table gap and a loose citation in docs/design/scope-guard.md, unrelated to this cause) is addressed by requiring cited records or history to be quoted inline or listed in a reachable file. Its changed files docs/design/scope-guard.md and docs/decisions.md are left as landed and not modified.
- Trigger design: the check applies when the reading has a capability and no question, and (a) a deliverable or criterion names a CI workflow (.github/workflows/), an external host action (push, pull request, gh, origin, remote), or (b) a criterion or the summary cites job records, history, the records branch or athena/records as a source. It is satisfied by an `Environment:` assumption that names the site performing the action, its permissions (read/write), and the data or refs it can reach; case (b) alternatively by quoting the figures inline in an assumption. Docs-only readings with none of the triggers are unaffected.
- Expected refusal rate: a small share of past readings carry these triggers; expected 8% of newly refused readings, below the 10% replay hold; the higher refusal rate is intended.
- The reader brief rule list in crates/athena-context/src/briefs.rs gains one line naming the new check so the reader writes the `Environment:` assumption; prompt files with no token headroom are not edited.
- Governance files (athena.toml, defaults.toml, workflows) are not changed.

## Acceptance criteria

1. [content] The CHANGELOG.md entry and the doc comment on `check_environment_reach` in crates/athena-core/src/reading.rs name the root cause: the reading stage had no step asking who performs each action, where, with what permissions, and what data it reaches; and cite jobs 20261024-220048-engm, 20261025-070538-8ra0 and 20261025-034247-nm0k.
2. [content] `refusing_check` in crates/athena-core/src/reading.rs contains a single `step!(check_environment_reach(..))` line, and crates/athena-context/src/briefs.rs lists the rule so the reader is told to write an `Environment:` assumption.
3. [behavior] Run `cargo test -p athena-core`: unit tests show a reading that adds a CI workflow deliverable, or a push/pull-request/gh action, without an `Environment:` assumption naming site, permissions and reachable data is refused by `check_environment_reach`, and the same reading with such an assumption is accepted. These tests fail when the new step is removed.
4. [behavior] Run `cargo test -p athena-core`: a test shows a reading citing job records, history or the records branch as a source is refused unless the figures are quoted inline or an `Environment:` assumption lists a reachable file, and that it is accepted when they are. Tests cover the class (separate readings shaped like each of the three jobs), not one job.
5. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the corpus has a refuse case and an accept case for `check_environment_reach`, and the test that every `step!` has a refusing corpus case passes.
6. [behavior] Docs-only readings and readings without these triggers are still accepted: a test with a plain docs reading passes `check_environment_reach`; the change refuses more readings and the higher refusal rate is intended, expected 8% of newly refused readings.
7. [behavior] Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Rebuilding the records-history incremental checking work of job 20261024-220048-engm (no candidate kept)
- The early-push and PR work of job 20261025-070538-8ra0: crates/athena-core/src/gh.rs, pr.rs, run.rs, run_pin_tests.rs are not changed and that candidate is dropped
- Editing docs/design/scope-guard.md or docs/decisions.md from job 20261025-034247-nm0k; landed as is
- Changes to athena.toml, defaults.toml, workflows or prompt files
- Adding an environment-capabilities table file or any record or data file

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests, a refusal corpus case, and fmt, clippy and test passing are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a gate step at the reading stage, not a fix for one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and check the refusal and acceptance behavior.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a check in refusing_check, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, one corpus pair, one brief line.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments state the cause for outside readers.
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

- path: <state>/jobs/20261101-061508-8w51/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261101-061508-8w51/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The CHANGELOG.md entry and the doc comment on `check_environment_reach` in crates/athena-core/src/reading.rs name the root cause: the reading stage had no step asking who performs each action, where, with what permissions, and what data it reaches; and cite jobs 20261024-220048-engm, 20261025-070538-8ra0 and 20261025-034247-nm0k.
`refusing_check` in crates/athena-core/src/reading.rs contains a single `step!(check_environment_reach(..))` line, and crates/athena-context/src/briefs.rs lists the rule so the reader is told to write an `Environment:` assumption.
Run `cargo test -p athena-core`: unit tests show a reading that adds a CI workflow deliverable, or a push/pull-request/gh action, without an `Environment:` assumption naming site, permissions and reachable data is refused by `check_environment_reach`, and the same reading with such an assumption is accepted. These tests fail when the new step is removed.
Run `cargo test -p athena-core`: a test shows a reading citing job records, history or the records branch as a source is refused unless the figures are quoted inline or an `Environment:` assumption lists a reachable file, and that it is accepted when they are. Tests cover the class (separate readings shaped like each of the three jobs), not one job.
Run `cargo test -p athena-core --test refusal_corpus`: the corpus has a refuse case and an accept case for `check_environment_reach`, and the test that every `step!` has a refusing corpus case passes.
Docs-only readings and readings without these triggers are still accepted: a test with a plain docs reading passes `check_environment_reach`; the change refuses more readings and the higher refusal rate is intended, expected 8% of newly refused readings.
Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
