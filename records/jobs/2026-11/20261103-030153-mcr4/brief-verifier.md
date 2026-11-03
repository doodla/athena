# Job 20261103-030153-mcr4: verify "Check landed athena.toml against the installed build, in every drain"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: A job can add a config field together with the code that reads it. The build that lands it, and the
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: self-landing-config-field-unknown-to-installed-build (20261025-010251-8gqz)
> Cause: A job can add a config field together with the code that reads it. The build that lands it, and the drain that keeps running afterward, is the older one, which does not know the field. Nothing checks that the installed binary accepts the la...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261101-085638-8zkm (job 20261101-061448-sdya); prevention it proposed: Two things together. (1) Before landing a job whose diff adds or changes an athena.toml field, parse the post-landing athena.toml with the currently installed binary. If it rejects the file, hold the landing, or land it and then rebuild and...
> Introspection 20261103-030136-jnn4 (job 20261101-085726-z7dg); prevention it proposed: Two code changes together. First, before dispatching or landing a job that adds a config field, check that the running build accepts the key. If not, land the job on a path that rebuilds the installed build first, or hold the job with a cle...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a job can land a new athena.toml field while the installed build (which keeps running) cannot parse it, and nothing checks that at the landing. Job 20261101-085726-z7dg added a drain-only post-landing check; this change moves the check into the landing outcome and fixes that job's open finding so the whole class ends the run cleanly with a reinstall note.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (State::land at ~1643, Steps::config_after_landing at ~528 and ~3576, Stop::NeedsReinstall at ~954, finish/rate at ~2338, set_stop keeps only the first reason), crates/athena-core/src/capability.rs (JobConfig::at parses athena.toml at a commit), crates/athena-cli/tests/drain.rs (real-steps tests). The entry is the drain's land step under the landing lock; the config is read with the running build's schema directly via JobConfig::at at the new head, not a stand-in. No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not involved.
- Job 20261101-085726-z7dg (landed, pass): its Stop::NeedsReinstall and post-landing config read are KEPT. Its findings: (1) set_stop keeps only the first reason, so under drain_concurrent an earlier stop hides the reinstall note and rating still reads the unreadable config - FIXED by keying skip-rating and the note on State.unreadable_config.is_some(), not on the stop reason; (2) any parse error counts as 'newer schema' - kept, the error text stays in the note; (3) the real-steps test not shown to fail on base - addressed by a test that fails without the change.
- Job 20261101-061448-sdya (landed, pass): its work is kept unchanged. Its two minor findings (duplicate origin parser in gh/client.rs vs gh/early.rs; full CI wait when the CI-branch push fails) are unrelated to this failure class and are not addressed; its changed files are excluded in out_of_scope.
- Goal and audience: the operator running athena drain; a landed job that adds a config key must never be recorded failed, and the run must end with a reinstall note. Also the failure of the same class when a non-first worker or a non-drain path reads the landed config.
- Default applied: the check is enforced by mechanism (a test), not prose; no email addresses or personal data are written anywhere.

## Acceptance criteria

1. [content] The CHANGELOG.md entry and code comments name the root cause: the landing never checks that the running build can read the config it lands, and the fix is where the landed head is read, not at one job.
2. [behavior] A test in crates/athena-cli/tests/drain.rs or drain.rs lands a job whose athena.toml diff adds a key the running schema rejects, while another worker has already set a stop reason: the job is recorded landed (not failed), rating is skipped, the reinstall note with the parse error is reported, and the run does not end as 'repeated failure'. It fails on the base tree and passes with the change. Run: cargo test -p athena-cli drain.
3. [behavior] A second test covers the class, not one job: each Steps/lander path that reports Landed (stub and real steps) yields the same reinstall outcome for any unreadable landed config, and a readable landed config still continues normally. Run: cargo test.
4. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test all pass.
5. [content] Files from job 20261101-061448-sdya (CI wait, gh/client.rs, routing.rs, config.rs, athena.toml) are unchanged in the diff (git diff).

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- All work of job 20261101-061448-sdya: crates/athena-core/src/gh/client.rs, routing.rs, run/mod.rs, run/tests.rs, run/verification.rs, crates/athena-data/src/config.rs, athena.toml (kept, not changed)
- Making the config parser tolerant of unknown keys
- Automatically reinstalling or rebuilding the binary
- Changes to athena.toml, defaults.toml, admission rules, or .github/workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the run ending cleanly with job landed, via tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy, test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix keyed on the unreadable-config state, not the stop reason or one job.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email addresses or personal data written.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff and Grep it for 'never checks' / 'running build'. Run: `git diff -- CHANGELOG.md crates/athena-cli/src/drain.rs crates/athena-core/src/capability.rs`. Observes: A CHANGELOG [Unreleased] Fixed bullet and comments in drain.rs (module doc ~117, Stop::NeedsReinstall docs, State.unreadable_config doc) and capability.rs (unreadable_at doc) name the root cause. The landing never checks that the running build can read the config it lands, and the fix is where the landed head is read, keyed on the unreadable-config state and not on the first stop reason.
- Criterion 2: New test `a_landing_with_an_unreadable_config_after_another_workers_stop_is_landed_and_ends_with_reinstall` in mod parallel. Entry point: public `drain::drain_concurrent(&Shared, limits, &Quiet, 2)`, with a plan of two independent items and bound 2. Seam: the existing `Shared` stub (tests/drain.rs:6157) gets two new fields in plan step 5. `unreadable_for: Vec<&str>` makes the stub's `Steps::config_after_landing` return Some(parse-error text) after those jobs land; the production check, Drain::land, is not in the stub. `wait_for: Vec<(&str,&str)>` makes a job's run spin up to 5 s until the log holds that entry, then sleep run_ms. The stub's `rate()` also pushes 'rate' to the log. Item `bad` is rejected twice, so worker 2 stores RepeatedFailure. Item `new` waits for 'end bad-retry' and then lands with the unreadable config. Asserts: report.stop == NeedsReinstall and != RepeatedFailure; `new` has landing 'landed' and outcome passed; the log has no 'rate'; a note contains the stub's parse error, 'Reinstall Athena' and 'athena drain'; failed_item is None and a note names `bad`; the text lacks 'repeated failure'. Run: `cargo test --locked -p athena-cli --test drain a_landing_with_an_unreadable_config_after_another_workers_stop`. Observes: Passes with the change. On the base tree report.stop is RepeatedFailure, 'rate' is in the log and no reinstall note exists.
  Fails without the behavior: Without keying on State.unreadable_config, set_stop(NeedsReinstall) is dropped because RepeatedFailure is already stored. finish rates the unreadable config and the report ends as repeated failure with no reinstall note.
  Test seam: Entry `drain::drain_concurrent` exists (drain.rs ~2653). `Shared` exists in tests/drain.rs mod parallel; the unreadable_for and wait_for fields and the 'rate' log line are added by plan step 5. The logic under test (Drain::land, finish, report assembly) stays in production code.
- Criterion 3: Two tests in tests/drain.rs plus one in main.rs. (A) `every_landing_path_ends_in_reinstall_for_any_unreadable_config` is a table with two dimensions. Dimension 1 is the landing path: serial `Stub` via `drain::drain`, `Shared` at bounds 1 and 2 via `drain_concurrent`, and `RealSteps` with `ConfigLander` (exists ~2641) committing a real athena.toml, using the existing Fixture. Dimension 2 is the stop already stored before the unreadable landing: none, RepeatedFailure (Shared `failing` plus `wait_for`), Requested (Shared `stop_at`), and Rebuilt (a Shared job whose land reports a build path as changed, and whose rebuild returns Ok, followed by a later unreadable landing with no further rebuild). Rows that a path cannot seed are skipped explicitly in the table, not silently. Unreadable causes: unknown top-level key, unknown section, wrong value type. One shared helper asserts stop NeedsReinstall, the job landed and passed, no 'rate', the note carries the error text and 'Reinstall Athena', no failure counted, and no restart for the Rebuilt row. A readable-config control per path must be rated and must not end in NeedsReinstall. One further row has a rebuild run after the unreadable landing: the flag clears and the stop is Rebuilt. (B) `unreadable_at_reports_what_the_running_schema_rejects` calls `JobConfig::unreadable_at` on a Fixture repo for each bad config (Some with error text) and a readable one (None). (C) a main.rs unit test of `reinstall_note_after_landing(git, merged)` using a tempdir repo like the checkpoint test. The bad athena.toml is on a commit that is not HEAD. Passing that commit gives Some containing 'Reinstall Athena', passing the good commit gives None, and HEAD is shown not to be the commit read. Run: `cargo test --locked`. Observes: Every path and every earlier-stop row yields the same reinstall outcome, and the readable control continues and is rated. Rows with an earlier stop (RepeatedFailure, Requested, Rebuilt) fail on the base tree because of behavior, not only because the new helper or fields are missing.
  Fails without the behavior: Without the state-keyed handling, the rows with an earlier stop end in that stop with the config rated. A Rebuilt stop would restart onto the old schema. A helper that reads head in main.rs fails (C).
  Test seam: Entries `drain::drain`, `drain::drain_concurrent`, `RealSteps` with a `Lander` impl (ConfigLander exists), `JobConfig::unreadable_at` (added by step 1) and `reinstall_note_after_landing` (added by step 7). The stubs only supply the error string and the timing or seed for the earlier stop. The RealSteps arm uses the real production parse. The Shared fields are added in step 5 and the table in step 6.
- Criterion 4: Run the three checks Athena will run: `cargo fmt --all --check`, the clippy command above and `cargo test --locked`. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: No fmt diff, no clippy warning, and all tests pass, including the budgets and boundary-word tests.
  Fails without the behavior: A formatting, lint or test failure in the new code, such as an unused import or a changed Stop meaning without its test, would fail a command.
  Test seam: The cargo commands on the whole workspace; no new seam.
- Criterion 5: Inspect the diff paths. Run: `git diff --stat`. Observes: Only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/capability.rs, crates/athena-cli/src/main.rs and CHANGELOG.md change. None of gh/client.rs, routing.rs, run/mod.rs, run/tests.rs, run/verification.rs, athena-data/src/config.rs or athena.toml appear.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8eb08f4d9dbe → candidate d5a7a6cd0e81. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  12 +
 crates/athena-cli/src/drain.rs       |  81 +++++--
 crates/athena-cli/src/main.rs        |  59 ++++-
 crates/athena-cli/tests/drain.rs     | 452 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/capability.rs |  14 ++
 5 files changed, 589 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41606953805 for d5a7a6cd0e819da33cd25fd271314cbaf34055c3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41606953805 for d5a7a6cd0e819da33cd25fd271314cbaf34055c3)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41606953805 for d5a7a6cd0e819da33cd25fd271314cbaf34055c3)
- ci-result: `not a check` passed (Check results for d5a7a6cd0e819da33cd25fd271314cbaf34055c3 come from the CI run https://github.com/doodla/athena/actions/runs/41606953805 (594s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
