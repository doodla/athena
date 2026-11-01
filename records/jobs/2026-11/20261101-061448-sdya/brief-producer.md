# Job 20261101-061448-sdya: Use GitHub CI result for the candidate in checks stage (retry, no

## Request (verbatim)

> try again on 20261031-230912-n9tm. Same change; the only fault was an email address in crates/athena-core/src/gh/client.rs, which admission refuses. Write no email address anywhere: not a git remote of the form user at host, not a bot or author address. Write GitHub remotes as https://github.com/owner/repo.git or ssh://github.com/owner/repo.git, as gh/tests.rs already does. The only allowed addresses are those in athena.toml [personal_data] email_allow (no-reply addresses and the example domains).
> Priority: urgent

## Reading (accepted by the user)

Retry of job 20261031-230912-n9tm: in the checks stage, pick the CI source and its wait together, so a GitHub origin reads the athena/ci/<job id> push run and a green run skips the local checks. The earlier fault was an email address in gh/client.rs. This time no email-like text is written anywhere.

Assumptions:
- Code survey: I read crates/athena-core/src/run/mod.rs (check_and_verify passes routing::NoCi to run_routed near line 1283), crates/athena-core/src/routing.rs (NoCi, CiSource), crates/athena-core/src/verify.rs (reuse_statement at line 60), crates/athena-data/src/config.rs ([personal_data] email_allow at line 456, [budget] schema) and athena.toml (email_allow at line 181). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. The CI wait is measured through a fake gh::Runner and a fake sleeper, not a stand-in.
- Job 20261031-230912-n9tm is addressed this way: its single finding was an email-like literal in crates/athena-core/src/gh/client.rs. The retry drops that literal, so the file takes no identity text, or none that contains an at-sign form. All its other file changes are kept as deliverables: CHANGELOG.md, athena.toml, routing.rs, run/mod.rs, run/tests.rs, run/verification.rs, test_support.rs and config.rs. verify.rs stays listed as a possible change.
- Goal and audience: the Athena maintainer wants the checks stage to reuse GitHub's CI result for the candidate commit, so local checks are skipped when CI is green. Reviewers judge it by the tests and by admission accepting the diff.
- No new text anywhere may match an email: no user-at-host remotes and no bot or author identities. GitHub remotes use https://github.com/owner/repo.git or ssh://github.com/owner/repo.git, as gh/tests.rs does. The only addresses allowed are those in athena.toml [personal_data] email_allow.
- The wait bound is a new [budget] ci_wait_s field, default 1500 seconds, with its schema field in config.rs. The wait is one read every 20 seconds. No GitHub origin, or a nested run, uses NoCi with one read and no wait.
- Existing code: this job reuses crates/athena-core/src/routing.rs, crates/athena-core/src/ci_result.rs, crates/athena-core/src/verify.rs and crates/athena-core/src/gh/ (for the Runner and the CI branch source). It does not create them.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new test through Athena::go with an https github.com origin, a fake gh::Runner replying a green push run for the candidate SHA, and early_seam recording fakes shows the recording local check executor is never called, and the recorded check notes name CI as their source.
2. [behavior] Run `cargo test -p athena-core`: a test with a red push run for the candidate SHA shows the job fails, the failure names the failing check, and no local check ran.
3. [behavior] Run `cargo test -p athena-core`: tests with a missing run, a cancelled run and a timed-out run each show the local checks run as before and the job proceeds.
4. [behavior] Run `cargo test -p athena-core`: a test with no GitHub origin (none, or a local path) shows NoCi is used, the fake runner gets no call, and the wait does not apply (no sleeping, one read).
5. [behavior] Run `cargo test -p athena-core`: a test shows that when CI is green the verifier's input contains the text of verify::reuse_statement for the candidate SHA.
6. [content] run/mod.rs no longer passes routing::NoCi unconditionally at the run_routed call; Grep `with_ci_source` shows a function choosing the source and wait together, and the candidate SHA is passed to resolution.
7. [content] athena.toml (a listed deliverable) sets the CI wait bound to 1500 seconds, and crates/athena-data/src/config.rs has the matching schema field; the config load tests still pass.
8. [content] Every new test with an https github.com origin installs both the gh::Runner fake and early_seam recording fakes (Grep `early_seam::install` in the new tests); no test reaches the network.
9. [content] No added line in the diff contains an email address or user-at-host remote outside the athena.toml [personal_data] email_allow list; Grep the diff for an at-sign form, including crates/athena-core/src/gh/client.rs, and it finds none.
10. [content] CHANGELOG.md has an entry for the change that also states a resumed job carries no reuse statement.
11. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

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
- crates/athena-core/src/run/verification.rs (modify)

## Out of scope

- Pushing or deleting the CI branch (job 3rze's landed work, kept unchanged, including ci.yml, gh/early.rs, gh/remote.rs, gh/tests.rs and gh/mod.rs).
- Changing .github/workflows/ci.yml or what the workflow checks.
- Changing the SHA rules in ci_result.rs or the local check implementation.
- Real network calls in tests, or giving the producer GitHub access.
- A reuse statement for resumed jobs whose checks were recorded earlier from CI.
- Avoiding the needless remote_head lookup in delete_ci_branch (3rze nit).

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust and TOML.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy -D warnings and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the green, red, missing, cancelled, timed-out and no-origin cases.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email or identity literal is written anywhere.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the checks stage uses the CI result, shown by the test through Athena::go.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The wait bound is set in athena.toml with a schema field.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The email literal is removed and the diff is scanned for any other.
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

23 more qualifying classes are left out.

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

Run `cargo test -p athena-core`: a new test through Athena::go with an https github.com origin, a fake gh::Runner replying a green push run for the candidate SHA, and early_seam recording fakes shows the recording local check executor is never called, and the recorded check notes name CI as their source.
Run `cargo test -p athena-core`: a test with a red push run for the candidate SHA shows the job fails, the failure names the failing check, and no local check ran.
Run `cargo test -p athena-core`: tests with a missing run, a cancelled run and a timed-out run each show the local checks run as before and the job proceeds.
Run `cargo test -p athena-core`: a test with no GitHub origin (none, or a local path) shows NoCi is used, the fake runner gets no call, and the wait does not apply (no sleeping, one read).
Run `cargo test -p athena-core`: a test shows that when CI is green the verifier's input contains the text of verify::reuse_statement for the candidate SHA.
run/mod.rs no longer passes routing::NoCi unconditionally at the run_routed call; Grep `with_ci_source` shows a function choosing the source and wait together, and the candidate SHA is passed to resolution.
athena.toml (a listed deliverable) sets the CI wait bound to 1500 seconds, and crates/athena-data/src/config.rs has the matching schema field; the config load tests still pass.
Every new test with an https github.com origin installs both the gh::Runner fake and early_seam recording fakes (Grep `early_seam::install` in the new tests); no test reaches the network.
No added line in the diff contains an email address or user-at-host remote outside the athena.toml [personal_data] email_allow list; Grep the diff for an at-sign form, including crates/athena-core/src/gh/client.rs, and it finds none.
CHANGELOG.md has an entry for the change that also states a resumed job carries no reuse statement.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
