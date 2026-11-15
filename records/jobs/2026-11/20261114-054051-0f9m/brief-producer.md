# Job 20261114-054051-0f9m: Brief producers on flaky-test fixes: reproduce, root-cause, guard class

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The fix was applied call site by call site with no root-cause design and no mechanical way to find e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: flaky-test-fix-patches-call-sites-not-root-cause-the person running Athena-supplies-design (20261106-065823-5fm2)
> Cause: The fix was applied call site by call site with no root-cause design and no mechanical way to find every site. Nothing listed all the places that write an executable stand-in, and nothing failed when one was left behind. The criterion was a...
> Seen 4 time(s) in 4 jobs. The introspections that record it:
> Introspection 20261106-095756-138r (job 20261106-065752-jexx); prevention it proposed: Add a repo test that scans all test code for in-process executable writes (`set_permissions` with an executable mode, `from_mode(0o755)`) outside the one shared helper and fails on any hit. The test would cover all call sites, including xta...
> Introspection 20261106-104005-m047 (job 20261106-095820-edjm); prevention it proposed: Fix the root cause once, in one place, instead of detecting call sites. Test stand-ins would be created only through a single helper that writes the file and sets its mode in a child process (or writes to a temp name and renames after the h...
> Introspection 20261106-113954-57pn (job 20261106-080857-9daz); prevention it proposed: Give intermittent-failure jobs a required, recorded diagnosis-and-soak step that Athena runs itself. Before the verdict it would (1) run the named test N times under parallel cargo load on the base tree, to reproduce the flake and record th...
> Introspection 20261114-054031-8mxk (job 20261112-005306-pa2j); prevention it proposed: Add a flaky-test step to the job flow. Before editing, the producer must reproduce the flake under load and record the output among the checks. It must state the root cause in terms of the code, and the repaired test must still fail when th...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause of the repeated failure (jobs 20261106-065752-jexx, 20261106-095820-edjm, 20261106-080857-9daz, 20261112-005306-pa2j): the change producer's brief never said how to fix a flake or racy-fixture failure. It fixed one site, skipped a reproduction under load, and left no mechanical check for the class. The fix gives the producer that information in crates/athena-cap-change/prompts/producer.md, with a test that pins it. It adds no refusal rule.

Assumptions:
- Goal and audience: the change producer, whose brief gains the missing guidance, and the verifier, who judges whether a flaky-test fix is root-cause and class-wide. Athena maintainers read the result.
- Code survey: read crates/athena-cap-change/prompts/producer.md (one paragraph, loaded by include_str! at crates/athena-cap-change/src/lib.rs:21, whose #[cfg(test)] module already tests prompt text) and crates/athena-testkit/{src/lib.rs,tests/no_in_process_exec_writes.rs,tests/stand_in_stress.rs}. The testkit helper write_executable and the syn-free text scan are already in the tree. The scan flags set_permissions, from_mode, set_mode, .mode(arg), set_executable( wrappers and chmod children, and it carries planted-violation tests. This addresses edjm's major finding and jexx's gitleaks finding. No records or data file is added, so check_record_file, AnyRecord and the round-trip test are untouched.
- Prior jobs 20261106-065752-jexx and 20261106-095820-edjm: their testkit work (all 21-22 changed files, including crates/athena-testkit/*, the six converted test sites, Cargo.toml and Cargo.lock edits, docs/architecture.md, docs/decisions.md, docs/decision-links.toml, xtask/shard-timings.txt and CHANGELOG.md) is kept as already present in the tree and is not re-edited. The findings they raised (the xtask gitleaks fake, the scan missing wrappers and chmod) are already fixed in the tree, and a Grep for set_executable( in test code confirms it.
- Prior job 20261106-080857-9daz: its drain.rs fix is kept as is. Its finding was no reproduction under load and no recorded 200-run loop. The new producer guidance requires exactly that evidence for future flake jobs, which is the class-level fix.
- Existing code: crates/athena-cap-change/prompts/producer.md and crates/athena-cap-change/src/lib.rs are extended. The criterion below reuses lib.rs's existing prompt tests.
- Prompt headroom: crates/athena-cap-change/prompts/producer.md has 157 tokens left. The added text stays under about 110 tokens. It is not in the verifier prompt (59 left), and the shared contract and reader prompts are untouched.
- Guidance content, for the producer: when the failure is intermittent or a race, (1) reproduce it under parallel load on the base tree and report the command and counts; (2) state the cause in terms of the code; (3) fix it where it originates, through one shared helper rather than at each call site, and add or extend a repo test that scans for every site of the pattern; (4) show the new test fails when the cause is reintroduced; (5) report a soak run after the fix.
- No refusal rule is added, so the expected-refusal-share requirement does not apply.
- No governance file (athena.toml, defaults.toml, workflows) needs a change.

## Acceptance criteria

1. [content] crates/athena-cap-change/prompts/producer.md names the root cause, a failure class that comes from an information gap in the producer brief. It tells the producer, for an intermittent or racy failure, to reproduce under parallel load, state the cause in code terms, fix it in one shared helper rather than per call site, add a repo scan test for all sites, and show the test fails with the cause reintroduced. The check is a diff of the file.
2. [behavior] A new unit test in crates/athena-cap-change/src/lib.rs asserts that the producer prompt text contains the flaky-test guidance. It fails on the base tree and passes with the change. Run `cargo test -p athena-cap-change`.
3. [behavior] The prompt token-budget tests still pass with the larger producer prompt. Run `cargo test` and read the recorded Athena test check on the final tree.
4. [behavior] fmt, clippy and test pass: the recorded Athena checks `cargo fmt --check`, `cargo clippy` and `cargo test` on the final tree.
5. [content] Jobs 20261106-065752-jexx, 20261106-095820-edjm and 20261106-080857-9daz are addressed. The diff does not touch crates/athena-testkit, drain.rs or the converted test sites, and a Grep for `set_executable(` in test code shows no in-process stand-in write left. CHANGELOG.md gains one entry that states only what the diff does.
6. [content] The diff adds no refusal rule and no check that rejects readings or jobs.

## Expected deliverables

- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- All files changed by jobs 20261106-065752-jexx and 20261106-095820-edjm are kept and not re-edited: crates/athena-testkit/**, crates/athena-core/src/{checks,reading,test_support}.rs, crates/athena-agent/src/claude.rs, crates/athena-git/{src/snapshot.rs,src/lib.rs}, crates/athena-cli/tests/reconcile.rs, the Cargo.toml and Cargo.lock edits, xtask/src/main.rs, xtask/Cargo.toml, xtask/shard-timings.txt and docs/architecture.md.
- crates/athena-cli/src/drain.rs, the file changed by job 20261106-080857-9daz, is kept as is.
- Any new refusal rule, gate or job-flow step that holds or rejects work.
- athena.toml, defaults.toml, workflows, and the verifier and shared contract prompts, which have no headroom.
- Retries at run sites and non-Linux support.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the missing producer guidance that caused the class, and does not patch one flake.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A prompt test is added and fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion is a test that fails when the guidance is removed.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that the guidance stays in the producer prompt.
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

- path: <state>/jobs/20261114-054051-0f9m/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-054051-0f9m/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-cap-change/prompts/producer.md names the root cause, a failure class that comes from an information gap in the producer brief. It tells the producer, for an intermittent or racy failure, to reproduce under parallel load, state the cause in code terms, fix it in one shared helper rather than per call site, add a repo scan test for all sites, and show the test fails with the cause reintroduced. The check is a diff of the file.
A new unit test in crates/athena-cap-change/src/lib.rs asserts that the producer prompt text contains the flaky-test guidance. It fails on the base tree and passes with the change. Run `cargo test -p athena-cap-change`.
The prompt token-budget tests still pass with the larger producer prompt. Run `cargo test` and read the recorded Athena test check on the final tree.
fmt, clippy and test pass: the recorded Athena checks `cargo fmt --check`, `cargo clippy` and `cargo test` on the final tree.
Jobs 20261106-065752-jexx, 20261106-095820-edjm and 20261106-080857-9daz are addressed. The diff does not touch crates/athena-testkit, drain.rs or the converted test sites, and a Grep for `set_executable(` in test code shows no in-process stand-in write left. CHANGELOG.md gains one entry that states only what the diff does.
The diff adds no refusal rule and no check that rejects readings or jobs.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
