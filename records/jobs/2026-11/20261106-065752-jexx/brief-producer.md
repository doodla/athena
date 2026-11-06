# Job 20261106-065752-jexx: Write test stand-in scripts from a child process to avoid ETXTBSY

## Request (verbatim)

> Make the shared stand-in script helper immune to 'Text file busy' without retries at the call sites.
> 
> Tests write small executable stand-in scripts and then run them, sometimes through production code. On Linux, if another test thread forks while this process holds the script open for writing, running the script fails with ETXTBSY. A retry at each run site is not enough, because many runs happen inside production code.
> 
> Write the script's bytes from a child process instead: the helper starts a short-lived child (for example sh -c 'cat > tmp && chmod 755 tmp && mv tmp final', with the bytes on stdin) and waits for it. The test process never holds a write handle to the script, so no fork in another thread can inherit one.
> 
> Acceptance:
> - every test that writes an executable stand-in uses this one helper (xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-git/src/snapshot.rs)
> - a stress test runs many write-then-exec cycles while other threads fork; it passes with the helper, and a variant that writes in-process is shown in the recorded check output to hit ETXTBSY at least once (or the test documents how often it does on this machine)
> - the decision that covers crates/athena-git/src/snapshot.rs gets an appended entry saying it was checked and stands, with its link
> - xtask/shard-timings.txt lists any new test file
> - CHANGELOG states only what the diff does
> 
> Files changed: the shared test helper, the tests above, docs/decisions.md and docs/decision-links.toml for the entry, xtask/shard-timings.txt, CHANGELOG.md.

## Reading (accepted by the user)

Add one shared test helper that writes an executable stand-in script via a short-lived `sh -c 'cat > tmp && chmod 755 tmp && mv tmp final'` child fed on stdin, so the test process never holds a write handle. Switch all six listed test sites to it, add a fork-stress test, and record the decision, shard-timing and CHANGELOG entries. Audience: developers and CI running the Linux test suite.

Assumptions:
- Code survey: read crates/athena-git/src/snapshot.rs (tests around lines 495-552, set_permissions at 438-478), crates/athena-git/src/lib.rs (testutil module at 1093), crates/athena-agent/src/claude.rs (retry helper at ~45, stand-in at ~618-645), crates/athena-core/src/checks.rs (~998-1009), crates/athena-core/src/reading.rs (~8141-8156, already matches 'Text file busy'), crates/athena-core/src/test_support.rs, xtask/src/main.rs (stand_in_claude ~833), crates/athena-cli/tests/reconcile.rs (~140). No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. Entry point is the helper called by each test; the property is measured directly by exercising real write-then-exec cycles, not through a stand-in.
- No existing helper is shared across xtask, athena-cli tests, athena-agent, athena-core and athena-git, so the shared helper needs a home every one of them can dev-depend on; I assume a small new dev-only workspace crate (or an existing one if the dependency graph allows), which means Cargo.toml and Cargo.lock change.
- The stress test lives in the helper's home (a new test file or module), forks from other threads (e.g. spawning `true`) while the main thread loops write-then-exec; the in-process variant (std::fs::write then exec) is reported with its measured ETXTBSY count on this machine, printed in test output.
- Existing ETXTBSY retry code in claude.rs and reading.rs may be removed or left; I leave production code untouched and only change test code, apart from removing now-redundant test-only retries if they sit in test modules.
- The decision covering crates/athena-git/src/snapshot.rs is D08 (docs/decision-links.toml line ~180); the appended entry says it was checked and stands, with its link.
- Non-Linux/non-unix cfg gating is kept as existing tests are unix-only.

## Acceptance criteria

1. [content] Grep for `set_permissions` and `from_mode(0o755)` in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs shows no test code writing an executable stand-in itself; each such test calls the one shared helper.
2. [content] The helper's source starts a child process (sh) with the script bytes on stdin that runs cat to a temp file, chmod 755, then mv to the final path, and waits for it; it never opens the script for writing in the test process.
3. [behavior] Run `cargo test` for the stress test: many write-then-exec cycles with other threads forking pass with the helper with zero ETXTBSY, and an in-process-write variant prints/records its ETXTBSY count on this machine in the test output (at least once, or the test documents how often it hit).
4. [behavior] `cargo test --locked` passes across the workspace, `cargo clippy` is clean and `cargo fmt --check` passes.
5. [content] docs/decisions.md has an appended entry for the decision covering crates/athena-git/src/snapshot.rs saying it was checked and stands, and docs/decision-links.toml carries its link; the repo's decision-link check passes under cargo test.
6. [content] xtask/shard-timings.txt lists any new test file or unit added by the change (and Cargo.toml/Cargo.lock list any new crate).
7. [content] CHANGELOG.md has an entry that describes only what the diff does, no more.
8. [content] No secrets, credentials or personal data are committed; the helper and test are documented for an outside reader.

## Expected deliverables

- crates/athena-testkit/src/lib.rs (create)
- crates/athena-testkit/Cargo.toml (create)
- crates/athena-testkit/tests/stand_in_stress.rs (create)
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
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Out of scope

- Retries at run sites or in production code
- Changing production behavior of claude.rs or other non-test code
- Non-Linux platform support
- athena.toml, defaults.toml and workflow changes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: stress test added; cargo test and clippy must pass
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: verifier runs cargo test and checks the recorded ETXTBSY count
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria check real write-then-exec under forking, not file existence
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fixes the cause (held write handle) instead of retrying
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one helper, mechanical call-site swaps
- strangers: Code, commits, docs, and records are written for an outside reader. Here: helper documented for outside readers
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: helper and tests in Rust
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: no secrets or personal data
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: uses std::process and sh rather than a new crate dependency for the write itself
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: the outcome is no ETXTBSY under fork stress
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

- path: <state>/jobs/20261106-065752-jexx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-065752-jexx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Grep for `set_permissions` and `from_mode(0o755)` in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs shows no test code writing an executable stand-in itself; each such test calls the one shared helper.
The helper's source starts a child process (sh) with the script bytes on stdin that runs cat to a temp file, chmod 755, then mv to the final path, and waits for it; it never opens the script for writing in the test process.
Run `cargo test` for the stress test: many write-then-exec cycles with other threads forking pass with the helper with zero ETXTBSY, and an in-process-write variant prints/records its ETXTBSY count on this machine in the test output (at least once, or the test documents how often it hit).
`cargo test --locked` passes across the workspace, `cargo clippy` is clean and `cargo fmt --check` passes.
docs/decisions.md has an appended entry for the decision covering crates/athena-git/src/snapshot.rs saying it was checked and stands, and docs/decision-links.toml carries its link; the repo's decision-link check passes under cargo test.
xtask/shard-timings.txt lists any new test file or unit added by the change (and Cargo.toml/Cargo.lock list any new crate).
CHANGELOG.md has an entry that describes only what the diff does, no more.
No secrets, credentials or personal data are committed; the helper and test are documented for an outside reader.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
