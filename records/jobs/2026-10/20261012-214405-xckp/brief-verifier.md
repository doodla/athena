# Job 20261012-214405-xckp: verify "Make study fetch firstmate at a resolved pinned SHA, readable by agents"

## Request (verbatim)

> try again on 20261012-212953-rw6t

## Reading (accepted by the user)

Study is already registered in the CLI wiring. The missing work is a core job-start hook that resolves firstmate's main to a SHA, records it, and places the pinned tree inside the agents' confined read-only working directory. [capabilities.study] must also be added to athena.toml. This retry fixes the earlier unreadable-source and missing hook-test findings.

Assumptions:
- The hook is not in the tree: grep finds no on_job_start, so the earlier attempt never landed and all of it is redone.
- The firstmate URL is https://github.com/kunchenguid/firstmate, configured in [capabilities.study]; no SHA is configured. Athena's code resolves the main branch to a full SHA at study start with git ls-remote, then checks out that SHA.
- The resolved SHA is recorded in the job's brief note and in a record or artifact the job already writes, not in athena.toml.
- The pinned tree is copied or exported into each agent's snapshot under a read-only subpath, and is excluded from the change list, so confine accepts reads of it.
- Study cargo grants run offline via CARGO_NET_OFFLINE=true in the study env.

## Acceptance criteria

1. [content] athena_core::capability::Capability has a default no-op job-start hook returning a brief note and extra env. Change keeps the default, and Study overrides it.
2. [content] run.rs calls the hook at job start before the producer. The note appears in the producer and verifier briefs, and the env reaches both agents.
3. [content] athena.toml gains [capabilities.study] (named in this criterion) with the firstmate URL, a branch of main, and offline cargo grants and env. It holds no hard-coded SHA.
4. [content] Study resolves the configured branch to a full 40-hex SHA when a study starts, checks out that SHA, verifies HEAD equals it, and records it. A missing URL, failed resolution or mismatch gives a clear error.
5. [content] The pinned tree sits inside each agent's confined working directory (producer and verifier snapshots), read-only, and is excluded from the change list and admission. The path named in the brief resolves inside the policy root.
6. [behavior] Run cargo test. A dispatch-level test with a fake capability shows the hook ran, its note reached the brief and its env reached the agents. It fails if the run.rs call is removed.
7. [behavior] Run cargo test. A study test builds a local git repo, resolves its branch to a SHA, and shows Read, Grep and Glob accept the materialized tree under the path policy and that it is absent from the change list.
8. [behavior] Run cargo test. The reader catalog lists change and study, and the reader schema's capability enum includes study.
9. [behavior] Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test. All pass.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/Cargo.toml (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Study run step, containment, replay and publishing.
- Changing the change capability's behavior.
- Giving agents network access.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Dispatch-level hook test, a study fetch test using a local git repo, and a catalog test; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The pin is verified by code (HEAD equals the resolved SHA) and the confinement is enforced by the path policy; neither is prose-only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change on the hook, the study fetch and the config section.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use the git CLI for resolve and checkout instead of a new crate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and docs/decisions.md note the resolve-on-start choice.

## The change

Base 655a41c45d0d → candidate 3b2f8319ae51. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                |   6 +
 Cargo.lock                                  |   4 +
 athena.toml                                 |  15 ++
 crates/athena-agent/src/policy.rs           |  69 +++++++-
 crates/athena-cap-study/Cargo.toml          |   4 +
 crates/athena-cap-study/prompts/producer.md |  10 +-
 crates/athena-cap-study/src/lib.rs          | 252 +++++++++++++++++++++++++++-
 crates/athena-cap-study/src/source.rs       | 119 +++++++++++++
 crates/athena-cli/tests/lifecycle.rs        | 188 ++++++++++++++++++++-
 crates/athena-cli/tests/live.rs             |   1 +
 crates/athena-context/src/briefs.rs         |  20 +++
 crates/athena-core/src/capability.rs        |  35 ++++
 crates/athena-core/src/reading.rs           |   3 +
 crates/athena-core/src/run.rs               |  29 +++-
 crates/athena-data/src/config.rs            |   6 +
 crates/athena-git/src/snapshot.rs           |  58 +++++++
 docs/architecture.md                        |   2 +-
 docs/decisions.md                           |  27 +++
 18 files changed, 829 insertions(+), 19 deletions(-)
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
