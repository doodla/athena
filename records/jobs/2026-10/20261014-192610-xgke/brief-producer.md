# Job 20261014-192610-xgke: Make the steward an agent that reads Athena broadly and queues work

## Request (verbatim)

> The steward role never calls a model: steward.rs counts repeated failure classes, and its stored prompt reaches nothing. Make it what the BRIEF.md addendum '2026-10-14: A loop that steers Athena, roles, and a ladder of requirements' describes: an agent with that short stable prompt and liberal read access to everything about Athena (code, prompts, briefs, records, git history, and GitHub CI, PRs and deploys fetched where they live), that works out what should change and queues it.

## Reading (accepted by the user)

Replace the steward's built-in failure counting with a model-backed agent run: the stored short, stable prompt becomes the system prompt, and the agent gets read-only access to code, prompts, briefs, records, git history and GitHub CI/PRs/deploys (via gh), then returns work items and ladder-gated changes that Athena queues. The audience is the Athena owner, who judges whether the steward now reasons over Athena and steers it as the 2026-10-14 BRIEF addendum describes.

Assumptions:
- The agent runs through the existing AgentDriver and Claude Code headless path, so it is budgeted, hermetic and recorded; the driver must be made available to the background thread (e.g. shared via Arc or a fresh driver built from wiring).
- A new Role variant (steward) or reuse of an existing one is chosen by the producer; adding a variant requires updating exhaustive matches, scoring and invariants that treat reader specially.
- GitHub data is fetched where it lives by read-only gh commands inside the agent's policy, not stored in Athena; a missing gh or network is a blind spot, not an error.
- The stored prompt stays short and stable; if its text changes it is a new role version (steward-2) per roles.rs convention.
- Cadence (every five finished tasks) and the ladder limit (defaults-conventions) are kept unchanged; BRIEF says no triggers beyond the parallel loop.
- records/ is not edited directly by the producer; steward output lands through the existing committer.

## Acceptance criteria

1. [content] steward::run invokes an AgentDriver with the stored STEWARD_PROMPT as the system prompt (via the role definition, unchanged in meaning) and records that invocation (role, usage, cost) like other roles' invocations; the call site in note_finished still runs in the background and never blocks the main flow.
2. [content] The steward's invocation is read-only: its tool policy allows reading the repository snapshot, records, prompts, briefs and git history (git log/show/diff) plus read-only GitHub queries (gh run/pr/api GET for CI, PRs, deploys), and denies writes, pushes, and mutating gh/git commands; policy tests cover both allowed and denied cases.
3. [content] The agent returns structured output (work items with kind, key, title, rung, source and evidence references, and a request text); Athena validates it, then still routes every item through decide(): rungs at or below the role's limit queue a request via RequestQueue, higher rungs (owner goals, brief requirements, design decisions) queue only a proposal with evidence.
4. [content] Each source the agent reports it could not read (including GitHub when gh or network is unavailable) becomes a blind-spot work item naming that source; items whose key already exists in records are not queued twice.
5. [content] The old repeated_failures counting is no longer the steward's only reasoning: it is either removed or provided to the agent as input, and docs/design/roles-and-ladder.md and CHANGELOG.md say the steward now calls a model.
6. [behavior] Run `cargo test` (including crates/athena-cli/tests/steward.rs using the fake driver): a steward run with a scripted fake agent output commits the expected work items and queued request, makes no commit when the output is empty, and a nested run makes no real model call.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-data/src/roles.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/invariant.rs (modify)

## Out of scope

- Changing BRIEF.md, athena.toml, defaults.toml or CI workflows
- Adding new steward cadence triggers or a second role
- Storing GitHub data in Athena's records (it is pulled as needed)
- Letting the steward edit files directly instead of queueing requests
- Changing the ladder's rungs or the may_change rule

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-driver tests for the steward run and policy tests; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Read-only access is enforced by the tool policy hook, and the ladder limit by decide(), not by prompt text.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The steward's prompt reaching nothing is fixed by wiring a model call, not by editing the prompt text.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog explain the steward for an outside reader.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing AgentDriver, policy and gh CLI rather than a GitHub client crate.
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
