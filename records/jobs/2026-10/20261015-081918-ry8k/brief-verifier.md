# Job 20261015-081918-ry8k: verify "Make verifier snapshot unwritable even for root; rerun once on change"

## Request (verbatim)

> try again on 20261015-063409-hekk

## Reading (accepted by the user)

The verifier runs against a snapshot it cannot write to even as root (a write deny in its grant enforced by the agent policy, plus a root-proof lock such as the immutable attribute where available). If HEAD or the tree still differs, the verdict is discarded, verification reruns once on a fresh snapshot, and a system fault is recorded instead of rejecting the work. Audience: Athena maintainers. Protection tests must prove they can fail: the write succeeds unlocked, is refused locked, and fail loudly rather than skip when the lock cannot be enforced.

Assumptions:
- Athena's agents run as root, so permission bits alone do not stop writes; the enforcing mechanism must be the grant's write deny in crates/athena-agent/src/policy.rs (and/or an immutable attribute or unprivileged user).
- The previous attempt's work is discarded; this starts from the base and keeps the same rerun-once and system-fault semantics.
- When the budget runs out before the rerun, the job's reason must report the budget failure, not 'changed its snapshot twice'.
- Tests run as the same user as the verifier and never silently return early when a write succeeds.

## Acceptance criteria

1. [behavior] A test first shows a file edit and a git commit succeed in an unlocked snapshot under the current user, then shows both are refused once the verifier's lock/grant is applied, and that HEAD and work-tree tree are unchanged. It fails (not skips or returns early) if the write is not refused. Run: cargo test -p athena-git and cargo test -p athena-core.
2. [behavior] The verifier-grant write deny is tested in policy: a write tool call against the snapshot path is denied even when the process is root. Run: cargo test -p athena-agent.
3. [behavior] A fake-agent test shows a changed snapshot fingerprint after the first verifier run triggers exactly one more verifier invocation on a freshly created snapshot, and the first verdict is discarded. Run: cargo test -p athena-core.
4. [behavior] A test shows that after a changed fingerprint the outcome is recorded as a system fault and the judgment has no 'verifier changed its snapshot' blocker and does not reject the work; an unchanged snapshot runs the verifier once and uses its verdict. Run: cargo test -p athena-core.
5. [behavior] A test shows that when the budget is exhausted before the rerun, the job's reason reports the budget failure rather than 'snapshot changed twice'. Run: cargo test -p athena-core.
6. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass on the workspace.
7. [content] verify.rs no longer turns snapshot_changed into a blocker against the work, and run.rs near the old snapshot_changed branch discards the verdict and reruns once.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/pin.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Changing producer snapshot handling
- Changes to records/ or governance files such as athena.toml and defaults.toml
- Redesigning the verifier's command allow-list

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for lock, rerun, fault and budget paths; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Write protection is enforced by a grant deny and a lock, not by prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Prevent the write rather than only detecting it, and make tests prove they can fail.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Tests exercise the write attempt behavior directly.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Prefer existing OS facilities (immutable attribute, policy deny) over new machinery.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base ede67e41d9d1 → candidate c85ecaee3b29. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  13 +++
 crates/athena-agent/src/policy.rs       |  50 +++++++++
 crates/athena-cap-change/src/lib.rs     |   6 ++
 crates/athena-core/src/capability.rs    |   2 +
 crates/athena-core/src/pin.rs           |   2 +
 crates/athena-core/src/run.rs           | 136 +++++++++++++++--------
 crates/athena-core/src/run_pin_tests.rs | 185 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/verify.rs        |  63 +++++++++--
 crates/athena-git/src/snapshot.rs       | 178 ++++++++++++++++++++++++++++++
 9 files changed, 584 insertions(+), 51 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (668 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
