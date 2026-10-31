# Job 20261031-142738-g437: Make the CI result fetch read only the push run on the job's CI branch

## Request (verbatim)

> try again on 20261031-001542-anbv

## Reading (accepted by the user)

Retry of job 20261031-001542-anbv, cut to its first slice so one run can finish. Change only the GitHub CI result fetch (`parse_ci_run` and the `CiSource` impl in gh/client.rs, plus tests in gh/tests.rs) so it returns just the `push` run on branch `athena/ci/<job id>`, read from the aggregate `fmt, clippy, tests` job. The goal is a trustworthy exact-tree CI result for the Athena maintainer; wiring it into the run is a follow-up.

Assumptions:
- Job 20261031-001542-anbv failed in planning (round 3 of 3) and its candidate changed no files, so none of its work is kept. Its major finding (the newest `ci` run may be a pull_request run that tests a merge with main) is fixed in this slice by selecting only event `push` on branch `athena/ci/<job id>`. Its minor finding that the reread path calls `self.dispatch` from inside `dispatch` does not apply yet, because this slice does not touch dispatch; it moves to the follow-up that wires the run. Its six scope findings are resolved by listing gh/tests.rs here and moving the run/* files and gh/early.rs to follow-ups.
- Code survey: crates/athena-core/src/gh/client.rs. `parse_ci_run` (about line 519) picks the newest run named `ci` by created_at with no event or branch filter. It copies the conclusion of the one step starting with `cargo fmt --check` to all three checks. `impl CiSource for GhClient::fetch` (about line 570) queries `actions/runs?head_sha=..&per_page=20`. crates/athena-core/src/ci_result.rs: `CiSource::fetch(&self, sha)` and `resolve` check only `run.sha == sha`. .github/workflows/ci.yml has an aggregate job `ci` named `fmt, clippy, tests` (about line 400) next to the separate fmt, clippy and test jobs. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by feeding fixture JSON for both runs to the parser and the source.
- The branch is passed by a new constructor or wrapper, for example `GhClient::ci_source(branch)`, so the `CiSource` trait signature stays unchanged. It adds `&event=push&branch=<branch>` to the runs query, and the parser also filters `event == push` and `head_branch == branch` as a second guard.
- Kept to one producer run: no governance file, workflow, athena.toml or prompt change. This is not a refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core gh`. A new fixture test has two runs for the same SHA, a `pull_request` run with the newer created_at and a different conclusion, and a `push` run on `athena/ci/<job id>`. It shows that only the push run's result is returned.
2. [behavior] Run `cargo test -p athena-core gh`. A new test where only a `pull_request` run exists, or a push run exists on another branch, shows that the source returns no result (None), so `resolve` reports it missing and never uses that run.
3. [behavior] Run `cargo test -p athena-core gh`. A new test shows that the HTTP path built by the CI source includes `event=push` and the branch name. Use the recording transport in gh/tests.rs if present, or assert on the built query.
4. [behavior] Run `cargo test -p athena-core gh`. A new fixture test shows that a failed aggregate `fmt, clippy, tests` job gives red steps, and a successful one gives green steps for all three check names. The current behaviour of copying the `cargo fmt --check` step to all three checks is gone (Grep for `CI_CHECK_STEP` in client.rs).
5. [content] crates/athena-core/src/gh/client.rs filters runs on `event` and `head_branch`. The other changed files are crates/athena-core/src/gh/tests.rs and, only if needed, ci_result.rs call sites. No change appears in run/mod.rs, ci.yml, athena.toml or CHANGELOG.md.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Follow-up request A: wire `resolve` and `run_routed` into the run in run/mod.rs, replacing `NoCi`. This covers the verifier reuse statement, the CI wait bound in athena.toml and config.rs, and the local fallback.
- Follow-up request B: push `athena/ci/<job id>` after admission and delete it at close (athena-git, run/mod.rs, and the reread path calling `dispatch` recursively so the delete runs once). This also covers run/ci.rs, run/verification.rs, run/ci_tests.rs, run/tests.rs and gh/early.rs.
- Follow-up request C: make .github/workflows/ci.yml run on `athena/ci/**` pushes, update docs/design/ci-routing.md section 10 items 4 and 5 and the 'either run serves' text, and add the CHANGELOG.md entry.
- Changing what the CI workflow checks, giving the producer GitHub access, or making real network calls in tests.
- Files the failed job's candidate changed: none were recorded, so nothing is kept or dropped.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and its tests are Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the filter; fmt, clippy and the full test suite must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused slice of the request; the rest is named in out_of_scope as follow-ups.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run fixture tests with competing runs, not just check that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The parser and source are fixed, not just the one call site.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments on the changed functions state which run is trusted and why.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 37 cases in 37 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a failure is attributed to the job or to the base. Add a plan-t...
- 11 cases in 10 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena a disk lifecycle. Before the checks stage, it should automatically delete the finished producer build folder and any other regenerable artifacts (verify-side builds, caches) once they are no longer needed. If free space is still below the guard, it should hold the job instead of failing ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

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

Run `cargo test -p athena-core gh`. A new fixture test has two runs for the same SHA, a `pull_request` run with the newer created_at and a different conclusion, and a `push` run on `athena/ci/<job id>`. It shows that only the push run's result is returned.
Run `cargo test -p athena-core gh`. A new test where only a `pull_request` run exists, or a push run exists on another branch, shows that the source returns no result (None), so `resolve` reports it missing and never uses that run.
Run `cargo test -p athena-core gh`. A new test shows that the HTTP path built by the CI source includes `event=push` and the branch name. Use the recording transport in gh/tests.rs if present, or assert on the built query.
Run `cargo test -p athena-core gh`. A new fixture test shows that a failed aggregate `fmt, clippy, tests` job gives red steps, and a successful one gives green steps for all three check names. The current behaviour of copying the `cargo fmt --check` step to all three checks is gone (Grep for `CI_CHECK_STEP` in client.rs).
crates/athena-core/src/gh/client.rs filters runs on `event` and `head_branch`. The other changed files are crates/athena-core/src/gh/tests.rs and, only if needed, ci_result.rs call sites. No change appears in run/mod.rs, ci.yml, athena.toml or CHANGELOG.md.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
