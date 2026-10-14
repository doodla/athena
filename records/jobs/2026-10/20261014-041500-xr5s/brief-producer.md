# Job 20261014-041500-xr5s: Add `athena retry <job>` seeded from the introspection record

## Request (verbatim)

> Build step 2 of the build order in docs/design/defaults-loop.md section 12: retry-from-record (section 3.5). Today, after a rejected or failed job, the user writes the retry request by hand, folding in the verifier's findings and the cause; today's examples include 69gf, t1y6, 3p3y and jk8v. Instead, a retry of a rejected or failed job starts from its introspection record: the proposed next request, cause and prevention, plus the original scope and accepted corrections that retries already carry. Give the user one short command to accept it, such as 'athena retry <job>', which shows the reading as 'ask' does. Earlier cases of the same class should be visible to the reader of the retry, so a fourth repeat is seen as a repeat. Verify with fixture records in a temporary directory.

## Reading (accepted by the user)

Add an `athena retry <job>` command that builds the retry request for a rejected or failed job from its introspection record (proposed_request, cause, prevention, earlier cases), plus the original scope and accepted corrections already carried by "try again on". It then reads and shows the reading as `ask` does. The reader's brief shows earlier cases of the same class and any prevention already tried, so a repeat is visible as one. Fixture records in a temp directory verify it.

Assumptions:
- `athena retry <job>` goes through the same flow as `ask` (reading shown, Enter accepts on a terminal, --yes and --json supported). It never submits a run on its own, per design section 3.5.
- The introspection record is read from the job's records through the existing introspection view/record types; the retry request text is the stored proposed_request, with cause, prevention and earlier-case ids added to the EarlierAttempt carried to the reader.
- If the job has no introspection record or no proposed_request, the command falls back to today's 'try again on' behavior and says so.
- Earlier cases come from the stored earlier_cases ids on the introspection; the retry does not re-run class matching or Jev.
- Only rejected or failed jobs can be retried, as with earlier_attempt today.
- The design doc's build-order step 2 is left as-is, with at most a short status note.

## Acceptance criteria

1. [behavior] Run `cargo test`: a new test builds fixture records in a temp directory (a rejected job with an introspection record) and shows that retrying it yields a reading input whose request carries the proposed_request, cause and prevention, the original request and the accepted corrections.
2. [behavior] Run `cargo test`: a fixture test shows earlier cases of the same class (ids and one-line what) appear in the retry's reader brief, and that a prevention already tried in an earlier introspection of the same class is shown as already tried.
3. [behavior] Run `cargo test`: a fixture test shows `retry` on a job that is not rejected or failed is refused with a clear message. A second shows a job with no introspection falls back to the 'try again on' behavior.
4. [behavior] Run `cargo run -- retry --help`: the command exists and takes a job argument, with the same --yes and --json options as `ask`.
5. [content] crates/athena-cli/src/main.rs has a `Retry` subcommand that goes through the same reading display as `ask`, and does not submit the run unless the user accepts.
6. [content] The reader brief text in athena-context includes the cause, prevention and earlier-case section only when the retry came from an introspection record, with a test of that.
7. [behavior] Run `cargo test`, `cargo clippy` and `cargo fmt --check`: all pass with no new warnings.
8. [content] CHANGELOG.md has an entry for the new command, and README or CLI docs mention `athena retry <job>` if they list commands.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/retry.rs (create)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Steps 1 and 3 or later of the build order.
- Changing introspection generation, class matching or the class registry.
- Automatic submission of the retry.
- Changes to governance files: athena.toml, defaults.toml, CI, toolchain and lint configuration.
- Any change to records/.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests use fixture records in a temp directory; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command and its reader-brief wiring, reusing the existing earlier_attempt path.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is exercised by running the fixture tests and the CLI help.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Details came from design section 3.5, the existing reading.rs retry code and the introspection types.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments, CHANGELOG entry and messages are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The retry starts from the cause record, not the last finding, as C2 in the design requires.
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
