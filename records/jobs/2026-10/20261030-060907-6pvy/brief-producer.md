# Job 20261030-060907-6pvy: Give OLY-B-D2-02 a mechanism: site accessibility, tracks, learning path

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-D2-02 a mechanism
> 
> Requirement OLY-B-D2-02 in docs/spec.toml is active and has no mechanism.
> Requirement: The site is first-class on mobile, reads in a core track and an expert track, and offers a learning path, dark and light modes, reduced motion and accessibility basics.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-D2-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-D2-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Extend the explainer site generator (xtask/src/site.rs) so every page has a skip link, a core/expert track switch, a learning-path list, a reduced-motion rule and focus styles, on top of the existing viewport and dark mode. Then point OLY-B-D2-02 in docs/spec.toml at that code, with a measure ending `status: not shown` and the test that shows it.

Assumptions:
- Code survey: read xtask/src/site.rs: finish() builds the page wrapper with viewport meta; the CSS const (lines ~893-911) has prefers-color-scheme dark and a max-width:40rem rule but no reduced-motion, focus, skip link, tracks or learning path; tests at ~1608 assert viewport and prefers-color-scheme. Entry point is site::build -> render -> finish. The site is in xtask, not crates/; docs/spec.toml mechanisms cite paths like crates/athena-core/src/introspect.rs. No record or data file is added, so check_record_file/AnyRecord are not involved.
- Existing code: extends xtask/src/site.rs (CSS, finish, render) and docs/design/explainer-site.md; reuses existing page tests.
- Audience: the site's readers and the verifier checking the page output; goal is a page that visibly offers the listed features. 'Dark and light modes' is satisfied by the existing prefers-color-scheme plus color-scheme; a manual toggle is optional and not required.
- The track control is CSS/HTML only (no JavaScript) to keep the static page simple; core is default and expert sections are shown via a checkbox or details element.
- No governance files (athena.toml, defaults.toml, workflows) need changing.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-D2-02 has mechanism naming xtask/src/site.rs (a file that exists), owner Athena, and a measure ending `status: not shown` that names the test, replacing mechanism = "none".
2. [content] xtask/src/site.rs page wrapper and CSS contain: a skip link to main content, a core/expert track control with expert content marked, a learning-path ordered list of sections, an `@media (prefers-reduced-motion:reduce)` rule, a `:focus-visible` style, and keep the viewport meta and prefers-color-scheme rules.
3. [behavior] A new unit test in xtask/src/site.rs renders the real study page and asserts each feature above; run `cargo test -p xtask site`. The test fails when the new markup/CSS is removed (it asserts on strings the old page lacks) and passes with it.
4. [behavior] Existing site tests still pass: `cargo test` passes.
5. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
6. [content] CHANGELOG.md has an entry for this change.

## Expected deliverables

- xtask/src/site.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)

## Out of scope

- Client-side JavaScript or a build toolchain for the site
- Changing other spec requirements
- Editing docs/decisions.md or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test, clippy, fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts rendered page output, not just wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in site.rs and spec.toml.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261030-060907-6pvy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060907-6pvy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml requirement OLY-B-D2-02 has mechanism naming xtask/src/site.rs (a file that exists), owner Athena, and a measure ending `status: not shown` that names the test, replacing mechanism = "none".
xtask/src/site.rs page wrapper and CSS contain: a skip link to main content, a core/expert track control with expert content marked, a learning-path ordered list of sections, an `@media (prefers-reduced-motion:reduce)` rule, a `:focus-visible` style, and keep the viewport meta and prefers-color-scheme rules.
A new unit test in xtask/src/site.rs renders the real study page and asserts each feature above; run `cargo test -p xtask site`. The test fails when the new markup/CSS is removed (it asserts on strings the old page lacks) and passes with it.
Existing site tests still pass: `cargo test` passes.
`cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
CHANGELOG.md has an entry for this change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
