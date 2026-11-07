# Job 20261107-200727-xwss: Make the reading check cover ontology terms that were misread

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Athena turns request wording into a reading without checking it against the outcome the user will ob
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: ambiguous-term-misread-as-narrow-deliverable (20261014-082433-5rw0)
> Cause: Athena turns request wording into a reading without checking it against the outcome the user will observe. For verbs like publish, deploy or share, the real goal is a user-visible state, such as being reachable at a URL. Nothing requires a ...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261014-082423-9m1q (job 20261014-082328-g6vy); prevention it proposed: Change reading generation so that every reading of a request is written as an observable outcome: what the user can do or see when the work is done. For delivery verbs such as publish, deploy, share or release, the reading must name the acc...
> Introspection 20261014-174922-2n9v (job 20261014-174815-7bf8); prevention it proposed: Add a step to reading acceptance. For any request that uses a figurative or structural term (ladder, layer, tier, publish, and similar), the reading must define the term in terms of what it ranks or does, and give an observable acceptance c...
> Introspection 20261017-174855-d7s1 (job 20261017-173508-0wtc); prevention it proposed: Keep a glossary of project terms and owner conventions in the repo. It would hold the terms above, each with its meaning and source, plus the rule to refer to Sri by name or as "they". Load it into every reading step. Add a check that flags...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a reading is not checked against the outcome the user will observe. The delivery-verb, governance and structural-term checks already close the g6vy and 7bf8 cases, but 20261017-173508-0wtc (the Hermes misread) is still not caught. This job extends `check_defined_terms` so a request that uses an ambiguous name recorded in docs/ontology.toml (Hermes) is refused until the reading restates it in a `Term Hermes:` assumption. The audience is the Athena reading gate and Sri.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (`refusing_check` runs check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_history_fixture, check_design_existing_code, check_governance_coverage, check_outcome_state, check_environment_reach, check_defined_terms, then the boundary and reference checks; `check_defined_terms` reads `STRUCTURAL_TERMS`, `defined_terms` and `plain_request_prose`), crates/athena-core/tests/refusal_corpus.rs (CITED_REQUESTS, `delivery_verb_replay_over_the_cited_jobs`, `defined_terms_replay_over_the_cited_jobs`, which assert 1 of 3 and read CHANGELOG.md), and crates/athena-cli/tests/ontology.rs (validates docs/ontology.toml). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by running `refusing_check` on the recorded request of 20261017-173508-0wtc with a reading that has no `Term Hermes:` line.
- Existing job state: the checks for the three jobs are on main already. Job 20261014-082328-g6vy (publish an explainer page) is covered by check_outcome_state and check_governance_coverage; its candidate files (pages.yml, xtask/src/site.rs, xtask/src/main.rs, xtask/tests/policy.rs, .gitignore, README.md, docs/design/explainer-site.md, docs/design/requirements-map.md, CHANGELOG.md) are not redone: the page work is dropped here and left to its own follow-up. Job 20261014-174815-7bf8 landed and passed, its findings were scope notes, and its work is kept untouched. Job 20261017-173508-0wtc was rejected for the Hermes definition; docs/ontology.toml on main now holds both Hermes senses, so that finding no longer applies, and this job keeps that file and its test and extends the reading check to cover the remaining class.
- Term ladder: not used by the request in its own prose; the word appears only in quoted introspection text that Athena wrote.
- The ambiguous-name list is a small constant in reading.rs, starting with Hermes (the one ontology term whose second sense caused a reading misread); a test requires every name on it to be a term in docs/ontology.toml, so the list cannot drift from the shared vocabulary.
- Matching follows the existing rules of check_defined_terms: whole words in plain prose, skipping quoted, backtick, path and generated lines. The error names the term and the repair `Term Hermes: <meaning of at least three words>`.
- Expected 2% of newly refused readings, an estimate: few requests name Hermes outside quotes or generated text. Past readings should be refused because the request now asks the reader to define an ambiguous recorded name before building on it.
- Environment: none; no CI workflow, push or pull request is involved, and the change reads only the request, the reading and a repo file included at compile time.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a test replays the recorded request of 20261017-173508-0wtc through `refusing_check` with a reading that has no `Term Hermes:` assumption and asserts check_defined_terms refuses it naming Hermes; the same reading with a `Term Hermes:` line of three or more words is accepted. The test fails on the code before the change.
2. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the replay over the three cited jobs now asserts check_defined_terms catches 2 of 3 (7bf8 and 0wtc), check_outcome_state still catches g6vy, and CHANGELOG.md states the same counts.
3. [behavior] Run `cargo test -p athena-core`: cases show the class, not one job: other requests using Hermes in plain prose are refused, while Hermes inside quotes, backticks, paths or Athena-generated lines (root-cause and approval requests) is not, and a request with no listed name is unchanged.
4. [behavior] Run `cargo test -p athena-core`: a test reads docs/ontology.toml and fails if any name on the ambiguous-name list in reading.rs is not a `[[term]]` name there.
5. [content] The doc comment of `check_defined_terms` in crates/athena-core/src/reading.rs names the root cause (readings are not checked against the outcome the user observes), cites jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, states that it refuses more readings with `expected 2% of newly refused readings`, and no longer says the Hermes term is uncovered; the error text of the check is updated to cover ambiguous names.
6. [content] CHANGELOG.md has an entry naming the root cause and the three jobs, and says which checks cover which job.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no new warnings.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Publishing the explainer page or any pages.yml, xtask/src/site.rs, xtask/src/main.rs, xtask/tests/policy.rs, .gitignore, README.md, docs/design/explainer-site.md or docs/design/requirements-map.md work from job 20261014-082328-g6vy: dropped, not redone here
- Redoing the steward, roles and ladder work of job 20261014-174815-7bf8: kept as landed, no files of it change
- Rewriting docs/ontology.toml or crates/athena-cli/tests/ontology.rs from job 20261017-173508-0wtc: kept as they are on main
- Changes to athena.toml, defaults.toml, workflows, Cargo files or lint configuration
- Adding ontology terms not caused by a misread, or wiring the whole ontology into agent prompts
- Prompt files under the reader, verifier or other prompt sets (no token headroom)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core and its tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New corpus and unit tests; fmt, clippy and test are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Extends the existing reading gate where structural-term misreads originate, with a sync test against the ontology.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the replay and name the refusal and acceptance.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a check in refusing_check and a drift test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check extended, a few files.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks that the recorded 0wtc request is refused until the reading defines the term.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and CHANGELOG name cause and jobs.
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

- path: <state>/jobs/20261107-200727-xwss/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-200727-xwss/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core --test refusal_corpus`: a test replays the recorded request of 20261017-173508-0wtc through `refusing_check` with a reading that has no `Term Hermes:` assumption and asserts check_defined_terms refuses it naming Hermes; the same reading with a `Term Hermes:` line of three or more words is accepted. The test fails on the code before the change.
Run `cargo test -p athena-core --test refusal_corpus`: the replay over the three cited jobs now asserts check_defined_terms catches 2 of 3 (7bf8 and 0wtc), check_outcome_state still catches g6vy, and CHANGELOG.md states the same counts.
Run `cargo test -p athena-core`: cases show the class, not one job: other requests using Hermes in plain prose are refused, while Hermes inside quotes, backticks, paths or Athena-generated lines (root-cause and approval requests) is not, and a request with no listed name is unchanged.
Run `cargo test -p athena-core`: a test reads docs/ontology.toml and fails if any name on the ambiguous-name list in reading.rs is not a `[[term]]` name there.
The doc comment of `check_defined_terms` in crates/athena-core/src/reading.rs names the root cause (readings are not checked against the outcome the user observes), cites jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, states that it refuses more readings with `expected 2% of newly refused readings`, and no longer says the Hermes term is uncovered; the error text of the check is updated to cover ambiguous names.
CHANGELOG.md has an entry naming the root cause and the three jobs, and says which checks cover which job.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no new warnings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
