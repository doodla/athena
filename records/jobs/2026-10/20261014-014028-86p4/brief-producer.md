# Job 20261014-014028-86p4: Add host-error record, landing watcher, decision-state; backtest D87

## Request (verbatim)

> Build step 4 of the build order in docs/design/decision-sets.md section 8: the host-error record written by the gh client (section 4), landing as a watcher trigger, the watcher with exact matching, the athena.decision-state/1 record, and the athena.default-expectations/1 record if the design still calls for it; hold back the brief component if it needs set selection (step 3) first. Two real cases from today: GitHub refused a merge commit with HTTP 405 (job 9h2j, PR #41), which falsified D87's assumption and must put D87 under review on the first occurrence; and 'athena land --pr' for job w4cj failed on a transient HTTP 500 from 'gh repos/doodla/athena' and needed a manual rerun. A transient 5xx should be recorded and retried a bounded number of times by Athena, and only a non-transient error reaches the user. Then extend 'athena backtest' so that, given a host-error record for the 405, it reports D87 under review.

## Reading (accepted by the user)

Implement decision-sets §8 step 4 minus the brief component: GhCli writes athena.host-error/1 (status_code, operation, closed message_class), a landing-time watcher with exact matching writes append-only athena.decision-state/1, and athena.default-expectations/1 is added. Transient 5xx calls are recorded and retried a bounded number of times; the 405 on merge puts D87 under review on first occurrence, and athena backtest reports it.

Assumptions:
- The brief component (hashing decision-state fold into the reading) is held back because the request says to hold it back if it needs step 3 set selection; step 3 is not built, so it is deferred.
- athena.default-expectations/1 is still called for by design §2 and §6 step 1, so only its record type, writer and reader are added, with no migration of the 12 defaults.
- Bounded retry applies to 5xx host statuses (and a status-less failure is not retried); the bound is a small constant (3 attempts) with no backoff sleep in tests, set via an injectable delay.
- Transient means HTTP 500, 502, 503, 504; 405 and 422 on merge are hard falsifiers; other statuses are non-transient and reach the user.
- Status is parsed from gh stderr text of the form 'HTTP NNN', pinned by a test; if absent, status_code is absent.
- D87 gets its typed header with the hard host-error signal in code (the decision view in decisions.rs/backtest.rs), not by editing docs/decisions.md.
- Record kinds are added to athena-data records.rs and a records directory, following existing conventions; schema docs in docs/records.md are updated.
- A CHANGELOG.md entry is added per D89.

## Acceptance criteria

1. [content] athena-data defines athena.host-error/1 with status_code (optional integer), operation (closed list including merge, create-pr, read-repo), job, phase, at, and a closed message_class; the raw stderr text is not a stored field.
2. [content] athena-data defines append-only athena.decision-state/1 (decision id, state, firing record ids, date) with a fold to current state, and athena.default-expectations/1 (default_id, assumptions, signals, defaults version, proposal).
3. [content] The gh client extracts the HTTP status from a refused call, writes a host-error record, and retries a 5xx a bounded number of times before returning the error; non-5xx errors are not retried.
4. [content] The land path invokes the watcher after a host error and at landing; the watcher loads D87's hard signal (operation merge, status 405 or 422, one job), validates values against the vocabulary, and does not evaluate records from a firing's own introspection.
5. [content] No brief component and no set selection are added; BRIEF.md, defaults.toml, athena.toml and records/ are unchanged.
6. [content] docs/design/decision-sets.md §8 step 4 and docs/records.md describe what was built and what was held back; CHANGELOG.md has an entry.
7. [behavior] Run cargo test: a test where the fake gh returns HTTP 500 twice then success lands with 2 host-error records and no user-facing error; a test with persistent 500 stops after the bound and surfaces the error.
8. [behavior] Run cargo test: a test where merge returns HTTP 405 writes one host-error record and one decision-state record putting D87 under review on the first occurrence; a 404 or a 500 does not.
9. [behavior] Run cargo test: athena backtest over a temp records fixture containing the 405 host-error record reports D87 under review, with the record id cited; without the record it does not.
10. [behavior] Run cargo clippy --all-targets and cargo fmt --check: both clean.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/watcher.rs (create)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-data/src/paths.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- docs/design/decision-sets.md (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)

## Out of scope

- Brief component for decision-state and set selection (step 3)
- Jev / evaluate calls
- Migrating defaults or typed headers for D01-D90
- Editing docs/decisions.md, defaults.toml, athena.toml or BRIEF.md
- Backtesting other signals (step 5 beyond D87 and the 405)
- Clearing under-review state

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust in existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the 405 record, the 500 retry then success, retry exhaustion, the watcher firing on first 405, no-fire on soft or unrelated errors, and the backtest report; clippy clean.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Follows the reviewed design in decision-sets.md; the doc status line is updated to mark step 4 built.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Brief component and set selection are left out to keep the change focused.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Retry and recording are done in the host client, not in the land command only.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Firing is enforced by code-evaluated exact matching and tests, not prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by cargo test, including backtest run against a fixture.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Host-error records omit raw stderr; no personal data.
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
