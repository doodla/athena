# Job 20261026-232402-6qme: Move Git remote and Pages rows out of gh/mod.rs into submodules

## Request (verbatim)

> Priority: high
> Structure pass job G3 (docs/design/structure-pass.md, section 4). Move the Git remote and Pages rows. No behavior change. Depends on 20261026-232356-ds51.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/remote.rs, crates/athena-core/src/gh/pages.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure-pass job G3: move Remote trait, GitRemote and helpers into gh/remote.rs, and the Pages items (PAGES_WORKFLOW, PagesRun, PagesHost and its GhClient impl, PagesReport, wait constants, wait_pages_run, watch_pages) into gh/pages.rs. Text is moved verbatim, gh/mod.rs re-exports so public paths are unchanged, and a CHANGELOG entry is added. The audience is the verifier checking the section 5 proof.

Assumptions:
- Job G1 has landed so crates/athena-core/src/gh/mod.rs exists.
- Visibility of moved private items may widen to pub(super)/pub(crate) only as needed for the split.
- Docs still citing gh.rs are left to a later job (G1 verification noted this; section 5 rule 7).

## Acceptance criteria

1. [content] gh/remote.rs holds the Remote trait, GitRemote, its impl and helpers used only by them; gh/pages.rs holds PAGES_WORKFLOW, PagesRun, PagesHost, the GhClient impl of PagesHost, PagesReport, the PAGES_* wait constants, wait_pages_run and watch_pages. Moved text is identical to the old text apart from visibility and import lines (git diff shows moved blocks unchanged).
2. [content] gh/mod.rs declares the remote and pages modules and re-exports every previously public name with pub use, so gh::Remote, gh::GitRemote, gh::PagesHost, gh::PagesReport, gh::watch_pages and other public paths are unchanged.
3. [content] Existing tests, including after_a_merge_the_branch_is_resynced_then_pages_watched_then_code_health_checked, are edited only on import lines; no test is added, removed or renamed.
4. [content] No file outside crates/athena-core/src/gh/ names a moved item by an old module path (gh::remote::... or gh::pages::... would be new internal paths only); the verifier confirms with a Grep over crates/ for those paths and for moved names, finding every caller still uses the gh:: re-export.
5. [content] CHANGELOG.md has an entry for the G3 move stating behavior is unchanged.
6. [behavior] Run cargo test --locked -- --list before (git stash-free: compare against the base via git show/diff of test names) and after; the lists of test names are identical.
7. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked; all three pass.

## Expected deliverables

- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/remote.rs (create)
- crates/athena-core/src/gh/pages.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Updating docs that cite gh.rs
- Moving other gh rows (other G jobs)
- Any behavior change
- Editing introspect.rs or the external_calls_guard

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria
- small: Prefer one focused change; size is a design choice, not a wish list. Here: only the two rows move
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: verifier greps and compares test lists
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: test-list comparison and grep of callers
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: n/a pure move; re-exports keep paths
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 30 cases in 30 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when the report is absent, and is not sent to the verifier to be ...
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

16 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

gh/remote.rs holds the Remote trait, GitRemote, its impl and helpers used only by them; gh/pages.rs holds PAGES_WORKFLOW, PagesRun, PagesHost, the GhClient impl of PagesHost, PagesReport, the PAGES_* wait constants, wait_pages_run and watch_pages. Moved text is identical to the old text apart from visibility and import lines (git diff shows moved blocks unchanged).
gh/mod.rs declares the remote and pages modules and re-exports every previously public name with pub use, so gh::Remote, gh::GitRemote, gh::PagesHost, gh::PagesReport, gh::watch_pages and other public paths are unchanged.
Existing tests, including after_a_merge_the_branch_is_resynced_then_pages_watched_then_code_health_checked, are edited only on import lines; no test is added, removed or renamed.
No file outside crates/athena-core/src/gh/ names a moved item by an old module path (gh::remote::... or gh::pages::... would be new internal paths only); the verifier confirms with a Grep over crates/ for those paths and for moved names, finding every caller still uses the gh:: re-export.
CHANGELOG.md has an entry for the G3 move stating behavior is unchanged.
Run cargo test --locked -- --list before (git stash-free: compare against the base via git show/diff of test names) and after; the lists of test names are identical.
Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked; all three pass.

## Report

End with the producer report as your structured output.
