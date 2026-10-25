# Job 20261017-075358-y8dn: Add the auditor role and fix the stale max_concurrency test

## Request (verbatim)

> try again on 20261017-044408-wcwy

## Reading (accepted by the user)

Retry the rejected auditor job: add an auditor role (fixed prompt, typed finding output, core module, `athena audit` command), judged against BRIEF.md and owner goals. Also fix the base-failing drain test `a_change_to_max_concurrency_alone_does_not_invalidate_a_reading`, which blocked the last attempt. The check-attribution machinery from the introspection is left out as a separate request. The audience is the owner.

Assumptions:
- Audience is the owner, who wants a grader whose yardstick jobs cannot change.
- The failing test is stale, not a product bug: athena.toml now has `max_concurrency = 1`, but the test (drain.rs ~1102) replaces `max_concurrency = 2`, so `before` and `after` are equal and assert_ne! fails. This was read from the repository, not run.
- The fix is to the test: replace `max_concurrency = 1` with 2 and 3. athena.toml is not changed.
- The introspection's proposed baseline-vs-change check attribution is a separate, larger request. Only the typed request (retry the auditor job) plus the one blocking test is done, per the small default.
- The warden is excluded.
- The first attempt's deliverable list is reused, plus crates/athena-cli/tests/drain.rs.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`. All tests pass, including `a_change_to_max_concurrency_alone_does_not_invalidate_a_reading`, and new tests that run the auditor against a fake driver and check it records a typed finding.
2. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
3. [content] In drain.rs the max_concurrency test builds `before` and `after` from the config's actual `max_concurrency = 1` value, so the two differ and the test still asserts that the reading is not invalidated. athena.toml is unchanged.
4. [content] roles.rs defines an auditor role with a fixed prompt constant and a valid RoleDefinition. The prompt says to judge outcomes against BRIEF.md and the owner's goals rather than a job's own criteria, to look at real outcomes rather than records, and to cite evidence for each finding.
5. [behavior] Run `cargo test`. A test shows the auditor's brief contains BRIEF.md and not the job's acceptance criteria. Another shows a finding with no evidence is rejected.
6. [behavior] Run `cargo run -- audit --help`, or the equivalent test. The command exists and, with the fake driver, produces a recorded auditor invocation visible to the owner in CLI output.
7. [content] Role::Auditor round-trips through the parse and as_str functions in records.rs, and every exhaustive match on Role compiles.
8. [content] No warden code is added. BRIEF.md, athena.toml, defaults.toml and workflow files are not edited. CHANGELOG.md has an auditor entry.
9. [content] The docs or code comments state that the auditor's prompt is protected from jobs only by being a fixed constant, that the warden request will enforce this, and that this is prose-only.

## Expected deliverables

- crates/athena-data/src/roles.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/auditor.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/auditor.rs (create)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- The warden role.
- Baseline-versus-change check attribution and per-criterion verdicts when a check fails (a separate request).
- Edits to BRIEF.md, athena.toml, defaults.toml or workflows.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New auditor tests; cargo test and clippy must be clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The check-attribution work is split off; only the auditor and the one blocking test are changed.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy, fmt and the audit command.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check a recorded finding and the rejection of a finding with no evidence, not that files exist.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The prompt protection is marked prose-only.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The stale test fixture is fixed so the test follows the config's value.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the changelog are written for an outside reader.
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
