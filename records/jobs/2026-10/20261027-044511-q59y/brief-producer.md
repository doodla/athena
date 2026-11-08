# Job 20261027-044511-q59y: Refuse prompt edits that contradict verdict rules in verify.rs

## Request (verbatim)

> When a request or plan edits an agent prompt, the reading and plan critic are shown the verdict rules that judge that agent's output, such as the D-numbered evidence rules in verify.rs and the structured fields the judging code reads. Add a check that fails when a prompt instruction contradicts one of those rules, or when the prompt does not name a field the judging code requires. A conflict blocks dispatch until the prompt and the rule agree. Acceptance: a test shows a prompt edit that conflicts with a verdict rule is refused, with the rule named; a test shows a consistent edit is accepted; the prompt budget test and the full test suite pass.
> 
> Acceptance:
> - The change is made as described: Changes to agent instructions are not checked against the rules that judge that agent's output; conflicts show up only as verifier rejections
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: verify.rs

## Reading (accepted by the user)

Add to crates/athena-core/src/verify.rs a table of verdict rules (D-numbered evidence rules and judged output fields) plus a check that compares a proposed edit of verifier prompt text against them. A contradicting instruction, or a missing required field, is refused with the rule named. Audience: Athena maintainers whose prompt edits currently surface only as verifier rejections.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (D43, D63, D115 evidence rules; finding construction near line 601), crates/athena-context/prompts/*.md and crates/athena-context/src/briefs.rs and outputs.rs for prompt composition and structured fields. The check is a pure function over prompt text and a static rule table; it runs at reading/plan admission when the request edits a prompt file. Nothing is added as a record or data file, so check_record_file/AnyRecord are not touched. The property is measured directly by scanning the actual prompt text for contradicting phrases and required field names.
- Scope is limited to verify.rs (as the request's Files line says) plus its in-file tests; wiring into the reading and plan critic briefs is only done if a minimal hook exists already, else the function is exposed for them.
- The check uses simple phrase and field-name matching against rules registered in a table, with each rule carrying an id like D43 so the refusal names it.
- The prompt budget test is unchanged because no prompt file is edited.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/verify.rs feeds a prompt edit that contradicts a D-numbered verdict rule (e.g. telling the verifier to accept uncited commands against D43) and asserts it is refused with the rule id in the message; run `cargo test -p athena-core verify`.
2. [behavior] A new unit test feeds a consistent prompt edit and asserts it is accepted; run `cargo test -p athena-core verify`.
3. [behavior] A new test asserts a prompt that omits a field the judging code requires is refused naming the missing field; run `cargo test -p athena-core verify`.
4. [behavior] The real current prompts in crates/athena-context/prompts/ pass the new check (a test runs it over include_str! of verifier.md), so no existing behavior is refused; run `cargo test`.
5. [behavior] The change refuses more prompt edits than before and the higher refusal rate is intended: expected 5% of newly refused readings.
6. [behavior] The prompt budget test and the full suite pass, with `cargo fmt --check` and `cargo clippy` clean.
7. [content] The git diff touches only crates/athena-core/src/verify.rs and no governance files (athena.toml, defaults.toml, workflows).

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Editing any prompt file under crates/athena-context/prompts/
- Changing athena.toml, defaults.toml or workflows
- Changing existing verdict rules D43/D63/D115 behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust with Rust tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing refusal and acceptance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single-file change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
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
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

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

- path: <state>/jobs/20261027-044511-q59y/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-044511-q59y/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/verify.rs feeds a prompt edit that contradicts a D-numbered verdict rule (e.g. telling the verifier to accept uncited commands against D43) and asserts it is refused with the rule id in the message; run `cargo test -p athena-core verify`.
A new unit test feeds a consistent prompt edit and asserts it is accepted; run `cargo test -p athena-core verify`.
A new test asserts a prompt that omits a field the judging code requires is refused naming the missing field; run `cargo test -p athena-core verify`.
The real current prompts in crates/athena-context/prompts/ pass the new check (a test runs it over include_str! of verifier.md), so no existing behavior is refused; run `cargo test`.
The change refuses more prompt edits than before and the higher refusal rate is intended: expected 5% of newly refused readings.
The prompt budget test and the full suite pass, with `cargo fmt --check` and `cargo clippy` clean.
The git diff touches only crates/athena-core/src/verify.rs and no governance files (athena.toml, defaults.toml, workflows).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
