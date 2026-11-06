# Job 20261106-104038-cnaa: Add shared child-process script writer; use it in all stand-in tests

## Request (verbatim)

> Make tests that write and run stand-in scripts immune to 'Text file busy', with one shared helper and no source scan.
> 
> Tests write small executable stand-in scripts and run them, sometimes through production code. On Linux, if another test thread forks while this process holds the script open for writing, running it fails with ETXTBSY. This has failed correct jobs.
> 
> Add one shared test helper that writes the script's bytes from a short-lived child process (for example sh -c 'cat > tmp && chmod 755 tmp && mv tmp final', bytes on stdin) and waits for it, so the test process never holds a write handle. Make every test that writes an executable stand-in use it.
> 
> Do not add a test that scans source text for patterns; reviewers found such scans miss cases. List each converted call site in the job report instead.
> 
> Acceptance:
> - the helper exists, and every test in the workspace that writes a file and then makes it executable (by set_permissions, from_mode, set_mode, a wrapper such as xtask's set_executable, or an external chmod) uses it; the report lists each one by file and test name
> - a stress test runs many write-then-exec cycles through the helper while other threads fork, and passes
> - if a converted file is governed by a decision in docs/decisions.md, an appended entry says it was checked and stands, with its link
> - any new test file is listed in xtask/shard-timings.txt
> - all tests pass; CHANGELOG states only what the diff does
> 
> Files changed: the shared helper, the tests that write stand-ins, docs/decisions.md and docs/decision-links.toml if needed, xtask/shard-timings.txt if needed, CHANGELOG.md.

## Reading (accepted by the user)

Add one shared test helper that writes an executable script via a short-lived `sh -c 'cat > tmp && chmod 755 tmp && mv tmp final'` child (bytes on stdin, waited on), so no test process holds a write handle. Convert every test that writes a file then makes it executable, add a fork-stress test, and record the D08 check for the governed athena-git snapshot.rs. The audience is maintainers whose correct jobs fail on ETXTBSY.

Assumptions:
- Code survey: I grepped all *.rs under crates/ and xtask/. Test sites that write a file and then chmod: crates/athena-core/src/reading.rs (~8185, in the stub checker test helper next to check_stub's 'Text file busy' retry), crates/athena-core/src/checks.rs (~1009), crates/athena-agent/src/claude.rs (~641, near retry_busy), crates/athena-cli/tests/reconcile.rs (~140, fake gh), crates/athena-git/src/snapshot.rs (~514, test user_repo, run.sh fixture), xtask/src/main.rs (tests at ~837 and ~1023 via set_executable). The producer must re-grep to find any missed site, including ones that use set_executable or an external chmod.
- Code survey: Non-test uses stay unchanged: crates/athena-core/src/toolconfig.rs, pin.rs and crates/athena-core/src/snapshot.rs set modes for production, and xtask set_executable at ~415, ~490 and ~550 is production code. crates/athena-agent/src/policy.rs and crates/athena-cap-change/src/lib.rs only mention chmod as a policy string. Production set_executable in xtask can stay; only the xtask test sites change.
- Code survey: Existing test-sharing mechanisms: athena-core has a `test-support` feature (crates/athena-core/src/test_support.rs, lib.rs:120) that athena-cli enables, and athena-git has `pub(crate) mod testutil` (lib.rs:1093). The helper must be reachable from athena-core, athena-agent, athena-git, athena-cli tests and xtask. The producer chooses the home, for example a `test-support` feature in athena-git (which xtask already depends on) or a small dev-only crate. It must be a plain Rust fn that takes a path and bytes. No records or data file is added under crates/, so check_record_file and AnyRecord are not involved.
- Code survey: The change adds no refusal rule, so no refusal-rate criterion applies. The property is measured directly: the stress test runs the helper's write-then-exec cycles while other threads fork, and the existing sleep-retry code in reading.rs check_stub and claude.rs retry_busy is not the measure. Retry code in production, such as retry_busy in claude.rs, is kept unless it is test-only.
- docs/decision-links.toml has a [[governs]] D08 entry covering crates/athena-git/src/snapshot.rs. Converting the test there triggers the D156 reconciliation, so a new entry is appended to docs/decisions.md saying D08 was checked and stands, plus an `amended` [[link]] in docs/decision-links.toml whose line matches the new phrase. Other converted files are not in a [[governs]] table, which the producer should recheck.
- A new stress test is placed inside the helper's own crate or an existing test module, so no new test file is made and xtask/shard-timings.txt stays unchanged. If a new test file is made instead, it is added there.
- Cargo.toml dev-dependency or feature edits for sharing the helper are permitted, but no lint, toolchain or other governance config changes.

## Acceptance criteria

1. [content] A single shared helper exists that writes script bytes by spawning a child process (sh with bytes on stdin: cat to a tmp file, chmod 755, mv into place) and waits for it; the test process never opens the script for writing. Confirm by reading it.
2. [content] Every test in the workspace that writes a file and then makes it executable (set_permissions, from_mode, set_mode, xtask set_executable, or an external chmod) calls the helper. This covers at least reading.rs, checks.rs, claude.rs, reconcile.rs, athena-git snapshot.rs and the xtask tests. Grep of test code finds no remaining write-then-chmod pairs. Production set_executable uses are untouched.
3. [content] A stress test runs many write-then-exec cycles through the helper while other threads fork, and asserts every exec succeeds.
4. [behavior] Run `cargo test` for the workspace. All tests pass, including the stress test and every converted test. Run `cargo clippy` (clean) and `cargo fmt --check`.
5. [content] No new test scans source text for patterns. The change adds no source-scanning test.
6. [content] docs/decisions.md has an appended entry saying D08 was checked for the crates/athena-git/src/snapshot.rs conversion and stands. docs/decision-links.toml has a matching `amended` [[link]] with older D08 and the new entry as newer. Its `line` matches the entry's phrase line in docs/decisions.md. Any other governed converted file is handled the same way.
7. [content] Any new test file is listed in xtask/shard-timings.txt; otherwise that file is unchanged.
8. [content] CHANGELOG.md has an entry that states only what the diff does: the helper, the converted tests and the stress test.
9. [content] The job report lists each converted call site by file and test name. Because the reader checks only the diff, the verifier compares that list with a Grep of the diff.

## Expected deliverables

- crates/athena-git/src/lib.rs (modify)
- crates/athena-git/src/script_writer.rs (create)
- crates/athena-git/Cargo.toml (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-git/src/snapshot.rs (modify)
- xtask/src/main.rs (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Out of scope

- A test that scans source text for patterns
- Changing production code paths (toolconfig.rs, pin.rs, core snapshot.rs, xtask production set_executable)
- Removing the existing ETXTBSY retry code unless it is test-only and made redundant
- Changes to athena.toml, defaults.toml, workflows or lint configuration

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The helper and stress test are Rust; the helper only shells out to sh as the request requires.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The stress test and cargo test, clippy and fmt must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A behavior criterion runs cargo test, clippy and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The stress test exercises write-then-exec under concurrent forks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The helper carries a doc comment explaining ETXTBSY and why the child writes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One helper and mechanical conversions only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that exec succeeds under concurrent forks, not that the helper exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

32 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-104038-cnaa/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-104038-cnaa/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A single shared helper exists that writes script bytes by spawning a child process (sh with bytes on stdin: cat to a tmp file, chmod 755, mv into place) and waits for it; the test process never opens the script for writing. Confirm by reading it.
Every test in the workspace that writes a file and then makes it executable (set_permissions, from_mode, set_mode, xtask set_executable, or an external chmod) calls the helper. This covers at least reading.rs, checks.rs, claude.rs, reconcile.rs, athena-git snapshot.rs and the xtask tests. Grep of test code finds no remaining write-then-chmod pairs. Production set_executable uses are untouched.
A stress test runs many write-then-exec cycles through the helper while other threads fork, and asserts every exec succeeds.
Run `cargo test` for the workspace. All tests pass, including the stress test and every converted test. Run `cargo clippy` (clean) and `cargo fmt --check`.
No new test scans source text for patterns. The change adds no source-scanning test.
docs/decisions.md has an appended entry saying D08 was checked for the crates/athena-git/src/snapshot.rs conversion and stands. docs/decision-links.toml has a matching `amended` [[link]] with older D08 and the new entry as newer. Its `line` matches the entry's phrase line in docs/decisions.md. Any other governed converted file is handled the same way.
Any new test file is listed in xtask/shard-timings.txt; otherwise that file is unchanged.
CHANGELOG.md has an entry that states only what the diff does: the helper, the converted tests and the stress test.
The job report lists each converted call site by file and test name. Because the reader checks only the diff, the verifier compares that list with a Grep of the diff.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
