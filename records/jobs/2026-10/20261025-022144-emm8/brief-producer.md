# Job 20261025-022144-emm8: Stop Athena- trailers in commits after cutover C0 (retry)

## Request (verbatim)

> try again on 20261025-012657-hac3

## Reading (accepted by the user)

Retry of D111 job 7 part 1b: after cutover C0, Committer::build_commit_in, commit_with_staged and squash_text write no Athena- trailers, while trace and the auditor still read trailers up to C0. Keeps the earlier design; fixes the two verifier failures with a scratch repo whose pre-C0 history passes the old rules, so check_history and check_attribution report no violations, and tests the fallbacks through the real entry points.

Assumptions:
- Audience: the independent verifier and maintainers.
- Design is kept from the last attempt, per the user's reply.
- Dependency job 20261025-001232-vesh (the cutover) is already landed in the base.
- Same deliverable set as the accepted earlier reading; unneeded files will simply not change.

## Acceptance criteria

1. [behavior] Run cargo test: a scratch-repository test builds pre-C0 history that passes the old history rules (bootstrap commit, a job landed the old way with trailers, then the cutover job as C0). It then builds ONE job branch holding an intervention commit (commit_with_staged), a records-only commit (Committer) and the job landing commit, and lands it via land_pr with a squash merge. The squash commit and each of the three PR commits (via the landing record's second parent) have no 'Athena-' line.
2. [behavior] Run cargo test: in that repository the landing record classifies the three commits as declared_intervention, records_only and job_landing, and the test asserts check_history and check_attribution return an empty violations list on the post-C0 base. It makes no assertion that violations exist.
3. [behavior] Run cargo test: squash_text has a strict form and a relaxed post-C0 form, each tested. The relaxed form emits no Athena- line and is used for post-C0 landings.
4. [behavior] Run cargo test: a test in commits.rs covers the after-C0 strip (cutover landed: no trailers) and the kept cases (no cutover_job, or no landing record yet: trailers still written). Existing tests pass unchanged.
5. [behavior] Run cargo test: a test in status.rs drives the real athena trace/status entry point on a post-C0 job, and a separate test in auditor.rs drives the auditor reading a post-C0 job's files. Each runs in a scratch repository and covers both sides of C0: trailers read for commits up to and including C0, C0..head fallback after it, with pre-C0 results unchanged.
6. [behavior] Run cargo test: after C0 check_history runs only the secrets and attribution checks. The gate's own check keeps content and jobs checks and drops only the trailer-dependent invariant::check.
7. [content] CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b). They name the readers that still use trailers up to C0 and state that after C0 the history check runs only secrets and attribution. The change description lists each trailer reader and writer with its covering test.
8. [behavior] Run cargo clippy --all-targets, cargo fmt --check and cargo test: all clean. athena.toml and .github/workflows files are unchanged, and pre-C0 behavior is unchanged.

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

- Changing behavior for pre-C0 history
- Changes to athena.toml, defaults.toml or CI workflows
- Removing trailer readers for commits up to C0
- Other parts of the records-history plan

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New behavior has tests per reader and writer; clippy, fmt and test must be clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert that check_history and check_attribution pass and that trailers are absent, not that wiring exists.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Keeps the single focused design of the earlier attempt.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Each criterion maps to a test whose assertion states the criterion's expected outcome.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog are written for an outside reader.
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

Run cargo test: a scratch-repository test builds pre-C0 history that passes the old history rules (bootstrap commit, a job landed the old way with trailers, then the cutover job as C0). It then builds ONE job branch holding an intervention commit (commit_with_staged), a records-only commit (Committer) and the job landing commit, and lands it via land_pr with a squash merge. The squash commit and each of the three PR commits (via the landing record's second parent) have no 'Athena-' line.
Run cargo test: in that repository the landing record classifies the three commits as declared_intervention, records_only and job_landing, and the test asserts check_history and check_attribution return an empty violations list on the post-C0 base. It makes no assertion that violations exist.
Run cargo test: squash_text has a strict form and a relaxed post-C0 form, each tested. The relaxed form emits no Athena- line and is used for post-C0 landings.
Run cargo test: a test in commits.rs covers the after-C0 strip (cutover landed: no trailers) and the kept cases (no cutover_job, or no landing record yet: trailers still written). Existing tests pass unchanged.
Run cargo test: a test in status.rs drives the real athena trace/status entry point on a post-C0 job, and a separate test in auditor.rs drives the auditor reading a post-C0 job's files. Each runs in a scratch repository and covers both sides of C0: trailers read for commits up to and including C0, C0..head fallback after it, with pre-C0 results unchanged.
Run cargo test: after C0 check_history runs only the secrets and attribution checks. The gate's own check keeps content and jobs checks and drops only the trailer-dependent invariant::check.
CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b). They name the readers that still use trailers up to C0 and state that after C0 the history check runs only secrets and attribution. The change description lists each trailer reader and writer with its covering test.
Run cargo clippy --all-targets, cargo fmt --check and cargo test: all clean. athena.toml and .github/workflows files are unchanged, and pre-C0 behavior is unchanged.

## Report

End with the producer report as your structured output.
