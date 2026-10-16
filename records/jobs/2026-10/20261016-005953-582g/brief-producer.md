# Job 20261016-005953-582g: Make drain work user-queued (ask --later) requests too

## Request (verbatim)

> try again on 20261016-003525-5wwj

## Reading (accepted by the user)

steward::queued_work keeps only channel "steward", so requests from `ask --later` (channel "cli") are never drained. Include both channels in the shared selection, keep ordering and retry rules unchanged, and prove it with tests using fresh ids. Audience: the user who queues requests and expects drain to work them.

Assumptions:
- `ask --later` records channel "cli" (as the earlier reading found); the fix is the channel filter in queued_work at steward.rs ~line 341.
- Selection is by channel set, never by job id; tests use ids other than the four named ones.
- Final report should include a criterion-to-evidence table, but that is a report-format matter, not a repository deliverable.
- Retry and ordering logic (received_at, then job id) stay untouched.

## Acceptance criteria

1. [behavior] A unit test in steward.rs, with job ids unlike the named ones, shows queued_work returns unread Queued requests from both the steward and cli channels, oldest first (ties by job id), and excludes non-Queued jobs and other channels. Run `cargo test -p athena-core queued_work`.
2. [behavior] A drain test with a cli-channel queued request (fresh id) in the records shows drain's queue lists it and works it in the same way as a steward item. Run `cargo test -p athena-cli drain`.
3. [behavior] Existing drain ordering and retry/thrash tests pass unchanged and are not edited to fit. Run `cargo test`.
4. [content] The change is a channel-set filter in queued_work (steward.rs), with no job-id special case; a grep of the diff finds none of the named ids in non-test code.
5. [content] Doc comments saying drain takes only the steward channel (steward.rs REQUEST_CHANNEL, drain.rs, docs/decisions.md ~line 2206 area) are corrected, and a new decision entry and a CHANGELOG line record the change and why.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- Changing drain ordering, retry or thrash rules
- Adding a harness gate for producer reports (already tried in earlier cases)
- Changing how `ask --later` records requests
- Editing athena.toml, defaults.toml or workflows

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the shared channel filter, not the named jobs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy, fmt pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests with fresh ids showing drain works the cli request.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decision entry explain the change for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to queue selection with docs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the named cargo test commands.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
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
