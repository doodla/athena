# Job 20261031-082345-0hmz: verify "Cap Hestia's own requests at high; skip causes a person closed"

## Request (verbatim)

> Priority: normal
> Stop Hestia from marking its own requests urgent; keep urgent for the person running Athena
> 
> On 2026-10-31 the queue held 13 urgent and 14 high items, so priority no longer ordered anything. Hestia's error review marks the five highest-ranked repeated causes urgent on every backfill (`athena hestia review --backfill`), and each new root-cause request is queued urgent. It also re-queued a closed item (gdgv, as 7ym7) at urgent.
> 
> Change Hestia and the error review so the requests they queue are high at most. A request they queue for a cause whose earlier request was closed by a person is not re-queued unless the cause has recurred since that close. Urgent stays available to people (`athena priority` and a `Priority: urgent` line they write).
> 
> Acceptance:
> - A test shows the backfill and a new root-cause request are queued at high, never urgent.
> - A test shows a cause whose request was closed with `athena close` is not queued again until a new introspection of that cause is recorded after the close.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

Hestia's backfill, new root-cause requests and other Hestia-queued requests (spec, throughput) are capped at high, so urgent is only for people (`athena priority`, a `Priority: urgent` line). The error review also stops re-queuing a cause whose request a person closed with `athena close`, unless a newer introspection of that cause was recorded after the close. The goal is for urgent to order the queue again, for the person running Athena.

Assumptions:
- Code survey: crates/athena-core/src/hestia.rs: root_cause_request (hard-codes Priority::Urgent), plan_backfill (BACKFILL_URGENT = 5 top-ranked get Urgent, the rest Normal), addressing_request (skips closed non-landed requests, so a person-closed cause is queued again), existing_request/backfill_step, error_review/try_error_review, spec_request (~line 604) and priority_for_minutes (throughput, Urgent at 10+ minutes). crates/athena-core/src/store.rs (JobState/OutcomeState derivation, line ~109 priority), crates/athena-core/src/manage.rs (athena close writes Dropped/Abandoned outcomes), crates/athena-data/src/records.rs (OutcomeState). The entry point is `athena hestia review --backfill` -> error_review_backfill -> plan_backfill. No new record or data file is added, so check_record_file and AnyRecord are unaffected. The rule is measured directly on job priority() and on the plan steps.
- The request's phrase 'the requests they queue' covers all Hestia-queued requests, including spec and throughput ones, so those are capped too; the throughput scale becomes High at 5+ minutes. If the owner wants only the error review changed, the throughput and spec caps would be reverted.
- A person's close is distinguished from Hestia's own requeue close (Dropped, written by Hestia) by the outcome's origin, found in manage.rs; the producer must decide how, for example by the close record's origin or note. Recurrence means an introspection of the cause (same cause_key) with a timestamp later than the close outcome.
- This change does not add or tighten a refusal rule on readings, so no newly-refused-readings percentage applies; it only holds back Hestia from queueing.

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/hestia.rs shows that plan_backfill over records with more than five repeated causes queues every request at Priority::High or lower (none urgent), and that root_cause_request produces a `Priority: high` request, not urgent. Run `cargo test -p athena-core hestia`.
2. [behavior] A test shows a cause whose earlier request was closed with `athena close` (the manage.rs close path) gets no new request or requeue from the backfill or the per-job error review. A second case shows it is queued again, at high, once a new introspection of that cause is recorded after the close. Run `cargo test -p athena-core hestia`.
3. [behavior] The backfill and the error review still queue a cause whose request was closed by Hestia itself (requeue Dropped) or was rejected or failed. A test covers at least one of these. Run `cargo test -p athena-core`.
4. [content] BACKFILL_URGENT and the Priority::Urgent uses in Hestia's request builders (root-cause, spec_request, throughput priority_for_minutes) are removed or capped at High. Grep shows no Priority::Urgent in non-test code of hestia.rs. `athena priority` and a person-written `Priority: urgent` line are unchanged: the plan.rs/store.rs parsing of urgent is untouched.
5. [content] Doc comments in hestia.rs (module docs, plan_backfill, root_cause_request) and any docs mention of urgent for Hestia are updated to say high at most.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has an entry that names the cap at high and the no-requeue after a person's close.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/ (modify)

## Out of scope

- Changing `athena priority`, plan.rs priority parsing, or letting people mark requests urgent
- Re-prioritising items already in the queue, including 7ym7 and gdgv
- Changing the repeat threshold or the ranking of causes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the high cap and the person-close rule, plus fmt, clippy and test passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests that exercise the queueing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cap is applied in every Hestia request builder, not only the backfill.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the CHANGELOG entry are updated.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, mostly in hestia.rs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 68129700bc18 → candidate 7f35c454b08c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  15 ++
 crates/athena-core/src/hestia.rs | 365 +++++++++++++++++++++++++++++++++------
 docs/design/hestia.md            |  10 +-
 docs/spec.toml                   |   6 +-
 4 files changed, 334 insertions(+), 62 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41389349655 for 7f35c454b08ce279eb7ccbbdfdadc78e6a4a0674)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41389349655 for 7f35c454b08ce279eb7ccbbdfdadc78e6a4a0674)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41389349655 for 7f35c454b08ce279eb7ccbbdfdadc78e6a4a0674)
- ci-result: `not a check` passed (Check results for 7f35c454b08ce279eb7ccbbdfdadc78e6a4a0674 come from the CI run https://github.com/doodla/athena/actions/runs/41389349655 (559s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: planned files left unchanged without a reason: docs/)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
