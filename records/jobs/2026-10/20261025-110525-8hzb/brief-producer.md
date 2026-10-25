# Job 20261025-110525-8hzb: Open the early pull request right after the candidate commit

## Request (verbatim)

> Priority: urgent
> Cut A saves nothing yet: the early pull request opens when verification starts, but Athena's own checks (about 12 minutes) run before verification, so CI gets about 2 minutes' head start on a 16-minute run. On PR #180 the early run's 'fmt, clippy, tests' passed at 10:46:49 but the landing commit's run started at 10:45:15, found no cached pass, and ran the full 16 minutes. Open or update the job's pull request as soon as the producer's candidate is committed, before Athena's checks, so CI runs alongside the checks and the verifier; the merge still waits for athena land after verification passes.
> Acceptance:
> - A test with the fake host shows the pull request opened or updated right after the candidate is committed and before the checks start.
> - A candidate that fails Athena's checks or the verifier is never merged, and its pull request is left for the retry or closed with the job.
> - The landing time of the next real landing is recorded in CHANGELOG.md as measured.
> Files: crates/athena-core/src/run.rs, crates/athena-core/src/gh.rs, CHANGELOG.md

## Reading (accepted by the user)

Move the early push and pull request call so it runs as soon as the producer's candidate is committed, ahead of admission and Athena's own checks. That way CI overlaps the checks and the verifier instead of starting about 12 minutes later. The merge stays gated on verification and athena land. The audience is the maintainer, who wants the landing commit to find a cached CI pass.

Assumptions:
- In run.rs early_pr is called after admission and fingerprinting, and a candidate that fails admission returns before it. The fix is to call it right after the candidate commit is made, before admission and the checks. Repository reading found the call at run.rs:1645, with the checks at about line 1700.
- The existing test in run_pin_tests.rs is the natural home for the ordering test even though the request lists only run.rs, gh.rs and CHANGELOG.md.
- The real landing time cannot be measured by the producer. The CHANGELOG records a placeholder or a note that it is to be recorded at the next real landing, and is not filled with a made-up number.
- A failed candidate's pull request stays open for the retry, which force-updates the same branch. Closing it with the job is existing behavior and is left as is.

## Acceptance criteria

1. [behavior] A test using the fake host (run_pin_tests.rs, extending early_pr_pushes_the_candidate_and_opens_the_pull_request...) shows the push and open_pr calls happen after the candidate commit exists and before the first Athena check starts and before the verifier starts. Run: cargo test -p athena-core early_pr
2. [behavior] A test shows that a candidate failing Athena's checks or rejected by the verifier produces no merge call on the fake host, and that its pull request is left for the retry or closed with the job. Run: cargo test -p athena-core
3. [behavior] A failing early push or open still only yields the single warning line and the same job outcome. Run: cargo test -p athena-core a_failing_early_step
4. [content] In run.rs the early_pr call sits directly after the candidate is committed, before the admission and check steps, with the comment updated to say so. gh.rs changes only if the early-PR helper needs it.
5. [content] CHANGELOG.md has an entry for this change. It says the landing time of the next real landing is to be recorded there as measured and labels any number not yet measured as an estimate. It does not invent a measured landing time.
6. [behavior] cargo clippy and cargo fmt --check are clean and cargo test passes.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Merging before verification passes
- Changing how athena land waits for CI or merges
- Changing which checks Athena runs or their duration
- Changes to CI workflow files, athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-host tests cover ordering and the never-merged-on-failure case. Clippy and fmt are clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check call order and the absence of a merge on the fake host, not that the wiring exists.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The CHANGELOG landing time is labeled measured or estimate.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix moves the call to the earliest point after the candidate commit.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused reordering plus tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
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

A test using the fake host (run_pin_tests.rs, extending early_pr_pushes_the_candidate_and_opens_the_pull_request...) shows the push and open_pr calls happen after the candidate commit exists and before the first Athena check starts and before the verifier starts. Run: cargo test -p athena-core early_pr
A test shows that a candidate failing Athena's checks or rejected by the verifier produces no merge call on the fake host, and that its pull request is left for the retry or closed with the job. Run: cargo test -p athena-core
A failing early push or open still only yields the single warning line and the same job outcome. Run: cargo test -p athena-core a_failing_early_step
In run.rs the early_pr call sits directly after the candidate is committed, before the admission and check steps, with the comment updated to say so. gh.rs changes only if the early-PR helper needs it.
CHANGELOG.md has an entry for this change. It says the landing time of the next real landing is to be recorded there as measured and labels any number not yet measured as an estimate. It does not invent a measured landing time.
cargo clippy and cargo fmt --check are clean and cargo test passes.

## Report

End with the producer report as your structured output.
