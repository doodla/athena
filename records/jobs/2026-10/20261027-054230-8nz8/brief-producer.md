# Job 20261027-054230-8nz8: Refuse delivery-verb readings that name no user-visible outcome

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

Root cause: reading acceptance never compared a delivery verb (publish, deploy, share, release) with the state the user will observe, so jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc were read as narrow file deliverables. Add `check_outcome_state` to reading acceptance. When the request uses such a verb, it requires a criterion that names the observable state, and its error says what to add. Retries of the three earlier jobs are addressed by what the check enforces, not by redoing their changes.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs. The entry point is check_reading_acceptance, which calls refusing_check; refusing_check runs one step! line per check in order: prior result, criteria evidence, code survey, design existing code, governance coverage, boundary and repo files, references. The new check is added as one step! line, placed after check_criteria_evidence. crates/athena-core/tests/refusal_corpus.rs asserts that every step! has a corpus case in which it refuses. This change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly by calling check_outcome_state on a Reading and by running real request shapes through refusing_check.
- Code survey: crates/athena-context/src/briefs.rs holds READING_RULES, and a test asserts exactly 6 rules. The reader prompt has 3 tokens of headroom, so the brief is not changed. The rule reaches the reader through the refusal text, which Athena::read_for feeds into one re-read.
- Job 20261014-082328-g6vy: its governance-file miss is already prevented by check_governance_coverage. What remains is the underlying misread, publish read as 'write a page' rather than 'reachable at a URL'. Its candidate files (xtask site, pages.yml, docs, README, CHANGELOG and others) are not kept or redone by this job; they are listed in out_of_scope as dropped.
- Job 20261014-174815-7bf8: it landed and passed. Its findings about extra files and the A10-1 status are not this cause, and the files it changed are kept as they are and listed in out_of_scope. Its ladder/figurative-term finding is addressed only for delivery verbs. Defining ladder-style terms is left to the ontology work.
- Job 20261017-173508-0wtc: it was rejected for the Hermes definition and its relations in docs/ontology.toml. That is a content error in a file this job does not create, so the candidate is dropped and the glossary proposal is out of scope. The shared cause, a term misread as a narrow deliverable, is covered by the delivery-verb check.
- Goal and audience: the goal is that a reading states what the user will see when the work is done. The audience is the user who judges the result and the verifier who checks the criteria. Observable state words accepted by the check include a URL, reachable, served, live at, downloadable, installable, a tag, a published release, and visible to. The word list is small and deliberately conservative.
- The check applies only when the reading has a capability and deliverables, the same guard as the other checks. A request word counts only as a whole word, matched with has_phrase. Expected N% of newly refused readings is a low number, since delivery verbs are rare among requests.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A new unit test in crates/athena-core/src/reading.rs shows check_outcome_state refusing a reading for the request 'Publish a first explainer page on how firstmate handles work.' whose criteria only say a file exists. The error names the missing observable state, such as the URL or reachability. The test fails when the check is removed.
2. [behavior] Run `cargo test -p athena-core`. A unit test shows the same request accepted when a criterion names a reachable URL or served state. Further cases cover deploy, share and release, and show no refusal when the request has no delivery verb, for example 'Add a timeout setting'.
3. [content] Grep crates/athena-core/src/reading.rs shows a `step!(check_outcome_state(` line inside refusing_check, with a doc comment that names the root cause (a delivery verb read as a narrow deliverable, with no check against the state the user observes) and cites jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc.
4. [behavior] Run `cargo test -p athena-core --test refusal_corpus`. The corpus has a case in which check_outcome_state refuses and a matching accepting case. The corpus test still passes, including its rule that every step! has a refusing case.
5. [behavior] The change refuses more readings. Expected 5% of newly refused readings. A replay of past readings is expected to hold the job only above 10%. The higher refusal rate is intended: the check applies only to requests with a delivery verb whose reading names no observable state.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.
7. [content] CHANGELOG.md has an entry that names the root cause and the new check. No governance file (athena.toml, defaults.toml, workflows, toolchain or lint config) changes, and none is named in a deliverable.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing job 20261014-082328-g6vy: xtask site, .github/workflows/pages.yml, docs/design/explainer-site.md, README.md, .gitignore, xtask/tests/policy.rs and requirements-map.md edits are dropped
- Job 20261014-174815-7bf8's landed files (steward, roles, ladder) are kept and not touched
- Job 20261017-173508-0wtc's docs/ontology.toml, ontology test and decisions entry are dropped; no glossary or ontology is built here
- Changing the reader prompt or READING_RULES in crates/athena-context/src/briefs.rs, which has 3 tokens of headroom
- Defining figurative terms such as ladder, layer or tier, beyond delivery verbs
- Changes to athena.toml, defaults.toml, .github/workflows or toolchain and lint configuration
- Enabling GitHub Pages or any deployment

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix sits in reading acceptance, where a delivery verb is turned into a reading, not in one job's output.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and corpus cases fail without the check; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show the refusal and the acceptance happening.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced as a step! check, not prose.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, with tests and corpus cases.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check requires an observable user outcome in the reading.
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

- path: <state>/jobs/20261027-054230-8nz8/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054230-8nz8/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`. A new unit test in crates/athena-core/src/reading.rs shows check_outcome_state refusing a reading for the request 'Publish a first explainer page on how firstmate handles work.' whose criteria only say a file exists. The error names the missing observable state, such as the URL or reachability. The test fails when the check is removed.
Run `cargo test -p athena-core`. A unit test shows the same request accepted when a criterion names a reachable URL or served state. Further cases cover deploy, share and release, and show no refusal when the request has no delivery verb, for example 'Add a timeout setting'.
Grep crates/athena-core/src/reading.rs shows a `step!(check_outcome_state(` line inside refusing_check, with a doc comment that names the root cause (a delivery verb read as a narrow deliverable, with no check against the state the user observes) and cites jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc.
Run `cargo test -p athena-core --test refusal_corpus`. The corpus has a case in which check_outcome_state refuses and a matching accepting case. The corpus test still passes, including its rule that every step! has a refusing case.
The change refuses more readings. Expected 5% of newly refused readings. A replay of past readings is expected to hold the job only above 10%. The higher refusal rate is intended: the check applies only to requests with a delivery verb whose reading names no observable state.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.
CHANGELOG.md has an entry that names the root cause and the new check. No governance file (athena.toml, defaults.toml, workflows, toolchain or lint config) changes, and none is named in a deliverable.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
