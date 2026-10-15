# Job 20261015-112611-kccy: verify "Gate drain's 'target met' on a source check; fix places-held counter"

## Request (verbatim)

> Drain stopped on 'target met' because the audience rated the explainer 4 of 5, but an independent reader who checked the page against firstmate's pinned source rated it 3: the body still states the work-slot counter wrongly (only a recorded PR or cleanup frees a place) and model.json contradicts itself on it. A newcomer reading only the page can't catch wrong facts, so the check that ends drain has to.

## Reading (accepted by the user)

Drain stops on 'target met' from the audience rating alone, which can't catch wrong facts. I read this as two fixes: correct the places-held counter in the firstmate study (a place frees only when a worker records its ready PR or is cleaned up, per claim c3) and make model.json consistent. Then make drain's stop on target met depend on a check of the page against the pinned source, not only on the rating. The audience is the drain operator and the newcomer who reads the page.

Assumptions:
- The wrong body text is in the firstmate-work-handling study (intro.json and model.json); I did not find a different page source.
- The 'independent check' is a mechanism inside drain, such as a model-free consistency check of model.json against claims or an independent reader step, and it may reuse the existing validation rules in studies/*/validation.json.
- The cheapest sound design is an extra gate on TargetMet; a full independent LLM reader inside drain is more than this change needs.
- No change to athena.toml, defaults.toml or workflows is needed.

## Acceptance criteria

1. [content] In studies/firstmate-work-handling/model.json the places-held counter's changed_by lists every transition that frees or takes a place (spawn t1, the ready-PR report t3, and cleanup t4), matching claim c3 and t2's retry.changes 'falls when another worker records its ready PR or is cleaned up'; the file no longer contradicts itself.
2. [content] No page text in intro.json says a place is freed by anything other than a recorded PR or a cleanup; each such sentence cites c3.
3. [behavior] Drain no longer reports Stop::TargetMet from a rating at or above the target alone: it also requires a passing check of the page against the pinned source. A test in crates/athena-cli/tests/drain.rs shows a 4 rating with a failing check does not stop drain, and a 4 rating with a passing check does. Run `cargo test -p athena-cli`.
4. [behavior] A test shows the check fails on the earlier contradiction (a counter's changed_by omitting a transition whose retry changes it). Run `cargo test`.
5. [behavior] The drain report says which check held drain back when it keeps running. Run `cargo test`; `cargo clippy` is clean and `cargo fmt --check` passes.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- studies/firstmate-work-handling/validation.json (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Re-running the study against firstmate's source
- Changing the audience role's rating scale or prompt
- Changing the other four drain stop reasons
- Editing records/

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is that the stop depends on a rating that cannot detect wrong facts, so the fix gates the stop, not only the one wrong sentence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain tests cover the gate; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The check is enforced in code, not left as prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are run with cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the data fix plus one gate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 75a2e36db460 → candidate 8f8f7743b26a. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                               |  12 +++
 crates/athena-cli/src/display.rs           |   8 ++
 crates/athena-cli/src/drain.rs             | 164 ++++++++++++++++++++++++++++-
 crates/athena-cli/tests/drain.rs           |  66 +++++++++++-
 docs/decisions.md                          |  21 ++++
 studies/firstmate-work-handling/intro.json |   4 +-
 studies/firstmate-work-handling/model.json |   2 +-
 7 files changed, 269 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (684 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
