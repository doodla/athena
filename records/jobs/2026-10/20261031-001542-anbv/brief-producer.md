# Job 20261031-001542-anbv: Run job checks on GitHub CI via athena/ci/<job id> branch, with local

## Request (verbatim)

> Priority: urgent
> Let a job's checks run on GitHub instead of the local machine, and use that result
> 
> Today every job builds and tests its candidate locally: about 15 to 18 minutes and an 11 to 18 GB build directory, which keeps filling the disk (d103 and sz3z failed on low disk on 2026-10-30). GitHub can run the same checks for free. Most of the code already exists but is not wired in: crates/athena-core/src/ci_result.rs (`resolve`, the SHA rules) and crates/athena-core/src/routing.rs (`run_routed`), see docs/design/ci-routing.md section 10 items 4 and 5, "Not done".
> 
> Wire it in:
> 1. Right after the candidate commit is admitted, push it to a branch named `athena/ci/<job id>` on the remote, and make ci.yml run on pushes to `athena/ci/**`.
> 2. In the checks stage, call `resolve` with the candidate SHA. Wait for the CI run up to a bound in athena.toml (start at 25 minutes). A green run for that exact SHA is used and the local checks do not run. A red run fails the job, with the failing check's name in the reason. A missing, cancelled or timed-out run falls back to the local checks, as today.
> 3. Give the verifier the reuse statement (`verify::reuse_statement`), so it knows where the check results came from.
> 4. Delete the `athena/ci/<job id>` branch when the job closes, whatever the outcome.
> 5. When no remote or no GitHub is configured, nothing changes: the checks run locally.
> 
> The producer cannot reach GitHub. Test with the injected platform source that ci_result.rs already uses.
> 
> Acceptance:
> - A test shows that the checks stage runs no local check when the injected source has a green result for the candidate's exact SHA, and the job's check records name CI as the source.
> - Tests show that a red result fails the job without a local run, that a result for another SHA is ignored, and that a missing result falls back to the local run.
> - A test shows the CI branch name is pushed after admission and deleted at close, through the injected git and platform sources.
> - athena.toml has the wait bound; ci.yml runs on `athena/ci/**` pushes.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

Wire the existing ci_result::resolve and routing::run_routed into the job run. Push the admitted candidate to athena/ci/<job id>, wait up to a configured bound for CI on that exact SHA, and use green or red results. Fall back to local checks otherwise, pass verify::reuse_statement to the verifier, and delete the branch at close. The goal is to cut local build time and disk use; the audience is the Athena maintainer.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs (run_routed call near line 1265, where check_runs is produced), crates/athena-core/src/routing.rs (run_routed, CiSource stub with no push yet), crates/athena-core/src/ci_result.rs (resolve, Wait, Source, CiRecord), crates/athena-core/src/verify.rs (reuse_statement), crates/athena-core/src/gh/client.rs (GhClient implements CiSource), crates/athena-data/src/config.rs (athena.toml config structs), crates/athena-git/src/lib.rs (git operations). Order: admission, then push of the CI branch, then the checks stage resolves CI, then the verifier. No new record or data file is added, so the records validation allow-list (check_record_file, AnyRecord) is not touched. Source is measured directly through the injected fakes counting local check invocations.
- The wait bound is a new key in athena.toml with a default of 25 minutes (1500 s), read through crates/athena-data/src/config.rs.
- Branch deletion runs on every close path (accepted, rejected, failed) and a deletion failure is logged, not fatal.
- Push failure, no remote, or no GitHub configured means the local checks run as today.
- The exact name of the git push and delete methods is chosen by the producer from crates/athena-git/src/lib.rs, adding them if missing, behind an injectable trait for tests.

## Acceptance criteria

1. [behavior] cargo test passes, including a new test in crates/athena-core where the injected CI source returns a green result for the candidate's exact SHA: the local check runner is called zero times and the check records name CI as the source.
2. [behavior] cargo test includes new tests showing a red CI result fails the job with the failing check's name in the reason and no local run, a result for a different SHA is ignored and the local checks run, and a missing, cancelled or timed-out result falls back to the local checks.
3. [behavior] cargo test includes a test where the injected git and platform sources record that athena/ci/<job id> is pushed after admission and deleted at close, for accepted and for failed outcomes.
4. [behavior] cargo test includes a test showing that with no remote or no GitHub configured, no push happens and the checks run locally.
5. [content] The verifier prompt input built in crates/athena-core/src/run/mod.rs includes the text from verify::reuse_statement when the check record's source is CI (Grep for the call).
6. [content] athena.toml has a CI wait bound key set to 25 minutes, and the config struct in crates/athena-data/src/config.rs parses it.
7. [content] .github/workflows/ci.yml triggers on push to branches matching athena/ci/** (Grep the on: block).
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
9. [content] CHANGELOG.md has an entry describing CI-backed checks, and docs/design/ci-routing.md section 10 items 4 and 5 are updated from Not done.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/ci-routing.md (modify)

## Out of scope

- Changing what the CI workflow checks beyond its trigger.
- Giving the producer access to GitHub, or running real network calls in tests.
- Changing the local check implementation or the SHA rules in ci_result.rs beyond what wiring needs.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus config and workflow files.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus clippy and fmt clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test, clippy and fmt.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The wait bound is configuration in athena.toml and the SHA rule is enforced in code.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check local checks not running and the branch pushed and deleted, not wiring existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is no local check run on a green exact-SHA CI result.
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

cargo test passes, including a new test in crates/athena-core where the injected CI source returns a green result for the candidate's exact SHA: the local check runner is called zero times and the check records name CI as the source.
cargo test includes new tests showing a red CI result fails the job with the failing check's name in the reason and no local run, a result for a different SHA is ignored and the local checks run, and a missing, cancelled or timed-out result falls back to the local checks.
cargo test includes a test where the injected git and platform sources record that athena/ci/<job id> is pushed after admission and deleted at close, for accepted and for failed outcomes.
cargo test includes a test showing that with no remote or no GitHub configured, no push happens and the checks run locally.
The verifier prompt input built in crates/athena-core/src/run/mod.rs includes the text from verify::reuse_statement when the check record's source is CI (Grep for the call).
athena.toml has a CI wait bound key set to 25 minutes, and the config struct in crates/athena-data/src/config.rs parses it.
.github/workflows/ci.yml triggers on push to branches matching athena/ci/** (Grep the on: block).
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
CHANGELOG.md has an entry describing CI-backed checks, and docs/design/ci-routing.md section 10 items 4 and 5 are updated from Not done.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
