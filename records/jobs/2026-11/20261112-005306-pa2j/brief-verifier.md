# Job 20261112-005306-pa2j: verify "Fix flaky drain test that leaves CI shard 3 unfinished"

## Request (verbatim)

> Diagnose and fix the test parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn in crates/athena-cli/tests/drain.rs. In jobs 20261029-204942-2zqw and 20261111-213010-8jgz, its CI test shard (xtask test-shard 3 of 6) was recorded as 'did not finish' and the job was rejected. Neither job changed drain code, and both verifiers reran the test alone and saw it pass. Job 20261029-233047-ab43 also saw it fail in a full parallel run and pass alone. Steps: (1) Run the test alone many times, and again under parallel load, as in the full suite. Measure how long it takes. Find the cause, for example the drain_concurrent barrier waiting out its deadline with a bound of 1, or a wait that depends on timing. State the cause from those measured runs. (2) Fix that cause in the test or in its helpers, so the test passes in 20 repeated runs under load and finishes well within the shard limit. Do not just raise a timeout or skip the test. (3) Add a CHANGELOG.md entry. Run cargo fmt --check, cargo clippy --all-targets -D warnings and the full cargo test.
> 
> Acceptance:
> - The change is made as described: One drain test, a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn, leaves its CI shard unfinished and rejects jobs that never touch the drain
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/tests/drain.rs., CHANGELOG.md

## Reading (accepted by the user)

Retries/follows up jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43: measure parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn alone and under load, find the timing-dependent cause, and fix it in crates/athena-cli/tests/drain.rs (or its helpers) without raising timeouts or skipping. Add a CHANGELOG.md entry. Audience: CI and verifiers who reject jobs on this flake.

Assumptions:
- Code survey: read crates/athena-cli/tests/drain.rs mod parallel (line 8205 on): Shared stub implements Steps; run() logs start/end, and has 5-second spin barriers for `meet` and `wait_for`; the target test (line 8523) uses neither, with run_ms 0, calling drain::drain_concurrent with bounds 3, 1 and 3. The entry point is drain_concurrent; the cause must be found by running the test, not assumed. The records validation allow-list and record files are not touched.
- Cause is unproven: I could not run anything. The producer must measure alone and under load (e.g. cargo test -p athena-cli --test drain repeated, plus parallel load) and state the cause from the timings. Likely candidates: the drain worker threads under CPU load, a deadline wait in the shared stub, or an order assumption that depends on thread scheduling, in assertions like started[..3] contains s1 or the run <= 2 streak.
- Job 20261029-204942-2zqw: its candidate changes (docs/spec.toml, briefs.rs, planner.rs, CHANGELOG.md for OLY-8-05) are dropped, not kept; its only finding that applies here is the flaky drain test, which this job fixes. Its other findings concern a different change and no longer apply.
- Job 20261111-213010-8jgz: its candidate changes (introspection.rs, status.rs, display.rs, status_build_measurement.rs, docs/spec.toml, CHANGELOG.md) are dropped; its flake finding is fixed here, and its other findings do not apply to this job.
- Job 20261029-233047-ab43: landed, so its work (learning.rs, lib.rs, status.rs, display.rs, docs/spec.toml, CHANGELOG.md) is already in the tree and not touched. Only its observed flake (failed in a full parallel run, passed alone) is addressed.
- Environment: the CI shard is run by xtask test-shard 3 of 6 on GitHub-hosted runners; this job only runs local cargo commands and cannot read CI or push. Shard time limit is unknown to me; the producer should record the measured time of the test.

## Acceptance criteria

1. [behavior] Running `cargo test -p athena-cli --test drain a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn` repeatedly (20 times) passes every time, including while the rest of the drain test binary runs in parallel, and each run finishes in seconds (well under the shard limit).
2. [content] The diff in crates/athena-cli/tests/drain.rs removes the cause of the flake (a timing-dependent wait or scheduling-dependent assertion) and does not merely raise a timeout, add #[ignore], or delete the test; its assertions on start order (s1 among first three, at most two non-Steward starts in a row, 8 starts, improvement_turns equal to s1, s2) are kept.
3. [content] CHANGELOG.md has a new entry naming the test and the measured cause, and mentioning jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (recorded Athena checks, D115).
5. [content] No file under crates/ other than crates/athena-cli/tests/drain.rs changes, and no drain production code behavior changes, so the other tests are unaffected.

## Expected deliverables

- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing drain production code in crates/athena-core or elsewhere unless the measured cause is there
- Changes to athena.toml, defaults.toml, xtask or CI workflows, including the shard layout or time limits
- docs/spec.toml, OLY-8-05, OLY-10-03 and OLY-9-09 work from the earlier jobs; these candidates are dropped
- Changing other drain tests except shared helpers needed for the fix

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and full cargo test criteria
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the measured cause, not raise a timeout or skip
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require repeated runs under load
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to one test file plus CHANGELOG
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 920fe055dba5 → candidate 2b1befdc0837. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  1 +
 crates/athena-cli/tests/drain.rs | 22 +++++++++++++++++++++-
 2 files changed, 22 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261112-005306-pa2j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-005306-pa2j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43560526237 for 2b1befdc08376f94de66ab53b0fd700b3b4078d1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43560526237 for 2b1befdc08376f94de66ab53b0fd700b3b4078d1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43560526237 for 2b1befdc08376f94de66ab53b0fd700b3b4078d1)
- ci-result: `not a check` passed (Check results for 2b1befdc08376f94de66ab53b0fd700b3b4078d1 come from the CI run https://github.com/doodla/athena/actions/runs/43560526237 (893s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (recorded Athena checks, D115).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
