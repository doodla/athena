# Job 20261111-075222-nsk3: verify "Retry: define each site term once and link elsewhere"

## Request (verbatim)

> try again on 20261029-212839-1vav

## Reading (accepted by the user)

Retry of job 20261029-212839-1vav: the firstmate study page defines yolo, green/red, in scope, places and home once (Glossary or Words used first) and links to them everywhere else, with a test that also catches reworded copies. Each finding of the rejected attempt is fixed, and its three changed files are kept as the starting point and revised. The audience is newcomers reading the homepage.

Assumptions:
- Retry of job 20261029-212839-1vav (rejected, verdict fail). The earlier candidate changed CHANGELOG.md, studies/firstmate-work-handling/intro.json and xtask/src/site.rs; all three are kept as deliverables and re-done from the base, with the earlier [[term]] link helper approach reused.
- Finding 1 (major, intro.json what[4] and limits[6]): fixed by cutting the overview secondmate clause to a bare link to home and ending the limit after freeing a place, so no definition wording survives outside the Glossary or Words used first.
- Finding 2 (major, site.rs test): fixed by replacing the exact-sentence-count check with a near-duplicate check: no run of 6 or more normalised words from a definition may appear in prose outside the definitions.
- Finding 3 (minor): fixed by linking the remaining mentions (t2, t11 summaries, glossary exit 75 and secondmate entries, t10 trigger) and rewording the t10 guard in model.json so it does not echo the red definition.
- Code survey: xtask/src/site.rs renders the study loaded via crates/athena-cap-study/src/lib.rs, with the model in crates/athena-cap-study/src/model.rs and validation in crates/athena-cap-study/src/rules.rs and records.rs. The model.json t10 guard text is study data, so changing it must keep the study validation tests passing and claim citations intact. No new record files under crates/ are added. The property is measured on the real rendered page HTML text, not on the source JSON.
- Existing tests in site.rs and the athena-cap-study validation tests must keep passing; any changed test expectation is shown in the diff.
- Term ladder: not applicable; 'in scope', 'places', 'home' are glossary terms of the firstmate study, not structural terms.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`. A test in xtask/src/site.rs renders the real firstmate study and asserts that no run of 6 or more normalised words from a term definition (yolo, green, red, in scope, places, home) appears anywhere else in the page text, so reworded copies are caught; it passes.
2. [behavior] Run `cargo test -p xtask`. A test in xtask/src/site.rs asserts each of yolo, green, red, in scope, places and home has exactly one definition on the rendered page and that every href="#term-..." has a matching id.
3. [content] Grep of studies/firstmate-work-handling/intro.json shows what[4] no longer contains 'persistent home of its own' style wording (only a link to home), and limits[6] no longer repeats 'a local-only ship or a scout ... keeps its place until cleanup'.
4. [content] Grep of intro.json and studies/firstmate-work-handling/model.json shows the t2 and t11 summaries, glossary exit 75 and secondmate entries and the t10 trigger use [[term]] links, and the t10 guard no longer restates the red definition ('a red PR or a required check that has not reported is never merged').
5. [content] The Glossary and Words used first still hold the definitions with claim citations; the existing citation tests pass.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass.
7. [content] `git diff` touches only xtask/src/site.rs, studies/firstmate-work-handling/intro.json, studies/firstmate-work-handling/model.json (guard text only) and CHANGELOG.md; the output path, evidence section and claims listing are unchanged.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing what the terms mean, or the claims and citations behind them
- Changes to other studies or to the evidence/decisions pages
- Changes to athena.toml, defaults.toml, CI workflows or crates/ code
- Redesigning the page layout or styling

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered page for repetition and resolving links, not just file edits.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the page text and its test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 04bfdaa625b9 → candidate 121870396b8a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   6 ++
 studies/firstmate-work-handling/intro.json |  36 +++----
 studies/firstmate-work-handling/model.json |   4 +-
 xtask/src/site.rs                          | 157 +++++++++++++++++++++++++++--
 4 files changed, 174 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-075222-nsk3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-075222-nsk3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42986091887 for 121870396b8a16fa7cff4e89f0902897f4f0283f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42986091887 for 121870396b8a16fa7cff4e89f0902897f4f0283f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42986091887 for 121870396b8a16fa7cff4e89f0902897f4f0283f)
- ci-result: `not a check` passed (Check results for 121870396b8a16fa7cff4e89f0902897f4f0283f come from the CI run https://github.com/doodla/athena/actions/runs/42986091887 (720s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
