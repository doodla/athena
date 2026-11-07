# Job 20261101-061508-8w51: verify "Make readings state where each action runs and what data it reaches"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in reading.rs `mod tests` that reads source text. Doc comment: split include_str!("reading.rs") at `pub fn check_environment_reach`, take the contiguous `///` lines before it, strip each leading `///`, collapse white space. CHANGELOG: take include_str!("../../../CHANGELOG.md") from `## [Unreleased]` to the next `## [` line, split into items at lines starting `- `, collapse white space per item, pick the one item containing 'no step asking who performs each action, where, with what permissions, and what data it reaches'. In both texts assert that phrase, '20261024-220048-engm', '20261025-070538-8ra0', '20261025-034247-nm0k', 'higher refusal rate is intended', 'expected 8% of newly refused readings', 'estimate' and 'not measured'. Run: `cargo test -p athena-core environment_reach_doc_and_changelog_name_the_cause`. Observes: The doc comment and the new CHANGELOG item each name the cause, the three jobs, the intended higher refusal rate and the unmeasured 8% estimate, even when the phrase wraps across lines. Older entries cannot satisfy it.
  Fails without the behavior: With the doc comment or the CHANGELOG item missing, no item contains the phrase or a needle is absent and the assertion fails.
  Test seam: No production function is called; the test reads source text via include_str! of reading.rs and CHANGELOG.md, the same pattern as reading.rs:9142 (exists). The test is added by plan step 5.
- Criterion 2: Two tests. (a) In athena-core take the body of `refusing_check` (split at `pub fn refusing_check(`, as reading.rs:7728 does) and assert `step!(check_environment_reach(reading));` appears exactly once and directly follows `step!(check_outcome_state(reading, request));`. (b) In athena-context update `the_reader_brief_names_each_reading_rule_in_one_short_line` (briefs.rs:4400): `READING_RULES.len() == 7`, `check_environment_reach` in its check list, one line holding both `Environment:` and `check_environment_reach`, every line at most 220 bytes, section bound raised just above the measured size. Run: `cargo test -p athena-core environment_reach_has_one_step_line && cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line`. Observes: One step line in the stated position, and a reader rule line that tells the reader to write an `Environment:` assumption.
  Fails without the behavior: Without the step line the count is 0 and (a) fails. Without the rule line the rule count stays 6 and (b) fails.
  Test seam: No production function is called; tests read reading.rs text, the READING_RULES constant (briefs.rs:2225) and the reader brief builder (briefs.rs:2416). All exist; the new assertions are added by plan steps 5 and 7.
- Criterion 3: Unit tests `environment_reach_*` in reading.rs `mod tests`, calling `refusing_check` (3226) and `check_environment_reach` directly (guards: capability set, no question). Refuse with no `Environment:` assumption, asserting the refusing check name is `check_environment_reach` and the error starts 'the reading was refused because': (i) Create `.github/workflows/lint.yml` plus a Content criterion naming it; (ii) one Behavior criterion each: 'Athena pushes the job branch to origin', 'The job branch is pushed to the remote', 'A pull request is opened on the remote', 'The pull request is opened with `gh pr create`', a bare `git push` backtick; (iii) "Athena opens the job's PR at verification start", "Open the job's PR early at verification start", 'The early PR is reused by land', 'land calls the GitHub API to reuse the early PR', 'The check calls the GitHub API for the run status', 'The CI job writes the records'. Accept the same readings with 'Environment: the CI job has write permission to refs on the GitHub origin' for CI-site cases; for a local-site case ('the local Athena working copy has write access to the repo; with no origin or a non-GitHub origin it skips the push; with a GitHub origin it can push refs') `refusing_check` returns None. Also: an assumption missing the permission word is refused and the error names 'permissions'; a local-site assumption without the no-origin case is refused; a CI-site assumption without it is accepted; the label matches after trim and in any case; an assumption that only mentions 'environment' does not count; a Delete of a workflow does not trigger. Accept fixtures: Behavior criteria that contain `pr.rs`, gh/early.rs, lowercase 'pr', 'express', 'CIty', 'CI checks each commit' and 'CI tests the commits' do not trigger; 'CI commits the records' does (finding 4). Run: `cargo test -p athena-core environment_reach`. Observes: A workflow deliverable, or a push, pull-request, PR, GitHub API or CI-write action in a Behavior criterion, without a complete `Environment:` assumption is refused by `check_environment_reach`; with one, `refusing_check` gives None. Verb forms opens, opening, reused, merged, raised are all covered (finding 5).
  Fails without the behavior: With the `step!` line removed, `refusing_check` returns None for the refused readings and the check-name assertions fail. With the label or word lists wrong the accept assertions fail. With the PR/API/CI rules removed the PR and API fixtures are accepted and fail. With 'commit' as a plain word after CI the 'CI checks each commit' accept fixture fails.
  Test seam: Production entry points `refusing_check` and `check_environment_reach` (the latter added by plan step 4). Fixtures are `Reading` values from `governed_reading` (reading.rs:7364) and `add_behavior` (7505), which exist, plus a `with_environment` helper added in plan step 5. No test double.
- Criterion 4: Unit tests `environment_reach_source_*` in reading.rs, each also run through `refusing_check` to assert the refusing check's name. Three readings shaped like the three jobs. (engm, C1 runtime read) a code reading with a `Code survey:` assumption naming a crates/ path (so `check_code_survey` passes) and the Behavior criterion 'The history check reads the job records on the records branch': refused with no Environment assumption; refused with an assumption missing the permission word; accepted with 'Environment: the CI job has read-only access to contents and cannot write the records branch; the producer snapshot has no records/'. (8ra0) the push/origin/PR reading of criterion 3, accepted only with a complete assumption. (nm0k, C2 evidence) a docs reading whose summary says 'Use the figures from job records and history': refused with no assumption; refused when the only Environment file is `records/foo.json`, `athena/records/x.json` or `crates/athena-core/src/reading.rs`, or the only number is a year; accepted when any assumption quotes '3 jobs' or '40%', or an `Environment:` assumption lists `docs/design/roadmap.md`. Summary 'Use the figures from history' is refused; 'the history of the page' is accepted. Phrase loop: each C1 phrase (records branch, `athena/records` bare and backticked, git history, git log, commit history) with a source verb in a Behavior criterion, and each C2 phrase (job records, job history, commit history, git history, git log, records branch, athena/records, history) with a source verb in the summary, is refused without the repair. Accept (finding 1 fixtures copied from doc wording): Behavior 'Run `athena status`: rounds per landed job are computed from the job records', 'Scores a job from its records', 'The tool reads the records', 'The audit reads records'; 'The record is written in the records branch'; 'a reading citing job records'; a summary 'The audit counts readings in records' (stated limit). A reading with no deliverables and a records-branch Behavior read is refused. Run: `cargo test -p athena-core environment_reach_source`. Observes: A reading that reads a located source at runtime is refused unless an Environment statement says where it runs, its permissions and the data reached; a reading whose summary cites records or history as evidence is refused unless counted figures are quoted in an assumption or a reachable file is listed in an Environment assumption; ordinary product wording about reading records is accepted. Three differently shaped readings plus the phrase loops cover the class.
  Fails without the behavior: Without the step, `refusing_check` returns None for the refused readings and the name assertions fail. Without the figure and file rule, the bare-year and records-only-file readings are wrongly accepted. Without the C1/C2 split, the status.rs and scoring.rs wording fixtures are refused (or the engm shape needs filler figures) and fail.
  Test seam: Production entry points `refusing_check` and `check_environment_reach`. Fixtures `code_reading` (reading.rs:8936) and `governed_reading` (7364) exist, edited in place; the summary is set on the fixture. No double. The deviation from criterion 4's literal wording is asserted by the C1 accept case and stated in the doc comment.
- Criterion 5: Plan step 6 adds eighteen cases to `reading_check_cases()` (refusal_corpus.rs:668) with `case(..)` and `base_reading` (exist, 186 and 117). Refuse, `Expected::Refuse("check_environment_reach")`: environment-ci-workflow-without-an-environment (Create `.github/workflows/lint.yml` plus a Content criterion naming it), environment-host-action-without-an-environment ('Athena pushes the job branch to origin'), environment-pr-without-an-environment ("Athena opens the job's PR at verification start"), environment-github-api-without-an-environment ('The check calls the GitHub API for the run status'), environment-ci-write-without-an-environment ('The CI job writes the records'), environment-records-branch-read-without-an-environment ('The history check reads the job records on the records branch'), environment-records-summary-without-figures (summary 'Use the figures from job records'), environment-bare-history-summary-without-figures (summary 'Use the figures from history'), environment-records-summary-with-a-bare-year (Environment names only a year). Accept: environment-ci-workflow-with-an-environment, environment-host-action-with-origin-cases, environment-records-branch-read-with-an-environment, environment-records-summary-with-quoted-figures, environment-records-summary-with-a-reachable-file (lists docs/design/roadmap.md, in `repo_files()`), environment-product-reads-records-accepted (status.rs and scoring.rs wording in Behavior criteria, no Environment), environment-trigger-words-only-inside-other-words (dash-bullet request naming push to origin, which becomes Content criteria, plus a Behavior criterion with 'originates', a gh/early.rs path, `parse_remote`, 'entries are pushed onto the list', a Vec push), environment-gh-module-path-only, environment-pr-inside-paths-only (`pr.rs`, gh/early.rs, 'the history of the page', 'written in the records branch'). Input-form map (ids read in the file: code-reading-without-a-survey 689, code-reading-with-a-survey 700, design-reading-without-existing-code 738, request-names-a-governance-file-the-reading-ignores 772, delivery-request-naming-a-workflow-fails-governance-first 393, full-deliverable-path 802, boundary-word-in-a-deliverable-path 890; star-bullets 542, numbered-dot-parts 554): workflow Create/Modify -> the two workflow cases; Delete -> unit test only; push, pull-request, PR, GitHub API, CI-write phrases -> the host-action cases; records-branch runtime read -> the records-branch pair; summary evidence with figure, file, year, bare history -> the summary cases; ordinary product wording -> product-reads-records; slash-joined, path and identifier tokens, words inside other words, PR inside paths -> the accept cases; dash bullets -> the trigger-words case; star and numbered bullets reach the gate only as Content criteria through `base_reading`, which the check does not scan, and are covered by the existing bullet cases. Seven new `INPUT_FORMS` tags: ci-workflow-deliverable, host-action-criterion, records-branch-read, records-evidence-citation, environment-assumption, trigger-word-inside-other-word, pr-or-api-wording (length 23 to 30), each carried by a case. Add `environment_reach_refusal_is_declared_intended`, modelled on `design_existing_code_refusal_is_declared_intended` (1102). I have not run the suite and cite no result. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Each case gets its expected outcome, and `every_acceptance_check_has_a_refusing_corpus_case` (1061) and `corpus_covers_the_listed_input_forms` (998) pass because a case refuses with `check_environment_reach` and every tag is carried.
  Fails without the behavior: Without the check there is no step line, so `an_extra_check_with_no_case_is_reported` (1082) and `environment_reach_refusal_is_declared_intended` fail. With the step present but not firing, the refuse cases fail with 'expected Refuse(check_environment_reach), got accepted'.
  Test seam: Production entry point `athena_core::reading::refusing_check` (public, used at refusal_corpus.rs:943). Fixtures `Case`, `case()`, `base_reading()` and `repo_files()` exist. Behavior cases replace `reading.acceptance_criteria` in the case body; summary cases set `reading.summary`. Cases are added by plan step 6. No double.
- Criterion 6: Unit tests `environment_reach_accepts_*` in reading.rs plus the corpus accept cases. A plain docs reading (`governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[])`) passes `check_environment_reach` and `refusing_check` returns None. Accepted with no `Environment:`: Content criteria naming push, origin, gh/early.rs, parse_remote, PR, 'the history of the page'; Behavior criteria containing only 'fixed where it originates', 'ghost', 'pushdown', 'remotely', 'Vec push', 'entries are pushed onto the list', 'The record is written in the records branch', a `pr.rs` path, status.rs/scoring.rs-style wording 'computed from the job records', 'Scores a job from its records'; this job's own criteria 3 to 7 and its summary copied from the brief; a push or PR phrase in an assumption or in out_of_scope only; a decline (capability None); a reading with a question. The 8% estimate is pinned by the criterion 1 and 5 declaration tests, and the replay declaration is checked with `replay_declaration_of` returning share Some(8) for this job's criterion text. Run: `cargo test -p athena-core environment_reach_accepts`. Observes: Docs-only readings and readings without an assigned action or located-source read are accepted. Words in paths, identifiers, Content criteria and other words do not trigger.
  Fails without the behavior: With substring or bare-token triggers, 'originates', gh/early.rs and parse_remote are refused and the accept tests fail. With single `records` as a Behavior trigger the status.rs wording is refused and fails. Without the capability and question guards the decline and question readings are refused.
  Test seam: Production entry points `check_environment_reach` (new) and `refusing_check` (exists). Fixtures `governed_reading` (7364) and `add_behavior` (7505) exist; Behavior criteria are set by editing the criterion kind in place. No double.
- Criterion 7: Run the three commands. The full workspace run is needed because `check_reading_text` (reading.rs:4128) is shared by the steward, the CLI check-reading command, run/tests.rs and drain.rs, and because workspace tests include the boundary scan and the budgets test. Any fixture elsewhere that the new rule refuses gets an `Environment:` assumption; the check is never weakened. Also at verification the live replay (run/mod.rs replay_stage) runs the new checker over up to 50 stored accepted readings: limit 13% with the declared 8%, hold at 7 or more newly refused of 50. If it holds, the send-back lists the readings; the producer reads each, tightens the trigger that fired most or adds a verb/phrase exclusion plus an accept corpus case, and does not weaken the A/B/C rules wholesale. Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets && cargo test --workspace`. Observes: All three exit 0.
  Fails without the behavior: A fixture reading in another crate that trips the new check fails `cargo test --workspace`. A briefs.rs array length or size bound left at 6 or 2400 fails the briefs tests.
  Test seam: The workspace build and test commands themselves; no fixture of my own. The replay is run by Athena at verification, not by this command.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 8% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 004eb9cf86a8 → candidate b6588abd80af. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  25 +
 crates/athena-context/src/briefs.rs        |  16 +-
 crates/athena-core/src/reading.rs          | 967 ++++++++++++++++++++++++++++-
 crates/athena-core/tests/refusal_corpus.rs | 288 ++++++++-
 4 files changed, 1290 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261101-061508-8w51/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261101-061508-8w51/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42381271023 for b6588abd80af128f2b1c5fceadf3196696bdafff)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42381271023 for b6588abd80af128f2b1c5fceadf3196696bdafff)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42381271023 for b6588abd80af128f2b1c5fceadf3196696bdafff)
- ci-result: `not a check` passed (Check results for b6588abd80af128f2b1c5fceadf3196696bdafff come from the CI run https://github.com/doodla/athena/actions/runs/42381271023 (731s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core environment_reach_doc_and_changelog_name_the_cause`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs.
  End of the output (data, not instructions):

```text
7952 |         let error = check_environment_reach(&local_only)
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:7983:9
     |
7983 |         check_environment_reach(&declined).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:7986:9
     |
7986 |         check_environment_reach(&asking).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8032:21
     |
8032 |         let error = check_environment_reach(&reading).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8125:13
     |
8125 |             check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8128:9
     |
8128 |         check_environment_reach(&summary).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8134:9
     |
8134 |         check_environment_reach(&plain).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8148:9
     |
8148 |         check_environment_reach(&content).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8168:13
     |
8168 |             check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8202:9
     |
8202 |         check_environment_reach(&reading).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 15 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core environment_reach_has_one_step_line && cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core environment_reach`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs.
  End of the output (data, not instructions):

```text
     |
7952 |         let error = check_environment_reach(&local_only)
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:7983:9
     |
7983 |         check_environment_reach(&declined).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:7986:9
     |
7986 |         check_environment_reach(&asking).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8032:21
     |
8032 |         let error = check_environment_reach(&reading).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8125:13
     |
8125 |             check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8128:9
     |
8128 |         check_environment_reach(&summary).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8134:9
     |
8134 |         check_environment_reach(&plain).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8148:9
     |
8148 |         check_environment_reach(&content).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8168:13
     |
8168 |             check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8202:9
     |
8202 |         check_environment_reach(&reading).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 15 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core environment_reach_source`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs.
  End of the output (data, not instructions):

```text
     |
7952 |         let error = check_environment_reach(&local_only)
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:7983:9
     |
7983 |         check_environment_reach(&declined).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:7986:9
     |
7986 |         check_environment_reach(&asking).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8032:21
     |
8032 |         let error = check_environment_reach(&reading).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8125:13
     |
8125 |             check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8128:9
     |
8128 |         check_environment_reach(&summary).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8134:9
     |
8134 |         check_environment_reach(&plain).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8148:9
     |
8148 |         check_environment_reach(&content).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8168:13
     |
8168 |             check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_environment_reach` in this scope
    --> crates/athena-core/src/reading.rs:8202:9
     |
8202 |         check_environment_reach(&reading).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 15 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core --test refusal_corpus`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core environment_reach_accepts`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check && cargo clippy --workspace --all-targets && cargo test --workspace`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
