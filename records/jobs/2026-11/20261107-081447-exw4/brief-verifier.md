# Job 20261107-081447-exw4: verify "Retry: make planner and critic derive history rules from all commit"

## Request (verbatim)

> try again on 20261027-054225-ycks

## Reading (accepted by the user)

Retry of job 20261027-054225-ycks: fix the root cause (landing, history and attribution rules derived from the happy-path commit shape and tested on an unrealistic fixture) in the planner and critic prompts, with a pinning test. Audience: Athena's planner and critic, and users who see fewer repeated landing failures. The earlier candidate is kept in substance and redone, plus the docs budget file it had to touch.

Assumptions:
- Job 20261027-054225-ycks was rejected on 'check test failed'. The recorded failure was the flaky drain test athena_cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain plus an unfinished xtask shard. The verifier said it passes locally and is unrelated to the change, so it is treated as CI noise and retried unchanged, not fixed here. If it recurs, that is a separate job.
- The earlier candidate's five files are all kept as deliverables: CHANGELOG.md, critic.md, planner.md, lib.rs, and docs/system-facts.md. The last one is added because the budget lines (planner 966, critic 902) must be updated when the prompts grow; the verifier called it a necessary consequence, so it is now an expected deliverable.
- Introspection 20261107-081426-m469 proposes first improving the failed-check output extraction. The user's typed request takes precedence, so that is out of scope here; the producer should write the change description and the test names so a failing test is easy to see.
- Code survey: crates/athena-context/src/lib.rs has prompt-set tests, including a frame-size bound (estimate_tokens < 700, around line 404) that the earlier attempt raised to 900, and the system-facts budget test system_facts_budgets_match_computed. crates/athena-context/prompts/planner.md and critic.md have 334 and 398 tokens of headroom against 1300. No new record or data file is added, so the records allow-list is not touched.
- Keep the planner and critic additions compact so each stays under 1300 tokens; the earlier attempt reached 1175 and 992. Prefer not raising the 700 frame bound if the new text can fit, otherwise raise it minimally and say why.
- Root cause named in the CHANGELOG entry: a rule derived from the happy path, not from the full set of shapes the system produces. The guard is prose plus a phrase-pinning test, marked prose-only, per the mechanism default.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md requires that, for any landing, history-check or attribution rule, the plan include an enumerated table of every commit shape the real branch contains (job commits, records-only, owner-path, intervention, retry-after-block landing) and state the rule's outcome for each; Grep finds the requirement.
2. [content] planner.md requires demonstrations of such rules to use a realistic-branch fixture with at least one intervention commit, one records-only commit and the job landing, to say where the fixture differs from the real repository, and to run the tests of every crate that calls the changed classification functions.
3. [content] crates/athena-context/prompts/critic.md has a checklist item that raises a major finding when a landing, history or attribution plan lacks the commit-shape table, lacks the realistic fixture, or omits those crates' tests.
4. [content] The CHANGELOG.md entry names the root cause (rule derived from the happy path, not the full set of shapes), the fix location (planner and critic prompts), the class seen in introspections 20261024-154443-51wt, 20261024-161503-q0pp and 20261025-012736-2q6w, and that the guard is prose-only.
5. [content] docs/system-facts.md budget lines match the computed planner and critic token counts, both at most 1300; reader.md, producer.md, verifier.md, athena.toml, defaults.toml and the workflows are unchanged.
6. [behavior] A new test in crates/athena-context/src/lib.rs asserts that the planner and critic prompts contain the commit-shape table and realistic-fixture requirements. Run `cargo test -p athena-context`: it fails when the prompt edits are reverted and passes with them, and system_facts_budgets_match_computed passes.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` pass; the unrelated drain test is not modified.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Code changes to land.rs, gh.rs, reconcile.rs, run.rs, invariant.rs or other landing and history code
- Editing reader.md, producer.md or verifier.md (reader and verifier budgets are full)
- Changing athena.toml, defaults.toml, .gitattributes or CI workflows
- Adding a code-level check that rejects plans automatically
- Fixing the flaky drain test in crates/athena-cli/src/drain.rs
- Improving failed-check output extraction (introspection 20261107-081426-m469's proposal)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the planner and critic prompts require the table and fixture, and a test fails without the change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the planner and critic prompts, where the happy-path derivation originates.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A phrase-pinning test is added, and the guard is marked prose-only in the CHANGELOG.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New athena-context test; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test must fail on the reverted prompts.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt change, plus the budget doc it forces.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c3c6386e7b87 → candidate e18e4c329094. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  8 +++++++
 crates/athena-context/prompts/critic.md  |  1 +
 crates/athena-context/prompts/planner.md |  1 +
 crates/athena-context/src/lib.rs         | 36 ++++++++++++++++++++++++++++++--
 docs/system-facts.md                     |  4 ++--
 5 files changed, 46 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-081447-exw4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-081447-exw4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42356238111 for e18e4c32909404e7afad5da9ac801859e179d68a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42356238111 for e18e4c32909404e7afad5da9ac801859e179d68a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42356238111 for e18e4c32909404e7afad5da9ac801859e179d68a)
- ci-result: `not a check` passed (Check results for e18e4c32909404e7afad5da9ac801859e179d68a come from the CI run https://github.com/doodla/athena/actions/runs/42356238111 (740s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` pass; the unrelated drain test is not modified.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
