# Job 20261025-172350-xrbk: verify "Limit CI push trigger to main to stop duplicate PR runs"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the on: block of the changed workflow and its diff. Run: `git diff -- .github/workflows/ci.yml`. Observes: The push entry has only `branches: [main]`; the pull_request, workflow_dispatch and cron lines are unchanged; a comment above the push entry states the cache-scope reason and cites D125 and PR #185.
- Criterion 2: Read the appended decision text. Run: `git diff -- docs/decisions.md`. Observes: A new `## D125` entry follows D124 with Case, Decision, Alternatives (the rejected options above), Tradeoff, Reversal cost and Provenance. It gives the PR #185 timings (pull_request run 26 s via cached pass, two push runs about 16 min, landing waited 17m51s) and labels them as stated in the request. Earlier lines are unchanged.
- Criterion 3: Read the changelog diff. Run: `git diff -- CHANGELOG.md`. Observes: One new bullet under `## [Unreleased]` in `### Changed`, after the D124 bullet, describing the push trigger limit to main and pointing to D125.
- Criterion 4: Confirm the change set and that the job name line is untouched. Run: `git diff --stat`. Observes: Only .github/workflows/ci.yml, docs/decisions.md and CHANGELOG.md appear. In the ci.yml diff, `name: fmt, clippy, tests` and the job structure are unchanged.
- Criterion 5: Read the header comment (lines 1-11) to confirm it is unchanged and still accurate for the new trigger. Run: `git diff -- .github/workflows/ci.yml`. Observes: Lines 1-11 do not appear in the diff. The text still says only a default-branch push saves shared caches and that pass markers are saved on pull-request branches, which remains true with the new trigger.
- Criterion 6: Run the Rust test and lint commands. Run: `cargo test --locked; cargo clippy`. Observes: Both finish with exit status 0 and no warnings. The policy tests that read ci.yml (xtask/tests/policy.rs, checks_job_steps) and the D122 log-hash test still pass.
  Fails without the behavior: A broken YAML edit, a changed job name, or an edit to an earlier decision entry would fail cargo test; a clippy warning would fail cargo clippy.
- Criterion 7: Cannot be run before landing. The producer and verifier cannot observe the next landing's pull request. It is recorded in D125 as a follow-up. Run: `none runnable before landing`. Observes: After landing: the next landing's pull request shows one CI run per commit. Until then the claim is not made.

Planned files:

- .github/workflows/ci.yml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 8c88bd0bcee3 → candidate 03ed9053cc92. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml |  5 +++++
 CHANGELOG.md             |  5 +++++
 docs/decisions.md        | 31 +++++++++++++++++++++++++++++++
 3 files changed, 41 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1239 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
