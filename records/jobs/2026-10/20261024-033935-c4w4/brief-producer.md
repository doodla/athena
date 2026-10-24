# Job 20261024-033935-c4w4: Add read-only host check for records-history D111 rules

## Request (verbatim)

> Add a read-only check, part 3 of job 1 of the records-history plan (docs/design/records-history.md section 3, decision D111): it asks the host whether force-pushes and deletion are forbidden on the branch athena/records and whether main requires branches to be up to date before merging, and reports each as in force or not, with tests against fixture replies. It changes no host setting.

## Reading (accepted by the user)

Add a new athena-core module (part 3 of job 1, D111, design §3) that uses the gh Runner to GET the rulesets view `repos/{owner}/{repo}/rules/branches/{branch}` for athena/records and main, and reports three rules as in force or not. Audience: Sri, who must set these host rules before the records cutover. Tests use fixture replies; nothing on the host is changed.

Assumptions:
- Implemented as a library function in athena-core using the existing gh::Runner trait; no new CLI command is wired unless trivial (not required).
- Repo owner/name is passed in by the caller, as with existing gh helpers.
- JSON parsing uses serde_json already in the workspace; no new crates.
- Rule types per the user: non_fast_forward, deletion, required_status_checks with parameters.strict_required_status_checks_policy true.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: tests feed fixture replies in the rules/branches shape (JSON list of rule objects) and assert: athena/records with a non_fast_forward rule => force-push forbidden in force; a deletion rule => deletion forbidden in force; main with a required_status_checks rule whose parameters.strict_required_status_checks_policy is true => up-to-date required in force.
2. [behavior] Tests assert an empty list `[]` reports all rules as not in force, and that a required_status_checks rule with strict_required_status_checks_policy false (or absent) reports up-to-date as not in force; a partial list (only non_fast_forward) reports force-push in force and deletion not in force.
3. [behavior] A recording fake Runner test asserts the check issues only GET calls to `repos/{owner}/{repo}/rules/branches/athena/records` and `.../rules/branches/main` (not the classic /protection endpoint), with no method override and no field/body arguments, so no host setting changes.
4. [behavior] A test where the runner returns an error (e.g. 403 or 5xx) or unparseable JSON asserts the affected rules are reported as unknown/error, not as 'not in force' and not as in force.
5. [content] The report names each of the three rules and its state in plain text, e.g. 'athena/records: force-push forbidden: in force', so Sri can see which host rule to set.
6. [content] Doc comments cite D111 and docs/design/records-history.md §3 and describe this as part 3 of job 1; the module is registered in athena-core lib.rs.
7. [behavior] Run `cargo clippy --all-targets`, `cargo fmt --check` and `cargo test`: all clean and passing for the workspace.

## Expected deliverables

- crates/athena-core/src/host_rules.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## Out of scope

- Changing any host setting, ruleset, or branch protection
- Wiring the check into a gate, CLI command, or cutover flow
- Other parts/jobs of the records-history plan

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-based unit tests; cargo test and clippy clean.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses existing gh Runner and serde_json.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module plus its registration.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert reported states from fixtures and GET-only calls.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite D111 and design §3 for outside readers.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria runnable via cargo test/clippy/fmt.
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
