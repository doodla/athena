# Job 20261114-045722-vbq4: Tell the producer to check each summarized term against its sources

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 7 time(s) in 7 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> Introspection 20261102-102715-564y (job 20261031-172040-kq9v); prevention it proposed: Add a code check, run in the repo tests and before verification, that computes the athena binary's real input set (every file the build compiles in, including everything xtask input_files should list) and fails if the hashFiles globs in eve...
> Introspection 20261106-030930-hb84 (job 20261106-002344-jn5t); prevention it proposed: Add a fast code check that runs in the producer's loop, and again before the verifier. It fails when any test file in the repo is missing from xtask/shard-timings.txt, and its message names the file and the line to add. Also keep the real f...
> Introspection 20261106-074349-rag3 (job 20261106-062207-cbcc); prevention it proposed: Add a check in the decisions test and in the producer's local checks. It compares docs/decisions.md with its base version and fails unless the base is an exact byte prefix of the new file, so every change is added text after the last old li...
> Introspection 20261106-192737-fqkn (job 20261106-191125-zy9w); prevention it proposed: Add a pre-verdict check that runs after the producer finishes. For each behavior criterion it requires a recorded command and exit status, and it re-runs the named test commands itself. If a named test is missing or not run, the producer ge...
> Introspection 20261110-002842-axg3 (job 20261109-231857-tpgp); prevention it proposed: Run the study's drain source check (the same check the criterion names) automatically after the producer edits claims.json, and feed any claim it still reports back to the producer as a failing item to fix before the job goes to the verifie...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer brief never says that content summarizing named sources (an ontology term, a decision, a fact) must be checked against every source for each term's senses, so a wrong definition shows up only at the final verdict (jobs 20261017-173508-0wtc, 20261025-214509-y179, 20261031-172040-kq9v). The fix adds a "Check content against its sources" section to the producer brief with a test. It adds no refusal rule.

Assumptions:
- Goal and audience: producers get the missing information in their brief; the verifier and owner judge the result by whether a seeded definition or claim survives a search of its sources.
- Code survey: read crates/athena-context/src/briefs.rs (producer_brief_with_evidence at line 678, which assembles the sections; checks_before_verifier_section; SYSTEM_FACTS) and crates/athena-context/src/briefs/boundary.rs (mirror_check_section and MIRROR_CHECK_INSTRUCTION at line 87, the existing pattern of a short producer-brief section with a constant). The new section follows that pattern, is placed after mirror_check_section, and is tested in the briefs tests beside producer_briefs_tell_the_producer_to_run_the_mirror_check. It adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured on the rendered brief text from producer_brief, not through a stand-in.
- Existing code: extends crates/athena-context/src/briefs.rs and crates/athena-context/src/briefs/boundary.rs; docs/system-facts.md is only referenced, and docs/ontology.toml and crates/athena-cli/tests/mirrors.rs are named in the new text if present.
- Information-first: the missing information goes to the producer brief. No refusal rule is added, so no refused-share estimate applies.
- Job 20261017-173508-0wtc (ontology, rejected): its finding was a Hermes term with one sense when BRIEF.md also uses Hermes as the Orchestration role. This is addressed at the cause by the new brief section, which tells the producer to search each term in the governing sources and list every sense. Its files (docs/ontology.toml, crates/athena-cli/tests/ontology.rs, docs/decisions.md and CHANGELOG.md edits) are dropped, not re-applied; the ontology now exists in the repository.
- Job 20261025-214509-y179 (test speed target, rejected): its finding was a stated goal not met and per-binary causes inferred, not traced. The new section tells the producer to check each claim against its source or measurement and to say what it did not measure. Its files (secrets.rs, checks.rs, decisions.md and CHANGELOG.md edits) are dropped, not re-applied.
- Job 20261031-172040-kq9v (CI cache, rejected): its finding was a cache key that left out files the binary compiles in, against BUILD_PATHS in crates/athena-core/src/status.rs. The new section tells the producer to compare a hand-listed set with the source that defines it. Its files (ci.yml, xtask/src/main.rs, xtask/tests/policy.rs and CHANGELOG.md edits) are dropped, not re-applied.
- Environment: none. No CI workflow, push, PR or records read is needed; the change is local code and tests.
- Prompt token budgets are not affected: the text is Rust constant text in briefs, not a prompt file.

## Acceptance criteria

1. [content] Root cause is named in the new docs/decisions.md entry, appended after the last line with a Provenance bullet and a CHANGELOG.md entry: the producer stage lacked an instruction to check content that summarizes named sources against those sources, for every sense of a term.
2. [content] crates/athena-context/src/briefs/boundary.rs (or briefs.rs) holds a new section constant telling the producer to search each governing source for every term, claim or list it writes, list every sense found, compare any hand-kept list with the file that defines it, and state what it did not check; producer_brief_with_evidence includes it.
3. [behavior] Run `cargo test -p athena-context`. The new brief test passes: producer_brief's output contains the new section for a fresh job and for a retry. The test fails when the section is removed from producer_brief_with_evidence.
4. [behavior] Run `cargo test -p athena-cli`. Existing tests that read the producer brief still pass.
5. [behavior] Run `cargo fmt --check`. Run `cargo clippy --all-targets -- -D warnings`. Run `cargo test`. All three pass; a passing Athena-recorded check on the final tree counts.
6. [content] The diff adds no refusal or gating rule: no change to check, gate, admission or landing code, and none to athena.toml, defaults.toml or workflows.

## Expected deliverables

- crates/athena-context/src/briefs/boundary.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Any refusal rule or gate that rejects work for a contradicted term
- Re-applying the dropped candidates of 20261017-173508-0wtc, 20261025-214509-y179 and 20261031-172040-kq9v: docs/ontology.toml, crates/athena-cli/tests/ontology.rs, secrets.rs, checks.rs, ci.yml, xtask/src/main.rs, xtask/tests/policy.rs
- Changing athena.toml, defaults.toml, workflows or tool pins
- Changing the verifier or reader prompt files, which are at their token limits
- Editing or removing existing docs/decisions.md entries or anything under records/

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that the rendered producer brief carries the instruction, with a test that fails without it.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new brief test, with fmt, clippy and test required to pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the producer brief, where the missing information originates, not in one job's files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The brief text is enforced by a test on the rendered brief; the producer's compliance is prose-only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Time budget

Each producer run, the first and any send-back, is stopped after 86400 seconds of wall-clock time. A stopped run sends no report, so what is on disk when it stops must already be a usable change.

Pace the work by its order. Do the smallest change that meets every acceptance criterion first, one file at a time, and leave extras for afterwards; report as soon as the criteria are met. Leave no half-edited file and no broken build between steps. Use targeted tests while you work (`cargo test -p <crate> <filter>`). The full suite runs at most once, and Athena runs it again on your result (see "Checks before the verifier" below), so a second full run only spends this limit.

You have no clock of your own. If the commands Bash may run (below) include `date`, note the time when you start and read it between steps; without it, count steps against the order above.

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

- path: <state>/jobs/20261114-045722-vbq4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045722-vbq4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Root cause is named in the new docs/decisions.md entry, appended after the last line with a Provenance bullet and a CHANGELOG.md entry: the producer stage lacked an instruction to check content that summarizes named sources against those sources, for every sense of a term.
crates/athena-context/src/briefs/boundary.rs (or briefs.rs) holds a new section constant telling the producer to search each governing source for every term, claim or list it writes, list every sense found, compare any hand-kept list with the file that defines it, and state what it did not check; producer_brief_with_evidence includes it.
Run `cargo test -p athena-context`. The new brief test passes: producer_brief's output contains the new section for a fresh job and for a retry. The test fails when the section is removed from producer_brief_with_evidence.
Run `cargo test -p athena-cli`. Existing tests that read the producer brief still pass.
Run `cargo fmt --check`. Run `cargo clippy --all-targets -- -D warnings`. Run `cargo test`. All three pass; a passing Athena-recorded check on the final tree counts.
The diff adds no refusal or gating rule: no change to check, gate, admission or landing code, and none to athena.toml, defaults.toml or workflows.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
