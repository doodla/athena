# Job 20261112-135930-pczv: verify "Guard Athena's pushes to origin only; give OLY-B-BND-01 a mechanism"

## Request (verbatim)

> try again on 20261030-013721-4gwn

## Reading (accepted by the user)

Retry of job 20261030-013721-4gwn (failed in plan critique round 3, before any code). Add a guard so Athena's own publishing (push, replace, delete_branch, PR/GitHub writes) reaches only the repo's own `origin`, record it as OLY-B-BND-01's mechanism and measure in docs/spec.toml, with a test that fails without it. Goal: Themis sees the boundary enforced by code and a test. The "try again" request is read as a retry of the original job, not the introspection's proposed reading-time check.

Assumptions:
- Prior job 20261030-013721-4gwn: it produced no candidate (no files kept), so all its work is dropped and redone from the base. Its plan-critique findings are addressed as follows. (1) CHANGELOG.md contradiction: the old criterion 7 said CHANGELOG.md untouched, which admission (admission.rs changelog_required) refuses; it is dropped, and CHANGELOG.md is a deliverable with one Unreleased line. (2) origin_url must read `git remote get-url --all origin` and `git remote get-url --push --all origin` and refuse when either lists more than one URL or the URLs name different repositories; tests cover a doubled push URL and a doubled fetch URL with no push URL. (3) crates/athena-git/src/lib.rs is now a listed deliverable. (4) A land_pr test or text assertion shows the origin lookup comes before any write when origin is absent. (5) GitRemote's three write methods recheck origin's URL so the limit is not left to the spec text.
- Code survey: read crates/athena-git/src/lib.rs (push_if_remote_at at line 384, push_branch at 432) and searched crates/ for GitRemote push, replace and delete_branch in crates/athena-core/src/gh/remote.rs. The planning critique also cited landing.rs (land_pr, origin_client), early.rs, gh/tests.rs ScriptRunner and UnreachedHost, and hestia parse_spec, measure_problem and spec_gaps. The producer must re-read them. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly by calling the real push, replace and delete_branch against temp bare repos.
- Existing code: crates/athena-core/src/gh/remote.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/tests.rs, crates/athena-git/src/lib.rs and docs/spec.toml are extended; the guard reuses GitRemote and the existing test doubles.
- Environment: the tests use only local temp git repos with bare `origin` and `upstream` remotes and a fake runner. They make no network or GitHub call, need no credentials, and read nothing from the records branch.
- The measure must end with `status: not shown` plus the test name, since no `athena status` line exists for upstream writes.
- Out of scope items from the earlier job are kept as they were. Blocking pushes to the project's own origin stays out of scope because it is the allowed path.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-BND-01 has `mechanism` naming crates/athena-core/src/gh/remote.rs (not "none"), keeps owner Themis, and `measure` ends with `status: not shown` plus the name of the showing test (git diff docs/spec.toml).
2. [content] Grep finds the guard function in crates/athena-core/src/gh/remote.rs and its use in GitRemote push, replace and delete_branch; crates/athena-git/src/lib.rs push_branch/push_if_remote_at are guarded to origin.
3. [behavior] Run `cargo test -p athena-core`: a new test builds a temp repo with bare `origin` and `upstream` remotes, calls GitRemote push, replace and delete_branch with name `upstream`, asserts an error and unchanged upstream refs, and the same calls to `origin` succeed.
4. [behavior] Run `cargo test -p athena-core`: tests show no origin gives an error and no write, a local-path origin makes no GitHub call, a GitHub https origin targets only the parsed owner/repo, and a doubled push URL and a doubled fetch URL (no push URL) are refused.
5. [content] The new tests assert on the refusal (error and unchanged refs), so removing the guard call from GitRemote::push makes a test fail; the diff shows this and not only wiring.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] CHANGELOG.md has one new Unreleased line naming the OLY-B-BND-01 guard; athena.toml, defaults.toml and .github/workflows are untouched (git status).

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/gh/remote.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how agents' git/gh commands are policed in crates/athena-agent/src/policy.rs.
- Blocking Athena's own pushes and PRs to the project's `origin`.
- Adding an `athena status` line for upstream writes.
- Changes to athena.toml, defaults.toml or .github/workflows.
- The introspection's proposed reading-time check for criteria that contradict defaults; it is a separate job.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in crates/athena-core/src/gh/tests.rs; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert refusal of non-origin writes, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a guard in code and named as the mechanism in docs/spec.toml.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The measure ends with a status label naming the showing test.
- changelog: (not in the defaults model) Here: CHANGELOG.md gets one Unreleased entry, which admission requires.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A new test parses the real docs/spec.toml with `include_str!` (the producer confirms the relative path from crates/athena-core/src/gh/) and `hestia::parse_spec`, finds OLY-B-BND-01 and asserts: mechanism is not `none` and contains `gh/remote.rs`; owner is `Themis`; measure ends with `status: not shown`; `hestia::measure_problem(&measure)` is `None`; the measure names the showing test and `include_str!("tests.rs")` contains `fn <that name>(`; `hestia::spec_gaps` has no entry for the id. Then `git diff docs/spec.toml` shows only that entry's mechanism and measure lines changed. Run: `cargo test -p athena-core --lib gh::tests::oly_b_bnd_01_names_the_origin_guard_as_its_mechanism`. Observes: The test passes, and the diff touches only the mechanism and measure of OLY-B-BND-01 (owner Themis kept).
  Fails without the behavior: With the old `mechanism = "none"` and measure `status: requirement`, the mechanism assertion fails and spec_gaps lists the id.
  Test seam: Production entry points `hestia::parse_spec` (hestia.rs:1172), `measure_problem` (hestia.rs:1219), `spec_gaps` (hestia.rs:1291) on the real file; same style exists (status.rs:1632-1657). No fake. The test is added by plan step 6.
- Criterion 2: Grep for `fn guard_origin_write` and for `guard_origin_write(` in the bodies of `GitRemote::push`, `replace` and `delete_branch` (before `self.git.run`) in remote.rs; Grep for `own_origin_url(` in `Git::push_branch` and `Git::push_if_remote_at` in athena-git/src/lib.rs, each before the push call. The wiring is read only to find the code; behavior is shown by criteria 3 to 5. Run: `git diff crates/athena-core/src/gh/remote.rs crates/athena-git/src/lib.rs`. Observes: One guard function in remote.rs with three call sites, and two call sites in athena-git, each ahead of the git push.
- Criterion 3: `git_remote_refuses_writes_to_any_remote_but_origin` builds a temp repo (`Git::isolated`, as at gh/tests.rs:2680-2703) with two bare repos as remotes `origin` and `upstream`, one commit, and a seed branch pushed to both with raw `git.run(["push", ...])`. Control, same user and environment: a raw `git push upstream` of a new branch succeeds and `ls-remote --heads upstream` is the baseline. Then `GitRemote { git, name: "upstream" }` `push`, `replace(branch, new, seed)` and `delete_branch(seed)` each return Err and upstream's refs equal the baseline; a third remote `mirror` is refused the same way. The same three calls on `GitRemote { name: "origin" }` succeed and origin's refs show the pushed, replaced and deleted branches. A twin test in athena-git/src/lib.rs does the same through `Git::push_branch` and `Git::push_if_remote_at`. Also run `cargo test -p athena-git`. Run: `cargo test -p athena-core --lib gh::tests::git_remote_refuses_writes_to_any_remote_but_origin`. Observes: The control push to upstream lands (so it is possible without the guard), every guarded write to upstream is an error with refs unchanged, and the same writes to origin succeed.
  Fails without the behavior: Removing `guard_origin_write` from `GitRemote::push` lets the push to `upstream` land, so the is_err and unchanged-refs assertions fail.
  Test seam: Production entry points `GitRemote::push`/`replace`/`delete_branch` (remote.rs:52-79) and `Git::push_branch`/`push_if_remote_at` (athena-git lib.rs:384, 432); real git over temp bare repos, no fake; fixture style exists (gh/tests.rs:2680-2752). The tests and the guard are added by plan steps 2 to 6. The guard runs before any git push is spawned.
- Criterion 4: Tests in gh/tests.rs and athena-git. (a) `git_remote_with_no_origin_refuses_and_writes_nothing`: only `upstream`; `GitRemote{name:"origin"}` push/replace/delete_branch each Err; upstream refs unchanged. (b1) `git_remote_refuses_a_doubled_push_url`: origin accepts a push (control); then `git config --add remote.origin.pushurl <origin's own URL>` twice; the test asserts `git remote get-url --push --all origin` prints two lines and the first equals the fetch URL; push/replace/delete_branch Err and the bare is unchanged (only the count check can refuse). (b2) same with the second push URL on another bare: Err and both bares unchanged. (b3) `git_remote_refuses_a_doubled_fetch_url`: `git config --add remote.origin.url <second bare>` (two fetch URLs, no push URL); the test asserts `get-url --push --all` prints two lines; push Err, both bares unchanged. (c) `git_remote_refuses_a_push_url_naming_another_repository`: one `remote.origin.pushurl` on another bare; Err, unchanged (only the same-repository check can refuse). (d) `origin_repository_reads_only_the_parsed_owner_and_repo`: `own_origin_repository(git)` returns `example/project` for `https://github.com/example/project.git`; Err for no origin; Err for a local-path origin; Err for a local path containing `github.com/o/r`; Err for `https://example.com/github.com/owner/repo`; Err for `ssh://github.com/example/project.git` (the accepted behavior change); Err for the https GitHub origin with a doubled push URL (real double as in b1) and with a push URL on another repo. A close_pr(9) ScriptRunner path check is kept in the module that defines ScriptRunner as a regression note only. (e) `a_local_path_origin_makes_no_host_or_remote_call`, `a_doubled_github_origin_makes_no_host_or_remote_call`, `a_mismatched_github_push_url_makes_no_host_or_remote_call`: an `Athena` from `code_health::tests::set_up()` with an `EarlyFake` (test_support.rs:275) in the early seam, guard held to the end. Control: single-URL `https://github.com/example/project.git` makes the fake record a push and an open_pr. Local-path, doubled (real double) and mismatched origins: `early_pr`, `push_ci_branch`, `delete_ci_branch` leave `calls` and `ci_calls` empty; with the top-level `Lines` Ui (gh/tests.rs:3504) the doubled case prints the 'more than one URL or names two repositories' line and not 'no GitHub origin', and the local-path case still prints 'no GitHub origin'. (f) `land_pr_builds_the_remote_and_sink_only_after_the_checked_origin_read`: ordering of `own_origin_repository(` before `GitRemote {` and `GitLandingRecords::for_remote` in landing.rs (ordering only). (g) athena-git unit tests: normalisation (https vs scp-style same repo accepted; other repo refused; explicit port equal to itself accepted, refused against portless; path-case difference refused) and the hook test (control through `push_branch` and `push_if_remote_at` returns Ok with no hook; with `pre-receive` exit 1 each returns `GitError::Failed` with git's hook-declined text, not `Refused`; refs unchanged). Also run `cargo test -p athena-git`. Run: `cargo test -p athena-core --lib gh::tests`. Observes: Every refusal case returns Err (or makes no call) and every ref listing is unchanged. The doubled cases are proven doubled by the two-line assertion. A single-URL https GitHub origin yields only the parsed owner/repo; a non-GitHub host, an ssh form or a local path yields none. The control makes the EarlyFake record calls, so empty calls in the other cases mean refusal.
  Fails without the behavior: Removing only the 'exactly one line' check on `get-url --push --all` makes (b1) push to the same repository twice and fail its Err assertion, while (c) still passes; removing only the same-repository check makes (b2) and (c) fail while (b1) still passes; removing the host check in `own_origin_repository` makes the example.com/github.com, ssh and local-path cases yield a repository; without the checked read in `github_repository`, the doubled GitHub origin makes the EarlyFake record a push and an open_pr; without the distinct skip line, the doubled case prints 'no GitHub origin'.
  Test seam: Production entry points: `GitRemote` methods, `Git::own_origin_url`, `own_origin_repository`, `Athena::early_pr`/`push_ci_branch`/`delete_ci_branch` (early.rs:384-437, pub(crate), same crate; `with_ci_remote` takes the seam first at early.rs:369-373 but `github_repository` still reads git, which is what the doubled case tests). Fixtures: real temp bare repos; `EarlyFake` injected through `gh::early_seam::install` (gh/mod.rs:99, exists, used at run_pin_tests.rs:425); `Lines` Ui exists (gh/tests.rs:3504); ScriptRunner exists (gh/tests.rs:4794). `own_origin_repository`, `Git::own_origin_url` and the tests are added by plan steps 2 to 6. No double holds the logic under test.
- Criterion 5: Read the diff: every new write test asserts `is_err()` (or the error variant and text) and compares the refs listing before and after; none asserts only that a function was called. As a local experiment (reverted), delete the `guard_origin_write` line from `GitRemote::push` and re-run the criterion 3 test. Run: `git diff crates/athena-core/src/gh/tests.rs`. Observes: The tests assert on the refusal and unchanged refs, not the wiring; with the push guard call removed, the criterion 3 test fails.
  Fails without the behavior: With the push guard call removed, the push to `upstream` succeeds and `git_remote_refuses_writes_to_any_remote_but_origin` fails on the is_err and unchanged-refs assertions.
  Test seam: Same as criterion 3: `GitRemote::push` over real temp bare repos; no fake.
- Criterion 6: Run, as separate simple commands: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Also `cargo test -p athena-core --test refusal_corpus` (this change adds no reading rule or parser of request text, so no case is added; its result is not read here), `cargo test -p athena-core --test external_calls_guard`, `cargo test -p athena-git -p athena-gate`, `cargo test -p athena-cli --test trailer_stop --test reconcile --test binary --test drain --test external_calls`, `cargo test -p xtask --test policy`, `cargo test -p xtask --test spec`, `cargo test -p athena-core hestia::`. The two pushurl-to-a-missing-path tests (attribution.rs:1690-1707, cli tests/binary.rs:631-660) still pass, now refused by the guard; no existing test is weakened. Run: `cargo test --locked`. Observes: All commands exit 0 and the new guard tests are among those that ran and passed.
  Fails without the behavior: A missing guard turns the criterion 3 and 4 tests and the athena-git twins red; unformatted code or a lint turns fmt or clippy red; a guard or host check too strict for the literal-origin callers turns the records_branch, attribution, binary, trailer_stop, reconcile and land_through_pr tests red.
  Test seam: Existing suite plus the new tests from criteria 3-5; no further seam.
- Criterion 7: Check that CHANGELOG.md has one added line under `## [Unreleased]` / `### Added` naming OLY-B-BND-01 and the origin guard (and, in the same line, that landing now needs an https GitHub origin), and that athena.toml, defaults.toml and .github/workflows are not in the status. The line has no email-like address or banned word (`cargo test -p xtask --test policy`). Run: `git status`. Observes: CHANGELOG.md plus the six other deliverables are modified and nothing else; admission's `changelog_required` (admission.rs:819-831) is met.

Planned files:

- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh/remote.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8f59e7757383 → candidate a02d212678e0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   1 +
 crates/athena-core/src/gh/early.rs   |  21 +-
 crates/athena-core/src/gh/landing.rs |  13 +-
 crates/athena-core/src/gh/remote.rs  |  33 ++-
 crates/athena-core/src/gh/tests.rs   | 477 +++++++++++++++++++++++++++++++++++
 crates/athena-git/src/lib.rs         | 377 +++++++++++++++++++++++++++
 docs/spec.toml                       |   4 +-
 7 files changed, 916 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261112-135930-pczv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-135930-pczv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43245369020 for a02d212678e02d172e4e1e0ef2af7681e7ac654a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43245369020 for a02d212678e02d172e4e1e0ef2af7681e7ac654a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43245369020 for a02d212678e02d172e4e1e0ef2af7681e7ac654a)
- ci-result: `not a check` passed (Check results for a02d212678e02d172e4e1e0ef2af7681e7ac654a come from the CI run https://github.com/doodla/athena/actions/runs/43245369020 (706s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib gh::tests::oly_b_bnd_01_names_the_origin_guard_as_its_mechanism`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/remote.rs, crates/athena-git/src/lib.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
   Compiling winnow v1.0.4
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling itoa v1.0.18
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling tempfile v3.27.0
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars_derive v1.2.2
   Compiling regex-automata v0.4.18
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `super::remote::own_origin_repository`
    --> crates/athena-core/src/gh/tests.rs:5175:9
     |
5175 |     use super::remote::own_origin_repository;
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `own_origin_repository` in `gh::remote`

error[E0425]: cannot find function `own_origin_repository` in module `crate::gh::remote`
    --> crates/athena-core/src/gh/tests.rs:4890:50
     |
4890 |         let (_, repository) = crate::gh::remote::own_origin_repository(&git).unwrap();
     |                                                  ^^^^^^^^^^^^^^^^^^^^^ not found in `crate::gh::remote`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 2 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib gh::tests::git_remote_refuses_writes_to_any_remote_but_origin`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/remote.rs, crates/athena-git/src/lib.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `super::remote::own_origin_repository`
    --> crates/athena-core/src/gh/tests.rs:5175:9
     |
5175 |     use super::remote::own_origin_repository;
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `own_origin_repository` in `gh::remote`

error[E0425]: cannot find function `own_origin_repository` in module `crate::gh::remote`
    --> crates/athena-core/src/gh/tests.rs:4890:50
     |
4890 |         let (_, repository) = crate::gh::remote::own_origin_repository(&git).unwrap();
     |                                                  ^^^^^^^^^^^^^^^^^^^^^ not found in `crate::gh::remote`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 2 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib gh::tests`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/remote.rs, crates/athena-git/src/lib.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `super::remote::own_origin_repository`
    --> crates/athena-core/src/gh/tests.rs:5175:9
     |
5175 |     use super::remote::own_origin_repository;
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `own_origin_repository` in `gh::remote`

error[E0425]: cannot find function `own_origin_repository` in module `crate::gh::remote`
    --> crates/athena-core/src/gh/tests.rs:4890:50
     |
4890 |         let (_, repository) = crate::gh::remote::own_origin_repository(&git).unwrap();
     |                                                  ^^^^^^^^^^^^^^^^^^^^^ not found in `crate::gh::remote`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 2 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff crates/athena-core/src/gh/tests.rs`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
