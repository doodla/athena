# Job 20261114-045636-95ed: Give design-job producers the existing-code inventory in their brief

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Design jobs have no step that makes the producer inventory existing code, modules and fakes for the
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: design-ignores-existing-code-caught-only-at-final-verdict (20261026-061735-8rvt)
> Cause: Design jobs have no step that makes the producer inventory existing code, modules and fakes for the area before it proposes a build plan. Nothing compares the doc's proposed components against the repo, so a doc written from the request tex...
> Seen 13 time(s) in 13 jobs. The introspections that record it:
> Introspection 20261026-061725-z6pk (job 20261026-043137-zc0v); prevention it proposed: For design-type jobs, add a required "existing state" step before the producer drafts. The producer lists the existing modules, clients, fakes and tests for the area, with file paths. The design doc must carry an "Existing code" section, an...
> Introspection 20261026-065135-7wqy (job 20261026-065046-0c1d); prevention it proposed: Add a step before a design reading is accepted. It extracts the named components, endpoints and modules from the reading and searches the snapshot for existing code and doc sections that cover them. For each one it finds, it requires the re...
> Introspection 20261029-135005-vb9w (job 20261029-073414-yg89); prevention it proposed: Add a required premise-check stage to design jobs, before the producer writes and again before the verdict. The producer must list every existing component, tool, config and resource the design depends on (caches, code modules, limits). For...
> Introspection 20261104-123908-zkkv (job 20261029-160024-80qj); prevention it proposed: Add a required "current state" step to design-doc jobs. Before proposing a mechanism, the planner or producer must list the existing code paths and results that already serve the capability. For a speed design, it must also give measured st...
> Introspection 20261104-214928-6mdq (job 20261104-192155-3b3e); prevention it proposed: Add a mandatory "current state" step to every design job, enforced in the plan and by a code check on the doc. The doc must contain a section "Current behaviour" that cites the existing code (file and function or type names, for example Lan...
> Introspection 20261105-060952-5713 (job 20261104-123932-23d3); prevention it proposed: Add a required "current state" step to every design job that changes an existing flow. Before drafting, the producer must write an inventory: the existing code, workflow and config for the area, each cited by file and function or step. It m...
> Introspection 20261105-172007-jsan (job 20261105-170035-1704); prevention it proposed: Add two things to design and measurement jobs. First, a required inventory step: before writing any sample or population claim, the producer queries the job records and state directory, then records counts by category (complete jobs, reques...
> Introspection 20261108-002646-hsvn (job 20261107-044655-wwcf); prevention it proposed: For jobs that diagnose a regression, split the job in two. First, a measuring step produces a table from the recorded per-job stage timings: median producer time by date, and the change points. The table is committed. Second, the cause and ...
> Introspection 20261109-020700-1s96 (job 20261109-014709-kxrd); prevention it proposed: Add a required premise-check step to design jobs that justify a change by a measured problem. Before the reading is accepted, the reader must open each source the request cites and the job records behind the headline number. It must state i...
> Introspection 20261109-061451-zg6h (job 20261109-055645-r796); prevention it proposed: Add a required premise-check step for design jobs. Before drafting, the producer lists each headline number in the request and states its source: a record query, or the cited note. It also states whether that source flags the number as an a...
> Introspection 20261109-203545-vn3k (job 20261109-201651-ggxj); prevention it proposed: Add a required survey-and-check step to every design job before the producer writes. (a) Trace the proposed flow step by step against the code that enforces it, here the landing gate's rule about which commit may land and what records it mu...
> Introspection 20261110-193951-hbcg (job 20261110-191311-r241); prevention it proposed: Add a required survey step to every design job before the note is drafted. The producer writes an Existing code inventory: for each proposed change, the file and function that does it today, or a statement that none exists. For each rule at...
> Introspection 20261111-061031-x8mv (job 20261111-055238-5har); prevention it proposed: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such cl...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader refuses a design reading without an `Existing code:` assumption (check_design_existing_code), but the producer brief never tells the design producer to use that inventory. The doc is written from the request text and the mismatch shows only at the final verdict. Fix: add a producer-brief section, for design readings, that lists the `Existing code:` paths and requires an "Existing code" section in the doc that reuses or extends them, with a test.

Assumptions:
- Goal and audience: design-job producers get the inventory of existing code in their brief, so the designs Sri and the verifier judge match the repository. The fix gives information to the stage that lacked it and adds no refusal rule, so no refusal share applies.
- Code survey: crates/athena-context/src/briefs.rs producer_brief_with_evidence (line 681) builds the producer brief from reading_sections, and its tests sit in the same file (for example producer_briefs_list_the_boundary_words_before_the_checks). crates/athena-core/src/reading/surveys.rs check_design_existing_code (line 1409) and existing_code_assumptions (line 1328) already require the reader to supply `Existing code:` assumptions for design readings. The records validation allow-list and AnyRecord are untouched because no record or data file is added. The property is measured directly: the text of producer_brief for a design reading with and without an `Existing code:` assumption.
- Existing code: crates/athena-context/src/briefs.rs producer_brief_with_evidence and its section helpers (checkable_evidence_section, existing_tests_section); crates/athena-core/src/reading/surveys.rs check_design_existing_code and existing_code_assumptions; crates/athena-context/src/briefs/reader.rs rule line for check_design_existing_code. The change extends these and rebuilds none of them.
- Job 20261026-043137-zc0v (rejected: the design ignored the existing Jev client, FakeJev and labeled.rs): addressed at the cause. A design producer now sees the listed existing code and must carry an Existing code section. Its docs/design/jev-code-health.md, docs/decisions.md and CHANGELOG.md changes were not landed and are not redone here, so they are out of scope (dropped).
- Job 20261026-065046-0c1d (landed, verdict pass, 2 minor findings): the later retry already fixed its predecessor's findings. Its candidate files docs/design/jev-code-health.md, docs/decisions.md and CHANGELOG.md are kept as they are, and the two minor findings are doc wording that this change does not touch.
- Job 20261029-073414-yg89 (rejected: sccache treated as active although D84 removed it): addressed at the cause. The new brief section tells the producer to check each dependency, such as a cache or config, against the repository before relying on it. Its docs/design/async-drain.md and its decisions.md and CHANGELOG.md changes were not landed and are out of scope (dropped); they can be retried as a separate job.
- Field meaning: no field, CLI argument, record key or config key is reused or added. The producer brief gains one text section.
- Environment: none. The change reads and writes only files in the repository snapshot; no CI workflow, push or records access is involved.
- Verifier commands available are cargo test, clippy and fmt --check, so the behavior criteria are unit tests run by cargo test.

## Acceptance criteria

1. [content] Root cause named in the doc comment of the new section function in crates/athena-context/src/briefs.rs: a design producer was never given the inventory of existing code that the reader already requires, and nothing compared the doc's components with the repository.
2. [content] crates/athena-context/src/briefs.rs adds a producer-brief section, written only when the reading's class_of_work is design, that quotes each `Existing code:` assumption of the reading. It tells the producer to open those paths, to put an "Existing code" section in the doc that names each reused or extended path, and to state for each component the design proposes whether it exists in the repository. It also says to check any cache, config or limit the design depends on against the repository.
3. [behavior] Run cargo test -p athena-context: a new unit test builds producer_brief for a design reading with an `Existing code:` assumption naming crates/athena-core/src/jev.rs. It asserts the brief contains that path, the heading Existing code and the instruction to reuse or extend. The test fails on the base commit and passes with the change.
4. [behavior] Run cargo test -p athena-context: a new unit test builds producer_brief for a code reading and for a docs reading and asserts the new section is absent, so other classes' briefs are unchanged. Existing brief tests still pass.
5. [behavior] The recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test).
6. [content] CHANGELOG.md has an entry for this change, and the new text is added as a Rust section in briefs.rs, not as a prompt file or a refusal rule.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Any new refusal rule or change to check_design_existing_code
- docs/design/jev-code-health.md, docs/design/async-drain.md and the decisions.md and CHANGELOG.md changes of jobs 20261026-043137-zc0v and 20261029-073414-yg89 (not redone; the first landed doc of 20261026-065046-0c1d is kept as is)
- athena.toml, defaults.toml, workflows and records/
- Changes to the reader, verifier or critic prompts

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the producer brief, where the missing information originates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run producer_brief and check the text.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in one file.
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

36 more qualifying classes are left out.

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

- path: <state>/jobs/20261114-045636-95ed/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045636-95ed/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Root cause named in the doc comment of the new section function in crates/athena-context/src/briefs.rs: a design producer was never given the inventory of existing code that the reader already requires, and nothing compared the doc's components with the repository.
crates/athena-context/src/briefs.rs adds a producer-brief section, written only when the reading's class_of_work is design, that quotes each `Existing code:` assumption of the reading. It tells the producer to open those paths, to put an "Existing code" section in the doc that names each reused or extended path, and to state for each component the design proposes whether it exists in the repository. It also says to check any cache, config or limit the design depends on against the repository.
Run cargo test -p athena-context: a new unit test builds producer_brief for a design reading with an `Existing code:` assumption naming crates/athena-core/src/jev.rs. It asserts the brief contains that path, the heading Existing code and the instruction to reuse or extend. The test fails on the base commit and passes with the change.
Run cargo test -p athena-context: a new unit test builds producer_brief for a code reading and for a docs reading and asserts the new section is absent, so other classes' briefs are unchanged. Existing brief tests still pass.
The recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test).
CHANGELOG.md has an entry for this change, and the new text is added as a Rust section in briefs.rs, not as a prompt file or a refusal rule.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
