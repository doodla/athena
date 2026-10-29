# Job 20261029-023338-pt6z: Move Early PR and CI wait row to gh/early.rs without a cycle

## Request (verbatim)

> try again on 20261026-232411-2zv3

## Reading (accepted by the user)

Retry of job 20261026-232411-2zv3 (G6): move the Early pull request and CI wait row from gh/mod.rs into gh/early.rs with text unchanged, but keep EARLY_RUN_WAIT in client.rs beside PrHost::early_run_wait so no client↔early cycle exists. Audience: the maintainers and the verifier judging the structure pass.

Assumptions:
- Code survey: read crates/athena-core/src/gh/mod.rs (EARLY_RUN_WAIT at line 62, wait_for_early_run at ~437 uses job.host.early_run_wait()) and crates/athena-core/src/gh/client.rs (line 5 imports super::EARLY_RUN_WAIT; line 73-74 PrHost impl returns it). Siblings: transport.rs, remote.rs, records.rs, pages.rs, client.rs. The behavior is measured by cargo test --locked and --list, not through a stand-in.
- Job 20261026-232411-2zv3 finding major (client.rs:5/74 ↔ early.rs cycle) is fixed by keeping EARLY_RUN_WAIT defined in client.rs (the host row), re-exported from gh/mod.rs so the public path gh::EARLY_RUN_WAIT holds; early.rs imports it from client if needed. This deviation is recorded in docs/design/structure-pass.md.
- Finding minor (early.rs depends on remote.rs and LandPr, which the section 3.1 table omits) is addressed by a note in docs/design/structure-pass.md that early uses remote and LandPr until G7, so G7 must not make landing use early in the opposite direction in a way that cycles.
- Finding nit (CHANGELOG says mod.rs drops crate::pr imports, though two are kept under cfg(test)) is fixed by an accurate CHANGELOG G6 entry.
- The candidate's three changed files (CHANGELOG.md, new gh/early.rs, gh/mod.rs) are dropped as such; the retry starts from base and redoes them, keeping the same approach with the cycle fix.
- Depends on G5 (20261026-232408-tdy1), already landed. The earlier proposed dependency-direction test was a suggestion for prevention; the user's request is only to retry, so a new test is out of scope (the test list must stay unchanged).

## Acceptance criteria

1. [content] gh/early.rs exists and holds replace_early, wait_for_early_run, name_failure, Waited, wait_for_checks, EarlyPr, open_early_pr, https_github_repository, Athena::early_pr, early_pr_to and the EARLY_RUN_* constants other than EARLY_RUN_WAIT; git diff shows them removed from mod.rs with identical text apart from visibility keywords.
2. [content] Grep of crates/athena-core/src/gh/client.rs, transport.rs, remote.rs, pages.rs, records.rs finds no `early` reference (no `super::early`, no import of a name defined in early.rs); early.rs imports nothing from a higher row, and no two sibling modules use each other circularly.
3. [content] gh/mod.rs declares `mod early;` and re-exports every previously public moved name (EarlyPr, open_early_pr, EARLY_RUN_WAIT), so Grep of `gh::(EarlyPr|open_early_pr|EARLY_RUN_WAIT)` across crates/ finds the same occurrences as at base and no caller outside gh changed.
4. [content] Existing tests are unmoved and unedited except import lines; no test is added or removed.
5. [behavior] Run `cargo test --locked -- --list` and compare to base: the test list is unchanged; `cargo test --locked` passes, including land_pr_waits_for_the_early_run_before_pushing and land_pr_replaces_the_early_head_pinned_and_reuses_the_early_pull_request.
6. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`; both pass.
7. [content] CHANGELOG.md has an accurate G6 entry listing every moved name (including private constants), every visibility change, that EARLY_RUN_WAIT stays in client.rs, and the retained cfg(test) crate::pr imports in mod.rs.
8. [content] docs/design/structure-pass.md records the EARLY_RUN_WAIT placement deviation and the early→remote/LandPr dependency note for G7.
9. [content] Each moved name has a caller check (a Grep of its uses) shown in the diff-verifiable form: the same occurrences resolve through gh re-exports or inside gh.

## Expected deliverables

- crates/athena-core/src/gh/early.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/client.rs (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving any other row (G7 Landing, G8, R jobs)
- Adding a dependency-direction test or xtask check (the test list must stay unchanged)
- Updating docs paths such as docs/system-facts.md or docs/architecture.md
- Any behavior or signature change beyond visibility needed for re-exports and cross-module calls
- Editing introspect.rs or external_calls_guard.rs
- Redoing G5 or earlier gh jobs; transport.rs, pages.rs, records.rs, remote.rs stay as is

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Full cargo test, clippy and fmt required.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs test list comparison and checks.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Cycle fixed by placing the shared constant in the lower row.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check test results and grep outcomes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and design note written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261026-232411-2zv3

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> No lower-row module (transport, remote, client, pages, records) names early; early.rs uses only host/client/transport siblings; no circular use.: Checked by reading the code. crates/athena-core/src/gh/client.rs:5 has `use super::{EARLY_RUN_WAIT, GhFailure, HOST, HostErrorLog, Retry, Runner};` and line 74 returns EARLY_RUN_WAIT. That constant is now defined in early.rs:22 and reaches client.rs only through mod.rs's `pub use early::{EARLY_RUN_WAIT, ...}`. early.rs imports PrHost, GhClient, Checks, CiRunState, NewPr and parse_remote from client, so client and early use each other: a circular use. early.rs also uses `GitRemote` and `Remote` from remote.rs, and `LandPr` from mod.rs (a Landing-row type), so it does not use only host/client/transport. structure-pass.md section 3.1 says a circular use means the job stops.

## Findings you must address

The verifier of job 20261026-232411-2zv3 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/gh/client.rs:5, crates/athena-core/src/gh/client.rs:74, crates/athena-core/src/gh/early.rs:7-10): The move creates a client ↔ early dependency cycle, which the design document says must stop the job. client.rs (host row, lower) imports `super::EARLY_RUN_WAIT`, which now lives in early.rs; early.rs imports PrHost, GhClient and others from client. structure-pass.md section 3.1: "No module in a lower row names a higher one. A circular `use` between two siblings is a sign a move was cut in the wrong place and the job stops." client.rs and docs/design/structure-pass.md were in the expected deliverables but were not edited, and the CHANGELOG does not mention the cycle. Simpler alternative: keep EARLY_RUN_WAIT with the PrHost::early_run_wait default in client.rs and record that deviation in structure-pass.md. Otherwise, stop and report as the document requires.

All of the verification's findings:

> major (crates/athena-core/src/gh/client.rs:5, crates/athena-core/src/gh/client.rs:74, crates/athena-core/src/gh/early.rs:7-10): The move creates a client ↔ early dependency cycle, which the design document says must stop the job. client.rs (host row, lower) imports `super::EARLY_RUN_WAIT`, which now lives in early.rs; early.rs imports PrHost, GhClient and others from client. structure-pass.md section 3.1: "No module in a lower row names a higher one. A circular `use` between two siblings is a sign a move was cut in the wrong place and the job stops." client.rs and docs/design/structure-pass.md were in the expected deliverables but were not edited, and the CHANGELOG does not mention the cycle. Simpler alternative: keep EARLY_RUN_WAIT with the PrHost::early_run_wait default in client.rs and record that deviation in structure-pass.md. Otherwise, stop and report as the document requires.
> minor (crates/athena-core/src/gh/early.rs:7-10): early.rs depends on remote.rs and on LandPr (a Landing-row type), which the 3.1 dependency table does not allow. early.rs has `use super::{..., GitRemote, LandPr, ..., Remote, ...}`. The table gives early.rs's dependencies as `host`, `transport` only. Remote is a lower row, so no direction is broken. But once G7 moves LandPr to landing.rs, early → landing plus landing → early becomes a cycle. With the text kept identical this cannot be avoided, so the design document should note it before G7.
> nit (CHANGELOG.md (G6 entry)): The CHANGELOG says mod.rs 'drops' the crate::pr imports that only the moved code used, but two are kept for tests. mod.rs:17-18 keeps `#[cfg(test)] use crate::pr::{load_early_link, store_early};`.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

18 more qualifying classes are left out.

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

The work starts from the rejected candidate of job 20261026-232411-2zv3: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

gh/early.rs exists and holds replace_early, wait_for_early_run, name_failure, Waited, wait_for_checks, EarlyPr, open_early_pr, https_github_repository, Athena::early_pr, early_pr_to and the EARLY_RUN_* constants other than EARLY_RUN_WAIT; git diff shows them removed from mod.rs with identical text apart from visibility keywords.
Grep of crates/athena-core/src/gh/client.rs, transport.rs, remote.rs, pages.rs, records.rs finds no `early` reference (no `super::early`, no import of a name defined in early.rs); early.rs imports nothing from a higher row, and no two sibling modules use each other circularly.
gh/mod.rs declares `mod early;` and re-exports every previously public moved name (EarlyPr, open_early_pr, EARLY_RUN_WAIT), so Grep of `gh::(EarlyPr|open_early_pr|EARLY_RUN_WAIT)` across crates/ finds the same occurrences as at base and no caller outside gh changed.
Existing tests are unmoved and unedited except import lines; no test is added or removed.
Run `cargo test --locked -- --list` and compare to base: the test list is unchanged; `cargo test --locked` passes, including land_pr_waits_for_the_early_run_before_pushing and land_pr_replaces_the_early_head_pinned_and_reuses_the_early_pull_request.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`; both pass.
CHANGELOG.md has an accurate G6 entry listing every moved name (including private constants), every visibility change, that EARLY_RUN_WAIT stays in client.rs, and the retained cfg(test) crate::pr imports in mod.rs.
docs/design/structure-pass.md records the EARLY_RUN_WAIT placement deviation and the early→remote/LandPr dependency note for G7.
Each moved name has a caller check (a Grep of its uses) shown in the diff-verifiable form: the same occurrences resolve through gh re-exports or inside gh.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
