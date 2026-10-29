# Job 20261026-232414-p8n4: Move the Landing row from gh/mod.rs into gh/landing.rs

## Request (verbatim)

> Priority: high
> Structure pass job G7 (docs/design/structure-pass.md, section 4). Move the Landing row; its diff is only a move. No behavior change. Depends on 20261026-232405-wqeg and 20261026-232408-tdy1 and 20261026-232411-2zv3.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/landing.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass G7: cut the Landing row (work_branch, landed_commit*, squash_text_for, Poll, LandPr, PrReport, land_pr, confirm_existing_merge, refusal_text, publish, Athena::land_pr, Athena::after_merge) out of gh/mod.rs into new gh/landing.rs with identical text, re-exporting public names so gh:: paths are unchanged. Audience: the owner and verifier judging a pure-move diff. Builds on landed G4 (20261026-232405-wqeg), G5 (20261026-232408-tdy1) and the rejected G6 (20261026-232411-2zv3).

Assumptions:
- Code survey: read crates/athena-core/src/gh/mod.rs (mod declarations at lines 31-36, pub use re-exports, LandPr at 157, land_pr at 215, publish at 411), gh/early.rs (imports LandPr, GitRemote, Remote via super), gh/client.rs (EARLY_RUN_WAIT now lives at client.rs:18) and gh/records.rs. The entry point is the public gh:: API used by main.rs, drain.rs, run.rs, display.rs; there is no records validation or new data file, so check_record_file and AnyRecord do not apply.
- Prior job 20261026-232405-wqeg (G4, landed, pass): its two findings (stale test count 1397 vs 1617; reconcile.rs listed but unchanged) are addressed by not citing a fixed test count (compare the test list before and after) and not listing reconcile.rs. Its files records.rs, mod.rs, CHANGELOG.md are kept as landed; only mod.rs and CHANGELOG.md are touched again.
- Prior job 20261026-232408-tdy1 (G5, landed, pass): its CHANGELOG inaccuracies (api/repo_path visibility, TEST_NAME_LIMIT, CI_CHECK_STEP) concern the G5 entry; they are kept as is and not rewritten here. The G7 entry must itself list every moved name and every visibility change precisely, avoiding the same mistakes. client.rs is kept as landed.
- Prior job 20261026-232411-2zv3 (G6, rejected): its major finding (client<->early cycle through EARLY_RUN_WAIT) no longer applies in this base because EARLY_RUN_WAIT is now defined in client.rs and re-exported from mod.rs. Its minor finding (early.rs uses LandPr, so early->landing plus landing->early) applies: G7 must keep early.rs importing LandPr through super (the mod.rs re-export) with text unchanged, and the cycle must be noted in the CHANGELOG entry; if landing.rs cannot avoid a cycle with early.rs beyond that, report it. Its CHANGELOG nit (cfg(test) pr imports kept) is addressed by describing mod.rs imports accurately.
- Visibility widening is allowed only where a moved name is called from a sibling or from tests left in mod.rs (pub(super)); each such change is listed in the CHANGELOG and the report.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [content] A new crates/athena-core/src/gh/landing.rs holds the Landing row items (work_branch, landed_commit*, squash_text_for, Poll, LandPr, PrReport, land_pr, confirm_existing_merge, refusal_text, publish, Athena::land_pr, Athena::after_merge); git diff shows these as removed from gh/mod.rs and added to landing.rs with identical text apart from import lines and listed pub(super) visibility changes.
2. [content] gh/mod.rs declares `mod landing;` and re-exports every public moved name (land_pr, LandPr, PrReport, Poll, landed_commit*, etc.), so a Grep for `gh::(land_pr|LandPr|PrReport|Poll|landed_commit)` across crates/ and xtask/ shows callers unchanged and none edited.
3. [content] The diff changes no test function names or bodies: only `use` lines in test modules differ, and the set of #[test] fn names in the crate is unchanged (Grep count of `#[test]` before/after equal).
4. [content] gh/landing.rs imports only from host (client), early, records, remote, pages and transport siblings as in structure-pass.md 3.1; no lower-row module imports landing except via the mod.rs re-export, and the early.rs LandPr dependency is noted in CHANGELOG.md.
5. [content] CHANGELOG.md has a G7 entry that accurately lists every moved name, every visibility change and every import change kept in mod.rs, and the caller check (which files use each moved name) for each moved name.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all exit 0, including every land_pr_* and early test, with the same number of tests as the recorded base run (1617).
7. [behavior] Run `cargo test --locked -- --list` and confirm the list of gh:: test names is identical to the base (git show of base is not needed: the diff touches no test names).

## Expected deliverables

- crates/athena-core/src/gh/landing.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving any other row (G8 test modules, R jobs)
- Redoing G4, G5 or G6 or rewriting their CHANGELOG entries
- Updating docs that cite gh.rs (system-facts.md, architecture.md)
- Editing introspect.rs, external_calls_guard.rs, athena.toml, defaults.toml or workflows
- Any behavior change or test edits beyond import lines
- Edits to callers in main.rs, drain.rs, run.rs, display.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust module move.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and full cargo test criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single row move, no other changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs fmt/clippy/test and diffs the moved text.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check unchanged test list and passing land_pr tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Proof of section 5 criteria.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new crates/athena-core/src/gh/landing.rs holds the Landing row items (work_branch, landed_commit*, squash_text_for, Poll, LandPr, PrReport, land_pr, confirm_existing_merge, refusal_text, publish, Athena::land_pr, Athena::after_merge); git diff shows these as removed from gh/mod.rs and added to landing.rs with identical text apart from import lines and listed pub(super) visibility changes.
gh/mod.rs declares `mod landing;` and re-exports every public moved name (land_pr, LandPr, PrReport, Poll, landed_commit*, etc.), so a Grep for `gh::(land_pr|LandPr|PrReport|Poll|landed_commit)` across crates/ and xtask/ shows callers unchanged and none edited.
The diff changes no test function names or bodies: only `use` lines in test modules differ, and the set of #[test] fn names in the crate is unchanged (Grep count of `#[test]` before/after equal).
gh/landing.rs imports only from host (client), early, records, remote, pages and transport siblings as in structure-pass.md 3.1; no lower-row module imports landing except via the mod.rs re-export, and the early.rs LandPr dependency is noted in CHANGELOG.md.
CHANGELOG.md has a G7 entry that accurately lists every moved name, every visibility change and every import change kept in mod.rs, and the caller check (which files use each moved name) for each moved name.
Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all exit 0, including every land_pr_* and early test, with the same number of tests as the recorded base run (1617).
Run `cargo test --locked -- --list` and confirm the list of gh:: test names is identical to the base (git show of base is not needed: the diff touches no test names).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
