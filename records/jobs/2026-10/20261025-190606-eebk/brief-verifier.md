# Job 20261025-190606-eebk: verify "Cut slowest athena-cli test binaries' run time without weakening tests"

## Request (verbatim)

> try again on 20261025-183644-t67q

## Reading (accepted by the user)

Retry of the test-suite speedup, following the user's reply: nextest is rejected (584 s vs 673 s, about 13%), so the job instead finds where lifecycle, job_plan, drain, retry and steward tests spend time and cuts it. Audience: the user, who reads docs/decisions.md and later job times.

Assumptions:
- The user's latest reply (do not adopt nextest; speed up the slowest binaries; target full cargo test under 6 minutes) overrides the earlier nextest readings.
- Test paths are inferred from the user's binary names; the producer finds the real files and shared helpers.
- The 6-minute target is measured by the producer on its own machine and reported; the verifier can only confirm tests pass.
- Request priority is high and speed first, but this is not marked urgent since it only asks for a speedup.

## Acceptance criteria

1. [content] docs/decisions.md has a new decision with per-binary before and after times (measured in the job, labeled as measured) for athena-cli tests/lifecycle.rs, job_plan.rs, drain.rs, retry.rs and steward.rs, and the cause found for each (sleeps, poll intervals, repeated setup, locks, or real git/process work).
2. [content] The decision records the compile 67 s versus run 673 s split (4 cores, measured by the brain) and lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as a rejected alternative with that reason; it also lists sccache per D79 and a shared target dir as considered alternatives, and claims no unmeasured number.
3. [content] No test is deleted, ignored, or has its assertions weakened; the test count under cargo test --locked is not lower than before, and no athena.toml, ci.yml or tool pin is changed. CHANGELOG.md has an entry.
4. [content] Speedups come from test-only mechanisms such as shorter test intervals, fixtures built once, or avoiding needless git/process work, and production defaults are unchanged unless a test-only override is added.
5. [behavior] Run `cargo test --locked`: it passes with every test still running, and the job's reported wall time is under 6 minutes on its machine. A test that fails still fails the run.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-core/src/run.rs (modify)

## Out of scope

- Adopting cargo-nextest or any new pinned tool
- Changing athena.toml, ci.yml, or the test check
- Removing, ignoring, or weakening any test
- Adopting sccache or a shared target directory

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require measured before/after times and a passing full suite, not just edited files.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision numbers must be measured in the job or labeled as brain-measured or estimates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must be clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to the test timing change, with no tool or CI changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria use cargo commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the new decision in the diff and compare its numbers with the producer's reported timings. Run: `git diff -- docs/decisions.md`. Observes: A new decision with before/after seconds labeled measured for lifecycle, job_plan, drain, retry and steward, and a cause per binary.
- Criterion 2: Read the decision text. Run: `git diff -- docs/decisions.md`. Observes: Compile 67 s vs run 673 s on 4 cores labeled brain-measured; nextest 0.9.148 (584 s vs 673 s, about 13%) listed as rejected with that reason; sccache per D79 and a shared target dir listed as considered; no other unmeasured number.
- Criterion 3: Check the changed-file list and the test diffs; compare the `cargo test --locked` test totals with the baseline total in the producer report. Run: `git diff --stat`. Observes: No athena.toml, ci.yml or tool pin changed; CHANGELOG.md has an entry; no #[test] removed or #[ignore] added; assertions are unchanged.
- Criterion 4: Read the diff of checks.rs, run.rs and the tests. Run: `git diff -- crates`. Observes: Only the poll backoff (the 200 ms cap unchanged) and fixture/test changes; no default config or timeout value changed.
- Criterion 5: Run the full suite and read the totals. Run: `cargo test --locked`. Observes: All tests pass with none ignored; the timeout and kill-switch tests in checks.rs still pass, so a failing or slow check still fails the run. Wall time is the producer's report; the verifier confirms only the passes.
  Fails without the behavior: If the backoff broke deadline or kill handling, the checks.rs timeout tests (300 ms deadline on `sleep 30`) would fail; if a test were dropped, the total count would fall.
- Criterion 6: Run clippy, then `cargo fmt --all --check`. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Both exit 0 with no warnings or diffs.
  Fails without the behavior: Unformatted or lint-warning code would make them fail.

Planned files:

- crates/athena-core/src/checks.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base fbc7c24ccb52 → candidate ebf7dfda0532. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  8 +++++
 crates/athena-core/src/checks.rs  |  6 +++-
 crates/athena-gate/src/secrets.rs | 71 +++++++++++++++++++++++++++++++++++----
 docs/decisions.md                 | 55 ++++++++++++++++++++++++++++++
 4 files changed, 133 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1240 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the recorded check results: acceptance criteria not met or not reported: "Run `cargo test --locked`: it passes with every test still running, and the job's reported wall time is under 6 minutes on its machine. A test that fails still fails the run.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
