# Job 20261027-011733-vrsk: Move Transport row from gh/mod.rs to gh/transport.rs (G2 retry)

## Request (verbatim)

> try again on 20261027-004517-gk9e

## Reading (accepted by the user)

Retry of structure-pass job G2: move the Transport items verbatim into gh/transport.rs, re-export them from gh/mod.rs so public paths are unchanged, and add a CHANGELOG entry. Criteria rest only on what the verifier can run itself (Grep, cargo test --list), not on the producer's report.

Assumptions:
- Audience is the verifier applying section 5 of docs/design/structure-pass.md.
- Dependency job 20261026-232356-ds51 has landed.
- Adding use lines to test modules is allowed as import edits.
- The handover-check work in the introspection's proposal is out of scope; the user's request is only to retry G2.

## Acceptance criteria

1. [content] gh/transport.rs holds the Transport row items (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog) with the same text as before; git diff shows only moved lines plus mod/re-export lines.
2. [content] gh/mod.rs declares `mod transport;` and re-exports each moved item that had an external path. Test modules differ only in `use` lines.
3. [behavior] No file outside crates/athena-core/src/gh/ names a moved item by a path that stops resolving: a Grep of crates/ for `gh::transport::` finds no hits outside gh/, and every `gh::<moved name>` hit outside gh/ still resolves because cargo check passes with no caller file changed. The verifier runs the Grep itself.
4. [behavior] Run `cargo test --locked -- --list` and compare the test names with the base (git stash-free: compare against output at base via git show); the list is identical. Run `cargo test --locked`; it passes.
5. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`; both are clean.
6. [content] CHANGELOG.md has an entry for the move; only gh/mod.rs, gh/transport.rs and CHANGELOG.md change, with no config, workflow or other docs changes.

## Expected deliverables

- crates/athena-core/src/gh/transport.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing Athena's producer-to-verifier handover code
- Any behavior change to the moved code
- Changes to docs/design/structure-pass.md or config

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run Grep and test list comparison.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move in three files.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that paths and tests are unchanged.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code stays Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261027-004517-gk9e

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> The report lists the caller-check Grep for each moved name's old path, as section 5 requires.: The producer's report was not in my brief or my working directory, so I cannot confirm what it lists. I ran the caller Grep myself on the candidate with the Grep tool, pattern `gh::(Runner|GhCli|...|HostErrorLog)` over crates/. It found 14 hits in 9 files, all outside gh/ and none in a changed file, so the counts before and after are equal by construction. The full test suite and clippy --all-targets passing shows every hit still compiles. That covers the substance of rule 5, but not the stated criterion, which is about the report's contents.

## Findings you must address

The verifier of job 20261027-004517-gk9e raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Criterion 3 (the report lists the caller-check Grep) cannot be confirmed: the producer's report was not given to the verifier. Neither the brief nor the snapshot contains the report. My own Grep over crates/ for the old `gh::<name>` paths finds 14 hits in 9 files outside gh/, and no file outside gh/ changed, so the caller check passes in substance. Athena holds the report and can check whether it lists this Grep with counts before and after.

All of the verification's findings:

> major: Criterion 3 (the report lists the caller-check Grep) cannot be confirmed: the producer's report was not given to the verifier. Neither the brief nor the snapshot contains the report. My own Grep over crates/ for the old `gh::<name>` paths finds 14 hits in 9 files outside gh/, and no file outside gh/ changed, so the caller check passes in substance. Athena holds the report and can check whether it lists this Grep with counts before and after.
> nit (crates/athena-core/src/gh/mod.rs:2335, 2479): Two `use` lines were added to the gh test modules, although section 5.1 rule 1 says no test line changes for G1-G7. The lines are `use std::cell::RefCell;` and `use athena_data::records::{HostError, MessageClass};`. Rule 1 and criterion 2 both allow edits to `use` lines, and these were needed because mod.rs no longer imports those names.
> nit (crates/athena-core/src/gh/mod.rs:38): clock_entropy is brought into mod.rs with a private `use`, not the `pub(super) use` that section 4 suggests. A private `use` is enough here: GhClient in mod.rs and the child test modules (through `use super::*`) can both reach it. Nothing outside gh used the private function before.

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

The work starts from the rejected candidate of job 20261027-004517-gk9e: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

gh/transport.rs holds the Transport row items (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog) with the same text as before; git diff shows only moved lines plus mod/re-export lines.
gh/mod.rs declares `mod transport;` and re-exports each moved item that had an external path. Test modules differ only in `use` lines.
No file outside crates/athena-core/src/gh/ names a moved item by a path that stops resolving: a Grep of crates/ for `gh::transport::` finds no hits outside gh/, and every `gh::<moved name>` hit outside gh/ still resolves because cargo check passes with no caller file changed. The verifier runs the Grep itself.
Run `cargo test --locked -- --list` and compare the test names with the base (git stash-free: compare against output at base via git show); the list is identical. Run `cargo test --locked`; it passes.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`; both are clean.
CHANGELOG.md has an entry for the move; only gh/mod.rs, gh/transport.rs and CHANGELOG.md change, with no config, workflow or other docs changes.

## Report

End with the producer report as your structured output.
