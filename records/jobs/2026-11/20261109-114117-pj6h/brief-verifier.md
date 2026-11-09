# Job 20261109-114117-pj6h: verify "Make the steward decide itself; ask the owner only for env or token"

## Request (verbatim)

> try again on 20261109-080220-zkhy

## Reading (accepted by the user)

Retry of 20261109-080220-zkhy. The steward steps back across repeated failures and blind spots and queues the fix itself. The owner gets a five-field brief only for an item that needs an environment change or a token Athena lacks. Code in steward.rs and work_decisions.rs enforces that routing, not only the prompt. The audience is the owner (Sri), who judges by how few decisions reach them.

Assumptions:
- Previous result, job 20261109-080220-zkhy (rejected, 10 findings): its prompt-only approach is dropped and rebuilt. Its candidate's six changed files (CHANGELOG.md, steward.rs, records.rs, roles.rs, docs/decisions.md, docs/design/roles-and-ladder.md) are kept as deliverables, redone from base. Findings addressed: (1) env/token never reached the owner: decide() now routes by the presence of the brief. (2) over-limit proposals reached the owner without a brief: open_proposals now lists only briefed proposals. (3) 600-token budget unmeasured on the real prompt: a new test composes the real STEWARD_PROMPT. (4) missing brief tests: added in steward.rs and crates/athena-cli/tests/steward.rs. (5) records.rs edit out of scope: it is now a deliverable, a path update only. (6) misread request: the routing is fixed as above.
- Code survey: read crates/athena-core/src/steward.rs. decide() (about line 693) applies a change when role.may_change(rung) and keeps change.brief only for Decision::Proposed. blind_spot_change() (about line 739) is on Ladder::Implementation with brief None. crates/athena-core/src/work_decisions.rs open_proposals() (about line 43) lists every kind Proposal item with no decision. Plan: a change carrying a brief means the steward judged it needs an environment or token the owner must supply, so decide() records it as Decision::Proposed with the brief whatever the rung. A change without a brief is applied if within the limit and otherwise recorded without reaching the owner. open_proposals lists only items with a brief. A missing-token blind spot gets a briefed constructor. The measured check is the item's brief field and open_proposals output in tests. Records validation: crates/athena-data/src/records.rs check_record_file returns AnyRecord and a round-trip test covers the role file. The steward role bumps to version 3 (steward-3.json), so the path asserted at records.rs:2265 and roles.rs:306 (steward-2.json) is updated. check_record_file and the round-trip test accept the new role record unchanged. Prompt budget: the Steward budget is 600 tokens, checked by check_budget in crates/athena-context/src/lib.rs, which today composes a placeholder. Add a test on the real STEWARD_PROMPT.
- Term ladder: the ordered rungs of change (Ladder::Implementation lowest, up to design). A role may change itself only up to its highest_rung; above it the change is only proposed.
- The steward prompt is a model prompt, so the step-back and decide-yourself wording is prose-only. The routing of briefs and owner decisions is enforced by code and tests.
- The change adds no refusal rule. It does not touch athena.toml, defaults.toml, workflows or records/, so no refusal-rate figure applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A new steward.rs test shows a change with no env/token need and no brief, at a rung above the limit, produces a work item with brief None that open_proposals does not list.
2. [behavior] Run `cargo test -p athena-core -p athena-cli`. A test shows a change with a five-field brief (an env or token need, e.g. a missing GH_TOKEN) is recorded as Proposed with its brief at any rung, and open_proposals lists it. A crates/athena-cli/tests/steward.rs test exercises this end to end.
3. [content] Grep/Read of the diff: crates/athena-core/src/work_decisions.rs open_proposals and crates/athena-core/src/steward.rs decide() carry the routing rule above. blind_spot_change or a new constructor gives a missing-token source a brief.
4. [content] Grep/Read of crates/athena-data/src/roles.rs: STEWARD_PROMPT tells the steward to step back across repeated failures and blind spots, to decide and queue a request itself, and to write a brief only for an environment change or a token or credential. A roles.rs test asserts this wording.
5. [behavior] Run `cargo test -p athena-data -p athena-context`. A test composes the real STEWARD_PROMPT with the contract and asserts the estimated tokens are at most 600. The steward role is version 3, and the steward-3.json path is updated in roles.rs and records.rs tests.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass clean.
7. [content] CHANGELOG.md has an entry for the new steward behaviour. docs/design/roles-and-ladder.md and docs/decisions.md no longer describe the old behaviour (briefs for every over-limit proposal). The docs mark the prompt rule prose-only and the routing as code-enforced. The change does not touch athena.toml, defaults.toml, .github/workflows or records/. Check with git diff.

## Expected deliverables

- crates/athena-data/src/roles.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the steward's highest_rung or cadence
- Adding a code check that refuses briefs
- Changes to athena.toml, defaults.toml, .github/workflows or records/
- Changing the drain, audience or auditor roles

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the owner-routing behavior (briefs and open_proposals), not just that the wording exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover each routing case; cargo test, clippy and fmt must be clean.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Routing is enforced in code; the prompt wording is marked prose-only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in steward.rs mod tests and work_decisions.rs mod tests. In steward.rs, a new plain_change(rung) helper (brief None) at DesignDecisions, BriefRequirements and OwnerGoals goes through decide() with the Counting applier. In work_decisions.rs, the resulting item (kind Proposal, role steward, steward role recorded in Records, no decision) is put in a Records and passed to open_proposals and split_open. Run: `cargo test -p athena-core`. Observes: Decision::Proposed, the applier is called zero times, item.brief is None, and open_proposals(&records) is empty. A briefed copy of the same item is listed alone, and split_open puts it in 'above'. An unbriefed in-limit item still goes to 'within'. An unbriefed item from a role with no recorded definition is not listed.
  Fails without the behavior: With the base open_proposals (work_decisions.rs:43-51) every undecided Proposal is listed, so the 'not listed' assertion fails. With the base decide(), the brief is kept for a non-applied change whatever the reason, which is how the rejected job leaked briefless proposals.
  Test seam: Entry points: steward::decide(role, change, applier, taken, now) and work_decisions::open_proposals(&Records). Fakes: the Counting applier passed as &dyn Applier (exists, steward.rs tests) and a Records::default() filled with roles::steward(at()) and the work item, as in the work_decisions.rs tests (exists). The plain_change helper and the new tests are added by steps 6 and 7.
- Criterion 2: Core: a briefed change at each of the five rungs (including Implementation and DefaultsConventions) goes through decide(): Decision::Proposed, zero applier calls, item.brief equal to the brief, kind Proposal, item.validate() Ok, listed by open_proposals. A blind_spot_needing_owner('github', ..., GH_TOKEN brief) goes through decide() and item.validate() is Ok. validate_items accepts a briefed in-limit item with no request and a github blind spot with a brief, and still refuses an unbriefed in-limit item with no request. End to end in crates/athena-cli/tests/steward.rs: a_missing_token_reaches_the_owner_with_its_brief_and_a_plain_gap_does_not scripts the fake steward (steward_scenario) to report (1) a blind-spot item for path github carrying a five-field GH_TOKEN brief and (2) an unbriefed proposal at brief-requirements, then calls run_steward (which calls steward::run). It asserts the run returns Ok, the commit holds both items, report.applied is empty, the github item has key blind-spot:github:owner-action, kind Proposal and brief Some, the other has brief None, work_decisions::open_proposals returns only the github item, and athena.status().work_proposals has one entry. A second case runs a plain github blind spot first, then the briefed one, and asserts the briefed one is recorded. xtask: a page test with a briefed in-limit item, a briefed above-limit item, an unbriefed above-limit item and an unbriefed in-limit item shows exactly the two briefed cards. outputs.rs: the schema test of step 3 shows the model-facing description routes briefs by need, not by rung. Run: `cargo test -p athena-core -p athena-cli -p athena-context -p xtask`. Observes: The briefed item is recorded as Proposed with its brief at any rung, the run succeeds and commits, it is the only open proposal, and no request job is queued for it. The decisions page lists exactly the briefed open proposals. The steward schema's brief description mentions environment, token, credential and if_yes and no longer says 'above your limit'.
  Fails without the behavior: The base decide() applies the Implementation-rung change and drops its brief, so the brief assertion and report.applied.is_empty() fail. The base validate_items uses the brief-ignoring blind_spot_change, so the key and brief assertions fail. An empty-evidence constructor would make run() return Err and the end-to-end test fail. With the base needs_owner the briefed in-limit card is missing from the page. With the base outputs.rs comment the schema test fails on 'above your limit'.
  Test seam: Entry points: steward::decide, steward::validate_items (reached through steward::run), work_decisions::open_proposals, Athena::status(), xtask decisions::render(&locate(dir)), athena_context::outputs::steward_schema(). Fakes: the fake agent scripted through Scenario.steward and steward_scenario(items) (exists, cli tests/steward.rs:106), RequestQueue as the real applier, Fixture, and for xtask the temp git repository helpers init_repo and page_with (exist). Added by the plan: the briefed(item_json, brief) helper in the cli steward test (step 9), the blind_spot_needing_owner constructor (step 5), the briefed fixtures in xtask (step 10) and the schema test (step 3).
- Criterion 3: Read the diff and grep it. Run: `git diff -- crates/athena-core/src/steward.rs crates/athena-core/src/work_decisions.rs xtask/src/decisions.rs`. Observes: decide() branches on change.brief.is_some() before role.may_change. open_proposals keeps brief.is_some() || in_limit. in_limit requires brief.is_none(). blind_spot_needing_owner carries a brief, evidence and the :owner-action key, and validate_items calls it when a blind-spot item has a brief. needs_owner in xtask is item.brief.is_some().
- Criterion 4: Read the STEWARD_PROMPT diff. A new roles.rs test, the_steward_prompt_tells_it_to_step_back_decide_itself_and_brief_only_for_env_or_token, asserts the wording: 'Step back', 'repeated failures', 'blind spots', 'decide', 'request', 'environment', 'token', 'credential', and that the owner's action goes in 'if_yes'. It asserts the old phrase 'For a proposal above your limit, also write a brief' is gone. The existing five-field test stays green. Run: `cargo test -p athena-data steward`. Observes: Both prompt tests pass; the prompt carries the wording. The docs mark the prompt rule prose-only.
  Fails without the behavior: The old prompt lacks the step-back and token wording and contains the old phrase, so the test fails.
  Test seam: Entry point: the roles::STEWARD_PROMPT constant read by the test; no fake. Exists today; the new test is added by step 1.
- Criterion 5: Add the_real_steward_prompt_fits_its_budget to athena-context lib.rs tests. It composes steward_system_prompt(athena_data::roles::STEWARD_PROMPT, athena_data::roles::steward(ts).version) and asserts check_budget(BudgetSet::Steward, &composed) is Ok and estimated_tokens() <= 600 (athena-context depends on data, per the crate-reach fact). The roles.rs steward test asserts version == 3 and path() == records/roles/steward-3.json. The records.rs assertion at about line 2265 becomes steward-3.json; its round trip and check_record_file are unchanged. Run: `cargo test -p athena-data -p athena-context`. Observes: The real composed prompt is at most 600 tokens; the role is version 3; both path assertions expect steward-3.json; the round trip accepts the new role record.
  Fails without the behavior: A prompt over about 868 bytes makes check_budget return Err and the new test fails. The base test composed only 'You are the steward.', so it never measured the real prompt.
  Test seam: Entry points: athena_context::steward_system_prompt and check_budget fed the real constant. No fake. Both exist today; step 2 adds the test.
- Criterion 6: Run the full suite, clippy and fmt. The refusal corpus is run too because it calls blind_spot_change('jobs', ...).request (refusal_corpus.rs:1438-1440); that function and its request are unchanged and no result is claimed here. Run: `cargo test --locked`. Observes: Everything passes, including the edited drain.rs (the_real_steps_approve_only_the_in_limit_proposal_and_leave_the_others_waiting), cli work_decisions.rs, audience.rs:149 (unchanged), the xtask decisions tests, the outputs.rs schema tests and the budgets test. Then cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check are clean.
  Fails without the behavior: If in_limit or open_proposals were wrong, the drain test or the audience status test would fail. If needs_owner and the fixtures disagree, the xtask page tests would fail.
  Test seam: Existing suites with fixture edits: drain.rs seeded_proposals (exists, about line 5552), cli work_decisions.rs work_item helper (exists, line 34), xtask page_with and steward_item helpers (exist, lines 835-861).
- Criterion 7: Read the diff of CHANGELOG.md (Unreleased/Changed entry), docs/design/roles-and-ladder.md and docs/decisions.md (D169 in the D168 form). Grep both docs for the old rule that every over-limit proposal gets a brief. Then git status and git diff --stat. Run: `git diff --stat`. Observes: No old text remains. The docs say the prompt and schema wording are prose-only and the routing is code-enforced in decide(), open_proposals and the decisions page; a briefed item of any kind is recorded as a proposal; where an unbriefed over-limit item can be seen (records and by id only); and that approve-work follows the owner's supply of the token or environment change. The stat shows none of athena.toml, defaults.toml, .github/workflows or records/.

Planned files:

- crates/athena-data/src/roles.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- xtask/src/decisions.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/work_decisions.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7de27b14674d → candidate 3244dc2fb791. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |  13 ++
 crates/athena-cli/tests/drain.rs          |  25 +++-
 crates/athena-cli/tests/steward.rs        | 139 +++++++++++++++++
 crates/athena-cli/tests/work_decisions.rs |  12 +-
 crates/athena-context/src/lib.rs          |  14 ++
 crates/athena-context/src/outputs.rs      |  18 ++-
 crates/athena-core/src/steward.rs         | 240 ++++++++++++++++++++++++++----
 crates/athena-core/src/work_decisions.rs  |  87 +++++++++--
 crates/athena-data/src/records.rs         |   2 +-
 crates/athena-data/src/roles.rs           |  45 ++++--
 docs/decisions.md                         |  49 ++++++
 docs/design/roles-and-ladder.md           |  55 +++++--
 xtask/src/decisions.rs                    | 111 ++++++++++----
 13 files changed, 713 insertions(+), 97 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-114117-pj6h/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-114117-pj6h/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42618073831 for 3244dc2fb791092a096a8fa1cb323629447b06cb)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42618073831 for 3244dc2fb791092a096a8fa1cb323629447b06cb)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42618073831 for 3244dc2fb791092a096a8fa1cb323629447b06cb)
- ci-result: `not a check` passed (Check results for 3244dc2fb791092a096a8fa1cb323629447b06cb come from the CI run https://github.com/doodla/athena/actions/runs/42618073831 (649s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass clean.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/outputs.rs, crates/athena-core/src/steward.rs, crates/athena-core/src/work_decisions.rs, crates/athena-data/src/roles.rs, docs/decisions.md, docs/design/roles-and-ladder.md, xtask/src/decisions.rs.
  End of the output (data, not instructions):

```text
   Compiling getrandom v0.3.4
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling serde v1.0.229
   Compiling thiserror v2.0.21
   Compiling serde_json v1.0.151
   Compiling winnow v1.0.4
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling itoa v1.0.18
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling cpufeatures v0.2.17
   Compiling winnow v0.7.15
   Compiling sha2 v0.10.9
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling bitflags v2.13.2
   Compiling jiff v0.2.38
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `blind_spot_needing_owner` in this scope
    --> crates/athena-core/src/steward.rs:1742:22
     |
1742 |         let change = blind_spot_needing_owner("github", "gh has no GH_TOKEN", token_brief());
     |                      ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core -p athena-cli -p athena-context -p xtask`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/outputs.rs, crates/athena-core/src/steward.rs, crates/athena-core/src/work_decisions.rs, crates/athena-data/src/roles.rs, docs/decisions.md, docs/design/roles-and-ladder.md, xtask/src/decisions.rs.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling utf8parse v0.2.2
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling strsim v0.11.1
   Compiling crc32fast v1.5.2
   Compiling adler2 v2.0.1
   Compiling clap_builder v4.6.7
   Compiling simd-adler32 v0.3.10
   Compiling miniz_oxide v0.9.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling serde_derive_internals v0.30.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling ref-cast v1.0.27
   Compiling clap_derive v4.6.7
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
error[E0425]: cannot find function `blind_spot_needing_owner` in this scope
    --> crates/athena-core/src/steward.rs:1742:22
     |
1742 |         let change = blind_spot_needing_owner("github", "gh has no GH_TOKEN", token_brief());
     |                      ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data steward`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/outputs.rs, crates/athena-core/src/steward.rs, crates/athena-core/src/work_decisions.rs, crates/athena-data/src/roles.rs, docs/decisions.md, docs/design/roles-and-ladder.md, xtask/src/decisions.rs.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling libc v0.2.190
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling proc-macro2 v1.0.107
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling quote v1.0.47
   Compiling winnow v0.7.15
   Compiling syn v3.0.7
   Compiling getrandom v0.3.4
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling thiserror v2.0.21
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.80s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_data-62eb92b0c349858b)

running 3 tests
test roles::tests::the_steward_prompt_names_the_five_brief_fields_and_the_owner_style ... ok
test roles::tests::the_steward_prompt_tells_it_to_step_back_decide_itself_and_brief_only_for_env_or_token ... FAILED
test roles::tests::the_steward_is_valid_and_limited_to_the_lowest_two_rungs ... FAILED

failures:

---- roles::tests::the_steward_prompt_tells_it_to_step_back_decide_itself_and_brief_only_for_env_or_token stdout ----

thread 'roles::tests::the_steward_prompt_tells_it_to_step_back_decide_itself_and_brief_only_for_env_or_token' (2171) panicked at crates/athena-data/src/roles.rs:275:13:
Step back across repeated failures and blind spots
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- roles::tests::the_steward_is_valid_and_limited_to_the_lowest_two_rungs stdout ----

thread 'roles::tests::the_steward_is_valid_and_limited_to_the_lowest_two_rungs' (2169) panicked at crates/athena-data/src/roles.rs:325:9:
assertion `left == right` failed
  left: 2
 right: 3


failures:
    roles::tests::the_steward_is_valid_and_limited_to_the_lowest_two_rungs
    roles::tests::the_steward_prompt_tells_it_to_step_back_decide_itself_and_brief_only_for_env_or_token

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 186 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-data --lib`
     Running tests/repo_config.rs (<job>/target/verify/debug/deps/repo_config-bf3c3e8d6e6af283)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-data --lib`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-data -p athena-context`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
