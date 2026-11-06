# Job 20261106-042920-9e47: Write test stand-in scripts via one helper that avoids ETXTBSY

## Request (verbatim)

> Stop tests that write a stand-in script and run it at once from failing with 'Text file busy'.
> 
> Several tests write a small shell script, mark it executable and run it straight away (xtask/src/main.rs stand_in_claude, and the same pattern in crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-git/src/snapshot.rs). On Linux, when another test thread forks while the script's write handle is still open, the child keeps that handle and running the script fails with ETXTBSY. These tests fail now and then under load, which rejects correct jobs; with two jobs at once on this machine it happens more often.
> 
> Write each stand-in through one shared test helper that cannot hit this: for example write to a temporary name, close it, set the mode, then rename it into place, and retry the run a few times if it still returns ETXTBSY.
> 
> Acceptance:
> - one shared helper writes executable stand-ins, and every test above uses it
> - a test runs the helper many times while other threads fork, and none of the runs fails with ETXTBSY
> - the existing tests still pass
> - CHANGELOG states only what the diff does
> 
> Files changed: the shared test helper, the tests listed above, CHANGELOG.md.

## Reading (accepted by the user)

Add one shared test-support crate with a helper that writes an executable stand-in script safely (temp name, close, chmod, rename) and a run helper that retries on ETXTBSY. Switch every listed test to it. The audience is the maintainers, whose jobs get rejected by flaky tests.

Assumptions:
- Code survey: read crates/athena-agent/src/claude.rs (stand_in ~638), crates/athena-core/src/checks.rs (~998), crates/athena-core/src/reading.rs (~8141), crates/athena-git/src/snapshot.rs (~513), crates/athena-cli/tests/reconcile.rs (~140), xtask/src/main.rs (stand_in_claude ~833), and the crates' Cargo.toml files. No records or data files are added, so check_record_file/AnyRecord are unaffected. The ETXTBSY property is measured directly by a stress test that runs the helper's script while other threads fork, not through a stand-in.
- athena-git has no athena dependencies and the other crates differ in their graphs, so a shared helper needs a new small workspace crate (e.g. crates/athena-testutil) used as a dev-dependency by athena-agent, athena-core, athena-git, athena-cli and xtask. This adds the crate to the workspace Cargo.toml members and workspace dependencies. Alternative of reusing the athena-core test-support feature is rejected because athena-git and athena-agent cannot depend on core.
- Non-test snapshot/pin code that sets permissions on materialised files is out of the pattern (no run-at-once) and stays unchanged.
- Helper API: write_executable(path, contents) writes to a temp name in the same dir, drops the handle, chmods, renames; and a retry wrapper for Command runs that retries on raw_os_error ETXTBSY (26) a few times with a short sleep. Tests that spawn through Command use the wrapper where practical.

## Acceptance criteria

1. [content] A single shared helper (new crate under crates/, listed in workspace Cargo.toml and as a dev-dependency) writes executable scripts via temp file, close, chmod, rename; Grep shows no remaining write-then-from_mode(0o755) stand-in pattern in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-git/src/snapshot.rs tests.
2. [behavior] A stress test in the helper crate writes and runs scripts many times (e.g. 200) while other threads repeatedly fork/spawn processes, and every run succeeds with no ETXTBSY; run `cargo test` and it passes.
3. [behavior] All existing tests still pass: `cargo test --workspace` passes and `cargo clippy --workspace --all-targets` is clean and `cargo fmt --check` passes.
4. [content] CHANGELOG.md has an entry describing only what the diff does (shared stand-in helper, listed tests moved onto it, ETXTBSY retry, stress test), with no unrelated claims.
5. [content] The helper has doc comments explaining the ETXTBSY cause for an outside reader, and the diff touches only the helper crate, workspace/dev-dependency manifests, the listed tests, Cargo.lock and CHANGELOG.md.

## Expected deliverables

- crates/athena-testutil/Cargo.toml (create)
- crates/athena-testutil/src/lib.rs (create)
- Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-git/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- xtask/Cargo.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Non-test code that materialises files with permissions (pin.rs, snapshot.rs non-test code, toolconfig.rs)
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Helper and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Stress test added; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion runs helper under concurrent forks and checks no ETXTBSY.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the helper and call sites.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the ETXTBSY cause.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
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

- path: <state>/jobs/20261106-042920-9e47/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-042920-9e47/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A single shared helper (new crate under crates/, listed in workspace Cargo.toml and as a dev-dependency) writes executable scripts via temp file, close, chmod, rename; Grep shows no remaining write-then-from_mode(0o755) stand-in pattern in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-git/src/snapshot.rs tests.
A stress test in the helper crate writes and runs scripts many times (e.g. 200) while other threads repeatedly fork/spawn processes, and every run succeeds with no ETXTBSY; run `cargo test` and it passes.
All existing tests still pass: `cargo test --workspace` passes and `cargo clippy --workspace --all-targets` is clean and `cargo fmt --check` passes.
CHANGELOG.md has an entry describing only what the diff does (shared stand-in helper, listed tests moved onto it, ETXTBSY retry, stress test), with no unrelated claims.
The helper has doc comments explaining the ETXTBSY cause for an outside reader, and the diff touches only the helper crate, workspace/dev-dependency manifests, the listed tests, Cargo.lock and CHANGELOG.md.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
