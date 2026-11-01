# Job 20261031-230912-n9tm: Use GitHub CI result for the candidate in the checks stage

## Request (verbatim)

> Priority: urgent
> Depends on 20261031-230906-3rze
> Use GitHub's CI result for the candidate instead of the local checks
> 
> Second of two small jobs replacing 20261031-150034-17c3. After 20261031-230906-3rze, each candidate is pushed to athena/ci/<job id>, and reading a result already returns only that branch's push run (landed earlier on 2026-10-31).
> 
> In the checks stage, replace the NoCi source passed in run/mod.rs (about lines 1264-1275) with a source chosen together with its wait (with_ci_source): with a GitHub origin, the CI branch source and a wait bound from athena.toml (with its schema field in crates/athena-data/src/config.rs, starting at 25 minutes); with none, NoCi and no wait. Then call resolve with the candidate's SHA: a green run is used and the local checks do not run; a red run fails the job naming the failing check; a missing, cancelled or timed-out run falls back to the local checks as today. Give the verifier the reuse statement (verify::reuse_statement).
> 
> Tests must not reach the network: fake gh::Runner, and install early_seam recording fakes for any test with an https github.com origin.
> 
> Acceptance:
> - A test through Athena::go shows no local check runs when the fake has a green push run for the candidate's SHA, and the check records name CI as their source.
> - Tests show a red run fails the job with no local run, and a missing run falls back to the local checks.
> - A test shows a repository with no GitHub origin does not wait.
> - athena.toml has the wait bound and its schema field.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

Job 20261031-230912-n9tm: in check_and_verify (run/mod.rs ~1264-1289), choose the CI source and wait together (with_ci_source) so a GitHub origin reads athena/ci/<job id> with a configurable wait (default 25 min), and no origin uses NoCi with no wait. Green reuses CI and skips local checks, red fails the job naming the check, missing/cancelled/timed-out falls back to local checks, and the verifier gets verify::reuse_statement. This follows 3rze (landed, pushes the CI branch) and replaces failed 17c3.

Assumptions:
- Code Survey: Code survey: read crates/athena-core/src/run/mod.rs (check_and_verify; checks stage builds route, SnapExec, then run_routed with routing::NoCi and Wait{polls:1} at ~1264-1289; a resumed job takes recorded_checks and skips this), crates/athena-core/src/routing.rs (run_routed takes &dyn CiSource and Wait and calls resolve; NoCi at line 107), crates/athena-core/src/ci_result.rs (resolve, Wait, CiSource, CiRecord, Outcome), crates/athena-core/src/verify.rs:60 (reuse_statement), crates/athena-core/src/gh/client.rs:706 (GhClient::ci_source(branch)), crates/athena-core/src/gh/early.rs:321-360 (ci_branch, push_ci_branch), crates/athena-data/src/config.rs:288-298 (Budget with deny_unknown_fields; check_timeout_s) and athena.toml:43. Existing tests in crates/athena-core/src/run/tests.rs (~2004-2100) and run_pin_tests.rs:424 show the early_seam::install pattern. The change adds no record or data file, so no check_record_file/AnyRecord impact. Behavior is measured by recording fake check executions through Athena::go and reading the resulting CheckRun notes.
- Job 20261031-230906-3rze (landed, passed): its work is kept as is. Its minor finding about test_support.rs is addressed by listing test_support.rs here as a possible deliverable; its nit (delete_ci_branch doing a remote_head lookup when nothing was pushed) is not in this job's scope and stays unfixed.
- Job 20261031-150034-17c3 (failed, no candidate): its push and delete parts are superseded by 3rze. Its plan-critique findings apply here as follows: every test with an https github.com origin also installs early_seam recording fakes (major); the green test's canned jobs reply includes a failed history job beside a successful 'fmt, clippy, tests' job so the aggregate verdict is exercised (nit); a resumed job with CI-sourced recorded checks gets no reuse statement, and this is stated in the CHANGELOG entry (minor); the ci.yml comment finding no longer applies because 3rze already edited ci.yml.
- Wait bound is a new field in the Budget table of crates/athena-data/src/config.rs, named ci_wait_s (seconds, 1500), with matching entry in athena.toml. Poll count and pause derive from it. Tests use a tiny bound or a fake clock/pause of zero so none sleep.
- Only gh::Runner is faked, so the production GhClient::ci_source query is what is exercised. Audience: Athena maintainers and the verifier, who check the behavior via cargo test.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new test through Athena::go with an https github.com origin, a fake gh::Runner replying a green push run for the candidate SHA (with a failed history job next to a successful 'fmt, clippy, tests' job), and early_seam recording fakes shows the recording local check executor is never called, and the recorded CheckRuns' notes name CI as their source.
2. [behavior] Run `cargo test -p athena-core`: a test with a red push run for the candidate SHA shows the job fails, the failure names the failing check, and no local check ran.
3. [behavior] Run `cargo test -p athena-core`: tests with no run, a cancelled run, and a timed-out run for the SHA each show the local checks run as before and the job proceeds.
4. [behavior] Run `cargo test -p athena-core`: a test with no GitHub origin (none or a local path) shows NoCi is used, the fake runner receives no call, and the wait is not applied (no sleeping, one read).
5. [behavior] Run `cargo test -p athena-core`: a test shows that when CI is green the verifier's input contains the text from verify::reuse_statement for the candidate SHA.
6. [content] run/mod.rs no longer passes routing::NoCi unconditionally at the run_routed call; a function with_ci_source picks the source and wait together (Grep `with_ci_source`), and the candidate SHA is what is passed to resolution.
7. [content] athena.toml (listed deliverable) sets the CI wait bound to 1500 seconds, and crates/athena-data/src/config.rs (listed deliverable) has the matching schema field; the config round-trip/load tests still pass.
8. [content] Every new test with an https github.com origin installs both the gh::Runner fake and early_seam recording fakes (Grep `early_seam::install` in the new tests); no test reaches the network.
9. [content] CHANGELOG.md has an entry for the change that also states a resumed job carries no reuse statement.
10. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)

## Out of scope

- Pushing or deleting the CI branch (job 3rze's landed work, kept unchanged, including its files ci.yml, gh/early.rs, gh/remote.rs, gh/tests.rs, gh/mod.rs).
- Changing .github/workflows/ci.yml or what the workflow checks.
- Changing the SHA rules in ci_result.rs or the local check implementation.
- Real network calls in tests, or giving the producer GitHub access.
- Reuse statement for resumed jobs whose checks were recorded earlier from CI.
- Avoiding the needless remote_head lookup in delete_ci_branch (3rze nit).

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is exercised through Athena::go tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that local checks do not run and records name CI.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the checks stage; push/delete left alone.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Plain doc comments and CHANGELOG entry.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fake origin uses example URLs only; no secrets.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The wait bound is a schema field in config, not prose.
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

Run `cargo test -p athena-core`: a new test through Athena::go with an https github.com origin, a fake gh::Runner replying a green push run for the candidate SHA (with a failed history job next to a successful 'fmt, clippy, tests' job), and early_seam recording fakes shows the recording local check executor is never called, and the recorded CheckRuns' notes name CI as their source.
Run `cargo test -p athena-core`: a test with a red push run for the candidate SHA shows the job fails, the failure names the failing check, and no local check ran.
Run `cargo test -p athena-core`: tests with no run, a cancelled run, and a timed-out run for the SHA each show the local checks run as before and the job proceeds.
Run `cargo test -p athena-core`: a test with no GitHub origin (none or a local path) shows NoCi is used, the fake runner receives no call, and the wait is not applied (no sleeping, one read).
Run `cargo test -p athena-core`: a test shows that when CI is green the verifier's input contains the text from verify::reuse_statement for the candidate SHA.
run/mod.rs no longer passes routing::NoCi unconditionally at the run_routed call; a function with_ci_source picks the source and wait together (Grep `with_ci_source`), and the candidate SHA is what is passed to resolution.
athena.toml (listed deliverable) sets the CI wait bound to 1500 seconds, and crates/athena-data/src/config.rs (listed deliverable) has the matching schema field; the config round-trip/load tests still pass.
Every new test with an https github.com origin installs both the gh::Runner fake and early_seam recording fakes (Grep `early_seam::install` in the new tests); no test reaches the network.
CHANGELOG.md has an entry for the change that also states a resumed job carries no reuse statement.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
