# Job 20261024-153908-1rve: Classify PR commits by content in pr_commits, not Athena-Job trailers

## Request (verbatim)

> Part 2 of the fix for job 20261024-140624-ewbs: pr_commits in crates/athena-core/src/land.rs must stop reading Athena-Job trailers and classify each commit between the remote base and the PR head by its content, setting LandingCommit's kind (added in b6f22c2): a commit that leaves governed content unchanged (the gate's fingerprint, as crates/athena-gate's invariant computes it) is records_only; a commit that adds an intervention record under records/interventions/ is declared_intervention; a commit that adds a job's closing records (its candidate, verification and landed outcome) is that job's job_landing, with that job and verification; any other commit that changes governed content is refused with an error naming its sha. Replace the trailer-count tests with a test on a range shaped like this repository's real pull requests (reading, dispatch and introspection commits, a lease commit under operator/, an intervention, then the job's landing, none with an Athena-Job trailer) and a refusal test. Run only the athena-core land and gh tests you need while working.

## Reading (accepted by the user)

Rewrite `pr_commits` in land.rs to classify each commit in `remote_base..head` by its content. The goal is that `athena land --pr` stops refusing real pull requests. The audience is Sri and the verifier, who run the athena-core land and gh tests. The caller in gh.rs passes each commit's kind through instead of hard-coding JobLanding.

Assumptions:
- LandingCommit.kind and validation already exist in athena-data (b6f22c2), so athena-data does not change.
- A job's closing records are recognised by paths added under records/jobs/<month>/<job>/ for the candidate, verification and outcome. I did not confirm the exact file names, so the producer must read how they are written.
- The verification id comes from the existing `verification_id(job, commit)` in gh.rs.
- A commit is compared against its first parent. Merge commits are not expected in a PR range.
- Ordering of the checks: unchanged fingerprint first, then intervention record added, then job closing records, otherwise refuse.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core land::`. A test builds a range shaped like this repo's real PRs: reading, dispatch and introspection commits (records only), a lease commit under operator/, an intervention commit adding records/interventions/<month>/<id>.json, then a job's landing commit adding its candidate, verification and outcome records. No commit carries an Athena-Job trailer. The test passes and asserts kinds in order: records_only for the first four, declared_intervention, then job_landing with that job and the verification id.
2. [behavior] Run `cargo test -p athena-core land::`. A refusal test adds a commit that changes governed content without being an intervention or a job's closing records. `pr_commits` returns an error that contains that commit's sha.
3. [behavior] Run `cargo test -p athena-core gh::`. The gh tests still pass, and the commits built for a Landing record carry the kind from `pr_commits` and pass `LandingCommit::validate`.
4. [content] `pr_commits` and `PrCommit` no longer read trailers or mention Athena-Job. `PrCommit` carries the kind, plus an optional job and verification. The doc comments describe the content rules.
5. [content] A commit is records_only when its fingerprint equals its parent's, using the same `fingerprint(git, commit, base)` function the gate invariant uses. An empty range is still an error.
6. [content] The old trailer-count tests for `pr_commits` are removed or replaced, not left alongside the new behavior.
7. [behavior] Run `cargo clippy -p athena-core --all-targets` and `cargo fmt --check`. Both are clean.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the Landing record schema or LandingKind.
- Running the full workspace test suite.
- Changing the gate invariant's behavior.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New classification and refusal tests; clippy and fmt checks.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the kinds produced for a realistic range and the sha in the refusal error.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Trailer reliance is removed entirely instead of adding exceptions.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The change is confined to land.rs and its caller in gh.rs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior criteria name the cargo test commands the verifier can run.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Only Rust code and tests change.
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

Run `cargo test -p athena-core land::`. A test builds a range shaped like this repo's real PRs: reading, dispatch and introspection commits (records only), a lease commit under operator/, an intervention commit adding records/interventions/<month>/<id>.json, then a job's landing commit adding its candidate, verification and outcome records. No commit carries an Athena-Job trailer. The test passes and asserts kinds in order: records_only for the first four, declared_intervention, then job_landing with that job and the verification id.
Run `cargo test -p athena-core land::`. A refusal test adds a commit that changes governed content without being an intervention or a job's closing records. `pr_commits` returns an error that contains that commit's sha.
Run `cargo test -p athena-core gh::`. The gh tests still pass, and the commits built for a Landing record carry the kind from `pr_commits` and pass `LandingCommit::validate`.
`pr_commits` and `PrCommit` no longer read trailers or mention Athena-Job. `PrCommit` carries the kind, plus an optional job and verification. The doc comments describe the content rules.
A commit is records_only when its fingerprint equals its parent's, using the same `fingerprint(git, commit, base)` function the gate invariant uses. An empty range is still an error.
The old trailer-count tests for `pr_commits` are removed or replaced, not left alongside the new behavior.
Run `cargo clippy -p athena-core --all-targets` and `cargo fmt --check`. Both are clean.

## Report

End with the producer report as your structured output.
