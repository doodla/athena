# Job 20261107-110049-k8c2: Refuse readings that leave a structural request term undefined

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

Root cause: reading acceptance has no check of a reading against the outcome the user observes. check_outcome_state (job 20261014-082328-g6vy) covers only delivery verbs, so figurative terms like ladder or tier (jobs 20261014-174815-7bf8, 20261017-173508-0wtc) still pass. Add a second step in refusing_check, check_defined_terms, which refuses a reading that does not define a structural request term and give a behavior criterion for it. The audience is Sri and the verifier judging readings.

Assumptions:
- Code survey: I read crates/athena-core/src/reading.rs. The entry point is check_reading_acceptance, which calls refusing_check. refusing_check runs one step!(...) line per check in a fixed order, and the last of them is check_reading_references. check_outcome_state (about line 3680) covers only publish, deploy, share and release. Its own doc comment says the other two cited jobs are 'another class (figurative or structural terms that need a glossary)' and are not caught. crates/athena-core/tests/refusal_corpus.rs runs real shapes through refusing_check, replays the three cited requests in delivery_verb_replay_over_the_cited_jobs, and fails when a step! line has no corpus case where that check refuses. crates/athena-cli/tests/ontology.rs reads docs/ontology.toml. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not involved.
- Job 20261014-082328-g6vy: its governance finding (pages.yml not listed) is already fixed by check_governance_coverage, and its delivery-verb misread by check_outcome_state. Both are kept untouched. Its candidate files (xtask/src/site.rs, xtask/src/main.rs, xtask/tests/policy.rs, pages.yml, explainer-site.md, README.md, .gitignore, CHANGELOG.md, requirements-map.md) are dropped. Re-doing the site is not this request.
- Job 20261014-174815-7bf8 landed and passed. Its steward, roles and ladder work stays as it is. Its minor findings (extra files, A10-1 marked Covered) are not the cause here. It matters only as the 'ladder' request that the new check must refuse when the reading leaves 'ladder' undefined.
- Job 20261017-173508-0wtc was rejected: Hermes was defined too narrowly and the relations were off. That ontology candidate is not rebuilt or edited here. docs/ontology.toml, docs/decisions.md and crates/athena-cli/tests/ontology.rs are left as they are. The new check does not read docs/ontology.toml, so the Hermes finding does not apply. A glossary-fed check is left for later.
- The structural-term list is a small const in reading.rs: ladder, rung, tier, layer. Reading the ontology file at reading time is a bigger change that would need new plumbing, so it is skipped. A reading passes when each such word the request uses as a whole word appears in a summary or assumption line that defines it (the line starts with 'Term <word>:' and says what it ranks or does), and a behavior criterion names that word. Otherwise it fails with an error that gives both repairs, and read_for rereads with that error text. The reader prompt has 3 tokens left, so no prompt text changes.
- Expected refusal rate: 3% of newly refused readings (estimate, not measured). Only requests that use one of the four words as a whole word are affected. The dev must count the recorded requests that contain them and report the count.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new refusal_corpus case built from the recorded request of job 20261014-174815-7bf8 (a ladder of requirements), with a reading that has no 'Term ladder:' definition, is refused by check_defined_terms, and the same reading with a definition and a behavior criterion naming ladder is accepted.
2. [behavior] Run `cargo test -p athena-core`: a corpus case for a request that uses an unrelated word, such as 'stepladder' or 'delayer', or no structural word at all, is accepted. This shows the whole-word match does not refuse ordinary requests.
3. [behavior] Run `cargo test -p athena-core`: the structural-term test fails when the step!(check_defined_terms(..)) line is removed from refusing_check and passes with it. The existing test that requires every step! line to have a refusing corpus case also passes.
4. [content] The doc comment on check_defined_terms in crates/athena-core/src/reading.rs names the root cause (readings are not checked against the outcome the user observes, and structural terms stay undefined), cites jobs 20261014-174815-7bf8 and 20261017-173508-0wtc, and says what the check does not cover.
5. [content] The doc comment on check_defined_terms in crates/athena-core/src/reading.rs states that the check refuses more readings, that the higher refusal rate is intended, and gives 'expected 3% of newly refused readings'. The comment on check_outcome_state is updated so it no longer says the other two jobs are outside any check.
6. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.
7. [content] CHANGELOG.md has an entry that names the root cause and the new check.
8. [behavior] Outcome check: the corpus replay of the three cited job requests (g6vy, 7bf8, 0wtc) shows each one is now caught by a check when the reading does not satisfy it: g6vy by check_outcome_state, 7bf8 by check_defined_terms, and 0wtc by check_defined_terms if its request uses a structural word. If a request has no structural word, the test asserts that count and says it is outside the class. Run `cargo test -p athena-core`.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Re-doing the explainer site, pages.yml or any xtask changes from job 20261014-082328-g6vy (dropped)
- Changing the steward, roles or ladder work of job 20261014-174815-7bf8 (kept as landed)
- Editing docs/ontology.toml, its test, or docs/decisions.md D109 from job 20261017-173508-0wtc, or fixing its Hermes finding
- Making the check read docs/ontology.toml, or wiring the ontology into agent contexts
- Changes to athena.toml, defaults.toml, workflows, or prompt text (the reader prompt has no token room)
- Changing check_outcome_state or check_governance_coverage rules beyond their doc comments

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New corpus and unit tests, with fmt, clippy and test all required to pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is added at reading acceptance, where the unchecked reading originates, not for one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the check on readings and look at the accept or refuse outcome.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The cited requests are replayed and the outcome of each is asserted.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is a step! check in refusing_check, enforced in code.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, one const list, and tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The doc comments name causes and jobs for an outside reader.
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

- path: <state>/jobs/20261107-110049-k8c2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-110049-k8c2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`: a new refusal_corpus case built from the recorded request of job 20261014-174815-7bf8 (a ladder of requirements), with a reading that has no 'Term ladder:' definition, is refused by check_defined_terms, and the same reading with a definition and a behavior criterion naming ladder is accepted.
Run `cargo test -p athena-core`: a corpus case for a request that uses an unrelated word, such as 'stepladder' or 'delayer', or no structural word at all, is accepted. This shows the whole-word match does not refuse ordinary requests.
Run `cargo test -p athena-core`: the structural-term test fails when the step!(check_defined_terms(..)) line is removed from refusing_check and passes with it. The existing test that requires every step! line to have a refusing corpus case also passes.
The doc comment on check_defined_terms in crates/athena-core/src/reading.rs names the root cause (readings are not checked against the outcome the user observes, and structural terms stay undefined), cites jobs 20261014-174815-7bf8 and 20261017-173508-0wtc, and says what the check does not cover.
The doc comment on check_defined_terms in crates/athena-core/src/reading.rs states that the check refuses more readings, that the higher refusal rate is intended, and gives 'expected 3% of newly refused readings'. The comment on check_outcome_state is updated so it no longer says the other two jobs are outside any check.
Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.
CHANGELOG.md has an entry that names the root cause and the new check.
Outcome check: the corpus replay of the three cited job requests (g6vy, 7bf8, 0wtc) shows each one is now caught by a check when the reading does not satisfy it: g6vy by check_outcome_state, 7bf8 by check_defined_terms, and 0wtc by check_defined_terms if its request uses a structural word. If a request has no structural word, the test asserts that count and says it is outside the class. Run `cargo test -p athena-core`.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
