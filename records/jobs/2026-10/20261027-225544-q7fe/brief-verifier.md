# Job 20261027-225544-q7fe: verify "Fix firstmate study page newcomer gaps (retry of 20261027-122909-37mg)"

## Request (verbatim)

> try again on 20261027-122909-37mg

## Reading (accepted by the user)

Retry of job 20261027-122909-37mg for newcomer readers of the generated firstmate-work-handling site page. Make the four page edits (no f2 gap, study-id paragraph moved to end, one-sentence "what firstmate is" before "Words used first", README install command and example in Next steps) and rebuild. Its only major finding, the unverifiable "producer's report" criterion, is replaced by a criterion the verifier can run itself.

Assumptions:
- Job 20261027-122909-37mg's major finding (criterion 7, producer report not visible to verifier) is addressed by dropping that criterion and replacing it with a verifier-runnable one; the producer should still mention commands run, but no criterion depends on it.
- Its minor finding (page file outside working dir) is addressed by checking via cargo run and the site_page tests, which render the real page. Its nit is fixed: the closing note says 'This page's study id' instead of 'Its'.
- Page is built by xtask/src/site.rs from studies/firstmate-work-handling/*.json; the earlier deliverable list and out-of-scope list are kept.
- Audience: a newcomer reading the Athena site; install command and example come from README.md Quick start (./setup.sh and athena ask "add a changelog").

## Acceptance criteria

1. [behavior] Run `cargo run -p xtask -- site`: it exits successfully and regenerates the firstmate study page on the final snapshot.
2. [behavior] Run `cargo test -p xtask`: the site_page tests render the real page and assert that under 'Limits of this study' finding ids run f1, f2, f3 with no gap (or the page states why f2 is absent).
3. [behavior] Tests in xtask/tests/site_page.rs assert the study-id/studies/ folder paragraph is not in the header and appears at the end of the page after the claims, introduced with 'This page's study id' rather than an unclear 'Its'.
4. [behavior] Tests assert one plain sentence saying what firstmate is, carrying claim-id links, appears before the 'Words used first' heading.
5. [behavior] Tests assert Next steps contains './setup.sh' and 'athena ask "add a changelog"', both matching README.md Quick start.
6. [content] Every existing claim link and bracketed claim id still resolves to a claim on the page; claim-citation checks pass; no claim, claim source or pinned commit changes.
7. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass.
8. [content] No governance file (athena.toml, defaults.toml, workflows), README.md, other study or other page changes.

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

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered page content via tests and cargo run, not producer activity.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass; tests updated.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the page text and order.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the site build itself.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the one page.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 13925ccee78d → candidate 2fd2c4b02b66. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                  |  7 +++++
 studies/firstmate-work-handling/findings.json |  4 +--
 studies/firstmate-work-handling/intro.json    |  1 +
 xtask/src/site.rs                             | 43 +++++++++++++++++++++++----
 xtask/tests/fixtures/bad_firstmate_page.html  |  2 +-
 xtask/tests/site_page.rs                      | 32 ++++++++++++++++++++
 6 files changed, 80 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1480 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
