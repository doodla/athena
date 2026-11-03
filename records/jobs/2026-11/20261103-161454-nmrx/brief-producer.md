# Job 20261103-161454-nmrx: Judge candidate-shaped PR heads as pending in pr-head and history checks

## Request (verbatim)

> try again on 20261027-085452-nm2t

## Reading (accepted by the user)

Retry of job 20261027-085452-nm2t, which stopped in planning on a budget error (error_max_budget_usd) with no candidate, so nothing is kept and the accepted reading is redone unchanged. A PR's first CI run sees only the job's candidate commit, so `athena check --pr-head` and `--history` fail it; both will judge such a head as pending (pass, visible note) and still fully judge the landing head. Audience: maintainers and CI reviewing Athena PRs.

Assumptions:
- Prior job 20261027-085452-nm2t: it failed in planning with no candidate and no changed files, so there is no finding to fix and no work to keep or drop; this job starts from the base and re-does the accepted reading with the same deliverables and out_of_scope.
- Code survey: read crates/athena-core/src/pr_check.rs. check_pr_head lists base..head via rev-list, classifies each commit with land::classify_commit, rejects JobLanding commits through landing_problem (verification record, accepted reading, prediction_problem), rejects declared interventions after the last landing, then compares the head fingerprint to the last landing's verified_fingerprint. Also read crates/athena-cli/src/main.rs (the check command, around lines 1334-1381, which calls check_pr_head and display::pr_head_report) and crates/athena-cli/src/display.rs (pr_head_report prints 'pr-head {verdict}'). crates/athena-core/src/reconcile.rs and the history check tests (crates/athena-cli/tests/history_pr_head.rs, tests/reconcile.rs) are to be read and extended by the producer. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by running check_pr_head and the history check on fixture repositories.
- A candidate-shaped head means governed work plus the job's candidate record with no verification record and no outcome record. Such a head is reported as pending and passes. Any other head shape, including governed changes with no job records, is still judged as before.
- This change loosens a check for one head shape only and adds no new refusal, so there is no newly-refused-readings percentage to state.
- Pending is only safe because the landing head, which has verification and outcome records on top of the candidate, is still fully judged before merge.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-core pr_check`: a test whose head is shaped like the early-PR candidate (governed work plus the job's candidate record, no verification or outcome) makes check_pr_head pass as pending, with the pending state in the report.
2. [behavior] Run `cargo test --locked -p athena-cli --test history_pr_head`: a test shows the history check on a candidate-shaped PR head, with main and the records branch present, passes as pending.
3. [behavior] Run `cargo test --locked`: non-candidate heads still fail naming each offending commit (governed change with no job records, failed verification, declared intervention after the job landing); existing pr_check and history_pr_head tests pass.
4. [behavior] Run `cargo test --locked`: once verification and outcome records are on top of the candidate, the landing head is fully judged and passes; a wrong verified fingerprint fails.
5. [content] `athena check --pr-head` prints a distinct pending line (Grep crates/athena-cli/src/display.rs or crates/athena-core/src/pr_check.rs for it) so a CI log reader can tell pending from verified.
6. [content] docs/design/records-history.md §5.1/§10 and CHANGELOG.md describe the pending treatment and why the landing head is still fully checked before merge; docs/decisions.md records the call.
7. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both pass.
8. [content] git diff shows rust-toolchain.toml, setup.sh, athena.toml, defaults.toml and .github/workflows/ci.yml unchanged.

## Expected deliverables

- crates/athena-core/src/pr_check.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/history_pr_head.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- rust-toolchain.toml and setup.sh: not changed by this job
- athena.toml, defaults.toml and .github/workflows/ci.yml: not changed; making the checks required is the cutover job
- Making the checks required at cutover
- Landed work of jobs 20261026-232420-ctt9, 20261026-232356-ds51 and 20261025-171957-p5ns is kept as landed and not edited
- Loosening checks for non-candidate heads

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for pending, landing-head and non-candidate cases; cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the pending and fail outcomes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG explain the pending treatment for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The pending rule is enforced in check_pr_head and the history check, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The checks stop failing on the candidate shape itself, not on one PR.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the named cargo test commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

26 more qualifying classes are left out.

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

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked -p athena-core pr_check`: a test whose head is shaped like the early-PR candidate (governed work plus the job's candidate record, no verification or outcome) makes check_pr_head pass as pending, with the pending state in the report.
Run `cargo test --locked -p athena-cli --test history_pr_head`: a test shows the history check on a candidate-shaped PR head, with main and the records branch present, passes as pending.
Run `cargo test --locked`: non-candidate heads still fail naming each offending commit (governed change with no job records, failed verification, declared intervention after the job landing); existing pr_check and history_pr_head tests pass.
Run `cargo test --locked`: once verification and outcome records are on top of the candidate, the landing head is fully judged and passes; a wrong verified fingerprint fails.
`athena check --pr-head` prints a distinct pending line (Grep crates/athena-cli/src/display.rs or crates/athena-core/src/pr_check.rs for it) so a CI log reader can tell pending from verified.
docs/design/records-history.md §5.1/§10 and CHANGELOG.md describe the pending treatment and why the landing head is still fully checked before merge; docs/decisions.md records the call.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both pass.
git diff shows rust-toolchain.toml, setup.sh, athena.toml, defaults.toml and .github/workflows/ci.yml unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
