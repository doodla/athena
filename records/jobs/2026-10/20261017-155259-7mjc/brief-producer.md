# Job 20261017-155259-7mjc: Wire CI push and wait into checks without breaking land --pr

## Request (verbatim)

> try again on 20261017-152733-efyd

## Reading (accepted by the user)

Retry the rejected CI-routing wiring: push the candidate after the producer, wait a bounded time for CI on that exact SHA, and reuse it in the checks step with the local fallback kept. Unlike the rejected attempt, the push must not break the later `athena land --pr`, and the run.rs wiring and the push-to-land sequence need tests.

Assumptions:
- Goal: the Athena operator's job checks take CI's time rather than ~29 minutes locally; audience is that operator and the verifier.
- Reading follows the proposed next request: keep the earlier scope, and fix the push/land non-fast-forward conflict by pushing to a separate ref or making publish safe.
- Exact ref choice is left to the producer; a separate candidate ref is assumed simplest, provided CI triggers on it.
- athena.toml changes only if the wait bound becomes a config field.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: passes. A test on the run.rs wiring (not only routing::run_pushed with a test-only bound) shows the checks step gets a platform-backed CI source, not NoCi, and uses the CI_WAIT bound rather than one poll; it fails if run.rs reverts to NoCi or one poll.
2. [behavior] Run `cargo test`: with fake push and CI sources, the candidate is pushed before CI is read and the same SHA is used for both. A green run for that SHA runs no local check and records CI provenance with the run address. A run for another SHA, no run after the bounded wait, or a push failure each run the three checks locally exactly once. A red run fails without running locally.
3. [behavior] Run `cargo test`: an integration test on one job branch runs candidate push, then CI read, then publish/land --pr, and shows the land push is accepted, not refused as a non-fast-forward (cf. pr::tests::the_guard_refuses_a_non_fast_forward_push). Landing does not force-push over unrelated work.
4. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both are clean.
5. [content] The push uses the existing guarded path (check_push), never the default branch. It targets a ref landing's publish does not conflict with, or publish handles the existing branch safely. The candidate commit message no longer says 'Not on any branch'.
6. [content] The wait bound is a named, documented value with a stated basis; run output says whether checks came from CI or ran locally. The cost of waiting when CI never reports is documented. If athena.toml gets a field, it is named and the schema/load tests are updated.
7. [content] docs/design/ci-routing.md section 10 items on push and wait are no longer 'not done'; the verifier's reuse is still listed as not done, and no speedup is claimed without measurement. The fmt, clippy and test check definitions, verifier input and prompts are unchanged.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- docs/design/ci-routing.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- The verifier reusing CI results.
- Changing the fmt, clippy or test check definitions.
- Changing verifier input or prompts.
- Pushing to the default branch.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Behavior criteria name cargo test, clippy and fmt commands.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check wiring and land interaction by behavior, not mere existence.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the push/land interaction itself, not only the one failing case.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome criteria: CI reuse happens and land still works.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, verifier reuse excluded.
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
