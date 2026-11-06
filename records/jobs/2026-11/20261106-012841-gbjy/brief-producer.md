# Job 20261106-012841-gbjy: Link decisions to governed code and check reversals before verification

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The decision log is prose, and nothing links a decision to the code or behavior it governs. The prod
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: behavior-change-leaves-contradicting-decision-record-caught-only-at-final-verdict (20261104-153440-xf17)
> Cause: The decision log is prose, and nothing links a decision to the code or behavior it governs. The producer has no step that searches the decisions for statements the change reverses. The reading did list a docs criterion, but it gave the prod...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261104-153423-esay (job 20261010-225741-yvk0); prevention it proposed: Add a check that runs before verification. Every decision in docs/decisions.md that a change reverses must either be marked "amended by" or "superseded by" a later decision, or be reworded. The check works from a small machine-readable list...
> Introspection 20261106-012825-ay9k (job 20261105-180339-w5h7); prevention it proposed: Build the help text from the same constant that sets the ceiling, so it cannot go stale. Add a test that runs `athena drain --help` and asserts it shows the current ceiling value. For other limit changes, run a check that greps the repo (co...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: docs/decisions.md is prose and nothing ties a decision to the paths or limits it governs, so a change that reverses a decision (job 20261010-225741-yvk0: D08) or a stated limit (job 20261105-180339-w5h7: drain help "at most 8") is caught only at the final verdict. Fix: add a machine-readable `governs` list beside the existing docs/decision-links.toml, and a check that runs before verification and fails when a change touches governed code without amending or superseding the decision. Derive the drain help ceiling from MAX_CONCURRENCY_CEILING.

Assumptions:
- Code survey: read crates/athena-cli/src/decisions.rs (DecisionView, ConfirmedLinks parsed from docs/decision-links.toml, LinkStatus Superseded/Amended, tied to crates/athena-cli/src/backtest.rs parse_supersessions) and crates/athena-core/src/run/checks.rs and run/mod.rs check_and_verify (the checks that run before verification). The new check is added where checks run before the verifier; the governs list is read the same way as ConfirmedLinks. docs/decision-links.toml is a docs file, not a crates/ record, so no AnyRecord or check_record_file change is expected; the existing decision-links round-trip tests are extended.
- Goal and audience: the maintainer and the producer. The producer gets the failure at the producer step (a finding listing the decision ids the change reverses) instead of the verifier finding it at the end.
- Design: a `[[governs]]` table (decision id, path or pattern) in docs/decision-links.toml. When a diff touches a governed path and docs/decisions.md has no new 'Amended by'/'Superseded by' link for that id or a reworded entry, the check fails and names the decision id. Chosen over an LLM judgment because it is mechanical and testable.
- Second root-cause part: a stated limit in help or docs must come from the constant. The `athena drain --help` ceiling text in crates/athena-cli/src/main.rs is built from MAX_CONCURRENCY_CEILING (athena-data config.rs), so a change to the constant cannot leave it stale.
- Job 20261010-225741-yvk0: its candidate (git-history snapshot ATHENA-COMMITS.txt) was rejected. This job addresses the D08 finding by making that class mechanical: the check would have flagged D08. The commit-visibility work itself is dropped, not re-done; its changed files (CHANGELOG.md earlier entry, briefs.rs, run/mod.rs, producer.rs, verification.rs, run_pin_tests.rs, athena-git snapshot.rs and lib.rs, docs/enforcement.md) are excluded in out_of_scope.
- Job 20261105-180339-w5h7: its candidate (ceiling 16, athena.toml 12, group_sizes) was rejected for the stale main.rs help. This job addresses that finding by deriving the help text from the constant, with a test. The candidate's other changes (config.rs, drain.rs, display.rs, tests/drain.rs, athena.toml, design doc) are excluded in out_of_scope and not re-applied here.
- Deliverables of the change: modify crates/athena-cli/src/decisions.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, docs/decision-links.toml, docs/decisions.md, CHANGELOG.md. Governance files athena.toml, defaults.toml and workflows are not changed: the check runs in the existing pipeline.
- Standing defaults applied: tested, mechanism, root-cause, behavior-evidence, outcome-check, small, strangers.

## Acceptance criteria

1. [content] CHANGELOG.md and the new decisions.md entry name the root cause: the decision log is prose with no link from a decision to the code or limit it governs, so reversals were found only at the final verdict.
2. [content] docs/decision-links.toml holds a `[[governs]]` list with at least D08 mapped to the snapshot/history code paths, and ConfirmedLinks parsing in crates/athena-cli/src/decisions.rs reads and validates it (unknown decision ids are an error).
3. [behavior] Run `cargo test`: a new test builds a diff that touches a governed path with no amend/supersede link and asserts the pre-verification check fails naming the decision id; the same diff with 'Amended by' added passes. The test fails when the check is removed, and uses a decision other than D08 as well so it covers the class.
4. [behavior] Run `cargo test`: a test renders the `athena drain` help and asserts it contains the current MAX_CONCURRENCY_CEILING value; it fails if the text is hard-coded and the constant changes.
5. [content] crates/athena-core/src/run/mod.rs calls the new check before the verifier is started (Grep check_and_verify), so a failure is returned to the producer as a finding.
6. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.
7. [behavior] Expected 5% of newly refused readings: the new check refuses more changes (those touching governed paths without a link); a higher refusal rate on past governed-path changes is intended, because those are exactly the unreconciled reversals.

## Expected deliverables

- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/decision-links.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Re-doing the dropped candidate of job 20261010-225741-yvk0: ATHENA-COMMITS.txt, crates/athena-git/src/snapshot.rs, crates/athena-git/src/lib.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/run_pin_tests.rs, docs/enforcement.md
- Re-doing the dropped candidate of job 20261105-180339-w5h7: ceiling 16, athena.toml max_concurrency, group_sizes, crates/athena-cli/src/drain.rs, display.rs, tests/drain.rs, docs/design/agent-and-build-slots.md
- Editing athena.toml, defaults.toml or workflows
- Checking prose claims in design docs beyond the governed list
- Changing records/ or adding a record type

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the missing decision-to-code link, not the two stale instances.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A check enforces it before verification.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests plus fmt, clippy, test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on the check's pass and fail.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that reversals fail early.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused mechanism.
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

29 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-012841-gbjy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-012841-gbjy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md and the new decisions.md entry name the root cause: the decision log is prose with no link from a decision to the code or limit it governs, so reversals were found only at the final verdict.
docs/decision-links.toml holds a `[[governs]]` list with at least D08 mapped to the snapshot/history code paths, and ConfirmedLinks parsing in crates/athena-cli/src/decisions.rs reads and validates it (unknown decision ids are an error).
Run `cargo test`: a new test builds a diff that touches a governed path with no amend/supersede link and asserts the pre-verification check fails naming the decision id; the same diff with 'Amended by' added passes. The test fails when the check is removed, and uses a decision other than D08 as well so it covers the class.
Run `cargo test`: a test renders the `athena drain` help and asserts it contains the current MAX_CONCURRENCY_CEILING value; it fails if the text is hard-coded and the constant changes.
crates/athena-core/src/run/mod.rs calls the new check before the verifier is started (Grep check_and_verify), so a failure is returned to the producer as a finding.
Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.
Expected 5% of newly refused readings: the new check refuses more changes (those touching governed paths without a link); a higher refusal rate on past governed-path changes is intended, because those are exactly the unreconciled reversals.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
