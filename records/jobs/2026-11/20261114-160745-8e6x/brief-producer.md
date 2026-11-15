# Job 20261114-160745-8e6x: Record the pre-verdict criteria self-check on every producer attempt

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Nothing in the stored event shows the unmet criterion being checked before the final verdict. The li
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: criterion-unmet-at-final-verification (20261014-172333-6fy0)
> Cause: Nothing in the stored event shows the unmet criterion being checked before the final verdict. The likely gap is that criteria are verified only at the end, so a miss costs the whole job instead of triggering a repair pass. The goal of the j...
> Seen 43 time(s) in 43 jobs. The introspections that record it:
> Introspection 20261014-172320-k3k0 (job 20261014-170031-2r9b); prevention it proposed: Have the implementer check each acceptance criterion against concrete evidence before submitting, and give the job a bounded repair loop: when the verifier reports an unmet criterion or a blocker or major finding, the job returns to the imp...
> Introspection 20261014-174804-mgn1 (job 20261014-172342-rqgb); prevention it proposed: Add a pre-verdict step where the worker maps each acceptance criterion to concrete evidence and runs a repair pass for any criterion without it. For "update every mention" criteria, require a repo-wide search with zero remaining stale hits,...
> Introspection 20261014-181337-4ns7 (job 20261014-174937-hrzn); prevention it proposed: Add a pre-submission self-check step: before a job goes to the verifier, the worker lists each acceptance criterion with the evidence that it is met (for sweep criteria, the search command and its remaining-hit result). For a redo, the earl...
> Introspection 20261014-213402-g1b6 (job 20261014-192610-xgke); prevention it proposed: Before a job can be declared complete, require a self-check step. It runs every accepted criterion as a concrete check, such as a run showing the steward reads the named sources and writes items to the queue. It must record pass or fail per...
> Introspection 20261014-222026-jm1z (job 20261014-221551-09t4); prevention it proposed: At admission, turn each outcome into acceptance criteria that name a runnable check. For docs, that means link and build checks plus a defined newcomer-readability check. Require the job to run those checks before it ends its work phase and...
> Introspection 20261015-013952-zyzm (job 20261015-012203-y8dx); prevention it proposed: Add a check during the build, before final verification. It runs each acceptance criterion as a concrete test and reports unmet ones to the builder while there is still time to fix them. Record the unmet criteria and findings in the rejecti...
> Introspection 20261015-060222-3dap (job 20261015-054026-k2ej); prevention it proposed: Add a pre-submission gate in code. The producer cannot send its final report until (1) it has a per-criterion evidence entry for every acceptance criterion (file and line, or test name), and (2) the system itself has run cargo fmt --check, ...
> Introspection 20261015-092454-x3d0 (job 20261015-085652-5qem); prevention it proposed: Add a pre-report gate in code. The producer cannot submit a final report until (1) the workspace checks (cargo fmt --check, cargo clippy --all-targets, cargo test) have been run by the harness on the current tree and the results recorded, a...
> Introspection 20261015-124204-qxbg (job 20261015-104515-8wqq); prevention it proposed: Add a code gate on the producer's final report. It must carry a per-criterion evidence table, with the command output or file excerpt for each acceptance criterion. It must also carry recorded results of the declared gate commands (test, cl...
> Introspection 20261015-131120-cjyk (job 20261015-124217-6xhe); prevention it proposed: Add a pre-transfer check in the job runner. It should refuse to finish a build until the final report has one row for every acceptance criterion, each with non-empty evidence (a test name or HTML excerpt), plus a section with the exit status...
> Introspection 20261015-152122-10ms (job 20261015-145809-7565); prevention it proposed: Add a pre-handover criterion check to the build. Before a job can finish, the builder must produce a table mapping each acceptance criterion to its evidence. For a criterion that asks for a test, the evidence is a named test that exists and...
> Introspection 20261015-215411-2zxz (job 20261015-214235-be8x); prevention it proposed: Add a completion gate in code that blocks the producer's "done" report until it has (a) submitted a per-criterion evidence table, each row naming a file, line or command output, and (b) run the workspace gate (cargo test, cargo clippy, carg...
> Introspection 20261015-225720-e0r8 (job 20261015-223418-qb41); prevention it proposed: Add a code gate that rejects the producer's final report before verification unless: (a) it contains an evidence entry for every acceptance criterion id, and (b) the gate commands (cargo test, clippy, fmt, the site build) have recorded exit...
> Introspection 20261016-005745-2wcm (job 20261016-003525-5wwj); prevention it proposed: Add a gate that blocks a producer's final report until it includes a table mapping every acceptance criterion to evidence, and rejects the report as incomplete if any criterion has no row. For structural criteria such as "not a special case...
> Introspection 20261016-011709-3n00 (job 20261016-005953-582g); prevention it proposed: A completion gate in code that refuses a producer's final report as "done" until it contains a table mapping every acceptance criterion to evidence, and until the gate commands (cargo test, cargo clippy --all-targets, cargo fmt --check) hav...
> Introspection 20261016-170323-hzhy (job 20261016-163644-txka); prevention it proposed: Add a pre-handover code check that runs before the job is sent to the verifier. For every acceptance criterion that names a command (cargo test, clippy, fmt --check, or a CLI behavior), the check requires an evidence entry that records the ...
> Introspection 20261017-011645-513m (job 20261017-004000-824m); prevention it proposed: Add a pre-verification check for criteria of the form 'term X is no longer used for Y'. The check greps all job-owned files (model.json, site.rs, glossary and card text) for the retired term. It lists every remaining occurrence, and the job...
> Introspection 20261017-014659-8e83 (job 20261017-011658-5ap5); prevention it proposed: Add a pre-handover gate that runs before the job can reach final verification. For each acceptance criterion, the reading names the evidence file or path that proves it. For each expected deliverable file, the gate checks that the diff touc...
> Introspection 20261017-114101-rsa3 (job 20261017-111119-m988); prevention it proposed: Add a pre-handover check in the run stage. For every criterion that names commands (such as the cargo test, clippy and fmt gate), Athena itself runs the commands and records the command lines, exit codes and output tail as evidence, or it r...
> Introspection 20261017-155222-5b78 (job 20261017-152733-efyd); prevention it proposed: Add a pre-handover gate that runs before the verifier. It would require (1) each criterion to map to a named test, or to a command that ran with its output, so a criterion asking for tests showing specific wiring fails if no such test exist...
> Introspection 20261023-214309-m5t6 (job 20261023-183334-aej6); prevention it proposed: Add a pre-handover check that blocks the producer from finishing until every acceptance criterion has a recorded evidence entry. For criteria that name a test, the entry must point to a test that exists in the diff and passed in a recorded ...
> Introspection 20261024-015841-5c6v (job 20261023-234313-h49d); prevention it proposed: Add a pre-handover check to the producer stage. When a criterion says that some wording or rule must no longer appear, the reading must name the phrase, or a pattern for it. The check then searches the listed docs (decisions.md, the design ...
> Introspection 20261024-045439-zj2c (job 20261024-033933-ta60); prevention it proposed: Add a pre-handover check in the pipeline. Athena itself runs the gate commands (`cargo test`, `cargo clippy --all-targets`, `cargo fmt --check`) in the producer's working copy. It stores their command lines, exit codes and output tails in t...
> Introspection 20261024-074821-hxps (job 20261024-033935-c4w4); prevention it proposed: Add a pre-handover code check: before a job moves from producer to verifier, require an evidence record for every criterion. For command-based criteria such as the clippy, fmt and test gate, it must hold the exact command, exit status and c...
> Introspection 20261024-170056-k1na (job 20261024-161559-rxjg); prevention it proposed: Add a pre-handover code check that runs before the job moves to verification. It would require the producer's change description to exist and to contain, for each acceptance criterion, a named test or a command with its recorded output. For...
> Introspection 20261025-022100-waxw (job 20261025-012657-hac3); prevention it proposed: Add a pre-handover check that runs after the producer finishes and before the verifier. For each acceptance criterion it requires a named test or command. It runs them and refuses handover if one fails or is missing. It also flags tests who...
> Introspection 20261025-031419-czhp (job 20261025-022144-emm8); prevention it proposed: Add a pre-handover check, run by code, that the producer must pass before it can hand over. For a change to landing or history code, the producer lists every writer and reader of the changed path (land_pr, retry_landing, reconcile, check_hi...
> Introspection 20261026-155302-zdnq (job 20261026-150607-85mv); prevention it proposed: Add a pre-handover check for any job that moves or changes a set of readers or writers. It lists every reader from a single registry, or from a grep of the loader call sites. It then requires each one to have a named test in the deliverable...
> Introspection 20261026-180059-a51n (job 20261026-170042-cgq9); prevention it proposed: Add a pre-handover check for reader-move jobs. It would enumerate the readers by grep and require, for each, a named test that calls the reader's real entry point. It would also run a mutation step: revert each reader to the in-tree path an...
> Introspection 20261027-011257-jxh3 (job 20261027-004517-gk9e); prevention it proposed: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when ...
> Introspection 20261027-085315-n35f (job 20261027-082521-0sfz); prevention it proposed: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The pr...
> Introspection 20261028-232906-k1qr (job 20261027-160633-t5sh); prevention it proposed: Classify the usage-limit error once, at the single point where model-call errors are turned into results, as a typed "hold" outcome. Every caller (reading, planning, critic, producer, verifier) must receive that type, and the drain's failur...
> Introspection 20261029-023321-tn6t (job 20261026-232411-2zv3); prevention it proposed: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run wit...
> Introspection 20261030-004950-fqve (job 20261029-142853-x4nr); prevention it proposed: Add a test for any recorded-state feature that runs through the real implementation (RealSteps) instead of a fake. It would start a producer stage, read the in-flight table entry, and assert that the recorded path equals the directory the p...
> Introspection 20261030-013059-bf4h (job 20261030-005008-tf2s); prevention it proposed: Have the producer and checks code emit typed stage events (an enum with Producer, Checks and so on) at each transition, including the send-back transition, and have the in-flight table record only those. Do not parse display strings. Add a ...
> Introspection 20261030-065923-cp41 (job 20261029-233021-e5sg); prevention it proposed: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the fi...
> Introspection 20261031-111717-s4qq (job 20261031-110024-4qd3); prevention it proposed: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a f...
> Introspection 20261101-113931-j9at (job 20261101-102128-b1st); prevention it proposed: Add a pre-verdict step in code. Before the verifier runs, Athena runs the repo gate (fmt, clippy, test) itself and records the exact commands and exit codes. It also checks that every behavior criterion has a demonstration that names a comm...
> Introspection 20261101-163615-67fa (job 20261101-135902-9bxs); prevention it proposed: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If...
> Introspection 20261103-014831-3d1n (job 20261103-011406-wm2v); prevention it proposed: Add a pre-verdict gate in the job pipeline. After the producer stops, Athena itself runs fmt --check and clippy --all-targets -D warnings (and test) on the job's snapshot. If any fail, it sends the exact output back to the producer for one ...
> Introspection 20261103-101121-71yx (job 20261103-092929-s7ts); prevention it proposed: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a d...
> Introspection 20261103-202623-649w (job 20261016-005805-p806); prevention it proposed: Before the producer may finish, Athena runs the cheap mechanical gate on the working copy: the decision-link consistency test (every "amends/supersedes Dxx" link in docs/decisions.md is in docs/decision-links.toml), fmt, clippy, and a scan ...
> Introspection 20261103-222715-7xrj (job 20261103-193428-df2m); prevention it proposed: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-cha...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer's per-criterion self-check (unmet_criteria in run/send_back.rs, run in run/producer.rs) is stored on the attempt's invocation detail only when it fails and sends the producer back. A passing check, or one that reports met, leaves no stored event, so a criterion the verifier later finds unmet shows no earlier check. The fix stores the check on every attempt, passing included, and shows what it ran against.

Assumptions:
- Goal and audience: whoever reads a stored job record (the owner, introspection and the verifier) after a job fails final verification. They need to see the criterion check that ran before the verdict.
- Code survey: read crates/athena-core/src/run/send_back.rs (unmet_criteria, left_to_athena, send_back_text), crates/athena-core/src/run/producer.rs lines 440-710 (the self-check loop, the invocation detail set only at send-back, and PRODUCER_SEND_BACKS) and crates/athena-context/prompts/producer.md (it already asks for criteria_check). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. The detail field already exists on the invocation record, so the schema is unchanged. The property is measured directly by reading the stored invocation detail in the run tests.
- The producer already receives its brief with every criterion and a send-back for unmet ones. The missing part is the stored evidence, not the producer's information, so no refusal rule is added; this follows the 'give information, not refusal' preference.
- Prior jobs 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn are treated as a different, already-rejected effort on role prompts. Their changed files (prompts/reader.md, producer.md, verifier.md, athena-context/src/lib.rs, athena-core/src/verify.rs, docs/decisions.md, docs/design/*, docs/records.md) are dropped as out of scope, and CHANGELOG.md is kept as a deliverable.
- Existing code: crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs and crates/athena-core/src/run/tests.rs are extended, not replaced.
- The verifier model and the reader/verifier prompt budgets stay unchanged; reader.md has 2 tokens of headroom, so prompt edits are avoided.

## Acceptance criteria

1. [content] The change description names the root cause: the producer's per-criterion self-check is stored (invocation detail in crates/athena-core/src/run/producer.rs) only when it fails and sends the producer back, so a passing check or one reported as met leaves no stored event before the final verdict.
2. [content] In crates/athena-core/src/run/producer.rs, the attempt's invocation detail is set on the passing path (the `break Some(report)` at the empty-problems branch) as well as the send-back path. It lists each accepted criterion with its met, unmet or left-to-Athena status. A diff reader can find this in the producer.rs hunk.
3. [behavior] A new test in crates/athena-core/src/run/tests.rs runs the producer stage with the fake agent, on a report that marks every criterion met. It asserts that the stored invocation detail names each criterion and its status. It fails on the base tree and passes with the change; check with `cargo test -p athena-core`.
4. [behavior] A second new test in run/tests.rs covers the case where the report omits a criterion and the send-back is used up. It asserts that the stored detail still lists that criterion as unmet, so the stored event shows the check ran. Check with `cargo test -p athena-core`.
5. [content] The existing 'self-check failed' details asserted in run/tests.rs and run_pin_tests.rs keep their wording. The change adds no refusal rule, so no replay share applies: expected 0% of newly refused readings.
6. [content] CHANGELOG.md has a new entry for this change. It does not edit docs/decisions.md or anything under records/. Prior jobs 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn changed role prompts and the reader budget, and their findings (stale 1,800 mentions, over-wide doc lines) do not apply here. None of their files are kept except CHANGELOG.md.
7. [behavior] Athena's recorded checks for the final tree (cargo fmt --check, cargo clippy --all-targets, cargo test) all passed.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Prompt edits: crates/athena-context/prompts/reader.md, producer.md and verifier.md, and the reader token budget (the prior jobs' work, dropped)
- crates/athena-context/src/lib.rs, crates/athena-core/src/verify.rs, crates/athena-cli/tests/budgets.rs (dropped)
- docs/decisions.md, docs/records.md and docs/design/* edits from prior jobs 20261014-170031-2r9b, 20261014-172342-rqgb, 20261014-174937-hrzn (dropped)
- A new repair loop or changing PRODUCER_SEND_BACKS
- Changing the verifier or its verdict rules
- Edits to records/ or owner files

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the single place where the self-check outcome is recorded, covering all jobs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus the fmt, clippy and test gate.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the stored detail content, not that wiring exists.
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

- path: <state>/jobs/20261114-160745-8e6x/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-160745-8e6x/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change description names the root cause: the producer's per-criterion self-check is stored (invocation detail in crates/athena-core/src/run/producer.rs) only when it fails and sends the producer back, so a passing check or one reported as met leaves no stored event before the final verdict.
In crates/athena-core/src/run/producer.rs, the attempt's invocation detail is set on the passing path (the `break Some(report)` at the empty-problems branch) as well as the send-back path. It lists each accepted criterion with its met, unmet or left-to-Athena status. A diff reader can find this in the producer.rs hunk.
A new test in crates/athena-core/src/run/tests.rs runs the producer stage with the fake agent, on a report that marks every criterion met. It asserts that the stored invocation detail names each criterion and its status. It fails on the base tree and passes with the change; check with `cargo test -p athena-core`.
A second new test in run/tests.rs covers the case where the report omits a criterion and the send-back is used up. It asserts that the stored detail still lists that criterion as unmet, so the stored event shows the check ran. Check with `cargo test -p athena-core`.
The existing 'self-check failed' details asserted in run/tests.rs and run_pin_tests.rs keep their wording. The change adds no refusal rule, so no replay share applies: expected 0% of newly refused readings.
CHANGELOG.md has a new entry for this change. It does not edit docs/decisions.md or anything under records/. Prior jobs 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn changed role prompts and the reader budget, and their findings (stale 1,800 mentions, over-wide doc lines) do not apply here. None of their files are kept except CHANGELOG.md.
Athena's recorded checks for the final tree (cargo fmt --check, cargo clippy --all-targets, cargo test) all passed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
