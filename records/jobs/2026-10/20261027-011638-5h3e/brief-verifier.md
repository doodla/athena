# Job 20261027-011638-5h3e: verify "Require snapshot-checkable criteria in briefs and structure-pass §5"

## Request (verbatim)

> Add a reading and producer-brief rule that every acceptance criterion must be checkable from the snapshot, diff or commands the verifier runs, never from the producer's report or change description. Update docs/design/structure-pass.md section 5 so its required caller-check evidence is put in the snapshot (for example a file or test), not in the report. Acceptance: a test or policy check shows the reading brief contains the rule.
> 
> Acceptance:
> - The change is made as described: Reading writes criteria that the verifier can only check from the producer's report or change description, which it never sees (M8); jobs fail or land with an uncheckable criterion
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: docs/design/structure-pass.md

## Reading (accepted by the user)

Add to the reader brief and the producer brief a rule that each acceptance criterion is checkable from the snapshot, the diff or commands the verifier runs. Rewrite docs/design/structure-pass.md section 5 so the caller-check evidence (and similar report-held proof) lives in a committed file or test. Audience: the verifier that judges jobs, and maintainers reading the design doc.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (reader_brief at ~2256 with the 'Criteria evidence' rule line at ~2230 and 'What the verifier can run' at ~2429; producer_brief_with_evidence at ~626 calling criteria_check_section at ~746; existing test the_reader_brief_says_the_verifier_never_sees_the_producers_report at ~4588) and crates/athena-core/src/reading/checks.rs (check_criteria_evidence). No record or data file is added under crates/, so the records allow-list is not touched. The property is measured directly by rendering the brief strings and asserting on their text.
- The reader brief already says criteria rest on commands the verifier can check; the change makes the stated rule explicit (snapshot, diff or commands) with a test, and adds it to the producer brief. It does not add a refusal check, so no refusal-rate change is intended.
- The reader prompt file prompts/reader.md has 3 tokens of headroom and the verifier prompt 2, so the rule goes in the Rust-built briefs, not in prompt files.
- CHANGELOG.md gets one line if the repo convention requires it; no governance file (athena.toml, defaults.toml, workflows) changes.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-context`: a new test renders the reader brief via reader_brief and asserts it contains the rule that every acceptance criterion must be checkable from the snapshot, diff or commands the verifier runs; the test and all existing tests pass.
2. [behavior] Run `cargo test -p athena-context`: a new test renders producer_brief for a reading and asserts it contains the same rule, in a section that tells the producer to make its evidence a file or test in the snapshot; it passes.
3. [content] Grep for `Caller check` in docs/design/structure-pass.md shows item 5 of section 5 requires the old-path Grep counts to be put in the snapshot (a committed file or test) and no longer asks for them in a report; Grep `report` in section 5 shows no remaining requirement whose evidence lives only in a report or change description.
4. [content] `git diff` shows the existing reader brief wording and the left-to-Athena handling in the producer brief are unchanged apart from the added rule; no prompt file under crates/athena-context/prompts changes.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass with no new #[allow].
6. [behavior] No unrelated behavior changes: `git diff --stat` lists only briefs.rs, structure-pass.md and optionally CHANGELOG.md.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- docs/design/structure-pass.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing check_criteria_evidence or any refusal logic
- Editing prompt files under crates/athena-context/prompts
- Changing the moves G1-G8 and R1-R9 described in structure-pass.md, only the evidence rules of section 5

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both briefs; fmt, clippy and test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that render the briefs and check their text.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule, two briefs, one doc section.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the rule appearing in the rendered briefs, not the edit being made.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc and brief text written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test that renders the reader brief and checks its text. Run: `cargo test -p athena-context the_reader_brief_requires_criteria_checkable_from_the_snapshot && cargo test -p athena-context`. Observes: The rendered reader brief contains `# Checkable criteria`, the literal phrases `checkable from the snapshot, the diff or the commands the verifier runs` and `producer's report or change description`, and the section sits before `# What the verifier can run`. All existing athena-context tests also pass.
  Fails without the behavior: Without the section the contains assertions fail; they use literal phrases written in the test, not the constant, so deleting or changing the text fails the test.
  Test seam: Calls `reader_brief(&reader_inputs(&reading()))` (production entry point, briefs.rs ~2256; helpers reader_inputs ~4457 and reading() ~2486 exist). No fake or double; the verifier_can_run fixture is non-empty so the later section exists for the order check. Step 5 adds the test.
- Criterion 2: Unit test that renders the producer brief for an accepted reading and checks its text. Run: `cargo test -p athena-context producer_brief_requires_checkable_evidence_in_the_snapshot`. Observes: The producer brief contains `## Make your evidence checkable`, the same rule phrase as the reader brief, `a test in your change`, `only when a deliverable or a criterion names it`, `Your report is still where you state what you ran or read`, `report it unmet`, `not checkable by the verifier`, no `you commit` wording, the heading before `## criteria_check: copy the line`, and the LEFT_TO_ATHENA text `left to Athena's checks` still present.
  Fails without the behavior: Without the section the assertions on the heading, the rule phrase and the test-or-file instruction fail; with 'you commit' wording the negative assertion fails; without the status wording the `report it unmet` and `not checkable by the verifier` assertions fail.
  Test seam: Calls `producer_brief(&ProducerInputs{..})` (production entry point, briefs.rs ~626), built as in the existing test `producer_briefs_say_what_the_verifier_can_see` (~2710) with `AcceptedReading::new(&reading(), &defaults(), |_| false)`. Fixtures exist; no double. Step 6 adds the test.
- Criterion 3: Grep the doc for the rewritten evidence and read the hits, including that the item 3 exceptions survive. Run: `Grep pattern `Caller check` and pattern `report` in docs/design/structure-pass.md; then Grep `Flow::Stop|WRAPPER|D87` in the same file`. Observes: Item 5 says the proof is in the snapshot, that the committed tests (cargo test --locked) compile and run every hit, that the test named in 5.1 covers the PrHost fakes, that the verifier reruns the old-path Grep over crates/ and reads `git diff -U0 -- crates/`, and that the diff makes the counts equal by construction, with no count kept in a report; it has no 'cannot be rechecked' sentence. Every `report` hit between `## 5.` and `## 6.` is gone or says no item rests on the report. Item 3 still names the mod/use/pub use/visibility lines, the R5-R8 Flow::Stop and seam-struct lines and the G1 guard constant and D87/D88 paths as the allowed unmoved lines. Row 294 describes the early-return pairing read in the diff. 5.2 names `a_landing_touching_a_file_under_gh_selects_d87_and_d88` and asks for no new test.
- Criterion 4: Inspect the diff for removed lines and prompt paths. Run: `git diff`. Observes: In briefs.rs the hunks are added lines only (const, reader section, fn checkable_evidence_section and its call, two tests); VERIFIER_VISIBILITY, criteria_check_section, LEFT_TO_ATHENA, READING_RULES and the 'What the verifier can run' text show no removed lines; no path under crates/athena-context/prompts appears.
- Criterion 5: Run the three Athena checks from the repository root. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0, including system_facts_budgets_match_computed, the policy boundary scan and the reader-rules tests; `git diff` shows no added #[allow].
  Fails without the behavior: If the rule were put in READING_RULES, the len==8 and <2600-byte rules tests fail; if a prompt file changed, the budget test fails; banned words in the new text fail the policy scan; fmt or lint errors fail their commands.
  Test seam: The existing workspace test suite and the cargo fmt and clippy commands, run on the whole workspace from the repository root. No fixture or double. The seam exists; no step adds one.
- Criterion 6: List the changed paths in the producer's snapshot repository and compare them with the allowed files. Run: `git diff --stat`. Observes: The stat lists only crates/athena-context/src/briefs.rs, docs/design/structure-pass.md and CHANGELOG.md; `git status` shows no untracked or other modified file, nothing under crates/athena-context/prompts, and not crates/athena-core/src/reading/checks.rs.
  Fails without the behavior: If the producer also edited a prompt file, checks.rs, athena.toml or added an evidence file, `git diff --stat` or `git status` lists that path and the criterion fails.
  Test seam: Production entry point is `git diff --stat` and `git status` run in the producer's one-commit snapshot repository (Producer snapshot fact; working-directory text, briefs.rs ~656). The producer's edits are the input; no double. The seam exists (git is in the verifier's allowed commands). Step 9 runs it.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- docs/design/structure-pass.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9a77cd8c6b18 → candidate fb41b5630443. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  3 ++
 crates/athena-context/src/briefs.rs | 72 +++++++++++++++++++++++++++++++++++++
 docs/design/structure-pass.md       | 46 +++++++++++++++---------
 3 files changed, 104 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-011638-5h3e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-011638-5h3e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42474716992 for fb41b56304434fe3e057800da1cdc7e7a6433a5a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42474716992 for fb41b56304434fe3e057800da1cdc7e7a6433a5a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42474716992 for fb41b56304434fe3e057800da1cdc7e7a6433a5a)
- ci-result: `not a check` passed (Check results for fb41b56304434fe3e057800da1cdc7e7a6433a5a come from the CI run https://github.com/doodla/athena/actions/runs/42474716992 (764s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass with no new #[allow].)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-context the_reader_brief_requires_criteria_checkable_from_the_snapshot && cargo test -p athena-context`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context producer_brief_requires_checkable_evidence_in_the_snapshot`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, docs/design/structure-pass.md.
  End of the output (data, not instructions):

```text
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling generic-array v0.14.7
   Compiling zmij v1.0.23
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling syn v3.0.7
   Compiling libc v0.2.190
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling ref-cast v1.0.27
   Compiling winnow v1.0.4
   Compiling memchr v2.8.3
   Compiling getrandom v0.3.4
   Compiling cfg-if v1.0.5
   Compiling itoa v1.0.18
   Compiling thiserror v2.0.21
   Compiling serde_derive_internals v0.30.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling digest v0.10.7
   Compiling jiff-core v0.1.1
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling cpufeatures v0.2.17
   Compiling sha2 v0.10.9
   Compiling jiff v0.2.38
   Compiling schemars_derive v1.2.2
   Compiling toml v0.9.12+spec-1.1.0
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.76s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_context-6da838b4f8c2a29c)

running 1 test
test briefs::tests::producer_brief_requires_checkable_evidence_in_the_snapshot ... FAILED

failures:

---- briefs::tests::producer_brief_requires_checkable_evidence_in_the_snapshot stdout ----

thread 'briefs::tests::producer_brief_requires_checkable_evidence_in_the_snapshot' (31158) panicked at crates/athena-context/src/briefs.rs:2738:14:
the producer brief has a checkable evidence section
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    briefs::tests::producer_brief_requires_checkable_evidence_in_the_snapshot

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 123 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-context --lib`
error: 1 target failed:
    `-p athena-context --lib`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
