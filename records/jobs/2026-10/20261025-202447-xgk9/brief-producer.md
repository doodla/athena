# Job 20261025-202447-xgk9: Retry test-suite speedup: keep prior changes, suite under 7 minutes

## Request (verbatim)

> try again on 20261025-190606-eebk

## Reading (accepted by the user)

Retry of job 20261025-190606-eebk, following the user's latest reply: keep the prior approach, including the secrets.rs scan memo and the shorter run_check poll schedule (both allowed, each with a test proving unchanged behavior). The target is the full `cargo test --locked` under 7 minutes on the job's machine, with per-binary before/after times measured the same way and recorded in docs/decisions.md. Audience: the user, who reads docs/decisions.md and later job times.

Assumptions:
- The prior attempt's approach (memo + poll schedule, ~405 s run) is reapplied from the base since changes were discarded; the 7-minute target then holds with margin.
- The memo is keyed on scanner identity plus text, so a replaced binary is not served stale results, to address the earlier security concern; this is a judgment call.
- Before/after timings are measured identically (each binary run alone, both times).

## Acceptance criteria

1. [content] docs/decisions.md has a new decision with per-binary before and after times (lifecycle, job_plan, drain, retry, steward), both measured the same way in the job and labeled measured, with the cause found for each, and the total suite time.
2. [content] The decision records compile 67 s vs run 673 s (brain-measured, 4 cores), lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as rejected, and lists sccache per D79 and a shared target dir as considered; no unmeasured number is claimed.
3. [content] No test is deleted, ignored, filtered or has assertions weakened; test count is not lower than before. athena.toml, ci.yml and tool pins are unchanged. CHANGELOG.md has an entry.
4. [content] Production changes are limited to the secret-scan memo in crates/athena-gate/src/secrets.rs and the shorter poll schedule in crates/athena-core/src/checks.rs; each leaves behavior unchanged and has a test proving it (memo returns identical results to uncached scanning; poll schedule still detects completion, failure and timeout).
5. [behavior] Run `cargo test --locked`: it passes with every test running, the suite's run time is under 7 minutes on the job's machine, and a failing test still fails the run.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/secrets.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-core/src/run.rs (modify)

## Out of scope

- Adopting cargo-nextest
- Changing athena.toml, ci.yml or tool pins
- Deleting or weakening tests

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check suite time under 7 minutes and failing tests still failing.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria; tests prove each production change.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision numbers are measured or labeled.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Per-binary causes are found and recorded.
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

docs/decisions.md has a new decision with per-binary before and after times (lifecycle, job_plan, drain, retry, steward), both measured the same way in the job and labeled measured, with the cause found for each, and the total suite time.
The decision records compile 67 s vs run 673 s (brain-measured, 4 cores), lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as rejected, and lists sccache per D79 and a shared target dir as considered; no unmeasured number is claimed.
No test is deleted, ignored, filtered or has assertions weakened; test count is not lower than before. athena.toml, ci.yml and tool pins are unchanged. CHANGELOG.md has an entry.
Production changes are limited to the secret-scan memo in crates/athena-gate/src/secrets.rs and the shorter poll schedule in crates/athena-core/src/checks.rs; each leaves behavior unchanged and has a test proving it (memo returns identical results to uncached scanning; poll schedule still detects completion, failure and timeout).
Run `cargo test --locked`: it passes with every test running, the suite's run time is under 7 minutes on the job's machine, and a failing test still fails the run.
Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Report

End with the producer report as your structured output.
