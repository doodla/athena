# Job 20261103-231422-8dzf: verify "Retry 193428-df2m: re-reads needing acceptance skip the job limit"

## Request (verbatim)

> try again on 20261103-193428-df2m once the verifier can see the 'fails without the fix' run; the code in candidate b3e381696a71 was correct and only that evidence was missing.
> Depends on 20261103-231410-asf2

## Reading (accepted by the user)

Retry of rejected job 20261103-193428-df2m (code was correct; only the 'fails without the fix' evidence was missing). Re-apply the change so a drain re-read ending as needs accepting gives its --job-limit slot back, with a regression test at job limit 1. It depends on job 20261103-231410-asf2, which lets the verifier see Athena's own revert-and-run record. For operators of `athena drain`.

Assumptions:
- Code Survey: Code survey: crates/athena-cli/src/drain.rs: run_job takes a slot at `st.started += 1` (~line 1439) after the `st.started >= max_jobs` check (~1433); the `_ if run.needs_accepting` arm (~1537) is where the slot is returned. `needs_accepting` comes only from the re-read result (~3572 in the test stub). crates/athena-cli/tests/drain.rs holds the drain tests and its stub. The property is measured directly through the drain report (jobs outcomes, landed, stop reason), not through a stand-in. No record or data file is added, so records validation is unaffected. No refusal rule is added.
- Job 20261103-193428-df2m: its candidate's four files (CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-cli/tests/drain.rs) are kept as deliverables and redone from base, because earlier changes were discarded. The fix is unchanged: a decrement of st.started under the state lock in the needs-accepting arm, plus doc/help wording. Its one major finding (behavior criterion 2 had no recorded command) is addressed by the dependency 20261103-231410-asf2: Athena runs the planned revert demonstration and shows the verifier the record. Its nit (the test uses the stub rather than Source::Reread) is accepted as is.
- Job 20261103-231410-asf2 is a dependency, not part of this change. This job relies on the verifier receiving the recorded failing run of the new test against the old counter, and it does not touch the mechanism that provides it.
- The criteria below name a plan with one demonstration: revert the decrement in drain.rs, then run the new test.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test with max_jobs 1 has item a end as needs accepting and item b runnable. It asserts jobs[0].outcome == "needs accepting", landed == 1 and stop != Stop::JobLimit, and it passes.
2. [behavior] Fails without the fix: with the decrement of st.started in the needs-accepting arm of run_job reverted on a scratch copy, `cargo test -p athena-cli --test drain` fails on the new test. The verifier sees this as Athena's recorded revert run under 'Checks Athena ran'.
3. [content] `git diff` of crates/athena-cli/src/drain.rs shows the needs-accepting arm in run_job giving the slot back (`st.started` decremented under the state lock). The JobLimit check and the failed, held and passed paths are unchanged.
4. [behavior] `cargo test -p athena-cli --test drain` still passes the_job_limit_stops_the_loop and the existing needs-accepting tests, and `git diff` shows no removed or weakened asserts.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace` pass.
6. [content] Grep of crates/athena-cli/src/drain.rs and crates/athena-cli/src/main.rs for the --job-limit help and the Limits::max_jobs doc finds wording that a re-read that only needs accepting does not count toward the limit. CHANGELOG.md has a matching entry.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Avoiding the re-read itself, or changing which config changes trigger a re-read.
- Changing the money budget accounting; the re-read cost still counts against the budget.
- Changing how failed, held or budget-short outcomes count toward the job limit.
- Auto-accepting readings whose scope differs.
- The mechanism that shows the verifier the revert run (job 20261103-231410-asf2).
- Governance files: athena.toml, defaults.toml and workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New regression test plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the drain outcome and stop reason by running the tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the job counter, not at the one symptom.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, the same four files as the earlier attempt.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2dc26c2c0cfb → candidate 32f48822985e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  4 +++
 crates/athena-cli/src/drain.rs   |  9 +++++-
 crates/athena-cli/src/main.rs    |  4 +--
 crates/athena-cli/tests/drain.rs | 70 ++++++++++++++++++++++++++++++++++++++++
 4 files changed, 84 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41796444359 for 32f48822985ea3e39497dcfe6adab56db3a62a32)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41796444359 for 32f48822985ea3e39497dcfe6adab56db3a62a32)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41796444359 for 32f48822985ea3e39497dcfe6adab56db3a62a32)
- ci-result: `not a check` passed (Check results for 32f48822985ea3e39497dcfe6adab56db3a62a32 come from the CI run https://github.com/doodla/athena/actions/runs/41796444359 (606s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace` pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
