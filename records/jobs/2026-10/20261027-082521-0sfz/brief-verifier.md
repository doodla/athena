# Job 20261027-082521-0sfz: verify "Fix t3 trigger wording and make the decider test structural"

## Request (verbatim)

> try again on 20261027-074457-bx8e

## Reading (accepted by the user)

Retry of the rejected job. Make the t3 trigger in model.json say only what its cited claims support, and replace the prefix-match decider test in xtask/src/site.rs with a structural check backed by a model field. The audience is the newcomer reading the published study page, and the verifier who checks it.

Assumptions:
- The request means the introspection's proposed next request: fix the two remaining findings (t3 wording, structural decider test).
- A structural check needs a model field. I assume an optional 'trigger_actor' string is added to each transition and the schema or validator accepts it; if one already exists, the producer reuses it.
- t3 will be reworded so the actor is the worker or a script, as the claims (c3, c31, c37) say, with trigger_evaluated_by changed to match. If no claim says the agent accepts the report, the producer must not keep the agent as the subject.
- The earlier JSON spacing fix on t3, t8, t10 and t11 and the in-scope glossary entry already landed or carry over unchanged.

## Acceptance criteria

1. [content] In model.json the t3 trigger names only an event that a claim it cites supports (c3, c31, c37). It does not say the agent accepts or records the worker's report unless a cited claim says so. Its trigger_evaluated_by value matches the actor the claims name.
2. [content] The model records each transition's trigger actor in a field, not in the trigger text. Every cited claim id exists in claims.json.
3. [behavior] Run cargo test. The site test checks structurally, with no text prefix or keyword ban: for each transition with trigger_evaluated_by 'agent', the actor field says agent, and every cited claim id resolves. All tests pass.
4. [behavior] Run cargo test. A fixture with an agent-evaluated transition whose trigger describes another actor's event is rejected by the check, and a fixture with a valid agent trigger passes. The tests assert both outcomes.
5. [behavior] Run cargo run -p xtask -- site. The study builds and validates, and the generated page keeps the in-scope glossary entry and a next-step link before the first transition card. It has no f2/f3 model-limit note.
6. [behavior] cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
7. [content] The CHANGELOG.md entry describes only what was done and does not overclaim.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/claims.json (modify)

## Out of scope

- Changing how much of the page is about Athena rather than firstmate
- Other audience-rating findings that already landed
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy -D warnings and fmt --check must pass, with positive and negative fixtures.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The decider rule is enforced by a structural check on a model field, not by a text prefix.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require fixtures that the check rejects or accepts, and a run of the site build.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The wording is plain and the changelog is accurate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the t3 wording, the test, and the changelog change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base c8c993688cca → candidate 15bb49c0b880. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  7 +++
 crates/athena-cap-study/src/model.rs       |  3 +
 crates/athena-cap-study/src/rules.rs       |  1 +
 studies/firstmate-work-handling/model.json | 24 ++++----
 xtask/src/site.rs                          | 94 +++++++++++++++++++++++++-----
 5 files changed, 103 insertions(+), 26 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1429 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
