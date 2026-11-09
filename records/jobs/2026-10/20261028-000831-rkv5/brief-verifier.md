# Job 20261028-000831-rkv5: verify "Define wake, endpoint, captain call, heartbeat, durable record; fix ship"

## Request (verbatim)

> On the firstmate study page, add glossary entries in studies/firstmate-work-handling/intro.json for 'wake', 'endpoint', 'captain call', 'heartbeat' and 'durable record'. Rewrite the 'ship' entry so it says first that a ship is a task that changes a project, instead of 'a kind of task, named next to scouts'. Each definition must rest only on existing cited claims in studies/firstmate-work-handling/claims.json. Where the firstmate source does not define a term, say so, the way the 'in scope' entry does. Do not add statements that no claim supports. Acceptance: (1) the rendered page built by `cargo run -p xtask -- site` has a glossary entry for each of those five terms, and each entry cites at least one claim or says the source does not define the term; (2) the 'ship' entry's first sentence says what a ship does; (3) a site test fails if any of those five terms appears in a transition card while the glossary has no entry for it; (4) cargo fmt --check, cargo clippy and cargo test pass. Do not restructure the page.
> 
> Acceptance:
> - The change is made as described: Newcomer ratings keep flagging undefined terms (wake, endpoint, captain call, heartbeat) and a circular 'ship' entry, including after the gap-fix landed
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: studies/firstmate-work-handling/intro.json, studies/firstmate-work-handling/claims.json.

## Reading (accepted by the user)

Add five glossary entries to the firstmate study's intro.json, each resting only on existing claims or stating the source does not define the term, and reword 'ship' to open with what a ship does. Add a site test that fails when any of those terms appears in a transition card with no glossary entry. Goal: a newcomer reading the built page finds no undefined term and no circular 'ship' entry.

Assumptions:
- Code survey: I read xtask/src/site.rs and xtask/tests/site_page.rs. The entry point is `build` -> `load` -> `render`, which uses the `Study` fields intro, model, claims, rationales and findings. `render` writes the transition cards at about lines 376-419 from model.json transitions and intro.json summaries. It writes the Glossary at about lines 458-482. The glossary is sorted by lowercased term, and each entry gets `<dt id="term-...">`. A non-page entry must pass `cites`, which errors on an empty claims array or an unknown claim id. That is the records validation here. It is not the crates/ records allow-list, because no record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly on the rendered HTML of the real study, as the existing in_scope_is_defined_in_the_start_terms_and_the_glossary_with_a_citation unit test does, not through a stand-in. xtask has no library target, so the new test goes in xtask/src unit tests or xtask/tests/site_page.rs.
- Code survey: I also read crates/athena-core/src/reading.rs only by name; no crates/ code is changed. The change adds no refusal rule to Athena's pipeline. The site test is a new check on page content. It refuses no Athena readings, so no refused-readings percentage applies.
- Term definitions come from claims already in claims.json: c17 for stale wake and endpoint, c9 for heartbeat, c19 for durable record, c23 and c24 for captain call, and c2 and c3 for endpoint. The producer must not add claims. claims.json is a deliverable only if a claim needs a wording fix, which I expect to be unnecessary.
- The five terms are searched in the transition cards' rendered text (summary, trigger, guard, failure), case-insensitively. The test's list of terms is fixed to the five named in the request.
- Where the claims show a term in use but do not define it (probably wake, heartbeat and durable record), the entry says the source does not define it and gives only what the cited claim shows, in the way the 'in scope' entry does.

## Acceptance criteria

1. [content] studies/firstmate-work-handling/intro.json has glossary entries (the array with 'in scope' and 'ship') for 'wake', 'endpoint', 'captain call', 'heartbeat' and 'durable record'. Each has a non-empty claims array of ids present in claims.json, or states in its definition that the firstmate source does not define the term, as the 'in scope' entry does. Checked by Grep and reading the diff.
2. [content] The 'ship' entry's definition begins with a sentence saying a ship is a task that changes a project. It no longer opens with 'A kind of task, named next to scouts', and it keeps only claims c10, c12 and c18 or other existing claim ids. Checked in the diff of intro.json.
3. [content] Every new or reworded definition makes only statements found in the cited claims' text in claims.json. For example, a wake is described only through c17's stale wake, a heartbeat only through c9, an endpoint through c2, c3 and c17, a durable record through c19, and a captain call through c23 and c24. Where the claims do not say what a term is, the entry says so. A verifier can compare each definition with the cited claim text.
4. [behavior] Run `cargo run -p xtask -- site`. It succeeds, and the rendered index.html has a `<dt id="term-...">` glossary entry for each of the five terms. Each entry carries a `<span class="cites">` or says the source does not define the term. The 'ship' entry's first sentence says what a ship does.
5. [behavior] A new test, in the unit tests in xtask/src/site.rs or in xtask/tests/site_page.rs, scans the rendered transition cards of the real study for each of the five terms. It fails when a term appears in a card and the glossary has no entry for it. Show that it fails: `cargo test -p xtask` with the test's own fixture where one entry is missing must produce the failure, or the test must assert on a fixture study that lacks one entry. It passes on the real study.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass. The existing site tests still pass, including the glossary alphabetical-order test, the in-scope test and the unknown-claim-citation test. No change is made to page structure or render code beyond what the new test needs.
7. [content] No governance files (athena.toml, defaults.toml, workflows, Cargo configuration) are changed, and CHANGELOG.md is not edited unless the repository's habit requires it. Checked with `git diff --stat`.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- xtask/src/site.rs (modify)
- xtask/tests/site_page.rs (modify)
- studies/firstmate-work-handling/claims.json (modify)
- CHANGELOG.md (modify)

## Out of scope

- Restructuring the page or changing the render code.
- Adding or editing claims, sources or model.json content.
- Defining terms beyond the five requested plus the 'ship' rewrite.
- Changing governance files, CI workflows or CHANGELOG.md.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the built page and a failing-case site test, not just that the files changed.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo fmt --check, clippy and test are required to pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the glossary data and one test.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every definition must cite existing claims or say the source does not define the term.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 357ef044779b → candidate 639436473223. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 studies/firstmate-work-handling/intro.json |  7 +++-
 xtask/src/site.rs                          | 64 ++++++++++++++++++++++++++++++
 2 files changed, 70 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261028-000831-rkv5/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-000831-rkv5/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42650065203 for 6394364732232189764e127e93e4c3483879ac3e)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42650065203 for 6394364732232189764e127e93e4c3483879ac3e)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42650065203 for 6394364732232189764e127e93e4c3483879ac3e)
- ci-result: `not a check` passed (Check results for 6394364732232189764e127e93e4c3483879ac3e come from the CI run https://github.com/doodla/athena/actions/runs/42650065203 (737s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass. The existing site tests still pass, including the glossary alphabetical-order test, the in-scope test and the unknown-claim-citation test. No change is made to page structure or render code beyond what the new test needs.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
