# Job 20261026-042558-5p65: Add code-health check, status table, and design-review cadence

## Request (verbatim)

> Priority: urgent
> Code health is watched, not pointed out (BRIEF.md addendum of 2026-10-26, 'The question is I had to point it out'). Add a code-health check: per file, production line count and the longest function; per file, the number of landings that changed it in the last 7 days and how many failures or rejections cited it; and the test suite's run time. Show it in athena status with thresholds (for example a production file over 2,000 lines, a function over 200 lines, a file both over 1,500 lines and changed in 5 or more landings in a week), and when a threshold is crossed queue a structural-pass request for that file, citing the numbers, unless one is already open. Run it after every landing. Also run the periodic design review (request 20261017-205558-dn2e) every 10 landed jobs on its own cadence, queuing it at high priority.
> Acceptance:
> - A test with fixture files and landing records shows the measures, a threshold breach, and exactly one structural-pass request queued for it (none when one is open).
> - athena status shows the code-health table.
> - A test shows the design-review request is queued at the 10th landing since the last one.
> Files: crates/athena-core/src/code_health.rs, crates/athena-core/src/status.rs, crates/athena-core/src/gh.rs, crates/athena-cli/src/display.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Add crates/athena-core/src/code_health.rs measuring per-file production lines, longest function, 7-day landing churn, failure citations, and test run time. Show it in athena status with thresholds, queue one structural-pass request per breaching file unless one is open, run it after each landing, and queue the design review at high priority every 10 landings. The audience is Sri and the verifier, who need breaches caught without being pointed out.

Assumptions:
- Goal: Athena watches its own code health and queues structural passes itself, so Sri need not point problems out.
- Production lines exclude #[cfg(test)] modules; function length is measured with a Rust parser (syn, if already a dependency) or a simple brace-matching scan, and the choice is recorded in decisions.md.
- Landings per file come from landing records in the state dir; failures cited come from job failure and rejection records that name the file.
- Thresholds are the example values from the request, held as constants or in config, with no change to athena.toml.
- The design-review role from job 20261017-205558-dn2e already exists; this job only adds a landing-count trigger that queues it at high priority every 10 landings, separate from the steward tick.
- Test suite run time is measured when the check runs, or read from the last recorded run, and not by running cargo test inside the check on every landing; the producer picks one and notes it.
- The 'run after every landing' hook goes in the existing post-landing path, likely in gh.rs or land.rs.

## Acceptance criteria

1. [behavior] Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in last 7 days, and failure/rejection citations; a breach of each threshold (file >2000 lines, function >200 lines, >1500 lines with >=5 landings in a week) is reported.
2. [behavior] Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; running the check again, or with an open structural-pass request for that file, queues none.
3. [behavior] Run cargo test: the design-review request is queued at high priority at the 10th landed job since the last review and not at the 9th, on its own cadence independent of the steward's.
4. [behavior] Run cargo test: the check runs after every landing (a test of the post-landing path shows it invoked), and the test suite run time is recorded in the report.
5. [behavior] Run cargo test: the athena status output (rendered in a test) contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, and test run time.
6. [behavior] cargo clippy with -D warnings and cargo fmt --check are clean and cargo test passes.
7. [content] CHANGELOG.md has an entry and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length, and the 10-landing cadence.

## Expected deliverables

- crates/athena-core/src/code_health.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-core/src/land.rs (modify)

## Out of scope

- Performing the structural-pass refactors themselves
- Changing the design-review role's prompt or its output format
- Changing the steward's cadence or behavior
- Changing athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-based unit tests for the measures, queue dedup, and the cadence, with clippy and fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the breach produces exactly one queued request and the status table renders, not that the code exists.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse an existing parser dependency for function length if one is present, and record the call in decisions.md.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checkable with cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module plus thin hooks into status, display and the landing path.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The check runs automatically after each landing and queues requests by code, not by prose.
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

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in last 7 days, and failure/rejection citations; a breach of each threshold (file >2000 lines, function >200 lines, >1500 lines with >=5 landings in a week) is reported.
Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; running the check again, or with an open structural-pass request for that file, queues none.
Run cargo test: the design-review request is queued at high priority at the 10th landed job since the last review and not at the 9th, on its own cadence independent of the steward's.
Run cargo test: the check runs after every landing (a test of the post-landing path shows it invoked), and the test suite run time is recorded in the report.
Run cargo test: the athena status output (rendered in a test) contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, and test run time.
cargo clippy with -D warnings and cargo fmt --check are clean and cargo test passes.
CHANGELOG.md has an entry and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length, and the 10-landing cadence.

## Report

End with the producer report as your structured output.
