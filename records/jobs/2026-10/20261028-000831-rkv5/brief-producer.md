# Job 20261028-000831-rkv5: Define wake, endpoint, captain call, heartbeat, durable record; fix ship

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261028-000831-rkv5/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-000831-rkv5/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

studies/firstmate-work-handling/intro.json has glossary entries (the array with 'in scope' and 'ship') for 'wake', 'endpoint', 'captain call', 'heartbeat' and 'durable record'. Each has a non-empty claims array of ids present in claims.json, or states in its definition that the firstmate source does not define the term, as the 'in scope' entry does. Checked by Grep and reading the diff.
The 'ship' entry's definition begins with a sentence saying a ship is a task that changes a project. It no longer opens with 'A kind of task, named next to scouts', and it keeps only claims c10, c12 and c18 or other existing claim ids. Checked in the diff of intro.json.
Every new or reworded definition makes only statements found in the cited claims' text in claims.json. For example, a wake is described only through c17's stale wake, a heartbeat only through c9, an endpoint through c2, c3 and c17, a durable record through c19, and a captain call through c23 and c24. Where the claims do not say what a term is, the entry says so. A verifier can compare each definition with the cited claim text.
Run `cargo run -p xtask -- site`. It succeeds, and the rendered index.html has a `<dt id="term-...">` glossary entry for each of the five terms. Each entry carries a `<span class="cites">` or says the source does not define the term. The 'ship' entry's first sentence says what a ship does.
A new test, in the unit tests in xtask/src/site.rs or in xtask/tests/site_page.rs, scans the rendered transition cards of the real study for each of the five terms. It fails when a term appears in a card and the glossary has no entry for it. Show that it fails: `cargo test -p xtask` with the test's own fixture where one entry is missing must produce the failure, or the test must assert on a fixture study that lacks one entry. It passes on the real study.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass. The existing site tests still pass, including the glossary alphabetical-order test, the in-scope test and the unknown-claim-citation test. No change is made to page structure or render code beyond what the new test needs.
No governance files (athena.toml, defaults.toml, workflows, Cargo configuration) are changed, and CHANGELOG.md is not edited unless the repository's habit requires it. Checked with `git diff --stat`.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
