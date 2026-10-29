# Job 20261029-142849-04d9: Add [drain.pipeline] config table with validation and safe defaults

## Request (verbatim)

> Priority: urgent
> Async drain build job A2 (docs/design/async-drain.md, decision D145). Add the [drain.pipeline] config table (design section 11 and 12, job 2): schema, validation and defaults that keep today's behavior (max_in_flight = 1), with tests. With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-data/src/config.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a `drain` section with a `pipeline` table (max_in_flight, max_check_slots, read_ahead, reserve_disk_gb, max_conflict_returns) to athena-data's Config, with design section 11 defaults (1,1,1,10,2), validation, tests, and a CHANGELOG entry. Config-only: nothing reads the values yet, so drain behaves as today; audience is the later build jobs and reviewers of the async-drain design.

Assumptions:
- Code survey: read crates/athena-data/src/config.rs. Entry point is Config::parse (toml::from_str with deny_unknown_fields, then Config::validate, which returns ConfigError variants); reading_inputs resets non-reading sections (parallel, landing, job_plan, disk) before digesting. No probing/resolution order applies beyond parse then validate.
- Code survey: no record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved; the records allow-list is untouched.
- Design: docs/design/async-drain.md sections 11 and 12 (job 2). Defaults: max_in_flight=1, max_check_slots=1, read_ahead=1, reserve_disk_gb=10, max_conflict_returns=2. Absent [drain] must equal those defaults.
- Following the `disk`/`job_plan` pattern: the new `drain` field is Option, skipped when None in serialization and reset in reading_inputs, so existing config digests and readings stay valid.
- Validation bounds are not given by the design. I assume max_in_flight 1..=8 (matching MAX_CONCURRENCY_CEILING), max_check_slots >= 1 and <= max_in_flight-independent small ceiling, read_ahead >= 1, max_conflict_returns >= 1, reserve_disk_gb any non-negative u64; a ConfigError variant names the offending key.
- Config-only: no drain runtime code is changed and athena.toml is not edited (job 10 raises max_in_flight). Higher refusal rate: the change refuses only configs that newly contain a [drain.pipeline] table with out-of-range values, so expected 0% of newly refused past readings.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-data`: tests through Config::parse show an athena.toml with no [drain] section yields pipeline values 1,1,1,10,2, and an explicit [drain.pipeline] with those values parses equal to it.
2. [behavior] Run `cargo test -p athena-data`: tests through Config::parse show max_in_flight = 0 and above the ceiling, max_check_slots = 0, read_ahead = 0, max_conflict_returns = 0 and an unknown key under [drain.pipeline] are each rejected with an error naming the key.
3. [behavior] Run `cargo test -p athena-data`: a test shows Config::reading_inputs is identical with and without a non-default [drain.pipeline], and the repository's real athena.toml still parses with max_in_flight resolving to 1.
4. [behavior] Run `cargo test --workspace`: the existing drain and config tests pass with no edits to existing test bodies.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`: both succeed.
6. [content] CHANGELOG.md has a new entry describing the [drain.pipeline] table, its defaults, and that max_in_flight = 1 keeps today's behavior.
7. [content] Each new public item in config.rs has a doc comment stating its meaning and default, citing docs/design/async-drain.md section 11; the change expects 0% of newly refused readings because absent or default tables stay valid.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Reading or acting on the new values in the drain runtime (later build jobs)
- Editing athena.toml or raising max_in_flight to 3 (build job 10)
- Disk-guard, check-slot, read-ahead or conflict-return behavior
- Changes to defaults.toml or .github/workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust in athena-data.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; cargo test and clippy -D warnings required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise Config::parse outcomes, not struct existence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to config.rs plus the changelog.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite the design section for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check parsed defaults and refusals, not activity.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
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

20 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

Run `cargo test -p athena-data`: tests through Config::parse show an athena.toml with no [drain] section yields pipeline values 1,1,1,10,2, and an explicit [drain.pipeline] with those values parses equal to it.
Run `cargo test -p athena-data`: tests through Config::parse show max_in_flight = 0 and above the ceiling, max_check_slots = 0, read_ahead = 0, max_conflict_returns = 0 and an unknown key under [drain.pipeline] are each rejected with an error naming the key.
Run `cargo test -p athena-data`: a test shows Config::reading_inputs is identical with and without a non-default [drain.pipeline], and the repository's real athena.toml still parses with max_in_flight resolving to 1.
Run `cargo test --workspace`: the existing drain and config tests pass with no edits to existing test bodies.
Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`: both succeed.
CHANGELOG.md has a new entry describing the [drain.pipeline] table, its defaults, and that max_in_flight = 1 keeps today's behavior.
Each new public item in config.rs has a doc comment stating its meaning and default, citing docs/design/async-drain.md section 11; the change expects 0% of newly refused readings because absent or default tables stay valid.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
