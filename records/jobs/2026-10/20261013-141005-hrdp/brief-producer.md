# Job 20261013-141005-hrdp: Add athena land --pr (owned mode) with a gh REST host client

## Request (verbatim)

> Retry 20261013-135309-9e8t.

## Reading (accepted by the user)

Retry of the failed PR-landing slice: add an `athena land <job> --pr` command that pushes the job's branch through the push guard, opens the PR, waits for checks, merges, and runs the merged-content check. The host client in a new gh.rs shells out to `gh api` REST endpoints only (no GraphQL). It must contain no email addresses outside athena.toml's email_allow list, which caused the last refusal.

Assumptions:
- The failed job's only defect was the disallowed email address, so its design is otherwise sound and is redone here.
- `gh` is invoked as an external process (gh api), wrapped behind a small command-runner trait so tests need no network or gh binary.
- Check waiting polls the REST check-runs and combined status endpoints with a bounded timeout and interval.
- Merge uses the REST merge endpoint; the merged commit is found by the stored PR number via the pulls endpoint, never from commit trailers.
- The command is `athena land <job> --pr`, in owned mode only; stealth mode stays a later slice.

## Acceptance criteria

1. [content] A new crates/athena-core/src/gh.rs implements the pr::Host trait with `gh api` REST calls (pulls create, pulls get, check-runs/status, pulls merge); no `graphql` call or `gh pr` subcommand appears in it.
2. [content] No file in the change contains an email address outside athena.toml's email_allow list; fixtures use noreply@ or @example.com forms only, or avoid addresses entirely.
3. [content] A `land` subcommand with a `--pr` flag is added to the CLI. It pushes only via pr::check_push (never forced), stores the PrLink with store_link, waits for checks, merges, then calls check_merged and apply_outcome.
4. [content] The command exits 0 only when the outcome is Verified/Landed, and 2 for a merged-but-unverified or failed-checks result, matching main.rs's exit-code convention.
5. [behavior] Run `cargo test`: new unit tests for gh.rs use a fake command runner, no network. They check the exact REST argument lists, response parsing, and the checks-waiting and merge paths, and all pass together with the existing tests.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.
7. [content] docs/design/native-pr-landing.md and CHANGELOG.md note that the command and REST-only host client are done, with the remaining slices (stealth mode and others) still listed as later.

## Expected deliverables

- crates/athena-core/src/gh.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/design/native-pr-landing.md (modify)
- README.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Stealth mode
- GraphQL or any gh subcommand other than gh api
- Changes to athena.toml, the email allow-list, or other governance files
- Multi-job squash handling
- Automated live tests against GitHub

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: gh.rs and the new command get unit tests with fakes; cargo test and clippy must pass.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email addresses outside the allow-list, the cause of the earlier refusal, and no tokens in fixtures.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One slice: the command and its host client only.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use the existing gh CLI rather than an HTTP or octocrab client.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the design and changelog notes are written for outside readers.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The scope was recovered from the quoted prior request, pr.rs, and the design doc.

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
