# Job 20261109-161516-a834: verify "Define five glossary terms, fix 'ship', add test and changelog entry"

## Request (verbatim)

> try again on 20261028-000831-rkv5

## Reading (accepted by the user)

Retry of job 20261028-000831-rkv5: add glossary entries for wake, endpoint, captain call, heartbeat and durable record to the firstmate study intro.json, reword 'ship' to open with what a ship does, add a site test that fails when a card term has no entry, and now add the CHANGELOG.md entry whose absence failed the last verdict. Audience: a newcomer reading the built firstmate page.

Assumptions:
- Code survey: I read xtask/src/site.rs, where the glossary is rendered at about lines 458-470 (entries read from intro.json 'glossary', sorted by term, claims cited by id, unknown claim id errors), with unit tests around lines 729-1015 (in-scope, alphabetical order, real study). I also read crates/athena-data/src/records.rs (check_record_file at line 1375, AnyRecord at line 1049). The change adds no record or data file under crates/, so the records allow-list and its round-trip test are not touched; studies/ files are study data, not records. The entry point is `cargo run -p xtask -- site`, which loads the study, renders the transition cards, then the glossary. The property, that no card term lacks an entry, is measured directly on the rendered HTML: the test extracts card text and the glossary <dt> terms and compares them, not via a stand-in.
- Job 20261028-000831-rkv5's candidate changed intro.json and xtask/src/site.rs. Both are kept as the base for the retry and corrected. Its out_of_scope item forbidding CHANGELOG.md is dropped, because the changelog default and repository habit require an entry.
- xtask/tests/site_page.rs and claims.json are listed as possible deliverables but expected unchanged; the new test goes in xtask/src/site.rs unit tests as before.
- Out of scope items: restructuring the page or render code; adding or editing claims, sources or model.json; defining other terms; governance files.
- Goal and audience: a newcomer reading the built firstmate page finds no undefined term and no circular 'ship' entry.

## Acceptance criteria

1. [content] studies/firstmate-work-handling/intro.json glossary has entries for 'wake', 'endpoint', 'captain call', 'heartbeat' and 'durable record'. Each has a non-empty claims array of ids that exist in claims.json, or says the firstmate source does not define the term, as 'in scope' does. Verifier compares each definition with the cited claim text in the diff.
2. [content] The 'ship' entry's definition begins with 'A task that changes a project' and no longer opens with 'A kind of task, named next to scouts'. Its claims include c33 (PR-based ship frees its place when its ready PR is recorded) if it says work is delivered as a PR, and every claim id exists in claims.json. Checked in the diff of intro.json (fixes job 20261028-000831-rkv5's minor finding on c37).
3. [content] CHANGELOG.md has a new entry recording the glossary change for the firstmate study, in the style of the earlier firstmate glossary entries. This is required, not conditional (fixes job 20261028-000831-rkv5's major finding). Checked with git diff --stat.
4. [behavior] Run `cargo run -p xtask -- site`: it succeeds and the rendered firstmate index.html has a <dt id="term-..."> entry for each of the five terms, each with a cites span or a statement that the source does not define it. The ship entry's <dd> starts with its what-a-ship-does sentence.
5. [behavior] A new test in xtask/src/site.rs (or xtask/tests/site_page.rs) fails when any of the five terms appears in a transition card with no glossary entry, matching whole words so 'awake' does not trigger 'wake' (fixes the substring minor finding). It asserts on a fixture lacking entries to show it fails, and passes on the real study. The ship test pins the opening to the <dd> start with starts_with (fixes the nit). Run `cargo test -p xtask`.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and the full `cargo test` pass, including existing glossary alphabetical-order, in-scope and unknown-claim-citation tests. No render code or page structure changes beyond the new test.
7. [content] No governance file (athena.toml, defaults.toml, workflows, Cargo configuration) changes; claims.json, sources and model.json are unchanged. Checked with git diff --stat.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- xtask/tests/site_page.rs (modify)
- studies/firstmate-work-handling/claims.json (modify)

## Out of scope

- Restructuring the page or changing the render code.
- Adding or editing claims, sources or model.json content.
- Defining terms beyond the five requested plus the 'ship' rewrite.
- Changing governance files (athena.toml, defaults.toml, workflows, Cargo configuration).

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered page and a test that fails on a missing term, not just file edits.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and full cargo test must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each definition rests on cited claims or states the source does not define the term.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to glossary data, one test, one changelog entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ad40bdcbb18e → candidate fc1ab547e5d3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  6 +++
 studies/firstmate-work-handling/intro.json |  7 ++-
 xtask/src/site.rs                          | 82 ++++++++++++++++++++++++++++++
 3 files changed, 94 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-161516-a834/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-161516-a834/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42653328517 for fc1ab547e5d31fa62ed7f61dfc58171610fd8513)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42653328517 for fc1ab547e5d31fa62ed7f61dfc58171610fd8513)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42653328517 for fc1ab547e5d31fa62ed7f61dfc58171610fd8513)
- ci-result: `not a check` passed (Check results for fc1ab547e5d31fa62ed7f61dfc58171610fd8513 come from the CI run https://github.com/doodla/athena/actions/runs/42653328517 (604s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and the full `cargo test` pass, including existing glossary alphabetical-order, in-scope and unknown-claim-citation tests. No render code or page structure changes beyond the new test.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
