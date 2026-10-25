# Job 20261013-135309-9e8t: Add athena land --pr for owned mode with a REST-only gh client

## Request (verbatim)

> Next PR-landing slice: the athena land --pr command for owned mode (push the job's branch, open the PR, wait for checks, merge, then run the merged-content check). Its host client must work with gh's REST API only (gh api), not GraphQL.

## Reading (accepted by the user)

Build the command slice D86 deferred: `athena land --pr <job>` pushes the job's branch through `check_push`, opens a PR, waits for green checks, and then lands only when it observes the merge, with `check_merged` as the final check. The host client shells out to `gh api` REST paths only. Per the user's reply, Athena merges via REST only when a setting, off by default, opts in; otherwise it stops at an open, green PR.

Assumptions:
- The opt-in is a boolean setting in athena.toml's config struct (athena-data config), default false, named something like land.merge; its name is the producer's call and is documented.
- Without opt-in, the command opens the PR, waits for checks, reports 'open, green; merge it yourself', and exits; a later `athena land --pr <job>` run (or the same command re-run) looks up the stored link, observes the merge via REST, and runs check_merged.
- The REST calls are: GET git ref and compare for the remote head, POST pulls, GET commits/{sha}/check-runs and status, PUT pulls/{n}/merge (opt-in only), and GET pulls/{n} for merged state and merge_commit_sha.
- The push itself uses git (not gh), after check_push; the branch is the job's work branch, with no force.
- Check waiting uses an injected clock/poll interval and a timeout, so tests do not sleep.
- Athena's own governance files (athena.toml) do not change; the new setting lives in code defaults.

## Acceptance criteria

1. [content] A new host-client module in crates/athena-core implements `Host` and new traits (open PR, read check status, merge) by running `gh api` with REST paths only. No source contains `graphql` outside the test that asserts its absence, and no `gh pr` subcommand is used.
2. [content] The CLI has `athena land --pr <job>` with an optional `--json`, wired in crates/athena-cli and documented in `--help`.
3. [content] The push goes through `check_push` first and has no force option. The link is saved with `store_link`, outcomes go through `apply_outcome`, and the link survives `clean_job` until the merge is observed.
4. [content] A setting, default off, decides whether Athena merges. When off, Athena never calls the merge endpoint; it stops at an open PR with green checks and reports that. When on, it merges via the REST merge endpoint.
5. [content] Landing is recorded only after the merge is observed by PR number and `check_merged` returns Verified; a merge of other content gives merged-unverified.
6. [behavior] Run `cargo test`. Fake-runner tests pass for: green checks with merge off (no merge call, PR left open), merge on (merged then landed), failed checks (no merge), check timeout, non-fast-forward push refused, merged-unverified, and every recorded gh api call is REST (no graphql).
7. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`; both are clean. `cargo test` for the whole workspace passes, including the existing pr.rs tests.
8. [content] docs/decisions.md has a new entry recording the buy-first call (gh versus an HTTP crate), the REST-only choice, and the merge opt-in. docs/design/native-pr-landing.md says the command is built, and CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-core/src/gh.rs (create)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/fake.rs (modify)
- docs/decisions.md (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/enforcement.md (modify)
- crates/athena-core/Cargo.toml (modify)

## Out of scope

- Stealth mode landing
- GraphQL or GitHub auto-merge
- An HTTP client crate
- Force pushes or pushing branches other than the job's work branch
- Changes to athena.toml, CI workflows, or other governance files
- Hosts other than GitHub

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust; the gh CLI is only an external program invoked, no scripts.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Shell out to the maintained gh CLI rather than adding an HTTP crate; the call is recorded in a decision.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake runner/host tests cover each outcome; cargo test and clippy are clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One slice: the owned-mode command only, not stealth mode or auto-merge.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test asserts every recorded gh api call is REST and never graphql; merge only runs when the setting is on.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision entry are written for an outside reader.

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
