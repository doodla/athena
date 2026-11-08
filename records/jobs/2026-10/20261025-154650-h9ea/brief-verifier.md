# Job 20261025-154650-h9ea: verify "Make readings state GitHub Actions cache branch scoping for CI work"

## Request (verbatim)

> Change the reading role's guidance: when a request touches CI caching or CI speed, the reading must say how the GitHub Actions cache is scoped by branch (which branches can restore which entries) and must state that constraint in the criteria. Add a test or prompt fixture showing the constraint is named.

## Reading (accepted by the user)

Add one rule to the reader prompt (reader.md): when a request touches CI caching or CI speed, the reading explains how the GitHub Actions cache is scoped by branch and states that constraint in an acceptance criterion. Add a unit test in athena-context showing the prompt names it. Audience: the maintainer reviewing readings. The reader set has only 3 tokens of headroom, so the producer must trim existing reader.md wording to fit within the unchanged 2000-token budget.

Assumptions:
- Code survey: read crates/athena-context/src/lib.rs (READER_FRAME includes ../prompts/reader.md; BudgetSet::Reader budget is 2_000; existing tests compose frames and call check_budget) and crates/athena-core/src/run_pin_tests.rs and crates/athena-cli/tests/budgets.rs (reader budget line). The change adds no record or data file, so check_record_file and AnyRecord are not involved. The property is measured directly on READER_FRAME.text, not a stand-in.
- The reader set has 3 tokens left, so the new bullet (about 40-60 tokens) requires trimming equivalent tokens from reader.md; raising the budget is not assumed.
- Cache scoping facts (GitHub docs): restore from current branch, base branch, default branch; not from sibling branches.
- The 'fixture' requested is satisfied by a unit test on the prompt text.

## Acceptance criteria

1. [content] crates/athena-context/prompts/reader.md has a bullet saying that when a request touches CI caching or CI speed, the reading states how the GitHub Actions cache is scoped by branch and puts that constraint in an acceptance criterion.
2. [content] The bullet states the scoping correctly: a run restores caches from its own branch (or PR merge ref), its base branch, and the default branch; it cannot restore caches from sibling or unrelated branches; caches saved by a feature branch are not visible to the default branch or other branches.
3. [behavior] A new test in crates/athena-context/src/lib.rs asserts READER_FRAME.text names the CI cache, branch scoping, base/default branch and acceptance criterion. Run `cargo test -p athena-context`: it passes, and it would fail if the bullet were removed.
4. [behavior] The reader token budget is unchanged at 2000 (BudgetSet::Reader in crates/athena-context/src/lib.rs); existing wording is tightened so the existing budget tests still pass. Run `cargo test -p athena-context` and `cargo test -p athena-cli --test budgets`.
5. [behavior] No other prompt set exceeds its budget and the whole workspace is healthy. Run `cargo test`, `cargo clippy --all-targets` (clean) and `cargo fmt --check`.
6. [content] Governance files (athena.toml, defaults.toml, workflows) are not changed; the existing guidance in reader.md is trimmed only in wording, not in meaning.

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing CI workflows or actual cache configuration
- Raising token budgets in athena.toml or lib.rs
- Changing other prompt files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts the prompt wording is present.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One bullet plus one test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Test written in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3deeebc3f108 → candidate 731ecd47a02b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  3 +++
 crates/athena-context/prompts/reader.md | 26 +++++++++++++-------------
 crates/athena-context/src/lib.rs        | 17 +++++++++++++++++
 docs/design/context-engineer.md         |  2 +-
 docs/system-facts.md                    |  2 +-
 5 files changed, 35 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261025-154650-h9ea/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261025-154650-h9ea/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42434165680 for 731ecd47a02bcd5efb973b91c92990eb0a081df9)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42434165680 for 731ecd47a02bcd5efb973b91c92990eb0a081df9)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42434165680 for 731ecd47a02bcd5efb973b91c92990eb0a081df9)
- ci-result: `not a check` passed (Check results for 731ecd47a02bcd5efb973b91c92990eb0a081df9 come from the CI run https://github.com/doodla/athena/actions/runs/42434165680 (552s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No other prompt set exceeds its budget and the whole workspace is healthy. Run `cargo test`, `cargo clippy --all-targets` (clean) and `cargo fmt --check`.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
