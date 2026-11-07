# Job 20261106-194635-cg5z: Add athena-testkit write_executable helper and exec-write scan test

## Request (verbatim)

> try again on 20261106-104023-w7my

## Reading (accepted by the user)

Retry of job 20261106-104023-w7my, which was refused at admission only because crates/athena-testkit/Cargo.toml (a governance file with [lints]) was not named in a criterion. Same plan: one test-only helper writing executable stand-ins via an sh child, all test sites converted, plus a scan test; now with the Cargo.toml named in a criterion.

Assumptions:
- Prior job 20261106-104023-w7my: its only recorded finding was the admission refusal for crates/athena-testkit/Cargo.toml as an unlisted-in-criteria governance file. It is fixed by listing that file as a deliverable and naming it in a criterion. The candidate's 25 file changes are all kept as deliverables, including the deletion of crates/athena-git/src/script_writer.rs.
- Code survey: read crates/athena-git/src/snapshot.rs (script_writer use at ~513; production set_permissions at 445-478), crates/athena-git/src/lib.rs (pub mod script_writer), crates/athena-core/src/checks.rs (~1003), crates/athena-core/src/reading.rs (~8217), crates/athena-agent/src/claude.rs (~639), crates/athena-cli/tests/reconcile.rs (~134), crates/athena-cli/tests/slot_exec.rs (~54, a raw in-process set_permissions 0o755). Production uses in crates/athena-core/src/{toolconfig,pin,snapshot}.rs and crates/athena-cli/src/slot_exec.rs are not stand-in writes. The change adds no record or data file under crates/ and no refusal rule on readings.
- Existing code: crates/athena-git/src/script_writer.rs, crates/athena-git/src/lib.rs and docs/architecture.md are extended or replaced by the new crate. The existing helper is moved to athena-testkit.
- Governance: the new crate's Cargo.toml follows the workspace [lints] convention. Cargo.toml, Cargo.lock and the other crates' Cargo.toml files change only to add the dev-dependency and workspace member. athena.toml, defaults.toml and workflows do not change.
- Audience: maintainers running cargo test, who need flaky ETXTBSY failures gone and a mechanical guard against new in-process executable writes.

## Acceptance criteria

1. [content] crates/athena-testkit/Cargo.toml exists as a std-only dev-dependency crate with a workspace [lints] table, and is a workspace member in Cargo.toml. CHANGELOG.md names the root cause: executable stand-ins written in-process with no single helper and no mechanical enumeration of write sites. It also names the fix at the origin: athena_testkit::write_executable, which writes via an sh child (cat > tmp && chmod 755 tmp && mv tmp final).
2. [content] Grep of test code (xtask/src/main.rs, crates/athena-cli/tests/*.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/{checks,reading}.rs, crates/athena-git/src/snapshot.rs tests) finds no in-process executable write. The xtask gitleaks fake and crates/athena-cli/tests/slot_exec.rs use the helper, and crates/athena-git/src/script_writer.rs is deleted with no remaining references.
3. [behavior] Run `cargo test -p athena-testkit`. The scan test no_in_process_exec_writes passes on the tree. Its planted-violation unit tests show it fails on set_executable(&fake) wrappers, set_permissions with 0o755, from_mode and Command::new("chmod") in test code. These tests would fail without the detection logic.
4. [behavior] Run `cargo test -p athena-testkit --test stand_in_stress`. The helper variant records 0 ETXTBSY over many write-then-exec cycles with concurrent forking threads. The in-process variant's hit count appears in the recorded test output.
5. [content] git diff shows no non-test production change. The busy-file retry in reading.rs check_stub is removed, and production retry_busy in claude.rs is untouched.
6. [content] docs/decisions.md and docs/decision-links.toml have an appended entry saying the decision covering crates/athena-git/src/snapshot.rs was checked and stands. xtask/shard-timings.txt lists the new test files. docs/architecture.md's crate table lists athena-testkit.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`. All pass.

## Expected deliverables

- crates/athena-testkit/Cargo.toml (create)
- crates/athena-testkit/src/lib.rs (create)
- crates/athena-testkit/tests/stand_in_stress.rs (create)
- crates/athena-testkit/tests/no_in_process_exec_writes.rs (create)
- Cargo.toml (modify)
- Cargo.lock (modify)
- xtask/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-git/Cargo.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-cli/tests/slot_exec.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-git/src/script_writer.rs (delete)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
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
- Production-code set_permissions uses in crates/athena-core/src/{toolconfig,pin,snapshot}.rs, crates/athena-cli/src/slot_exec.rs and crates/athena-git/src/snapshot.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The testkit has unit and integration tests, and the workspace test and clippy criterion is included.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is one helper plus a scan, not per-site patches.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the scan against planted violations and the stress test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The scan test enforces the rule mechanically.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The stress test measures ETXTBSY counts directly.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
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

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

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

- path: <state>/jobs/20261106-194635-cg5z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-194635-cg5z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-testkit/Cargo.toml exists as a std-only dev-dependency crate with a workspace [lints] table, and is a workspace member in Cargo.toml. CHANGELOG.md names the root cause: executable stand-ins written in-process with no single helper and no mechanical enumeration of write sites. It also names the fix at the origin: athena_testkit::write_executable, which writes via an sh child (cat > tmp && chmod 755 tmp && mv tmp final).
Grep of test code (xtask/src/main.rs, crates/athena-cli/tests/*.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/{checks,reading}.rs, crates/athena-git/src/snapshot.rs tests) finds no in-process executable write. The xtask gitleaks fake and crates/athena-cli/tests/slot_exec.rs use the helper, and crates/athena-git/src/script_writer.rs is deleted with no remaining references.
Run `cargo test -p athena-testkit`. The scan test no_in_process_exec_writes passes on the tree. Its planted-violation unit tests show it fails on set_executable(&fake) wrappers, set_permissions with 0o755, from_mode and Command::new("chmod") in test code. These tests would fail without the detection logic.
Run `cargo test -p athena-testkit --test stand_in_stress`. The helper variant records 0 ETXTBSY over many write-then-exec cycles with concurrent forking threads. The in-process variant's hit count appears in the recorded test output.
git diff shows no non-test production change. The busy-file retry in reading.rs check_stub is removed, and production retry_busy in claude.rs is untouched.
docs/decisions.md and docs/decision-links.toml have an appended entry saying the decision covering crates/athena-git/src/snapshot.rs was checked and stands. xtask/shard-timings.txt lists the new test files. docs/architecture.md's crate table lists athena-testkit.
Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`. All pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
