# Job 20261027-044511-kktq: verify "Add version-pin and check-tool-path defaults to defaults.toml"

## Request (verbatim)

> Add two entries to defaults.toml, in the form the reading step already reads. (1) Version pins in operator config, such as claude_version, stay exact and never float. The operator bumps them with one explicit command, and any refusal caused by a mismatch names that command. (2) A check never relies on inherited environment or ambient tools: the runner passes every tool path a check needs explicitly. When a check can use a preferred tool or a fallback, its output records which one it used, and a silent fallback counts as a defect. Done when both entries are in defaults.toml, a test shows the reading step's default context includes them, and the existing defaults and budget tests pass. Today defaults.toml contains neither policy (read: a case-insensitive grep found no match for either).
> 
> Acceptance:
> - The change is made as described: The reading step keeps asking the user policy questions that a written default should answer (version pins; check tool paths and silent runner fallback)
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: defaults.toml

## Reading (accepted by the user)

Add two [[default]] entries to defaults.toml in the existing id/statement/applies_to/source form: exact version pins bumped by one named command, and checks that get explicit tool paths and record any fallback. The audience is the reader step, which should stop asking the user these questions. A test shows the reader's default context contains both.

Assumptions:
- Goal: the reader step answers version-pin and check-tool-path questions from written defaults instead of asking the user. Audience: the reader, producer and verifier that receive defaults.
- Code survey: crates/athena-cli/tests/budgets.rs (every_role_set_is_within_its_budget composes the reader from defaults().render()), crates/athena-context/src/lib.rs (BudgetSet::Reader = 2_000 tokens, check_budget), crates/athena-core/src/proposals.rs (DefaultsModel::parse and defaults.toml editing). defaults.toml is data parsed at runtime, not a record file under crates/, so check_record_file and AnyRecord do not apply.
- Budget risk: the reader prompt has 3 tokens left, and it renders every default. Two new entries add roughly 100 tokens, so the producer must shorten existing long statements (root-cause, behavior-evidence) or the budget test fails. Raising the 2_000 limit in crates/athena-context/src/lib.rs is a fallback only if trimming cannot pass, and it is not planned.
- The new test goes in an existing test file in crates/athena-cli/tests/ or the crates/athena-core tests, wherever defaults() can be imported; no governance file other than defaults.toml changes.
- Exact applies_to: ["config", "build", "code"] for both entries. Both are prose-only unless a mechanism exists, and the statement says so.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry with id "pinned-versions" (or similar). Its statement says version pins in config, such as claude_version, stay exact and never float, are bumped only by one explicit command, and any refusal caused by a mismatch names that command. Check with Grep on defaults.toml.
2. [content] defaults.toml has a second new [[default]] entry whose statement says a check never relies on inherited environment or ambient tools, and the runner passes every tool path a check needs explicitly. It also says that when a check can use a preferred tool or a fallback, its output records which one was used, and a silent fallback is a defect. Check with Grep on defaults.toml.
3. [content] Both entries have applies_to and a source citing the request or job, matching the form of the other entries, so DefaultsModel::parse accepts the file. The diff touches only defaults.toml plus a test file, and the version field and existing entries keep their meaning.
4. [behavior] A new test builds the reader's default context from the real defaults.toml (defaults().render(), as in crates/athena-cli/tests/budgets.rs) and asserts the text contains both new ids and key phrases. Run cargo test and see it pass.
5. [behavior] The existing defaults tests and every_role_set_is_within_its_budget pass under cargo test; the reader set stays within its 2,000-token budget, which has 3 tokens of headroom today, by keeping the new statements terse and tightening wordy existing statements without dropping their content.
6. [behavior] cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test all pass.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing mechanisms that enforce the version-pin or tool-path policies in the runner.
- Adding a bump command or changing any claude_version handling.
- Changing athena.toml or CI workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass, with a new test for the reader's default context.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the rendered reader context contains both entries.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to defaults.toml plus one test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The entries state whether they are prose-only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The reader context must show the policies so that questions are no longer asked.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 19c70bcf8a21 → candidate ef4a1d8aa1bb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |  9 +++++++++
 crates/athena-cli/tests/budgets.rs | 32 ++++++++++++++++++++++++++++++++
 defaults.toml                      | 22 +++++++++++++++++-----
 docs/system-facts.md               |  2 +-
 4 files changed, 59 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-044511-kktq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-044511-kktq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42488553593 for ef4a1d8aa1bb209cf0587de136f2d31b7079fdcf)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42488553593 for ef4a1d8aa1bb209cf0587de136f2d31b7079fdcf)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42488553593 for ef4a1d8aa1bb209cf0587de136f2d31b7079fdcf)
- ci-result: `not a check` passed (Check results for ef4a1d8aa1bb209cf0587de136f2d31b7079fdcf come from the CI run https://github.com/doodla/athena/actions/runs/42488553593 (668s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
