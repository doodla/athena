# Job 20261114-045744-tj6b: verify "Tell the verdict which check failures the job's paths could not cause"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Nothing separates a failure the job caused from one already on the base, or from a clash with a conc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: failing-check-blocks-all-criteria-unjudged-no-baseline (20261017-075355-rwrh)
> Cause: Nothing separates a failure the job caused from one already on the base, or from a clash with a concurrent config and test change. The check stage has no baseline run and no mapping from tests to the change. A failing check also ends the jo...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261017-075336-m1y1 (job 20261017-044408-wcwy); prevention it proposed: Add a code check at admission and before handover. Admission would run the full test suite on the base commit and record which tests fail. Before handover the producer must run the same suite, and the system would compare the two. A test th...
> Introspection 20261017-184719-0s4h (job 20261017-182453-a5sf); prevention it proposed: Before the producer starts, run the repo's full check suite on the base commit and record which checks already fail. At verification, compare the failures after the job to that baseline. Failures that already existed are reported separately...
> Introspection 20261029-084637-aa8f (job 20261026-232434-6wfq); prevention it proposed: Run the same test command on the base commit when a job is admitted and record which tests fail. When a check fails after the job, compare the two runs and say whether each failing test is new or already failing. Store the names and output ...
> Introspection 20261108-031125-np1m (job 20261108-023653-70wy); prevention it proposed: Before dispatch, run the test suite on the unmodified base and record the failing tests as a baseline. At verification, judge only tests that fail now and passed in the baseline. Report a baseline failure as an environment or main-health pr...
> Introspection 20261108-034239-naz9 (job 20261108-023645-456v); prevention it proposed: Before dispatching a job, run the workspace checks on the base commit and record which tests already fail. Judge the "tests pass" criterion against that baseline: a failure present on the base counts as pre-existing and does not make the cr...
> Introspection 20261113-194925-8bkq (job 20261113-190327-f794); prevention it proposed: When a check fails on a job whose diff touches only docs, Athena should automatically run the same failing test on the base tree. It should record the result: failing on base means pre-existing, and the job is judged on its criteria with th...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the baseline only separates failures the base also had; a failure from a concurrent merge or an unrelated config/doc change reaches the verdict with no link between the failing test's files and the job's changed paths (jobs 20261017-044408-wcwy, 20261017-182453-a5sf, 20261026-232434-6wfq and the other three). The change gives the verdict inputs and the verifier's brief that link, as a finding, and adds no refusal rule.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (is_pre_existing, failure_split, mark_pre_existing, blocking_checks at ~447) and crates/athena-core/src/run/checks.rs (take_baseline, baseline_for). A per-test baseline already exists; the gap is concurrent-change failures not in the baseline. No record or data file under crates/ is added, so check_record_file/AnyRecord need no change.
- Goal and audience: the verifier and the owner reading a rejected job's findings; they should see at once whether a red check could come from the job.
- Previous jobs (all rejected on a red check, none to be re-run): 20261017-044408-wcwy (auditor role; its failing drain test was caused by a concurrent athena.toml change; its work is dropped here), 20261017-182453-a5sf (ontology; failure from concurrent docs/design/job-planning.md; dropped), 20261026-232434-6wfq (R5 producer split; the failing test output was cut off; dropped). Their findings are addressed by the new finding, not by redoing their work.
- Environment: none; no CI, push or PR is used.
- The 'tests that fail now' output carries file:line paths in panic messages; paths are parsed from it. Failures with no paths yield no finding.

## Acceptance criteria

1. [content] The change description names the root cause: the check stage has a baseline of base-commit failures but no mapping from a failing test's reported file paths to the paths the job changed, so a failure from a concurrent change is indistinguishable from the job's own.
2. [content] crates/athena-core/src/verify.rs gains a function that extracts repository paths from a failing check's output and reports which of them the job changed and which it did not; the check's verdict reasons are unchanged (no new refusal rule; expected 0% of newly refused readings, because the finding is Minor and informational).
3. [behavior] A new unit test in crates/athena-core/src/verify.rs builds a failing check whose output cites a path the job did not change (e.g. docs/design/job-planning.md:6) and a baseline without it; the judgment still fails, and carries a finding naming the path as not touched by the job. It fails on the base tree and passes with the change. Run `cargo test --locked -p athena-core verify`.
4. [behavior] A second unit test shows that when the cited path is in the job's changed paths, the finding says the job touched it. Run `cargo test --locked -p athena-core verify`.
5. [content] The verifier brief (built in crates/athena-core/src/run/verification.rs) includes the failing check's untouched-path finding so criteria are still judged; a test in the same crate asserts the brief text contains it.
6. [content] CHANGELOG.md has an entry for the change; docs/decisions.md is untouched except possibly a new appended decision with a Provenance bullet.
7. [behavior] cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass (recorded Athena checks on the final tree).

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- The auditor role and every file job 20261017-044408-wcwy changed (roles.rs, records.rs, auditor.rs, fake.rs, display.rs, main.rs, tests/auditor.rs, docs/records.md, roles-and-ladder.md and the rest): dropped
- The ontology work of job 20261017-182453-a5sf (docs/ontology.toml, tests/ontology.rs, architecture.md): dropped
- The R5 producer split of job 20261026-232434-6wfq (run/producer.rs, run/mod.rs, structure-pass.md): dropped
- Any new rule that refuses or holds work; the verdict on a failing check is unchanged
- Editing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: unit tests added and fmt, clippy, test criterion
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: root cause named in the change description
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: tests check the finding appears, not that wiring exists
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the new CHANGELOG entry and the appended D186 in the candidate's diff (the verifier's snapshot holds the base then the candidate as two commits, verification.rs:398). Run: `git diff HEAD~1 HEAD -- CHANGELOG.md docs/decisions.md`. Observes: CHANGELOG.md gains one entry under [Unreleased] / ### Fixed. D186 is appended after D185. Both state the root cause in the criterion's words: the check stage has a baseline of base-commit failures but no mapping from a failing test's reported file paths to the paths the job changed, so a failure from a concurrent change is indistinguishable from the job's own. Both name `failure_paths`, `failure_path_finding` and `judge_with_tree` in verify.rs as the fix and `verifier_check_views` in run/verification.rs. Both say no refusal rule is added (0% of past accepted readings newly refused). Both then correct the earlier note at CHANGELOG.md:1652-1653: a concurrent change can reach the tested tree only by being in the base, and then a usable baseline already marks it pre-existing; without one the finding says 'no usable baseline'.
- Criterion 2: The unit test `failure_paths_resolves_cited_paths_against_the_tracked_files` calls `verify::failure_paths` on failed CheckRuns whose `output_tail` holds realistic output. Cited forms: a `---- tests::a stdout ----` header; `thread 'tests::a' panicked at xtask/tests/policy.rs:245:9:`; a message line `docs/design/job-planning.md:6: <term>`; a rustc arrow `--> src/verify.rs:3:1`; a backticked `./README.md`; a bare `Cargo.toml`; a repeated path. Excluded forms: `took 1.5s`, `tests::a`, `https://example.org/docs/a.md`, `/rustc/abc/library/core/src/fmt.rs:1:1`, `target/debug/x.rs:1`, `../x.rs:1` and an untracked `crates/ghost/src/none.rs:9`. `tree_paths` lists the tracked files, with the same `src/verify.rs` suffix in two crates and a root and a crate `Cargo.toml`. A second test, `judge_with_tree_adds_only_findings`, runs `judge` and `judge_with_tree` on the same inputs and compares reasons, criteria, agent verdict and final verdict. The extractor parses check output, not a reading, so it is not a refusal or matching rule. The corpus tags `crate-relative-path` (refusal_corpus.rs:101) and the case `crate-relative-deliverable-path` (refusal_corpus.rs:2242-2244) concern deliverable paths in a reading, so no corpus case maps to it. `cargo test --locked -p athena-core --test refusal_corpus` is run as a regression guard only; I read no result of it. Run: `cargo test --locked -p athena-core verify`. Observes: Paths are listed in order of first appearance, without line numbers or duplicates. Touched: README.md. Untouched: xtask/tests/policy.rs, docs/design/job-planning.md and the root Cargo.toml (an exact match wins over the crate one). Ambiguous: `src/verify.rs` (two matches), never called touched or untouched. Every excluded form is absent. A passed check, an empty changed list and an empty tree list each give an empty result; more than 10 distinct paths are cut at 10. `judge_with_tree` leaves reasons, criteria, agent verdict and final verdict equal to `judge`'s, so no refusal rule was added.
  Test seam: Production entry points `verify::failure_paths` and `verify::judge_with_tree` (both added in step 2). Fixtures: the existing `check`, `failing`, `baseline_with` helpers (verify.rs:1806, 1835, 1973) plus the new helpers `cited` and `judge_both`. Nothing is faked.
- Criterion 3: The test `a_failure_citing_a_path_the_job_did_not_change_carries_that_finding` calls `judge_both` (step 3), which builds one `VerdictInputs` (the same shape as `judge_against`, verify.rs:1984) and returns `(judge(..), judge_with_tree(.., tree_paths))`. The check is a failing `test` with `failed_tests` empty and output of the realistic shape (a panic site in xtask/tests/policy.rs and a message naming docs/design/job-planning.md:6). `baseline_with(vec![check(true)], None)` gives the same check passing on the base, so it is not pre-existing. `changed_paths` is `["crates/athena-core/src/verify.rs"]`. Cases: (A) the baseline above; (B) baseline `None`; (C) a baseline with a note; (D) a baseline without the check; (E) a reused check (`reused_from_ci`-based, verify.rs:1824); (F) the same output on a pre-existing failure (`baseline_with(vec![check(false)], None)`); (G) a failing check whose output cites no tracked path (output cut before the panic line, or only an untracked `crates/ghost/src/none.rs:9`). Run: `cargo test --locked -p athena-core verify`. Observes: Case A: final verdict Fail; reasons exactly `["check test failed"]`, equal to the plain judgment's. `with_tree.findings` is the plain findings plus exactly one at the end, Minor and Athena-sourced, whose summary starts `Failing check test cites paths` and contains `the job did not change: xtask/tests/policy.rs, docs/design/job-planning.md`. Its evidence contains `The same check passed on the base commit`, `the base commit plus the job's change` and `does not show what caused the failure`, and no text naming a later merge. B: `No usable baseline exists`. C: names the note. D: `The baseline holds no result for this check`. E: `read from the CI run`. F: no path finding. G: `with_tree.findings == plain.findings` (no finding), the answer to revision finding 4.
  Fails without the behavior: On the base tree the test cannot build (`judge_with_tree` does not exist). If `judge_with_tree` just returned `judge(inputs)`, the assertion that exactly one Minor Athena finding names `docs/design/job-planning.md` under 'the job did not change' fails.
  Test seam: Production entry point `verify::judge_with_tree` (added in step 2), calling the existing `verify::judge` (verify.rs:408). Guards: the check must be failed and not `is_pre_existing`; `changed_paths` and `tree_paths` must be non-empty and the output must cite a tracked path. The baseline is injected through `VerdictInputs.baseline` (exists, verify.rs:184); `changed_paths` through `VerdictInputs.changed_paths` (exists, verify.rs:167); `tree_paths` as the second argument. Fixtures `check`, `failing`, `baseline_with`, `reused_from_ci` exist; `judge_both` is added in step 3. No double.
- Criterion 4: The test `a_failure_citing_a_path_the_job_changed_says_so` uses the same fixture. Case 1: output cites `README.md:1` and `docs/design/job-planning.md:6`, `changed_paths` is `["README.md"]`. Case 2: output cites only README.md. Case 3: output cites `src/verify.rs`, which two tracked files match. Case 4: `changed_paths` empty. A separate test, `path_findings_follow_every_other_finding_and_leave_the_verdict_alone` (revision finding 3), builds three failing checks (`test` with `failed_tests`, `clippy`, `fmt`), each citing an untouched tracked path, and a verifier output holding a Blocker and a Major finding. It then compares `judge` and `judge_with_tree`. Run: `cargo test --locked -p athena-core verify`. Observes: Case 1: the summary has `the job changed: README.md` and `the job did not change: docs/design/job-planning.md`. Case 2: only the changed clause; the summary never contains `did not change`. Case 3: `name matches several files: src/verify.rs` and no touched or untouched claim for it. Case 4: no finding. Every case keeps reasons exactly `["check test failed"]`. In the ordering test the plain judgment's findings are a prefix of `judge_with_tree`'s, every finding after that prefix is a Minor Athena path finding, and `with_tree.findings.iter().take(5)` still holds the verifier's Blocker and Major (display.rs:985). The finding is not blocking (`finding_blocks` false).
  Fails without the behavior: The test cannot build without `judge_with_tree`. If the function treated every cited path as untouched, the assertion on `the job changed: README.md` fails. If the finding were pushed inside the `blocking_checks` loop instead of appended, the prefix assertion fails and the first five lose a verifier finding.
  Test seam: Same as criterion 3: `verify::judge_with_tree` and the new `judge_both` helper. The verifier's findings are injected through the existing `output(commands, findings)` helper (verify.rs:1931) with `FindingOutput` values (as at verify.rs:3105). `tree_paths` is passed as an argument. Nothing is faked.
- Criterion 5: Two tests in crates/athena-core/src/run/tests.rs. (a) `a_failing_check_citing_an_untouched_path_reaches_the_verifier_brief`: `commit_file` (tests.rs:358) commits `docs/design/job-planning.md` and `xtask/tests/policy.rs` to the fixture repo before `ask`. The check comes from `leaked_sh_check` (tests.rs:4493) and is named `probe`. It runs `grep -q 'A demo' README.md || exit 0;` (the pattern of NEEDS_FIXED_FILE, tests.rs:4500), then echoes a panic line for xtask/tests/policy.rs:245:9, a line naming docs/design/job-planning.md:6 and `see README.md:1`, then `exit 1`. It passes on the base README (`# Demo`), so the baseline has it passing. It fails once the producer's edit (tests.rs:259) adds `A demo.`. The job runs through `run_checked_job` (tests.rs:4561) with `recording_verifier_prompts` (tests.rs:4547) over `editing(...)`, as `a_check_still_failing_after_the_send_back_is_flagged_to_the_verifier` does (tests.rs:4639-4696). (b) `a_failing_check_citing_a_changed_path_is_marked_changed_in_its_view` calls `verifier_check_views` on a failing check, a passing check and a pre-existing failing check. Run: `cargo test --locked -p athena-core a_failing_check_citing`. Observes: (a) The first verifier prompt holds `Failing check probe cites paths`, `the job changed: README.md`, `the job did not change: xtask/tests/policy.rs, docs/design/job-planning.md` and `The same check passed on the base commit`. The final state is Rejected. `done.verification.findings` ends with the same Minor Athena finding. `Records::load(..).jobs[job].briefs["brief-verifier"]` (as at tests.rs:5338-5340) holds the text too. (b) Only the failing, not pre-existing view's `detail` gains the text; the passing and pre-existing views keep the detail `check_result_view` gave them.
  Fails without the behavior: On the base tree `verification_stage` builds the check lines from `check_result_view` alone (verification.rs:74-77), so the verifier prompt and `brief-verifier` hold no path text and the assertions fail. The direct test does not compile without `verifier_check_views`.
  Test seam: Production entry points: `Athena::go` through `run_checked_job` (exists, tests.rs:4561), which reaches `verification_stage` (verification.rs:24); and `verifier_check_views` (added in step 4, `pub(super)`, so visible to the child module `run::tests`). Fixtures: `athena_with`, `repo`, `commit_file`, `leaked_sh_check`, `recording_verifier_prompts`, `editing` (all exist). The fake driver captures the verifier's `user_prompt`. Nothing under test is faked.
- Criterion 6: Read the diff of the two files. Run: `git diff HEAD~1 HEAD -- CHANGELOG.md docs/decisions.md`. Observes: CHANGELOG.md gains one entry under `## [Unreleased]` / `### Fixed` (CHANGELOG.md:18-24 holds the recent root-cause fixes; `### Added` is at line 26). The entry gives the root cause, the fix, the limits and the test commands. docs/decisions.md gains only a new `## D186` section after D185 (decisions.md:5410-5432) with Case, Decision, Alternatives, Tradeoff, Reversal cost and Provenance bullets in the form of D185. Nothing existing changes. The limits listed: a cited path is not proof of cause; only the last 200 lines of a failed run's output are kept (`FAILED_TAIL_LINES`, checks.rs:186), so a cut panic line gives no finding; no finding is made when no tracked path is cited, and D186 gives why; an unreadable snapshot gives no finding; the finding is in what a retry carries. docs/decision-links.toml governs only crates/athena-git/src/snapshot.rs and run/candidate.rs under D08 (decision-links.toml:220-224), so no link entry is needed.
- Criterion 7: Run the three recorded checks on the final tree: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. Targeted runs first: `cargo test --locked -p athena-core verify`, `cargo test --locked -p athena-core a_failing_check_citing`, `cargo test --locked -p athena-core --test refusal_corpus`, `cargo test --locked -p athena-context the_criteria_rules_quote_the_wording_the_enforcing_code_uses`. Run: `cargo fmt --all --check`. Observes: All three exit 0. Tests that read verify.rs source still hold because `judge`'s body, the header and `VERDICT_RULES` are untouched: `rule_sources_appear_in_the_header_or_judge_format_strings` (verify.rs:3384), `ci_reuse_replay_the_seen_commands_filter_turns_up_in_a_search_and_must_be_named` (admission.rs:1637) and `the_criteria_rules_quote_the_wording_the_enforcing_code_uses` (briefs.rs:4515). Existing tests that count or compare findings keep their results because they call `judge`, which is unchanged, or their failing checks cite no tracked path.
  Fails without the behavior: A formatting difference, a clippy warning (denied by -D warnings), an unused function, or any failing test (new or old) makes the matching Athena check fail.
  Test seam: The commands themselves (the configured checks fmt, clippy and test). No double.

Planned files:

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): crates/athena-core/src/verify.rs gains a function that extracts repository paths from a failing check's output and reports which of them the job changed and which it did not; the check's verdict reasons are unchanged (no new refusal rule; expected 0% of newly refused readings, because the finding is

## The change

Base f759cc97799b → candidate ade98114ff39. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   1 +
 crates/athena-core/src/run/mod.rs          |   3 +-
 crates/athena-core/src/run/tests.rs        | 117 +++++++
 crates/athena-core/src/run/verification.rs |  97 ++++--
 crates/athena-core/src/verify.rs           | 543 ++++++++++++++++++++++++++++-
 docs/decisions.md                          |  40 +++
 6 files changed, 771 insertions(+), 30 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045744-tj6b/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045744-tj6b/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43701845701 for ade98114ff3967ac6802033e1d27513305d1c50c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43701845701 for ade98114ff3967ac6802033e1d27513305d1c50c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43701845701 for ade98114ff3967ac6802033e1d27513305d1c50c)
- ci-result: `not a check` passed (Check results for ade98114ff3967ac6802033e1d27513305d1c50c come from the CI run https://github.com/doodla/athena/actions/runs/43701845701 (676s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass (recorded Athena checks on the final tree).)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-core verify`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/verify.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3286:21
     |
3286 |         let found = failure_paths(&cited(output), &paths(&["README.md"]), &tree());
     |                     ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3302:19
     |
3302 |         let one = failure_paths(
     |                   ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3317:13
     |
3317 |             failure_paths(&passed, &changed, &tree()),
     |             ^^^^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `FailurePaths` in this scope
    --> crates/athena-core/src/verify.rs:3318:13
     |
3318 |             FailurePaths::default()
     |             ^^^^^^^^^^^^ use of undeclared type `FailurePaths`

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3321:13
     |
3321 |             failure_paths(&cited(output), &[], &tree()),
     |             ^^^^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `FailurePaths` in this scope
    --> crates/athena-core/src/verify.rs:3322:13
     |
3322 |             FailurePaths::default()
     |             ^^^^^^^^^^^^ use of undeclared type `FailurePaths`

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3325:13
     |
3325 |             failure_paths(&cited(output), &changed, &[]),
     |             ^^^^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `FailurePaths` in this scope
    --> crates/athena-core/src/verify.rs:3326:13
     |
3326 |             FailurePaths::default()
     |             ^^^^^^^^^^^^ use of undeclared type `FailurePaths`

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3332:22
     |
3332 |         let capped = failure_paths(&cited(&long), &paths(&["other"]), &many);
     |                      ^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-core a_failing_check_citing`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/verify.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
3246 +         (judge(&inputs), judge_with(&inputs, tree))
     |

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3286:21
     |
3286 |         let found = failure_paths(&cited(output), &paths(&["README.md"]), &tree());
     |                     ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3302:19
     |
3302 |         let one = failure_paths(
     |                   ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3317:13
     |
3317 |             failure_paths(&passed, &changed, &tree()),
     |             ^^^^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `FailurePaths` in this scope
    --> crates/athena-core/src/verify.rs:3318:13
     |
3318 |             FailurePaths::default()
     |             ^^^^^^^^^^^^ use of undeclared type `FailurePaths`

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3321:13
     |
3321 |             failure_paths(&cited(output), &[], &tree()),
     |             ^^^^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `FailurePaths` in this scope
    --> crates/athena-core/src/verify.rs:3322:13
     |
3322 |             FailurePaths::default()
     |             ^^^^^^^^^^^^ use of undeclared type `FailurePaths`

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3325:13
     |
3325 |             failure_paths(&cited(output), &changed, &[]),
     |             ^^^^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `FailurePaths` in this scope
    --> crates/athena-core/src/verify.rs:3326:13
     |
3326 |             FailurePaths::default()
     |             ^^^^^^^^^^^^ use of undeclared type `FailurePaths`

error[E0425]: cannot find function `failure_paths` in this scope
    --> crates/athena-core/src/verify.rs:3332:22
     |
3332 |         let capped = failure_paths(&cited(&long), &paths(&["other"]), &many);
     |                      ^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
