# Job 20261024-170227-4scc: Reland content-based pr_commits with a full change description

## Request (verbatim)

> try again on 20261024-161559-rxjg

## Reading (accepted by the user)

Retry of rejected job 20261024-161559-rxjg: rewrite pr_commits in athena-core land.rs to classify commits in remote_base..head by content (records_only, declared_intervention, job_landing via the gate's closing_jobs), with no trailers. The earlier code passed every check; this retry must also give a change description listing every caller and writer with its covering test and the commands run. Audience: Sri and the verifier.

Assumptions:
- The earlier code was correct; the retry reproduces it and fixes the missing change description.
- run.rs and drain.rs do not call pr_commits, so they likely stay unchanged; the description says so.
- The two minor/nit findings (duplicated CLI fixtures, outcome-not-landed test using a missing outcome) are cheap to fix and are addressed.
- No athena.toml, defaults.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core land::`. A realistic-range test (reading, dispatch and introspection commits, an owner-path commit under a directory with a nested .gitattributes '* athena-exclude', an intervention adding records/interventions/<month>/<id>.json, then a job's candidate, verification and outcome; no Athena-Job trailers) yields records_only x4, declared_intervention, and job_landing carrying that job and its verification id.
2. [behavior] Run `cargo test -p athena-core land::`. Blocked-then-landed: a commit adding candidate, verification and a blocked outcome is records_only; a later commit adding only outcome-landed.json plus governed work is job_landing for that job, reusing the gate's closing_jobs.
3. [behavior] Run `cargo test -p athena-core land::`. Refusal tests each make pr_commits return an error containing the commit's sha: governed change that is neither intervention nor closing records; landing with no verification; landing whose outcome exists but is not landed (a real blocked outcome); owner-path commit without the exclusion.
4. [behavior] Run `cargo test -p athena-core` and `cargo test -p athena-cli --test reconcile`. All pass, including the core reconcile tests and the CLI reconcile tests. Run `cargo test -p athena-core gh::`: landing commits carry kind, job and verification from pr_commits and pass LandingCommit::validate.
5. [content] pr_commits and PrCommit no longer read trailers or mention Athena-Job; doc comments state the content rules and that closing_jobs is the single source of truth. Old trailer-count tests are removed. An empty range is still an error.
6. [content] The change description (commit message body and CHANGELOG entry) lists every caller of pr_commits and every writer of landing records (gh.rs GitLandingRecords::commits, reconcile.rs via GitLandingRecords, crates/athena-cli tests/reconcile.rs, run.rs retry_landing, crates/athena-cli/src/drain.rs PrLander), each with its covering test or the reason it needed no change, and cites the commands actually run.
7. [content] docs/design/records-history.md and CHANGELOG.md describe content-based classification and no longer describe the trailer rule.
8. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`; both are clean.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- The pre-handover evidence check in Athena proposed by the introspection
- Changing the gate's closing_jobs semantics
- Changes to records/ or owner files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run named test filters and check classification outcomes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Classification reuses closing_jobs rather than a new rule.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Change description written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change.
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

Run `cargo test -p athena-core land::`. A realistic-range test (reading, dispatch and introspection commits, an owner-path commit under a directory with a nested .gitattributes '* athena-exclude', an intervention adding records/interventions/<month>/<id>.json, then a job's candidate, verification and outcome; no Athena-Job trailers) yields records_only x4, declared_intervention, and job_landing carrying that job and its verification id.
Run `cargo test -p athena-core land::`. Blocked-then-landed: a commit adding candidate, verification and a blocked outcome is records_only; a later commit adding only outcome-landed.json plus governed work is job_landing for that job, reusing the gate's closing_jobs.
Run `cargo test -p athena-core land::`. Refusal tests each make pr_commits return an error containing the commit's sha: governed change that is neither intervention nor closing records; landing with no verification; landing whose outcome exists but is not landed (a real blocked outcome); owner-path commit without the exclusion.
Run `cargo test -p athena-core` and `cargo test -p athena-cli --test reconcile`. All pass, including the core reconcile tests and the CLI reconcile tests. Run `cargo test -p athena-core gh::`: landing commits carry kind, job and verification from pr_commits and pass LandingCommit::validate.
pr_commits and PrCommit no longer read trailers or mention Athena-Job; doc comments state the content rules and that closing_jobs is the single source of truth. Old trailer-count tests are removed. An empty range is still an error.
The change description (commit message body and CHANGELOG entry) lists every caller of pr_commits and every writer of landing records (gh.rs GitLandingRecords::commits, reconcile.rs via GitLandingRecords, crates/athena-cli tests/reconcile.rs, run.rs retry_landing, crates/athena-cli/src/drain.rs PrLander), each with its covering test or the reason it needed no change, and cites the commands actually run.
docs/design/records-history.md and CHANGELOG.md describe content-based classification and no longer describe the trailer rule.
Run `cargo clippy --all-targets` and `cargo fmt --check`; both are clean.

## Report

End with the producer report as your structured output.
