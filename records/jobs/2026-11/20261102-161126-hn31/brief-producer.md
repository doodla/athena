# Job 20261102-161126-hn31: Refuse readings that join a full timed test run with a fix

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The request joined two dependent steps, a full timed workspace test run and a code fix chosen from i
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: measure-then-fix-job-producer-makes-no-change-no-measurements-or-reason (20261102-001618-f0dn)
> Cause: The request joined two dependent steps, a full timed workspace test run and a code fix chosen from its result, in one producer run with one fixed deadline and budget. A full workspace test run is long and its outcome is unknown beforehand. ...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261102-001607-rgxf (job 20261031-043143-vbee); prevention it proposed: Treat measure-then-fix requests as two stages. First a measurement job whose only deliverable is the timings entry, which the producer must write incrementally, for example by saving the raw per-binary timings file as soon as the run finish...
> Introspection 20261102-161107-0634 (job 20261102-154842-4vkh); prevention it proposed: Split measure-then-fix jobs into two chained stages, each with its own deadline and budget. Stage one writes only the measurement entry (ten binaries, run id, the slowest binary's local time) and must either commit it or record a stated rea...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (jobs 20261031-043143-vbee and 20261102-154842-4vkh): a reading accepts a request that joins a full timed workspace test run and a fix chosen from its result in one producer run with one deadline. Add a reading-acceptance check that refuses such a reading, with a message that says to split it into a measurement stage and a fix stage. The earlier jobs changed no files, so there is no candidate work to keep or drop.

Assumptions:
- Code survey: crates/athena-core/src/reading.rs: refusing_check runs the checks in order as step! lines (check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_governance_coverage, ...). A new check is added there as one step! line. crates/athena-core/tests/refusal_corpus.rs must have a case in which the new check refuses. The change adds no record or data file, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly by calling check_reading_acceptance on readings and requests.
- Goal and audience: the Athena operator and the reader stage. Measure-then-fix requests should stop failing at the producer deadline.
- Job 20261031-043143-vbee made no change and no candidate exists: its finding (the producer could not read the timed run output, so it wrote no CHANGELOG entry) no longer applies once the work is split into stages, because the combined reading is refused. Job 20261102-154842-4vkh also made no change: its finding (no CI timings file supplied) no longer applies for the same reason. Neither job's CHANGELOG.md or test-file scope is kept.
- The check is a phrase-based heuristic over the request text and the reading's criteria. It refuses a reading when the text needs a full workspace test run with timing AND a code fix chosen from its result. A reading whose only deliverable is a measurement is accepted. A fix-only reading that cites already-recorded timings is accepted.
- Expected 3% of newly refused readings: the check matches only the narrow measure-then-fix shape.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-core. A new unit test in crates/athena-core/src/reading.rs passes with the change and fails without it: a reading whose request joins a full timed workspace test run with a fix chosen from its result is refused, and the error says to split it into a measurement stage and a fix stage.
2. [behavior] Run cargo test -p athena-core --test refusal_corpus. A new case built from the request text of job 20261031-043143-vbee is refused by the new check. A second case built from the request text of job 20261102-154842-4vkh is refused by the new check. A measurement-only reading and a fix-only reading that cites recorded timings are both accepted.
3. [content] Grep crates/athena-core/src/reading.rs: the doc comment of the new check names the root cause, which is two dependent steps (a long run of unknown outcome and a fix chosen from it) in one producer run with one deadline and budget. The comment cites jobs 20261031-043143-vbee and 20261102-154842-4vkh. The check is wired as a step! line in refusing_check.
4. [behavior] Run cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace. All pass.
5. [content] Grep crates/athena-context/src/briefs.rs: the reader brief's check list names the new check, as it does for check_criteria_evidence, and the existing brief tests still pass.
6. [content] Grep CHANGELOG.md: it has an entry for the new refusal rule and its reason.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Running or timing the full workspace test suite
- Changing any test binary, xtask/shard-timings.txt or .github/workflows/ci.yml
- Changing athena.toml or defaults.toml
- Changing job deadlines or budgets
- Automatically chaining or splitting jobs in the orchestrator
- The test-speed fix itself; it is a separate later request

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test and corpus case; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at the reading gate, which lets the combined shape through, not at one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced as a check in refusing_check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and check that a refusal happens.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the named cargo commands.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 20 cases in 18 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make the disk guard and build cleanup one ordered step. Before the free-space check, and again on any low-disk hold, Athena removes the regenerable builds of held, interrupted and finished jobs, then measures free space. Only the unfreeable shortfall holds a job. Held jobs resume automatically once ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

25 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test -p athena-core. A new unit test in crates/athena-core/src/reading.rs passes with the change and fails without it: a reading whose request joins a full timed workspace test run with a fix chosen from its result is refused, and the error says to split it into a measurement stage and a fix stage.
Run cargo test -p athena-core --test refusal_corpus. A new case built from the request text of job 20261031-043143-vbee is refused by the new check. A second case built from the request text of job 20261102-154842-4vkh is refused by the new check. A measurement-only reading and a fix-only reading that cites recorded timings are both accepted.
Grep crates/athena-core/src/reading.rs: the doc comment of the new check names the root cause, which is two dependent steps (a long run of unknown outcome and a fix chosen from it) in one producer run with one deadline and budget. The comment cites jobs 20261031-043143-vbee and 20261102-154842-4vkh. The check is wired as a step! line in refusing_check.
Run cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace. All pass.
Grep crates/athena-context/src/briefs.rs: the reader brief's check list names the new check, as it does for check_criteria_evidence, and the existing brief tests still pass.
Grep CHANGELOG.md: it has an entry for the new refusal rule and its reason.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
