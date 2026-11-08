# Job 20261027-011638-zsz9: Add real-call-site test convention to the change producer brief

## Request (verbatim)

> Add a producer-brief convention: each behavior change needs at least one test that goes through the real call site (the existing fake agent driver or nested-run harness where needed) and would fail if the production call were deleted. The producer must state which test that is. Acceptance: the producer brief contains the convention and a policy test checks it is present.
> 
> Acceptance:
> - The change is made as described: Producer's new tests exercise helpers, not the changed call site; verifier finds every test would still pass if the production change were reverted
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/jobs/2026-10/20261026-170042-cgq9/verification.json

## Reading (accepted by the user)

Add a short convention to the change producer brief: every behavior change needs at least one test through the real call site (existing fake agent driver or nested-run harness) that fails if the production call is deleted, and the producer names that test in its report. A policy test in athena-cap-change checks the brief contains it. Job 20261026-170042-cgq9 (a records-branch reader change) is dropped, not continued.

Assumptions:
- Goal and audience: the change producer agent reads the brief; the verifier and maintainers judge whether the convention is present and enforced by a test.
- Code survey: read crates/athena-cap-change/src/lib.rs (producer prompt is include_str!("../prompts/producer.md") at line 21, exposed via producer_section; two existing unit tests parse athena.toml and defaults.toml) and the list of files referencing producer.md (crates/athena-context/src/briefs.rs, crates/athena-context/src/lib.rs). The new policy test goes in crates/athena-cap-change/src/lib.rs tests and asserts on the producer prompt text. No record or data file is added, so no records allow-list or AnyRecord round-trip is affected. The property is measured directly on the embedded prompt text.
- Job 20261026-170042-cgq9 (retry source, rejected, verdict fail): its findings concern reader tests that only called helpers (records_branch_readers.rs) and a decisions note about reads versus writes. They no longer apply because that records-branch reader work is dropped entirely; this job only supplies the convention that would have caught those findings. All 18 files its candidate changed are excluded in out_of_scope.
- The request's 'Files:' entry names records/jobs/.../verification.json; records/ never changes, so it is read context only.
- Token headroom: producer (change) has 146 tokens left, so the addition is kept to about 60 tokens in crates/athena-cap-change/prompts/producer.md, the only target whose sets have room (it is not composed into the verifier or reader).
- No governance files (athena.toml, defaults.toml, workflows) need to change.

## Acceptance criteria

1. [content] Grep of crates/athena-cap-change/prompts/producer.md finds text requiring, for each behavior change, at least one test through the real call site (naming the fake agent driver or nested-run harness) that would fail if the production call were deleted.
2. [content] Grep of crates/athena-cap-change/prompts/producer.md finds an instruction that the producer must name which test that is in its report.
3. [content] A test in crates/athena-cap-change/src/lib.rs asserts the embedded producer prompt contains the convention's key phrases (real call site, would fail if the production call were deleted, name the test); Grep finds it and it asserts on the producer prompt text, not a copy of the string.
4. [behavior] Run `cargo test -p athena-cap-change`: the new policy test passes, and its assertions would fail if the convention paragraph were removed from producer.md (checkable by reading the diff's assertions against the paragraph).
5. [behavior] Run `cargo test`: the whole suite passes, including any existing token-budget or prompt tests, so the producer (change) prompt stays within its 1300-token budget.
6. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets`: both are clean.
7. [content] `git diff --stat` shows changes only to crates/athena-cap-change/prompts/producer.md and crates/athena-cap-change/src/lib.rs (plus CHANGELOG.md if the repository convention requires it); no unrelated behavior changes.

## Expected deliverables

- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Everything job 20261026-170042-cgq9's candidate changed is dropped, not kept: CHANGELOG.md records-branch entry, crates/athena-cli/src/backtest.rs, crates/athena-cli/tests/records_branch_readers.rs, crates/athena-core/src/{auditor,gh,introspect,manage,proposals,reading,run,status,store}.rs, crates/athena-data/src/{lib,paths}.rs, crates/athena-gate/src/records.rs, docs/decisions.md D134, xtask/src/main.rs, xtask/src/walltime.rs.
- Reworking records-branch readers or writers, or the cutover.
- Changing the study producer or verifier prompts, or any checker that enforces the convention on future diffs.
- Editing anything under records/.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A policy test is added and cargo test, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the convention text and the test's failure when the text is removed, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt paragraph plus one test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The convention is enforced by a policy test on the prompt text.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a producer brief lacking the convention fails the test.
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

- path: <state>/jobs/20261027-011638-zsz9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-011638-zsz9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Grep of crates/athena-cap-change/prompts/producer.md finds text requiring, for each behavior change, at least one test through the real call site (naming the fake agent driver or nested-run harness) that would fail if the production call were deleted.
Grep of crates/athena-cap-change/prompts/producer.md finds an instruction that the producer must name which test that is in its report.
A test in crates/athena-cap-change/src/lib.rs asserts the embedded producer prompt contains the convention's key phrases (real call site, would fail if the production call were deleted, name the test); Grep finds it and it asserts on the producer prompt text, not a copy of the string.
Run `cargo test -p athena-cap-change`: the new policy test passes, and its assertions would fail if the convention paragraph were removed from producer.md (checkable by reading the diff's assertions against the paragraph).
Run `cargo test`: the whole suite passes, including any existing token-budget or prompt tests, so the producer (change) prompt stays within its 1300-token budget.
Run `cargo fmt --check` and `cargo clippy --all-targets`: both are clean.
`git diff --stat` shows changes only to crates/athena-cap-change/prompts/producer.md and crates/athena-cap-change/src/lib.rs (plus CHANGELOG.md if the repository convention requires it); no unrelated behavior changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
