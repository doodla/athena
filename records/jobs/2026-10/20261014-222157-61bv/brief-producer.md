# Job 20261014-222157-61bv: Pass GitHub credentials to the steward so gh can read CI and PRs

## Request (verbatim)

> The steward could not read github: GitHub CI runs, pull requests and deploys could not be read: gh is not authenticated (`gh run list` exited 4 and asked for `gh auth login` or GH_TOKEN). Find out why, and make it readable, or record why it cannot be.

## Reading (accepted by the user)

The steward's `gh` fails because agents start from an empty environment and `athena.toml` `[agents] env_pass_if_set` lists no GitHub variable (`GH_TOKEN`, `GITHUB_TOKEN`). I will add the GitHub variables to that list, record the decision, and update the steward docs. The audience is the steward and the owner who reads its blind-spot report.

Assumptions:
- Root cause, read from crates/athena-agent/src/env.rs and athena.toml: agents start with an empty environment and only HOME and PATH are always passed. Neither GH_TOKEN nor GITHUB_TOKEN is on the list, so gh cannot authenticate unless a gh config under HOME exists.
- docs/design/roles-and-ladder.md already says the steward is only as able to read GitHub as env_pass_if_set allows, so the allow-list is the intended place for the fix.
- The change adds GH_TOKEN, GITHUB_TOKEN, GH_HOST and GH_ENTERPRISE_TOKEN, each passed only when set. I did not add GH_CONFIG_DIR.
- I cannot confirm that the owner's shell has a token set. If it does not, the change alone will not make gh readable. The docs will then say the owner must export a read-only token before running Athena.
- Only the steward's gh reads are allowed by policy.rs, and the token reaches every agent role's environment. The decision record will state that tradeoff and recommend a read-only token.

## Acceptance criteria

1. [content] athena.toml `[agents] env_pass_if_set` includes GH_TOKEN and GITHUB_TOKEN, and `env_pass` is unchanged. The comment above the list mentions GitHub.
2. [behavior] Run `cargo test -p athena-agent`. A new env.rs test shows that GH_TOKEN and GITHUB_TOKEN appear in the built environment when the lookup supplies them, and are absent when unset or empty.
3. [content] docs/decisions.md has a new decision, in the same format as D53, that explains why gh failed (an empty agent environment with no GitHub variable). It names the alternatives (a gh config under HOME; no access with a recorded blind spot) and the tradeoff of a token in the agent environment.
4. [content] docs/design/roles-and-ladder.md's GitHub bullet names the variables passed and tells the owner to export a read-only token. The steward docs state that without it the `github` blind spot is reported.
5. [behavior] Run `cargo test` and `cargo clippy`. Both pass, including any test that parses athena.toml.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- docs/design/roles-and-ladder.md (modify)
- crates/athena-agent/src/env.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-core/src/steward.rs (modify)

## Out of scope

- Creating, storing or printing any token or credential
- Running `gh auth login` or changing the owner's machine or shell setup
- Loosening the gh read-only command policy in crates/athena-agent/src/policy.rs
- Changing the model, budget or timeout of the steward

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix targets the allow-list that strips the credential, not a one-off workaround.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The allow-list in athena.toml is the mechanism, and a test pins that the GitHub variables pass only when set.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No token value is written anywhere; only variable names are.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a unit test for env::build with GH_TOKEN set and unset; cargo test and clippy stay clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision record cites env.rs and athena.toml as its sources.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One config line, one decision, one doc sentence and one test.
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

## Report

End with the producer report as your structured output.
