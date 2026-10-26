# Job 20261025-033740-5d0n: Add job-planning keep-rule evaluation and breach line to athena status

## Request (verbatim)

> Priority: high
> Have athena status evaluate the job-planning keep rule (docs/design/job-planning.md section 9, decision D110) and flag it: the landed share of planned jobs is 28 of 51 (55%), below the 64% baseline the rule holds planning to through the first 30 planned jobs, and nothing reported it. Compute the landed share of planned dispatched jobs, cost per landed job and the first-round accept rate, compare them with the rule at its thresholds (30 and 100 planned jobs), and show a breach in status with the numbers.
> Acceptance:
> - A status test with fixture records shows the measures and a breach line when the share is below the baseline.
> - athena status --json carries the measures.
> Files: crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a pure keep-rule evaluation in status.rs computing, over planned dispatched jobs, the landed share (stopped-in-planning counts as not landed), cost per landed job and the first-round accept rate. It compares them with job-planning.md §9 at the 30 and 100 planned-job thresholds, adds the result to StatusView (so `--json` carries it), and display.rs prints the measures and a breach line. The audience is the operator reading status; the real data case is 28/51 = 55% against the 64% baseline.

Assumptions:
- The 28 of 51 figure is from the request; the implementation computes it from records rather than hard-coding it, and fixtures reproduce that shape.
- Planned jobs means request jobs with at least one job-plan record and a dispatch; a job stopped in planning counts as dispatched and not landed.
- Cost per landed job is total cost of planned jobs divided by landed ones, compared with the $2.50 ceiling at 100 planned jobs; the first-round accept rate reuses the existing FirstRoundCounts logic.
- Through 30 planned jobs a share below 0.64 is a breach; at 100 the rule's keep / turn off / inconclusive outcomes apply.
- The rule only reports; it does not turn planning off in athena.toml.

## Acceptance criteria

1. [behavior] A status.rs unit test builds fixture records with 51 planned dispatched jobs, 28 landed, and asserts the measures (landed 28, planned 51, share about 0.549, cost per landed job, first-round accept rate) and a breach verdict because the share is below 0.64. Run: cargo test -p athena-core.
2. [behavior] Fixture tests cover the rule's branches: at most 30 planned jobs with share at or above 0.64 reads keep; below 0.64 reads breach; at 100 or more planned jobs, share at least 0.72 with cost per landed job at most $2.50 reads keep, share at or below 0.64 reads turn off, otherwise inconclusive; zero landed gives no division by zero. Run: cargo test -p athena-core.
3. [behavior] A display.rs test shows the rendered status text contains the measures with their numbers (28 of 51, 55%, baseline 64%, cost per landed job, first-round accept rate) and a breach line when breached, and no breach line when kept. Run: cargo test -p athena-cli.
4. [behavior] The StatusView serialization (what `athena status --json` prints) contains a keep-rule object with the measures and verdict; a test asserts the JSON keys and values. Run: cargo test.
5. [content] The denominator counts planned dispatched jobs, a job stopped in planning counts as not landed, and cost per landed job divides all cost of those jobs by landed jobs, as section 9 defines it; the thresholds (30, 100, 64%, 72%, $2.50) are named constants citing job-planning.md section 9 and D110.
6. [content] CHANGELOG.md has an entry for the keep-rule evaluation in status.
7. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Automatically disabling planning in athena.toml
- Changing the keep rule's thresholds or job-planning.md
- Hestia checks that schedule the evaluation

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture tests in both crates; cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the measures and breach line appear in rendered status and JSON.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is a breach line with numbers shown when the share is below baseline.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change in the three named files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria runnable by the verifier with cargo test.
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

A status.rs unit test builds fixture records with 51 planned dispatched jobs, 28 landed, and asserts the measures (landed 28, planned 51, share about 0.549, cost per landed job, first-round accept rate) and a breach verdict because the share is below 0.64. Run: cargo test -p athena-core.
Fixture tests cover the rule's branches: at most 30 planned jobs with share at or above 0.64 reads keep; below 0.64 reads breach; at 100 or more planned jobs, share at least 0.72 with cost per landed job at most $2.50 reads keep, share at or below 0.64 reads turn off, otherwise inconclusive; zero landed gives no division by zero. Run: cargo test -p athena-core.
A display.rs test shows the rendered status text contains the measures with their numbers (28 of 51, 55%, baseline 64%, cost per landed job, first-round accept rate) and a breach line when breached, and no breach line when kept. Run: cargo test -p athena-cli.
The StatusView serialization (what `athena status --json` prints) contains a keep-rule object with the measures and verdict; a test asserts the JSON keys and values. Run: cargo test.
The denominator counts planned dispatched jobs, a job stopped in planning counts as not landed, and cost per landed job divides all cost of those jobs by landed jobs, as section 9 defines it; the thresholds (30, 100, 64%, 72%, $2.50) are named constants citing job-planning.md section 9 and D110.
CHANGELOG.md has an entry for the keep-rule evaluation in status.
cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Report

End with the producer report as your structured output.
