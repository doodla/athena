# Job 20261025-012657-hac3: verify "Stop writing Athena- trailers in commits after cutover C0"

## Request (verbatim)

> try again on 20261025-001234-2hax

## Reading (accepted by the user)

Retry: after cutover C0, commits from Committer::build_commit_in, commit_with_staged and squash_text carry no Athena- trailers, while trailer readers keep working for commits up to C0. Demonstrated by one squash-landed job PR holding an intervention, a records-only and a job-landing commit, as the user specified.

Assumptions:
- Goal: stop writing Athena- trailers after cutover C0 while history/attribution checks keep passing; audience is the maintainer and the verifier running athena check.
- Dependence on the cutover job 20261025-001232-vesh is assumed already landed in the base.
- Reading widens the deliverables to include crates/athena-gate/src/lib.rs and crates/athena-core/src/manage.rs, as the earlier critique found necessary.
- The user's squash-merge test shape and strip-in-squash_text instruction override the earlier merge-commit plan.
- Trailer stripping is keyed on the parent's cutover state, in Committer::build_commit_in and commit_with_staged.

## Acceptance criteria

1. [behavior] Run cargo test: a scratch-repository test lands the cutover job, then builds ONE job branch whose range holds an intervention commit (commit_with_staged), a records-only commit (Committer) and the job's landing commit, and lands it through land_pr with a squash merge. It asserts the squash commit on the base has no 'Athena-' line, and each of the three PR commits (reachable via the landing record's second parent) has no 'Athena-' line. The test fails without the change.
2. [behavior] Run cargo test: in the same scratch repository the landing record classifies the three commits as declared_intervention, records_only and job_landing, and check_history and check_attribution pass on the post-C0 base (library-level; if a CLI-level athena check run is included in lifecycle.rs it has a clear assertion, otherwise it is omitted).
3. [behavior] Run cargo test: squash_text has a strict form and a relaxed post-C0 form, each tested; the relaxed form emits no Athena- line and is used for post-C0 landings.
4. [behavior] Run cargo test: commits from Committer::build_commit_in and commit_with_staged after C0 carry no Athena- trailer; with no cutover_job set, or before its landing record exists, they still carry trailers and all existing tests pass unchanged.
5. [behavior] Run cargo test: a test shows that after C0 check_history skips content, invariant and jobs checks (runs only secrets and attribution), and that the gate's own check keeps content and jobs checks but drops only trailer-dependent invariant::check.
6. [behavior] Run cargo test: status and audit still read trailers on commits up to and including C0 with a C0..head fallback, giving the same results as before.
7. [content] CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b), name which readers still use trailers up to C0, and describe the history check accurately (after C0 it runs only secrets and attribution).
8. [behavior] Run cargo clippy, cargo fmt --check and cargo test: all clean; no governance files (athena.toml, ci.yml) change.

## Expected deliverables

- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Landing a standalone intervention or records-only PR through land_pr
- Changing athena.toml, defaults.toml or CI workflows
- Removing the trailer readers or their pre-C0 behavior

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for each behavior; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert trailer-free commits and attribution results, not wiring.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier runs.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by tests and checks.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: records_flow test: cutover_world(true) lands the cutover job. Build one job branch: an intervention commit (Committer::commit_with_staged), a records-only commit (Committer::commit_records) and the job landing commit (Committer::build_commit) with Athena-Job, Athena-Verified-Content, Athena-Intervention and Co-Authored-By trailers given. Land through the free land_pr with squash: the scratch repo's Git (production text selection), FakeHost recording the text it is asked to merge with; make the squash commit with commit_tree from that text. Read the Landing record on the records branch, take its second parent and assert it equals the PR head, walk base..that parent. Run: `cargo test post_c0_squash_landing_has_no_athena_trailers`. Observes: The squash message and each of the three commits (second-parent walk, and commits[].sha equal to base..head in order) have no line starting 'Athena-'; Co-Authored-By survives.
  Fails without the behavior: Without the strip or the relaxed text chosen inside land_pr, commits carry Athena- lines, or strict squash text refuses the trailer-free landing commit.
- Criterion 2: Same scratch repo: read the landing record and call check_history and check_attribution on the post-C0 base. No CLI athena check is added to lifecycle.rs. Run: `cargo test post_c0_squash_landing_is_classified_and_checks_pass`. Observes: Kinds are [declared_intervention, records_only, job_landing] in order; check_history and check_attribution report no violations.
  Fails without the behavior: Trailer-free commits judged by old rules, or an unrecorded squash, give violations or wrong kinds.
- Criterion 3: athena-git unit tests: strict squash_text unchanged (exactly one Athena-Job, refuses none or two); squash_text_relaxed on a commit with Athena-Job, Athena-Verified-Content, Co-Authored-By and on one with none. gh.rs tests (names contain squash_text) of the SquashText impl for Git with a pre-C0 parent (strict, refuses a trailer-free commit) and a post-C0 parent (relaxed). The end-to-end test in criterion 1 also uses this path through land_pr. Run: `cargo test squash_text`. Observes: Relaxed body has no Athena- line, keeps Co-Authored-By and prose, and does not error without Athena-Job; the Git impl picks each form by cutover state; land_pr's merge call receives the relaxed text post-C0.
  Fails without the behavior: A relaxed form that keeps trailers fails the no-Athena- assertion; one requiring Athena-Job errors; land_pr using strict fails the criterion 1 test.
- Criterion 4: Repo A (cutover Shown), B (no cutover_job), C (cutover_job named, no landing record): call commit_records and commit_with_staged with Athena-Job, Athena-Intervention, Co-Authored-By trailers. Then the full cargo test with no existing tests edited. Run: `cargo test commits::tests`. Observes: A: no Athena- line, Co-Authored-By kept. B and C: trailers exactly as given. Existing suite passes.
  Fails without the behavior: No strip keeps trailers in A; a strip without the cutover check removes them in B and C and breaks existing tests.
- Criterion 5: history.rs cutover_tests: trailer-free post-C0 tip with a content violation, a governed change with no trailer, and a bad job record; a lib.rs test runs gate(Change::of_commit(post), Scope::Commit) on it, with a pre-C0 parent control. Run: `cargo test -p athena-gate cutover_tests`. Observes: check_history reports no Size, PersonalData, Invariant or jobs violation (only secrets and attribution run) while check_history_with_exempt reports them; gate() reports Size/PersonalData and the jobs violation but no Rule::Invariant; a pre-C0 parent still gets Invariant.
  Fails without the behavior: If gate kept invariant::check, Invariant appears; if it dropped content or jobs, those violations vanish; if check_history ran content, Size appears.
- Criterion 6: Three tests, one each in status.rs, auditor.rs and gh.rs (landed_commit), all named with the substring trace_and_audit_read_trailers_up_to_c0 so one filter runs them. Each uses a scratch repo with a pre-C0 commit carrying Athena-Job: X and a post-C0 trailer-free commit adding X's records, plus a no-cutover control. Run: `cargo test trace_and_audit_read_trailers_up_to_c0`. Observes: trace lists both commits; read_changed_files reads the files; landed_commit returns the post-C0 commit; the control equals trailer-only results.
  Fails without the behavior: Without the fallback the post-C0 commit is missing and the auditor/landed_commit fail with 'cannot audit job' / 'is not landed on this branch'.
- Criterion 7: Read the diff. CHANGELOG entry and D111 follow-up (job 7 part 1b) state that writers stop after C0, that readers still use trailers up to C0 (legacy history rule, trace, auditor, landed_commit with fallback), that after C0 the history check runs only secrets and attribution while the gate drops only the invariant check, and that run.rs retry_landing still needs trailers (follow-up). Run: `git diff -- CHANGELOG.md docs/decisions.md`. Observes: Both files carry the entries and match check_after_cutover in history.rs.
- Criterion 8: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked, then git diff --stat. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: All clean; no athena.toml, defaults.toml or ci.yml in the diff.
  Fails without the behavior: Any format, lint or test failure, or a governance file in the diff.

Planned files:

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 33f8bf9dc9e9 → candidate 1c5ee64907aa. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  18 +-
 crates/athena-core/src/auditor.rs     |   8 +
 crates/athena-core/src/commits.rs     | 146 ++++++-
 crates/athena-core/src/gh.rs          | 702 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/manage.rs      |  29 +-
 crates/athena-core/src/status.rs      |  13 +-
 crates/athena-gate/src/attribution.rs |  21 +
 crates/athena-gate/src/history.rs     | 136 ++++++-
 crates/athena-gate/src/lib.rs         |  14 +-
 crates/athena-git/src/lib.rs          | 124 ++++++
 docs/decisions.md                     |  27 +-
 11 files changed, 1195 insertions(+), 43 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1190 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
