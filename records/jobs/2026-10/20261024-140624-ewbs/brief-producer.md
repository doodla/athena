# Job 20261024-140624-ewbs: Classify PR commits by content in athena land, not by trailers

## Request (verbatim)

> athena land --pr refuses every pull request since 29fdd4c: pr_commits in crates/athena-core/src/land.rs requires exactly one Athena-Job trailer on every commit between the remote base and the PR head, but a pull request also carries commits that change no governed content (records only, or owner paths such as operator/ and BRIEF.md) and declared interventions, which carry none; job 20261024-114125-7z2t's landing failed on a lease commit (f499abc). Sri asked that attribution not depend on commit-message trailers. Classify each commit by its content instead: a commit that leaves governed content unchanged (the gate's fingerprint, as athena-gate's invariant does) is listed as records-only; a commit that adds an intervention record is listed as that declared intervention; a commit that adds a job's closing records (candidate, verification, landed outcome) is that job's landing; any other commit that changes governed content is refused, naming it. Extend the Landing record's per-commit entry to carry that kind (job landing, declared intervention, records-only), keeping validation strict. Test against a range shaped like this repository's real pull requests: reading, dispatch and introspection commits, a lease commit, an intervention, then the job's landing.

## Reading (accepted by the user)

Replace the Athena-Job trailer requirement in pr_commits with content classification: records-only (governed fingerprint unchanged), declared intervention, job landing, or refusal naming the commit. The Landing record's per-commit entry gains a strict kind. Audience: the maintainer running athena land --pr, and the verifier.

Assumptions:
- Kind is a new snake_case enum in athena-data history.rs (job_landing, declared_intervention, records_only), a required field on LandingCommit with deny_unknown_fields kept.
- job and verification are required only for job_landing; other kinds carry none (an intervention may carry its record id). Landing::validate enforces this per kind.
- Records-only is decided by comparing athena_git::attrs::fingerprint of a commit and its first parent, as athena-gate's invariant does.
- A job landing is recognised by the commit adding that job's candidate, verification and landed-outcome records; an intervention by an added intervention record. The job id is read from those records, not from trailers.
- Existing landing records on the records branch may lack a kind; the producer must check and decide on migration without loosening strictness for new records.
- CHANGELOG.md gets an entry.

## Acceptance criteria

1. [behavior] Run cargo test. A new test builds a range like a real PR (reading, dispatch and introspection commits, a lease commit, an intervention, then the job's landing) with no Athena-Job trailers, and pr_commits accepts it with kinds records_only x4, declared_intervention, job_landing in order.
2. [behavior] Run cargo test. A test where a commit changes governed content and is neither an intervention nor a job's closing records makes pr_commits fail with an error naming that commit's sha, and nothing is recorded.
3. [behavior] Run cargo test. Landing validation and parsing reject an unknown kind, a missing kind, an unknown field, and a job_landing entry lacking job or verification.
4. [content] pr_commits in crates/athena-core/src/land.rs no longer reads or requires Athena-Job trailers, and the old trailer-count tests are replaced by classification tests.
5. [content] LandingCommit in crates/athena-data/src/history.rs carries the kind, and the gh.rs code that builds LandingCommit passes it through; a landing-record round-trip test includes each kind.
6. [behavior] Run cargo clippy --all-targets and cargo fmt --check; both are clean, and the existing cargo test suite passes.
7. [content] CHANGELOG.md describes the change, and docs/design/records-history.md section 5.3 is updated if it states the trailer rule.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-data/src/history.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-git/src/attrs.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)

## Out of scope

- Removing Athena-Job trailers from other code paths such as commit writers
- Changing the reconcile algorithm beyond what the new field needs
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New classification and validation tests; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run a PR-shaped range and check kinds and the refusal.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix attribution by content rather than special-casing the lease commit.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to classification and the landing entry.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All Rust.
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

Run cargo test. A new test builds a range like a real PR (reading, dispatch and introspection commits, a lease commit, an intervention, then the job's landing) with no Athena-Job trailers, and pr_commits accepts it with kinds records_only x4, declared_intervention, job_landing in order.
Run cargo test. A test where a commit changes governed content and is neither an intervention nor a job's closing records makes pr_commits fail with an error naming that commit's sha, and nothing is recorded.
Run cargo test. Landing validation and parsing reject an unknown kind, a missing kind, an unknown field, and a job_landing entry lacking job or verification.
pr_commits in crates/athena-core/src/land.rs no longer reads or requires Athena-Job trailers, and the old trailer-count tests are replaced by classification tests.
LandingCommit in crates/athena-data/src/history.rs carries the kind, and the gh.rs code that builds LandingCommit passes it through; a landing-record round-trip test includes each kind.
Run cargo clippy --all-targets and cargo fmt --check; both are clean, and the existing cargo test suite passes.
CHANGELOG.md describes the change, and docs/design/records-history.md section 5.3 is updated if it states the trailer rule.

## Report

End with the producer report as your structured output.
