# Job 20261025-171957-p5ns: Derive evidence-based class labels for Jev scoring

## Request (verbatim)

> Priority: high
> Label Jev's class answers from evidence, not opinion (BRIEF.md addendum of 2026-10-25, 'Evidence should label jev imo'). athena score-jev needs labeled introspections (at least 10 in its test part) and has none: 187 introspections carry a recorded Jev shadow answer, and the only labeling path today is a person's athena review. Derive labels from what happened afterwards, each with the records it rests on: (1) the retry that followed the introspection's proposed request and prevention landed, or failed for the same cause; (2) after a prevention for the class landed, the class stopped or kept recurring; (3) the critic's or verifier's findings on the same job name the same cause. Write each derived label as a class-label record marked evidence-derived with its basis, leave cases with no evidence unlabeled, and let athena review still override. Then score Jev with score-jev.
> Acceptance:
> - A test with fixture records shows each of the three evidence kinds producing a label with its basis, and a case with no evidence left unlabeled.
> - Labels are records (append-only); a person's review label takes precedence.
> - Run on this repository's records, score-jev reports how many labels were derived and, if enough, a score and verdict.
> Files: crates/athena-core/src/introspect.rs, crates/athena-core/src/jev.rs, crates/athena-data/src/records.rs, crates/athena-cli/src/main.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Add an evidence-derived labeling path that writes append-only class-label records (labeler "evidence", with a basis citing record ids) from three kinds of later evidence: retry outcome after the proposed request/prevention, class recurrence after prevention, and critic/verifier findings naming the same cause. Cases without evidence stay unlabeled, a person's review label always wins, and score-jev reports the derived count and a score and verdict if at least 10 test rows exist. The audience is the maintainer running athena score-jev.

Assumptions:
- Labeler gains an evidence-derived variant and ClassLabel gains an optional basis field (list of cited record ids with the evidence kind), backward compatible with existing records.
- Derivation runs from a CLI step (a flag on athena score-jev or a subcommand) that appends labels not already derived, so reruns are idempotent.
- Precedence is implemented in newest_labels/build: any person or stronger-model label beats an evidence label regardless of time.
- Where evidence kinds conflict, the case is left unlabeled rather than guessed.
- Derived labels are real records written to records/ on this repository's run; the score-jev result is reported honestly even if under 10 test rows (no verdict then).
- Derivation logic may live in a new module in athena-core if introspect.rs/jev.rs would become too large.

## Acceptance criteria

1. [behavior] Run cargo test: a fixture-record test shows each of the three evidence kinds (retry outcome, recurrence after prevention, critic/verifier same-cause findings) yielding a class-label with evidence-derived labeler and a basis citing the record ids it rests on.
2. [behavior] Run cargo test: a fixture introspection with no evidence gets no label and counts as unlabeled.
3. [behavior] Run cargo test: when both a person label and an evidence label exist for one introspection, the person label is used by the labeled set, whichever is newer; and derivation never edits or deletes an existing record, and rerunning does not duplicate labels.
4. [behavior] Run cargo run -- score-jev on this repository's records: output states how many labels were derived from evidence (and per evidence kind) and, if the test part has at least 10 rows, a score and verdict; otherwise states the shortfall with no verdict.
5. [content] ClassLabel schema and docs/records.md-relevant schema tests accept old labels without a basis and round-trip new ones; CHANGELOG.md and docs/decisions.md record the evidence-derived labeling call and its alternatives.
6. [behavior] cargo clippy and cargo fmt --check are clean.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/evidence_labels.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/review.rs (modify)
- crates/athena-cli/tests/labeled.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/records.md (modify)

## Out of scope

- Changing Jev's prompt or model
- Editing or deleting existing records
- Changing the MIN_TEST_CASES threshold or scoring formulas
- Changing athena review's own flow beyond precedence

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-record tests per evidence kind, plus cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check labels produced with basis, and precedence, by running tests and score-jev.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Zero derived labels is reported as such, never as a passing score.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Run score-jev on the repository's records.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and decisions entry written for outside readers.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Labels come from outcomes, not from Jev's own answers.
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

Run cargo test: a fixture-record test shows each of the three evidence kinds (retry outcome, recurrence after prevention, critic/verifier same-cause findings) yielding a class-label with evidence-derived labeler and a basis citing the record ids it rests on.
Run cargo test: a fixture introspection with no evidence gets no label and counts as unlabeled.
Run cargo test: when both a person label and an evidence label exist for one introspection, the person label is used by the labeled set, whichever is newer; and derivation never edits or deletes an existing record, and rerunning does not duplicate labels.
Run cargo run -- score-jev on this repository's records: output states how many labels were derived from evidence (and per evidence kind) and, if the test part has at least 10 rows, a score and verdict; otherwise states the shortfall with no verdict.
ClassLabel schema and docs/records.md-relevant schema tests accept old labels without a basis and round-trip new ones; CHANGELOG.md and docs/decisions.md record the evidence-derived labeling call and its alternatives.
cargo clippy and cargo fmt --check are clean.

## Report

End with the producer report as your structured output.
