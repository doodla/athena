# Job 20261025-172350-xrbk: Limit CI push trigger to main to stop duplicate PR runs

## Request (verbatim)

> Priority: urgent
> CI runs every job twice on Athena's pull-request branches: .github/workflows/ci.yml triggers on every push to every branch as well as on pull_request, and push runs live in a different cache scope from pull-request runs, so they miss cut A's cached pass. On PR #185 the pull_request run's 'fmt, clippy, tests' reused the cached pass in 26 s, but two push-triggered runs of the same commit ran the full 16 minutes and the landing waited for them (17m51s). Limit the push trigger to main (keep pull_request, workflow_dispatch and the schedule), so a pull-request branch gets one run per commit, in the pull request's scope. The required checks stay satisfied by the pull_request run.
> Acceptance:
> - ci.yml's push trigger names only main, and its comment says why.
> - docs/decisions.md records it with the PR #185 timings; CHANGELOG.md has an entry.
> - The next landing's pull request shows one run per commit (checked after landing).
> Files: .github/workflows/ci.yml, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Restrict ci.yml's push trigger to main (keeping pull_request, workflow_dispatch, schedule) with a comment explaining why, record the decision with PR #185 timings in docs/decisions.md, and add a CHANGELOG entry. Goal: one CI run per PR commit, so landing stops waiting on duplicate 16-minute push runs.

Assumptions:
- Goal: cut duplicate CI runs on PR branches so landing stops waiting ~16 min; audience is Athena maintainers and the landing flow.
- Use `push: branches: [main]`; main is the default branch (cache saving already keyed on default-branch push).
- Decision number follows the latest D-number in docs/decisions.md (D121 is referenced in ci.yml).
- The post-landing observation criterion cannot be verified by the producer; recorded as a follow-up.

## Acceptance criteria

1. [content] .github/workflows/ci.yml's `on.push` is `branches: [main]` only, with pull_request, workflow_dispatch and the weekly schedule unchanged, and a comment saying why (push runs use a different cache scope and miss the pull-request run's cached pass; PR #185 ran twice for 16 min extra).
2. [content] docs/decisions.md has a new decision entry, numbered after the latest, with alternatives and the PR #185 timings (pull_request run 26 s via cached pass; two push runs ~16 min; landing waited 17m51s), labeled as taken from the request.
3. [content] CHANGELOG.md has an entry under [Unreleased] (Changed) describing the push trigger limit.
4. [content] Required checks remain satisfied by the pull_request run: job name 'fmt, clippy, tests' is unchanged and no other workflow or config edits are made.
5. [content] Header comment in ci.yml that refers to the cache scope remains consistent with the new trigger.
6. [behavior] cargo test and cargo clippy pass (including any xtask/tests that parse ci.yml or check docs/changelog consistency).
7. [content] Outcome check: the next landing's PR shows one run per commit; this can only be observed after landing and is noted in the decision as a follow-up, not claimed as done.

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing cache scopes, cache keys or the pass-marker logic
- Changing job names or branch protection settings
- Other workflows

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision entry cites PR #185 timings as given in the request.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comment and decision written for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The trigger restriction is enforced by the workflow itself.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion on one run per commit after landing.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across three named files.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy still pass.
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

.github/workflows/ci.yml's `on.push` is `branches: [main]` only, with pull_request, workflow_dispatch and the weekly schedule unchanged, and a comment saying why (push runs use a different cache scope and miss the pull-request run's cached pass; PR #185 ran twice for 16 min extra).
docs/decisions.md has a new decision entry, numbered after the latest, with alternatives and the PR #185 timings (pull_request run 26 s via cached pass; two push runs ~16 min; landing waited 17m51s), labeled as taken from the request.
CHANGELOG.md has an entry under [Unreleased] (Changed) describing the push trigger limit.
Required checks remain satisfied by the pull_request run: job name 'fmt, clippy, tests' is unchanged and no other workflow or config edits are made.
Header comment in ci.yml that refers to the cache scope remains consistent with the new trigger.
cargo test and cargo clippy pass (including any xtask/tests that parse ci.yml or check docs/changelog consistency).
Outcome check: the next landing's PR shows one run per commit; this can only be observed after landing and is noted in the decision as a follow-up, not claimed as done.

## Report

End with the producer report as your structured output.
