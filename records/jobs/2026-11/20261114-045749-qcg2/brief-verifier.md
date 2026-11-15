# Job 20261114-045749-qcg2: verify "Check decision links in the producer loop; key them on ids"

## Request (verbatim)

> Priority: high
> 
> Add a check, run inside the producer's loop before handover, for any job that edits docs/decisions.md. It fails if an entry in docs/decision-links.toml no longer resolves. Better: key those entries on decision ids, not line numbers. It also lists sentences in docs/ that still state the claim of an amended or superseded decision. Then re-queue the D08 amendment, which is still undone. Also record job 20261104-153450-y72x under failure class 20261104-153440-xf17 with 20261010-225741-yvk0, so the repeat is counted.
> 
> Introspections that record the repeated cause: 20261104-153423-esay, 20261106-012825-ay9k, 20261106-042756-h268, 20261106-191104-tvy1, 20261108-010633-1v8s
> 
> Acceptance:
> - The change is made as described: Amending a decision in docs/decisions.md leaves contradicting docs or breaks the line references in decision-links.toml; only the final verdict catches it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: docs/decisions.md., docs/decision-links.toml, docs/

## Reading (accepted by the user)

Add a pre-handover check, run for any job that edits docs/decisions.md, that fails when a docs/decision-links.toml [[link]] no longer resolves, re-keys links on decision ids instead of line numbers, and lists docs/ lines that still cite an amended or superseded decision. It follows up jobs 20261104-153450-y72x and 20261010-225741-yvk0, whose line-shift and stale-doc failures it prevents; the D08 re-queue and failure-class recording are queue and records actions outside the diff.

Assumptions:
- Goal and audience: producers and verifiers of any job that amends or supersedes a decision should be stopped by a mechanism before handover, not by the final verdict; the user judges the result.
- Code survey: read crates/athena-data/src/decision_links.rs (the one parser of [[governs]] and [[link]], pure rule `unreconciled`), crates/athena-core/src/run/checks.rs (DECISION_LINKS const, the pre-verifier governs check, line 620-689), and grep hits in crates/athena-cli/src/backtest.rs and crates/athena-cli/src/decisions.rs, crates/athena-core/src/run/tests.rs. The producer will extend the checks.rs gate and keep decision_links.rs as the shared parser. docs/decision-links.toml is data, not a record file under crates/, so no AnyRecord or check_record_file change is expected.
- Existing code: crates/athena-data/src/decision_links.rs, crates/athena-core/src/run/checks.rs, crates/athena-cli/src/backtest.rs, docs/decision-links.toml and docs/design/decision-sets.md are extended, not created.
- Environment: none; everything runs locally over the working tree with no CI, push or history reads.
- Field meaning: [[link]] `line` is documented in docs/decision-links.toml lines 3-4 as the line in docs/decisions.md of the phrase. The change makes the pair (older, newer) the key; `line` becomes optional or is dropped. Config: athena.toml unchanged.
- The sentence listing is advisory output (does not fail): lines in docs/ outside decisions.md that cite the id of a decision the change marks amended or superseded. A heuristic on ids; it cannot read meaning.
- D08 re-queue and recording job 20261104-153450-y72x under failure class 20261104-153440-xf17 with 20261010-225741-yvk0 are Athena queue and records actions; records/ never changes in this diff.
- Job 20261104-153450-y72x is addressed this way: its backtest findings (every link off by +3) cannot recur because links no longer depend on line numbers; its docs/design/athena-v0.md finding belongs to the dropped candidate. Its changed code and docs (snapshot.rs, lib.rs, producer.rs, verification.rs, run/mod.rs, run_pin_tests.rs, briefs.rs, enforcement.md) are dropped, not kept. Job 20261010-225741-yvk0's missing D08 amendment is handled by the re-queue.
- Existing decisions.md entries are not edited; the change is recorded in a new appended decision with a Provenance bullet, and a new [[link]] only if it amends an earlier decision (it may amend D156).

## Acceptance criteria

1. [behavior] A unit test in crates/athena-data/src/decision_links.rs (or athena-core checks tests) shows the new check fails when a [[link]] names an older/newer pair not found in docs/decisions.md and passes when decisions.md shifts by added lines; run `cargo test --locked` and it passes.
2. [behavior] The check runs in the producer loop only for jobs whose changed paths include docs/decisions.md, and a test in crates/athena-core/src/run/tests.rs shows a job not touching it is not checked and one touching it with a broken link is held before the verifier; run `cargo test --locked -p athena-core`.
3. [content] docs/decision-links.toml [[link]] entries are keyed by older/newer decision ids; no entry needs a line number to resolve, and the header comment no longer tells editors to update line numbers.
4. [behavior] `cargo test --locked -p athena-cli --test backtest` passes, including the confirmed-list tests, and a test shows inserting lines into a copy of decisions.md before a phrase does not produce a condition-a finding.
5. [behavior] A test shows the sentence listing returns the docs/ lines (outside decisions.md) that cite an amended or superseded decision id and does not fail the job; run `cargo test --locked`.
6. [content] docs/decisions.md gains a new appended decision (next free id, Provenance bullet) recording the check; no existing entry is edited; docs/design/decision-sets.md and CHANGELOG.md describe the check and the id keying.
7. [behavior] The check refuses nothing in the current repository: the real docs/decision-links.toml and docs/decisions.md pass it, shown by `cargo test --locked` (existing backtest tests on the real log).
8. [behavior] No unrelated behavior changes and fmt, clippy and test pass: Athena's recorded `cargo fmt --check`, `cargo clippy --locked --all-targets` and `cargo test --locked` on the final tree pass.

## Expected deliverables

- crates/athena-data/src/decision_links.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/tests/backtest.rs (modify)
- docs/decision-links.toml (modify)
- docs/decisions.md (modify)
- docs/design/decision-sets.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Rebuilding the ATHENA-COMMITS.txt history feature or any change to crates/athena-git/src/snapshot.rs, crates/athena-git/src/lib.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-context/src/briefs.rs (dropped from the earlier candidates)
- docs/enforcement.md and docs/design/athena-v0.md edits belonging to the dropped history feature
- Rewriting existing docs sentences automatically; the listing is advisory
- Edits or annotation of existing docs/decisions.md entries, and anything under records/
- Edits to athena.toml, defaults.toml or workflows
- Re-queuing job D08 and recording failure class 20261104-153440-xf17 (queue and records actions by Athena)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the check, id keying and listing; cargo test, clippy and fmt required.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a check in the producer loop, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Links keyed on ids removes the line-number cause rather than updating lines.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name tests that run the check on a broken and a shifted log.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in crates/athena-data/src/decision_links.rs call `link_problems(base_links, cand_links, base_log, cand_log)` on in-test string constants. (a) A candidate list has a new link D02 to D99 and the log has no phrase naming it: one NotNamed. (b) The base log and list are the same, and the candidate log is the base log with 10 lines inserted before the first phrase, the list unchanged: no problem, so the log shifted by added lines passes. (c) A pair broken before the base (listed, never named at the base): not blamed. (d) A base pair named at the base whose phrase the candidate removes: NotNamed. (e) A candidate entry that says `Amends D02` with no link: NoLink. (f) A candidate entry with `Superseded by` and no id on a new line: NoTarget; the same line already present in the base log: not reported. (g) An unparsable candidate list: one finding. Run: `cargo test --locked -p athena-data decision_links`. Observes: Each broken case returns the named finding with the pair or line text; the shifted-log, base-broken and clean cases return an empty list.
  Fails without the behavior: Without the check, (a), (e) and (f) return no finding and their assertions fail. Without id keying, (b) is not even expressible (a line-keyed pair would break on the shifted log). A delta that blamed the base would fail (c).
  Test seam: `link_problems`, a pure function added in step 2. The fixtures are in-test string constants, so nothing is injected and no double stands in for the rule. The production caller is `decision_gate` (checks.rs:21). It also uses the moved `parse_supersessions` and the existing `link_pairs` (decision_links.rs:94).
- Criterion 2: New tests in crates/athena-core/src/run/tests.rs use the existing `governed_repo`, `governed_job`, `editing`, `Counts` and `Lines` helpers (tests.rs:3168-3246), which run `Athena::go` with the fake agent `Script`. The base list is `GOVERNING_SRC` (governs only `src/`), so the producer's README edit is not governed. Case A: the producer edits the list to add a link D02 to D99 that no phrase names, and does not touch decisions.md: the job lands and the verifier ran once. Case B: the producer also appends a D13 entry with no phrase (`Checked D02; stands`) and a link D02 to D13: the job fails with a reason that starts with GATE_REASON_PREFIX and names D02 and D13, the producer ran twice (one send-back) and the verifier ran 0 times. Case C: the producer appends a D13 entry that says `Amends D02` and adds no link: the same hold, naming the missing [[link]]. Case D is the existing `governed_edit_with_an_amending_entry_and_link_reaches_the_verifier` (tests.rs:3790) with the fixed `reconciling_edits`: it lands. Run: `cargo test --locked -p athena-core governed`. Observes: The job that does not touch decisions.md is not checked; jobs that touch it with a broken link, or a phrase with no link, close before the verifier with the prefix and (producer, verifier) = (2, 0); the clean one lands.
  Fails without the behavior: Without the check, case B and case C land with the verifier count 1 and the closing-reason assertion fails. Without the `changed` guard, case A closes with the prefix and fails.
  Test seam: Production entry `Athena::go`, through the producer loop's `unreconciled_in_snapshot` (producer.rs:499) and the candidate gate (mod.rs:1944). The fake agent `Script` is built by the existing `editing` (tests.rs:3196), which adds `extra` edits to every producer pass; `governed_repo` seeds the log and list. All exist today. The only new pieces are the constant `GOVERNING_SRC` and the case texts, added in step 6.
- Criterion 3: A new test in crates/athena-cli/src/decisions.rs reads the real docs/decision-links.toml with the existing `repo_file(LINKS_PATH)` (decisions.rs:478) as a toml table and asserts: every [[link]] table has exactly the keys older, newer and status (none has `line`); every (older, newer) pair appears once; and no comment line of the file contains any of the forms `` `line` ``, `line number`, `the lines here` or `(line `. A second test shows an old-format list with `line` keys still parses (`ConfirmedLinks::parse`). Run: `cargo test --locked -p athena-cli --lib decisions`. Observes: The real list is keyed on ids only, has no duplicate pair, and its header gives no line-number advice (lines 3, 10, 12 and 22-23 of the current file hold such advice and are rewritten); an old list still loads.
  Fails without the behavior: The unedited toml has `line` keys, the duplicated D46/D79 and D87/D88 pairs, and header lines naming `line`, so every assertion of the test fails.
  Test seam: `repo_file` and `ConfirmedLinks::parse`, both existing (decisions.rs:478, 75). The toml is read as `toml::Table`; the `toml` crate is already a dependency of athena-cli (decisions.rs:77).
- Criterion 4: crates/athena-cli/tests/backtest.rs: drop `line` from the LINKS fixture (merging its two duplicate pairs). The known-hits test compares (older, newer, status) and no longer a line. The real-log test compares the set of hit pairs with the set of link pairs instead of `hits.len() == links.len()`. A new test reads the real docs/decisions.md, inserts 25 neutral lines after its first line (before the first `## D` heading, so the header the parse ignores holds them) and runs `backtest::run_on` with `ConfirmedLinks::load` and `Records::default()`, once on the real text and once on the shifted text. Run: `cargo test --locked -p athena-cli --test backtest`. Observes: No finding of condition `a` on the shifted copy, and its `decision_view` equals the one of the unshifted text, though every hit line moved by 25. The existing finding test (tests/backtest.rs:292, `hit not in the confirmed list: D80 superseded by D89`) keeps its text because only the not-found message loses its line.
  Fails without the behavior: With line-keyed matching (`Link::matches` today, decisions.rs:55) every hit after the insertion is unlisted and not found, so the shifted-copy test fails.
  Test seam: `backtest::run_on` and `ConfirmedLinks::load`, both existing (backtest.rs:447, decisions.rs:97). The shifted text is built in the test from the real file.
- Criterion 5: athena-data tests call `citing_lines(&["D08"], docs)` and `citation_notes(hits, docs)` on in-test strings. Lines with `D08`, `D08's`, `(D08)`, `D08.` and `D08-` are listed; `D080`, `D8`, `AD08` and `D08x` are not; a hit in docs/decisions.md is skipped; a non-.md path is skipped. An Amends hit with 8 citing lines lists 5 plus a count line; a Supersedes hit with 8 lists all 8. Two core tests in tests.rs commit docs/guide.md (`D02 says the README is the entry point.`) into the governed repo. (1) `governed_edit_lists_docs_lines_citing_an_amended_decision_and_still_lands`: the producer reconciles with the fixed `reconciling_edits`; the job is Landed, the verifier ran once, and the UI has a line that starts `decisions  note: docs/guide.md:1`. (2) `governed_send_back_carries_the_docs_lines`: the producer appends an `Amends D02` entry with no link on every pass, so the gate fails; the second producer prompt (the send-back) contains `docs/guide.md:1` and says the lines are advisory; the closing reason does not contain `docs/guide.md`. Run: `cargo test --locked -p athena-data citing && cargo test --locked -p athena-core governed_edit_lists && cargo test --locked -p athena-core governed_send_back_carries`. Observes: The listing returns the docs lines that cite an amended or superseded decision id, outside decisions.md; it reaches a producer that is sent back and the job log of a passing job; it never changes a job's state.
  Fails without the behavior: Without the listing the UI has no note line and the prompt has no `docs/guide.md:1`, so both core assertions fail. A listing that failed the job would turn Landed into Failed and the first test's state assertion fails. A substring match would list D080 and fail the athena-data token test.
  Test seam: `citing_lines`, `citation_notes` and `new_hits`, pure functions added in steps 2-3. The core tests use the existing `governed_job_with`, `editing`, `reconciling_on_send_back` and `Lines` UI capture (tests.rs:3225, 3196, 3250, 171). The production callers are `unreconciled_decisions` (checks.rs:626, UI lines) and `unreconciled_in_snapshot` (checks.rs:654, message). Docs are read through `git.ls_tree_under` (athena-git lib.rs:950) and `git.cat_blob` (lib.rs:552), both existing; in the loop the work tree's changed docs overlay the base through `snapshot.blob`, as `candidate_text` already does (checks.rs:661).
- Criterion 6: The producer appends D187 after D186 (decisions.md:5434) with Case, Decision (root cause: links keyed on log line numbers, and nothing resolved them before the final verdict), Alternatives, Tradeoff, Reversal cost and a Provenance bullet. It holds the phrase `Amends D156` once and none of the words `supersedes`, `superseded by` or `amends` elsewhere. It adds the link D156 to D187 (amended) to the toml, rewrites decision-sets.md §8 step 2 (id keying, the in-loop check, the advisory rule) and adds a CHANGELOG [Unreleased] entry that names the root cause. A new test in decisions.rs asserts D187 exists, has a Provenance bullet and exactly one supersession phrase, and that CHANGELOG's Unreleased section names D187. The existing `the_decision_log_is_well_formed_and_only_appended_to` (decisions.rs:247) hashes the first PINNED_LEN = 359622 bytes and applies the mirror rule `decision_log_problems` (mirrors.rs:354). Run: `cargo test --locked -p athena-cli --lib decisions && cargo test --locked -p athena-core mirrors && git diff docs/decisions.md`. Observes: The pin test and the mirror rule pass with the first 359622 bytes untouched; D187 is the next id after D186 and has a Provenance bullet; `git diff docs/decisions.md` shows only added lines at the end of the file.
  Fails without the behavior: If an existing entry is edited the pin hash differs and the guard test fails. If D187 lacks Provenance the mirror rule fails. If D187 carries a phrase with no link, or a link with no phrase, the real-log backtest test fails.
  Test seam: The existing guard `check_log` (decisions.rs:218) and `decision_log_problems` run on the real file; nothing is injected.
- Criterion 7: Two new tests in decisions.rs call `link_problems` on the real files. (1) With an empty base list and log (`link_problems("", real_list, "", real_log)`) the delta is the whole condition (a): it is empty. (2) A realistic change fixture: the base log is the real log cut before `## D187`, the base list is the real list without the D156 to D187 link, the candidate is the real files: empty. Then the candidate list with that link removed returns one NoLink for D156 and D187. The existing real-log backtest test (tests/backtest.rs:115) runs the same list through condition (a). Run: `cargo test --locked -p athena-cli --lib the_real_list && cargo test --locked -p athena-cli --test backtest the_real_log`. Observes: The real docs/decision-links.toml and docs/decisions.md give no problem, so the check refuses nothing in the current repository; the fixture shows D187's own change passes and that dropping its link would have been held.
  Fails without the behavior: A link that no phrase of the final log names, or a phrase of D187 with no link, fails the test.
  Test seam: `repo_file` and `link_problems`; the real repository files are the inputs, not a double. The fixture is built in the test from the real files.
- Criterion 8: Run the three recorded checks on the final tree. Also run the refusal corpus as a regression check. The plan changes no reading-gate rule: the corpus runs `refusing_check` (refusal_corpus.rs:1-2), and none of the planned rules (pair resolution, whole-token id match, phrase parse) is a reading, plan or request check, so no corpus case maps to them. I read only the corpus header (lines 1-60) and cite no case id. Their input forms, `D08`, `D08's`, `(D08)`, `D08.`, `D080`, `D8`, `AD08`, `D08x`, a path in docs/decisions.md, a non-.md path and the amended or superseded phrase kinds, each have a case in the new athena-data unit tests (criteria 1 and 5). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test --locked -p athena-core --test refusal_corpus`. Observes: All pass. The moved parse tests keep passing in decision_links.rs; the governed gate tests (tests.rs:3264-4440) pass with the fixed fixture and the new exact text; the retry-findings test (reading.rs:3664) passes unchanged.
  Fails without the behavior: A moved test left behind that calls a now-private helper, a missed `link.line` use (backtest.rs:329, tests/backtest.rs:144), a fixture that still carries the old wording (tests.rs:3371), or an unformatted file fails the suite.
  Test seam: None: the whole workspace test suite.

Planned files:

- crates/athena-data/src/decision_links.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/tests/backtest.rs (modify)
- docs/decision-links.toml (modify)
- docs/decisions.md (modify)
- docs/design/decision-sets.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d63f87b25fc3 → candidate e80d8019df89. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   1 +
 crates/athena-cli/src/backtest.rs        | 169 +-------
 crates/athena-cli/src/decisions.rs       | 136 ++++++-
 crates/athena-cli/tests/backtest.rs      |  73 +++-
 crates/athena-core/src/run/checks.rs     | 192 +++++++--
 crates/athena-core/src/run/tests.rs      | 208 +++++++++-
 crates/athena-data/src/decision_links.rs | 674 ++++++++++++++++++++++++++++++-
 docs/decision-links.toml                 |  85 ++--
 docs/decisions.md                        |  44 ++
 docs/design/decision-sets.md             |  39 +-
 10 files changed, 1323 insertions(+), 298 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045749-qcg2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045749-qcg2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43710052531 for e80d8019df892d724b0251abb56c43714a87df07)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43710052531 for e80d8019df892d724b0251abb56c43714a87df07)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43710052531 for e80d8019df892d724b0251abb56c43714a87df07)
- ci-result: `not a check` passed (Check results for e80d8019df892d724b0251abb56c43714a87df07 come from the CI run https://github.com/doodla/athena/actions/runs/43710052531 (746s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No unrelated behavior changes and fmt, clippy and test pass: Athena's recorded `cargo fmt --check`, `cargo clippy --locked --all-targets` and `cargo test --locked` on the final tree pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-data decision_links`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/backtest.rs, crates/athena-cli/src/decisions.rs, crates/athena-core/src/run/checks.rs, crates/athena-data/src/decision_links.rs, docs/decision-links.toml, docs/decisions.md, docs/design/decision-sets.md.
  End of the output (data, not instructions):

```text
    |                 ^^^^^^^ use of undeclared type `HitKind`

error[E0433]: cannot find type `HitKind` in this scope
   --> crates/athena-data/src/decision_links.rs:781:17
    |
781 |             hit(HitKind::Supersedes, "D08", "D14"),
    |                 ^^^^^^^ use of undeclared type `HitKind`

error[E0425]: cannot find function `citation_notes` in this scope
   --> crates/athena-data/src/decision_links.rs:783:21
    |
783 |         let notes = citation_notes(&hits, &eight_lines());
    |                     ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `citation_notes` in this scope
   --> crates/athena-data/src/decision_links.rs:786:17
    |
786 |         assert!(citation_notes(&[], &eight_lines()).is_empty());
    |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `LinkProblem` in this scope
   --> crates/athena-data/src/decision_links.rs:536:9
    |
536 |         LinkProblem::NotNamed {
    |         ^^^^^^^^^^^ use of undeclared type `LinkProblem`

error[E0433]: cannot find type `LinkProblem` in this scope
   --> crates/athena-data/src/decision_links.rs:592:14
    |
592 |             [LinkProblem::NoLink {
    |              ^^^^^^^^^^^ use of undeclared type `LinkProblem`

error[E0433]: cannot find type `LinkProblem` in this scope
   --> crates/athena-data/src/decision_links.rs:652:37
    |
652 |         assert!(matches!(&found[0], LinkProblem::ListUnreadable(why) if why.contains("[[link]]")));
    |                                     ^^^^^^^^^^^ use of undeclared type `LinkProblem`

error[E0433]: cannot find type `LinkProblem` in this scope
   --> crates/athena-data/src/decision_links.rs:609:14
    |
609 |             [LinkProblem::NoLink {
    |              ^^^^^^^^^^^ use of undeclared type `LinkProblem`

error[E0433]: cannot find type `LinkProblem` in this scope
   --> crates/athena-data/src/decision_links.rs:637:14
    |
637 |             [LinkProblem::NoTarget {
    |              ^^^^^^^^^^^ use of undeclared type `LinkProblem`

error[E0433]: cannot find type `LinkProblem` in this scope
   --> crates/athena-data/src/decision_links.rs:622:14
    |
622 |             [LinkProblem::NoLink {
    |              ^^^^^^^^^^^ use of undeclared type `LinkProblem`

Some errors have detailed explanations: E0422, E0425, E0433.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-data` (lib test) due to 57 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-core governed`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/backtest.rs, crates/athena-cli/src/decisions.rs, crates/athena-core/src/run/checks.rs, crates/athena-data/src/decision_links.rs, docs/decision-links.toml, docs/decisions.md, docs/design/decision-sets.md.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

     Running tests/context_compression.rs (<job>/target/verify/debug/deps/context_compression-6112e9d39a3b55ee)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/divergence.rs (<job>/target/verify/debug/deps/divergence-1ae4db8e62dcf162)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/lab_runs.rs (<job>/target/verify/debug/deps/lab_runs-6a6e3469b51eff19)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

     Running tests/mechanism_explanation.rs (<job>/target/verify/debug/deps/mechanism_explanation-4037148411066e09)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --lib decisions`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/backtest.rs, crates/athena-cli/src/decisions.rs, crates/athena-core/src/run/checks.rs, crates/athena-data/src/decision_links.rs, docs/decision-links.toml, docs/decisions.md, docs/design/decision-sets.md.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling utf8parse v0.2.2
   Compiling colorchoice v1.0.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-parse v1.0.0
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.14
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling anstream v1.0.0
   Compiling clap_lex v1.1.1
   Compiling clap_builder v4.6.7
   Compiling serde_derive_internals v0.30.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling clap_derive v4.6.7
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling clap v4.6.7
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0432]: unresolved import `athena_data::decision_links::link_problems`
   --> crates/athena-cli/src/decisions.rs:588:13
    |
588 |         use athena_data::decision_links::link_problems;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `link_problems` in `decision_links`

error[E0433]: cannot find `LinkProblem` in `decision_links`
   --> crates/athena-cli/src/decisions.rs:603:43
    |
603 |             [athena_data::decision_links::LinkProblem::NoLink {
    |                                           ^^^^^^^^^^^ could not find `LinkProblem` in `decision_links`

Some errors have detailed explanations: E0432, E0433.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `athena-cli` (lib test) due to 2 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked -p athena-cli --test backtest`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test --locked -p athena-data citing && cargo test --locked -p athena-core governed_edit_lists && cargo test --locked -p athena-core governed_send_back_carries`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test --locked -p athena-cli --lib decisions && cargo test --locked -p athena-core mirrors && git diff docs/decisions.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test --locked -p athena-cli --lib the_real_list && cargo test --locked -p athena-cli --test backtest the_real_log`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test --locked -p athena-core --test refusal_corpus`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
