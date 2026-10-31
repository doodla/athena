# Job 20261031-001537-0rz6: Split CI into parallel fmt, clippy and sharded test jobs with xtask

## Request (verbatim)

> Priority: urgent
> Split CI's test run across many GitHub runners so a full check of one commit takes minutes, not 16
> 
> The `fmt, clippy, tests` job in .github/workflows/ci.yml runs everything on one runner, about 16 minutes on a cold cache (docs/decisions.md D125). GitHub runners are free and parallel, so use many of them. This is the first step of moving Athena's checks off the local machine (docs/design/ci-routing.md); the request to read the result back into a job is separate.
> 
> Change ci.yml so that fmt, clippy and the tests are separate jobs that run at the same time, and the tests are split into a matrix of shards (start with 6) that together run every test exactly once. Keep the existing cache layers, the fingerprint pass marker (D121) and the SHA pinning of actions. Give every job a timeout-minutes. Add one final job named `ci` that needs all the others and fails if any failed, so a single check name still says whether a commit is green.
> 
> The producer cannot run GitHub Actions. Acceptance is checked from the files and local tests only; the real run time is measured later by a throughput review.
> 
> Acceptance:
> - ci.yml runs fmt, clippy and the test shards as separate jobs, plus a final `ci` job that needs them all; every job has timeout-minutes.
> - An xtask test (or a test in an existing crate) reads ci.yml and fails if the shard list is not a partition: the test command each shard runs, taken together, covers every workspace test target exactly once.
> - The shard split is done by a command the repository owns (an xtask subcommand), so the same split can be run locally with `cargo run -p xtask -- test-shard <k> <n>`; running all n shards locally runs the same tests as `cargo test --workspace`.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: .github/workflows/ci.yml, xtask/, CHANGELOG.md

## Reading (accepted by the user)

Replace the single `checks` job in ci.yml with parallel fmt, clippy and a 6-shard test matrix, plus a final `ci` gate job. Add an `xtask test-shard <k> <n>` subcommand that partitions workspace test targets, and a test that reads ci.yml and checks the shards form an exact partition. The audience is Athena's maintainers, who need a fast single required check.

Assumptions:
- Code survey: read .github/workflows/ci.yml and xtask/src/main.rs (entry `main` dispatches subcommands; `ci()` at ~line 536 runs fmt --all --check, clippy --all-targets --locked -D warnings, then `cargo test --locked` in sequence). xtask/src/ci_cache.rs holds fingerprint and ci-cache check/mark/report. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched; xtask/tests/ already holds integration tests.
- The D121 pass marker stays: the fingerprint and gate steps run once in the fmt/clippy/test jobs' shared prelude (or a small first job whose outputs the others need); the marker is written only by the `ci` job or a job that needs all others, after all succeeded, so a failed shard never leaves a marker. Producer picks the least invasive design and documents it in comments.
- Sharding unit is the workspace test target (each lib, bin, integration test file), enumerated deterministically by xtask (e.g. cargo metadata), assigned by index modulo n; shard k runs `cargo test --locked -p <pkg> --test <name>` / `--lib` for its targets, including doctests assigned to exactly one shard. All n shards together equal `cargo test --workspace`.
- Cache layers keep their keys; each job gets its own target cache key (target-fmt/clippy/test) to avoid collisions, with restore-only on every event and save on default branch only. Existing history, attribution and pr-head jobs are unchanged apart from nothing.
- The `ci` gate uses `if: always()` and fails unless every needed job result is `success`; the old `checks` name is replaced by `ci` as the required check (branch protection is outside the repo).

## Acceptance criteria

1. [content] Grep of .github/workflows/ci.yml shows separate jobs for fmt, clippy and a test job with `strategy.matrix` of 6 shards, plus a job with `name: ci` whose `needs` lists fmt, clippy and the test job and which uses `if: always()` and fails when any needed result is not success; every job has `timeout-minutes`.
2. [content] All `uses:` lines in ci.yml remain pinned by 40-hex commit SHA, the three cache layers (toolchain, registry, target) and the D121 fingerprint/ci-pass lookup, gate and mark steps are still present, and the pass marker is saved only after all checks succeeded.
3. [behavior] An xtask test reads .github/workflows/ci.yml, extracts each shard's test command and the shard count, and fails when the shards do not partition the workspace test targets exactly once; run `cargo test -p xtask` and see it pass, and a negative case inside the test (a mutated shard list with a dropped or duplicated shard) is shown to fail the check.
4. [behavior] `cargo run -p xtask -- test-shard <k> <n>` exists; for n=6 and k in 0..5 (or the documented index base) the selected targets are disjoint and their union equals the target set of `cargo test --workspace`; a test in xtask asserts this, and invalid k/n (k>=n, n=0) exits with an error. Run `cargo test -p xtask`.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry describing the parallel jobs, the shard matrix, the `ci` gate job and `xtask test-shard`.
7. [content] Outcome check: the real run time is not claimed; comments or the changelog state it is to be measured later by the throughput review, and no number is presented as measured.

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/src/main.rs (modify)
- xtask/src/shard.rs (create)
- xtask/tests/ci_shards.rs (create)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/design/ci-routing.md (modify)

## Out of scope

- Reading CI results back into a job (a separate request)
- Measuring real GitHub run time or changing branch protection settings
- Changing the history, attribution and pr-head jobs
- Changes to athena.toml, defaults.toml, setup.sh or rust-toolchain.toml
- Tests that call real models

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The shard command and its test are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask tests; cargo test and clippy -D warnings must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The partition rule is enforced by a test that reads ci.yml.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the partition and the shard command by running tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion states the run time is not claimed and is measured later.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to ci.yml plus xtask.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use cargo metadata and the existing test runner rather than adding a sharding tool; record the call in a comment.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Workflow comments and changelog written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 7 cases in 7 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Land b5hm first and extend it. Before the checks stage, and again when free disk drops below the guard, Athena should delete regenerable artifacts itself: the verify-side build folder of finished baseline checks, the workspace target, and the uv and npm caches. If that is still not enough, the job s...

22 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Grep of .github/workflows/ci.yml shows separate jobs for fmt, clippy and a test job with `strategy.matrix` of 6 shards, plus a job with `name: ci` whose `needs` lists fmt, clippy and the test job and which uses `if: always()` and fails when any needed result is not success; every job has `timeout-minutes`.
All `uses:` lines in ci.yml remain pinned by 40-hex commit SHA, the three cache layers (toolchain, registry, target) and the D121 fingerprint/ci-pass lookup, gate and mark steps are still present, and the pass marker is saved only after all checks succeeded.
An xtask test reads .github/workflows/ci.yml, extracts each shard's test command and the shard count, and fails when the shards do not partition the workspace test targets exactly once; run `cargo test -p xtask` and see it pass, and a negative case inside the test (a mutated shard list with a dropped or duplicated shard) is shown to fail the check.
`cargo run -p xtask -- test-shard <k> <n>` exists; for n=6 and k in 0..5 (or the documented index base) the selected targets are disjoint and their union equals the target set of `cargo test --workspace`; a test in xtask asserts this, and invalid k/n (k>=n, n=0) exits with an error. Run `cargo test -p xtask`.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has an entry describing the parallel jobs, the shard matrix, the `ci` gate job and `xtask test-shard`.
Outcome check: the real run time is not claimed; comments or the changelog state it is to be measured later by the throughput review, and no number is presented as measured.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
