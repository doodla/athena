# Job 20261024-161559-rxjg: Classify PR commits by content in pr_commits; fix all dependents

## Request (verbatim)

> try again on 20261024-154552-e65v

## Reading (accepted by the user)

Rewrite pr_commits in athena-core land.rs to classify each commit in remote_base..head by content (records_only, declared_intervention, job_landing via the gate's closing_jobs), and update every caller and fixture (gh.rs, reconcile.rs, athena-cli reconcile tests, run.rs retry_landing, drain.rs if affected). Audience: Sri and the verifier running athena-core and athena-cli reconcile tests.

Assumptions:
- Owner-path fixture may use any directory name with a nested .gitattributes '* athena-exclude' (owner-notes/ fine, per user reply); the word the boundary scan bans is not used.
- Gate closing_jobs logic in crates/athena-gate/src/invariant.rs is reused, possibly made public, as single source of truth.
- Reconcile fixtures are updated to follow the new content rules (real closing records) rather than loosening the code; the change description states this.
- Callers found by grep: gh.rs, reconcile.rs, land.rs, athena-cli/tests/reconcile.rs; run.rs and drain.rs are audited for GitLandingRecords/pr_commits use and changed only if they depend on it.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core land::`. A realistic-range test (reading, dispatch, introspection commits, an owner-path commit under a directory with nested .gitattributes '* athena-exclude' named in the doc comment, an intervention adding records/interventions/<month>/<id>.json, then a job's candidate, verification and outcome; no Athena-Job trailers) yields records_only x4, declared_intervention, job_landing with that job and verification id.
2. [behavior] Run `cargo test -p athena-core land::`. Blocked-then-landed: a commit adding candidate, verification and blocked outcome is records_only; a later commit adding only outcome-landed.json plus governed work is job_landing for that job, reusing the gate's closing_jobs logic.
3. [behavior] Run `cargo test -p athena-core land::`. Refusal tests each make pr_commits return an error containing the commit's sha: governed change that is neither intervention nor closing records; landing with no verification; landing whose outcome is not landed; owner-path commit without the exclusion.
4. [behavior] Run `cargo test -p athena-core` and `cargo test -p athena-cli --test reconcile`. All pass, including the 5 core reconcile tests and the 2 CLI reconcile tests that failed before; the change description says whether fixtures or code changed and why.
5. [behavior] Run `cargo test -p athena-core gh::`. Landing commits carry kind/job/verification from pr_commits and pass LandingCommit::validate.
6. [content] pr_commits and PrCommit no longer read trailers or mention Athena-Job; doc comments state the content rules and that closing_jobs is the single source of truth. Old trailer-count tests are removed. An empty range is still an error.
7. [content] Change description lists every caller of pr_commits and every writer of landing records (gh.rs, run.rs retry_landing, reconcile.rs via GitLandingRecords, drain.rs if relevant) with the test covering each, and cites commands actually run.
8. [content] docs/design/records-history.md and CHANGELOG.md describe content-based classification and no longer describe the trailer rule.
9. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`; both clean.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/drain.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Adding a repo-wide caller-finding check mechanism
- Changing the gate's fingerprint semantics

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Run athena-core and athena-cli reconcile tests, clippy, fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert classified kinds and sha-bearing errors.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Reuse gate closing_jobs; fix all dependent callers and fixtures.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change to classification and its dependents.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and change description written for outside readers.
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

Run `cargo test -p athena-core land::`. A realistic-range test (reading, dispatch, introspection commits, an owner-path commit under a directory with nested .gitattributes '* athena-exclude' named in the doc comment, an intervention adding records/interventions/<month>/<id>.json, then a job's candidate, verification and outcome; no Athena-Job trailers) yields records_only x4, declared_intervention, job_landing with that job and verification id.
Run `cargo test -p athena-core land::`. Blocked-then-landed: a commit adding candidate, verification and blocked outcome is records_only; a later commit adding only outcome-landed.json plus governed work is job_landing for that job, reusing the gate's closing_jobs logic.
Run `cargo test -p athena-core land::`. Refusal tests each make pr_commits return an error containing the commit's sha: governed change that is neither intervention nor closing records; landing with no verification; landing whose outcome is not landed; owner-path commit without the exclusion.
Run `cargo test -p athena-core` and `cargo test -p athena-cli --test reconcile`. All pass, including the 5 core reconcile tests and the 2 CLI reconcile tests that failed before; the change description says whether fixtures or code changed and why.
Run `cargo test -p athena-core gh::`. Landing commits carry kind/job/verification from pr_commits and pass LandingCommit::validate.
pr_commits and PrCommit no longer read trailers or mention Athena-Job; doc comments state the content rules and that closing_jobs is the single source of truth. Old trailer-count tests are removed. An empty range is still an error.
Change description lists every caller of pr_commits and every writer of landing records (gh.rs, run.rs retry_landing, reconcile.rs via GitLandingRecords, drain.rs if relevant) with the test covering each, and cites commands actually run.
docs/design/records-history.md and CHANGELOG.md describe content-based classification and no longer describe the trailer rule.
Run `cargo clippy --all-targets` and `cargo fmt --check`; both clean.

## Report

End with the producer report as your structured output.
