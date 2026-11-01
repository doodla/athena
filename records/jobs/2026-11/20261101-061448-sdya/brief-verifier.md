# Job 20261101-061448-sdya: verify "Use GitHub CI result for the candidate in checks stage (retry, no"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Test ci_reuse_green_skips_local_checks in run/tests.rs drives Athena::go through the helper ci_reuse_run with origin https://github.com/owner/repo and the CiSteps capability (fmt, clippy, test). A CiReplies Runner fake replies with a green push run for the head_sha and branch it reads from the request path, as the server filter would, and with a green jobs reply. Asserts the recording check executor was called zero times, the runner fake was called at least once, the sleeper recorded no sleep, and every recorded check note names the CI run address (source: CI) and the candidate SHA. Asserts the job reaches Landed. Run: `cargo test -p athena-core ci_reuse_green`. Observes: Zero local check calls, runner calls above zero, and each recorded check note naming the CI run, with the job proceeding to the verifier and landing.
  Fails without the behavior: Without with_ci_source the call still passes NoCi, so the runner fake is never asked (the runner-call assertion fails), the recording executor is called for fmt, clippy and test, and the notes do not name CI.
  Test seam: Production entry point Athena::go (reaches check_and_verify, route, run_routed_with_record and with_ci_source). Guards: origin is an https github.com URL, not a nested run, and the capability covers fmt/clippy/test so route.reuse is true. Fakes: gh::early_seam::install(fake, fake) (exists, gh/mod.rs), check_seam::install with a recording closure (exists, run/mod.rs), ci_seam::install(runner, sleeper) with the CiReplies gh::Runner and a recording sleeper (ci_seam added in plan step 3, fakes and CiSteps in step 8). The fakes supply JSON and record calls only; the SHA match, run choice and reuse stay in production resolve.
- Criterion 2: Test ci_reuse_red_fails_without_local_checks: the CiReplies fake returns a failed push run whose jobs reply has clippy failing and the aggregate job failing. Same helper and fakes through Athena::go. Asserts the state is not Landed, the checks output line and recorded check records name clippy as failed, the recording executor was called zero times, the runner fake was called, and the sleeper recorded no sleep. Run: `cargo test -p athena-core ci_reuse_red`. Observes: Job not landed; failing check clippy named; zero local check calls; runner called.
  Fails without the behavior: Without the CI read, local checks run and pass in the recording fake, so the job lands and the executor count is non-zero.
  Test seam: Same as criterion 1: Athena::go, early_seam (exists), check_seam recording fake (exists), ci_seam CiReplies red JSON and CiSteps capability (steps 3 and 8).
- Criterion 3: Test ci_reuse_falls_back_to_local_checks, one go run per scenario through ci_reuse_run: empty workflow_runs, cancelled aggregate job, conclusion timed_out, and a run in_progress on every read. Each asserts the runner fake was called at least once, the recording executor ran fmt, clippy and test once each, and the job lands. For the never-finishing run it asserts 76 reads and 75 sleeps of 20 s on the fakes (the 1500 s bound, no real sleeping). Run: `cargo test -p athena-core ci_reuse_falls_back`. Observes: Local checks run as before in all four cases and the job proceeds; the wait bound shows through the fake sleeper.
  Fails without the behavior: A fallback bug (a non-green run treated as a pass, or the job failed when no result exists) changes the executor counts and the state. Without a CI read the cases cannot be told from NoCi, so the runner-call count is asserted too.
  Test seam: Athena::go; early_seam (exists), check_seam recording fake (exists), ci_seam runner and sleeper fakes and CiSteps (steps 3 and 8). The 75 and 76 come from production routing::Paced and ci_polls (step 4); the fake only returns JSON and records sleeps.
- Criterion 4: Test ci_reuse_no_github_origin_uses_no_ci loops the origin forms of the existing no_github_origin_pushes_no_ci_branch test (none, local path, lookalike path containing github.com, https://gitlab.example.com/owner/repo, https://example.com/github.com/owner/repo) with the same CiSteps capability and ci_reuse_run. Each go run installs a counting runner fake and a counting sleeper fake. Asserts runner calls 0, sleeper calls 0, the recording executor ran fmt, clippy and test once each, and the job lands. A routing unit test checks NoCi with a one-poll wait does one read and no sleep. Run: `cargo test -p athena-core ci_reuse_no_github_origin`. Observes: Runner calls 0 and sleeper calls 0 for every form; local checks run once; the job lands.
  Fails without the behavior: If the CI source were chosen without checking the origin, then with route.reuse on (CiSteps) the fake runner would record a call, or the sleeper would record 75 sleeps, for the none, local-path, lookalike and other-host origins. The test fails there, unlike under the Docs capability where the CI path is never entered.
  Test seam: Athena::go; early_seam (exists); check_seam recording fake (exists); ci_seam counting fakes and CiSteps (steps 3 and 8). It also covers https_origin_repository (step 3).
- Criterion 5: Test ci_reuse_statement_reaches_the_verifier_input: the green setup with an agent script wrapping counting(...) as the existing tests in run/tests.rs do, to capture spec.user_prompt when spec.role is Verifier. Asserts the prompt contains verify::reuse_statement(&record, candidate), where the record is built from the same JSON the CiReplies fake served (parsed by production ci_result) and the candidate SHA is read from the CI branch push recorded on the EarlyFake. Run: `cargo test -p athena-core ci_reuse_statement`. Observes: The verifier prompt contains the statement naming the CI run address, the candidate SHA and the step results.
  Fails without the behavior: Without the Checked.reuse entry in verification.rs, the verifier prompt has no such text and the contains assertion fails.
  Test seam: Athena::go; existing athena_with-style construction and counting script (exist); the fakes of criterion 1 (steps 3 and 8). The expected string comes from production verify::reuse_statement (exists), not from copied logic in a fake.
- Criterion 6: Read the diff of run/mod.rs and Grep with_ci_source over crates/. Check that the run_routed call no longer takes a literal NoCi, that with_ci_source is defined once and returns NoCi or the GitHub source together with the Wait, and that the candidate &c is still passed as the SHA. Run: `git diff -- crates/athena-core/src/run/mod.rs`. Observes: with_ci_source is defined once and called from check_and_verify; the only NoCi mention is inside it for the no-origin and nested branch; the SHA argument is the candidate commit.
- Criterion 7: Config tests: an assertion that Config loaded from the repository athena.toml has budget.ci_wait_s == 1500, and that a budget table without the field loads with 1500. Read the diff of athena.toml and config.rs. Run: `cargo test -p athena-data config`. Observes: athena.toml has ci_wait_s = 1500 under [budget]; Budget has pub ci_wait_s: u64 with the serde default; existing config load tests pass.
- Criterion 8: Grep early_seam::install and ci_seam::install in run/tests.rs. Every new github-origin test goes through the single helper ci_reuse_run, which installs early_seam, ci_seam (Runner and sleeper fakes) and the recording check_seam before go and keeps the Guards alive through go. No test installs GhCli or Command; ci_seam is #[cfg(test)] only. Run: `git diff -- crates/athena-core/src/run/tests.rs`. Observes: Every new github-origin test installs the early fakes, the runner and sleeper fakes and the check fake before go. With no runner installed the path uses NoCi, so no test reaches the network.
- Criterion 9: Scan the added lines of the whole diff for an at-sign and user-at-host forms: gh/client.rs, athena.toml, CHANGELOG.md, tests. New URLs are https://github.com/owner/repo; the userinfo split is '\u{40}'. The producer runs the same scan before finishing. athena.toml email_allow is unchanged. Run: `git diff`. Observes: No added line contains an at-sign form; email_allow is unchanged.
- Criterion 10: Read the new Unreleased CHANGELOG entry. Run: `git diff -- CHANGELOG.md`. Observes: The entry says the checks stage reads the push run on athena/ci/<job id>; green skips local checks; red fails; missing, cancelled, timed-out or unfinished runs fall back; [budget] ci_wait_s is 1500 polled every 20 s; the reuse statement reaches the verifier; a resumed job carries no reuse statement. No at-sign.
- Criterion 11: Run the three checks Athena runs on the result: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings, and cargo test --locked. The test run includes every new test and every existing github-origin test (ci_run in run/tests.rs, early_run in run_pin_tests.rs, config load tests, routing unit tests). Run: `cargo test --locked`. Observes: fmt reports no diff, clippy reports no warnings (no unused Paced or ci_seam item), and every test passes. Existing github-origin tests with no ci_seam installed see NoCi, so none sleeps 1500 s or calls the real gh.
  Fails without the behavior: If the behavior were missing or broken, cargo test --locked fails: the new green, red, fallback, no-origin and verifier-statement tests fail, the config load tests fail if ci_wait_s is missing from athena.toml or the schema, and existing github-origin tests fail or hang if with_ci_source reached the real gh or a real 1500 s wait when no ci_seam is installed. cargo fmt --check fails on unformatted new code and clippy -D warnings fails on warnings in it.
  Test seam: The cargo commands are the entry points; they run the test suite, whose tests call production Athena::go. Seams: gh::early_seam::install and check_seam::install (both exist), and ci_seam::install with the CiReplies runner and recording sleeper fakes (ci_seam added in step 3, fakes in step 8). With no ci_seam installed, test builds use NoCi (step 5). The fakes supply JSON replies and record calls only; the decision logic is production code.

Planned files:

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/verify.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 61f7d1da1491 → candidate 7060753d0ca9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  16 ++
 athena.toml                                |   3 +
 crates/athena-core/src/gh/client.rs        |  42 +++
 crates/athena-core/src/routing.rs          | 179 +++++++++++-
 crates/athena-core/src/run/mod.rs          | 140 ++++++++-
 crates/athena-core/src/run/tests.rs        | 445 ++++++++++++++++++++++++++++-
 crates/athena-core/src/run/verification.rs |  14 +
 crates/athena-data/src/config.rs           |  29 ++
 8 files changed, 835 insertions(+), 33 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1896 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
