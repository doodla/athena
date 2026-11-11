# Job 20261111-075222-nsk3: Retry: define each site term once and link elsewhere

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

## Findings you must address

The verifier of job 20261029-212839-1vav raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (studies/firstmate-work-handling/intro.json: what[4] and limits[6]): Two definitions are still repeated almost word for word, in the overview and the failure list In the overview: "A secondmate is a persistent [[home]], not a backlog item." The home Glossary entry: "A secondmate is a persistent home of its own, not an item in a backlog." In the limits: "...a local-only ship or a scout that records no PR keeps its place until cleanup." The place Glossary entry: "...a local-only ship or a scout keeps its place until cleanup...". The request says "Do not repeat the definition text in the overview ... or the failure list". The complaint behind it is repetition "almost word for word". Fix: cut the overview clause to "A secondmate is a persistent [[home]]" or drop it. End the limit after "free a [[place]]", or move the secondmate sentence out of the home definition.
> major (xtask/src/site.rs each_term_is_defined_once_and_every_term_link_resolves): Soundness: the test only catches exact copies of a definition sentence, so a lightly reworded copy passes It runs page.matches(sentence).count() == 1 on each whole sentence split on ". ". Both copies in the finding above pass it because a few words differ ("of its own", "an item in a backlog" vs "a backlog item"). The request is about near-verbatim repetition. A more correct check flags any long word run a definition shares with prose outside it (for example 6+ normalised words), or uses a token-overlap ratio per sentence.

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

- path: <state>/jobs/20261111-075222-nsk3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-075222-nsk3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p xtask`. A test in xtask/src/site.rs renders the real firstmate study and asserts that no run of 6 or more normalised words from a term definition (yolo, green, red, in scope, places, home) appears anywhere else in the page text, so reworded copies are caught; it passes.
Run `cargo test -p xtask`. A test in xtask/src/site.rs asserts each of yolo, green, red, in scope, places and home has exactly one definition on the rendered page and that every href="#term-..." has a matching id.
Grep of studies/firstmate-work-handling/intro.json shows what[4] no longer contains 'persistent home of its own' style wording (only a link to home), and limits[6] no longer repeats 'a local-only ship or a scout ... keeps its place until cleanup'.
Grep of intro.json and studies/firstmate-work-handling/model.json shows the t2 and t11 summaries, glossary exit 75 and secondmate entries and the t10 trigger use [[term]] links, and the t10 guard no longer restates the red definition ('a red PR or a required check that has not reported is never merged').
The Glossary and Words used first still hold the definitions with claim citations; the existing citation tests pass.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass.
`git diff` touches only xtask/src/site.rs, studies/firstmate-work-handling/intro.json, studies/firstmate-work-handling/model.json (guard text only) and CHANGELOG.md; the output path, evidence section and claims listing are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
