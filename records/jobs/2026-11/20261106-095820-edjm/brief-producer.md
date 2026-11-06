# Job 20261106-095820-edjm: Finish ETXTBSY fix: all test stand-ins via shared child-process helper

## Request (verbatim)

> try again on 20261106-065752-jexx

## Reading (accepted by the user)

Retry of job 20261106-065752-jexx for developers and CI running the Linux suite: a new test-only crate athena-testkit holds one helper writing executable stand-ins from an `sh -c 'cat > tmp && chmod 755 tmp && mv tmp final'` child. Every test site, including the missed xtask gitleaks fake, uses it, and a scan test fails on any in-process executable write in test code.

Assumptions:
- Job 20261106-065752-jexx findings addressed: (1) major, the xtask test a_cached_gitleaks_is_skipped_only_when_it_reports_the_pinned_version still wrote in-process via std::fs::write + set_executable, so it is converted to the helper; (2) minor, the CHANGELOG overclaim is fixed by writing the entry from the final diff; (3) minor, docs/architecture.md was changed outside the deliverables, and it is now a listed deliverable (a crate-table row for athena-testkit is kept); (4) nit, the in-process ETXTBSY count was visible only with --nocapture, so the stress test now prints it in a way that appears in recorded output (for example, it asserts nothing on the count but the test run uses println via a documented command, or the count is written into the test's passing output through a non-captured channel). The earlier candidate is the starting point: its 21 changed files are all kept as deliverables.
- Code survey: grep over crates/ and xtask shows the test-side executable writes at xtask/src/main.rs:809 (set_executable helper) and :837, crates/athena-core/src/reading.rs:8144, crates/athena-core/src/checks.rs:1009, crates/athena-agent/src/claude.rs:641 and crates/athena-cli/tests/reconcile.rs:140. The other set_permissions hits in crates/athena-core/src/{toolconfig,pin,snapshot}.rs and crates/athena-git/src/snapshot.rs lines 445-516 are production code and stay. No records or data file under crates/ is added, so the records allow-list (check_record_file, AnyRecord) is not touched.
- The scan test lives in the new crate athena-testkit (tests/), scans test code in the workspace, ignores production code and the helper itself, and has a unit check that a planted violation string is detected.
- The helper crate is named athena-testkit as in the earlier candidate; the follow-up text's 'athena-testutil' is read as the same shared helper.
- The in-process variant's ETXTBSY count depends on scheduling; the test reports it and does not assert a minimum.

## Acceptance criteria

1. [content] Grep for `set_permissions` and `from_mode(0o755)` in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs finds no test code that writes an executable; only production code matches. The xtask gitleaks-version test calls athena_testkit::write_executable.
2. [content] The helper source in crates/athena-testkit/src/lib.rs spawns sh with the script bytes on stdin running cat to a temp file, chmod 755, then mv to the final path, waits for it, and never opens the script for writing in the test process.
3. [behavior] Run `cargo test --locked -p athena-testkit`: the scan test passes on the repo and a unit test shows it flags a planted in-process executable write; the stress test passes with 0 ETXTBSY for the helper, and the in-process variant's ETXTBSY count appears in the recorded output.
4. [behavior] Run `cargo test --locked` across the workspace; it passes, including the converted tests in xtask, athena-cli, athena-agent, athena-core and athena-git, and the decision-link check. `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] docs/decisions.md has an appended entry saying the decision covering crates/athena-git/src/snapshot.rs was checked and stands, and docs/decision-links.toml carries its link.
6. [content] xtask/shard-timings.txt lists every new test file or unit, and Cargo.toml and Cargo.lock list the new crate and its dependents.
7. [content] CHANGELOG.md has one entry that describes only what the final diff does. docs/architecture.md gets only the crate-table row for athena-testkit.
8. [content] The diff has no retries at run sites, no production-behavior change in claude.rs or other non-test code, no secrets or personal data, and the helper and tests are documented for an outside reader.

## Expected deliverables

- crates/athena-testkit/src/lib.rs (create)
- crates/athena-testkit/Cargo.toml (create)
- crates/athena-testkit/tests/stand_in_stress.rs (create)
- crates/athena-testkit/tests/no_in_process_exec_writes.rs (create)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- Cargo.toml (modify)
- Cargo.lock (modify)
- xtask/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-git/Cargo.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- docs/architecture.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Out of scope

- Retries at run sites or in production code
- Changing production behavior of claude.rs or other non-test code
- Non-Linux platform support
- athena.toml, defaults.toml and workflow changes
- Production-code set_permissions uses in crates/athena-core/src/{toolconfig,pin,snapshot}.rs and crates/athena-git/src/snapshot.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Helper, stress test and scan test are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New crate has passing tests; cargo test and clippy are required clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands the verifier runs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: A scan test covers every site instead of patching only the one that surfaced.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check ETXTBSY counts and the scan failing on a planted violation.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Helper and tests are documented for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule that stand-ins go through the helper is enforced by the scan test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261106-065752-jexx raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (xtask/src/main.rs:1020-1021 (test a_cached_gitleaks_is_skipped_only_when_it_reports_the_pinned_version)): One executable stand-in in a listed file still writes in-process and runs through production code, so the ETXTBSY race remains there `std::fs::write(&fake, "#!/bin/sh\necho 8.1.0\n")` followed by `set_executable(&fake)` (set_permissions 0o755 at line 809). The fake is then executed by `gitleaks_present` → `found` → `Command::new`. If ETXTBSY hits there, `found` returns None and the assertion fails. The request requires every such test to use the helper. Fix: `athena_testkit::write_executable(&fake, "#!/bin/sh\necho 8.1.0\n").unwrap();`.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

31 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-095820-edjm/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-095820-edjm/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Grep for `set_permissions` and `from_mode(0o755)` in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs finds no test code that writes an executable; only production code matches. The xtask gitleaks-version test calls athena_testkit::write_executable.
The helper source in crates/athena-testkit/src/lib.rs spawns sh with the script bytes on stdin running cat to a temp file, chmod 755, then mv to the final path, waits for it, and never opens the script for writing in the test process.
Run `cargo test --locked -p athena-testkit`: the scan test passes on the repo and a unit test shows it flags a planted in-process executable write; the stress test passes with 0 ETXTBSY for the helper, and the in-process variant's ETXTBSY count appears in the recorded output.
Run `cargo test --locked` across the workspace; it passes, including the converted tests in xtask, athena-cli, athena-agent, athena-core and athena-git, and the decision-link check. `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
docs/decisions.md has an appended entry saying the decision covering crates/athena-git/src/snapshot.rs was checked and stands, and docs/decision-links.toml carries its link.
xtask/shard-timings.txt lists every new test file or unit, and Cargo.toml and Cargo.lock list the new crate and its dependents.
CHANGELOG.md has one entry that describes only what the final diff does. docs/architecture.md gets only the crate-table row for athena-testkit.
The diff has no retries at run sites, no production-behavior change in claude.rs or other non-test code, no secrets or personal data, and the helper and tests are documented for an outside reader.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
