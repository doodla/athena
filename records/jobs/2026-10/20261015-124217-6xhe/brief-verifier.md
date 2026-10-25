# Job 20261015-124217-6xhe: verify "Make the explainer site introduce Athena and fix newcomer findings"

## Request (verbatim)

> try again on 20261015-104515-8wqq

## Reading (accepted by the user)

Rework the generated explainer page so a newcomer at doodla.github.io/athena first learns what Athena is and that the page is an example study of firstmate. Add a next-step section, linked claim sources, and a plain-language f2 limit. The producer must run the gates and map each criterion to evidence before reporting.

Assumptions:
- Audience is a newcomer arriving at the site.
- Athena repo URL is https://github.com/doodla/athena; firstmate URL is taken from the study pin.
- Source links point to files at the pinned commit on the firstmate repository host.
- Study text edits may require regenerating or updating validation.json; edit it only if the validator requires.
- The producer's final report includes a per-criterion evidence table and the results of the gate commands.

## Acceptance criteria

1. [content] In xtask/src/site.rs the page title and first heading name Athena, and an opening paragraph says what Athena is, that the page is an example of what its study capability produces, and that firstmate is the system studied. A named test asserts this.
2. [content] The 'What firstmate is' text says who uses firstmate and that its workers are AI coding agents. The captain glossary entry says plainly who the captain is, or what the study does and does not establish, and is not only 'The text says no more about who the captain is.'
3. [content] The page has a visible next-step section with anchor links to the Athena repository and firstmate's repository. Each claim source renders as an anchor to its file at the pinned commit, with link text HTML-escaped. Named tests cover the next-step links, the source links and the escaping.
4. [content] The text for finding f2 in 'Limits of this study' no longer contains 'guard_evaluated_by for nobody' and is in plain language. A test asserts the phrase is absent from the rendered HTML.
5. [behavior] Run `cargo test -p xtask`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass with no warnings. Run `cargo test` for the study validation too.
6. [behavior] Run `cargo run -p xtask -- site`. It writes target/site/index.html containing the Athena heading and the repository links, and `git status` shows no changes in the work tree from the build.
7. [content] The final report has a table mapping each criterion to evidence (test name or HTML excerpt) and records the gate command results.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- studies/firstmate-work-handling/validation.json (modify)

## Out of scope

- Changing the study's claims or pin
- Changing GitHub Pages workflow or deployment
- Adding install or run tooling for firstmate
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New site tests; cargo test, clippy and fmt must be clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Site is built and its HTML inspected.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Page copy written for an outside newcomer.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the site generator and its study text.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the framing in the generator and the study text rather than patching one string.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 3e9eb31d781b → candidate 32a06421abc6. In your working directory, `git diff HEAD~1` shows it.

```text
 studies/firstmate-work-handling/intro.json |  2 +-
 xtask/src/site.rs                          | 61 ++++++++++++++++++++++++++++++
 2 files changed, 62 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (689 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
