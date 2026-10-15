# Job 20261015-215711-ayqz: Pin study jobs to the commit the study's pin.json names

## Request (verbatim)

> Study jobs read the wrong firstmate commit: job 20261015-201650-cyvb's pinned source resolved to fc8972cd while studies/firstmate-work-handling/pin.json says 6407052f, so the producer cited lines from the newer code under the old label and the verifier rejected it. A study job must read exactly the commit its study's pin.json names.

## Reading (accepted by the user)

The core resolves a study job's source from athena.toml's repo_ref ("main"), ignoring the pin in studies/<id>/pin.json, so a re-study reads newer code under the old label. Change the pinning step in athena-core so that when the studied repository already has a study with a pin.json, the job pins that exact sha, and add tests. The audience is study producers and verifiers, who must see the mounted tree match the pin.

Assumptions:
- Root cause, read from crates/athena-core/src/run.rs pin_repo and athena.toml [capabilities.study]: pin_repo uses only the global repo_url/repo_ref ("main") and never reads studies/*/pin.json, so any later study job gets the branch tip.
- The request names no study, so the study is found from the repository: the sole study whose pin.json repository_url equals the configured repo_url, else the one whose study_id appears in the request text; otherwise the configured ref applies (a new study).
- The fix is in the core pinning step, not in athena.toml: editing repo_ref to a sha would pin every future study to one commit.
- The existing firstmate-work-handling study is not repinned or re-cited here; the user can ask for that separately.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core (run_pin_tests.rs) dispatches a study job whose head holds studies/<id>/pin.json with a repository_url matching the configured repo_url and a sha differing from what repo_ref ("main") resolves to. It asserts the pinner is asked for that sha, and the job record's pin sha equals it. Run: cargo test -p athena-core.
2. [behavior] Tests cover the fallbacks: with no existing study for the repository the configured repo_ref is used as before; with several studies for one repository, the one whose id the request names is chosen, and if none is named the behavior is documented and never silently picks a different commit. Run: cargo test.
3. [behavior] A pin.json whose sha is not a full 40-hex id, or whose pin does not resolve to that same sha, stops the job before dispatch with a message naming the study, rather than falling back to main. Run: cargo test.
4. [behavior] cargo test --locked, cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check all pass.
5. [content] The doc comment on pin_repo and a CHANGELOG entry state the rule (an existing study's pin.json overrides repo_ref; athena.toml's repo_ref is used only for a new study) and name the cause: job 20261015-201650-cyvb mounted fc8972cd while the study pin said 6407052f.
6. [content] No file under studies/, athena.toml or defaults.toml is changed; re-pinning or re-citing the study is a separate user decision.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/pin.rs (modify)
- crates/athena-core/src/fake.rs (modify)

## Out of scope

- Re-pinning or re-citing studies/firstmate-work-handling to fc8972cd
- Fixing the t3 local-only/scout wording in model.json and intro.json
- The rules.rs STUDY_SHA test mapping
- Changing athena.toml repo_ref

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fix and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy, fmt criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the pinning step that ignores pin.json, not the single study's citations.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert which sha the pinner is asked for and the recorded pin.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code with tests rather than a prose rule.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in pin_repo and its tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands to run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and CHANGELOG explain the cause for outside readers.
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
