# Job 20261013-192158-x22e: Make 'athena land --pr' work on squash-only repositories

## Request (verbatim)

> Make 'athena land --pr' work on repositories that allow only squash merges, as this one does (GitHub reports allow_squash_merge true; allow_merge_commit and allow_rebase_merge false). Sri chose this over enabling merge commits, so Athena works on any user's repository. Root cause of today's failures: squash merges break ancestry, so after one job lands, the local branch no longer descends from main and the next job's PR conflicts (PR #40 needed a hand rebuild of the landed commit on main, same tree and message, pushed with a lease). Three parts. (1) Read the allowed merge methods from the host and request one that is allowed; a squash's commit message carries the job's trailers in its last paragraph with exactly one Athena-Job, so 'athena check --history' passes on main. (2) After a squash lands, Athena brings the local branch onto main by itself: later landed jobs and records are rebuilt on top of the squash commit without losing any unlanded work, so the next 'athena land --pr' opens a clean PR. (3) Revise the merge-commit landing decision recorded by job 9h2j in docs/decisions.md and the design doc, citing Sri's choice. Test against a real local git repository.

## Reading (accepted by the user)

Athena will read the repository's allowed merge methods from GitHub and request an allowed one (squash when it is the only one, with the job's trailers in the last paragraph and one Athena-Job). After a squash lands, Athena rebuilds the local branch onto main, replaying later landed jobs and records without losing unlanded work. The merge-commit decision from job 9h2j in docs/decisions.md and the design doc is revised, citing Sri's choice.

Assumptions:
- Method preference order when several are allowed is merge, then squash, then rebase, which keeps today's behavior on repositories that allow merge commits.
- Allowed methods come from GET repos/{owner}/{name} (allow_merge_commit, allow_squash_merge, allow_rebase_merge) through the PrHost trait in crates/athena-core/src/gh.rs.
- The squash commit message is supplied to the merge call as commit_title and commit_message, built from the job's trailers.
- Local rebuild after a squash lands is done in the landing flow (athena-core/athena-gate/athena-git), reusing the existing squash-chain machinery in docs/design/squash-chain-landing.md where it fits.
- The 'design doc' to revise is docs/design/native-pr-landing.md, which records the merge-commit landing decision; the decision entry is the one near line 1901 of docs/decisions.md.
- Sri's choice is cited by name as the owner's decision, with no personal data such as an email address.

## Acceptance criteria

1. [behavior] The host client reads allow_merge_commit, allow_squash_merge and allow_rebase_merge, and merge requests an allowed method. With only squash allowed it sends merge_method=squash, and with none allowed it fails with a clear error. Check with cargo test -p athena-core.
2. [behavior] A squash commit message ends with a last paragraph holding the job's trailers with exactly one Athena-Job line. A test builds a squash on a real local git repository and shows that the history check passes on main. Check with cargo test.
3. [behavior] A test on a real temporary git repository simulates a squash landing. It then shows that the local branch is rebuilt on the squash commit, that the later landed jobs and records are kept, that unlanded work is not lost, and that the next PR branch merges with main without conflict. Check with cargo test.
4. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.
5. [content] docs/decisions.md has a new or revised entry superseding the 9h2j merge-commit landing decision. It cites Sri's choice of squash-only support over enabling merge commits, and it states the host's reported flags.
6. [content] The design doc (docs/design/native-pr-landing.md) is updated to describe method discovery, the squash message format and the post-squash resync, and no longer says that PRs always land as merge commits.
7. [content] CHANGELOG.md notes the change. No governance or records files change.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/decisions.md (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/architecture.md (modify)
- crates/athena-core/src/chain.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)

## Out of scope

- Enabling merge commits or changing the repository's settings on the host.
- Support for hosts other than github.com.
- Rewriting the PR #40 history or any records.
- Changes to athena.toml, defaults.toml, CI workflows or lint configuration.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests use a real temporary local git repository; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier runs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix targets broken ancestry after a squash, not the single PR #40 instance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and commit text are written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision revision cites Sri's choice and the host's reported flags.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change, limited to merge method selection, squash message, local resync and the docs.

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
