# Job 20261028-174834-xncn: Add reading size gate comparing forecast wall time to deadlines

## Request (verbatim)

> try again on 20261027-054148-g63s

## Reading (accepted by the user)

Retry of job 20261027-054148-g63s (dispatched, never verified, no findings recorded, so nothing to fix but the original work remains undone): add a deterministic size gate in build_reading (crates/athena-core/src/reading.rs) that refuses a reading whose own forecast wall_time_s.p50 exceeds config.budget.producer_timeout_s or verifier_timeout_s, telling the reader to pick one deliverable and name the rest as follow-up requests. Audience: Athena maintainers and the reader loop, which receives a clear retry message.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: build_reading (line ~499) parses ReadingOutput, validates class, capability, producer model, then builds the Forecast and calls reader.validate(); check_reading_acceptance (line ~2231) runs later with no config access. The gate goes in build_reading right after forecast validation, using jc.config.budget.producer_timeout_s and verifier_timeout_s (defined in crates/athena-data/src/config.rs lines ~232-233). Records validation is unaffected: no record or data file is added, so check_record_file and AnyRecord are untouched. The property is measured directly: the reading's own wall_time_s.p50 against the configured timeouts.
- Job 20261027-054148-g63s's earlier reading used the name limits.producer_timeout_s; the actual config path is budget.producer_timeout_s, corrected here. The gate is a threshold on existing config values; no new config keys are added, so athena.toml needs no change (named out of scope).
- Deliverables crates/athena-context/src/outputs.rs (reader output guidance wording, if the retry message needs mirroring) and crates/athena-data/src/config.rs are kept in the allowed scope from the earlier job but are unlikely to change; the work is mostly reading.rs and CHANGELOG.md.
- Root cause named in the change: reading validation never compares a reading's scope with the producer and verifier deadlines, so multi-item sources yield readings no single run can finish.

## Acceptance criteria

1. [content] The change description and a doc comment on the gate in crates/athena-core/src/reading.rs name the root cause: reading validation never compared reading scope with the producer/verifier deadline; the fix sits in build_reading.
2. [behavior] A new unit test in athena-core builds a reading whose forecast wall_time_s.p50 exceeds budget.producer_timeout_s and asserts build_reading returns an error whose message tells the reader to pick one deliverable and name the rest as follow-up requests. Run cargo test -p athena-core.
3. [behavior] A companion test shows a reading with p50 within both deadlines is still accepted, and a test with p50 above only verifier_timeout_s is refused; the oversize tests fail if the gate is removed and use a generic oversize forecast, not one job's data. Run cargo test -p athena-core.
4. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings (only readings forecasting a p50 wall time beyond a configured timeout).  Existing tests and the default athena.toml readings still pass: cargo test --workspace.
5. [behavior] cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass.
6. [content] CHANGELOG.md records the new reading size gate.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Changing athena.toml limits or adding new config keys
- Retry-scope tracking across jobs or re-ask handling
- Splitting documents into sections automatically
- Changing producer or verifier timeouts

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests for the gate; cargo test, clippy and fmt must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix placed in reading validation where the missing comparison originates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check a refusal and an acceptance happening, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The size rule is enforced by code in build_reading, not prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and CHANGELOG entry written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
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

18 more qualifying classes are left out.

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

The change description and a doc comment on the gate in crates/athena-core/src/reading.rs name the root cause: reading validation never compared reading scope with the producer/verifier deadline; the fix sits in build_reading.
A new unit test in athena-core builds a reading whose forecast wall_time_s.p50 exceeds budget.producer_timeout_s and asserts build_reading returns an error whose message tells the reader to pick one deliverable and name the rest as follow-up requests. Run cargo test -p athena-core.
A companion test shows a reading with p50 within both deadlines is still accepted, and a test with p50 above only verifier_timeout_s is refused; the oversize tests fail if the gate is removed and use a generic oversize forecast, not one job's data. Run cargo test -p athena-core.
The change refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings (only readings forecasting a p50 wall time beyond a configured timeout).  Existing tests and the default athena.toml readings still pass: cargo test --workspace.
cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass.
CHANGELOG.md records the new reading size gate.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
