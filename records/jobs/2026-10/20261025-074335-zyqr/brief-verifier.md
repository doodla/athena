# Job 20261025-074335-zyqr: verify "Open the job's PR early at verification start, without email literals"

## Request (verbatim)

> try again on 20261025-070538-8ra0

## Reading (accepted by the user)

Retry of the early-PR job. When verification starts in athena go, push the candidate and open or update the job's PR through the host seam, with no merge, so CI caches by fingerprint. Failures are non-fatal, a missing or non-GitHub origin skips silently, land reuses the PR, and no email-like text is added (tests and docs use https URLs).

Assumptions:
- Goal: a faster pipeline for the maintainer; CI caches by fingerprint while the verifier works, then land reuses the open PR.
- The earlier refusal came from SSH-remote-like text in gh.rs tests; https URLs avoid it.
- Deliverable set follows the accepted earlier reading; I did not re-read the source this time.

## Acceptance criteria

1. [behavior] A fake-host test shows that when verification starts in Athena::go, the candidate is pushed to the job's PR branch and the PR is opened or updated via Remote::push and PrHost::open_pr before verification completes, with no merge call. Run: cargo test -p athena-core.
2. [behavior] A fake-host test shows a failing early push or early open_pr leaves the job's outcome identical to the no-failure case (job does not fail; land proceeds as today). Run: cargo test -p athena-core.
3. [behavior] A test shows land still pushes the landing commit to the same branch, reuses the early PR (no second open_pr), uses the lease-pinned replace through check_replace, sets link.head to the landing commit after the replace and before the head check, and merges guarded with the landing commit (merge_heads == [L]). Run: cargo test -p athena-core.
4. [behavior] A test runs go with no origin or a non-GitHub origin and shows the early step is skipped silently: no output line contains 'warning'. Only a host error on a GitHub remote prints a warning. Run: cargo test --workspace.
5. [behavior] cargo test --workspace (existing CLI lifecycle tests, no new 'warning' line), cargo clippy --workspace --all-targets, and cargo fmt --check all pass clean. Each new test fails without the new behavior.
6. [content] The diff adds no email-address-like text to any source, test, or doc file (no SSH remotes of the form user@host:owner/repo); tests and docs use https://github.com/owner/repo URLs. Any identity comes from existing config or git settings. Check with git diff.
7. [content] The production early-push implementation builds its own GhCli runner, so the Athena::go signature and CLI call sites are unchanged.
8. [content] CHANGELOG.md says verification start opens or updates the job's PR so CI caches by fingerprint, that it never merges, that a missing or non-GitHub origin is skipped quietly, and that failures are non-fatal.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Changing the Athena::go signature or CLI call sites
- Merging the PR early
- Changing CI workflow files
- Adding a new identity or email configuration

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New fake-host tests; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check pushes, PR opening, and non-fatal failure through the fake host.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria name commands the verifier can run.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email literals added; https URLs used in tests and docs.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Core test drives real `Athena::go` via the make_athena fake-driver rig with the cfg(test) seam holding recording FakeRemote and FakeHost. At the moment of each push and open_pr call, the fakes record the fake driver's verifier invocation count and whether the verification record file exists. Origin is set to an https://github.com/owner/repo URL. Run: `cargo test -p athena-core early_pr`. Observes: One push of the candidate to athena/<job> and one open_pr with head athena/<job> are recorded. Each was recorded with verifier invocations == 0 and no verification record present. merge was never called, so the calls came before verification completed.
  Fails without the behavior: With the early step removed, no push or open_pr is recorded, so the assertions fail.
- Criterion 2: Run `go` on identical fixtures: (a) working fakes, (b) failing push, (c) push ok but failing open_pr. Compare state, reason and outcome. Then run land on (b) and (c) and on (a). Run: `cargo test -p athena-core early_pr`. Observes: Outcomes of (b) and (c) equal (a), and the only difference is one warning line. After land in (c), where the remote holds C and the early head is stored but there is no link, land replaces C with L, calls open_pr once, and gets merge_heads == [L], the same as (a).
  Fails without the behavior: If errors propagated, go would fail and the equality assertion would break. If the head were not stored before open_pr, land in (c) would be refused with NotFastForward.
- Criterion 3: gh.rs land_pr tests with FakeRemote (records replace and its expected-old argument), FakeHost and a spy ancestry. Cases: a stored link and early head E on the remote; a stored early head with no link; a foreign head X. Run: `cargo test -p athena-core land_pr`. Observes: With a link: open_pr count 0, one replace(branch, L, E), check_replace ran, link.head == L before the head check, merge_heads == [L]. With no link: one open_pr after the replace. With foreign X: no replace, and the existing refusal text.
  Fails without the behavior: Without the replace path, the remote head E != L hits the refusal, or open_pr is called twice.
- Criterion 4: Core tests run `go` with (i) no origin, (ii) a path origin, (iii) a path origin like <tmp>/github.com/example/project.git, collecting UI lines. A fourth test uses an https GitHub origin with a failing host. The CLI lifecycle tests are extended to assert no output line contains 'warning'. Test (iii) also runs land and shows it succeeds as before. Run: `cargo test --workspace`. Observes: For (i), (ii) and (iii): no 'warning' in any line, no push, no open_pr, and land succeeds. For the failing GitHub host: exactly one warning line.
  Fails without the behavior: If the step ran for path origins, it would push or warn and the no-'warning' assertion would fail.
- Criterion 5: Run the full suites. Each new test asserts a push, open_pr, replace or skip call that the base code cannot produce. Run: `cargo test --workspace`. Observes: All tests pass, including the existing CLI lifecycle tests, with no new 'warning' line. `cargo clippy --workspace --all-targets` and `cargo fmt --check` exit 0 clean.
  Fails without the behavior: A test that does not exercise the new behavior would pass against the base. Each new test is written to fail with the early step or replace path removed.
- Criterion 6: Inspect the diff for '@' followed by a host, or user@host:owner/repo text, in source, tests and CHANGELOG. Run: `git diff`. Observes: Added lines contain only https://github.com/owner/repo URLs and no email-like text.
- Criterion 7: Inspect the diff. Run: `git diff`. Observes: The `go` signature is unchanged, there is no crates/athena-cli/src hunk, and `GhCli` is constructed inside `Athena::early_pr`.
- Criterion 8: Read the CHANGELOG hunk. Run: `git diff`. Observes: The entry says verification start opens or updates the job's PR so CI caches by fingerprint, that it never merges, that a missing or non-GitHub origin is skipped quietly, and that failures are non-fatal.

Planned files:

- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)

## The change

Base b791f6a2e2c9 → candidate a4728d026892. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  10 +
 crates/athena-cli/tests/lifecycle.rs    |   6 +
 crates/athena-core/src/gh.rs            | 417 ++++++++++++++++++++++++++++++--
 crates/athena-core/src/land.rs          |   3 +
 crates/athena-core/src/pr.rs            | 126 ++++++++++
 crates/athena-core/src/run.rs           |   4 +
 crates/athena-core/src/run_pin_tests.rs | 247 +++++++++++++++++++
 crates/athena-core/src/test_support.rs  | 134 ++++++++++
 8 files changed, 933 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1208 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
