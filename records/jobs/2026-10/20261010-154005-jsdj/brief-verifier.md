# Job 20261010-154005-jsdj: verify "Trailer says no verifier ran for jobs rejected by a check"

## Request (verbatim)

> Give a job that failed its checks before any verifier ran an Athena-Verifier trailer saying that no verifier ran (n2, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

Review note n2 says a job rejected by a check before any verifier ran still gets `Athena-Verifier: <model> fail`, which reads as if the verifier judged it. The closing commit should carry `Athena-Verifier: none (check <name> failed)` instead. The audience is anyone reading a job's commit trailers.

Assumptions:
- In run.rs (~line 1476-1488) the trailer is emitted from final_verdict whenever a verification exists, whatever Verification.verifier_invocation says. That is the root cause.
- A verifier ran if and only if verification.verifier_invocation is Some.
- With no verifier run, the first failed check in verification.checks (passed == false) names the check. If none failed, the form falls back to `none (no verifier ran)`.
- Several failed checks: name the first, or join the names with commas. Either is acceptable if documented.
- The extraction of trailer construction into a small pure function is allowed so it can be unit tested.

## Acceptance criteria

1. [behavior] A new unit test in athena-core builds the closing trailers for a verification with verifier_invocation None and a failed check named fmt. It asserts the Athena-Verifier value is exactly `none (check fmt failed)` and does not contain the verifier model name. Run `cargo test -p athena-core`.
2. [behavior] A test asserts that when verifier_invocation is Some, the trailer is still `<verifier-model> pass` for a pass verdict and `<verifier-model> fail` for a fail verdict. Run `cargo test -p athena-core`.
3. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.
4. [content] In crates/athena-core/src/run.rs the Athena-Verifier trailer is chosen by whether a verifier invocation exists. The model name appears only when a verifier ran.
5. [content] docs/records.md documents the `Athena-Verifier: none (check <name> failed)` form beside the `<model> pass` line, so a reader of a rejected job's commit can tell that no verifier judged it.
6. [content] The change is focused. No governance files change. CHANGELOG.md gets an entry only if the repo's convention requires one.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)

## Out of scope

- Changing how verdicts are computed or recorded in verification.json
- Changing the gate's reading of records
- Editing the review note itself

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests, with cargo test and clippy passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Choose the trailer by whether a verifier invocation exists, not just by the verdict.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Update docs/records.md so outside readers can interpret the trailer.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the actual trailer values.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in run.rs and docs/records.md.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 44df1be3bbff → candidate 91e8ca438fa6. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                  |   6 +++
 crates/athena-core/src/run.rs | 107 ++++++++++++++++++++++++++++++++++++++----
 docs/design/athena-v0.md      |   3 +-
 docs/records.md               |   5 ++
 4 files changed, 111 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (795 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
