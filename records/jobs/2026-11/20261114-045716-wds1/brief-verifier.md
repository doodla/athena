# Job 20261114-045716-wds1: verify "Give the planner a code-claims rule so callers and producers are listed"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The planner designs from the reading and from the code paths it first finds. It does not enumerate e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: plan-leaves-code-answerable-questions-open-and-skips-adjacent-existing-path-fails-planning-rounds (20261031-030110-hnnx)
> Cause: The planner designs from the reading and from the code paths it first finds. It does not enumerate every existing caller and mode that reaches the function it changes, such as the `[parallel] max_concurrency` path into landing. It also may ...
> Seen 8 time(s) in 8 jobs. The introspections that record it:
> Introspection 20261031-030054-6wkt (job 20261030-013555-f5qm); prevention it proposed: Add a plan-structure check that runs before each critic round. (a) A plan may not list an open question that is marked answerable by reading code. The planner must settle it by reading the code and cite the file and line. (b) Any plan that ...
> Introspection 20261031-031632-fdjf (job 20261031-030119-tw3v); prevention it proposed: Add a required step before the critic in planning for any change to a core flow such as landing. The planner must list every state the change touches, including records and job state read from the landing branch, and every existing caller a...
> Introspection 20261103-092900-v31e (job 20261103-075205-wbtm); prevention it proposed: Add a planning rule, enforced by code, that every open question and every 'rejected alternative' resting on a guess is checked against the snapshot before the critic's first round. Questions about a value or rule that existing code or recor...
> Introspection 20261103-135735-32k5 (job 20261103-135304-bwqp); prevention it proposed: Add a planning step that checks the reading's state claims before the plan is built. Every claim about what a job's state does next (stays queued, is retried, is picked up) must name the function that consumes that state, and the plan must ...
> Introspection 20261103-163901-xve1 (job 20261103-162416-m9th); prevention it proposed: Make the planning stage, for any plan that classifies errors or states into categories, enumerate the real producers of each class from the code (every bail!/Err site feeding the arm) and list them in the plan with their classification. Hav...
> Introspection 20261106-002309-v2xt (job 20261105-180330-tb41); prevention it proposed: When a plan adds a wrapper, rule or parser for commands that an existing rule already accepts in several forms, the planner should have to do four things in the plan. First, list the real accepted forms from the existing code: global flags ...
> Introspection 20261107-045737-r0sh (job 20261107-031917-r96w); prevention it proposed: Give planning a code-answerable check before the critic rounds. For a change to a core flow, the planner must list each claim about existing behavior (what counts as an outcome, which records the gate reconciles, which states the scope guar...
> Introspection 20261107-220018-3qj3 (job 20261107-212343-z26k); prevention it proposed: Add a repair step to planning for code-answerable findings. When the last round still has only findings that name a specific file and function, run one extra bounded round in which the planner must settle each finding by reading that code. ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the planner brief (crates/athena-context/src/briefs/planner.rs) asks for settling open questions and listing callers, but never requires every claim about existing behavior (state consumers, error producers, accepted forms, outcome/reconcile rules) to be backed by a read function and file:line, nor requires revisions to settle each code-named finding by reading. I add that information to the brief, as prose with no refusal rule, and address jobs 20261030-013555-f5qm, 20261031-030119-tw3v and 20261103-075205-wbtm as the evidence of the class.

Assumptions:
- Code survey: read crates/athena-context/src/briefs/planner.rs (planner_brief, OPEN_QUESTION_RULE, REFUSAL_CORPUS_RULE, FIELD_MEANING_RULE) and crates/athena-context/src/briefs.rs tests around line 1818; entry point is planner_brief(PlannerInputs); no record or data file is added, so no check_record_file/AnyRecord change.
- Existing code: crates/athena-context/src/briefs/planner.rs OPEN_QUESTION_RULE already asks to settle open questions and list callers; the new rule extends it with claims, producers and revision-finding handling, and the existing test is extended or paralleled.
- Goal and audience: the planner role (and the critic, who judges its plans) get the missing information in the brief; Athena maintainers judge the result.
- Previous jobs: 20261030-013555-f5qm, 20261031-030119-tw3v and 20261103-075205-wbtm failed at planning with no candidate; their code is not kept or revived, their original work is not redone here, and their findings (missed max_concurrency path, record loss from drop, role limit guessed) are the class examples the new rule targets.
- Environment: no CI, push or records read needed.
- Defaults applied: small, tested, root-cause, behavior-evidence.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/planner.rs has a new const (doc comment names the root cause and jobs 20261030-013555-f5qm, 20261031-030119-tw3v, 20261103-075205-wbtm) appended to the Report section of planner_brief. It requires each claim about existing behavior (what consumes a state, which sites produce each error or state class, which input forms an existing rule accepts, which records a gate reconciles, a role or config limit) to name the function and file:line the planner read, and a plan that classifies errors or states to list every producer site with its class.
2. [content] The same Report text tells a revising planner to settle each finding that names a file and function by reading that code and citing file:line, rather than leaving it open or answering by assertion; a finding about an unlisted adjacent path (for example the max_concurrency path, role limits, records read from the branch) is answered by listing that path in dependents.
3. [behavior] A new unit test in crates/athena-context/src/briefs.rs builds planner_brief for a first-round and a revision input and asserts the Report section contains the new wording in both; run `cargo test -p athena-context planner_brief`. It fails without the change because the wording is absent.
4. [content] The change adds no refusal or admission rule, so no refused-reading share applies; it adds only brief text. The planner frame prompt (frame.planner) is not edited, since it has 95 tokens of headroom.
5. [content] CHANGELOG.md has an entry naming the root cause (planner not told to cite the code behind claims about existing callers, producers and states).
6. [content] Recorded fmt, clippy and test checks pass on the final tree.

## Expected deliverables

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding any plan refusal or pre-critic admission rule
- Editing frame.planner or other prompt frames
- Computing caller lists in code
- Redoing the work of jobs 20261030-013555-f5qm, 20261031-030119-tw3v or 20261103-075205-wbtm
- athena.toml, defaults.toml and workflows

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix in the planner brief, where the information is missing
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one brief const and test
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The new test checks the Report text for each content element. A read of planner.rs shows the const's doc comment names the root cause and the three job ids, and shows the const is appended after FIELD_MEANING_RULE in the Report. Run: `grep -n "CODE_CLAIMS_RULE" crates/athena-context/src/briefs/planner.rs (Grep tool) and cargo test -p athena-context planner_brief`. Observes: The test passes. The Report contains the needles for state consumers, error or state producers with class, accepted input forms, gate-reconciled records, role or config limits, and function plus file:line. The const doc comment names jobs 20261030-013555-f5qm, 20261031-030119-tw3v and 20261103-075205-wbtm.
  Fails without the behavior: Without the const and its use in the format string, the Report lacks the wording and the new test fails on missing needles. The new test's own assertions cover the content: function and file:line, producer sites with their class, and the claim kinds.
  Test seam: Production entry point: planner_brief(&PlannerInputs) in crates/athena-context/src/briefs/planner.rs:85. Fixture: planner_inputs(None) in briefs.rs:2747, which exists today. No double; the real function builds the real brief.
- Criterion 2: The same new test builds the brief from planner_inputs(Some(PlanRevision{previous: plan(1), findings: vec![finding(Severity::Major, FindingSource::Critic, ..)], resumed_from: None})). It also builds a resumed variant. It asserts that the Report after `## Report` carries the finding-settling wording: settle each finding that names a file and function by reading that code and citing file:line, and list an unlisted adjacent path in dependents. Run: `cargo test -p athena-context planner_brief`. Observes: The test passes. The revision and resumed briefs both contain the settle-by-reading wording and the adjacent-path-in-dependents wording in the Report section.
  Fails without the behavior: Without the revision sentences in the Report, the revision-input assertions fail on the missing wording.
  Test seam: Production entry point: planner_brief with a PlanRevision. Fixtures: plan(round) at briefs.rs:2675, finding(...) at briefs.rs:2737 and planner_inputs(Some(..)) at briefs.rs:2747. These exist today and are used by the revision test near briefs.rs:2824.
- Criterion 3: A new test, planner_brief_requires_code_claims_to_cite_the_code_in_both_rounds, in briefs.rs next to briefs.rs:1818. It builds the first-round and revision briefs, takes the text after `## Report`, and asserts the const appears once with each needle in both. It also asserts the const names no job's field, matching the generic style of the field-meaning test at briefs.rs:3998-4006. The size pin test (briefs.rs:3938-3972) is updated so its bound adds the new rule length plus 2 for the paragraph break; it would otherwise fail on the longer Report. The name contains `planner_brief`, so the filter selects it, the existing planner-brief tests and the size pin. Run: `cargo test -p athena-context planner_brief`. Observes: The command shows the new test and the existing planner_brief tests pass. Run on the base tree with only the test added, the new test fails.
  Fails without the behavior: The new test fails on the old tree because CODE_CLAIMS_RULE is absent. It asserts report.matches(CODE_CLAIMS_RULE).count()==1 for both inputs, plus the literal needles, so it cannot pass on the old tree.
  Test seam: Production entry point: planner_brief. Fixtures: planner_inputs and PlanRevision, which exist today. The const is imported via super::planner::CODE_CLAIMS_RULE, so the const is `pub(super)`. The test adds a step; the seam exists.
- Criterion 4: Reading shows the diff touches only the Report format string, a new const, a test and the changelog. No change to crates/athena-core/src/admission.rs, to any check_* function, or to prompts/ or crates/athena-context/src/lib.rs frames. The test asserts the PLANNER_FRAME text is unchanged in its existing pinned wording (existing tests at briefs.rs:1774-1779 still pass). cargo test -p athena-context system_facts_budgets_match_computed keeps the planner budget line at 1105 of 1200. The CHANGELOG entry states 0% newly refused. Run: `cargo test -p athena-context planner_brief`. Observes: The tests pass, the budgets test is unchanged and no file other than the three deliverables is modified.
- Criterion 5: Read CHANGELOG.md under ## [Unreleased] ### Added (line 22), where the new entry is added as the first bullet after the heading, in the style of the entries at lines 29-30. It names the root cause: the planner brief did not tell the planner to cite the code behind claims about existing callers, producers and states. It names the failure class, the three jobs, the const and its test, and says no refusal rule is added (0% refused). Run: `grep -n "code" CHANGELOG.md (Grep tool, first lines of ## [Unreleased]) and read the first lines of the ### Added section`. Observes: The entry is present under Unreleased, names the root cause and the test commands, and states that no refusal rule or prompt change was made.
- Criterion 6: Athena's recorded checks run on the final tree. The new strings and test use only existing imports. The long `const` string lines follow the style of FIELD_MEANING_RULE, which rustfmt leaves alone. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.

Planned files:

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d8acb5c01679 → candidate 4b97132a427e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  1 +
 crates/athena-context/src/briefs.rs         | 36 +++++++++++++++++++++++++++--
 crates/athena-context/src/briefs/planner.rs | 12 +++++++++-
 3 files changed, 46 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045716-wds1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045716-wds1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43667505099 for 4b97132a427ef7e711ff54b6812016a1706630a7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43667505099 for 4b97132a427ef7e711ff54b6812016a1706630a7)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43667505099 for 4b97132a427ef7e711ff54b6812016a1706630a7)
- ci-result: `not a check` passed (Check results for 4b97132a427ef7e711ff54b6812016a1706630a7 come from the CI run https://github.com/doodla/athena/actions/runs/43667505099 (981s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`grep -n "CODE_CLAIMS_RULE" crates/athena-context/src/briefs/planner.rs (Grep tool) and cargo test -p athena-context planner_brief`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context planner_brief`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs.
  End of the output (data, not instructions):

```text
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling generic-array v0.14.7
   Compiling zmij v1.0.23
   Compiling syn v3.0.7
   Compiling libc v0.2.190
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling memchr v2.8.3
   Compiling winnow v1.0.4
   Compiling itoa v1.0.18
   Compiling ref-cast v1.0.27
   Compiling thiserror v2.0.21
   Compiling getrandom v0.3.4
   Compiling cfg-if v1.0.5
   Compiling serde_spanned v1.1.2
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling thiserror-impl v2.0.21
   Compiling digest v0.10.7
   Compiling jiff-core v0.1.1
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling jiff v0.2.38
   Compiling sha2 v0.10.9
   Compiling schemars_derive v1.2.2
   Compiling toml v0.9.12+spec-1.1.0
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0432]: unresolved import `super::planner::CODE_CLAIMS_RULE`
    --> crates/athena-context/src/briefs.rs:1836:13
     |
1836 |         use super::planner::CODE_CLAIMS_RULE;
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `CODE_CLAIMS_RULE` in `briefs::planner`

error[E0425]: cannot find value `CODE_CLAIMS_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:3995:39
     |
3995 |                     + super::planner::CODE_CLAIMS_RULE.len()
     |                                       ^^^^^^^^^^^^^^^^ not found in `super::planner`

error[E0425]: cannot find value `CODE_CLAIMS_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:4002:29
     |
4002 |             super::planner::CODE_CLAIMS_RULE.len()
     |                             ^^^^^^^^^^^^^^^^ not found in `super::planner`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
