# Job 20261025-154650-h9ea: Make readings state GitHub Actions cache branch scoping for CI work

## Request (verbatim)

> Change the reading role's guidance: when a request touches CI caching or CI speed, the reading must say how the GitHub Actions cache is scoped by branch (which branches can restore which entries) and must state that constraint in the criteria. Add a test or prompt fixture showing the constraint is named.

## Reading (accepted by the user)

Add one rule to the reader prompt (reader.md): when a request touches CI caching or CI speed, the reading explains how the GitHub Actions cache is scoped by branch and states that constraint in an acceptance criterion. Add a unit test in athena-context showing the prompt names it. Audience: the maintainer reviewing readings. The reader set has only 3 tokens of headroom, so the producer must trim existing reader.md wording to fit within the unchanged 2000-token budget.

Assumptions:
- Code survey: read crates/athena-context/src/lib.rs (READER_FRAME includes ../prompts/reader.md; BudgetSet::Reader budget is 2_000; existing tests compose frames and call check_budget) and crates/athena-core/src/run_pin_tests.rs and crates/athena-cli/tests/budgets.rs (reader budget line). The change adds no record or data file, so check_record_file and AnyRecord are not involved. The property is measured directly on READER_FRAME.text, not a stand-in.
- The reader set has 3 tokens left, so the new bullet (about 40-60 tokens) requires trimming equivalent tokens from reader.md; raising the budget is not assumed.
- Cache scoping facts (GitHub docs): restore from current branch, base branch, default branch; not from sibling branches.
- The 'fixture' requested is satisfied by a unit test on the prompt text.

## Acceptance criteria

1. [content] crates/athena-context/prompts/reader.md has a bullet saying that when a request touches CI caching or CI speed, the reading states how the GitHub Actions cache is scoped by branch and puts that constraint in an acceptance criterion.
2. [content] The bullet states the scoping correctly: a run restores caches from its own branch (or PR merge ref), its base branch, and the default branch; it cannot restore caches from sibling or unrelated branches; caches saved by a feature branch are not visible to the default branch or other branches.
3. [behavior] A new test in crates/athena-context/src/lib.rs asserts READER_FRAME.text names the CI cache, branch scoping, base/default branch and acceptance criterion. Run `cargo test -p athena-context`: it passes, and it would fail if the bullet were removed.
4. [behavior] The reader token budget is unchanged at 2000 (BudgetSet::Reader in crates/athena-context/src/lib.rs); existing wording is tightened so the existing budget tests still pass. Run `cargo test -p athena-context` and `cargo test -p athena-cli --test budgets`.
5. [behavior] No other prompt set exceeds its budget and the whole workspace is healthy. Run `cargo test`, `cargo clippy --all-targets` (clean) and `cargo fmt --check`.
6. [content] Governance files (athena.toml, defaults.toml, workflows) are not changed; the existing guidance in reader.md is trimmed only in wording, not in meaning.

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing CI workflows or actual cache configuration
- Raising token budgets in athena.toml or lib.rs
- Changing other prompt files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts the prompt wording is present.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One bullet plus one test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Test written in Rust.
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

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261025-154650-h9ea/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261025-154650-h9ea/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/prompts/reader.md has a bullet saying that when a request touches CI caching or CI speed, the reading states how the GitHub Actions cache is scoped by branch and puts that constraint in an acceptance criterion.
The bullet states the scoping correctly: a run restores caches from its own branch (or PR merge ref), its base branch, and the default branch; it cannot restore caches from sibling or unrelated branches; caches saved by a feature branch are not visible to the default branch or other branches.
A new test in crates/athena-context/src/lib.rs asserts READER_FRAME.text names the CI cache, branch scoping, base/default branch and acceptance criterion. Run `cargo test -p athena-context`: it passes, and it would fail if the bullet were removed.
The reader token budget is unchanged at 2000 (BudgetSet::Reader in crates/athena-context/src/lib.rs); existing wording is tightened so the existing budget tests still pass. Run `cargo test -p athena-context` and `cargo test -p athena-cli --test budgets`.
No other prompt set exceeds its budget and the whole workspace is healthy. Run `cargo test`, `cargo clippy --all-targets` (clean) and `cargo fmt --check`.
Governance files (athena.toml, defaults.toml, workflows) are not changed; the existing guidance in reader.md is trimmed only in wording, not in meaning.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
