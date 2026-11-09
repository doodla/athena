# Job 20261109-080220-zkhy: verify "Steward steps back and decides itself; asks owner only for env or token"

## Request (verbatim)

> The steward should be doing the step back. I don't want decisions really, unless it's an environment change or token that Athena needs

## Reading (accepted by the user)

Reads "do the step back" as: the steward should look at the whole pattern of failures and blind spots and decide the fix itself, instead of sending the owner decision briefs. It asks the owner only when Athena needs an environment change or a token (for example a missing GH_TOKEN). The audience is the owner (Sri), who judges by how few decisions reach them.

Assumptions:
- Code survey: read crates/athena-data/src/roles.rs (STEWARD_PROMPT, steward(), prompt test), crates/athena-core/src/steward.rs (brief(), check_brief, decide(), Decision::Proposed/Applied) and crates/athena-context/src/lib.rs (BudgetSet::Steward = 600 tokens). No record or data file is added under crates/. The steward is invoked after every five finished tasks, and its output is measured directly in the unit tests with the fake driver.
- Reading: 'the step back' means the steward reviews patterns and root causes, not single events. The mechanism that sends proposals above the steward's rung to the owner as briefs stays. The prompt and brief text change so the steward uses it only for environment or token needs.
- Term ladder: the requirements ladder of rungs in roles-and-ladder.md; the steward may itself change rungs at or below DefaultsConventions. Work above that rung is still only proposed by Athena, and this job leaves that code rule as it is.
- Existing code: extends crates/athena-data/src/roles.rs, crates/athena-core/src/steward.rs and docs/design/roles-and-ladder.md; nothing is created.
- Enforcement is by prompt, so prose-only; tightening it by code (refusing briefs for other items) is out of scope because it would add a refusal rule.

## Acceptance criteria

1. [content] In crates/athena-data/src/roles.rs, STEWARD_PROMPT tells the steward to step back (look across repeated failures and blind spots for the root cause) and to decide and queue a request itself. It limits a brief for the owner to an environment change or a token or credential Athena needs. Check with Grep/Read of the diff.
2. [content] The steward prompt no longer asks for an owner brief on any other kind of proposal, and the 'Your report' / 'Your limit' text in crates/athena-core/src/steward.rs agrees with the prompt. Check by reading the diff.
3. [behavior] Run `cargo test -p athena-data -p athena-core -p athena-cli`. The updated unit test in roles.rs asserts the prompt names the step back and the environment/token exception. A steward.rs test shows that an item needing no environment or token gets no brief, and one that does keeps its five-field brief.
4. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean. The steward prompt still fits the Steward token budget (600) per the existing check_budget test.
5. [content] docs/design/roles-and-ladder.md and CHANGELOG.md describe the new steward behaviour. If the stored role version changes, the version bump is applied consistently where steward() and its tests use it. The rule is marked prose-only (a model prompt) unless a mechanism is added.
6. [content] The change does not touch athena.toml, defaults.toml, workflows or records/, and adds no refusal rule, so no refusal-rate figure applies.

## Expected deliverables

- crates/athena-data/src/roles.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the steward's highest_rung or cadence
- Adding a code check that refuses briefs
- Changes to athena.toml, defaults.toml, workflows or records/
- Changing the drain, audience or auditor roles

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Located the steward's prompt and brief code in the repository rather than asking.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt to pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is marked prose-only because it lives in a model prompt.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the steward prompt and its brief text.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 80bcb0b2b35d → candidate b801ced28238. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  5 +++++
 crates/athena-core/src/steward.rs | 16 ++++++++++++++--
 crates/athena-data/src/records.rs |  2 +-
 crates/athena-data/src/roles.rs   | 29 +++++++++++++++++++----------
 docs/decisions.md                 | 14 ++++++++++++++
 docs/design/roles-and-ladder.md   |  7 +++++++
 6 files changed, 60 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-080220-zkhy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-080220-zkhy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42608696251 for b801ced28238f1a70b82f66ace764861dd37f7f1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42608696251 for b801ced28238f1a70b82f66ace764861dd37f7f1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42608696251 for b801ced28238f1a70b82f66ace764861dd37f7f1)
- ci-result: `not a check` passed (Check results for b801ced28238f1a70b82f66ace764861dd37f7f1 come from the CI run https://github.com/doodla/athena/actions/runs/42608696251 (768s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean. The steward prompt still fits the Steward token budget (600) per the existing check_budget test.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
