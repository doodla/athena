# Job 20261103-235114-kawe: verify "Rewrite decision cards with per-proposal question and outcomes"

## Request (verbatim)

> try again on 20261103-201548-qt18

## Reading (accepted by the user)

Retry of rejected job 20261103-201548-qt18: rework each card on decisions.html (xtask/src/decisions.rs) so the owner sees a plain yes/no question, an "If yes" and "If no" line that differ per proposal, a short cost/risk line, and a recommendation only when a record states one; evidence moves into a collapsed details block. Over-limit proposals say "you must decide" rather than "Recommend no".

Assumptions:
- Code survey: read crates/athena-data/src/records.rs (WorkItem: role, kind, key, title, rung, source, evidence; validate()), crates/athena-core/src/steward.rs (ProposedChange, title/request text and tests near line 1504) and xtask/src/decisions.rs (Brief::new, Brief::check, MAX_WORDS, render, locate). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. Behavior is measured by rendering Brief::html directly in xtask tests.
- Job 20261103-201548-qt18 addressed: its candidate changed CHANGELOG.md, docs/conventions/owner-style.md, xtask/src/decisions.rs, xtask/src/site.rs. All four are kept as deliverables and reworked from the base (the candidate was rejected and discarded). The collapsed details layout, word cap and recommendation-line refusal idea are reused; each of its findings is fixed: prefix-only question, identical yes/no lines, invented 'Recommend no', unchecked jargon in real titles, the 'Recommend yes' wording, and the reading misread gap.
- WorkItem has no recorded change, cost, risk or recommendation field, and adding fields is out of scope. So per-card text is built from recorded fields that do vary: title, rung, raising role's recorded limit, source kind and evidence count and kinds. The question is rewritten around the proposal's title content by rung-specific wording, not a fixed prefix on the title. If a title cannot be phrased as a decision, the card says so plainly rather than faking it.
- No field records a recommendation, so no card shows one. An over-limit or role-unknown proposal states 'You must decide'; a proposal within the role's limit says Athena may change it itself. Brief::check no longer demands a recommendation line but requires a decision-owner line.
- Brief::check also enforces the jargon ban (rung names, role names, 'job', 'work item', 'proposal', 'blind spot', 'commit', 'path') on visible text, so real titles that contain jargon are refused or reworded, not only fixtures.
- Audience: the owner, who said on 3 Nov the cards were 'garbo'. Filtering which proposals show stays with job 20261103-201526-t4qv.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test builds cards from at least two realistic Steward-shaped WorkItems (different title, rung, source, evidence count) and asserts the question lines differ, the 'If yes' lines differ, the 'If no' lines differ, each contains a fact from its own proposal, and no question equals the title with a fixed prefix.
2. [behavior] Run `cargo test -p xtask`: a test asserts a proposal above the raising role's limit renders 'you must decide' and the text 'Recommend no' appears on no card; a proposal with no recorded recommendation renders none.
3. [behavior] Run `cargo test -p xtask`: a test asserts the byte offsets of the question (ending '?'), 'If yes', 'If no' and cost/risk lines are all smaller than the first evidence path, record id or record link, and that these sit inside a `<details>` element without `open`.
4. [behavior] Run `cargo test -p xtask`: a test renders a sample page and checks each card's visible words are within MAX_WORDS, it has exactly one question, and Brief::check refuses a card whose visible text contains banned jargon, including from a real-looking title.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass, including existing xtask site tests that read decisions.html.
6. [content] Grep xtask/src/decisions.rs: 'Rung:', 'Raised by:', 'Source:' appear only inside the details section, and the module doc describes the new layout; CHANGELOG.md has an entry and docs/conventions/owner-style.md matches the behavior.
7. [content] The diff touches no file under records/, crates/, athena.toml, defaults.toml or .github/workflows.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- xtask/src/site.rs (modify)
- docs/conventions/owner-style.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing which proposals are listed (job 20261103-201526-t4qv).
- Adding fields to WorkItem or any record schema, or any file under crates/.
- athena.toml, defaults.toml, .github/workflows, records/ and prompt files.
- Recording or applying decisions from the page; it stays display-only.

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert that text differs per proposal and carries its facts, not just that lines exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: xtask tests plus fmt, clippy and workspace tests.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Jargon ban and decision-owner line enforced in Brief::check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the invented recommendation and prefix-only question causes, not the one instance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog updated for outside readers.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks target the owner reading the card, not the code wiring.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1e18fcf123cf → candidate 563fa4a8e8a9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                    |  23 +-
 docs/conventions/owner-style.md |  24 +-
 xtask/src/decisions.rs          | 714 ++++++++++++++++++++++++++++++++++------
 xtask/src/site.rs               |   1 +
 4 files changed, 639 insertions(+), 123 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41745345160 for 563fa4a8e8a9d991c06385acf06a1817a00c23c4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41745345160 for 563fa4a8e8a9d991c06385acf06a1817a00c23c4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41745345160 for 563fa4a8e8a9d991c06385acf06a1817a00c23c4)
- ci-result: `not a check` passed (Check results for 563fa4a8e8a9d991c06385acf06a1817a00c23c4 come from the CI run https://github.com/doodla/athena/actions/runs/41745345160 (619s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass, including existing xtask site tests that read decisions.html.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
