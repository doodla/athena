# Job 20261025-214509-y179: Cut slow athena-cli test time; full cargo test under 7 minutes

## Request (verbatim)

> try again on 20261025-202447-xgk9

## Reading (accepted by the user)

Retry of job 20261025-202447-xgk9, keeping the user's decided approach. No nextest. Cut real time from the slowest athena-cli test binaries. Add the secrets.rs scan memo and a shorter run_check poll schedule, each with a behavior-preserving test. Goal: full `cargo test --locked` under 7 minutes. Audience: the user, who reads docs/decisions.md and later job times.

Assumptions:
- The user's replies to the interrupted attempt carry over to this retry unchanged.
- The job's machine is comparable to the 4-core brain machine; times are measured in the job.
- The previous attempt left no findings and nothing carries over, so the producer starts from the base.

## Acceptance criteria

1. [content] docs/decisions.md has a new decision with per-binary before and after times (lifecycle, job_plan, drain, retry, steward), measured the same way in the job and labeled measured, with the cause found for each and the total suite time.
2. [content] The decision records compile 67 s vs run 673 s (brain-measured, 4 cores). It lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as rejected. It lists sccache (D79) and a shared target dir as considered. It claims no unmeasured number.
3. [content] No test is deleted, ignored, filtered or has assertions weakened, and the test count is not lower than before. athena.toml, ci.yml and tool pins are unchanged. CHANGELOG.md has an entry.
4. [content] Production changes are limited to the scan memo in crates/athena-gate/src/secrets.rs and the shorter poll schedule in crates/athena-core/src/checks.rs. Each leaves behavior unchanged and has a test: the memo equals uncached scanning, and the poll schedule still detects completion, failure and timeout.
5. [behavior] Run `cargo test --locked`. It passes with every test running, and the run is under 7 minutes on the job's machine. A failing test still fails the run.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`. Both are clean.

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

- Adopting cargo-nextest or sccache
- Changing athena.toml, ci.yml or tool pins
- Removing, ignoring or weakening any test

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision labels measured numbers and cites the brain's measurements.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Test, clippy and fmt criteria are included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The suite time and the memo and poll behavior are checked by running tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Production changes are limited to two files.
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

docs/decisions.md has a new decision with per-binary before and after times (lifecycle, job_plan, drain, retry, steward), measured the same way in the job and labeled measured, with the cause found for each and the total suite time.
The decision records compile 67 s vs run 673 s (brain-measured, 4 cores). It lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as rejected. It lists sccache (D79) and a shared target dir as considered. It claims no unmeasured number.
No test is deleted, ignored, filtered or has assertions weakened, and the test count is not lower than before. athena.toml, ci.yml and tool pins are unchanged. CHANGELOG.md has an entry.
Production changes are limited to the scan memo in crates/athena-gate/src/secrets.rs and the shorter poll schedule in crates/athena-core/src/checks.rs. Each leaves behavior unchanged and has a test: the memo equals uncached scanning, and the poll schedule still detects completion, failure and timeout.
Run `cargo test --locked`. It passes with every test running, and the run is under 7 minutes on the job's machine. A failing test still fails the run.
Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`. Both are clean.

## Report

End with the producer report as your structured output.
