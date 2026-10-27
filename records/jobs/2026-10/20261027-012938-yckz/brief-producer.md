# Job 20261027-012938-yckz: Move Transport row from gh/mod.rs to gh/transport.rs

## Request (verbatim)

> Priority: high
> Structure pass job G2 (docs/design/structure-pass.md, section 4). Move the Transport row to gh/transport.rs. No behavior change. Depends on 20261026-232356-ds51.
> Acceptance:
> - The proof of section 5 holds, shown only by what the verifier can run itself: existing tests pass with only import lines edited; the `cargo test -- --list` output is the same before and after; moved text is the same text; public paths are unchanged through re-exports; a Grep shows no file outside gh/ names a moved item by its old module path.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/transport.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure-pass job G2: move the Transport items (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog) verbatim into gh/transport.rs, re-exported from gh/mod.rs so public paths are unchanged. No behavior change; audience is the verifier checking the section 5 proof with commands it can run itself.

Assumptions:
- G1 (gh.rs renamed to gh/mod.rs) has landed.
- The exact item list is taken from the Transport row of docs/design/structure-pass.md section 4; the producer follows that row.
- Moved items may need pub(super)/pub(crate) visibility or use-line adjustments; this is the only permitted text change.
- Other docs citing gh.rs are not edited (section 5 rule 7).

## Acceptance criteria

1. [content] gh/transport.rs holds the Transport-row items; their text matches the text removed from gh/mod.rs apart from visibility and use lines, visible in git diff.
2. [content] gh/mod.rs declares the transport module and re-exports every moved item, so existing paths such as crate::gh::Runner remain valid.
3. [content] A Grep the verifier runs shows no file outside crates/athena-core/src/gh/ names a moved item by an old module path (e.g. gh::transport::X where it was gh::X).
4. [content] Existing test code is unchanged except for import lines, visible in git diff.
5. [behavior] Run cargo test --locked -- --list on the base commit and on the change: the outputs are identical.
6. [behavior] cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all pass.
7. [content] CHANGELOG.md has an entry for G2 stating no behavior change.
8. [content] No criterion rests on the producer's report; all checks are runnable by the verifier.

## Expected deliverables

- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/transport.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Moving other rows of the structure pass
- Updating docs citing gh.rs
- Any behavior change
- Editing introspect.rs or the external_calls_guard test

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check test list parity and passing tests, not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy, and cargo test criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the commands itself.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test list diff and Grep are run by the verifier.
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

gh/transport.rs holds the Transport-row items; their text matches the text removed from gh/mod.rs apart from visibility and use lines, visible in git diff.
gh/mod.rs declares the transport module and re-exports every moved item, so existing paths such as crate::gh::Runner remain valid.
A Grep the verifier runs shows no file outside crates/athena-core/src/gh/ names a moved item by an old module path (e.g. gh::transport::X where it was gh::X).
Existing test code is unchanged except for import lines, visible in git diff.
Run cargo test --locked -- --list on the base commit and on the change: the outputs are identical.
cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all pass.
CHANGELOG.md has an entry for G2 stating no behavior change.
No criterion rests on the producer's report; all checks are runnable by the verifier.

## Report

End with the producer report as your structured output.
