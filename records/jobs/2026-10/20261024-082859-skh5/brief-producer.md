# Job 20261024-082859-skh5: Add read-only host-rules check for athena/records and main (D111, part

## Request (verbatim)

> try again on 20261024-033935-c4w4

## Reading (accepted by the user)

Retry of job 20261024-033935-c4w4: add an athena-core module that GETs the rulesets view rules/branches/{branch} for athena/records and main and reports force-push forbidden, deletion forbidden and up-to-date required as in force, not in force, or unknown. Audience: Sri, who must set these host rules before the records cutover. The retry also fixes the earlier findings: the docs state the check sees rulesets only, the CHANGELOG cites §3 and §10, and the gate evidence cites the commands that ran.

Assumptions:
- The typed request means redo the original check job; the introspection's proposed pre-handover evidence gate is a separate pipeline change and is not built here.
- The user's correction stands: use GET repos/{owner}/{repo}/rules/branches/{branch}, not the classic /protection endpoint (403 here).
- Rules: non_fast_forward => force-push forbidden; deletion => deletion forbidden; required_status_checks with parameters.strict_required_status_checks_policy true => up-to-date required.
- Runner errors or unparseable JSON give an 'unknown' state, never 'in force' or 'not in force'.
- The check sees rulesets only; classic branch protection is not visible through this view.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: fixture replies in the rules/branches shape (JSON list of rule objects) give athena/records with non_fast_forward => force-push forbidden in force; deletion rule => deletion forbidden in force; main with required_status_checks whose strict_required_status_checks_policy is true => up-to-date required in force.
2. [behavior] Run `cargo test -p athena-core`: tests assert `[]` reports all rules not in force; strict_required_status_checks_policy false or absent reports up-to-date not in force; a partial list (only non_fast_forward) reports force-push in force and deletion not in force.
3. [behavior] Run `cargo test -p athena-core`: a recording fake Runner test asserts only GET calls to `repos/{owner}/{repo}/rules/branches/athena/records` and `.../rules/branches/main`, with no method override and no field/body arguments, and no /protection call.
4. [behavior] Run `cargo test -p athena-core`: a runner error (403 or 5xx) or unparseable JSON makes the affected rules report unknown, not in force and not not-in-force.
5. [content] The plain-text report names each of the three rules per branch and its state, e.g. 'athena/records: force-push forbidden: in force', so Sri can see which host rule to set; the report or module doc states that only rulesets are seen, not classic branch protection.
6. [content] Module doc comments cite D111 and docs/design/records-history.md §3 and call this part 3 of job 1; the module is registered in crates/athena-core/src/lib.rs; the CHANGELOG [Unreleased] entry cites both §3 and §10.
7. [behavior] Run `cargo clippy --all-targets`, `cargo fmt --check` and `cargo test` from the repo root: all exit 0 for the workspace. The producer's evidence for this criterion cites each command as run with its exit status and output tail.

## Expected deliverables

- crates/athena-core/src/host_rules.rs (create)
- crates/athena-core/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-core/src/gh.rs (modify)

## Out of scope

- Changing any host setting or ruleset
- The pre-handover evidence gate in the job pipeline
- Wiring the check into a CLI command or the cutover flow
- Reading classic branch protection

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Module and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-based unit tests; cargo test and clippy clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Gate commands run and cited with output.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert reported states from fixture replies and the GET-only calls, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module plus registration and docs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite D111 and the design section.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The doc now says rulesets only and the evidence cites commands, fixing the findings.
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
