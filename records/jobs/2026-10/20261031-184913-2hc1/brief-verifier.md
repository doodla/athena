# Job 20261031-184913-2hc1: verify "Refuse readings that rely on repo paths absent from the snapshot"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Nothing checks a reading's references to earlier jobs against the current repo. A job id named in th
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-assumes-prior-job-deliverable-exists-unverified (20261026-050336-ykx8)
> Cause: Nothing checks a reading's references to earlier jobs against the current repo. A job id named in the request or notes is taken to mean its work has landed. The earlier job may be queued, unmerged or never produced. The critic is the first ...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261026-050325-8red (job 20261026-042558-5p65); prevention it proposed: Add a check at reading acceptance. For every earlier job id, role, prompt or file the reading calls existing, it confirms the artifact is in the snapshot (landed in main, with a path found). If not, the reading must either list it as a deli...
> Introspection 20261026-065033-9gdg (job 20261026-064950-38jx); prevention it proposed: Add a check before a reading is accepted. It lists the roles, prompts and files the reading depends on and confirms each exists in the current snapshot. When one is missing, the check requires the reading either to make any request it queue...
> Introspection 20261026-092111-01tx (job 20261026-065228-02zf); prevention it proposed: Before a reading is accepted, run a check that every field, state, role or function the reading relies on is found in the current snapshot (by grep of the named symbols). It should flag any dependency that is absent, such as a "resolved" st...
> Introspection 20261026-145400-8x0c (job 20261026-145124-ssmw); prevention it proposed: Add a required step to the reading stage: for every existing path, record type, role or function the reading says a feature will use, it must cite the file and line where it exists in the current snapshot, and for each transfer between comp...
> Introspection 20261026-145920-qjpb (job 20261025-001236-brde); prevention it proposed: Add a required step to reading and planning for any request that changes "all" readers, writers or call sites. Before the plan is accepted, the stage must list every real call site found by searching the snapshot, mark each as in scope or o...
> Introspection 20261026-150536-zmp5 (job 20261026-145955-s7e2); prevention it proposed: Require any plan that changes "all readers/callers" of something to include a call-site inventory produced by actually running the scan against the current tree. Each hit must be marked move, allow-listed or out of scope, with a reason and ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: reading acceptance (check_reading_acceptance in crates/athena-core/src/reading.rs) checks only deliverable paths against the base commit, so a reading that says an earlier job's file, role or prompt "already exists" is accepted unverified and the critic finds it later (failure class reading-assumes-prior-job-deliverable-exists-unverified, seen in jobs 5p65, 38jx, 02zf and others). Add a deterministic check, for the planner/critic's reader audience, that every repo path the reading's text relies on exists in the base snapshot or is created by this change.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: check_reading_acceptance (line ~2849) runs check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_governance_coverage, then check_boundary_and_repo_files, which checks only deliverable paths (path_exists_in) and banned words over reading_text_fields; repo_files is None in a replay with no repository. No record or data file is added under crates/, so check_record_file and AnyRecord are untouched.
- Goal and audience: the reader stage and the critic. A reading that relies on a path (file, prompt, role source) that does not exist at the base commit must be repaired by the reader in its one correction, not found by the critic after planning.
- Design: a new check_reading_references in reading.rs, called last from check_reading_acceptance (or folded into check_boundary_and_repo_files). It extracts repo-path tokens (crates/, docs/, xtask/, studies/, .github/ and root files) from the summary, assumptions and acceptance criteria, and refuses any that is not a file or directory in repo_files, not a create deliverable, and not in a sentence that states it is absent or to be created. With repo_files None it checks nothing. It is deterministic and calls no model.
- Earlier jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf are the instances that surfaced the class. Their code is not redone here: each was a failed or rejected attempt whose reading assumed a design-review role or a resolved finding state that did not exist. None of their candidate files is kept or changed by this job, and none of their findings is fixed here; a later retry of them would be checked by the new rule.
- Refusing more readings is intended. expected 3% of newly refused readings.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-core: a new unit test in crates/athena-core/src/reading.rs gives a reading whose assumption says a path such as crates/athena-core/src/design_review.rs already exists, with a repo_files set lacking it, and shows check_reading_acceptance refuses it naming the path; the same test fails when the check is removed.
2. [behavior] Run cargo test -p athena-core: tests show the same reading is accepted when the path is in repo_files, when it is a create deliverable, when the text says it does not exist, and when repo_files is None; the refusal is for the class (any path), not for one job id.
3. [content] Grep of crates/athena-core/src/reading.rs shows the new check called from check_reading_acceptance, and a doc comment naming the root cause (reading acceptance never compared the reading's references with the snapshot) and jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf.
4. [content] The refusal message tells the reader how to repair it: list the missing artifact as a create deliverable, or say it does not exist; CHANGELOG.md has an entry for the new check.
5. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked all pass; the verifier runs them.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing the code-health check or the post-mortem field work of jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf
- Checking job ids against landed state or queue position
- Changes to athena.toml, defaults.toml or prompt files
- Call-site inventory checks for plans (the other proposals in the failure class)

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is placed at reading acceptance, where the unverified reference originates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests plus fmt, clippy and test criteria.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A deterministic check, not prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests that show refusal and acceptance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file of logic plus changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in reading.rs. It builds a reading with the existing `governed_reading` helper (a modify of crates/athena-core/src/reading.rs, so the deliverable rule passes) and sets the assumption to a sentence saying crates/athena-core/src/design_review.rs already exists. It calls `check_reading_acceptance(&reading, "add a greeting", None, &[], Some(&files))` where `files` is a BTreeSet holding crates/athena-core/src/reading.rs but not design_review.rs. The same sentence is also put in the summary and in an acceptance criterion. Run: `cargo test -p athena-core a_reading_that_relies_on_an_absent_path_is_refused_and_the_path_is_named`. Observes: The call returns Err; the message contains crates/athena-core/src/design_review.rs and the label `assumptions[0]` (and `summary`, `acceptance_criteria[0]` in the other cases).
  Fails without the behavior: Without check_reading_references, check_reading_acceptance returns Ok for the reading, so the unwrap_err in the test panics and the test fails.
  Test seam: Production entry point `check_reading_acceptance` (exists, line ~2849). Fixture `governed_reading` (exists, line ~6349) and a local BTreeSet of paths passed as the `repo_files` argument (the argument exists). A small `files_with_core()` helper is added to the tests module in step 5. No double replaces any logic.
- Criterion 2: Unit tests in reading.rs on the same reading as criterion 1: (a) accepted when `files` holds the path; (b) accepted when the path is also a Create deliverable; (c) accepted when the sentence says the path does not exist or is to be created, while a second still-absent path in a different sentence is refused; (d) accepted with repo_files None; (e) a table of different absent paths (docs/design/other.md, xtask/src/new.rs, .github/workflows/x.yml, studies/a/b.md) each refused; (f) glob, URL, bare `reading.rs`, trailing-dot and `path:123` forms handled as stated; (g) a reading with a question or no capability accepted. Run: `cargo test -p athena-core the_reference_rule_accepts`. Observes: Ok in cases a, b, c (first path), d, f and g; Err naming the path in case e and the second path in case c. No job id appears in any assertion.
  Fails without the behavior: An over-broad check would refuse the accepted cases. A check keyed on one job id or one path would not refuse the different paths in the table, so case e fails; with the check removed, case e and the second path of case c fail.
  Test seam: Production entry point `check_reading_acceptance` with `repo_files` as `Some(&files)` or `None` (exists). Inputs from `governed_reading` (exists) and `reading_with_deliverable` (exists, line ~6648). No fake.
- Criterion 3: Grep reading.rs for the new function name and the three job ids, then read the lines to see the call inside `check_reading_acceptance` and the doc comment. Run: `grep -n "check_reading_references\|20261026-042558-5p65\|20261026-064950-38jx\|20261026-065228-02zf" crates/athena-core/src/reading.rs`. Observes: A definition of `check_reading_references`, a call to it in the body of `check_reading_acceptance` after `check_boundary_and_repo_files`, and a doc comment naming the root cause (reading acceptance never compared the reading's references with the snapshot) and the three job ids.
- Criterion 4: Unit test asserts the refusal text contains the path, advice to list it as a create deliverable, and advice to say it does not exist. Then grep CHANGELOG.md for the new entry. Run: `cargo test -p athena-core the_reference_refusal_tells_the_reader_how_to_repair_it`. Observes: The error tells the reader to list the artifact as a create deliverable or say it does not exist. `grep -n "check_reading_references" CHANGELOG.md` finds the entry under `## [Unreleased]`.
  Fails without the behavior: If the message lacks the repair text, the assertions on `create deliverable` and `does not exist` fail.
  Test seam: Production entry point `check_reading_acceptance` (exists); same fixtures as criterion 1; the message text is produced by the production check, not the test.
- Criterion 5: Run the three gate commands, each as its own simple command, on the finished tree. The test run includes every existing athena-core test that calls `check_reading_acceptance` or `read_for` with `Some(&files)`, so a reading text that now names an absent repo path is caught, as are the new tests. Run: `cargo test --locked`. Observes: Exit 0 for `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`, with the new tests listed as passing. The function reads no model and no clock, so results are deterministic.
  Fails without the behavior: If the change were missing or wrong, the new tests from criteria 1, 2 and 4 would fail under `cargo test --locked`; if it breaks an existing test fixture (a text naming an absent path), that test fails; formatting or lint errors in the new code fail the fmt and clippy runs.
  Test seam: Production entry points: the workspace test harness running `check_reading_acceptance` through the athena-core unit tests (exists), plus `cargo fmt` and `cargo clippy` over the tree. Fixtures are the existing test helpers `governed_reading`, `files_at_head` and `reading_with_deliverable` (exist); the new tests are added by step 5. No fake.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 621ca4291859 → candidate 428853ac5482. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  11 ++
 crates/athena-core/src/reading.rs | 325 +++++++++++++++++++++++++++++++++++++-
 docs/decisions.md                 |  30 ++++
 3 files changed, 364 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1850 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked all pass; the verifier runs them.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
