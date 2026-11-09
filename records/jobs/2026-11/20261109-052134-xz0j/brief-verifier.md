# Job 20261109-052134-xz0j: verify "Retry: trim agent prompts and rebase budgets, keeping every hard rule"

## Request (verbatim)

> try again on 20261109-035221-jj8g

## Reading (accepted by the user)

Retry of job 20261109-035221-jj8g: rewrite the role and capability prompts leaner and rebase the token budgets, restoring the verifier's cited-document rule and the other dropped guidance. Audience is the maintainer, who found the prompts mediocre; no hard rule may be removed without a stated new home.

Assumptions:
- Job 20261109-035221-jj8g is addressed finding by finding: (major) the verifier's cited-document rule is restored in verifier.md and pinned in a test anchor; (minor) 'Judge user-facing output as its audience would', 'personal data' in the harms list, 'Change nothing', and the reader's producer_model cheaper-vs-default guidance are restored or given a named home; (minor) docs/system-facts.md is kept as a deliverable because system_facts_budgets_match_computed forces its budget lines to follow; (nit) the contract size test stays an exact pin, updated to the new length, not loosened to an upper bound.
- The previous candidate's work is kept as the starting point where it passed: the rewritten prompts, docs/agent-reference.md, CHANGELOG.md and the budget edits. Only the findings above are corrected.
- Code survey: crates/athena-context/src/lib.rs (BudgetSet budgets, CONTRACT size test, frame anchors tests), crates/athena-context/src/briefs.rs (headroom rows), crates/athena-context/prompts/*.md and crates/athena-cap-*/prompts/*.md. A Grep of crates/ for 'cites a document' today finds only verifier.md line 6 (base). A Grep for FRAME_ANCHORS in the base finds nothing, so the anchor list is in the candidate; the producer must locate the equivalent pin list that the tests read.
- Before editing, the producer lists each prompt's hard rules (bullets and must/never lines). Afterward it states for every removed rule where it now lives. This is the prevention the introspection proposed, applied by hand in the decision entry.
- Prior prevention attempts (pre-dispatch headroom figure, running prompt tests after each cut) already exist; this job just follows them and adds the rule list and relocation table.
- Budgets rise nowhere without a matching cut elsewhere; restored rules must fit the verifier's 1300-token budget (the candidate was at 1153).
- docs/decisions.md gets an entry listing each removed passage with file and reason, claiming nothing beyond the diff. It is a docs file, not governance.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: all tests pass, including the athena-context budget tests (every set within its budget), headroom-row tests in crates/athena-context/src/briefs.rs, and system_facts_budgets_match_computed.
2. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.
3. [content] Grep of crates/athena-context/prompts/verifier.md finds the rule 'If the request cites a document, read it and compare the change with it' with 'request_reading_mismatch', and a test in crates/athena-context/src/lib.rs pins that phrase.
4. [content] Grep of the new prompts shows the other dropped guidance is present or relocated: 'Judge user-facing output as its audience would', 'personal data' in the verifier harms, 'Change nothing' for verifier/reader, and the producer_model cheaper-model guidance in reader.md.
5. [content] docs/decisions.md has an entry listing each removed prompt passage with its file, reason and new home (or 'restates the brief'), and every rule in the base prompts is either still in the prompts or listed there; git diff shows no hard rule removed without a stated home.
6. [content] The contract-size test in crates/athena-context/src/lib.rs remains an exact assert_eq! on the new length, not an upper bound.
7. [content] Each rewritten prompt that points to docs/agent-reference.md names a path that exists in the diff, and docs/agent-reference.md contains no claim (e.g. 'every rule is stated in the prompt') contradicted by the prompts.
8. [content] Outcome check: CHANGELOG.md has an entry with measured before/after token counts per prompt set, taken from the headroom rows or check_budget; no budget in lib.rs rises, and docs/system-facts.md budget lines match the computed values.

## Expected deliverables

- crates/athena-context/prompts/contract.md (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/agent-reference.md (create)
- docs/system-facts.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- The USD [budget] caps in athena.toml and defaults.toml
- Changing which roles exist or the pipeline's logic
- Study-capability behavior beyond prompt wording
- Building the proposed automated prompt-rule diff check as new code
- records/ and owner files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: before/after token count criterion
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: grep and test criteria check the rules present, not just files
- strangers: Code, commits, docs, and records are written for an outside reader. Here: decision entry and docs written for outside readers
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: restores every dropped rule and keeps the exact size pin
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused prompt-trim change
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1cdcb71270c2 → candidate 22a86e723c5c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |  24 ++++++
 crates/athena-cap-change/prompts/producer.md |   2 +-
 crates/athena-cap-change/prompts/verifier.md |   4 +-
 crates/athena-cap-study/prompts/producer.md  |   2 +-
 crates/athena-cap-study/prompts/verifier.md  |   6 +-
 crates/athena-context/prompts/contract.md    |   4 +-
 crates/athena-context/prompts/critic.md      |  16 ++--
 crates/athena-context/prompts/planner.md     |  20 ++---
 crates/athena-context/prompts/producer.md    |  13 ++-
 crates/athena-context/prompts/reader.md      |  15 ++--
 crates/athena-context/prompts/verifier.md    |  14 +--
 crates/athena-context/src/lib.rs             | 123 +++++++++++++++++++++++++--
 docs/agent-reference.md                      |  97 +++++++++++++++++++++
 docs/decisions.md                            |  37 ++++++++
 docs/system-facts.md                         |  12 +--
 15 files changed, 324 insertions(+), 65 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-052134-xz0j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-052134-xz0j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42567393551 for 22a86e723c5ce58af09290dd12c1d7279f10e19b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42567393551 for 22a86e723c5ce58af09290dd12c1d7279f10e19b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42567393551 for 22a86e723c5ce58af09290dd12c1d7279f10e19b)
- ci-result: `not a check` passed (Check results for 22a86e723c5ce58af09290dd12c1d7279f10e19b come from the CI run https://github.com/doodla/athena/actions/runs/42567393551 (745s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked`: all tests pass, including the athena-context budget tests (every set within its budget), headroom-row tests in crates/athena-context/src/briefs.rs, and system_facts_budgets_match_computed.; Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
