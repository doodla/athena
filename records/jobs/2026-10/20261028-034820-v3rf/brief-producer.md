# Job 20261028-034820-v3rf: Replay candidate reading rules over 50 past requests via subcommand

## Request (verbatim)

> Priority: urgent
> A new rule that can refuse work (a reading rule, an admission rule, a dispatch hold) must be tried on real history before it lands. On 2026-10-28 a reading rule that matched common request words to governance files landed after passing verification, then refused almost every request that mentioned a common word, stopped three drains and refused its own fix three times; no one had run it against a single past request. Fix: when a candidate adds or changes a refusal rule, Athena replays the rule over the last 50 recorded requests and their accepted readings, and records how many it would now refuse and which. The verifier is shown that replay, and a candidate whose rule would refuse more than 10% of past accepted readings is held with the list, unless its criteria say a higher rate is intended.
> Acceptance:
> - A test with fixture requests shows the replay counts and lists refusals for a candidate rule, and holds a rule that refuses more than 10% of accepted readings.
> - The verifier brief carries the replay summary for a candidate that changes a refusal rule, and stays within its size limit.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md

## Reading (accepted by the user)

Add an `athena` subcommand that applies the reading acceptance rules to a stored request and reading and prints accept or refuse with the reason. After checks pass, run it from the base and candidate builds over the last 50 recorded requests with accepted readings. The replay result is the list the candidate refuses that the base accepted. It is recorded, shown to the verifier, and a candidate is held if more than 10% of those 50 are newly refused, unless its criteria intend a higher rate. The goal is to stop a refusal rule from landing untried, and the audience is the maintainers and the verifier.

Assumptions:
- Per the user's reply there is no flag detecting refusal-rule changes; every candidate is replayed.
- The new subcommand lives in crates/athena-cli/src/main.rs and calls the acceptance rules in reading.rs.
- Records are read through existing record-reading code; tests use fixture records.
- The 10% threshold is a code constant; it applies to newly refused requests out of the replayed requests.
- With an empty newly-refused list no model call is made, and an empty history is reported as no replay, not a pass.
- 'Criteria intend a higher rate' is detected by a code check on the candidate's acceptance criteria text.

## Acceptance criteria

1. [behavior] A new athena subcommand takes a stored request and reading and prints accept or refuse with the reason, using the same acceptance rules as reading.rs; a test runs it on fixtures (cargo test).
2. [behavior] A test builds a candidate-like rule change over fixture records and shows the newly refused requests (refused by candidate, accepted by base) are listed (cargo test -p athena-core).
3. [behavior] A test shows the replay uses only the latest 50 requests that have accepted readings, and that an empty history is reported as no replay, not a pass (cargo test).
4. [behavior] A test shows a candidate newly refusing more than 10% is held with the list, a candidate at or under 10% is not held, and a candidate whose criteria intend a higher rate is not held, with the replay still recorded (cargo test).
5. [behavior] A test shows that with no rule change the replay list is empty and makes no model call (cargo test).
6. [behavior] crates/athena-core/src/run/mod.rs runs the replay after checks pass for every candidate, using the base and candidate builds, and records the result; a test covers the hold path in the pipeline (cargo test).
7. [behavior] A test in crates/athena-context/src/briefs.rs shows the verifier brief carries the replay summary (newly refused count, total, list) and stays within its size limit with a long list (cargo test -p athena-context).
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.
9. [content] CHANGELOG.md has an entry describing the subcommand, the replay, the 10% hold and the verifier brief summary.
10. [content] No governance file (athena.toml, defaults.toml, workflows) changes; the 10% threshold is a code constant.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/lib.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Changing any existing reading rule
- Detecting refusal-rule changes by a flag or by matching diffs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-based tests; cargo test and clippy -D warnings pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the replay and hold.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a rule change is listed as newly refused on fixture history.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The hold is enforced in the run pipeline, not by prose.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty history is reported as no replay, not as a pass.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the changelog are written for outside readers.
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

16 more qualifying classes are left out.

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

A new athena subcommand takes a stored request and reading and prints accept or refuse with the reason, using the same acceptance rules as reading.rs; a test runs it on fixtures (cargo test).
A test builds a candidate-like rule change over fixture records and shows the newly refused requests (refused by candidate, accepted by base) are listed (cargo test -p athena-core).
A test shows the replay uses only the latest 50 requests that have accepted readings, and that an empty history is reported as no replay, not a pass (cargo test).
A test shows a candidate newly refusing more than 10% is held with the list, a candidate at or under 10% is not held, and a candidate whose criteria intend a higher rate is not held, with the replay still recorded (cargo test).
A test shows that with no rule change the replay list is empty and makes no model call (cargo test).
crates/athena-core/src/run/mod.rs runs the replay after checks pass for every candidate, using the base and candidate builds, and records the result; a test covers the hold path in the pipeline (cargo test).
A test in crates/athena-context/src/briefs.rs shows the verifier brief carries the replay summary (newly refused count, total, list) and stays within its size limit with a long list (cargo test -p athena-context).
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.
CHANGELOG.md has an entry describing the subcommand, the replay, the 10% hold and the verifier brief summary.
No governance file (athena.toml, defaults.toml, workflows) changes; the 10% threshold is a code constant.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
