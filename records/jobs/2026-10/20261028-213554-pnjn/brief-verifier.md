# Job 20261028-213554-pnjn: verify "Fix newcomer-rated gaps on the firstmate study site page"

## Request (verbatim)

> try again on 20261027-123701-77zh

## Reading (accepted by the user)

Retries job 20261027-123701-77zh (dispatched, never verified, no candidate): redo four fixes so the generated Athena site page, rated 3/5 by a newcomer, has no unexplained f2 gap, no early study-id paragraph, a one-sentence firstmate definition before "Words used first", and README install and example request in Next steps.

Assumptions:
- Job 20261027-123701-77zh had no candidate and no findings, so nothing is kept or dropped from it; all five of its deliverable files are re-listed here and its out_of_scope items are repeated below.
- Code survey: I read crates/athena-data/src/records.rs (AnyRecord at line 2386, check_record_file at line 2686) and crates/athena-cap-study/src/rules.rs (line 1435 loads studies/firstmate-work-handling for the study rules). The page is built by xtask/src/site.rs, entry `cargo run -p xtask -- site`, which loads pin, claims, model, rationales, findings and intro, and fails on an uncited element. The change adds no record or data file under crates/. studies/ JSON is not a records/ file, so check_record_file and AnyRecord are untouched. The study-rules test in crates/athena-cap-study/src/rules.rs (cargo test) must keep passing when findings.json or intro.json change. Properties are measured directly by building the page and grepping the generated index.html, not through a stand-in.
- README.md Quick start (lines 12-16) gives `./setup.sh` and `athena ask "add a changelog"`; these are the install command and example request, copied verbatim.
- For f2, the producer chooses between renumbering the findings and adding a stated reason on the page. Renumbering means editing claim-subject references too.
- The audience is a newcomer to Athena who rates the page. The change is not a refusal-rule change.

## Acceptance criteria

1. [behavior] Run `cargo run -p xtask -- site`. In the generated index.html under target/site, the Limits of this study finding ids are consecutive, or the page states why f2 is missing. Check by grep.
2. [behavior] In the generated index.html, the text 'Its study id, firstmate-work-handling' is absent from the header. If it is kept, it appears after the Claims and sources content.
3. [behavior] In the generated index.html, a one-sentence plain description of firstmate with a claim link appears before the 'Words used first' heading. Check the byte offsets with grep.
4. [behavior] In the generated index.html, the Next steps section contains `./setup.sh` and `athena ask "add a changelog"` (HTML-escaped), matching README.md Quick start. Check by grep.
5. [behavior] Run `cargo test -p xtask`, `cargo test`, `cargo clippy` and `cargo fmt --check`. All pass. The build still errors on an element that cites no claim.
6. [content] Only xtask/src/site.rs, studies/firstmate-work-handling/intro.json and findings.json, CHANGELOG.md and docs/design/explainer-site.md change. athena.toml, defaults.toml, workflows, claims.json, model.json, rationales.json and pin.json are untouched.
7. [content] A newcomer reading top to bottom learns what firstmate is before any firstmate-specific term, and can install and try Athena from Next steps without leaving the page.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)

## Out of scope

- Claims, model, rationales and pin data of the study (claims.json, model.json, rationales.json, pin.json)
- Athena README content, athena.toml, defaults.toml and workflows
- Glossary rewrites, the repetition of yolo rules, and showing cost or checker output on the page
- Records under records/
- Committing the rebuilt site output

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the generated page, not the activity.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria grep the built index.html.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the page.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page is written for a newcomer.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 67b4b3efae3c → candidate 486a2fb53554. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                  |  8 ++++++++
 docs/design/explainer-site.md |  5 +++++
 xtask/src/site.rs             | 15 +++++++++++++++
 3 files changed, 28 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
