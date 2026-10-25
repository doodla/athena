# Job 20261014-020435-qb2r: Fix D87 intervention class; derive host-error records from interventions

## Request (verbatim)

> Two follow-ups to job 86p4, from running 'athena backtest' on the real records (condition (a) now passes; (b) fails). First, the D87 intervention signal still matches class 'docs'; docs/design/decision-sets.md section 9 revised it to class 'code', so fix the code to match. Second, the 405 that falsified D87 (PR #41, job 9h2j) happened before host errors were recorded; it exists only in intervention 20261013-192156-ar1x, whose text says GitHub refused a merge commit with HTTP 405. Add a way to derive host-error records from past interventions whose text states a host refusal: the derived record is marked derived, cites its source intervention, and is never presented as having been observed by the client. Derivation is a typed decision that may later use Jev, but here use a strict exact pattern in code ('HTTP <code>' plus a merge operation). Rerunning the backtest should then flag D87 under review, citing ar1x through the derived record, and flag nothing else. Report the real result either way.

## Reading (accepted by the user)

Change D87's intervention signal in backtest.rs from class 'docs' to 'code' (per decision-sets.md section 9). Add a strict-pattern derivation ('HTTP <code>' plus a merge operation) that turns past interventions into host-error records marked derived and citing the source intervention. The backtest uses those records, so rerunning it should flag D87 via ar1x and nothing else; the real result is reported either way.

Assumptions:
- Derived records are held in memory during the backtest and not written under records/ (records/ never changes in a job).
- HostError gains an optional provenance field (derived, with the source intervention id). It is absent for observed records, so existing records still parse.
- The pattern is matched against the intervention's what and why text: 'HTTP <3-digit code>' plus the word 'merge'. The job, at, and phase come from the intervention (landing phase for a merge). If the intervention has no job, nothing is derived.
- Derived records are distinguishable everywhere they are listed, and the report never says the client observed them.
- The exact-match intervention signal for D87 also needs the intervention text read in the backtest, so InterventionRow gains what/why or the derivation reads the full record.
- Whether the real run flags only D87 depends on records/ data, which I cannot see; the producer reports the actual result.

## Acceptance criteria

1. [content] backtest.rs D87's intervention signal uses class_of_work "code" (name and doc comments updated to say code), matching decision-sets.md section 9.
2. [content] The HostError record carries a derived marker and the source intervention id. An observed record has neither, and record validation rejects a derived record without a source.
3. [content] A derivation function takes an intervention and returns a derived HostError only when its text matches 'HTTP <code>' plus a merge operation exactly; otherwise it returns none. Unit tests cover a match, a non-match, a missing job, and an out-of-range code.
4. [content] The backtest report lists derived records separately from observed ones, labels them derived, and cites the source intervention id. D87's firing cites the derived record and, through it, ar1x.
5. [behavior] Run `cargo test` and `cargo clippy --all-targets`, then `cargo fmt --check`: all pass. This includes a backtest test with a fixture intervention whose text states an HTTP 405 on a merge, showing D87 flagged under review through a derived record, and a fixture with no such text showing nothing flagged.
6. [behavior] `cargo run -p athena-cli -- backtest` (or the actual subcommand form) runs. The producer's report states the real result for D87 and for all other decisions, whatever it is.

## Expected deliverables

- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/tests/backtest.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- CHANGELOG.md (modify)
- docs/design/decision-sets.md (modify)

## Out of scope

- Using Jev or any model to derive records.
- Writing derived records under records/.
- Changing the landing watcher's behavior on live host errors.
- Clearing a decision's review state.
- Changing D88 or any decision other than D87.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and integration tests for the derivation and the backtest; clippy clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The report cites the source intervention and states the real backtest result.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The derived marker is a typed field on the record, not a special case in the backtest.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the derived marker and the exact pattern.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The backtest is run and its result reported.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change covering both follow-ups; no extra features.
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

## Report

End with the producer report as your structured output.
