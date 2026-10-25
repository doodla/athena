# Job 20261017-062529-jqsr: Route checks by job kind, skipping fmt/clippy only for prose-only change

## Request (verbatim)

> try again on 20261017-054106-xgc2

## Reading (accepted by the user)

Build ci-routing.md section 10 item 5: a routing function in athena-core that maps the plan's class_of_work to the checks and their source (reused CI result via item 4 `resolve`, or local), following the 3.2 table. The 'as today' skip stays path-based (`skip_when_only`), so a job of any kind that changes Rust, Cargo.toml or Cargo.lock still runs fmt and clippy. The audience is the Athena maintainer and ci-routing.md reviewers.

Assumptions:
- Section 3.2 says docs/design 'fmt and clippy skipped as today'; today's rule is path-based `skip_when_only` (docs/ and *.md), and section 6.1 says that rule is unchanged and verification is not weakened. Job kind never skips fmt or clippy by itself.
- Study, config, tests and code jobs never get fmt/clippy skipped by kind. A study job gets the same gates as a change; any prose-only skip comes only from the path rule.
- The `test` check always runs for every kind, because the policy tests read prose.
- A build job adds the workflow's own run and never reuses a prior CI result.
- A missing or unrecognised kind routes like config (fmt, clippy, test, none skipped), per section 7.
- CI result reuse goes through item 4 `resolve` with the candidate SHA. A result for another SHA is ignored and the check runs locally. A red CI result fails and does not rerun locally.
- The kind-specific data is only the reuse/source decision and the build extra. The skip decision stays on the existing `skip_when_only` fields, so athena.toml and the schema in athena-data/config.rs need little or no change. If a field is added, the schema, athena.toml and the capability crates' field copies change together.
- No speedup is claimed or measured.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core routing`: a docs-kind job whose changed paths are only markdown or under docs/ skips fmt and clippy and still runs test.
2. [behavior] Run `cargo test -p athena-core routing`: for each of docs, design and study, a job that changes a .rs file runs fmt and clippy (one named test per kind). The same holds when only Cargo.toml or Cargo.lock changes.
3. [behavior] Run `cargo test -p athena-core routing`: a code job and a tests job run fmt, clippy and test with none skipped. A config job never has any check skipped. A build job adds the workflow's own run and never reuses a prior result.
4. [behavior] Run `cargo test -p athena-core routing`: a missing or unrecognised kind routes as config (fmt, clippy, test, none skipped).
5. [behavior] Run `cargo test -p athena-core`: the routed path calls `resolve` with the candidate SHA. A CI result for another SHA is ignored and the check falls back to a local run. A red CI result fails the check and does not rerun locally.
6. [behavior] Run `cargo test` including a test that loads the committed athena.toml with the schema in the tree: athena.toml parses, and representative jobs of each kind route as 3.2 says with the committed config.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both pass, and `cargo test` passes in full.
8. [content] The routing function is called from the job run path (run.rs) to select the checks by the plan's kind, so a reader can find the call site. Skipping still uses the path rule, and no code path skips fmt or clippy by kind alone.
9. [content] docs/design/ci-routing.md section 10 item 5 states what was built and what is not done, and does not claim a measured speedup. Its wording keeps 'skipped as today' as the path rule. CHANGELOG.md has an entry.
10. [content] If the config schema or athena.toml changed, the change is limited to routing, is listed in the diff, and every crate that copies the field compiles.

## Expected deliverables

- crates/athena-core/src/routing.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- docs/design/ci-routing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Measuring CI speedup or cache hit rates
- Changing the verifier's allowed commands
- Changing the CI workflow file or the build job (other items of section 10)
- Changing the existing path-based skip_when_only rule for prose-only changes

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-case routing tests; cargo test and clippy pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier runs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check routing outcomes per kind, including the Rust-changing docs job.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The skip rule stays path-based so a mislabelled kind cannot weaken checks.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One routing function with minimal config change.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The doc update states what is not measured.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by tests, including a loaded-athena.toml test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
