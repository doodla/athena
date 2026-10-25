# Job 20261013-030737-zka6: verify "Configure firstmate as the study repo; fix pin tests that clash"

## Request (verbatim)

> Retry 20261012-225445-6y25.

## Reading (accepted by the user)

Set repo_url and repo_ref in athena.toml's [capabilities.study] to firstmate on main, as the first study instance. The two run_pin_tests failed because their config() helper inserts repo_url/repo_ref into the real athena.toml, which would now duplicate keys. Make the helper strip existing keys first. Running the M1 study itself is not part of this change.

Assumptions:
- The study 'work handling (M1)' itself is a later study job; this job only configures the instance.
- The failure came from config() in run_pin_tests.rs inserting repo_url/repo_ref beside the same keys now in athena.toml, giving duplicate TOML keys.
- repo_url is https://github.com/kunchenguid/firstmate.git and repo_ref is main.
- athena.toml is a governance file; it is listed and named in a criterion.

## Acceptance criteria

1. [content] athena.toml [capabilities.study] sets an uncommented repo_url naming the kunchenguid/firstmate repository and repo_ref = "main".
2. [content] The config() helper in crates/athena-core/src/run_pin_tests.rs removes any existing repo_url and repo_ref lines before adding or omitting them, so the with-repo and without-repo cases both work against the real athena.toml.
3. [behavior] Run `cargo test`: all tests pass, including both run_pin_tests tests.
4. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Running the study of firstmate's work handling
- Changing the study capability's code
- Changes under records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test must pass, including the two run_pin_tests, with the configured repo.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The config is read by the study capability, which refuses to start without it; the tests guard the clash.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the config lines and the test helper change.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The retry reference and the failing test output supplied the missing detail.

## The change

Base dd45ba2bb3be → candidate 6d236cc1daae. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  3 +++
 athena.toml                             |  5 ++---
 crates/athena-core/src/run_pin_tests.rs |  9 ++++++++-
 docs/decisions.md                       | 15 +++++++++++++++
 4 files changed, 28 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (302 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
