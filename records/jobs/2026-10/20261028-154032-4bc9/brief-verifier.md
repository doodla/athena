# Job 20261028-154032-4bc9: verify "Ask root-cause requests for missing stage information before a refusal"

## Request (verbatim)

> Priority: high
> The error review's root-cause requests default to adding a new rule that refuses work (for example 'add a reading-stage check'), which is how the word-matching rule of 2026-10-27 came about, and on 2026-10-28 such a request (20261028-082300-y4yj) was stopped twice in planning because its new rule would refuse past accepted readings. Most of these causes are a stage missing information, not a stage needing to be refused. Change the shared steward request builder's root-cause template in hestia.rs: (1) ask first for the information the failing stage lacked to be given to it (in its brief, the system facts or the reader's inputs); (2) allow a new refusal rule only when giving the information cannot prevent the failure, and then require the request to state the expected share of past accepted readings it would refuse, so the replay can hold it against that number.
> Acceptance:
> - A test shows a root-cause request built from a fixture introspection asks first for the missing information to be supplied to the stage, and mentions a refusal rule only with an expected refusal share.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/hestia.rs, CHANGELOG.md

## Reading (accepted by the user)

Change the body built by root_cause_request_at in hestia.rs so a root-cause request first asks that the information the failing stage lacked be given to it (its brief, the system facts or the reader's inputs), and allows a new refusal rule only when that cannot prevent the failure, stating the expected share of past accepted readings it would refuse. Adds a fixture test and a CHANGELOG.md entry. Job 20261028-082300-y4yj (the motivating request, stopped in planning) is not retried: its reading/briefs changes are dropped.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs (root_cause_request, root_cause_request_at at ~464-521, which builds the body and acceptance with StewardRequest and checks the text with plain_boundary_text; existing tests near lines 3792 and 4020 build requests from fixture introspections via root_cause_request). The change edits text only, adds no record or data file, so the records allow-list (check_record_file/AnyRecord) is not touched. The property is measured directly by calling root_cause_request on a fixture cause and asserting on the returned text.
- Goal and audience: the Hestia steward's root-cause requests are read by the reader/planner/producer of the follow-up job; they should steer toward supplying missing information rather than adding refusal rules.
- Job 20261028-082300-y4yj failed in planning because its reading had no intended-refusal criterion and a new refusal rule would refuse past readings. Its finding does not apply here: this change adds no refusal rule in reading.rs, so no replay refusal is intended and no intended-refusal criterion is needed. Its candidate changed no files, so nothing is kept; the reading.rs, briefs.rs and run/mod.rs work it scoped is dropped and left to a follow-up request built with the new template.
- The wording must pass plain_boundary_text (admission word scan) and the builder's no-bullet-line rule; the producer rewords if the scan rejects a word.
- The template wording is mine to choose: information step first, then a refusal rule only if information cannot prevent the failure, requiring the line 'Expected share of past accepted readings it would refuse: N%'.
- athena.toml, defaults.toml and workflows need no change; existing tests asserting the request text are updated only where the old body text is pinned.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/hestia.rs builds a root-cause request with root_cause_request from a fixture introspection/cause and asserts the text asks first (earlier position in the text) for the missing information to be supplied to the failing stage (brief, system facts or reader's inputs). Check: cargo test -p athena-core hestia passes and the test is listed.
2. [behavior] The same test asserts the text mentions adding a refusal rule only as allowed when giving the information cannot prevent the failure, and requires the request to state the expected share of past accepted readings it would refuse; the asserted refusal-rule mention comes after the information ask. Check: cargo test -p athena-core hestia.
3. [content] The test fails against the old body text: the diff of crates/athena-core/src/hestia.rs shows the old body (find the cause where it originates...) extended in root_cause_request_at with the new paragraphs and no other change to the acceptance list semantics.
4. [content] Existing root-cause tests (e.g. the ones near the_root_cause_request_is_high_and_refuses_a_cause_that_reads_as_acceptance) still pass, and the built request still passes plain_boundary_text and the no-bullet-line rule. Check: cargo test -p athena-core.
5. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
6. [content] CHANGELOG.md has a new entry describing the root-cause template change (information first, refusal rule only with an expected refusal share). Check: git diff CHANGELOG.md.
7. [content] The file list of the change contains only crates/athena-core/src/hestia.rs and CHANGELOG.md. Check: git status.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding or changing any refusal rule in reading.rs, check_reading_acceptance or the replay (job 20261028-082300-y4yj's scope, dropped)
- Changes to crates/athena-context/src/briefs.rs and crates/athena-core/src/run/mod.rs
- Changing athena.toml, defaults.toml or workflows
- Changing planner, critic or reader prompts
- Changing gate-review or other steward request templates

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require a new test plus cargo test and clippy -D warnings.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts the built request text, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only hestia.rs and CHANGELOG.md.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 3075dd4d2b8a → candidate 4e07aa40e1bf. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  4 ++++
 crates/athena-core/src/hestia.rs | 35 ++++++++++++++++++++++++++++++++++-
 2 files changed, 38 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261028-154032-4bc9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-154032-4bc9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42752210363 for 4e07aa40e1bf92c32f6ce05de4fd3eaee32ed26f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42752210363 for 4e07aa40e1bf92c32f6ce05de4fd3eaee32ed26f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42752210363 for 4e07aa40e1bf92c32f6ce05de4fd3eaee32ed26f)
- ci-result: `not a check` passed (Check results for 4e07aa40e1bf92c32f6ce05de4fd3eaee32ed26f come from the CI run https://github.com/doodla/athena/actions/runs/42752210363 (728s): fmt success, clippy failure, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
