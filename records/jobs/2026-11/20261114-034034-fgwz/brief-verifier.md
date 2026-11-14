# Job 20261114-034034-fgwz: verify "Make Hestia re-queue a cause that recurs after its fix landed"

## Request (verbatim)

> From Sri (the owner), word for word: "go with a. I'd also like some fix to Hestia or the steward that allow it to catch the issues the review found. Athena fixes go first in the queue"
> 
> Background (not Sri's own words; added so the request can be read on its own):
> 
> "a" is this plan, which Sri chose to have Athena carry out itself:
> 1. Build the queued fixes first. Hold the October feature backlog for a week. Athena runs its own queued fixes (steward and Hestia) first, starting with: the misread-field bug (critic writes "None..." in request_reading_mismatch and planning stops; steward work item 20261114-022805-ws3t, request 20261114-023201-cdp5); the batch-ordering test that keeps rejecting unrelated jobs (parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn; request 20261112-005306-pa2j); the throughput clock correction (20261107-231406-f1bx); planning stopping the whole job over a reading quibble (20261108-220029-48yj); and the two items that loop on the queue (20261017-205552-y1bh $0.05 short of budget; 20261030-060410-9e5h, reading always refused): close or fix them.
> 2. Keep planning, make it cheaper: give the plan critic the earlier rounds' findings so later rounds check they were fixed. No new refusal rules unless shown to reduce rejections.
> 3. Judge the week by: landed changes per day, dollars per landed change, and queued fixes that actually ran.
> 
> "The review" is an independent review (two outside reviewers, read-only, 14 Nov) of what Athena's self-review misses. Its findings:
> 1. Steward requests are filed but never run (52 this week, none read or dispatched); nothing measures "fix queued -> fix run".
> 2. Hestia treats a failure cause as addressed forever once any landed request cites it, even if new cases follow (hestia.rs addressing_request: the landed branch has no recency check; the closed-by-person branch does).
> 3. The throughput median uses the wrong landing timestamp (f1bx), and throughput reviews keep proposing designs from it.
> 4. Hestia sweeps report nothing while errors pile up (208 "could not delete the CI branch" errors; claims without evidence 306 -> 406).
> 5. Keep-rule verdicts (e.g. the planning keep rule, sweep 20261110-104309-kk49) produce no recorded decision.
> 6. The plan critic has no memory of earlier rounds; any major finding forces another round (37 jobs stopped after 5 rounds).
> 7. Many "refuse a reading that..." gates added (18 jobs, $81) with no fall in final-verdict rejections.
> 8. fmt, clippy and the whole test run count as evidence for behavior criteria (318 of 1,733 behavior criteria in passing verifications cite only those).
> 9. Whether CI results replace local checks depends on the reading's class label, not on changed paths; a change to the CI workflow could judge itself.
> 10. A failing test on the base rejects unrelated jobs; one introspection blamed the producer.
> 11. A candidate that makes tests hang is recorded as an environment problem and resumed unchanged.
> 12. Nothing reviews landed work; introspection runs only on rejected and failed jobs; forecasts are scored against the verifier, not later outcomes.
> 13. A landed config change (planner_revision_* keys) was refused by the running Athena; the drain logged "not landed" while outcome.json says landed; nothing was introspected.

## Reading (accepted by the user)

Reads "go with a" plus the Hestia ask as one focused change: fix finding 2, so a landed request stops addressing a failure cause once a newer introspection of that cause is recorded (as the closed-by-person branch already does), in hestia.rs. Jobs 20261114-023201-cdp5, 20261112-005306-pa2j and 20261107-231406-f1bx stay queued as separate jobs, not redone here.

Assumptions:
- Goal and audience: Sri (owner) and Hestia's repeated-cause review want recurring failures to be queued again instead of hidden behind an old landed fix; the check is the unit tests and the hestia.rs doc.
- Code survey: read crates/athena-core/src/hestia.rs. addressing_request (line ~830) is called from repeated_causes (~927) and ~3052. Its landed branch (~863) accepts any landed request that cites an introspection id, with no recency check; the closed-by-person branch compares closed_at with `newest`, the newest introspection time of the group. The property is measured directly by calling addressing_request on fixture Records. No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are unaffected.
- Existing code: reuse `newest` and the `o.at` outcome time already in crates/athena-core/src/hestia.rs addressing_request; extend the landed branch with the same comparison. Update the doc comments at hestia.rs lines ~49 and ~824.
- Term ladder: not used; no structural terms in this reading.
- Previous jobs: 20261114-023201-cdp5 (misread yes/no field), 20261112-005306-pa2j (drain test) and 20261107-231406-f1bx (throughput landing clock) are queued, unverified, with no candidate and no changed files. Their work is neither kept nor dropped here: they keep their own queue entries, and this reading does not touch their files (outputs.rs, planning.rs, verify.rs, display.rs, status.rs, drain.rs, throughput.rs). Sri's 'fixes first' ordering and the other plan items (cheaper critic, week metrics, 20261017-205552-y1bh and 20261030-060410-9e5h closure) are queue or operations actions outside this single code change.
- Recurrence rule: a landed request addresses the group only if no introspection of the group has `at` later than the request's landed outcome `at`; an open request still addresses it. A request landed after the newest introspection still addresses it.
- No refusal rule is added. This only lets Hestia queue more requests for recurring causes, and that higher queue rate is intended.
- Environment: none; no CI, push or records-branch reads, tests use in-memory fixtures.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/hestia.rs shows addressing_request returns None for a cause when a landed request cites it but an introspection of the cause was recorded after the landing; run `cargo test -p athena-core hestia`.
2. [behavior] A unit test shows a landed request still addresses the cause when its landing is later than the newest introspection, and an open request still addresses it even if newer introspections exist; run `cargo test -p athena-core hestia`.
3. [behavior] Existing hestia tests (closed-by-person, repeated_causes backfill) still pass; run `cargo test -p athena-core`.
4. [content] The doc comments on addressing_request and the module doc in crates/athena-core/src/hestia.rs state that a landed request stops addressing a cause once a newer introspection is recorded.
5. [content] CHANGELOG.md has a new entry describing the Hestia recurrence fix; no other deliverables are changed beyond crates/athena-core/src/hestia.rs and CHANGELOG.md.
6. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass (recorded Athena checks on the final tree count).

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fixes for jobs 20261114-023201-cdp5, 20261112-005306-pa2j and 20261107-231406-f1bx and their files (outputs.rs, planning.rs, verify.rs, display.rs, status.rs, status/throughput.rs, crates/athena-cli/tests/drain.rs); they run as their own jobs.
- Queue ordering, holding the October backlog, closing 20261017-205552-y1bh and 20261030-060410-9e5h, and the week's metrics.
- Plan critic memory of earlier rounds and any new refusal rules.
- Review findings 1 and 3-13 other than the Hestia recency check, including a steward fix-queued-to-fix-run measure and Hestia sweeps reporting error pile-ups; propose these as follow-up requests.
- athena.toml, defaults.toml, workflows, docs/decisions.md and records/.

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change (Hestia recency) rather than the whole plan and thirteen findings.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests plus fmt, clippy, full test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run named cargo test commands on fixture records.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base be4734bf2396 → candidate 9ac56dfc7d22. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   4 ++
 crates/athena-core/src/hestia.rs | 103 ++++++++++++++++++++++++++++++++++++++-
 2 files changed, 105 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-034034-fgwz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-034034-fgwz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43555912865 for 9ac56dfc7d228cd6dbda9af0572d4203901a9662)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43555912865 for 9ac56dfc7d228cd6dbda9af0572d4203901a9662)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43555912865 for 9ac56dfc7d228cd6dbda9af0572d4203901a9662)
- ci-result: `not a check` passed (Check results for 9ac56dfc7d228cd6dbda9af0572d4203901a9662 come from the CI run https://github.com/doodla/athena/actions/runs/43555912865 (651s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass (recorded Athena checks on the final tree count).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
