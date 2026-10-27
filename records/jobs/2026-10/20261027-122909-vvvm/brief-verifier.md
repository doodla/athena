# Job 20261027-122909-vvvm: verify "Enforce rendered-page assertions for firstmate study page criteria"

## Request (verbatim)

> Add a convention, and a test in xtask that enforces it for the firstmate-work-handling study: when a job's acceptance criteria are about what a generated study page contains or must not contain, the producer adds assertions on the rendered HTML from `cargo run -p xtask -- site`, runs them before handover, and a failure blocks handover. Write the convention where producer briefs and the reading stage pick it up. Start with the firstmate page: assert the glossary entry and the next-step link come before the first transition card, assert which model-limit findings are rendered, and assert that agent-decided transitions are checked through the model's trigger_evaluated_by field. Keep a fixture that must fail.
> 
> Acceptance:
> - The change is made as described: Firstmate study page follow-up jobs are rejected one after another; nothing checks the rendered page against the criteria before the verifier sees it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-10/20261027-085315-n35f.json

## Reading (accepted by the user)

Add a convention that jobs whose criteria concern what a generated study page contains must add assertions on the rendered HTML from `cargo run -p xtask -- site`, run them before handover, and treat failure as blocking. Implement the firstmate page checks as an xtask test (glossary and next-step link before first transition card, rendered model-limit findings, agent-decided transitions checked via trigger_evaluated_by), plus a fixture page that must fail. Audience: producers and the reader/verifier stages that judge study page jobs.

Assumptions:
- Prompt token budgets prevent editing the reader prompt (3 tokens left) and verifier (2 left); the producer prompt for change has 219 tokens left, so a short rule goes there and fuller text in docs/.
- Existing unit tests in xtask/src/site.rs partly cover these checks; the new integration test in xtask/tests exercises the built HTML output and adds the failing fixture.
- The records/introspections file named under Files is context only and is not changed.
- No governance files (athena.toml, defaults.toml, workflows) need to change.

## Acceptance criteria

1. [behavior] `cargo test -p xtask` passes, including a new test that renders the real firstmate-work-handling study and asserts the glossary section and the `#next-steps` link both appear before the first transition card in the HTML.
2. [behavior] The new test asserts exactly which model-limit findings from studies/firstmate-work-handling/model.json are present in the rendered HTML (and that others are absent), and `cargo test -p xtask` passes.
3. [behavior] The new test asserts that every transition whose model `trigger_evaluated_by` is `agent` is rendered as agent-decided, derived from model.json rather than hard-coded, and passes.
4. [behavior] A committed fixture page (e.g. a deliberately wrong HTML file) is run through the same assertion helpers by a test that expects failure (glossary after first card, missing link, wrong findings, wrong evaluator), proving each check can fail; `cargo test -p xtask` passes.
5. [content] The convention is written in a producer prompt/brief location (crates/athena-cap-change/prompts/producer.md, within its token headroom) stating: criteria about generated page contents require HTML assertions on `cargo run -p xtask -- site` output, run before handover, and failure blocks handover. Reader prompt and other budget-tight prompts are not grown.
6. [content] The convention is also recorded in a doc under docs/ (stranger-readable) naming the xtask test as its enforcing mechanism.
7. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass with no unrelated behavior changes to site rendering.

## Expected deliverables

- xtask/tests/site_page.rs (create)
- xtask/tests/fixtures/bad_firstmate_page.html (create)
- crates/athena-cap-change/prompts/producer.md (modify)
- docs/design/requirements-map.md (modify)
- docs/conventions/rendered-page-assertions.md (create)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the rendered page layout or study content
- Editing the reader, verifier, or study prompts
- Changing governance files such as athena.toml or defaults.toml
- Assertions for studies other than firstmate-work-handling

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask test plus must-fail fixture; fmt, clippy, test pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Convention enforced by an xtask test rather than prose alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered HTML behavior, not file existence.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Convention documented for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to firstmate page.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Addresses missing pre-verifier render checks rather than one page fix.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 3422924da37e → candidate 8715c14227f5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |   8 ++
 crates/athena-cap-change/prompts/producer.md |   2 +-
 docs/conventions/rendered-page-assertions.md |  36 +++++
 docs/design/requirements-map.md              |   2 +-
 xtask/src/site.rs                            |   3 +
 xtask/tests/fixtures/bad_firstmate_page.html |  16 +++
 xtask/tests/site_page.rs                     | 197 +++++++++++++++++++++++++++
 7 files changed, 262 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1479 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` pass with no unrelated behavior changes to site rendering.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
