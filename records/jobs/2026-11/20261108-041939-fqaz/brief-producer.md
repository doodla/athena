# Job 20261108-041939-fqaz: Run cargo fmt on the producer's work tree before checks and verifier

## Request (verbatim)

> try again on 20261108-023649-67f6

## Reading (accepted by the user)

Follow-up to rejected job 20261108-023649-67f6: its skill work is dropped and not retried here. This job does the fix its introspection (20261108-041919-1x5m) proposed: after the producer stage, Athena runs `cargo fmt` on the work tree and then `cargo fmt --check`, so a formatting-only defect no longer rejects a job. Audience: Sri and the verifier judging whether jobs are rejected for work rather than layout.

Assumptions:
- Code survey: read crates/athena-core/src/run/producer.rs (producer_snapshot creates the work tree; producer_loop runs the producer, then send-backs for criteria, scope, mirrors and decisions), crates/athena-core/src/run/mod.rs (produce_and_verify at about line 1737; checks run through routing::run_routed_with_record and SnapExec near line 2021, on the candidate), crates/athena-core/src/checks.rs (run_check) and crates/athena-agent/src/policy.rs (denies cargo fmt without --check to agent roles). The format step belongs at the end of producer_loop, before the change list is taken. No record or data file under crates/ is added, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly: the fmt check on the formatted tree.
- Job 20261108-023649-67f6 addressed: its skill work (skills/diagnose-failure.md, introspect.rs and briefs.rs changes, its introspection test) is dropped, not kept. Its findings are not fixed here. The fmt finding is fixed at the root by this mechanism. The skill-path and PromptAsset-versioning findings do not apply because the skill is not re-added. A separate request can redo the skill.
- Where a candidate is tree-seeded from an earlier rejected one, the formatter runs on it too, since the work tree is formatted after each producer stage.
- Unfixable fmt errors (a parse error) are left in the tree; the normal fmt check fails and reports them.
- The base and any pre-existing unformatted files can be reformatted by `cargo fmt`; accepted because the repository is expected to be fmt-clean at base.
- Governance: athena.toml, defaults.toml, workflows, rust-toolchain.toml and rustfmt config are not changed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core builds a work tree whose Rust file is mis-formatted (e.g. a long constant on one line), runs the new post-producer format step, and asserts the file is rewritten and `cargo fmt --check` then passes. Run `cargo test -p athena-core` and it passes.
2. [behavior] A new test shows a fmt failure the formatter cannot fix (e.g. a file that does not parse) still yields a failed fmt result whose output text appears in the failed-check output. Run `cargo test -p athena-core`.
3. [content] Grep of crates/athena-core/src/run shows the format step is called after the producer loop and before the checks and verifier run, so the check and the verifier see the formatted tree.
4. [behavior] A test asserts the recorded candidate diff (change list) includes the formatter's changes, not only the producer's own edits. Run `cargo test -p athena-core`.
5. [content] The format step runs only for a work tree with a Cargo.toml and is Athena code, not an agent command, so the policy in crates/athena-agent/src/policy.rs that denies `cargo fmt` without --check to agent roles is unchanged.
6. [content] CHANGELOG.md has an Unreleased entry stating the outcome: a job whose only defect is rustfmt layout reaches the verifier formatted.
7. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean on the workspace.
8. [content] Job 20261108-023649-67f6's candidate files are not present: crates/athena-context/skills/diagnose-failure.md does not exist, and crates/athena-core/src/introspect.rs and crates/athena-cli/tests/introspection.rs are unchanged. No secrets or personal data are added.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/format.rs (create)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Job 20261108-023649-67f6 files, dropped: crates/athena-context/skills/diagnose-failure.md, crates/athena-core/src/introspect.rs, crates/athena-cli/tests/introspection.rs, and its CHANGELOG entry
- Adding the diagnose-failure skill or any other skill
- Changing athena.toml, defaults.toml, workflows or rustfmt/toolchain configuration
- Auto-fixing clippy or test failures
- Letting agent roles run cargo fmt without --check
- Relaxing the fmt gate

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the format step; cargo test, clippy and fmt --check are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Formatting is fixed by a mechanism in the pipeline, not by reformatting one file.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A code step enforces it, not a prompt rule.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check that a mis-formatted tree is reformatted and that an unfixable error still fails.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused pipeline step.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261108-023649-67f6 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/introspect.rs:1029): cargo fmt --check fails on the new constant `cargo fmt --all --check` exits 1. rustfmt wants `pub const DIAGNOSE_FAILURE_SKILL: &str =` split from the `include_str!(...)` on the next line. The recorded CI fmt check also failed. This violates criterion 3 and the 'tested' default.

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

- path: <state>/jobs/20261108-041939-fqaz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-041939-fqaz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-core builds a work tree whose Rust file is mis-formatted (e.g. a long constant on one line), runs the new post-producer format step, and asserts the file is rewritten and `cargo fmt --check` then passes. Run `cargo test -p athena-core` and it passes.
A new test shows a fmt failure the formatter cannot fix (e.g. a file that does not parse) still yields a failed fmt result whose output text appears in the failed-check output. Run `cargo test -p athena-core`.
Grep of crates/athena-core/src/run shows the format step is called after the producer loop and before the checks and verifier run, so the check and the verifier see the formatted tree.
A test asserts the recorded candidate diff (change list) includes the formatter's changes, not only the producer's own edits. Run `cargo test -p athena-core`.
The format step runs only for a work tree with a Cargo.toml and is Athena code, not an agent command, so the policy in crates/athena-agent/src/policy.rs that denies `cargo fmt` without --check to agent roles is unchanged.
CHANGELOG.md has an Unreleased entry stating the outcome: a job whose only defect is rustfmt layout reaches the verifier formatted.
`cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean on the workspace.
Job 20261108-023649-67f6's candidate files are not present: crates/athena-context/skills/diagnose-failure.md does not exist, and crates/athena-core/src/introspect.rs and crates/athena-cli/tests/introspection.rs are unchanged. No secrets or personal data are added.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
