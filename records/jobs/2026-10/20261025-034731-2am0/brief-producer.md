# Job 20261025-034731-2am0: Reuse passing CI result by governed fingerprint on landing

## Request (verbatim)

> Priority: high
> Speed first, cut A (BRIEF.md addendum of 2026-10-25): stop GitHub CI rerunning the full suite on a landing commit whose code was already tested. Measured from the logs of 33 jobs between 2026-10-23 and 2026-10-25: landing takes a median 15.4 minutes, most of it the pull request's 'fmt, clippy, tests' job, although the landing commit differs from the verified candidate only by records-only commits. Key CI's check result on the governed-content fingerprint (the one athena-gate already computes, which ignores records/ and owner paths) in CI's own cache: when a run for the same fingerprint already passed, the job reports that result and skips rebuilding. Have athena land push the verified candidate for CI early (when verification starts), so a passing result exists by landing. CI trusts only its own cached results, never Athena's records. Keep the job name 'fmt, clippy, tests' (the ruleset requires it).
> Acceptance:
> - A workflow test or script test shows a records-only change after a passing run reuses the result, and any governed change runs the full job.
> - A failed or missing cached result never passes the job.
> - docs/decisions.md records the measured landing time before, and after on landings run after the change.
> Files: .github/workflows/ci.yml, xtask/src/main.rs, crates/athena-core/src/gh.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Make the 'fmt, clippy, tests' job key a branch-scoped CI cache on the governed-content fingerprint, skipping the build when a passing entry exists, and have athena land push the candidate to its PR branch at verification start so the entry exists by landing. Record measured before-times in docs/decisions.md and add a changelog entry.

Assumptions:
- Goal: cut landing time for Athena maintainers by letting the required 'fmt, clippy, tests' job reuse a passing result for an unchanged governed fingerprint.
- Fingerprint computation is exposed from athena-gate logic to xtask, reusing existing code rather than duplicating it.
- Cache is actions/cache scoped to the PR branch; the early push goes to that branch per the user's reply.
- The 'after' numbers will be pending unless real landings exist after the change.

## Acceptance criteria

1. [behavior] A new xtask test (run: cargo test -p xtask) shows that after a passing result is stored for a fingerprint, a change touching only records/ or owner paths yields the same fingerprint and reuses the result, and a change to a governed file yields a different fingerprint and runs the full job.
2. [behavior] A test (run: cargo test -p xtask) shows a missing, failed, malformed or unreadable cached result never lets the job pass; the full checks run.
3. [content] ci.yml keeps the job name 'fmt, clippy, tests', keys the cache only on the governed fingerprint, saves an entry only after the full run passes, reads no Athena records, and states its trust model in a comment. Main's shared caches are written only by pushes to main.
4. [behavior] A test in gh.rs (run: cargo test -p athena-core) shows athena land, when verification starts, pushes the candidate through the Host seam to the job's own pull-request branch (the branch land later points the PR at), and a push failure does not fail verification. Landing still waits on the required check.
5. [content] docs/decisions.md has a dated entry with measured landing time before (median 15.4 minutes over 33 jobs, 2026-10-23 to 2026-10-25), the mechanism, rejected alternatives, and the branch-scoped cache explanation: the entry is saved by the run on the PR branch and restored by the landing run on that same branch (ci-routing section 4.1). The 'after' section is filled only from real post-change landings, or labeled pending with no invented numbers.
6. [content] CHANGELOG.md has an entry. cargo test, cargo clippy -D warnings and cargo fmt --check pass.

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/src/main.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the ruleset or required check name
- Changing athena-gate's fingerprint definition
- Trusting Athena records in CI
- Writing main's shared caches from non-main branches

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in xtask and gh.rs; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check reuse and full-run behavior via tests.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision entry cites measured 15.4 min median over 33 jobs.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the workflow or docs.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Cache trust enforced in the workflow and tested logic.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The after-landing time is the outcome check, recorded only when measured.
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

A new xtask test (run: cargo test -p xtask) shows that after a passing result is stored for a fingerprint, a change touching only records/ or owner paths yields the same fingerprint and reuses the result, and a change to a governed file yields a different fingerprint and runs the full job.
A test (run: cargo test -p xtask) shows a missing, failed, malformed or unreadable cached result never lets the job pass; the full checks run.
ci.yml keeps the job name 'fmt, clippy, tests', keys the cache only on the governed fingerprint, saves an entry only after the full run passes, reads no Athena records, and states its trust model in a comment. Main's shared caches are written only by pushes to main.
A test in gh.rs (run: cargo test -p athena-core) shows athena land, when verification starts, pushes the candidate through the Host seam to the job's own pull-request branch (the branch land later points the PR at), and a push failure does not fail verification. Landing still waits on the required check.
docs/decisions.md has a dated entry with measured landing time before (median 15.4 minutes over 33 jobs, 2026-10-23 to 2026-10-25), the mechanism, rejected alternatives, and the branch-scoped cache explanation: the entry is saved by the run on the PR branch and restored by the landing run on that same branch (ci-routing section 4.1). The 'after' section is filled only from real post-change landings, or labeled pending with no invented numbers.
CHANGELOG.md has an entry. cargo test, cargo clippy -D warnings and cargo fmt --check pass.

## Report

End with the producer report as your structured output.
