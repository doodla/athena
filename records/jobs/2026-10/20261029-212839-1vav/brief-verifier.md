# Job 20261029-212839-1vav: verify "Define each site term once and link to it elsewhere"

## Request (verbatim)

> In the site that xtask/src/site.rs generates, define each term (yolo, green/red, in scope, places, home) in exactly one place, the Glossary or Words used first, and link to that definition everywhere else. Do not repeat the definition text in the overview, the walkthrough steps, the t4/t10 cards or the failure list. Acceptance: a test on the rendered page shows that no definition sentence appears more than once; every term still resolves to one definition; fmt, clippy and the existing tests pass.
> 
> Acceptance:
> - The change is made as described: Five newcomer ratings say the homepage repeats the same definitions almost word for word; no job has been asked to fix it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: xtask/src/site.rs, green/red, t4/t10

## Reading (accepted by the user)

The rendered firstmate study page restates the yolo, green/red, in scope, places and home definitions in the overview, walkthrough, t4/t10 cards and failure list. The change keeps one definition per term (the Glossary, or Words used first) and links to it from every other place. A test on the rendered page checks that no definition sentence appears twice and that every term resolves to one definition. The audience is newcomers reading the homepage.

Assumptions:
- Code survey: read xtask/src/site.rs, where render() calls start_here (what_is, 'Words used first' from intro start_terms, what, walkthrough, limits), states, counters, transitions (deciders and t-cards from intro summaries), rationales, glossary (each dt gets id term-<term with - for spaces>), findings, next_steps and claims_section, in that order. I also read crates/athena-cap-study/src/lib.rs and rules.rs. A study job may only change one studies/<id>/ directory, and rules.rs validates studies/ trees, citations and conventions. That validation is not a records allow-list, and this change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test do not apply. The definition text lives in studies/firstmate-work-handling/intro.json: the summaries t4 and t10, walkthrough, limits, what, start_terms and glossary all carry copies. The property is measured directly on the output of render(&real_study()), by splitting the page into sentences, not by checking JSON.
- Existing tests in xtask/src/site.rs must keep passing. They include in_scope_is_defined_in_the_start_terms_and_the_glossary_with_a_citation, the_real_page_titles_each_design_choice_and_defines_the_start_here_terms, the card-term glossary tests, and the one that puts the 'Words used first' h3 after the what-is sentence. The first two may need updating where they require in scope to be defined in two places. That is an intended consequence of the request, not an unrelated behavior change.
- Which place holds each definition: 'Words used first' for yolo, green, red and in scope; the Glossary for places-held and home. In-text mentions elsewhere become links to #term-<slug> anchors, using the existing glossary anchors. Where a term is defined in Words used first, the same links need an anchor on that list's dt, or the Glossary entry stays the single definition and the start list links to it.
- Term resolution: every term mentioned in prose links to exactly one definition anchor that exists on the page.
- Test placement: xtask has no library target, so the new test goes in the unit tests module in xtask/src/site.rs.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`. A new test in xtask/src/site.rs renders the real study and splits the page text into sentences. It asserts that no definition sentence for yolo, green, red, in scope, places or home appears more than once, and it passes.
2. [behavior] The same test, or a sibling in xtask/src/site.rs, asserts that each of the terms yolo, green, red, in scope, places and home has exactly one definition on the page. It also asserts that every link to a term (href="#term-...") has a matching id on the page.
3. [content] `git diff` shows that studies/firstmate-work-handling/intro.json (and any site.rs template text) no longer restates the definitions in the overview 'what' list, the walkthrough steps, the t4 and t10 summaries or the limits (failure) list. Those places link to the term instead, for example 'yolo' becomes a link to its definition.
4. [content] The Glossary and 'Words used first' sections still contain the definitions, with their claim citations kept. The rendered page still cites claims for every factual statement, so the existing citation tests pass.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All pass, including the existing site.rs tests and the athena-cap-study study validation tests. Any existing test changed carries a diff showing that it now expects the single definition.
6. [content] No unrelated behavior changes. The diff touches only xtask/src/site.rs and the study's intro.json (and CHANGELOG.md if the repository's convention requires an entry); the build output path, evidence section and claims listing are unchanged.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing what the terms mean, or the claims and citations behind them
- Changes to other studies or to the evidence/decisions pages
- Changes to athena.toml, defaults.toml, CI workflows or crates/ code
- Redesigning the page layout or styling

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt to pass, plus a new test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test checks the rendered page, not the JSON or wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to site.rs and the study's intro text.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that no definition sentence repeats and every term resolves.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9632c2bbe013 → candidate 8a8cc8386923. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   4 +
 studies/firstmate-work-handling/intro.json |  44 ++++-----
 xtask/src/site.rs                          | 143 +++++++++++++++++++++++++----
 3 files changed, 150 insertions(+), 41 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-212839-1vav/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-212839-1vav/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42980553085 for 8a8cc8386923f5b4f15e37366d7fa65fc2140930)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42980553085 for 8a8cc8386923f5b4f15e37366d7fa65fc2140930)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42980553085 for 8a8cc8386923f5b4f15e37366d7fa65fc2140930)
- ci-result: `not a check` passed (Check results for 8a8cc8386923f5b4f15e37366d7fa65fc2140930 come from the CI run https://github.com/doodla/athena/actions/runs/42980553085 (708s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All pass, including the existing site.rs tests and the athena-cap-study study validation tests. Any existing test changed carries a diff showing that it now expects the single definition.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
