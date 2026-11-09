# Job 20261109-161516-a834: Define five glossary terms, fix 'ship', add test and changelog entry

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

## Retry: your work starts from the rejected candidate of job 20261028-000831-rkv5

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> No governance files (athena.toml, defaults.toml, workflows, Cargo configuration) are changed, and CHANGELOG.md is not edited unless the repository's habit requires it. Checked with `git diff --stat`.: Only intro.json and xtask/src/site.rs changed, so no governance file was touched. But the repository's habit does require a changelog entry for this kind of change. CHANGELOG.md [Unreleased] records earlier glossary edits to this same study (line 3099: 'the Glossary gains green, red and home'; line 3438: 'Glossary: `exit 1` is added ... `ship` now says ...'). CHANGELOG.md is also listed as an expected deliverable, and the changelog default applies. The candidate adds no entry.

## Findings you must address

The verifier of job 20261028-000831-rkv5 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (CHANGELOG.md): No CHANGELOG.md entry, although the changelog default applies, the expected deliverables list it, and the repository's habit is to record glossary changes to this study. git diff HEAD~1 --stat shows only intro.json and xtask/src/site.rs. CHANGELOG.md lines 3099-3104 and 3438-3442 record earlier glossary changes to the firstmate page. The brief's 'changelog' default says 'the change records itself in CHANGELOG.md'. The brief contradicts itself: its out-of-scope list names CHANGELOG.md, but criterion 7 allows the edit when the habit requires it, and here the habit does.

All of the verification's findings:

> major (CHANGELOG.md): No CHANGELOG.md entry, although the changelog default applies, the expected deliverables list it, and the repository's habit is to record glossary changes to this study. git diff HEAD~1 --stat shows only intro.json and xtask/src/site.rs. CHANGELOG.md lines 3099-3104 and 3438-3442 record earlier glossary changes to the firstmate page. The brief's 'changelog' default says 'the change records itself in CHANGELOG.md'. The brief contradicts itself: its out-of-scope list names CHANGELOG.md, but criterion 7 allows the edit when the habit requires it, and here the habit does.
> minor (studies/firstmate-work-handling/intro.json:105): In the ship entry, 'its work is delivered as a PR' is only implied by the cited claims; the claim that states it is not cited. c37 says 'A local-only ship has no PR'. That implies other ships make PRs but does not say so. c33 ('a PR-based ship frees its place when its ready PR is recorded') states it more directly and is not in the entry's claims.
> minor (xtask/src/site.rs card_terms_without_a_glossary_entry): The card-term check matches plain substrings, so it can fire on longer words that contain a term (for example 'awake' contains 'wake'). It uses `cards.contains(t)` on lowercased HTML. With five fixed terms and current data there are no false hits. A word-boundary match would be more exact.
> nit (xtask/src/site.rs the_ship_entry_opens_by_saying_what_a_ship_is): The ship test checks that the text follows somewhere after the ship <dt>, not that the <dd> starts with it. It uses `html[at..].contains("<dd>A task that changes a project")`. A starts_with on the slice that begins at the <dd> would pin it to the ship entry.

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

The work starts from the rejected candidate of job 20261028-000831-rkv5: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-161516-a834/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-161516-a834/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

studies/firstmate-work-handling/intro.json glossary has entries for 'wake', 'endpoint', 'captain call', 'heartbeat' and 'durable record'. Each has a non-empty claims array of ids that exist in claims.json, or says the firstmate source does not define the term, as 'in scope' does. Verifier compares each definition with the cited claim text in the diff.
The 'ship' entry's definition begins with 'A task that changes a project' and no longer opens with 'A kind of task, named next to scouts'. Its claims include c33 (PR-based ship frees its place when its ready PR is recorded) if it says work is delivered as a PR, and every claim id exists in claims.json. Checked in the diff of intro.json (fixes job 20261028-000831-rkv5's minor finding on c37).
CHANGELOG.md has a new entry recording the glossary change for the firstmate study, in the style of the earlier firstmate glossary entries. This is required, not conditional (fixes job 20261028-000831-rkv5's major finding). Checked with git diff --stat.
Run `cargo run -p xtask -- site`: it succeeds and the rendered firstmate index.html has a <dt id="term-..."> entry for each of the five terms, each with a cites span or a statement that the source does not define it. The ship entry's <dd> starts with its what-a-ship-does sentence.
A new test in xtask/src/site.rs (or xtask/tests/site_page.rs) fails when any of the five terms appears in a transition card with no glossary entry, matching whole words so 'awake' does not trigger 'wake' (fixes the substring minor finding). It asserts on a fixture lacking entries to show it fails, and passes on the real study. The ship test pins the opening to the <dd> start with starts_with (fixes the nit). Run `cargo test -p xtask`.
`cargo fmt --check`, `cargo clippy --all-targets` and the full `cargo test` pass, including existing glossary alphabetical-order, in-scope and unknown-claim-citation tests. No render code or page structure changes beyond the new test.
No governance file (athena.toml, defaults.toml, workflows, Cargo configuration) changes; claims.json, sources and model.json are unchanged. Checked with git diff --stat.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
