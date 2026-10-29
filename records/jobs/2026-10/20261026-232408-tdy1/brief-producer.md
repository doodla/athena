# Job 20261026-232408-tdy1: Move the Host API client row to gh/client.rs

## Request (verbatim)

> Priority: high
> Structure pass job G5 (docs/design/structure-pass.md, section 4). Move the Host API client row (caller-heavy: list the caller check for every moved name). No behavior change. Depends on 20261026-232359-msh3.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/client.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job G5: move the Host API client row of docs/design/structure-pass.md section 2.1 (Checks, NewPr, PrDraft, PrHost, CiRunState, MergeMethod(s), GhClient and its impls, parse_* helpers, classify_checks, the CommitHost impl) from gh/mod.rs into new gh/client.rs, with re-exports so public paths stay. Audience: maintainers and the verifier. Depends on job 20261026-232359-msh3 (G2, Transport row, gh/transport.rs); that job was dispatched with no candidate recorded and no findings, but transport.rs is already in the tree, so I treat it as landed and build on it.

Assumptions:
- Code survey: read crates/athena-core/src/gh/ (mod.rs, transport.rs, remote.rs, pages.rs, records.rs) and docs/design/structure-pass.md rows 42, 168, 269; the entry point is the gh module's public API, and no records, data files or validation allow-lists are added, so check_record_file and AnyRecord are not involved. Behavior is measured directly by running the existing test suite and a Grep of call sites.
- Job 20261026-232359-msh3 (G2) had no candidate changed files and no findings recorded; its scope (transport.rs, mod.rs, CHANGELOG.md) is not redone here. This job keeps whatever it left in the tree: gh/transport.rs is kept as is and not touched.
- The request names the new file gh/client.rs, while the design table names it gh/host.rs; the request wins.
- Moved names keep their visibility; only what the move forces (pub(super) or pub(crate) for items mod.rs still uses) may change, and any such change is visible in the diff.
- The verifier checks caller safety by running Grep itself over crates/ for each moved name, not from a producer report.

## Acceptance criteria

1. [content] crates/athena-core/src/gh/client.rs exists and holds the Host API client row (Checks, NewPr, PrDraft, PrHost, CiRunState, MergeMethod, MergeMethods, GhClient with its Host, PrHost and CiSource impls, the CommitHost impl, parse_ci_runs, parse_check_run, parse_ci_run, parse_merged_pr, parse_remote, classify_checks, failing_test_name); `git diff` shows these definitions removed from gh/mod.rs with the same text and no body edits.
2. [content] Public paths are unchanged: gh/mod.rs has `mod client;` and re-exports every previously public moved name, so a Grep of crates/ and xtask/ for `gh::(GhClient|PrHost|Checks|CiRunState|MergeMethod|parse_)` finds the same paths as before and each resolves; no caller outside gh/ has an edited import.
3. [content] Caller check by Grep: for every moved name, a Grep of crates/ and xtask/ for the name finds no use that fails to resolve; in particular GhClient, PagesHost, land_pr( and landed_commit sites in main.rs, display.rs, reconcile.rs, trailer_stop.rs and external_calls.rs are untouched in the diff.
4. [content] Existing tests pass with only import lines edited: `git diff` on test code shows only `use` line changes, and the tests the_client_runs_only_gh_api_with_rest_paths, the_merge_method_is_chosen_from_what_the_host_allows and checks_are_classified_from_the_two_replies still exist under the same names (test list unchanged).
5. [content] Any visibility widening in the diff (pub(super), pub(crate)) applies only to items mod.rs or sibling modules still use, and no signature or body changes appear.
6. [content] CHANGELOG.md has a new entry describing the Host API client move to gh/client.rs as a no-behavior-change refactor.
7. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass and the test count equals the count before the change.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/transport.rs (modify)
- crates/athena-core/src/gh/pages.rs (modify)
- crates/athena-core/src/gh/remote.rs (modify)

## Out of scope

- Moving any other row of structure-pass.md section 2.1 (G3, G4, G6, G7 onward)
- Updating docs paths such as docs/system-facts.md or docs/architecture.md
- Any behavior, signature or visibility change beyond what re-exports need
- Editing introspect.rs or external_calls_guard.rs
- Splitting the job by trait unless the producer reports it cannot finish
- Redoing job 20261026-232359-msh3 (gh/transport.rs stays as is)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust module move.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the checks and Greps callers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the one row moves.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria rest on the test run and Grep of call sites.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 32 cases in 32 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Classify the usage-limit error once, at the single point where model-call errors are turned into results, as a typed "hold" outcome. Every caller (reading, planning, critic, producer, verifier) must receive that type, and the drain's failure counter must not count it. Add a test that injects the lim...
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

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/gh/client.rs exists and holds the Host API client row (Checks, NewPr, PrDraft, PrHost, CiRunState, MergeMethod, MergeMethods, GhClient with its Host, PrHost and CiSource impls, the CommitHost impl, parse_ci_runs, parse_check_run, parse_ci_run, parse_merged_pr, parse_remote, classify_checks, failing_test_name); `git diff` shows these definitions removed from gh/mod.rs with the same text and no body edits.
Public paths are unchanged: gh/mod.rs has `mod client;` and re-exports every previously public moved name, so a Grep of crates/ and xtask/ for `gh::(GhClient|PrHost|Checks|CiRunState|MergeMethod|parse_)` finds the same paths as before and each resolves; no caller outside gh/ has an edited import.
Caller check by Grep: for every moved name, a Grep of crates/ and xtask/ for the name finds no use that fails to resolve; in particular GhClient, PagesHost, land_pr( and landed_commit sites in main.rs, display.rs, reconcile.rs, trailer_stop.rs and external_calls.rs are untouched in the diff.
Existing tests pass with only import lines edited: `git diff` on test code shows only `use` line changes, and the tests the_client_runs_only_gh_api_with_rest_paths, the_merge_method_is_chosen_from_what_the_host_allows and checks_are_classified_from_the_two_replies still exist under the same names (test list unchanged).
Any visibility widening in the diff (pub(super), pub(crate)) applies only to items mod.rs or sibling modules still use, and no signature or body changes appear.
CHANGELOG.md has a new entry describing the Host API client move to gh/client.rs as a no-behavior-change refactor.
Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass and the test count equals the count before the change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
