# Job 20261025-095436-hasm: Add urgent priority and a priority-record command; guard decisions.md

## Request (verbatim)

> try again on 20261025-034452-wqp8

## Reading (accepted by the user)

Retry of the rejected job: add Urgent above High in plan ordering, a CLI command appending a priority record that plan and status prefer over request text, and the reader's urgent-with-reason rule (in outputs.rs/briefs.rs, not reader.md). New decisions go at the end of docs/decisions.md, with a repo test guarding the log's structure so the D121/D122 damage cannot recur. Audience: the user and the verifier running cargo checks.

Assumptions:
- The retry starts from base; the earlier broken D121/D122 edit is absent, so the fix is to append a new decision after the last entry (currently D121) with its own Provenance.
- The introspection's decisions-log check is folded in as a small test, as the earlier failure came from it; the feature remains the goal.
- Fixture and loader files outside the listed ones (store, status, fake agent, test common, docs/records.md) are likely needed for the new record kind and field.
- The reader set is at 1997 of 2000 tokens, so the urgent rule goes in outputs.rs or briefs.rs net of removals; reader.md does not grow.
- Existing decision-links.toml uses line numbers, so only appending to decisions.md keeps links valid.

## Acceptance criteria

1. [behavior] Run cargo test: a plan test shows urgent before high, high before normal, with a dependency still ordered first even when the dependent is urgent.
2. [behavior] Run cargo test: a test shows a priority record for a queued request overrides the priority in its text; the order in plan and the priority in status change, and no new request record is written.
3. [behavior] Run cargo test: a reading test parses a ReadingOutput with an urgent reason and with null, and shows the reason reaches the recorded Reading; a speed-improving request is marked urgent with a reason and an ordinary one is not.
4. [content] The new CLI command appends a priority record, rejects unknown priority values and unknown or non-queued requests with a clear error; Priority::from_text parses 'Priority: urgent'; main.rs help mentions urgent and the command.
5. [content] The urgent-marking rule (makes Athena faster or cuts wasted time: failed or rejected attempts, planning stops, timeouts, redo-work retries, thrash, duplicated checks; reason required) is in outputs.rs or briefs.rs; reader.md does not grow.
6. [behavior] Run cargo test --locked -p athena-cli --test budgets: the reader set stays within 2000 tokens.
7. [behavior] Run cargo test: a new test parses docs/decisions.md into entries and checks each has a Provenance bullet, ids are unique and ascending, and against the base commit existing entries are byte-identical with additions only at the end; it passes on the final file.
8. [content] CHANGELOG.md has an entry; docs/decisions.md has a new decision appended after the last entry with alternatives and its own Provenance, and D121 is unchanged from base.
9. [behavior] cargo fmt --check, cargo clippy --all-targets and the full cargo test pass.

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
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/records.md (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-core/src/pr_check.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)

## Out of scope

- Rewriting or reordering existing decisions in docs/decisions.md
- Growing reader.md
- Changing governance files such as athena.toml or defaults.toml
- Rewriting request records

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for ordering, override, reading parse path, and decisions log structure; fmt, clippy, test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and check the ordering, override and reading behavior, not that types exist.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry with alternatives and its own Provenance, plus a CHANGELOG entry.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One feature; the log guard is a small test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the named cargo commands.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The decisions-log structure rule is enforced by a repo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Guards the append-only log structure rather than only fixing the one bad insertion.
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
Run cargo test: a test shows a priority record for a queued request overrides the priority in its text; the order in plan and the priority in status change, and no new request record is written.
Run cargo test: a reading test parses a ReadingOutput with an urgent reason and with null, and shows the reason reaches the recorded Reading; a speed-improving request is marked urgent with a reason and an ordinary one is not.
The new CLI command appends a priority record, rejects unknown priority values and unknown or non-queued requests with a clear error; Priority::from_text parses 'Priority: urgent'; main.rs help mentions urgent and the command.
The urgent-marking rule (makes Athena faster or cuts wasted time: failed or rejected attempts, planning stops, timeouts, redo-work retries, thrash, duplicated checks; reason required) is in outputs.rs or briefs.rs; reader.md does not grow.
Run cargo test --locked -p athena-cli --test budgets: the reader set stays within 2000 tokens.
Run cargo test: a new test parses docs/decisions.md into entries and checks each has a Provenance bullet, ids are unique and ascending, and against the base commit existing entries are byte-identical with additions only at the end; it passes on the final file.
CHANGELOG.md has an entry; docs/decisions.md has a new decision appended after the last entry with alternatives and its own Provenance, and D121 is unchanged from base.
cargo fmt --check, cargo clippy --all-targets and the full cargo test pass.

## Report

End with the producer report as your structured output.
