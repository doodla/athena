# Job 20261114-045630-wqa6: Give the reader a numbered request-items and source-facts brief section

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Two gaps allowed this. First, the reading was accepted without the user confirming it against the re
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: reading-or-provenance-mismatch-caught-only-at-final-verdict (20261015-205707-ycg1)
> Cause: Two gaps allowed this. First, the reading was accepted without the user confirming it against the request, so criteria derived from it could all pass while the request went unmet. Second, the commit label and the source actually read come f...
> Seen 21 time(s) in 21 jobs. The introspections that record it:
> Introspection 20261015-205658-9fta (job 20261015-201650-cyvb); prevention it proposed: Add a code check that runs before the verifier. It reads the commit of the mounted source tree and compares it with the pinned commit in pin.json. It also compares both with the commit label in every claims and rationales file, and it fails...
> Introspection 20261016-163602-mx80 (job 20261016-161517-y9dy); prevention it proposed: Add a reading-admission check that lists every symptom, outcome and exclusion in the request as a numbered item. Each item must map to at least one acceptance criterion, or be explicitly marked out of scope and shown to the user. Add a pre-...
> Introspection 20261017-062450-ft3t (job 20261017-054106-xgc2); prevention it proposed: Add a pre-build reading check. When a request cites a design section, each invariant or exclusion in that section (such as "skip only when prose alone changed" or "verification is not weakened") must map to a named acceptance criterion and ...
> Introspection 20261027-082504-fd7n (job 20261027-074457-bx8e); prevention it proposed: Add a code check in the site test that walks every transition in the study's model.json structurally. For each transition it should resolve the claim ids the trigger cites, require them to exist, and require that the trigger's actor matches...
> Introspection 20261029-110514-bp4m (job 20261029-084645-sza7); prevention it proposed: Add a reading-vs-request check before the producer is dispatched. It would extract each distinct requirement from the request text (here: the 1800s value, and the timed-out-check-as-environment behavior). It would require every one to map t...
> Introspection 20261029-224519-wmrd (job 20261029-155620-djjt); prevention it proposed: Before the producer runs, check the reading in code. Every record type, field or code location the request names must be listed in the reading and resolved to a real writer in the repo. The reading's criteria must then name that record, not...
> Introspection 20261107-094312-hkeg (job 20261027-054230-8nz8); prevention it proposed: When a request cites past jobs as the evidence for a new rule, the reading stage should pull each cited job's recorded case and write down how it was classified. A reading-refusal job should then be required to include a replay test that ru...
> Introspection 20261107-151816-b2b5 (job 20261107-144832-7wsq); prevention it proposed: Two checks would have prevented the class. (1) Before a reading is accepted, a step compares it with the request and with the named scheme or decision it depends on, lists every required behavior (here, link plus clean after each command), ...
> Introspection 20261108-063542-t8sk (job 20261026-144234-bz4e); prevention it proposed: Add a code check that runs when a reading is accepted. For any request that names a size or line-count target, it measures the named files in the current snapshot and records the measured values in the reading. It refuses or flags the readi...
> Introspection 20261108-100303-9qwv (job 20261026-144242-sjp3); prevention it proposed: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then ...
> Introspection 20261108-223920-r9y6 (job 20261028-000831-pm4f); prevention it proposed: Add a code check that runs before the final verdict. For a job whose criteria name specific job ids, the check reads the live records and lists, per id, whether an outcome record exists and whether a closed job cites a commit hash found in ...
> Introspection 20261109-004012-5dra (job 20261027-114907-99s2); prevention it proposed: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must s...
> Introspection 20261109-092950-9vqt (job 20261109-080749-p0fs); prevention it proposed: When a reading's deliverable states a limit, exception or "only N" claim about Athena's behavior, require the reading and plan to derive the list from the enforcing source (the admission path classification code and config) and cite it. Add...
> Introspection 20261109-155022-5gs7 (job 20261025-001238-mkmh); prevention it proposed: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must eithe...
> Introspection 20261110-151012-mqkr (job 20261110-124254-zegf); prevention it proposed: Add a reading-stage check for bug-style or "show X in the brief" requests. The reader must name the code path that currently produces or omits the behavior, and the reading is accepted only if it cites a function and line showing the gap. I...
> Introspection 20261110-163319-jskc (job 20261110-151108-70nx); prevention it proposed: Add a check at reading acceptance for requests of the root-cause or bug kind. The reading must list the request's stated cause as a claim. Each claim must cite the code location that shows the cause (file and lines) or the recorded instance...
> Introspection 20261110-165627-qsbx (job 20261110-163344-3gaa); prevention it proposed: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) ...
> Introspection 20261111-083835-fjtj (job 20261111-075222-nsk3); prevention it proposed: When a criterion says a test must assert a property of rendered output, the producer's brief should require two things before the job is done. The test must run against the real rendered page or the real source data. And it must be shown to...
> Introspection 20261112-002519-0es3 (job 20261030-001200-6mq2); prevention it proposed: Add a check before the producer runs that compares the accepted reading with the request's own wording. Require any new rule or check, and any measure text claiming what status shows, to be shown on real data: the rule must fail on a known-...
> Introspection 20261112-081109-b2cj (job 20261030-013416-weaa); prevention it proposed: Add a code check that runs before the verifier, at reading acceptance and again after the producer finishes. It would (1) validate every measure label a job adds or edits in docs/spec.toml against STATUS_MEASURE_LABELS by running the real-s...
> Introspection 20261112-172939-18dh (job 20261102-013753-m8sd); prevention it proposed: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader brief gives only the request as free text, with no numbered list of its requirements and no measured facts about the files and sources it names. Readings then drop qualifiers ("as today", the default budget room check), miss pin/mount mismatches or misstate limits, and this surfaces only at the final verdict. Fix it in the brief builder by adding two code-generated sections, request items and measured facts, with no new refusal rule. Retry of 20261015-201650-cyvb, 20261016-161517-y9dy, 20261017-054106-xgc2.

Assumptions:
- Code survey: crates/athena-context/src/briefs/reader.rs (reader brief assembly, PRIOR_RESULT_HEADING at line 126, Referenced jobs section at line 493) and crates/athena-context/src/briefs.rs tests are the entry point; the reader prompt file is at its token budget (2 left), so the change is in code-generated brief text, not prompt files. No record or data file is added under crates/, so no AnyRecord or check_record_file change. The property is measured directly by asserting on the built brief string.
- Goal and audience: the reader stage, and the user who judges its readings, get the information the failing stage lacked; this is preferred over a refusal rule, which the request allows only when information cannot prevent the failure.
- Item extraction is mechanical (sentence split with qualifier phrases kept verbatim); it informs the reader and refuses nothing.
- Out of scope: re-doing the three earlier jobs' features, re-pinning firstmate, changing athena.toml, defaults.toml or workflows. Consumers: none found for new config, since no config key is added.
- The 'user confirms the reading' half of the cause is handled by showing the numbered items in the reading output, not by a new approval gate.

## Acceptance criteria

1. [content] CHANGELOG.md has an entry naming the root cause: the reader brief carried the request only as free text, with no numbered request items and no measured facts (named file sizes, pinned vs mounted commit) for the reader to cover.
2. [behavior] Run `cargo test -p athena-context`: a new test builds a reader brief from a request with several sentences and a cited design section, and asserts the brief contains a numbered '# Request items' section listing each requirement and qualifier. The test fails on the code before the change.
3. [behavior] Run `cargo test -p athena-context`: a new test builds a brief for a request naming a file and a line limit and asserts the brief shows the measured size of that file; another asserts that when the mounted source commit differs from the pinned commit, both are shown in the brief. Both tests fail before the change and test the cause, not one job's text.
4. [behavior] Run `cargo test -p athena-context`: a test shows the new sections are absent from the brief when the request names nothing to measure, so ordinary requests do not grow.
5. [content] The change adds no refusal rule: no check_* function in crates/athena-core/src/reading/ is added or tightened, so the expected share of newly refused past readings is 0%.
6. [content] Prior jobs: 20261015-201650-cyvb (pin vs mount mismatch, shown in brief facts), 20261016-161517-y9dy (budget room check, covered by the request items), 20261017-054106-xgc2 (dropped 'as today' qualifier, kept in request items) are addressed through the brief, not by re-doing their changes. Their candidate files (studies/firstmate-work-handling/*, xtask/src/site.rs, steward.rs, drain.rs, routing.rs, athena.toml, ci-routing.md and the rest) are dropped, not kept.
7. [behavior] fmt, clippy and test pass: Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` on the final tree.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Re-doing the changes from jobs 20261015-201650-cyvb, 20261016-161517-y9dy and 20261017-054106-xgc2
- Adding refusal rules to reading checks
- Changing athena.toml, defaults.toml or workflows
- Editing prompt files

## User defaults that apply

- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Brief content is produced by code and tested.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the brief builder in tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in the brief, not per job.
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

- path: <state>/jobs/20261114-045630-wqa6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045630-wqa6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md has an entry naming the root cause: the reader brief carried the request only as free text, with no numbered request items and no measured facts (named file sizes, pinned vs mounted commit) for the reader to cover.
Run `cargo test -p athena-context`: a new test builds a reader brief from a request with several sentences and a cited design section, and asserts the brief contains a numbered '# Request items' section listing each requirement and qualifier. The test fails on the code before the change.
Run `cargo test -p athena-context`: a new test builds a brief for a request naming a file and a line limit and asserts the brief shows the measured size of that file; another asserts that when the mounted source commit differs from the pinned commit, both are shown in the brief. Both tests fail before the change and test the cause, not one job's text.
Run `cargo test -p athena-context`: a test shows the new sections are absent from the brief when the request names nothing to measure, so ordinary requests do not grow.
The change adds no refusal rule: no check_* function in crates/athena-core/src/reading/ is added or tightened, so the expected share of newly refused past readings is 0%.
Prior jobs: 20261015-201650-cyvb (pin vs mount mismatch, shown in brief facts), 20261016-161517-y9dy (budget room check, covered by the request items), 20261017-054106-xgc2 (dropped 'as today' qualifier, kept in request items) are addressed through the brief, not by re-doing their changes. Their candidate files (studies/firstmate-work-handling/*, xtask/src/site.rs, steward.rs, drain.rs, routing.rs, athena.toml, ci-routing.md and the rest) are dropped, not kept.
fmt, clippy and test pass: Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
