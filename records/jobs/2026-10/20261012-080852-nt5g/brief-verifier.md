# Job 20261012-080852-nt5g: verify "Match cited verifier commands against simple commands inside ran ones"

## Request (verbatim)

> D43 keeps rejecting producers' test evidence (latest 20261012-075010-86ja, also 20261012-073105-mj6d); 531f85b didn't fix it; fix the real cause

## Reading (accepted by the user)

D43 compares each cited command to the verifier's ran commands by exact whole-string match. A verifier that chains or pipes commands cites a piece, which then counts as unseen, so I read the real cause as the matcher. The change relaxes matching to whole simple commands inside what was run, adds tests, and records a decision that amends D43.

Assumptions:
- The real cause is in D43's exact whole-string match in verify.rs::judge: the verifier prompt says to list commands 'exactly as you ran them', but a verifier that chains or pipes commands (policy.rs allows `&&` and pipes) cites a piece, so the cited string is not in the ran list. Both cited commands in 86ja end in `2>&1`, which policy.rs allows, so the policy is not denying them. I could not read the job records (absent by design), so this is inferred from the code, not from the verifier's actual command log.
- The fix is to relax matching to whole simple commands within compounds, not to loosen D43 for unrun commands; the rule that an unrun command makes the criterion unmet stays.
- 531f85b is taken to be the commit that added D62 and the producer-brief 'What the verifier sees' section, which targeted the producer side and not the matcher.

## Acceptance criteria

1. [content] `judge` in crates/athena-core/src/verify.rs counts a cited command as seen when it equals a command the verifier ran, or equals one whole simple command inside a compound the verifier ran (split on `&&`, `;`, `||`, `|`), after trimming and collapsing whitespace. Split on shell separators as policy.rs does, and never match inside quotes. A trailing output filter such as `| tail -n 30` on a ran command does not hide the cited command before it.
2. [content] Matching stays strict where it matters: a cited command with different arguments, a different test filter, or a different package is still unseen. A command the verifier never ran is still reported as 'cited but not seen running' and the criterion is still unmet.
3. [content] docs/decisions.md gains a new decision that amends D43. It states the measured cause (exact whole-string matching failed verifiers that ran the cited command inside a compound or piped command, as in jobs 20261012-075010-86ja and 20261012-073105-mj6d), says why 531f85b (producer brief guidance) did not address it, and gives alternatives, tradeoff, and reversal cost. It cites no commit hash made during verification.
4. [content] crates/athena-context/prompts/verifier.md and the D43 text in the v0 design either stay consistent with the new rule or are updated to say that cited commands are matched as whole simple commands inside what was run. CHANGELOG.md has an entry under the correct existing heading.
5. [behavior] New unit tests in crates/athena-core/src/verify.rs cover: a cited command run alone; run after `cargo build &&`; run with `2>&1 | tail`; extra whitespace; a cited command with a different filter or package (unmet); a never-run command (unmet). Run `cargo test --locked -p athena-core verify` and expect all to pass.
6. [behavior] The whole workspace stays green: `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`, and `cargo fmt --check` all pass.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Loosening D43 so that unrun commands count as evidence.
- Changing how the stream records Bash commands (stream.rs) or the permission policy, unless the tests show a gap.
- Re-running or fixing jobs 20261012-075010-86ja and 20261012-073105-mj6d.
- Further changes to the producer brief (D62).

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the new matching rule in verify.rs; cargo test and clippy must pass.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The fix and its tests are Rust; the only non-Rust edits are existing docs and a prompt.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The new decision entry explains the cause with job ids and no commit hashes.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision cites the failing jobs' findings as its evidence and labels the inference about the cause.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the matcher plus its decision record.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria cite cargo commands the verifier is allowed to run.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The cause was recovered from verify.rs, policy.rs, stream.rs, and the verifier prompt instead of asking.

## The change

Base ce7188eca96c → candidate 71a1f4c0e70f. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |   5 ++
 crates/athena-context/prompts/verifier.md |   2 +-
 crates/athena-core/src/verify.rs          | 139 +++++++++++++++++++++++++++++-
 docs/decisions.md                         |  24 ++++++
 docs/design/athena-v0.md                  |   6 +-
 5 files changed, 169 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (227 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
