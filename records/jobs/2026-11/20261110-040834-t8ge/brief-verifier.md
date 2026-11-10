# Job 20261110-040834-t8ge: verify "Root-cause request: ask for missing information first (retry)"

## Request (verbatim)

> try again on 20261028-154032-4bc9

## Reading (accepted by the user)

Retry of job 20261028-154032-4bc9: change the body in root_cause_request_at (hestia.rs) so a root-cause request first asks that the information the failing stage lacked be given to it, and allows a refusal rule only when that cannot prevent the failure, with an expected refusal share stated. The prior candidate failed clippy (field_reassign_with_default in its new test); this retry must fix that and run clippy --all-targets before finishing.

Assumptions:
- Goal: steward root-cause requests steer producers/planners toward supplying missing information instead of adding refusal rules; audience is the planner and producer who read the request, and the verifier who runs the tests.
- Code survey: read crates/athena-core/src/hestia.rs: root_cause_request (line 467) calls root_cause_request_at (472), which builds the body string, then StewardRequest::new(...).render(), then plain_boundary_text over the whole text. The existing test the_root_cause_request_is_high_and_refuses_a_cause_that_reads_as_acceptance is near line 4020. No record or data file is added, so no AnyRecord or check_record_file change.
- Job 20261028-154032-4bc9 findings: the sole blocker was clippy field_reassign_with_default at the new test (Records built with Default::default() then fields assigned); fixed by using struct-update syntax `Records { introspections: vec![...], ..Default::default() }`. The earlier fmt and test results passed. Its candidate files hestia.rs and CHANGELOG.md are redone from base and kept as deliverables.
- Environment: none beyond the repository; no CI, push or history read is needed by the change.
- The change adds no refusal rule; it tells requesters to state an expected refusal share, which is prose in the generated request only. Expected 0% of newly refused readings.
- The earlier proposed prevention (a producer-stage clippy gate) is a separate pipeline change and is not part of this request; the producer should simply run cargo clippy --all-targets -- -D warnings itself.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/hestia.rs builds a root-cause request via root_cause_request from a fixture cause and records, and asserts the text asks first (earlier position) for the information the failing stage lacked to be supplied to it (brief, system facts or reader's inputs). Check: cargo test -p athena-core hestia passes with the test listed.
2. [behavior] The same test asserts a refusal rule is mentioned only as allowed when giving the information cannot prevent the failure, requires the request to state the expected share of past accepted readings it would refuse, and that mention comes after the information ask. Check: cargo test -p athena-core hestia.
3. [content] git diff of crates/athena-core/src/hestia.rs shows the body in root_cause_request_at extended with the new paragraphs; the new test builds Records with struct-update syntax (no field assignment after Default::default()).
4. [behavior] Existing root-cause tests still pass, and the built request still passes plain_boundary_text and the no-bullet-line rule. Check: cargo test -p athena-core.
5. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
6. [content] CHANGELOG.md has a new entry describing the root-cause template change (information first, refusal rule only with an expected refusal share). Check: git diff CHANGELOG.md.
7. [content] The changed files are only crates/athena-core/src/hestia.rs and CHANGELOG.md. Check: git status.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding or changing any refusal rule in reading.rs, check_reading_acceptance or the replay (job 20261028-082300-y4yj's scope, dropped)
- Changes to crates/athena-context/src/briefs.rs and crates/athena-core/src/run/mod.rs
- A producer-stage fmt/clippy repair round (the introspection's proposed prevention)
- Changing athena.toml, defaults.toml or workflows
- Changing planner, critic or reader prompts
- Changing gate-review or other steward request templates

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy --all-targets and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the built request text order and wording by running tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8770c02d2c51 → candidate e4ca2a758f8e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  4 ++++
 crates/athena-core/src/hestia.rs | 45 +++++++++++++++++++++++++++++++++++++++-
 2 files changed, 48 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-040834-t8ge/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-040834-t8ge/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42754545005 for e4ca2a758f8e8bd92f508366b9df34ae70916344)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42754545005 for e4ca2a758f8e8bd92f508366b9df34ae70916344)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42754545005 for e4ca2a758f8e8bd92f508366b9df34ae70916344)
- ci-result: `not a check` passed (Check results for e4ca2a758f8e8bd92f508366b9df34ae70916344 come from the CI run https://github.com/doodla/athena/actions/runs/42754545005 (770s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
