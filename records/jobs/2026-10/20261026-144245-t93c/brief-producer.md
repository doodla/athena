# Job 20261026-144245-t93c: Split site::render into per-section helper functions

## Request (verbatim)

> Structural pass: xtask/src/site.rs
> 
> Code health found xtask/src/site.rs over a limit: function render is 333 lines, over the limit of 200.
> 
> Measured: 637 production lines (test modules excluded); longest function render at 333 lines; 0 landings in the last 7 days; 13 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function render is 333 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: xtask/src/site.rs

## Reading (accepted by the user)

Break the 333-line `render` in xtask/src/site.rs into small private section functions (header, start-here, states, counters, transitions, rationales, glossary, findings, next steps, claims, footer/wrapper) so every function is under 200 lines and the file stays under its limits. The HTML output must be byte-identical; the audience is the code-health check and maintainers.

Assumptions:
- Code survey: read xtask/src/site.rs (entry point build -> load -> render; render validates claims.json/pin.json/intro.json in order, then appends sections to one `body` String in a fixed order: header, Start here, States, Counters, Transitions, Why, Glossary, Limits, Next steps, Claims, About, footer) and its callers in xtask/src/main.rs and xtask/src/decisions.rs; read crates/athena-core/src/code_health.rs as the place code health measures per-function lines (limit 200) and production lines, so the property is measured by that check on render's own line count, not through a stand-in. No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not involved, and no refusal rule is added or tightened.
- Goal: reduce render below 200 lines with no behavior change; helpers take `&mut String` body plus the needed study/known/github references and return Result<()>, preserving error order and messages.
- Single file stays: 637 production lines is well under the 2000 file limit, so no module split is required.
- No governance file (athena.toml, defaults.toml, workflows) needs to change; xtask has no library target so tests stay as unit tests in xtask/src/site.rs.
- Environment: none needed; no CI, push or history read.

## Acceptance criteria

1. [content] In xtask/src/site.rs, `pub fn render(study: &Study) -> Result<String>` keeps its signature but is no longer than 200 lines; Grep of the file shows new private `fn` helpers (one per section) called from render, and no function in the file exceeds 200 lines.
2. [content] The diff touches only xtask/src/site.rs; the `mod tests` block is not edited except possibly imports, and no test is deleted or weakened (git diff shows no removed #[test] functions).
3. [behavior] Run `cargo test -p xtask`: all existing site tests pass unchanged, including the_real_study_renders_with_an_intro_and_glossary and the_page_cites_claim_ids_and_the_pinned_commit, showing the rendered HTML behavior is unchanged.
4. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; both exit clean.
5. [content] Public items used by xtask/src/decisions.rs and main.rs (ATHENA_URL, CSS, DECISIONS_PAGE, escape, source_url, target_dir, default_out, check_out, load, build, render) keep their names, visibility and signatures.

## Expected deliverables

- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing HTML output, CSS, or error messages
- Splitting site.rs into multiple files
- Editing decisions.rs, main.rs or other files
- Adding new tests beyond what the refactor needs

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused refactor in a single file.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt required as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Existing HTML-content tests are run to show unchanged output.
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

- path: <state>/jobs/20261026-144245-t93c/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144245-t93c/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In xtask/src/site.rs, `pub fn render(study: &Study) -> Result<String>` keeps its signature but is no longer than 200 lines; Grep of the file shows new private `fn` helpers (one per section) called from render, and no function in the file exceeds 200 lines.
The diff touches only xtask/src/site.rs; the `mod tests` block is not edited except possibly imports, and no test is deleted or weakened (git diff shows no removed #[test] functions).
Run `cargo test -p xtask`: all existing site tests pass unchanged, including the_real_study_renders_with_an_intro_and_glossary and the_page_cites_claim_ids_and_the_pinned_commit, showing the rendered HTML behavior is unchanged.
Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; both exit clean.
Public items used by xtask/src/decisions.rs and main.rs (ATHENA_URL, CSS, DECISIONS_PAGE, escape, source_url, target_dir, default_out, check_out, load, build, render) keep their names, visibility and signatures.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
