# Job 20261029-212839-1vav: Define each site term once and link to it elsewhere

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-212839-1vav/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-212839-1vav/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p xtask`. A new test in xtask/src/site.rs renders the real study and splits the page text into sentences. It asserts that no definition sentence for yolo, green, red, in scope, places or home appears more than once, and it passes.
The same test, or a sibling in xtask/src/site.rs, asserts that each of the terms yolo, green, red, in scope, places and home has exactly one definition on the page. It also asserts that every link to a term (href="#term-...") has a matching id on the page.
`git diff` shows that studies/firstmate-work-handling/intro.json (and any site.rs template text) no longer restates the definitions in the overview 'what' list, the walkthrough steps, the t4 and t10 summaries or the limits (failure) list. Those places link to the term instead, for example 'yolo' becomes a link to its definition.
The Glossary and 'Words used first' sections still contain the definitions, with their claim citations kept. The rendered page still cites claims for every factual statement, so the existing citation tests pass.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All pass, including the existing site.rs tests and the athena-cap-study study validation tests. Any existing test changed carries a diff showing that it now expects the single definition.
No unrelated behavior changes. The diff touches only xtask/src/site.rs and the study's intro.json (and CHANGELOG.md if the repository's convention requires an entry); the build output path, evidence section and claims listing are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
