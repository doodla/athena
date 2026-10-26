# Job 20261026-065231-j9k2: Build Hestia sweep skeleton, findings record, and accept D129

## Request (verbatim)

> Priority: high
> Build job 1 of the Hestia design (docs/design/hestia.md section 9): the sweep skeleton and findings record, with the trigger settled. Also mark Hestia's decision accepted in docs/decisions.md, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26).
> Acceptance:
> - A test where every check returns nothing yields a report listing all checks with counts; a test where a check errors yields a finding for it.
> - A command prints the last sweep.
> - The decision is marked accepted.
> Files: crates/athena-core/src/hestia.rs, crates/athena-core/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-data/src/records.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Implements job 1 of docs/design/hestia.md §9: a `hestia` module in athena-core with a Finding type and a sweep runner that lists every check with a count and turns a check error into a finding. It also adds a CLI command that prints the last sweep, settles the trigger, and marks D129 accepted by the brain under Sri's delegation. The audience is Sri and the later Hestia build jobs, which plug checks into this skeleton.

Assumptions:
- The trigger is settled as an explicit command (`athena hestia sweep` to run, `athena hestia last` to print), with a drain hook deferred to a later job; the decision text records this and why (no timer daemon exists, a command is testable and does not gate drains). If the repo shows a drain hook is cheap, the producer may add it, but the command is the minimum.
- The sweep report is stored through athena-data records.rs as a new record type, and `last` reads the newest one; the exact storage mirrors existing record patterns there.
- The skeleton ships with an empty or injected check list; real checks (rows 1-8) are later jobs. The check set is injectable so tests can supply fake checks.
- D129 is the Hestia decision (docs/decisions.md line ~3233, currently 'proposed'). Its status and heading become accepted, with 'decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26)'. D130 is untouched.
- docs/design/hestia.md section 10 open point on the trigger may be updated to point at the decision; this is optional and small.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/hestia.rs registers several checks that each return no findings. The resulting report lists every check by id with a count of 0 and has no findings. Run `cargo test -p athena-core hestia`.
2. [behavior] A unit test has one check return an error while the others succeed. The report holds a finding for that check (check id, subject, evidence containing the error, age, state), and the other checks are still listed with counts. Run `cargo test -p athena-core hestia`.
3. [behavior] A record-level test shows a swept report round-trips through athena-data records.rs and that reading the last sweep returns the most recent one. Run `cargo test -p athena-data`.
4. [behavior] `cargo run -- hestia last` (or the equivalent athena-cli invocation) prints the last sweep, listing each check with its count and each finding. With no sweep recorded it prints an explicit 'no sweep recorded' message, not an empty success. A CLI test covers both cases.
5. [content] Finding has the fields check id, subject, evidence, age and state, as specified in docs/design/hestia.md §9 job 1, and the module is exported from crates/athena-core/src/lib.rs.
6. [content] docs/decisions.md marks the Hestia decision (D129) accepted in both heading and Status line, names it as decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), and states the settled trigger with the alternatives considered and why. The 'not accepted; build jobs not queued' wording is removed or updated.
7. [content] CHANGELOG.md has an entry for the Hestia sweep skeleton, the command and the accepted decision, written for an outside reader.
8. [behavior] `cargo test`, `cargo clippy` (no warnings) and `cargo fmt --check` all pass across the workspace.
9. [content] No governance files (athena.toml, defaults.toml, workflows, Cargo lint config) are changed, and no check beyond the skeleton (rows 1-8) is implemented.

## Expected deliverables

- crates/athena-core/src/hestia.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-data/src/records.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/hestia.md (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-data/src/lib.rs (modify)

## Out of scope

- Implementing any actual check (rows 1 to 8, jobs 2 to 10)
- Model-backed checks, budget enforcement, fix-job emission
- A timer or daemon trigger
- Changes to D130 or the scope guard (D127)
- Governance files and records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust in the existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the module, the record and the command; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the report contents and the printed output, not that the types exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The report lists checks with counts even when empty, and the 'last' command says explicitly when nothing was recorded.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Skeleton only; no real checks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The command is exercised through cargo run and tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the report and the printed sweep, which are the outcomes.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Check errors become findings by code, not by convention.
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

A unit test in crates/athena-core/src/hestia.rs registers several checks that each return no findings. The resulting report lists every check by id with a count of 0 and has no findings. Run `cargo test -p athena-core hestia`.
A unit test has one check return an error while the others succeed. The report holds a finding for that check (check id, subject, evidence containing the error, age, state), and the other checks are still listed with counts. Run `cargo test -p athena-core hestia`.
A record-level test shows a swept report round-trips through athena-data records.rs and that reading the last sweep returns the most recent one. Run `cargo test -p athena-data`.
`cargo run -- hestia last` (or the equivalent athena-cli invocation) prints the last sweep, listing each check with its count and each finding. With no sweep recorded it prints an explicit 'no sweep recorded' message, not an empty success. A CLI test covers both cases.
Finding has the fields check id, subject, evidence, age and state, as specified in docs/design/hestia.md §9 job 1, and the module is exported from crates/athena-core/src/lib.rs.
docs/decisions.md marks the Hestia decision (D129) accepted in both heading and Status line, names it as decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), and states the settled trigger with the alternatives considered and why. The 'not accepted; build jobs not queued' wording is removed or updated.
CHANGELOG.md has an entry for the Hestia sweep skeleton, the command and the accepted decision, written for an outside reader.
`cargo test`, `cargo clippy` (no warnings) and `cargo fmt --check` all pass across the workspace.
No governance files (athena.toml, defaults.toml, workflows, Cargo lint config) are changed, and no check beyond the skeleton (rows 1-8) is implemented.

## Report

End with the producer report as your structured output.
