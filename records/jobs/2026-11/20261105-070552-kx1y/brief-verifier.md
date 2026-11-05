# Job 20261105-070552-kx1y: verify "Design continuous landing inside the drain (review 20261105-070420-vq95)"

## Request (verbatim)

> Priority: high
> 
> Throughput design: Landing and rebuild takes a median of 440.1 of the 464.5 minutes per landed job; every other stage t
> 
> Athena's throughput review 20261105-070420-vq95 named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: Landing and rebuild takes a median of 440.1 of the 464.5 minutes per landed job; every other stage together takes about 21. The measured span runs from the verifier's end to the landing record, which is written only after the merge has been...
> Stage: landing and rebuild
> Proposed change: Make landing a continuous step inside the drain instead of a separate, later one. As soon as the verifier passes, the job goes onto the one ordered landing queue (async-drain.md sections 6 and 7). There its pull request is merged automatica...
> Predicted saving: 420.0 minutes per landed job; the median was 464.5 minutes per landed job when the review ran.
> Why: Measured (THROUGHPUT.md): landing and rebuild has a median of 440.1 min of the 464.5 min per landed job. Reading, planning, producer, checks and verifier together take about 21 min. Idle time is about 44 min per landed job (16070.4 / 366), ...
> 
> A change that implements this design must cite throughput review 20261105-070420-vq95 in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a new design note under docs/design/ that makes landing a continuous step of the drain: after the verifier passes, the job joins the one ordered landing queue, its PR auto-merges, and the rebuild leaves the critical path. It targets the maintainers who read the next throughput review, and it cites review 20261105-070420-vq95 and the predicted 420.0 min saving per landed job.

Assumptions:
- Goal: a reviewable design note, not an implementation. The audience is the Athena maintainers and the next throughput review, which scores the prediction.
- Existing code: docs/design/async-drain.md (sections 6 and 7, the ordered landing queue), docs/design/native-pr-landing.md, docs/design/squash-chain-landing.md and docs/design/warm-baseline-build.md. The note reuses or extends these and creates none of them.
- The note is a new file, docs/design/continuous-landing.md. The exact name is the producer's choice, but it must sit directly under docs/design/.
- THROUGHPUT.md is cited as the source of the measured numbers. If it is absent from the snapshot, the note cites the request text instead.
- No governance files (athena.toml, defaults.toml, workflows) change, because this is a design note only.

## Acceptance criteria

1. [content] A new markdown file directly under docs/design/ describes making landing a continuous drain step: after the verifier passes, the job joins the one ordered landing queue, and its PR merges automatically.
2. [content] The note has an Existing code section that reuses or extends docs/design/async-drain.md (sections 6 and 7) and docs/design/native-pr-landing.md. It says which parts it reuses and which it changes.
3. [content] The note explains, with the measured figures (440.1 of 464.5 min in landing and rebuild, about 21 min for all other stages), how the change removes the bottleneck. Grep finds the figures in the note.
4. [content] The note states the predicted saving of 420.0 minutes per landed job. It also names how the next throughput review measures it: the median minutes per landed job from the verifier's end to the landing record, compared with 464.5.
5. [content] The note cites throughput review 20261105-070420-vq95 by that id, and says a change implementing the design must cite it in its request text.
6. [content] The note lists alternatives considered, risks (ordering, rebuild failure, merge conflicts) and what is out of scope.
7. [behavior] Run cargo test from the repository root. It passes, because any docs-consistency tests still hold with the new file.
8. [content] Only files under docs/design/ change, and no governance file changes.

## Expected deliverables

- docs/design/continuous-landing.md (create)
- docs/design/async-drain.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/ or any code
- Changing athena.toml, defaults.toml or workflows
- Re-running or editing the throughput review itself

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note lists alternatives considered and the reasons for the choice.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The note cites measured numbers (440.1, 464.5, 21, 16070.4/366) to their source and labels the 420.0 min saving as a prediction.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires the note to name the measured minutes per landed job as the outcome the next review checks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The note is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One note and no code changes.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the working-tree file (not a git command, because the producer does not commit). `ls` shows the file sits directly under docs/design/. A case-insensitive grep finds the design sentences whatever the tense. The verifier then reads the Design section. Run: `ls docs/design/continuous-landing.md && grep -niE 'ordered landing queue|verifier pass|merg(es|ed) automatically|auto-?merge' docs/design/continuous-landing.md`. Observes: The file exists directly under docs/design/. The grep lines show the rule that after the verifier passes the job joins the one ordered landing queue and its pull request merges automatically, run by the queue head. (The producer is told to use the phrases 'ordered landing queue', 'verifier passes' and 'merges automatically' at least once each.)
  Fails without the behavior: Without the file, `ls` reports no such file and the grep prints nothing. Without the described step there is no design to judge.
  Test seam: Content check by reading a file. Production entry point: none, since no code changes. The fixture is the working-tree file docs/design/continuous-landing.md, which plan step 2 creates.
- Criterion 2: Content check of the working-tree file. The grep finds the Existing code heading and the table. The verifier reads the rows: sections 6 and 7 of async-drain.md and native-pr-landing.md each carry a 'reused' or 'changed' label, and so do LandingQueue, Drain::land, land_pr and merge_via_rest, Landing::Waiting and landing_from_report (changed), rebuild_if_due and finish (changed: when it is called, the lock invariant kept), and the landing and landing-intent records. Run: `grep -nE 'Existing code|async-drain.md|native-pr-landing.md|reused|changed|Landing::Waiting|landing_from_report|rebuild_if_due' docs/design/continuous-landing.md`. Observes: An Existing code section that names async-drain.md sections 6 and 7 and native-pr-landing.md, with reused or changed per row, including Landing::Waiting, landing_from_report and rebuild_if_due labeled changed.
  Fails without the behavior: Without the section the grep finds no Existing code heading and no labels, and the reader cannot tell what is new.
  Test seam: Content check by reading a file. No production entry point. The fixture is the new note from plan steps 2 to 5 in the working tree.
- Criterion 3: Text search of the working-tree note for the measured figures, then reading the paragraph 'How it removes the bottleneck'. Run: `grep -nF -e 440.1 -e 464.5 -e 16070.4 -e 'about 21' docs/design/continuous-landing.md`. Observes: Lines carrying 440.1 of 464.5 min in landing and rebuild, about 21 min for all other stages, and 16070.4 / 366 for idle time, each with its source (review 20261105-070420-vq95 request text; status.rs span definition). The explanation follows from them.
  Fails without the behavior: Without the figures the grep prints nothing and the claim that this is the biggest wait is unsupported.
  Test seam: Content check by grep on a file. No production entry point. The fixture is the new note from plan steps 3 to 5.
- Criterion 4: Text search of the working-tree note, then reading the 'Predicted saving and how it is measured' section. The rule must agree with `THROUGHPUT_WINDOW` and `StageMinutes::median_of` in crates/athena-core/src/status.rs, which exist today (read at lines 642-695 and 805-813). The producer cites them and runs no code. Run: `grep -nF -e 420.0 -e THROUGHPUT_WINDOW -e 'median_of' -e 'verifier' docs/design/continuous-landing.md`. Observes: 420.0 min per landed job stated as the review's prediction, not a measurement. The measurement rule is the median over the newest 20 landings from the verifier's end to the landing record, compared with 464.5 (stage 440.1), and the rebuild time reported separately because it adds to later jobs' spans through the landing lock.
  Fails without the behavior: Without it the next review has no figure to score and no rule for the window.
  Test seam: Content check by grep and reading a file. No production entry point. The cited status.rs items exist today.
- Criterion 5: Text search of the working-tree note for the review id and the citation duty. Run: `grep -nF -e 20261105-070420-vq95 -e 'request text' docs/design/continuous-landing.md`. Observes: The id 20261105-070420-vq95 appears, with the sentence that a change implementing the design must cite it in its request text.
  Fails without the behavior: Without the id the next review cannot match the change to its prediction, and the grep prints nothing.
  Test seam: Content check by grep on a file. No production entry point. The fixture is the new note from plan step 6.
- Criterion 6: Text search for the headings and named risks, then reading each section. The Risks section must hold, as separate items: ordering, head-of-line blocking (the Waiting limit H), rebuild failure and the rebuild lock invariant (landing moving the checkout mid-build), merge conflicts, saving misread, wrong phase. Run: `grep -niE '^#+ .*(Alternatives|Risks|Out of scope)|ordering|head-of-line|rebuild failure|merge conflict|checkout' docs/design/continuous-landing.md`. Observes: An Alternatives section (at least: raise concurrency only, host auto-merge, shorter CI wait, separate-worktree build, do nothing), a Risks section naming ordering, head-of-line blocking, rebuild failure and merge conflicts with a mitigation each, and an Out of scope list.
  Fails without the behavior: Without them the grep finds no such headings and the design is not reviewable under the 'designed' default.
  Test seam: Content check by grep and reading a file. No production entry point. The fixture is the new note from plan steps 4 to 7.
- Criterion 7: Run the full test suite from the repository root with the new note, D155 and the CHANGELOG entry present. Existing tests read docs: the boundary-word scan (xtask/tests/policy.rs:88), the decisions.md readers (crates/athena-cli/tests/backtest.rs lines 117, 131, 138, 780), the ontology scan (crates/athena-cli/tests/ontology.rs), the CHANGELOG readers (crates/athena-core/tests/refusal_corpus.rs:859-865; crates/athena-cli/tests/lifecycle.rs:2311-2324) and `system_facts_budgets_match_computed`. Run: `cargo test --locked`. Observes: All tests pass.
  Fails without the behavior: A banned word or email-like text fails the policy scan. 'amends', 'supersedes' or 'superseded by' with a decision id in D155 adds a hit and fails backtest.rs:131. A malformed '## D155 · ...' header or missing field fails the decisions readers. A removed CHANGELOG phrase fails the include_str! `contains` assertions.
  Test seam: Entry point: `cargo test --locked` from the repository root. No fake. The tests exist today (policy.rs:88, backtest.rs:117 and 131, ontology.rs:249, lifecycle.rs:2311, refusal_corpus.rs:859). The fixture is the working tree with plan steps 2 to 9 applied.
- Criterion 8: List modified and untracked paths together (`git diff` alone misses the new untracked note). No governance file (athena.toml, defaults.toml, workflows, tool or lint configuration) and nothing under crates/ or xtask/ changes. The deliverables list names docs/decisions.md and CHANGELOG.md, so those two are allowed beside docs/design/. Run: `git status --porcelain`. Observes: Exactly four paths: docs/design/continuous-landing.md (untracked), docs/design/async-drain.md, docs/decisions.md and CHANGELOG.md. No athena.toml, defaults.toml, .github/ workflow, crates/ or xtask/ path appears.
  Fails without the behavior: A listed athena.toml, defaults.toml, workflow or crates/ path shows the scope rule was broken.
  Test seam: Reads the producer's working tree state with `git status --porcelain`, which exists today. See the open question on docs/decisions.md and CHANGELOG.md versus 'only docs/design/'.

Planned files:

- docs/design/continuous-landing.md (create)
- docs/design/async-drain.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3398cf753ca9 → candidate 360a69e6a8ba. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   7 ++
 docs/decisions.md                 |  34 ++++++
 docs/design/async-drain.md        |   5 +
 docs/design/continuous-landing.md | 226 ++++++++++++++++++++++++++++++++++++++
 4 files changed, 272 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-070552-kx1y/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-070552-kx1y/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41991903175 for 360a69e6a8ba69b196efee8e789422e471d3e5e2)
- ci-result: `not a check` passed (Check results for 360a69e6a8ba69b196efee8e789422e471d3e5e2 come from the CI run https://github.com/doodla/athena/actions/runs/41991903175 (409s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`ls docs/design/continuous-landing.md && grep -niE 'ordered landing queue|verifier pass|merg(es|ed) automatically|auto-?merge' docs/design/continuous-landing.md`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`grep -nE 'Existing code|async-drain.md|native-pr-landing.md|reused|changed|Landing::Waiting|landing_from_report|rebuild_if_due' docs/design/continuous-landing.md`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`grep -nF -e 440.1 -e 464.5 -e 16070.4 -e 'about 21' docs/design/continuous-landing.md`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`grep -nF -e 420.0 -e THROUGHPUT_WINDOW -e 'median_of' -e 'verifier' docs/design/continuous-landing.md`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`grep -nF -e 20261105-070420-vq95 -e 'request text' docs/design/continuous-landing.md`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`grep -niE '^#+ .*(Alternatives|Risks|Out of scope)|ordering|head-of-line|rebuild failure|merge conflict|checkout' docs/design/continuous-landing.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, docs/decisions.md, docs/design/async-drain.md, docs/design/continuous-landing.md.
  End of the output (data, not instructions):

```text
   Doc-tests athena_agent

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cap_change

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cap_study

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cli

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_context

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_gate

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_git

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`git status --porcelain`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
