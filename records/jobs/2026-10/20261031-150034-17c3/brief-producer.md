# Job 20261031-150034-17c3: Push candidates to athena/ci/<job id> and use GitHub's result in checks

## Request (verbatim)

> Priority: urgent
> Depends on 20261031-142738-g437
> Second part of 20261031-001542-anbv: push each candidate to athena/ci/<job id> and use GitHub's result in the checks stage
> 
> After 20261031-142738-g437, reading a CI result returns only the push run of branch athena/ci/<job id> for an exact SHA. This part wires it in, as anbv asked:
> 1. Right after the candidate commit is admitted, push it to athena/ci/<job id>, and make .github/workflows/ci.yml run on pushes to athena/ci/**.
> 2. In the checks stage, call `resolve` (crates/athena-core/src/ci_result.rs) with the candidate SHA instead of the `NoCi` source passed at crates/athena-core/src/run/mod.rs (about line 1269). Wait up to a bound in athena.toml (with its schema field), starting at 25 minutes. A green run for that SHA is used and the local checks do not run; a red run fails the job naming the failing check; a missing, cancelled or timed-out run falls back to the local checks, as today.
> 3. Give the verifier the reuse statement (`verify::reuse_statement`).
> 4. Delete athena/ci/<job id> when the job closes, whatever the outcome.
> 5. With no GitHub remote configured, nothing changes.
> Test through the injected platform and git sources that ci_result.rs and gh already use; the producer cannot reach GitHub.
> 
> Acceptance:
> - A test through `Athena::go` shows no local check runs when the injected source has a green push run for the candidate's SHA, and the check records name CI as their source.
> - Tests show a red run fails the job with no local run, and a missing run falls back to the local checks.
> - A test shows athena/ci/<job id> is pushed after admission and deleted at close.
> - athena.toml has the wait bound with its schema field; ci.yml runs on athena/ci/** pushes.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

Wire job 20261031-142738-g437's push-run-only CI source into the run: push the admitted candidate to athena/ci/<job id>, call `resolve` instead of `NoCi` in the checks stage with a wait bound from athena.toml, pass the reuse statement to the verifier, and delete the branch at close. The audience is Athena's maintainers, who want less local build time and disk use.

Assumptions:
- Job 20261031-142738-g437 landed and passed: gh/client.rs and gh/tests.rs are kept as they are, not modified here. Its two findings are handled: CHANGELOG.md is changed here (its nit), and the hard-coded job names in aggregate_verdict (its minor finding) are left alone because the failure mode falls back safely to local checks.
- Job 20261031-001542-anbv failed with no candidate, so nothing is kept or dropped. Its round-3 findings are addressed: g437 restricts the source to the push run on the job's branch, so a pull_request run for the same SHA is never used. The reread path calls the inner dispatch so the branch delete runs once. The extra files it flagged (run/ci.rs, run/ci_tests.rs, gh/early.rs, run/verification.rs, run/tests.rs) are listed here or excluded.
- Code survey: read crates/athena-core/src/run/mod.rs (NoCi passed near line 1269 in the checks stage), crates/athena-core/src/routing.rs (CiSource, NoCi, run_routed), and the layout of crates/athena-core/src/run/ (candidate.rs, checks.rs, close.rs, tests.rs, verification.rs exist; ci.rs and ci_tests.rs do not). No records or data files are added under crates/, so check_record_file and AnyRecord are unaffected. Behavior is measured by Athena::go tests with injected platform and git sources, counting local check invocations.
- The wait bound is a new field in the athena.toml config schema in crates/athena-data/src/config.rs, default 1500 s (25 minutes).
- The CI source and push/delete helpers go in a new run/ci.rs, with tests in a new run/ci_tests.rs. athena-git gains a delete-remote-branch operation only if it has none.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: an Athena::go test with a green push run for the candidate's SHA in the injected source runs zero local checks, and the check records name CI as their source.
2. [behavior] Run `cargo test -p athena-core`: a red run fails the job with the failing check's name in the reason and no local run; a missing run, a cancelled run, a timed-out wait and a pull_request-only run each fall back to the local checks.
3. [behavior] Run `cargo test -p athena-core`: a test through injected git and platform sources shows athena/ci/<job id> pushed after admission and deleted at close for accepted, failed and rejected jobs, and deleted once on the reread path.
4. [behavior] Run `cargo test -p athena-core`: with no GitHub remote configured, no push or delete happens and checks run locally as before.
5. [content] Grep shows `NoCi` is no longer passed in crates/athena-core/src/run/mod.rs, and the verifier prompt input includes `verify::reuse_statement` when CI results are used.
6. [content] athena.toml has a CI wait-bound key set to 25 minutes, and its schema field is in crates/athena-data/src/config.rs.
7. [content] .github/workflows/ci.yml triggers on push to `athena/ci/**` and still runs the same checks.
8. [content] CHANGELOG.md has an entry for this change, and docs/design/ci-routing.md section 10 items 4 and 5 are updated to say they are done.
9. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/ci.rs (create)
- crates/athena-core/src/run/ci_tests.rs (create)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/ci-routing.md (modify)

## Out of scope

- Changing what the CI workflow checks beyond its trigger.
- Giving the producer access to GitHub, or making real network calls in tests.
- Changing crates/athena-core/src/gh/client.rs, crates/athena-core/src/gh/tests.rs, or the SHA rules in crates/athena-core/src/ci_result.rs: job 20261031-142738-g437's work is kept as landed.
- Reading CI job names from ci.yml (g437 minor finding): the failure mode is a safe fallback to local checks.
- Changing the local check implementation.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust, plus TOML and YAML config.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; fmt, clippy -D warnings and cargo test must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test runs through Athena::go.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The wait bound is a config field with a schema entry.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that local checks do or do not run, not that wiring exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused wiring change; the CI source and the egress parsing stay as landed.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the change.
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

Run `cargo test -p athena-core`: an Athena::go test with a green push run for the candidate's SHA in the injected source runs zero local checks, and the check records name CI as their source.
Run `cargo test -p athena-core`: a red run fails the job with the failing check's name in the reason and no local run; a missing run, a cancelled run, a timed-out wait and a pull_request-only run each fall back to the local checks.
Run `cargo test -p athena-core`: a test through injected git and platform sources shows athena/ci/<job id> pushed after admission and deleted at close for accepted, failed and rejected jobs, and deleted once on the reread path.
Run `cargo test -p athena-core`: with no GitHub remote configured, no push or delete happens and checks run locally as before.
Grep shows `NoCi` is no longer passed in crates/athena-core/src/run/mod.rs, and the verifier prompt input includes `verify::reuse_statement` when CI results are used.
athena.toml has a CI wait-bound key set to 25 minutes, and its schema field is in crates/athena-data/src/config.rs.
.github/workflows/ci.yml triggers on push to `athena/ci/**` and still runs the same checks.
CHANGELOG.md has an entry for this change, and docs/design/ci-routing.md section 10 items 4 and 5 are updated to say they are done.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
