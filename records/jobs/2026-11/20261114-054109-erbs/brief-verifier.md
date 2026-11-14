# Job 20261114-054109-erbs: verify "Fix flaky urgent-group drain test without weakening its assertions"

## Request (verbatim)

> try again on 20261112-005306-pa2j

## Reading (accepted by the user)

Retry of job 20261112-005306-pa2j (rejected): reproduce the flake of parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn under load, state its cause from the code, and fix it in drain.rs tests so the test can still fail on a wrong drain order. Audience: CI and verifiers rejecting jobs on this flake. Also follows up 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43.

Assumptions:
- Code survey: read crates/athena-cli/tests/drain.rs (the test and its stub), crates/athena-cli/src/drain.rs (run_batches at 2855 starts workers over one VecDeque; take_until_done at 2770 pops an item and runs it, logging start, before popping again) and crates/athena-core/src/plan.rs (groups_in_order at 583 gives one group u1..u6,s1,s2 for disjoint items). Consequence: with 3 workers, s1 is popped only after six u's, so s1 can appear at log index 4 or 5 but never in the first four starts; the base flake is log-order skew between popping and logging. No records or data files are added, so check_record_file/AnyRecord do not apply.
- Previous job 20261112-005306-pa2j: finding 1 (stub made to log starts in expected order, so the assertion tested the stub) is addressed by dropping that stub gate and instead asserting on the drain's deterministic hand-out order (report.plan_groups.concat() == u1..u6,s1,s2) plus a race-free log check (s1 start index >= 4 with 3 workers). Finding 2 (changelog failure example impossible) is addressed by rewriting the entry with a failure the drain and stub can produce (s1 at index 4 or 5 of the log, or whatever the new measurement shows), taken from a fresh reproduction. Finding 3 (CI 'did not finish' unexplained) is addressed by the entry saying plainly that a hang was not reproduced and the open question remains. Its candidate's drain.rs and CHANGELOG.md changes are dropped and redone from base.
- Producer must first reproduce the flake on the base tree by running the test repeatedly in parallel under load, measure runtimes, and state the cause from that output. The original first assertion (s1 among the first three starts) is racy only if it is the failing one; if the measured cause differs, the producer keeps the intent (s1 not ahead of the improvement turn) in a form the drain can fail.
- Mutation check: the producer deliberately breaks the drain's order (temporarily, not committed) or the test's input and shows the repaired assertions fail; this is reported but only the committed diff counts.
- Governance: no change to athena.toml, defaults.toml, xtask or CI workflows; shard layout and limits untouched.

## Acceptance criteria

1. [behavior] Running `cargo test -p athena-cli --test drain a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn` passes, and the full `cargo test --locked` (which runs the drain tests in parallel) passes, each in seconds for this test.
2. [content] The diff in crates/athena-cli/tests/drain.rs does not make the stub log starts in an expected order (no gate that holds a start entry until earlier entries exist), does not raise a timeout, add #[ignore] or delete the test.
3. [content] The test in crates/athena-cli/tests/drain.rs still asserts 8 starts, at most two non-Steward starts in a row, improvement_turns equal to s1, s2, and an order check that fails if s1 is misplaced: either report.plan_groups.concat() equals u1..u6,s1,s2, or s1's start index is at least 4 with 3 workers (verifiable by reading the assertions against take_until_done in crates/athena-cli/src/drain.rs).
4. [content] CHANGELOG.md has a new Fixed entry naming the test, a cause stated in terms of drain.rs worker pop-then-log behavior, a failure example the drain and stub can produce (s1 not in the first four log entries is impossible; example shows s1 at index 4 or 5 versus the old assertion), the jobs 20261029-204942-2zqw, 20261111-213010-8jgz and 20261029-233047-ab43, and a plain statement that a never-finishing run was not reproduced.
5. [content] No file other than crates/athena-cli/tests/drain.rs and CHANGELOG.md changes; no production code under crates/ changes behavior (checked with git diff).
6. [behavior] Recorded Athena checks `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (D115).

## Expected deliverables

- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing drain production code in crates/athena-core or crates/athena-cli/src unless the measured cause is there
- Changes to athena.toml, defaults.toml, xtask or CI workflows, including shard layout or time limits
- docs/spec.toml, OLY-8-05, OLY-10-03 and OLY-9-09 work from earlier jobs; those candidates are dropped
- Changing other drain tests except shared helpers needed for the fix
- Adding a flaky-test step to the Athena job flow (the introspection's prevention); not part of this fix

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix the measured cause, not the symptom
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria check that start order is still verified by the drain
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: behavior criteria name cargo commands the verifier runs
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bce4b76f52fb → candidate 89269165c18d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  1 +
 crates/athena-cli/tests/drain.rs | 17 ++++++++++++++---
 2 files changed, 15 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-054109-erbs/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-054109-erbs/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43563124112 for 89269165c18de8eedbf090811269f1e38a8ac989)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43563124112 for 89269165c18de8eedbf090811269f1e38a8ac989)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43563124112 for 89269165c18de8eedbf090811269f1e38a8ac989)
- ci-result: `not a check` passed (Check results for 89269165c18de8eedbf090811269f1e38a8ac989 come from the CI run https://github.com/doodla/athena/actions/runs/43563124112 (958s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Recorded Athena checks `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` pass on the final tree (D115).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
