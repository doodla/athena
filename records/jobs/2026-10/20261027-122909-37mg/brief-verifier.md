# Job 20261027-122909-37mg: verify "Fix newcomer gaps on the firstmate study site page"

## Request (verbatim)

> On the generated site page for the firstmate-work-handling study: (1) under Limits of this study, either renumber the findings so there is no gap after f1, or say on the page why f2 is missing; (2) remove the header paragraph that explains the study id and the studies/ folder, or move it to the end; (3) say in one plain sentence what firstmate is before the 'Words used first' list; (4) add to Next steps an install command and one example request taken from Athena's README. Keep every claim link. Rebuild with `cargo run -p xtask -- site` and report the commands that were run.
> 
> Acceptance:
> - The change is made as described: Newcomer rating 3/5 on the Athena site: missing f2 id, insider study-id paragraph, firstmate explained late, no install command or example request
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: studies/

## Reading (accepted by the user)

The page is built by xtask/src/site.rs from studies/firstmate-work-handling/*.json. I read the request as four edits for a newcomer reader. They are: no gap in finding ids, the study-id paragraph moved to the end of the page, a one-sentence "what firstmate is" before "Words used first", and an install command plus an example request from Athena's README in Next steps. After the edits, run the site rebuild.

Assumptions:
- The generator xtask/src/site.rs and its tests xtask/tests/site_page.rs must change even though the request says Files: studies/. The study-id paragraph and Next steps are hard-coded in site.rs.
- Finding ids are renumbered, f3 to f2 and f4 to f3, in findings.json. Any other references (validation.json, tests, fixtures, CHANGELOG) are updated to match. If a reference cannot be updated safely, the page instead states why f2 is missing.
- The study-id paragraph is moved to a short closing note after Claims and sources, so the sentence about citing claims stays near the top.
- The firstmate sentence comes from the existing intro.json 'what' text, with its claim links c1, c11, c21 and c22. It is placed before the Words used first heading, probably as a new intro.json field.
- The install command is './setup.sh' and the example request is 'athena ask "add a changelog"', both from README.md Quick start.
- The audience is a newcomer reading the generated site page, and the Athena site is the target of the stated 3/5 newcomer rating.

## Acceptance criteria

1. [behavior] Run `cargo run -p xtask -- site`, then read target/site/index.html with Read. Under 'Limits of this study' the finding ids run f1, f2, f3 with no gap, or the page says why f2 is absent.
2. [behavior] In the rebuilt page, the paragraph explaining the study id and the studies/ folder is not in the header. It is absent or sits at the end of the page, after the claims.
3. [behavior] In the rebuilt page, one plain sentence saying what firstmate is appears before the 'Words used first' heading. It carries claim-id links.
4. [behavior] In the rebuilt page, the Next steps section contains the install command './setup.sh' and the example request 'athena ask "add a changelog"', both matching README.md Quick start.
5. [content] Every existing claim link and bracketed claim id still resolves to a claim on the page. The site generator's claim-citation checks still pass.
6. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass. Updated tests in xtask/tests/site_page.rs assert the new page order and the new text.
7. [content] The producer's report lists the commands it ran, including `cargo run -p xtask -- site`.
8. [content] No unrelated behavior changes. No governance file (athena.toml, defaults.toml, workflows) is touched, and no other study or page changes.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- xtask/tests/site_page.rs (modify)
- studies/firstmate-work-handling/validation.json (modify)
- xtask/tests/fixtures/bad_firstmate_page.html (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing any claim, claim source, or the pinned firstmate commit
- Changing other pages or other studies
- Rewording the glossary or the transition cards
- Changing README.md or athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass, with site tests updated for the new layout.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Rebuild the page with cargo run -p xtask -- site and check the rendered output for each of the four changes.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered page text and order, not that the code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the single study page and its generator.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: New page text is plain sentences written for an outside newcomer.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The finding-id gap is fixed in the data (findings.json), not by hiding it in the renderer.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 2e0f14ff2290 → candidate 7f7bdb741a61. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                  |  7 +++++
 studies/firstmate-work-handling/findings.json |  4 +--
 studies/firstmate-work-handling/intro.json    |  1 +
 xtask/src/site.rs                             | 43 +++++++++++++++++++++++----
 xtask/tests/fixtures/bad_firstmate_page.html  |  2 +-
 xtask/tests/site_page.rs                      | 31 +++++++++++++++++++
 6 files changed, 79 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1480 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
