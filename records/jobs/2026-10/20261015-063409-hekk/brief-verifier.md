# Job 20261015-063409-hekk: verify "Make verifier snapshot read-only; rerun once on fingerprint change"

## Request (verbatim)

> Stop the verifier from changing its snapshot instead of only detecting the change afterwards. Today crates/athena-core/src/run.rs (around line 959) compares HEAD and the work tree after the verifier runs, and verify.rs turns any difference into a blocker that rejects the job. Change this so the verifier runs against a snapshot it cannot write to (for example a read-only copy or a deny on writes in the verifier grant). If the fingerprint still differs, throw that verdict away, run verification once more on a fresh snapshot, and record a system fault, not a rejection of the work. Add tests that show a verifier's write attempt leaves the snapshot unchanged and that a changed fingerprint triggers exactly one rerun. Done when those tests pass along with cargo fmt --check, cargo clippy and cargo test.

## Reading (accepted by the user)

The verifier will run against a snapshot it cannot write to (read-only permissions plus a write deny in its grant). If HEAD or the work tree still differs afterwards, the verdict is discarded and verification reruns once on a fresh snapshot, recording a system fault rather than a rejection of the work. Audience: Athena maintainers, who need verifier misbehaviour separated from bad work.

Assumptions:
- No 'system fault' concept exists in the code (grep found none), so the producer adds a minimal one: a field on the verification/closing data and a ui line, not a new record schema.
- Read-only is done by chmod on the verifier snapshot (as pin.rs make_read_only does) after checks, with a write-deny in the verifier grant if the grant type supports it; the .git dir stays writable only where git needs it, or is also locked.
- Checks (run_check and restore_to) still need a writable snapshot, so lock only for the verifier agent's run and unlock afterwards.
- If the rerun also differs, the job ends as a system fault and not as a rejection of the work; the old snapshot_changed blocker is removed or kept only for that second failure path, never as a work rejection.
- Rerun happens at most once; the retry's cost counts against the verifier budget.

## Acceptance criteria

1. [behavior] A test shows that a write attempt in the verifier's snapshot (a file edit or a git commit) fails or is denied, and that the snapshot's HEAD and work-tree tree are unchanged afterwards. Run: cargo test -p athena-core.
2. [behavior] A test using the fake agent shows a changed snapshot fingerprint after the first verifier run triggers exactly one more verifier invocation on a freshly created snapshot, and that the first verdict is discarded.
3. [behavior] A test shows that when the fingerprint changed, the outcome is recorded as a system fault, and the judgment does not contain the 'verifier changed its snapshot' rejection blocker or reject the work.
4. [behavior] When the snapshot is unchanged, the verifier runs once and the verdict is used as before; existing tests still pass.
5. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass on the workspace.
6. [content] In run.rs the snapshot-changed branch near the old line 959 now discards the verdict and reruns once, and verify.rs no longer turns snapshot_changed into a blocker for the work.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-cap-change/src/lib.rs (modify)

## Out of scope

- Changing the producer's snapshot or sandbox
- Changing the check-tool configuration handling in toolconfig.rs
- Changes to athena.toml, defaults.toml or CI workflows
- Changing how verdicts are judged apart from the snapshot-change path

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the write attempt and for the single rerun; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Prevent the write instead of only detecting it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Change limited to the verifier run path and its tests.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by file permissions and a grant deny, not prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behaviour exercised by tests run through cargo test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 8727ea80c3b1 → candidate ec9b6dc95b3b. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  13 +++
 crates/athena-core/src/pin.rs           |   3 +-
 crates/athena-core/src/run.rs           | 164 +++++++++++++++++++--------
 crates/athena-core/src/run_pin_tests.rs | 191 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/verify.rs        |  48 ++++++--
 crates/athena-git/src/snapshot.rs       | 111 +++++++++++++++++++
 6 files changed, 472 insertions(+), 58 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (665 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
