# Job 20261025-012657-hac3: Stop writing Athena- trailers in commits after cutover C0

## Request (verbatim)

> try again on 20261025-001234-2hax

## Reading (accepted by the user)

Retry: after cutover C0, commits from Committer::build_commit_in, commit_with_staged and squash_text carry no Athena- trailers, while trailer readers keep working for commits up to C0. Demonstrated by one squash-landed job PR holding an intervention, a records-only and a job-landing commit, as the user specified.

Assumptions:
- Goal: stop writing Athena- trailers after cutover C0 while history/attribution checks keep passing; audience is the maintainer and the verifier running athena check.
- Dependence on the cutover job 20261025-001232-vesh is assumed already landed in the base.
- Reading widens the deliverables to include crates/athena-gate/src/lib.rs and crates/athena-core/src/manage.rs, as the earlier critique found necessary.
- The user's squash-merge test shape and strip-in-squash_text instruction override the earlier merge-commit plan.
- Trailer stripping is keyed on the parent's cutover state, in Committer::build_commit_in and commit_with_staged.

## Acceptance criteria

1. [behavior] Run cargo test: a scratch-repository test lands the cutover job, then builds ONE job branch whose range holds an intervention commit (commit_with_staged), a records-only commit (Committer) and the job's landing commit, and lands it through land_pr with a squash merge. It asserts the squash commit on the base has no 'Athena-' line, and each of the three PR commits (reachable via the landing record's second parent) has no 'Athena-' line. The test fails without the change.
2. [behavior] Run cargo test: in the same scratch repository the landing record classifies the three commits as declared_intervention, records_only and job_landing, and check_history and check_attribution pass on the post-C0 base (library-level; if a CLI-level athena check run is included in lifecycle.rs it has a clear assertion, otherwise it is omitted).
3. [behavior] Run cargo test: squash_text has a strict form and a relaxed post-C0 form, each tested; the relaxed form emits no Athena- line and is used for post-C0 landings.
4. [behavior] Run cargo test: commits from Committer::build_commit_in and commit_with_staged after C0 carry no Athena- trailer; with no cutover_job set, or before its landing record exists, they still carry trailers and all existing tests pass unchanged.
5. [behavior] Run cargo test: a test shows that after C0 check_history skips content, invariant and jobs checks (runs only secrets and attribution), and that the gate's own check keeps content and jobs checks but drops only trailer-dependent invariant::check.
6. [behavior] Run cargo test: status and audit still read trailers on commits up to and including C0 with a C0..head fallback, giving the same results as before.
7. [content] CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b), name which readers still use trailers up to C0, and describe the history check accurately (after C0 it runs only secrets and attribution).
8. [behavior] Run cargo clippy, cargo fmt --check and cargo test: all clean; no governance files (athena.toml, ci.yml) change.

## Expected deliverables

- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Landing a standalone intervention or records-only PR through land_pr
- Changing athena.toml, defaults.toml or CI workflows
- Removing the trailer readers or their pre-C0 behavior

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for each behavior; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert trailer-free commits and attribution results, not wiring.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier runs.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by tests and checks.
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

Run cargo test: a scratch-repository test lands the cutover job, then builds ONE job branch whose range holds an intervention commit (commit_with_staged), a records-only commit (Committer) and the job's landing commit, and lands it through land_pr with a squash merge. It asserts the squash commit on the base has no 'Athena-' line, and each of the three PR commits (reachable via the landing record's second parent) has no 'Athena-' line. The test fails without the change.
Run cargo test: in the same scratch repository the landing record classifies the three commits as declared_intervention, records_only and job_landing, and check_history and check_attribution pass on the post-C0 base (library-level; if a CLI-level athena check run is included in lifecycle.rs it has a clear assertion, otherwise it is omitted).
Run cargo test: squash_text has a strict form and a relaxed post-C0 form, each tested; the relaxed form emits no Athena- line and is used for post-C0 landings.
Run cargo test: commits from Committer::build_commit_in and commit_with_staged after C0 carry no Athena- trailer; with no cutover_job set, or before its landing record exists, they still carry trailers and all existing tests pass unchanged.
Run cargo test: a test shows that after C0 check_history skips content, invariant and jobs checks (runs only secrets and attribution), and that the gate's own check keeps content and jobs checks but drops only trailer-dependent invariant::check.
Run cargo test: status and audit still read trailers on commits up to and including C0 with a C0..head fallback, giving the same results as before.
CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b), name which readers still use trailers up to C0, and describe the history check accurately (after C0 it runs only secrets and attribution).
Run cargo clippy, cargo fmt --check and cargo test: all clean; no governance files (athena.toml, ci.yml) change.

## Report

End with the producer report as your structured output.
