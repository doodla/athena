# Job 20261024-232101-zeh9: Order athena plan by request priority before age

## Request (verbatim)

> athena plan orders requests only by dependencies and age, so a drain works the oldest requests first however much a newer one matters: on 2026-10-24 the records-history cutover (D111 job 7) sat behind about 30 older requests. Let a request carry a priority (for example a line 'Priority: high' in its words, or a field set by a command) that plan orders by before age, and show it in plan and status.
> Acceptance:
> - A test shows a high-priority request ordered before older normal ones while dependencies are still respected.
> - athena plan and athena status show each request's priority.
> - A request without one keeps today's order.
> Files: crates/athena-core/src/plan.rs, crates/athena-cli/src/display.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Add a priority (high/normal/low) to each plan entry, read from a "Priority: high" line in the request's words. Plan orders ready items by priority, then rung, then age, never ahead of dependencies. Priority is shown in `athena plan` and `athena status`. The audience is the owner who drains the queue and needs urgent work to run first.

Assumptions:
- Priority is a closed set: high, normal (default), low. High sorts first, low last.
- Priority is parsed from a line 'Priority: <level>' in the request text; no new CLI command is added, since the request offered the words as one option.
- Priority is the primary sort key among ready items; dependency order stays strictly first, and the sort is stable, so equal priorities keep today's rung-then-age order.
- The status view needs the priority too, so a small change outside the listed files (status data and reading.rs plan_entry) is likely.
- Existing PlanEntry struct literals in tests use ..PlanEntry::bare so a new field defaults cleanly.

## Acceptance criteria

1. [behavior] A plan.rs unit test builds a newer high-priority entry and several older normal ones, and shows the high one ordered first. Run `cargo test -p athena-core plan`.
2. [behavior] A test shows a high-priority entry that depends on an older normal one still comes after it, so dependencies win over priority. Run `cargo test -p athena-core plan`.
3. [behavior] A test shows that entries with no priority keep the existing order (rung, then age). The existing plan tests pass unchanged. Run `cargo test`.
4. [behavior] A display test shows `athena plan` output has a priority line per item, and the `athena status` request lines show priority. Run `cargo test -p athena-cli`.
5. [behavior] A test shows a request whose text has a line 'Priority: high' (case-insensitive) is read as high priority, and a request without such a line as normal.
6. [content] The plan.rs module doc names the new ordering key: priority, then rung, then age, with dependencies first.
7. [content] CHANGELOG.md has an entry for request priority. docs/decisions.md records the choice, with alternatives considered (words-in-request versus a command-set field) and the cutover motivation.
8. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- A CLI command to set priority after submission
- Changing how rungs or dependencies are inferred
- Changing the parallel grouping logic
- Reordering already-running jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for ordering and parsing; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the observed order and printed output, not that the field exists.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: decisions.md entry records alternatives.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, no new command.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

A plan.rs unit test builds a newer high-priority entry and several older normal ones, and shows the high one ordered first. Run `cargo test -p athena-core plan`.
A test shows a high-priority entry that depends on an older normal one still comes after it, so dependencies win over priority. Run `cargo test -p athena-core plan`.
A test shows that entries with no priority keep the existing order (rung, then age). The existing plan tests pass unchanged. Run `cargo test`.
A display test shows `athena plan` output has a priority line per item, and the `athena status` request lines show priority. Run `cargo test -p athena-cli`.
A test shows a request whose text has a line 'Priority: high' (case-insensitive) is read as high priority, and a request without such a line as normal.
The plan.rs module doc names the new ordering key: priority, then rung, then age, with dependencies first.
CHANGELOG.md has an entry for request priority. docs/decisions.md records the choice, with alternatives considered (words-in-request versus a command-set field) and the cutover motivation.
`cargo clippy --all-targets` is clean and `cargo fmt --check` passes.

## Report

End with the producer report as your structured output.
