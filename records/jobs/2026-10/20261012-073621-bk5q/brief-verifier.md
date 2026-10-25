# Job 20261012-073621-bk5q: verify "Fix producer guidance causing D43 and synthetic-commit rejections"

## Request (verbatim)

> the design revision failed twice the same way (20261012-073105-mj6d); find out why and fix the cause

## Reading (accepted by the user)

The design-revision job failed twice because the producer's evidence cited git status/git diff runs the verifier never saw (D43), and it pinned BRIEF.md to a commit that exists only in the synthetic verification history. I read this as a producer-guidance defect. The fix tells the producer what the verifier can see and run, and a regression test covers it. The D43 gate stays as it is.

Assumptions:
- The job records are hidden, so the cause is taken from the user's reply. Both failures came from the producer's guidance, not the verifier's gate: the producer cited evidence the verifier never saw, and referenced a commit hash that exists only in the synthetic verification history.
- The right fix is to the producer's brief, and possibly the design-revision guidance, so the producer writes verifier-reproducible evidence and does not pin to unstable commit hashes. D43 itself is correct and stays.
- The producer brief is built in crates/athena-context/src/briefs.rs; I did not trace the design-revision capability path beyond this.

## Acceptance criteria

1. [content] The change's description names the root cause, taken from the user's reply: the verifier rejected behavior criteria whose evidence cited git status/git diff runs it never saw (D43), and the producer pinned BRIEF.md to a commit that exists only in the synthetic verification history. It cites the file and function that let this happen.
2. [content] The producer brief built in crates/athena-context/src/briefs.rs states that the verifier sees only the candidate diff in a fresh snapshot. It says evidence must cite commands the verifier can run itself, and that the producer's own git status/git diff runs do not count. It says that commit hashes made during verification do not exist in the real history and so must not be pinned or cited. Doc and design-revision text must not pin to such a commit.
3. [content] The fix is in the prompt or context construction, or the producer-side guidance. It does not loosen the D43 rule in crates/athena-core/src/verify.rs or any other gate, admission rule, or check.
4. [content] A regression test in athena-context asserts that the producer brief contains the new guidance. It fails on the base commit.
5. [behavior] Run `cargo test` from the repository root. It passes, including the new regression test.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings`, then `cargo fmt --all --check`. Both are clean.
7. [content] docs/decisions.md gets a short entry recording the cause and the fix. Governance files (athena.toml, defaults.toml, .gitattributes, CI workflows) and records/ are unchanged.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/verify.rs (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)

## Out of scope

- Relaxing the D43 rule or any verifier gate.
- Changing athena.toml, defaults.toml, CI workflows or records/.
- Re-running or repairing job 20261012-073105-mj6d itself.

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The failure text came from the user's reply, and the code was located by searching the repository.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A regression test covers the new brief guidance, and cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test, clippy and fmt, which the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the producer guidance, with a test and a decisions entry.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces the guidance in the brief so it cannot silently disappear.

## The change

Base 721100089129 → candidate 77c29aa95a7d. In your working directory, `git diff HEAD~1` shows it.

```text
 crates/athena-context/src/briefs.rs | 27 +++++++++++++++++++++++++++
 docs/decisions.md                   | 18 ++++++++++++++++++
 2 files changed, 45 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (222 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
