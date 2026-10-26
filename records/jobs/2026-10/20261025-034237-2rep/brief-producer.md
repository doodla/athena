# Job 20261025-034237-2rep: Make model and effort explicit per role; escalate hard producer jobs

## Request (verbatim)

> Priority: high
> Manage and upgrade the intelligence used in planning, and make effort explicit (BRIEF.md addendum of 2026-10-25, 'Tell Athena to manage and upgrade intelligence in planning. Effort should also be explicit.'). Today the planner is always claude-sonnet-5-5 and the critic claude-opus-5-5 at the CLI's default effort, whatever the job; 35% of plans pass the first round and 16 of 44 accepted plans still failed or were rejected. Make the model and the effort level of every agent invocation explicit in config per role, recorded on each invocation record, and let the reading choose a stronger planner model or higher effort for a job it judges hard (for example one that changes landing, the history check or records), with the choice and its reason recorded. Measure the effect: per model and effort, the first-round accept rate, the landed share and the cost per landed job, shown in athena status. Absorbs request 20261013-042934-a1j8 (record each invocation's effort level).
> Acceptance:
> - Every invocation record names its model and effort, and a test shows both reach the agent driver.
> - A test shows a reading that marks a job hard gets the stronger planner setting, with its reason recorded, and an ordinary job keeps the default.
> - athena status shows the planning measures broken down by model and effort.
> Files: athena.toml, crates/athena-data/src/config.rs, crates/athena-data/src/records.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/run.rs, crates/athena-core/src/status.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Config names model and effort for the reader, producer and verifier. Both reach the claude driver and are recorded on every invocation record. A reading that judges a job hard picks a stronger producer model or higher effort, with the reason recorded. athena status shows first-pass verification rate, landed share and cost per landed job by producer model and effort. Audience: the Athena owner tuning quality. Planner and critic are dropped from scope because planning is off.

Assumptions:
- Planner and critic roles are out because job planning is off (f80b05a); only reader, producer and verifier are configured.
- The reading output gains a hard flag and reason field; the escalation picks the producer model or effort from a configured hard setting.
- Effort values are a closed set matching the CLI's levels; the exact list comes from the claude driver's supported flag.
- Existing records stay loadable via optional fields.

## Acceptance criteria

1. [behavior] A test (cargo test) shows the configured model and effort for each role reach the claude driver as CLI arguments and appear on that invocation's record.
2. [behavior] A test (cargo test) shows a reading marking a job hard gets the stronger producer model or effort with its reason recorded, while an ordinary reading keeps the configured default and records no escalation reason.
3. [behavior] A test (cargo test) shows athena status output lists first-pass verification rate, landed share and cost per landed job per producer model and effort. A bucket with no landed jobs shows no division by zero and no false success.
4. [content] athena.toml (listed as a deliverable) sets model and effort for reader, producer and verifier, plus the hard-job producer setting. config.rs parses them and rejects unknown effort values. Older records without effort still load.
5. [content] CHANGELOG.md has an entry that notes request 20261013-042934-a1j8 is absorbed. docs/decisions.md records the decision with alternatives, including dropping the planner/critic roles because planning is off.
6. [behavior] cargo test, cargo clippy and cargo fmt --check pass.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Planner and critic roles (planning is off)
- Re-enabling job planning
- Changing the claude_version pin

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for each behavior; cargo test and clippy are clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the driver arguments, the escalation and the status output, not just that fields exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Status handles buckets with zero landed jobs explicitly.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Config validation rejects unknown effort values.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and changelog entries are written for an outside reader.
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

A test (cargo test) shows the configured model and effort for each role reach the claude driver as CLI arguments and appear on that invocation's record.
A test (cargo test) shows a reading marking a job hard gets the stronger producer model or effort with its reason recorded, while an ordinary reading keeps the configured default and records no escalation reason.
A test (cargo test) shows athena status output lists first-pass verification rate, landed share and cost per landed job per producer model and effort. A bucket with no landed jobs shows no division by zero and no false success.
athena.toml (listed as a deliverable) sets model and effort for reader, producer and verifier, plus the hard-job producer setting. config.rs parses them and rejects unknown effort values. Older records without effort still load.
CHANGELOG.md has an entry that notes request 20261013-042934-a1j8 is absorbed. docs/decisions.md records the decision with alternatives, including dropping the planner/critic roles because planning is off.
cargo test, cargo clippy and cargo fmt --check pass.

## Report

End with the producer report as your structured output.
