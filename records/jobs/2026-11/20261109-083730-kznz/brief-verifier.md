# Job 20261109-083730-kznz: verify "Require reader questions to be precise customer decisions with defaults"

## Request (verbatim)

> Also, clarifications. I'm the customer essentially. Athena is allowed to ask/clarify what I want. But my time is valuable, so it should know exactly what it needs from me before coming to me.

## Reading (accepted by the user)

The customer wants Athena free to ask him clarifying questions, but only once the repository and defaults can't settle the point, and with the exact decision, alternatives and fallback default stated. I read this as a refusal rule on the reader's `question` field, mirroring the planner's `User:`/`Host:` open-questions rule, plus tests, a CHANGELOG entry and a doc note.

Assumptions:
- Goal and audience: the customer (Sri) wants Athena to be able to ask him questions, but only after it has worked out exactly what it needs, so his time is spent on real decisions. The reader's `question` field is the place where this is enforced today and it is not checked. Planner open questions are already gated by admission::open_questions_findings (User:/Host: prefix).
- Code survey: read crates/athena-core/src/admission.rs (open_questions_findings, has_addressee, says_code_answers; the planner-side rule is called from plan_findings), crates/athena-core/src/reading.rs (build_reading at ~line 702, `question` handling at ~lines 762 and 1202, many tests that set `question` to short strings) and crates/athena-core/src/reading/{terms,outcome,surveys,checks}.rs (each skips checks when `reading.question.is_some()`). The reader entry point is build_reading, which first builds the Reading from the model output and then runs the per-check functions. The new rule is a check run there and measures the question text directly. It adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. The refusal corpus in crates/athena-core/tests/refusal_corpus.rs (open_question_cases) is the model for corpus tests.
- The ask is read as: apply to the reader the same discipline as the planner (settle what the repository can settle, ask only what only the customer can decide, with alternatives and a default). The reader's reading must still be complete under its stated assumption, as the reader brief already says.
- Reuse admission's has_addressee logic (make it pub(crate) or share it) rather than duplicate it. This is an extension of existing code, not a new tool.
- Prompt files are out of scope because the reader prompt has only 71 tokens of headroom. The refusal finding itself feeds back the required format.
- Consumers: none found (no config deliverable).

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs (or a new file under crates/athena-core/src/reading/) holds a reader-question rule, wired into build_reading, that gives a finding for a question that does not begin with `User:` (same prefix normalisation as admission::has_addressee), offers fewer than two concrete alternatives, or names no default the job runs on if unanswered. A Grep for the rule's function name finds its definition and its call site.
2. [behavior] `cargo test -p athena-core` passes. New unit tests show that a question like `User: Which greeting: (a) hello or (b) hi? Default: (a).` is accepted. Questions with no `User:` prefix, with no alternatives, or with no default are each refused. A question that says the code answers it is refused. A reading with no question is unaffected.
3. [content] The rule's doc comment says the change refuses more readings and the higher refusal rate is intended, and states `expected 3% of newly refused readings`. The reason given is that questions the repository or defaults could settle cost the customer time.
4. [behavior] The refusal finding text, shown by the new tests, tells the reader what to do: settle the question from the repository or defaults, or rewrite it as `User:` + the decision + the alternatives + the default. An outcome check: a stranger reading the finding knows exactly what a question must contain before it reaches the customer.
5. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes. The existing reading tests that set `question` to short strings such as `which file?` are updated to the new format and still pass.
6. [content] CHANGELOG.md has an entry for the rule. docs/design/job-planning.md or another existing doc under docs/ records that the reader's question follows the same addressee rule as the planner's open questions, with the alternatives.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading/questions.rs (create)

## Out of scope

- Changing how the customer's answers are delivered or stored
- Planner open-questions rule behavior
- Prompt files (reader token headroom is 71)
- athena.toml, defaults.toml and workflow changes
- Learned-defaults or 'dumb context questions' memory features

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the intent from the existing planner open-questions rule instead of asking the customer.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and clippy/fmt criteria for the new rule.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The reader's question has no gate, so the rule is added at the reader boundary, mirroring the planner fix.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check accepted and refused question texts by running tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Corpus-file test over the gate's source, plus a Grep. Production entry point: `refusing_check` (reading/checks.rs:518), reached from `Athena::read_for` through `check_reading_acceptance` straight after `build_reading` (reading.rs:659-689). Run: `cargo test -p athena-core --test refusal_corpus reader_question_check_is_wired_into_the_gate`. Observes: The test passes. It asserts that `reading_source()` contains `pub fn check_reader_question(` (defined in reading/questions.rs, added to the `include_str!` list in step 8). It asserts that `acceptance_check_names(reading_source())` lists `check_reader_question` once, read from the `step!(check_...` lines of `refusing_check`. A Grep for `check_reader_question` finds the definition in crates/athena-core/src/reading/questions.rs and the call site at the first `step!` line of `refusing_check` in crates/athena-core/src/reading/checks.rs. `read_for` reaches that line right after `build_reading`, so the call site is on the production path the criterion describes. `every_acceptance_check_has_a_refusing_corpus_case` passes in the same run.
  Fails without the behavior: Without the definition or the `step!` line, the wiring test fails; with the line but no refusing case, `every_acceptance_check_has_a_refusing_corpus_case` fails.
  Test seam: Existing: `reading_source()` and `acceptance_check_names` in refusal_corpus.rs (the helper used by `every_acceptance_check_has_a_refusing_corpus_case`). Step 8 adds the `include_str!` of questions.rs and the new test. No double.
- Criterion 2: Unit tests drive `check_reading_acceptance` and `refusing_check` with real `Reading` values. One test goes through the real `build_reading` first. Each refused variant differs from the accepted question by one part. The corpus runs the same cases, one per input form. Run: `cargo test -p athena-core`. Observes: All tests pass, including: `reader_question_accepts_a_user_decision_with_alternatives_and_a_default` (`User: Which greeting: (a) hello or (b) hi? Default: (a).` accepted; also `(1)`/`(2)`, `**User:**`, `user:`, and `file(s)` beside real labels). `reader_question_refuses_a_question_missing_any_one_part` shows each of these refused by `check_reader_question`: no `User:` prefix, `Host:`, `Username:`, no alternatives, one alternative, duplicate alternatives, unlabelled 'hello or hi', no `Default:`, a default naming no alternative, `Which file(s)? Default: (s).`, and 'by reading the code'. `a_reading_without_a_question_is_not_checked_by_the_question_rule` shows a `None` question is unaffected. `a_free_form_question_from_the_reader_is_refused_after_build_reading` builds a reading with `question: "which file?"` through `build_sized` and shows `check_reading_acceptance` refuses it. `corpus_cases_get_the_expected_outcome` runs the 17 new cases (map in step 8).
  Fails without the behavior: Without the step, the free-form, `Host:`, no-alternative and no-default questions would be accepted and the refusing tests would fail. A step that refused every question would fail the accepted-variant tests. A parser that took `(s)` as a label would fail the `file(s)` case.
  Test seam: Production entry points `check_reading_acceptance` and `refusing_check` (checks.rs:490, 518), and `Athena::build_reading` through the existing `build_sized` helper (reading.rs tests, uses `make_athena`, never runs an agent). Fixtures: existing `scoped_reading`/`governed_reading` in the reading.rs tests and `base_reading` in refusal_corpus.rs, with `.question` set. All exist today. `check_reader_question` and the new tests are added by steps 2-8. No double holds rule logic.
- Criterion 3: Corpus-file test reads the doc comment of `check_reader_question` and CHANGELOG.md, and reads the declared share the way the replay does. Run: `cargo test -p athena-core --test refusal_corpus reader_question_refusal_is_declared_intended`. Observes: The test passes. `replay_declaration` of a criterion containing 'expected 3% of newly refused readings' gives share 3. The doc comment, `///` lines collapsed, directly above `pub fn check_reader_question(`, contains 'This check refuses more readings; the higher refusal rate is intended', 'expected 3% of newly refused readings', 'estimate', and the cause: questions the repository or defaults could settle cost the customer time. CHANGELOG.md contains `check_reader_question`, 'higher refusal rate is intended' and 'expected 3% of newly refused readings'.
  Fails without the behavior: A missing or reworded doc comment or CHANGELOG entry fails the needle assertions.
  Test seam: Modelled on the existing `history_fixture_refusal_is_declared_intended` in refusal_corpus.rs (exists, ~line 2916). Step 8 adds the new test. It uses `include_str!` of questions.rs and CHANGELOG.md; `replay_declaration` exists in athena-core. No double.
- Criterion 4: A unit test refuses a free-form question through the gate and asserts the text of the error. Run: `cargo test -p athena-core reader_question_error_tells_the_reader_what_to_write`. Observes: The test passes. The error from `check_reading_acceptance` for `User: Which file?` names the gaps ('alternatives', 'default'), quotes the question, says to settle the question from the repository or the defaults and leave `question` null, and gives the form `User:` + the decision + alternatives labelled `(a)`, `(b)` + `Default: (a)` with the example `User: Which greeting: (a) hello or (b) hi? Default: (a).`. A second reading with several gaps (no prefix, no alternatives, no default) shows all gaps listed in one error.
  Fails without the behavior: A refusal that only said 'question refused' lacks the settle-it and required-form phrases the test asserts, so the test fails.
  Test seam: Production entry `check_reading_acceptance` (checks.rs:490) with a `Reading` from the existing `scoped_reading` helper. The error text is produced by `check_reader_question` (step 2). No double.
- Criterion 5: Run fmt and clippy and the updated tests. The reading.rs tests that reach the gate with a question (lines 5787, 5958 including its `{word}` loop, 6048, 6179, 6983, 7942, 8557 in the base snapshot) use a shared valid `User:` question. Tests that call one check directly (4299, 4761, 5023, 7265, 7588, 8155) pass unchanged. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy exits clean. Also run `cargo fmt --all --check` (no diff) and `cargo test -p athena-core reading::` (the updated question tests pass), `cargo test -p athena-context` (the reader-brief rules test, now with 12 rules), and `cargo test --locked` (other crates unaffected).
  Fails without the behavior: Without the test updates, `check_reading_acceptance(&asking, ...).unwrap()` (reading.rs ~5788) and the other gate-reaching question tests fail with the new refusal.
  Test seam: Production entry `check_reading_acceptance`/`refusing_check`. One shared `const VALID_QUESTION` in the reading.rs test module, added in step 5 (not step 8, per finding 4). No double.
- Criterion 6: Corpus-file test reads CHANGELOG.md and docs/design/job-planning.md through `include_str!` and asserts the recorded text; a briefs test pins the reader rule line. Run: `cargo test -p athena-core --test refusal_corpus reader_question_rule_is_recorded_in_the_docs`. Observes: The test passes. The collapsed CHANGELOG.md contains `check_reader_question`. The collapsed docs/design/job-planning.md contains 'reader_question', 'same addressee rule as the planner', 'open questions', 'alternatives' and 'Default:'. `cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line` shows the 12th rule line names `check_reader_question`.
  Fails without the behavior: Missing or reworded entries fail the needle assertions; a missing rules line fails the briefs test.
  Test seam: New corpus test (step 8) modelled on existing doc-needle tests in refusal_corpus.rs; briefs test exists and is extended in step 6. No double.

Planned files:

- crates/athena-core/src/reading/questions.rs (create)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 3 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 3% of newly refused readings (the job is held above the declared share plus 5 points).

- 20261109-080749-p0fs: the reading was refused because its question is not a precise decision for the customer: it does not begin with `User:`; it offers fewer than two distinct alternatives labelled (a), (b); it names no `
- 20261109-035221-jj8g: the reading was refused because its question is not a precise decision for the customer: it does not begin with `User:`; it offers fewer than two distinct alternatives labelled (a), (b); it names no `
- 20261108-023653-70wy: the reading was refused because its question is not a precise decision for the customer: it does not begin with `User:`; it offers fewer than two distinct alternatives labelled (a), (b); it names no `

## The change

Base 51482297f2ac → candidate 7adf410ca4bb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  17 +++
 crates/athena-context/src/briefs.rs         |  16 +-
 crates/athena-context/src/briefs/reader.rs  |   3 +-
 crates/athena-core/src/admission.rs         |  22 ++-
 crates/athena-core/src/reading.rs           | 170 ++++++++++++++++++++-
 crates/athena-core/src/reading/checks.rs    |   4 +-
 crates/athena-core/src/reading/questions.rs | 143 ++++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs  | 220 +++++++++++++++++++++++++++-
 docs/design/job-planning.md                 |  15 ++
 9 files changed, 590 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-083730-kznz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-083730-kznz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42596729441 for 7adf410ca4bb4e15db578e01934439ef58c5ac3f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42596729441 for 7adf410ca4bb4e15db578e01934439ef58c5ac3f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42596729441 for 7adf410ca4bb4e15db578e01934439ef58c5ac3f)
- ci-result: `not a check` passed (Check results for 7adf410ca4bb4e15db578e01934439ef58c5ac3f come from the CI run https://github.com/doodla/athena/actions/runs/42596729441 (764s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus reader_question_check_is_wired_into_the_gate`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/questions.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling winnow v1.0.4
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling itoa v1.0.18
   Compiling winnow v0.7.15
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling jiff v0.2.38
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling regex-automata v0.4.18
   Compiling tempfile v3.27.0
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
error: couldn't read `crates/athena-core/tests/../src/reading/questions.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:2823:13
     |
2823 |             include_str!("../src/reading/questions.rs"),
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: couldn't read `crates/athena-core/tests/../src/reading/questions.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:3136:10
     |
3136 |         &include_str!("../src/reading/questions.rs")
     |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `athena-core` (test "refusal_corpus") due to 2 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/questions.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error: couldn't read `crates/athena-core/tests/../src/reading/questions.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:2823:13
     |
2823 |             include_str!("../src/reading/questions.rs"),
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: couldn't read `crates/athena-core/tests/../src/reading/questions.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:3136:10
     |
3136 |         &include_str!("../src/reading/questions.rs")
     |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `athena-core` (test "refusal_corpus") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
error[E0425]: cannot find function `check_reader_question` in this scope
    --> crates/athena-core/src/reading.rs:4541:9
     |
4541 |         check_reader_question(&reading).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 1 previous error
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus reader_question_refusal_is_declared_intended`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/questions.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error: couldn't read `crates/athena-core/tests/../src/reading/questions.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:2823:13
     |
2823 |             include_str!("../src/reading/questions.rs"),
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: couldn't read `crates/athena-core/tests/../src/reading/questions.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:3136:10
     |
3136 |         &include_str!("../src/reading/questions.rs")
     |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `athena-core` (test "refusal_corpus") due to 2 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core reader_question_error_tells_the_reader_what_to_write`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core --test refusal_corpus reader_question_rule_is_recorded_in_the_docs`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
