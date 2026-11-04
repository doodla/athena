# Job 20261104-002852-g39r: verify "Add steward-written brief fields to proposals and render them first"

## Request (verbatim)

> Decision cards, third approach. Two tries rewrote proposal titles into questions with word-replacement code; both were rejected (a loop that hangs the site build, garbled questions). Plain-language questions need a model, not string rules. Instead: when the steward raises a proposal above its limit, it also writes five short fields into the work-item record: question (one yes/no line in plain words), if_yes, if_no, cost, recommendation with a one-line reason. The decisions page renders those fields first and puts evidence paths in a collapsed section. A proposal without the fields shows its title and 'No brief written yet'. No text transformation code in the site build. Acceptance: a test that a steward proposal record with the five fields renders them before any path; a test that a record without them renders 'No brief written yet'; the steward role prompt states the five fields and the owner-style rules (short, plain, no jargon).

## Reading (accepted by the user)

The steward, when it raises a proposal above its limit, writes five plain-language fields (question, if_yes, if_no, cost, recommendation with a one-line reason) into the work-item record. The decisions page (xtask/src/decisions.rs) renders them before any evidence path, puts paths in a collapsed section, and shows the title with 'No brief written yet' when absent. No text-rewriting code is added. Audience: the owner who decides proposals.

Assumptions:
- Code survey: I read crates/athena-data/src/records.rs: WorkItem (deny_unknown_fields, validate()) gains one optional `brief` field (serde default, skip if none) holding the five fields, so existing records still parse. The records validation is check_record_file (records.rs:3003), which parses bytes into AnyRecord::WorkItem and then runs WorkItem::validate; the new field is accepted there, and the existing round-trip test using sample_work_item (records.rs ~3737) is extended to cover a work item with a brief and one without. I also read crates/athena-core/src/steward.rs: decide() builds the WorkItem from a ProposedChange and returns Decision::Proposed above the role's limit, so the brief is carried from ProposedChange into the item only on that path. Every other WorkItem constructor (steward.rs, audience.rs:98, work_decisions.rs and drain.rs tests) must set brief None.
- Code survey: the entry point for rendering is xtask/src/decisions.rs: Brief::new from a WorkItem, Brief::html, render, and the existing check that a brief has a recommendation line and is under MAX_WORDS. The new fields are rendered directly from the record, as stored text escaped with the existing escape; the existing derived recommendation() string is replaced for items with a brief. Evidence paths move into an HTML details element. The existing word-limit check must not reject a record without a brief.
- Code survey: the steward prompt is STEWARD_PROMPT in crates/athena-data/src/roles.rs:120; the prompt budget in crates/athena-context/src/lib.rs (steward_system_prompt) must still fit, so the added text stays short. How the steward model's output reaches ProposedChange (its output parser in steward.rs) must be extended to read the five fields.
- The user's stated rule 'no text transformation code in the site build' is read as: no title-to-question rewriting; escaping and layout only.
- Governance files (athena.toml, defaults.toml, workflows) are not changed; the pages workflow already renders this page.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test builds a steward proposal WorkItem with all five fields and asserts the rendered page contains question, if_yes, if_no, cost and recommendation text, each at an earlier byte offset than the first evidence path, and that the paths sit inside a collapsed <details> element.
2. [behavior] Run `cargo test -p xtask`: a test renders a proposal WorkItem with no brief and asserts the output contains its title and the exact text 'No brief written yet', and that rendering does not fail.
3. [content] STEWARD_PROMPT in crates/athena-data/src/roles.rs names the five fields (question, if_yes, if_no, cost, recommendation with a one-line reason) and the owner-style rules (short, plain, no jargon); a unit test in crates/athena-data asserts these terms are present (checkable with Grep).
4. [behavior] Run `cargo test -p athena-data`: the records round-trip and check_record_file tests accept a work item with a brief and one without (old records still parse), and reject a brief with an empty field.
5. [behavior] Run `cargo test -p athena-core`: a steward test shows decide() on a rung above the role's limit yields a WorkItem carrying the brief, and that the steward output parser reads the five fields.
6. [content] Grep of xtask/src/decisions.rs shows no new code that rewrites, splits or replaces words in titles; the page code only escapes and lays out stored fields.
7. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including the existing decisions-page tests and the prompt budget tests.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/roles.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/audience.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/work_decisions.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any code that rewrites proposal titles into questions
- Back-filling briefs for existing proposals
- Changing athena.toml, defaults.toml or workflows
- Changing the decision approve/reject commands

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for both render cases, record round-trip and the prompt; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered output order and the exact fallback text.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The model writes the plain-language fields, not string rules.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Page text and prompt are written for the owner as an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One optional field and one render change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the named cargo tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cf1001b2864c → candidate b695fb75f34a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   7 ++
 crates/athena-cli/tests/drain.rs          |   2 +
 crates/athena-cli/tests/work_decisions.rs |   1 +
 crates/athena-context/src/outputs.rs      |  21 ++++-
 crates/athena-core/src/audience.rs        |   1 +
 crates/athena-core/src/steward.rs         | 131 +++++++++++++++++++++++++++++-
 crates/athena-core/src/work_decisions.rs  |   1 +
 crates/athena-data/src/records.rs         |  95 ++++++++++++++++++++++
 crates/athena-data/src/roles.rs           |  24 +++++-
 xtask/src/decisions.rs                    | 128 +++++++++++++++++++++++++++--
 10 files changed, 399 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41759069529 for b695fb75f34a37d22e0f45f5e7b4584b1a50dd96)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41759069529 for b695fb75f34a37d22e0f45f5e7b4584b1a50dd96)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41759069529 for b695fb75f34a37d22e0f45f5e7b4584b1a50dd96)
- ci-result: `not a check` passed (Check results for b695fb75f34a37d22e0f45f5e7b4584b1a50dd96 come from the CI run https://github.com/doodla/athena/actions/runs/41759069529 (516s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including the existing decisions-page tests and the prompt budget tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
