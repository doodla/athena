# Job 20261025-034452-wqp8: Add urgent priority and a priority-record command that plan uses

## Request (verbatim)

> Priority: high
> Speed first (BRIEF.md addendum of 2026-10-25, 'Athena should prioritize anything that'll help it move faster ( includes the time wasted )'): add a priority level above high, urgent, and make Athena rank by it. The reader marks a request urgent, with its reason recorded in the reading, when it makes Athena faster or cuts wasted time (failed or rejected attempts, planning stops, timeouts, retries that redo work, thrash, duplicated checks). Add a command that records a priority for a queued request as a record (requests stay append-only), which plan uses over the priority in the request's text, so the order can change without re-queuing.
> Acceptance:
> - A test shows urgent ordered before high, high before normal, with dependencies still first.
> - A test shows the command's priority record overriding the text's priority, and the change shown in athena plan and status.
> - A reading test shows a speed request marked urgent with a reason, and an ordinary one not.
> Files: crates/athena-core/src/plan.rs, crates/athena-core/src/reading.rs, crates/athena-cli/src/main.rs, crates/athena-data/src/records.rs, crates/athena-context/prompts/reader.md, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Add an Urgent priority above High to plan ordering, and a CLI command that appends a priority record for a queued request, which plan and status use over the request text's priority. The reader's urgent-marking rule goes in outputs.rs or briefs.rs (reader.md is at its token budget), with a reason recorded in the reading.

Assumptions:
- Goal: let Athena's operator reorder queued work without re-queuing and have speed-improving requests run first; audience is the operator and the reader agent.
- Per the user's reply, the reader rule goes in outputs.rs or briefs.rs, net of removals, since reader.md is at 1997/2000 tokens; reader.md is not edited unless text is removed.
- The reading records urgency as a priority plus reason field or text in the existing output; exact schema chosen by the producer.
- Priority records are a new record kind in athena-data records.rs; latest record wins.

## Acceptance criteria

1. [behavior] Run cargo test: a plan test shows urgent before high, high before normal, with a dependency still ordered first even when the dependent is urgent.
2. [behavior] Run cargo test: a test shows a priority record for a queued request overrides the priority in its text, and the order shown by plan and the priority shown by status change, with no new request record written.
3. [behavior] Run cargo test: a reading test shows a speed-improving request marked urgent with a recorded reason, and an ordinary request not urgent.
4. [content] The new CLI command appends a priority record (requests are not rewritten), rejects unknown priority values and unknown or non-queued requests with a clear error; Priority::from_text parses 'Priority: urgent'; main.rs help mentions urgent and the command.
5. [content] The urgent-marking rule (makes Athena faster or cuts wasted time: failed/rejected attempts, planning stops, timeouts, redo-work retries, thrash, duplicated checks; reason required in the reading) lives in the outputs.rs field doc or the briefs.rs reader brief, not in reader.md; reader.md does not grow.
6. [behavior] Run cargo test --locked -p athena-cli --test budgets: the reader set stays within its 2000-token budget.
7. [content] CHANGELOG.md has an entry and docs/decisions.md records the decision with alternatives considered.
8. [behavior] cargo clippy, cargo fmt --check are clean and the full cargo test passes.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Rewriting or mutating existing request records
- Growing reader.md beyond the token budget
- Changing athena.toml or defaults.toml
- Changing priority semantics beyond adding urgent and the override

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for ordering, override, and reading; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing ordering and override behavior.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Ordering enforced in plan code and tested; prompt budget enforced by existing budgets test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and changelog written for outside readers.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Decision with alternatives recorded in docs/decisions.md.
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

Run cargo test: a plan test shows urgent before high, high before normal, with a dependency still ordered first even when the dependent is urgent.
Run cargo test: a test shows a priority record for a queued request overrides the priority in its text, and the order shown by plan and the priority shown by status change, with no new request record written.
Run cargo test: a reading test shows a speed-improving request marked urgent with a recorded reason, and an ordinary request not urgent.
The new CLI command appends a priority record (requests are not rewritten), rejects unknown priority values and unknown or non-queued requests with a clear error; Priority::from_text parses 'Priority: urgent'; main.rs help mentions urgent and the command.
The urgent-marking rule (makes Athena faster or cuts wasted time: failed/rejected attempts, planning stops, timeouts, redo-work retries, thrash, duplicated checks; reason required in the reading) lives in the outputs.rs field doc or the briefs.rs reader brief, not in reader.md; reader.md does not grow.
Run cargo test --locked -p athena-cli --test budgets: the reader set stays within its 2000-token budget.
CHANGELOG.md has an entry and docs/decisions.md records the decision with alternatives considered.
cargo clippy, cargo fmt --check are clean and the full cargo test passes.

## Report

End with the producer report as your structured output.
