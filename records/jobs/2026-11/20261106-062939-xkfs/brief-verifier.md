# Job 20261106-062939-xkfs: verify "Send unreconciled decisions back to the producer once before failing"

## Request (verbatim)

> When the decision gate finds unreconciled decisions, send the change back to the producer instead of ending the job.
> 
> The gate in crates/athena-core/src/run/checks.rs (unreconciled_decisions) runs before the pull request. Today, when a change touches a path a decision governs and adds no reconciling entry, the job ends as failed with the gate's message. The fix is usually one appended log entry and one [[link]], which the producer can add in a minute. Tonight a test-only change to crates/athena-git/src/snapshot.rs ended this way because decision D08 governs that file.
> 
> Change:
> - the producer brief lists the decisions that govern the files the plan expects to change, with the rule for reconciling them, so the producer can add the entry the first time
> - when the gate still finds unreconciled decisions, the producer gets one send-back with the gate's message, as for unmet acceptance criteria; only if it is still unreconciled after that does the job end
> 
> Acceptance:
> - a test: a candidate that touches a governed path with no entry is sent back once; the producer's second candidate adds the entry and link and the job proceeds to checks
> - a test: still unreconciled after the send-back ends the job with today's message
> - a test: the producer brief names the governing decisions for a planned file
> - CHANGELOG states only what the diff does
> 
> Files changed: the gate's call site in the run loop, the producer brief, tests, CHANGELOG.md.

## Reading (accepted by the user)

When the decision gate (unreconciled_decisions) finds unreconciled decisions, the producer gets one send-back carrying the gate's message, as for unmet criteria, and the job fails with today's message only if it is still unreconciled. The producer brief also lists the decisions governing the files the plan expects to change, with the reconcile rule, so the producer can add the entry the first time. Audience: Athena operators whose jobs now die on a one-entry fix.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs (check_and_verify calls self.unreconciled_decisions after producer_loop and candidate_stage and ends the job via self.failure; dispatch builds producer_brief_with_evidence, then producer_prompt_with_plan appends the plan section), crates/athena-core/src/run/checks.rs (unreconciled_decisions builds the GATE_REASON_PREFIX message), crates/athena-core/src/run/producer.rs (producer_loop with PRODUCER_SEND_BACKS and send_back_text), crates/athena-core/src/run/send_back.rs (send_back_text_with_replay), crates/athena-data/src/decision_links.rs (parse_governs, Governs::covers, unreconciled), crates/athena-context/src/briefs.rs (ProducerInputs, producer_brief_with_evidence, producer_prompt_with_plan).
- The gate runs on a committed candidate in check_and_verify, after the producer loop. The producer is sent back with the same one-send-back budget as unmet criteria. The producer loop can run the gate on the work tree's change list, or the send-back can be added at the call site; the producer chooses the smallest route that keeps the gate's text and the retry reader's GATE_REASON_PREFIX unchanged.
- The one send-back is shared with the unmet-criteria send-back: if the producer was already sent back for criteria, a still-unreconciled result does not earn a second send-back, and the job ends with today's message.
- The governing-decisions section is built from the base's docs/decision-links.toml [[governs]] list (Governs::covers) over planned_paths(plan, accepted), appended to the producer prompt after the plan section; with no list or no governed planned file, the brief is unchanged.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. Property is measured directly by fake-agent run tests in crates/athena-core/src/run/tests.rs and a brief test next to the existing producer-brief tests.
- This change adds no new refusal; it makes an existing refusal less final, so no expected N% replay figure applies and the replay is unaffected.
- docs/decisions.md and docs/decision-links.toml are not edited: D08 and other governed paths in the touched source files may need their own reconciling entry if the change touches governed files, which the producer handles under the gate's own rule.

## Acceptance criteria

1. [behavior] A run test (cargo test -p athena-core) where the producer's first candidate touches a path governed by a decision in the base docs/decision-links.toml and adds no log entry or link shows the producer invoked a second time with a send-back brief that contains the gate's message (GATE_REASON_PREFIX and the decision id); the second candidate adds the entry and [[link]] and the job proceeds to checks and the verifier.
2. [behavior] A run test (cargo test -p athena-core) where the producer's second candidate is still unreconciled ends the job as failed with exactly today's gate message (starting with GATE_REASON_PREFIX, naming each decision and touched path and the fix), after exactly one send-back and no third producer run.
3. [behavior] A brief test (cargo test -p athena-context or -p athena-core) shows the producer prompt for a plan that expects a governed file to change names the governing decision id, the governed path, and the rule (log entry plus matching [[link]] whose older is the decision and newer the new entry); a plan with no governed file adds no such section.
4. [content] The unreconciled_decisions message text and athena_data::decision_links::GATE_REASON_PREFIX are unchanged, so the retry reader (reading::retry_findings) still recognises a gated job; the existing gate tests in crates/athena-core/src/run/tests.rs still pass.
5. [content] CHANGELOG.md has an entry that says only what the diff does: the producer brief lists governing decisions for planned files, and an unreconciled candidate is sent back once with the gate's message before the job ends.
6. [behavior] cargo test, cargo clippy and cargo fmt --check pass on the candidate (Athena's recorded checks).
7. [content] Changes stay within the gate's call site in the run loop, the producer brief, tests and CHANGELOG.md; no change to athena.toml, defaults.toml, workflows, prompt files or records/.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-data/src/decision_links.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Reconciling D08 or any decision for crates/athena-git/src/snapshot.rs; that test-only change is not part of this job.
- Changing which paths a decision governs, or editing docs/decisions.md or docs/decision-links.toml.
- Changing the gate's rule in athena_data::decision_links::unreconciled or the gate's message text.
- Adding a second send-back, or changing the send-back for unmet criteria or the replay.
- Prompt files, athena.toml, defaults.toml and workflows.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Three new tests plus cargo test and clippy clean as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the send-back and the job's end by running tests, not that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that a one-entry reconcile no longer fails the job, shown by the second-candidate test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the call site, the brief, tests and CHANGELOG.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The send-back is enforced in the run loop and covered by tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run test over a repository whose base list governs README.md by D02 (governed_repo(GOVERNING)). The fake producer edits README.md on its first turn; when its prompt contains '## Sent back' it also applies reconciling_edits() (a new D13 entry plus a [[link]] D02 to D13). Assert: producer count 2; the second prompt contains '## Sent back', GATE_REASON_PREFIX, 'D02 (README.md)' and '[[link]]'; the first prompt has no '## Sent back'; the job is not failed with the gate prefix, state Landed; verifier count 1; a UI line starting 'checks     ' and one 'producer   sent back: decisions not reconciled'. Run: `cargo test -p athena-core governed_edit_is_sent_back_once_and_the_second_candidate_reaches_the_verifier`. Observes: Two producer runs, the send-back prompt carries the gate's message, and the job reaches checks and the verifier.
  Fails without the behavior: Without the loop-level gate the first candidate ends the job at check_and_verify: producer count 1, verifier 0, state Failed with the gate prefix, so every assertion fails.
  Test seam: Entry point Athena::go (via the governed_job pattern, tests.rs:2252) driving producer_loop then check_and_verify. Fixtures governed_repo, athena_with, counting and editing exist (tests.rs:187, 2152, 2234). Step 7 adds the script `reconciling_on_send_back(counts, prompts)`, which wraps the existing editing/counting script and adds reconciling_edits() only when the prompt contains '## Sent back'. It is injected through the FakeAgentDriver script parameter of athena_with. The double only picks which file edits the fake producer makes; gate, send-back and loop are production code.
- Criterion 2: Same repository, driver `editing(counts, prompts, Vec::new())` that never reconciles. Assert producer count == 2 (not 3) and verifier 0; state Failed; reason starts with GATE_REASON_PREFIX, contains 'D02 (README.md)', does not contain 'D12', contains the fix text, and equals the literal message built from the constants (compared as governed_edit_closes_before_the_verifier_starts does). Assert the second prompt contains '## Sent back'. Assert the UI contains the exact line 'producer   decisions still not reconciled after the send-back; the job ends at the decision gate' and no line containing 'the verifier will judge it', no 'checks     ' line, and a line starting 'failed     decisions not reconciled:'. Run: `cargo test -p athena-core governed_edit_still_unreconciled_after_the_send_back_ends_with_the_gate_message`. Observes: Exactly 2 producer runs, no verifier, the gate-specific limit line present and the 'verifier will judge it' line absent, and the closing reason is today's gate message.
  Fails without the behavior: With no send-back the producer count is 1; with a second send-back it is 3; without the gate-specific branch the exact line is missing; a changed message fails the equality and prefix assertions.
  Test seam: Athena::go with the existing governed_job, editing and Counts (tests.rs:2234-2274), which exist today. The only edits to the existing tests: the producer assertions at tests.rs:2295 and 2320 change from one producer run to two. Nothing is injected beyond the fake script; the Lines UI capture exists (tests.rs:2373).
- Criterion 3: (a) In athena-context, unit tests on governing_decisions_section / producer_prompt_with_governing with entries for D08 and the governed path crates/athena-git/src/snapshot.rs: the result contains 'D08', the path, 'docs/decisions.md', 'docs/decision-links.toml', '[[link]]', 'older' and 'newer', and not an ungoverned path; empty entries leave the prompt exactly equal to the input. (b) In athena-data, governing() unit tests (match, directory prefix, no match, empty list, bad list). (c) In athena-core, the_producer_prompt_names_the_decisions_governing_the_planned_files: the first producer prompt of the governed job (README.md governed by D02) contains the section naming 'D02' and 'README.md'; a second job over a repository with no list has a prompt without the section heading. Run: `cargo test -p athena-context governing_decisions && cargo test -p athena-data governing && cargo test -p athena-core the_producer_prompt_names_the_decisions_governing_the_planned_files`. Observes: The prompt names the decision id, the governed path and the reconcile rule; with nothing governed it is unchanged.
  Fails without the behavior: Without the section the prompt has no 'D02' (read from the prompts mutex) and the unit tests' contains assertions fail.
  Test seam: Production functions athena_data::decision_links::governing and athena_context::briefs::producer_prompt_with_governing (added by plan steps 1 and 2) and dispatch's call to them (step 6). Tests (a) and (b) are pure functions over literal list strings. Test (c) reuses governed_repo, editing and the prompts capture, which exist today (tests.rs:2234). No logic in the double.
- Criterion 4: Run the existing gate and retry tests with their message assertions unchanged, and the retry-reader tests. Edits to existing tests are limited to producer counts, prompt counts/indexes, one added producer-count assertion, and the slice cut in the retry test. In governed_check_is_skipped_for_a_resume_at_the_verifier keep the WARNING count == 1 on the first run and add assert_eq!(counts.all().1, 1) (the unreadable list sends no one back). In the retry test, take prompts.len() == 4, use prompts[2] as the retry's first brief, and cut `block` at the first '\n\n## ' after the findings heading (as reading.rs:5960 does) so D02/README.md assertions check only the findings block. The criterion-2 equality assertion pins the message text; git diff shows no change to the format! text or GATE_REASON_PREFIX. Run: `cargo test -p athena-core governed && cargo test -p athena-core a_retry_of_a_gated_job && cargo test -p athena-core retry_findings && cargo test -p athena-data decision_links`. Observes: All pass. The gated job's reason still starts with GATE_REASON_PREFIX and names the decision, path and fix; the unreadable-list warning is printed exactly once; the retry brief's findings block still carries D02, README.md and 'decisions not reconciled:'.
  Fails without the behavior: A changed prefix or message fails the starts_with/contains assertions; a loop warning would make the WARNING count 2; a loop send-back on an unreadable list would make the producer count 2.
  Test seam: Existing tests; Athena::go and retry_findings are the production entry points. The only addition is the extra assertion and slice cut described above. No new seam.
- Criterion 5: Read the CHANGELOG.md diff: one entry under '## [Unreleased]' stating only what the diff does: the producer prompt lists, for each planned file a decision governs, the decision id, the path and the rule for reconciling it; a candidate that leaves governed code without a log entry and link is sent back once with the gate's message, sharing the unmet-criteria send-back, before the job ends as failed. No forecast or outcome claim. Run: `git diff -- CHANGELOG.md`. Observes: The diff adds only that entry and its wording matches the code.
- Criterion 6: Run Athena's three recorded checks on the candidate. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0 and the new tests are included in the run.
  Fails without the behavior: A formatting slip, a clippy warning (unused helper, needless clone) or a failing test fails the corresponding check.
  Test seam: Cargo itself; nothing injected.
- Criterion 7: Inspect changed paths against the allowed list. Run: `git status && git diff --stat`. Observes: Modified files are only run/*.rs, briefs.rs, decision_links.rs, tests and CHANGELOG.md. athena.toml, defaults.toml, workflows, prompt files, records/, docs/decisions.md, docs/decision-links.toml, athena-git/src/snapshot.rs and run/candidate.rs are absent, so D08 (which governs snapshot.rs and candidate.rs) does not fire on this change's own gate.

Planned files:

- crates/athena-data/src/decision_links.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f2cc3885a23d → candidate 961d2a49bea5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   9 ++
 crates/athena-context/src/briefs.rs      |  77 ++++++++++++++
 crates/athena-core/src/run/checks.rs     | 142 ++++++++++++++++++-------
 crates/athena-core/src/run/mod.rs        |  20 +++-
 crates/athena-core/src/run/producer.rs   |  63 +++++++----
 crates/athena-core/src/run/send_back.rs  |  57 ++++++++++
 crates/athena-core/src/run/tests.rs      | 173 +++++++++++++++++++++++++++++--
 crates/athena-data/src/decision_links.rs |  98 +++++++++++++++++
 8 files changed, 571 insertions(+), 68 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-062939-xkfs/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-062939-xkfs/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42192103313 for 961d2a49bea53f5df71d325382066dd292c5d548)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42192103313 for 961d2a49bea53f5df71d325382066dd292c5d548)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42192103313 for 961d2a49bea53f5df71d325382066dd292c5d548)
- ci-result: `not a check` passed (Check results for 961d2a49bea53f5df71d325382066dd292c5d548 come from the CI run https://github.com/doodla/athena/actions/runs/42192103313 (468s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy and cargo fmt --check pass on the candidate (Athena's recorded checks).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core governed_edit_is_sent_back_once_and_the_second_candidate_reaches_the_verifier`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-data/src/decision_links.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `athena_context::briefs::GOVERNING_HEADING`
    --> crates/athena-core/src/run/tests.rs:2449:13
     |
2449 |         use athena_context::briefs::GOVERNING_HEADING;
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `GOVERNING_HEADING` in `briefs`

error[E0425]: cannot find function `send_back_text_full` in this scope
   --> crates/athena-core/src/run/send_back.rs:364:13
    |
 90 | pub(crate) fn send_back_text(brief: &str, unmet: &[String], unchanged: &[String]) -> String {
    | ------------------------------------------------------------------------------------------- similarly named function `send_back_text` defined here
...
364 |             send_back_text_full("BRIEF", &unmet, &unchanged, None, None),
    |             ^^^^^^^^^^^^^^^^^^^
    |
help: a function with a similar name exists
    |
364 -             send_back_text_full("BRIEF", &unmet, &unchanged, None, None),
364 +             send_back_text("BRIEF", &unmet, &unchanged, None, None),
    |

error[E0425]: cannot find function `send_back_text_full` in this scope
   --> crates/athena-core/src/run/send_back.rs:368:20
    |
 90 | pub(crate) fn send_back_text(brief: &str, unmet: &[String], unchanged: &[String]) -> String {
    | ------------------------------------------------------------------------------------------- similarly named function `send_back_text` defined here
...
368 |         let with = send_back_text_full("BRIEF", &unmet, &unchanged, None, Some(message));
    |                    ^^^^^^^^^^^^^^^^^^^
    |
help: a function with a similar name exists
    |
368 -         let with = send_back_text_full("BRIEF", &unmet, &unchanged, None, Some(message));
368 +         let with = send_back_text("BRIEF", &unmet, &unchanged, None, Some(message));
    |

error[E0425]: cannot find function `send_back_text_full` in this scope
   --> crates/athena-core/src/run/send_back.rs:377:20
    |
 90 | pub(crate) fn send_back_text(brief: &str, unmet: &[String], unchanged: &[String]) -> String {
    | ------------------------------------------------------------------------------------------- similarly named function `send_back_text` defined here
...
377 |         let only = send_back_text_full("BRIEF", &[], &[], None, Some(message));
    |                    ^^^^^^^^^^^^^^^^^^^
    |
help: a function with a similar name exists
    |
377 -         let only = send_back_text_full("BRIEF", &[], &[], None, Some(message));
377 +         let only = send_back_text("BRIEF", &[], &[], None, Some(message));
    |

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 4 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core governed_edit_still_unreconciled_after_the_send_back_ends_with_the_gate_message`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-data/src/decision_links.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `athena_context::briefs::GOVERNING_HEADING`
    --> crates/athena-core/src/run/tests.rs:2449:13
     |
2449 |         use athena_context::briefs::GOVERNING_HEADING;
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `GOVERNING_HEADING` in `briefs`

error[E0425]: cannot find function `send_back_text_full` in this scope
   --> crates/athena-core/src/run/send_back.rs:364:13
    |
 90 | pub(crate) fn send_back_text(brief: &str, unmet: &[String], unchanged: &[String]) -> String {
    | ------------------------------------------------------------------------------------------- similarly named function `send_back_text` defined here
...
364 |             send_back_text_full("BRIEF", &unmet, &unchanged, None, None),
    |             ^^^^^^^^^^^^^^^^^^^
    |
help: a function with a similar name exists
    |
364 -             send_back_text_full("BRIEF", &unmet, &unchanged, None, None),
364 +             send_back_text("BRIEF", &unmet, &unchanged, None, None),
    |

error[E0425]: cannot find function `send_back_text_full` in this scope
   --> crates/athena-core/src/run/send_back.rs:368:20
    |
 90 | pub(crate) fn send_back_text(brief: &str, unmet: &[String], unchanged: &[String]) -> String {
    | ------------------------------------------------------------------------------------------- similarly named function `send_back_text` defined here
...
368 |         let with = send_back_text_full("BRIEF", &unmet, &unchanged, None, Some(message));
    |                    ^^^^^^^^^^^^^^^^^^^
    |
help: a function with a similar name exists
    |
368 -         let with = send_back_text_full("BRIEF", &unmet, &unchanged, None, Some(message));
368 +         let with = send_back_text("BRIEF", &unmet, &unchanged, None, Some(message));
    |

error[E0425]: cannot find function `send_back_text_full` in this scope
   --> crates/athena-core/src/run/send_back.rs:377:20
    |
 90 | pub(crate) fn send_back_text(brief: &str, unmet: &[String], unchanged: &[String]) -> String {
    | ------------------------------------------------------------------------------------------- similarly named function `send_back_text` defined here
...
377 |         let only = send_back_text_full("BRIEF", &[], &[], None, Some(message));
    |                    ^^^^^^^^^^^^^^^^^^^
    |
help: a function with a similar name exists
    |
377 -         let only = send_back_text_full("BRIEF", &[], &[], None, Some(message));
377 +         let only = send_back_text("BRIEF", &[], &[], None, Some(message));
    |

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 4 previous errors
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-context governing_decisions && cargo test -p athena-data governing && cargo test -p athena-core the_producer_prompt_names_the_decisions_governing_the_planned_files`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core governed && cargo test -p athena-core a_retry_of_a_gated_job && cargo test -p athena-core retry_findings && cargo test -p athena-data decision_links`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
