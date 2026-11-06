# Job 20261106-063000-vke2: Add shared stand-in script helper that avoids ETXTBSY; reconcile D08

## Request (verbatim)

> try again on 20261106-042920-9e47

## Reading (accepted by the user)

Retry of job 20261106-042920-9e47: add one shared test-support crate (athena-testutil) that writes executable stand-ins via temp name, close, chmod, rename and retries runs on ETXTBSY, move all listed tests onto it, and this time reconcile D08 (snapshot.rs is governed) with a new docs/decisions.md entry plus a decision-links.toml link. Audience: maintainers whose jobs are rejected by flaky tests.

Assumptions:
- Code survey: read docs/decision-links.toml header and docs/decisions.md D156 (the [[governs]] rule, which maps D08 to crates/athena-git/src/snapshot.rs and crates/athena-core/src/run/candidate.rs). Test code lives in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs. The new crate adds no record or data file, so check_record_file and AnyRecord are not touched. The property is measured directly by a stress test in crates/athena-testutil that writes and runs real scripts while other threads fork.
- Prior job 20261106-042920-9e47 failed only on its recorded finding: 'decisions not reconciled: D08 (crates/athena-git/src/snapshot.rs)'. It is addressed by adding a D157 entry in docs/decisions.md saying D08 was checked and stands (the change touches only the snapshot.rs test module, not snapshot behavior), and a [[link]] in docs/decision-links.toml with older D08, newer D157, status amended, whose line matches the phrase in the new entry (line numbers of existing links unaffected because the entry is appended at the end of the log).
- The earlier candidate's 17 changed files are kept as the starting point: all 16 earlier deliverables plus docs/architecture.md (one row for the new crate), which is now a deliverable. The only new work is the decision entry and link.
- Entry id is the next free id after D156 (assumed D157); the producer confirms the last id in docs/decisions.md.
- Scope stays on test code: non-test code in snapshot.rs, pin.rs and toolconfig.rs does not change.

## Acceptance criteria

1. [content] Grep of crates/athena-testutil/src/lib.rs shows one helper that writes an executable script via a temp file, close, chmod, then rename, and a run helper that retries on ETXTBSY; doc comments explain the ETXTBSY cause for an outside reader.
2. [content] Grep finds no write-then-from_mode(0o755) stand-in pattern left in xtask/src/main.rs (stand_in_claude), crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs or the tests of crates/athena-git/src/snapshot.rs; each calls the helper crate, which is a workspace member in Cargo.toml and a dev-dependency in the athena-agent, athena-core, athena-git, athena-cli and xtask manifests.
3. [behavior] Run `cargo test -p athena-testutil`: a stress test writes and runs at least 200 scripts while other threads repeatedly fork/spawn processes, and none of the runs fails with ETXTBSY.
4. [behavior] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check`: all pass or are clean.
5. [content] docs/decisions.md ends with a new entry that says D08 was checked and stands, and docs/decision-links.toml has a [[link]] with older = "D08" and newer = that entry's id, whose line matches the phrase in the entry, so the decisions-reconciled check passes.
6. [content] CHANGELOG.md has an entry describing only what the diff does (shared stand-in helper, listed tests moved onto it, ETXTBSY retry, stress test, D08 reconcile entry), with no unrelated claims.
7. [content] The diff touches only crates/athena-testutil, the manifests and Cargo.lock, the six listed test files, docs/architecture.md, docs/decisions.md, docs/decision-links.toml and CHANGELOG.md; athena.toml, defaults.toml and workflows are unchanged.

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
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- docs/architecture.md (modify)

## Out of scope

- Non-test code that materialises files with permissions (pin.rs, snapshot.rs non-test code, toolconfig.rs)
- Changing athena.toml, defaults.toml or workflows
- Changing D08's substance or crates/athena-core/src/run/candidate.rs
- Adding a general pre-dispatch decision-mapping step (the introspection's prevention idea)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Helper and stress test are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New crate has a passing stress test; workspace tests and clippy must pass.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Helper is a few lines of std; no crate fits the need, recorded in the helper docs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Stress test runs scripts under concurrent forks.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is no ETXTBSY failures plus the decisions check passing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the ETXTBSY cause.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One helper crate, call-site swaps, one decision entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261106-042920-9e47 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> decisions not reconciled: the change touches code that docs/decisions.md records decisions for: D08 (crates/athena-git/src/snapshot.rs). Add an entry to docs/decisions.md that amends or supersedes each decision (or says it was checked and stands), and a matching [[link]] in docs/decision-links.toml whose older is the decision and whose newer is the new entry.

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

- path: <state>/jobs/20261106-063000-vke2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-063000-vke2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Grep of crates/athena-testutil/src/lib.rs shows one helper that writes an executable script via a temp file, close, chmod, then rename, and a run helper that retries on ETXTBSY; doc comments explain the ETXTBSY cause for an outside reader.
Grep finds no write-then-from_mode(0o755) stand-in pattern left in xtask/src/main.rs (stand_in_claude), crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs or the tests of crates/athena-git/src/snapshot.rs; each calls the helper crate, which is a workspace member in Cargo.toml and a dev-dependency in the athena-agent, athena-core, athena-git, athena-cli and xtask manifests.
Run `cargo test -p athena-testutil`: a stress test writes and runs at least 200 scripts while other threads repeatedly fork/spawn processes, and none of the runs fails with ETXTBSY.
Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check`: all pass or are clean.
docs/decisions.md ends with a new entry that says D08 was checked and stands, and docs/decision-links.toml has a [[link]] with older = "D08" and newer = that entry's id, whose line matches the phrase in the entry, so the decisions-reconciled check passes.
CHANGELOG.md has an entry describing only what the diff does (shared stand-in helper, listed tests moved onto it, ETXTBSY retry, stress test, D08 reconcile entry), with no unrelated claims.
The diff touches only crates/athena-testutil, the manifests and Cargo.lock, the six listed test files, docs/architecture.md, docs/decisions.md, docs/decision-links.toml and CHANGELOG.md; athena.toml, defaults.toml and workflows are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
