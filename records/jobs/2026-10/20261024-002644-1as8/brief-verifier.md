# Job 20261024-002644-1as8: verify "Tell change verifiers to cite recorded checks, not rerun the suite"

## Request (verbatim)

> try again on 20261023-234149-jcd2

## Reading (accepted by the user)

Add one short instruction (under ~100 tokens) to the change-capability verifier prompt: cite the recorded fmt/clippy/test results under "Checks Athena ran on this tree" instead of rerunning the full suite, running them only if missing or failed. Add a CHANGELOG entry citing the 2026-10-18 verifier timeouts. Audience: verifiers and the maintainer reading the diff.

Assumptions:
- Per the user's reply, the instruction goes only in the change verifier prompt because the study set is at 1296/1300 tokens.
- The producer runs only targeted tests while editing and the full suite at most once.

## Acceptance criteria

1. [content] crates/athena-cap-change/prompts/verifier.md tells the verifier that the fmt, clippy and test results under 'Checks Athena ran on this tree' are recorded for this candidate: cite them and do not rerun the full cargo test suite; 'cargo test passes' is met by citing the record.
2. [content] The prompt says a failed or missing recorded check is not a pass and the verifier may then run just that check; new behavior is exercised with targeted commands like `cargo test -p <crate> <filter>`.
3. [content] CHANGELOG.md has an entry naming the 2026-10-18 verifier timeouts (full suite about 11 minutes) as the reason.
4. [content] The diff touches only crates/athena-cap-change/prompts/verifier.md and CHANGELOG.md: not the shared crates/athena-context/prompts/verifier.md, no Rust source, no config.
5. [behavior] Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`; it passes and the change set stays within its 1300-token budget (was 1147).
6. [behavior] Run `cargo test -p athena-context`; it passes, showing brief snapshot tests are unbroken.

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Shared crates/athena-context/prompts/verifier.md and the study verifier prompt
- Producer watchdog, checkpoint or retry machinery
- Rust source or config changes

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt sentence plus a changelog line.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Budgets and athena-context tests named as behavior criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the prompt text and budget test outcomes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog entry cites the dated cause.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff Run: `git diff -- crates/athena-cap-change/prompts/verifier.md`. Observes: Text says fmt, clippy and test results under 'Checks Athena ran on this tree' are recorded for this candidate; cite them, do not rerun the full cargo test suite; 'cargo test passes' is met by citing the record.
- Criterion 2: Read the diff Run: `git diff -- crates/athena-cap-change/prompts/verifier.md`. Observes: Text says a failed or missing recorded check is not a pass and only that check may be run; new behavior is exercised with `cargo test -p <crate> <filter>`.
- Criterion 3: Read the diff Run: `git diff -- CHANGELOG.md`. Observes: An entry in [Unreleased] naming the 2026-10-18 verifier timeouts (full suite about 11 minutes) as the reason.
- Criterion 4: List changed files Run: `git status`. Observes: Only crates/athena-cap-change/prompts/verifier.md and CHANGELOG.md are modified.
- Criterion 5: Run the test Run: `cargo test --locked -p athena-cli --test budgets -- --nocapture`. Observes: Passes; change set tokens printed at or under 1300 (was 1147).
  Fails without the behavior: The budget test would fail if the added text pushed the change set over 1300 tokens.
- Criterion 6: Run the test Run: `cargo test -p athena-context`. Observes: All tests pass, including brief snapshot tests.
  Fails without the behavior: A snapshot test would fail if a brief embedding the change prompt changed unexpectedly.

Planned files:

- crates/athena-cap-change/prompts/verifier.md (modify)
- CHANGELOG.md (modify)

## The change

Base 2532634869d1 → candidate 86277e242a97. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 | 3 +++
 crates/athena-cap-change/prompts/verifier.md | 2 ++
 2 files changed, 5 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (994 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
