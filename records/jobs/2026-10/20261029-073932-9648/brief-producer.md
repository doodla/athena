# Job 20261029-073932-9648: Require a mechanism, owner and measure for each active spec requirement

## Request (verbatim)

> Priority: urgent
> Make docs/spec.toml, Athena's own requirements, say how each requirement is met, so an active requirement can never sit without a mechanism (Sri, 2026-10-29: the brief is temporary and informs requirements). Today each requirement has a statement, kind, status and source quotes, but nothing records what meets it, so the requirement that Athena gets measurably faster and owns its acceleration went two weeks with no mechanism. Add to every active requirement: mechanism (the code path, role or check that meets it, or none), owner (the role responsible) and measure (how it is shown, and where in athena status). The Hestia sweep flags any active requirement with no mechanism or a failing measure and queues one request for it through the shared steward request builder (urgent for requirements about speed, per OP-23); athena status lists the requirements without a mechanism. Fill the new fields for the existing requirements from the current code; where nothing meets one, say none.
> Acceptance:
> - Every active requirement in docs/spec.toml has mechanism, owner and measure, and the spec test fails when one is missing.
> - A test with a fixture spec shows the sweep queues one request for a requirement whose mechanism is none, and none on a second run while it is open.
> - athena status lists active requirements with no mechanism.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: docs/spec.toml, xtask/tests/spec.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Add mechanism, owner and measure to every active requirement in docs/spec.toml, enforce them in the spec test, make the Hestia sweep queue one deduplicated steward request per requirement lacking a mechanism (urgent for speed), and list such requirements in athena status.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs (default_checks, run_sweep, repeated_causes, root_cause_request building via crate::steward::StewardRequest), crates/athena-core/src/steward.rs (StewardRequest::new, queued_work) and crates/athena-core/src/status.rs, plus xtask/tests/spec.rs (serde Spec/Requirement with deny_unknown_fields, so new fields must be added to the struct). The sweep checks run in run_sweep over default_checks; the new check parses docs/spec.toml, dedups against open requests by citing the requirement id in request text (as addressing_request does), and needs no new record type under crates/, so check_record_file and AnyRecord are untouched.
- Goal: make the spec enforce that each active requirement has a meeting mechanism; audience is Sri and the Hestia sweep and status readers.
- The spec parser in athena-core is a small serde struct mirroring the xtask one, since xtask tests cannot be imported; athena-core likely already has the toml crate.
- A failing measure is determined by a simple recorded measure check where feasible; otherwise only the missing or none mechanism triggers a request, and the measure check is limited to the field being present. Superseded or non-active requirements are exempt.
- Mechanism text is filled from current code by the producer, which is judgment-heavy across 135 requirements.
- No governance files (athena.toml, defaults.toml, workflows) need to change.

## Acceptance criteria

1. [content] Grep of docs/spec.toml shows every active [[requirement]] (135 entries today) carries mechanism, owner and measure; where nothing meets one, mechanism is the literal "none". The measure names where it appears in athena status.
2. [behavior] Run `cargo test -p xtask --test spec`: it passes on docs/spec.toml, and new in-memory negative cases fail an active requirement missing mechanism, owner or measure.
3. [behavior] Run `cargo test -p athena-core hestia`: a fixture-spec test shows the sweep queues exactly one request for an active requirement whose mechanism is none, built via StewardRequest, and none on a second run while the first is open. A requirement about speed gets an urgent request (OP-23). A failing measure is also flagged.
4. [behavior] Run `cargo test -p athena-core status`: a fixture test shows athena status lists active requirements with no mechanism, and lists none when all have one.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (workspace): all pass.
6. [content] CHANGELOG.md has an entry describing the mechanism, owner and measure fields, the sweep check and the status listing.
7. [content] The requirement that Athena gets measurably faster and owns its acceleration records a mechanism (or none) and an owner, so it is visible, not omitted.

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/spec.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-core/src/steward.rs (modify)

## Out of scope

- Changing BRIEF.md or the source quotes in the spec
- Building the acceleration mechanism itself
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require fmt, clippy -D warnings and tests passing.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The spec test and sweep enforce the rule rather than prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Fixture tests check queueing, idempotence and status listing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Fields and docs are written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change within the five named files.
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

Grep of docs/spec.toml shows every active [[requirement]] (135 entries today) carries mechanism, owner and measure; where nothing meets one, mechanism is the literal "none". The measure names where it appears in athena status.
Run `cargo test -p xtask --test spec`: it passes on docs/spec.toml, and new in-memory negative cases fail an active requirement missing mechanism, owner or measure.
Run `cargo test -p athena-core hestia`: a fixture-spec test shows the sweep queues exactly one request for an active requirement whose mechanism is none, built via StewardRequest, and none on a second run while the first is open. A requirement about speed gets an urgent request (OP-23). A failing measure is also flagged.
Run `cargo test -p athena-core status`: a fixture test shows athena status lists active requirements with no mechanism, and lists none when all have one.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (workspace): all pass.
CHANGELOG.md has an entry describing the mechanism, owner and measure fields, the sweep check and the status listing.
The requirement that Athena gets measurably faster and owns its acceleration records a mechanism (or none) and an owner, so it is visible, not omitted.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
