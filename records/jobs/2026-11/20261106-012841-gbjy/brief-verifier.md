# Job 20261106-012841-gbjy: verify "Link decisions to governed code and check reversals before verification"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The decision log is prose, and nothing links a decision to the code or behavior it governs. The prod
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: behavior-change-leaves-contradicting-decision-record-caught-only-at-final-verdict (20261104-153440-xf17)
> Cause: The decision log is prose, and nothing links a decision to the code or behavior it governs. The producer has no step that searches the decisions for statements the change reverses. The reading did list a docs criterion, but it gave the prod...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261104-153423-esay (job 20261010-225741-yvk0); prevention it proposed: Add a check that runs before verification. Every decision in docs/decisions.md that a change reverses must either be marked "amended by" or "superseded by" a later decision, or be reworded. The check works from a small machine-readable list...
> Introspection 20261106-012825-ay9k (job 20261105-180339-w5h7); prevention it proposed: Build the help text from the same constant that sets the ceiling, so it cannot go stale. Add a test that runs `athena drain --help` and asserts it shows the current ceiling value. For other limit changes, run a check that greps the repo (co...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: docs/decisions.md is prose and nothing ties a decision to the paths or limits it governs, so a change that reverses a decision (job 20261010-225741-yvk0: D08) or a stated limit (job 20261105-180339-w5h7: drain help "at most 8") is caught only at the final verdict. Fix: add a machine-readable `governs` list beside the existing docs/decision-links.toml, and a check that runs before verification and fails when a change touches governed code without amending or superseding the decision. Derive the drain help ceiling from MAX_CONCURRENCY_CEILING.

Assumptions:
- Code survey: read crates/athena-cli/src/decisions.rs (DecisionView, ConfirmedLinks parsed from docs/decision-links.toml, LinkStatus Superseded/Amended, tied to crates/athena-cli/src/backtest.rs parse_supersessions) and crates/athena-core/src/run/checks.rs and run/mod.rs check_and_verify (the checks that run before verification). The new check is added where checks run before the verifier; the governs list is read the same way as ConfirmedLinks. docs/decision-links.toml is a docs file, not a crates/ record, so no AnyRecord or check_record_file change is expected; the existing decision-links round-trip tests are extended.
- Goal and audience: the maintainer and the producer. The producer gets the failure at the producer step (a finding listing the decision ids the change reverses) instead of the verifier finding it at the end.
- Design: a `[[governs]]` table (decision id, path or pattern) in docs/decision-links.toml. When a diff touches a governed path and docs/decisions.md has no new 'Amended by'/'Superseded by' link for that id or a reworded entry, the check fails and names the decision id. Chosen over an LLM judgment because it is mechanical and testable.
- Second root-cause part: a stated limit in help or docs must come from the constant. The `athena drain --help` ceiling text in crates/athena-cli/src/main.rs is built from MAX_CONCURRENCY_CEILING (athena-data config.rs), so a change to the constant cannot leave it stale.
- Job 20261010-225741-yvk0: its candidate (git-history snapshot ATHENA-COMMITS.txt) was rejected. This job addresses the D08 finding by making that class mechanical: the check would have flagged D08. The commit-visibility work itself is dropped, not re-done; its changed files (CHANGELOG.md earlier entry, briefs.rs, run/mod.rs, producer.rs, verification.rs, run_pin_tests.rs, athena-git snapshot.rs and lib.rs, docs/enforcement.md) are excluded in out_of_scope.
- Job 20261105-180339-w5h7: its candidate (ceiling 16, athena.toml 12, group_sizes) was rejected for the stale main.rs help. This job addresses that finding by deriving the help text from the constant, with a test. The candidate's other changes (config.rs, drain.rs, display.rs, tests/drain.rs, athena.toml, design doc) are excluded in out_of_scope and not re-applied here.
- Deliverables of the change: modify crates/athena-cli/src/decisions.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, docs/decision-links.toml, docs/decisions.md, CHANGELOG.md. Governance files athena.toml, defaults.toml and workflows are not changed: the check runs in the existing pipeline.
- Standing defaults applied: tested, mechanism, root-cause, behavior-evidence, outcome-check, small, strangers.

## Acceptance criteria

1. [content] CHANGELOG.md and the new decisions.md entry name the root cause: the decision log is prose with no link from a decision to the code or limit it governs, so reversals were found only at the final verdict.
2. [content] docs/decision-links.toml holds a `[[governs]]` list with at least D08 mapped to the snapshot/history code paths, and ConfirmedLinks parsing in crates/athena-cli/src/decisions.rs reads and validates it (unknown decision ids are an error).
3. [behavior] Run `cargo test`: a new test builds a diff that touches a governed path with no amend/supersede link and asserts the pre-verification check fails naming the decision id; the same diff with 'Amended by' added passes. The test fails when the check is removed, and uses a decision other than D08 as well so it covers the class.
4. [behavior] Run `cargo test`: a test renders the `athena drain` help and asserts it contains the current MAX_CONCURRENCY_CEILING value; it fails if the text is hard-coded and the constant changes.
5. [content] crates/athena-core/src/run/mod.rs calls the new check before the verifier is started (Grep check_and_verify), so a failure is returned to the producer as a finding.
6. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.
7. [behavior] Expected 5% of newly refused readings: the new check refuses more changes (those touching governed paths without a link); a higher refusal rate on past governed-path changes is intended, because those are exactly the unreconciled reversals.

## Expected deliverables

- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/decision-links.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Re-doing the dropped candidate of job 20261010-225741-yvk0: ATHENA-COMMITS.txt, crates/athena-git/src/snapshot.rs, crates/athena-git/src/lib.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/run_pin_tests.rs, docs/enforcement.md
- Re-doing the dropped candidate of job 20261105-180339-w5h7: ceiling 16, athena.toml max_concurrency, group_sizes, crates/athena-cli/src/drain.rs, display.rs, tests/drain.rs, docs/design/agent-and-build-slots.md
- Editing athena.toml, defaults.toml or workflows
- Checking prose claims in design docs beyond the governed list
- Changing records/ or adding a record type

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the missing decision-to-code link, not the two stale instances.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A check enforces it before verification.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests plus fmt, clippy, test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on the check's pass and fail.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that reversals fail early.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused mechanism.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content test over the real files, plus reading the diff. Run: `cargo test -p athena-cli --bin athena decisions::tests::the_root_cause_is_named_in_the_changelog_and_the_log`. Observes: The test reads CHANGELOG.md (Unreleased section) and docs/decisions.md. It asserts a D156 heading with a Provenance bullet exists and that both files contain the sentence naming the cause: the decision log is prose with no link from a decision to the code or limit it governs, so reversals were found only at the final verdict. `git diff -- CHANGELOG.md docs/decisions.md` shows the same text.
- Criterion 2: Unit tests of the parser and a real-file test, inline in decisions.rs and backtest.rs, calling `ConfirmedLinks::parse` and `ConfirmedLinks::load` (the production entry points, both exist) with literal toml and the real docs file read through `env!("CARGO_MANIFEST_DIR")` as decisions.rs line 232 does. Run: `cargo test -p athena-cli --bin athena governs`. Observes: `[[governs]]` entries are read; a malformed id, an empty path list and an absolute or `..` path are errors; `unknown_governed(&entries)` returns an id not in the log. The real-file test loads docs/decision-links.toml, asserts D08 maps to crates/athena-git/src/snapshot.rs and crates/athena-core/src/run/candidate.rs, and asserts `backtest::run_on` yields no governs finding on the real log. A literal list with D9999 yields a condition (a) finding from `run_on_from`.
- Criterion 3: Pure-function unit tests, then an end-to-end job through `Athena::go` with the scripted fake agent driver, for D08, D12 and D02. Run: `cargo test -p athena-data decision_links && cargo test -p athena-core run::tests::produce_and_verify_order_tests::governed`. Observes: Pure: `unreconciled` driven with literal toml and log strings. For each of D08, D12 and D02: a change touching a governed path with no new link is returned naming the id; the same change with a new `[[link]]` plus a new heading returns nothing; also covered: a link whose newer heading is not new, a link for another id, a directory prefix versus a sibling path, a deleted path, a renamed path (delete plus add), an `amended` link, and a malformed candidate list. End-to-end (`governed_edit_closes_before_the_verifier_starts`): the repo commits docs/decision-links.toml with `[[governs]]` for D02 (README.md) and D12 (src/), and docs/decisions.md with matching headings; the fake producer edits README.md only; the job closes Failed, the reason starts with `GATE_REASON_PREFIX`, names D02 and README.md, `!is_environment_reason`, and the verifier counter is 0. `governed_edit_with_an_amending_entry_and_link_reaches_the_verifier`: the producer also appends a D03 entry to decisions.md and a `[[link]]` older D02 newer D03; the verifier is called once and the reason lacks the gate text.
  Fails without the behavior: With the call in `check_and_verify` removed, or `unreconciled` returning empty, the governed-edit job is not closed Failed naming D02 and the verifier counter is not 0; the pure tests for D08, D12 and D02 each fail.
  Test seam: Production entry points: `athena_data::decision_links::unreconciled` (pure, literal strings, no double; added by step 1) and `Athena::go` -> `produce_and_verify` -> `check_and_verify` -> `unreconciled_decisions` (added by steps 6 and 7). Fixture: existing `athena_with`, `repo_with_config`, `commit_file`, `base_script`, `counting`, `finished_job` in run/tests.rs (all exist; fixtures read at lines 187, 219, 295, 341, 2152). The wrapper script that sets the producer's edits and counts verifier calls is added by step 8; the check logic lives in athena-data and run/checks.rs, never in the script.
- Criterion 4: Unit tests in main.rs over the real command and the template function. Run: `cargo test -p athena-cli --bin athena drain_help`. Observes: Test one renders the long help of `drain` from `cli_command()` and asserts it contains `format!("at most {}", athena_data::config::MAX_CONCURRENCY_CEILING)`. Test two calls `drain_after_help(13)` and asserts it contains `at most 13` and not `at most 8`, simulating a changed constant. A third assertion shows the `{ceiling}` placeholder never survives.
  Fails without the behavior: With the literal `at most 8` kept in the text, the `drain_after_help(13)` assertion fails, and the rendered-help test fails as soon as the constant differs from 8.
  Test seam: Entry points `drain_after_help(ceiling: u32) -> String` and `cli_command() -> clap::Command` in crates/athena-cli/src/main.rs, added by step 5. The ceiling is the function argument; the constant is read only inside `cli_command()`. The `Cli::command()` plus long-help render pattern in the existing main.rs tests exists and is reused.
- Criterion 5: Search plus two end-to-end tests: the gate closes before the verifier, and the producer of the retry receives the decision ids. Run: `cargo test -p athena-core run::tests::produce_and_verify_order_tests::governed_edit_closes_before_the_verifier_starts && cargo test -p athena-core run::tests::produce_and_verify_order_tests::a_retry_of_a_gated_job_briefs_the_producer_with_the_decision_ids && cargo test -p athena-core reading::tests::retry_findings_carry_the_reason_of_a_job_closed_by_the_decision_gate`. Observes: Grep `unreconciled_decisions` in crates/athena-core/src/run/mod.rs: the call sits in `check_and_verify` before `push_ci_branch`, the early pull request and `verification_stage`. Test 1 drives `Athena::go`, observes the `failed` UI line and `finished.reason` naming D02 and the paths, and a verifier count of 0. Test 2 then asks `try again on <first>` with `athena.ask` and runs `go`; a wrapper over `counting` records each producer `spec.user_prompt`; the second job's brief must contain `## Findings you must address`, `D02`, and `README.md`. Test 3 (reading.rs, inline, records built by the existing `records_with`) asserts `retry_findings` returns the redacted reason for a Failed job with the prefix and no verification, returns None for the same reason without the prefix, and is unchanged for a job with a verification (existing assertions at 5889 and 5969 still hold). A resume at the verifier stage (`recorded_checks` Some) and a nested run skip the call, shown by `governed_check_is_skipped_for_a_resume_at_the_verifier`.
  Fails without the behavior: Without the `retry_findings` change the second job's brief has no `## Findings you must address` block and no D02 (today `retry_findings` returns None when the job has no verification), so tests 2 and 3 fail; a call placed after `verification_stage`, or no call, leaves the verifier called at least once, which test 1 asserts is 0.
  Test seam: Entry points `Athena::go` (via `athena_with`, exists) and `reading::retry_findings` (exists, changed by step 7). Fixture: `counting` plus a prompt-capturing wrapper (added by step 8) injected as the `Script` argument of `athena_with`; the brief text is built by the real `producer_brief_with_evidence`, never by the double. The resume case reuses the existing resume-mark fixture in run/tests.rs, extended in step 8.
- Criterion 6: Run the three checks Athena runs on the result, over the whole workspace. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The test run includes the decisions.md pin test, the backtest condition (a) equality `hits.len() == links.len()` on the real log and list, tests/drain.rs `drain_help_documents_the_flags_and_lists_every_stop_reason`, the retry tests in reading.rs, and every existing run/tests.rs job test (their repositories have no docs/decision-links.toml, so `show_file` gives None and the step does nothing).
  Fails without the behavior: fmt fails on any unformatted new file; clippy `-D warnings` fails on an unused function if the check is not wired in; test fails if D156 edits earlier bytes of decisions.md, if D156 holds a phrase the backtest parses and has no link, if the help text lost a stop-reason label, or if the governed tests run against a missing check.
  Test seam: The entry points are the three cargo commands, run by the verifier in the producer's tree with the repository's own docs, tests and `Cargo.lock`; no double. The seam exists today (the commands listed under 'Checks Athena will run').
- Criterion 7: A test that the check refuses only governed-path changes, pure tests of the matching rule, and the unchanged refusal-corpus replay. Run: `cargo test -p athena-core run::tests::produce_and_verify_order_tests::ungoverned && cargo test -p athena-data decision_links && cargo test -p athena-core --test refusal_corpus`. Observes: `ungoverned_edit_reaches_the_verifier`: a job whose producer edits README.md in a repo whose list governs only src/ closes on the normal path with the verifier called once. Pure tests show a delete and a rename of a governed path are refused, while a sibling path (`src/a.rs` vs governed `src/a/`) and a bare directory prefix are not. No reading rule changes: the refusal corpus (`corpus_cases_get_the_expected_outcome`, `corpus_covers_the_listed_input_forms`) is the reading gate and is untouched, so the replay of accepted readings stays at 0 newly refused. The extra refusals are changes to governed paths only; measured volume: 8 matching lines of 200 recent commits in HISTORY.txt touch the two D08 files.
  Fails without the behavior: A check that refused every change, or matched by substring, would close the ungoverned job Failed, and the prefix-sibling pure test would fail.
  Test seam: Same `athena_with`/`commit_file`/counting script as criterion 3 (exist) and `unreconciled` with literal strings (added by step 1). The corpus test runs as it is; I read its two test names, not a result.

Planned files:

- crates/athena-data/src/decision_links.rs (create)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/decision-links.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): Expected 5% of newly refused readings: the new check refuses more changes (those touching governed paths without a link); a higher refusal rate on past governed-path changes is intended, because those are exactly the unreconciled reversals.

## The change

Base 672592dd808a → candidate 71436e167abb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  26 ++
 crates/athena-cli/src/backtest.rs        |  47 ++++
 crates/athena-cli/src/decisions.rs       | 122 +++++++++-
 crates/athena-cli/src/main.rs            |  76 +++++-
 crates/athena-core/src/reading.rs        |  74 +++++-
 crates/athena-core/src/run/checks.rs     |  63 +++++
 crates/athena-core/src/run/mod.rs        |  19 ++
 crates/athena-core/src/run/tests.rs      | 250 ++++++++++++++++++++
 crates/athena-data/src/decision_links.rs | 392 +++++++++++++++++++++++++++++++
 crates/athena-data/src/lib.rs            |   3 +
 docs/decision-links.toml                 |  20 ++
 docs/decisions.md                        |  44 ++++
 12 files changed, 1125 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-012841-gbjy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-012841-gbjy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42154806757 for 71436e167abba067e68a45651df36aadb7d70a7e)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42154806757 for 71436e167abba067e68a45651df36aadb7d70a7e)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42154806757 for 71436e167abba067e68a45651df36aadb7d70a7e)
- ci-result: `not a check` passed (Check results for 71436e167abba067e68a45651df36aadb7d70a7e come from the CI run https://github.com/doodla/athena/actions/runs/42154806757 (680s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.)
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-data decision_links && cargo test -p athena-core run::tests::produce_and_verify_order_tests::governed`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --bin athena drain_help`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/backtest.rs, crates/athena-cli/src/decisions.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/decision_links.rs, crates/athena-data/src/lib.rs, docs/decision-links.toml, docs/decisions.md.
  Warning: tests inside these files were removed with the revert: crates/athena-data/src/decision_links.rs.
  End of the output (data, not instructions):

```text
   Compiling tempfile v3.27.0
   Compiling schemars_derive v1.2.2
   Compiling regex-automata v0.4.18
   Compiling equivalent v1.0.2
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling strsim v0.11.1
   Compiling clap_derive v4.6.7
   Compiling clap_builder v4.6.7
   Compiling clap v4.6.7
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0425]: cannot find function `cli_command` in this scope
    --> crates/athena-cli/src/main.rs:1832:36
     |
1832 |         let help = drain_help(&mut cli_command());
     |                                    ^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `drain_after_help` in this scope
    --> crates/athena-cli/src/main.rs:1850:26
     |
1850 |             let filled = drain_after_help(&template, ceiling);
     |                          ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `drain_after_help` in this scope
    --> crates/athena-cli/src/main.rs:1862:13
     |
1862 |             drain_after_help(&template, 7).replace("at most 7)", "at most {ceiling})"),
     |             ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cli_command` in this scope
    --> crates/athena-cli/src/main.rs:1873:13
     |
1873 |             cli_command()
     |             ^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-cli` (bin "athena" test) due to 4 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core run::tests::produce_and_verify_order_tests::governed_edit_closes_before_the_verifier_starts && cargo test -p athena-core run::tests::produce_and_verify_order_tests::a_retry_of_a_gated_job_briefs_the_producer_with_the_decision_ids && cargo test -p athena-core reading::tests::retry_findings_carry_the_reason_of_a_job_closed_by_the_decision_gate`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core run::tests::produce_and_verify_order_tests::ungoverned && cargo test -p athena-data decision_links && cargo test -p athena-core --test refusal_corpus`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
