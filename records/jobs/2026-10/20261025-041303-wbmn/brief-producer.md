# Job 20261025-041303-wbmn: Judge PR-only commits after C0 by the pre-merge rule in check --history

## Request (verbatim)

> Priority: high
> The cutover (C0 b4f6a8a, job 20261025-001232-vesh) broke the required 'the full history' check on every pull request: on a pull request's head, athena check --history hands every commit after C0 to the attribution rule, which requires a landing record naming each commit on main, so the pull request's own commits (not yet merged, so with no landing record) are all unattributed (PR #175: 50 violations, every one 'no landing record names it and no landing-intent matches its tree'). On a head that is not main, judge the commits after C0 that are not on main (the range from the merge base with main to the head) by the pre-merge rule instead (land.rs pr_commits: records-only, declared intervention, or a job landing whose verification passed), and keep the attribution rule for commits on main.
> Acceptance:
> - On the cutover fixture (test_support from 20261025-031609-k0tt), a pull-request branch after C0 holding records-only commits, an intervention and a job landing passes athena check --history, and a branch with an unverified governed change fails naming it.
> - athena check --history on main after C0 is unchanged.
> - The CI history job passes on a pull request built like this repository's (run on the PR's own head).
> Files: crates/athena-gate/src/history.rs, crates/athena-gate/src/attribution.rs, crates/athena-core/src/land.rs, .github/workflows/ci.yml, CHANGELOG.md

## Reading (accepted by the user)

On a head that is not main, `athena check --history` after C0 will judge commits in merge-base(main)..head by the pre-merge rule (records-only, declared intervention, or a verified job landing). It keeps the attribution rule for commits on main. The audience is the maintainer and CI, which currently fails the required 'the full history' check on every PR.

Assumptions:
- athena-gate does not depend on athena-core (gate Cargo.toml lists data, git, score), and land.rs classify_commit already uses gate's invariant::closing_jobs. So the classifier moves into the gate and land.rs pr_commits/classify_commit delegate to it, keeping their behavior and tests.
- 'main' means the remote main ref (origin/main or local main) available in the checkout; if the merge base cannot be found, the check falls back to today's behavior and adds a note.
- A head 'is main' when it is an ancestor of, or equal to, main's tip. Then the whole post-C0 range stays under attribution, so history on main is unchanged.
- The CI change is limited to what the PR-head run needs: fetch main and athena/records so the merge base and records resolve. The existing PR-event skip of the records fetch is revisited. The checkout token persistence is narrowed to the fetch step if feasible.
- Tests for the acceptance cases use the existing cutover fixture from crates/athena-core/src/test_support.rs (job 20261025-031609-k0tt), driven from crates/athena-cli/tests.

## Acceptance criteria

1. [behavior] A test on the cutover fixture creates a PR branch after C0 with a records-only commit, a declared-intervention commit and a verified job-landing commit, none named by a landing record. Run `cargo test -p athena-cli` (the new history test): `athena check --history` on that head reports no violations.
2. [behavior] A test adds a governed-content change with no verification or intervention to a PR branch after C0. Run `cargo test -p athena-cli` (the new history test): `athena check --history` fails and the violation names that commit's sha.
3. [behavior] A test shows `athena check --history` on the fixture's main after C0 gives the same report as before: commits on main still go through the attribution rule, and an unattributed commit on main still fails. Run `cargo test -p athena-gate -p athena-cli`.
4. [behavior] All existing history and attribution tests and the land.rs pr_commits tests pass unchanged. Run `cargo test --workspace`; `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean.
5. [content] The CI history job in .github/workflows/ci.yml has the refs it needs on a pull_request event (main, and athena/records where used) so the merge base resolves. The step comments say why. The job keeps the name 'the full history'.
6. [content] CHANGELOG.md has an entry that says a PR head's own commits after C0 are judged by the pre-merge rule and main's by attribution, and cites the PR #175 failure.
7. [content] The pre-merge classifier has one source of truth in the gate; land.rs pr_commits delegates to it, with no duplicated rule logic.

## Expected deliverables

- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-core/src/land.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-gate/src/legacy.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-cli/tests/history_pr_head.rs (create)
- crates/athena-cli/tests/cutover_fixture.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Ending commit-message trailers.
- Changing the attribution rule's treatment of commits on main.
- Changing branch protection or the other CI jobs (attribution, pr-head).
- Changing athena.toml or defaults.toml.
- Re-running or repairing the cutover import itself.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the PR-branch pass, the PR-branch failure and main unchanged; cargo test and clippy stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run athena check --history on fixture branches and check the pass/fail outcome and the named commit.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The range split is fixed in the history check itself, not by skipping the check in CI.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test on the fixture-based tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change: range split, classifier move, CI fetch tweak and changelog.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The behavior is enforced by the required history check and its tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust, plus the CI YAML.
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

A test on the cutover fixture creates a PR branch after C0 with a records-only commit, a declared-intervention commit and a verified job-landing commit, none named by a landing record. Run `cargo test -p athena-cli` (the new history test): `athena check --history` on that head reports no violations.
A test adds a governed-content change with no verification or intervention to a PR branch after C0. Run `cargo test -p athena-cli` (the new history test): `athena check --history` fails and the violation names that commit's sha.
A test shows `athena check --history` on the fixture's main after C0 gives the same report as before: commits on main still go through the attribution rule, and an unattributed commit on main still fails. Run `cargo test -p athena-gate -p athena-cli`.
All existing history and attribution tests and the land.rs pr_commits tests pass unchanged. Run `cargo test --workspace`; `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean.
The CI history job in .github/workflows/ci.yml has the refs it needs on a pull_request event (main, and athena/records where used) so the merge base resolves. The step comments say why. The job keeps the name 'the full history'.
CHANGELOG.md has an entry that says a PR head's own commits after C0 are judged by the pre-merge rule and main's by attribution, and cites the PR #175 failure.
The pre-merge classifier has one source of truth in the gate; land.rs pr_commits delegates to it, with no duplicated rule logic.

## Report

End with the producer report as your structured output.
