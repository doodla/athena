# Job 20261101-163757-483n: Check agent login before dispatch and each stage; hold on expiry

## Request (verbatim)

> Before dispatching a job (and before each agent stage), check that the agent's login is still valid, for example with a cheap authenticated call or the token's expiry time. If it has expired or will expire within the stage deadline, hold the job as an environment stop with a clear message to re-authenticate, instead of starting the producer. A 401 should no longer only show up mid-run. Add a test where an expired-login stub makes dispatch hold the job without running the producer.
> 
> Acceptance:
> - The change is made as described: Expired agent login fails the producer with a 401 after dispatch; nothing checks the login before a job is dispatched
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261028-112757-sygc.json

## Reading (accepted by the user)

Add a login-validity probe run before a job is dispatched and before each agent stage. If the login is expired, or expires within the stage deadline, hold the job as an environment stop with a re-authenticate message and start no producer, modelled on the existing disk_gate/hold_for_disk path. Audience: whoever drains jobs and sees an environment hold instead of a mid-run 401.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs (produce_and_verify, disk_gate and hold_for_disk run before the producer and hold the job as an environment stop with no candidate; producer_stop_reason in crates/athena-core/src/disk.rs gives the environment reason for an expired login) and crates/athena-agent/src/claude.rs (api_error_status classifies a 401; the driver retries it once after AUTH_RETRY_WAIT; tests use a shell stub with MODE=auth_always). Nothing checks the login before the producer starts. The new gate runs first in produce_and_verify and at dispatch, ahead of each agent stage, reusing the hold path and the 401 classifier.
- Records validation: no new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected; the hold reuses the existing environment-stop closing.
- The login check is a trait or method on the agent driver so a stub can return expired, valid, or expiring-soon; the real implementation uses a cheap authenticated call or the credential's expiry time. The stage deadline comes from the existing per-stage timeout config.
- The named file records/failure-classes/2026-10/20261028-112757-sygc.json is the failure-class evidence and is read-only; it is not a deliverable.
- Governance files (athena.toml, defaults.toml, workflows) are not changed; no new config key is added and the stage deadline reuses an existing value. Consumers: none found.
- Environment: no CI workflow, push or PR is touched; tests run offline against stubs.

## Acceptance criteria

1. [content] A new login-validity check in crates/athena-agent/src/ (or the run module) returns expired when the login is expired or expires within the stage deadline; the diff shows it called in crates/athena-core/src/run/mod.rs before the producer starts and before each agent stage, not only after a 401.
2. [content] When the check fails, the job is held through the existing environment-stop path (like hold_for_disk), with a message telling the user to re-authenticate; the diff shows no candidate or resume mark is created.
3. [behavior] A new test in crates/athena-core/src/run/tests.rs uses an expired-login stub; it asserts dispatch holds the job as an environment stop and the producer stub is never invoked (a call counter or marker stays zero). Run cargo test; the test passes, and fails if the gate call is removed.
4. [behavior] A second test with a valid-login stub shows the producer still runs and the job proceeds as before. Run cargo test and see it pass.
5. [behavior] Existing tests still pass and no unrelated behavior changes: run cargo test (the whole suite), cargo clippy and cargo fmt --check, each as its own command, and all pass.
6. [content] CHANGELOG.md gains an entry for the change, and the new rule is stated in docs where hold rules are described, recorded as a new appended decision in docs/decisions.md with a Provenance bullet; no existing entry is edited.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the existing mid-run 401 retry behavior
- Automatic re-authentication or token refresh
- Edits to records/ or athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt --check to pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks the producer is never run on expired login.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a gate in code, with a test that fails without it.
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

- path: <state>/jobs/20261101-163757-483n/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261101-163757-483n/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new login-validity check in crates/athena-agent/src/ (or the run module) returns expired when the login is expired or expires within the stage deadline; the diff shows it called in crates/athena-core/src/run/mod.rs before the producer starts and before each agent stage, not only after a 401.
When the check fails, the job is held through the existing environment-stop path (like hold_for_disk), with a message telling the user to re-authenticate; the diff shows no candidate or resume mark is created.
A new test in crates/athena-core/src/run/tests.rs uses an expired-login stub; it asserts dispatch holds the job as an environment stop and the producer stub is never invoked (a call counter or marker stays zero). Run cargo test; the test passes, and fails if the gate call is removed.
A second test with a valid-login stub shows the producer still runs and the job proceeds as before. Run cargo test and see it pass.
Existing tests still pass and no unrelated behavior changes: run cargo test (the whole suite), cargo clippy and cargo fmt --check, each as its own command, and all pass.
CHANGELOG.md gains an entry for the change, and the new rule is stated in docs where hold rules are described, recorded as a new appended decision in docs/decisions.md with a Provenance bullet; no existing entry is edited.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
