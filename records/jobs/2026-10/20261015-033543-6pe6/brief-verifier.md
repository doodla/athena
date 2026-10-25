# Job 20261015-033543-6pe6: verify "Watch the Pages deploy after a merge and re-run it once if it fails"

## Request (verbatim)

> When an external call fails with a transient error such as HTTP 5xx or a timeout, retry it a bounded number of times with backoff before failing the job. After a merge, watch the GitHub Pages deploy and re-run it once if it times out or is cancelled. Record each retry. Add tests for a 5xx error that is retried and then succeeds, and for one that keeps failing until the retries run out.

## Reading (accepted by the user)

The 5xx/timeout retry with backoff, per-retry reporting, and 5xx tests (retried-then-succeeds, persistent) already exist in gh.rs, jev.rs and crates/athena-cli/tests/external_calls.rs. The missing piece is the post-merge Pages step: after a PR merges, find the pages workflow run for the merged commit, wait for it, and re-run it once through the gh wrapper if it timed out or was cancelled, reporting that re-run. The audience is the repo owner running `athena land --pr`.

Assumptions:
- Existing transient-retry behavior (Retry::STANDARD, 3 attempts, doubling backoff with jitter, ui line per retry) already meets the first and third sentences, so it is not rebuilt.
- Only the Pages watch is new. Its call is a `gh api` REST call to list workflow runs for the merged commit and to POST the re-run. The wait is bounded, and a Pages failure is reported but does not undo a landed merge.
- "Record each retry" is met by the existing ui report line per retry; the Pages re-run uses the same mechanism and adds no new record type.
- The workflow file .github/workflows/pages.yml does not need to change.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core -p athena-cli`. A new test shows a Pages run that concludes `timed_out` after a merge is re-run exactly once via the gh wrapper and the run then succeeds. The test asserts that exactly one re-run call was made and that one report line names the run and the conclusion.
2. [behavior] Run `cargo test -p athena-cli`. A new test shows a Pages run concluding `cancelled` is re-run once, and that a second failure of the re-run is reported and not re-run again (no loop).
3. [behavior] Run `cargo test`. A test shows a Pages run that concludes `success` is not re-run, and a merge in a repository with no Pages run for the merged commit does not fail the landing.
4. [behavior] Run `cargo test`. The existing 5xx tests still pass: one is retried and succeeds, the other keeps failing until the three attempts run out. They cover the new Pages calls too (a 5xx on the runs listing is retried), and each retry is reported.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean, and `external_calls_guard` still passes: all GitHub calls stay in crates/athena-core/src/gh.rs.
6. [content] A decisions entry or doc note says how the Pages watch works: the bound on waiting, a single re-run, and that the outcome is reported to the user.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/tests/external_calls.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/land.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)

## Out of scope

- Changing the existing 5xx retry bounds or backoff for gh and Jev calls
- Editing .github/workflows/pages.yml or enabling the Pages source setting
- Retrying 4xx or auth failures
- Re-running CI workflows other than pages

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the Pages re-run use the injected runner and the existing fake gh pattern.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The retry is not duplicated, because the single wrapper already handles it. The Pages step reuses it.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The external_calls_guard test keeps every gh call inside gh.rs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change adding only the missing Pages step.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 712aba17cef1 → candidate ce1e3008ce60. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |   4 +
 crates/athena-cli/tests/external_calls.rs | 168 +++++++++++++++++++++++-
 crates/athena-core/src/gh.rs              | 206 ++++++++++++++++++++++++++++++
 docs/decisions.md                         |  24 ++++
 docs/design/explainer-site.md             |   8 ++
 5 files changed, 407 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (637 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
