# Job 20261025-033731-5hac: verify "Add docs/system-facts.md to planner and critic briefs with drift test"

## Request (verbatim)

> Priority: high
> Add a system-facts file that the planner and critic always read (BRIEF.md addendum of 2026-10-25): docs/system-facts.md, short, stating the facts about how Athena runs that reading the code cannot reveal, each with its source. Start with: the host squash-merges and main requires linear history and up-to-date branches; a producer works in a one-commit snapshot with no remote and no other branches; the reader, planner and critic get a plain copy without git or records/; prompt sets and their budgets with current sizes; the words the boundary scan bans; the request format athena plan parses (acceptance as bullets, dependencies as job ids, Priority: high); land lands only a job's pull request; records live on athena/records after C0. Add it to the planner and critic briefs, and add a test that fails when a prompt budget in the file differs from the budgets test's numbers.
> Acceptance:
> - The planner and critic briefs include the file, and a brief test shows it.
> - A test fails when the file's recorded budget sizes drift from the computed ones.
> - The planner and critic budgets still pass.
> Files: docs/system-facts.md, crates/athena-context/src/briefs.rs, crates/athena-cli/tests/budgets.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Create a short sourced docs/system-facts.md of run-time facts the code can't reveal (including the user's added facts), embed it in planner_brief and critic_brief, and add a budgets test failing when recorded budgets/sizes drift from computed ones. Audience: planner and critic agents, and maintainers editing the file.

Assumptions:
- The file is embedded via include_str! in briefs.rs so both briefs carry it.
- Budget numbers are recorded in a parseable form (e.g. 'reader: 1997 of 2000') so the test can extract them.
- The facts file may need to fit within the planner/critic budgets; if it does not, budgets in budgets.rs are not raised without need and the file is kept short.
- Boundary words are not listed; the file points to boundary_words() in xtask/tests/policy.rs as the source.
- No athena.toml or workflow change is needed; the [personal_data] allow-list is only referenced.

## Acceptance criteria

1. [content] docs/system-facts.md is short and states every requested and added fact (squash-merge/linear history/up-to-date branches; producer one-commit snapshot; plain copy without git or records/ for reader, planner, critic; prompt sets with budgets and current sizes; plan request format; land lands only a job's PR; records on athena/records after C0; SSH remote refused at admission; per-PR and per-branch caches; early PR and land waiting for CI), each with a source.
2. [content] The file does not list the banned boundary words and names boundary_words() in xtask/tests/policy.rs as the source; no email-like text appears in it.
3. [behavior] A briefs.rs unit test asserts planner_brief and critic_brief output contains the facts file text; run `cargo test -p athena-context`.
4. [behavior] A budgets.rs test fails when the budgets or current sizes recorded in the file differ from the computed ones, and a negative case with a doctored value is rejected; run `cargo test -p athena-cli --test budgets`.
5. [behavior] The existing planner and critic budget tests still pass with the file included; run `cargo test -p athena-cli --test budgets`.
6. [content] CHANGELOG.md has an entry and docs/decisions.md records the call (file, brief inclusion, drift test).
7. [behavior] cargo test, cargo clippy and cargo fmt --check pass at the repository root.

## Expected deliverables

- docs/system-facts.md (create)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing prompt budgets
- Listing the banned words in the file
- Changes to athena.toml or workflows
- Editing BRIEF.md

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit test in briefs.rs and drift test in budgets.rs; cargo test and clippy clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every fact in the file cites a source.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check brief output contains the file and that a doctored budget is rejected.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Plain wording for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No emails or secrets; SSH remote example uses a placeholder.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A briefs test asserts the file has a marker phrase for each required fact and a 'Source:' on every fact bullet and is under a byte cap; the verifier also reads the file against the criterion's list. Run: `cargo test -p athena-context`. Observes: The test passes; the file is short and each listed fact has a source.
- Criterion 2: A briefs test asserts the file has no '@', contains 'boundary_words' and 'xtask/tests/policy.rs', and contains none of the words from boundary_words(). Run: `cargo test -p athena-context`. Observes: The test passes.
  Fails without the behavior: If a banned word or an email-like string were in the file, or the source pointer were missing, the assertion would fail.
- Criterion 3: Unit tests build planner and critic inputs from the existing fixtures and assert each output contains SYSTEM_FACTS. Run: `cargo test -p athena-context`. Observes: Both outputs contain the whole file text.
  Fails without the behavior: If the section were not added to planner_brief or critic_brief, the contains assertion would fail.
- Criterion 4: system_facts_budgets_match_computed runs the checker on the real file; system_facts_drift_is_rejected changes one number in a copy and asserts Err naming the set. Run: `cargo test -p athena-cli --test budgets`. Observes: The real file passes and the doctored copy is rejected.
  Fails without the behavior: If the checker did not compare numbers, the doctored copy would return Ok and the negative test would fail; stale numbers in the file would fail the real-file test.
- Criterion 5: The existing every_role_set_is_within_its_budget and an_over_budget_role_set_fails run in the same binary, now over the briefs that include the facts file, with budgets unchanged in the test. Run: `cargo test -p athena-cli --test budgets`. Observes: Both pass with the planner and critic sets within their budgets.
  Fails without the behavior: If the included file pushed the planner or critic set over its budget, or the helper refactor broke the sets, every_role_set_is_within_its_budget would fail.
- Criterion 6: Review the diff of CHANGELOG.md and docs/decisions.md. Run: `git diff`. Observes: A CHANGELOG entry and a decisions entry covering the file, the brief inclusion and the drift test.
- Criterion 7: Run cargo test, cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check at the root. Run: `cargo test`. Observes: All three succeed.
  Fails without the behavior: A failing new test, a clippy warning in the new code, or unformatted code would make the matching command exit non-zero.

Planned files:

- docs/system-facts.md (create)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base f0922d37a7b6 → candidate 586896f620fe. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   7 ++
 crates/athena-cli/tests/budgets.rs  | 136 ++++++++++++++++++++++++++++++++++++
 crates/athena-context/src/briefs.rs | 114 ++++++++++++++++++++++++++++++
 docs/decisions.md                   |  27 +++++++
 docs/system-facts.md                |  50 +++++++++++++
 5 files changed, 334 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1227 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
