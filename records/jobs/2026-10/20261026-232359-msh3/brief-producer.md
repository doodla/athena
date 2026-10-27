# Job 20261026-232359-msh3: Move the Transport row from gh/mod.rs to gh/transport.rs

## Request (verbatim)

> Priority: high
> Structure pass job G2 (docs/design/structure-pass.md, section 4). Move the Transport row to gh/transport.rs. No behavior change. Depends on 20261026-232356-ds51.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/transport.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure-pass job G2: move the Transport items (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog; about 340 lines) verbatim into gh/transport.rs. Re-export them from gh/mod.rs so public paths stay unchanged. No behavior change. The audience is the verifier applying section 5 of structure-pass.md.

Assumptions:
- G1 (gh.rs renamed to gh/mod.rs) has landed. The Glob in my copy found no gh files, so I could not read the source. Line ranges 36-374 come from the design doc and may have shifted.
- Items used only inside the Transport row may need pub(super) or pub(crate) visibility so the rest of gh can still reach them. This is the minimum change needed and is not a behavior change.
- The external_calls_guard WRAPPER prefix covers gh/, so the new file passes the guard.

## Acceptance criteria

1. [content] gh/transport.rs holds the Transport row items listed in structure-pass.md section 2.1 (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog) with the same text as before the move (git diff -M --color-moved shows only moved lines).
2. [content] gh/mod.rs declares `mod transport;` and re-exports each moved public and pub(crate) item, so no file outside gh/ changes its import path. Only import lines are edited in test modules.
3. [content] The report lists the caller-check Grep for each moved name's old path, as section 5 rule 5 requires.
4. [content] Only gh/mod.rs, gh/transport.rs and CHANGELOG.md change. CHANGELOG.md has an entry for the move. No config, workflow or docs file other than CHANGELOG.md changes.
5. [behavior] Run `cargo test --locked -p athena-core -- --list` and check that the test names match the list before the move. Run `cargo test --locked` and check that it passes, including the retry tests that use the moved code.
6. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both must be clean.

## Expected deliverables

- crates/athena-core/src/gh/transport.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any other row of structure-pass.md section 2.1 (G3 onward)
- Updating docs paths such as docs/system-facts.md or docs/architecture.md
- Any behavior, signature or visibility change beyond what re-exports need
- Editing introspect.rs or external_calls_guard.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: verifier runs test list, tests, fmt and clippy
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one row moved, three files
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: the test list and the retry tests exercise the moved code
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry written for outside readers
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 29 cases in 29 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-handover check for reader-move jobs. It would enumerate the readers by grep and require, for each, a named test that calls the reader's real entry point. It would also run a mutation step: revert each reader to the in-tree path and confirm its test fails. The producer would have to pass th...
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

15 more qualifying classes are left out.

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

gh/transport.rs holds the Transport row items listed in structure-pass.md section 2.1 (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog) with the same text as before the move (git diff -M --color-moved shows only moved lines).
gh/mod.rs declares `mod transport;` and re-exports each moved public and pub(crate) item, so no file outside gh/ changes its import path. Only import lines are edited in test modules.
The report lists the caller-check Grep for each moved name's old path, as section 5 rule 5 requires.
Only gh/mod.rs, gh/transport.rs and CHANGELOG.md change. CHANGELOG.md has an entry for the move. No config, workflow or docs file other than CHANGELOG.md changes.
Run `cargo test --locked -p athena-core -- --list` and check that the test names match the list before the move. Run `cargo test --locked` and check that it passes, including the retry tests that use the moved code.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both must be clean.

## Report

End with the producer report as your structured output.
