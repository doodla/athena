# Job 20261017-204355-358k: Baseline the checks on the base commit; mark failures pre-existing

## Request (verbatim)

> Before the producer starts, run the job's checks on the base commit and record which ones already fail. At verification, mark those as pre-existing instead of rejecting the job. Judge each acceptance criterion on its own instead of leaving them all unjudged when one unrelated check fails.

## Reading (accepted by the user)

Before the producer starts, Athena runs the capability's checks on the base commit and records which fail. At verification, a check failing at the base is marked pre-existing and no longer rejects the job; the verifier now runs and judges every criterion even when a check fails. A check that fails only on the candidate still fails the job. The audience is Athena's operator and the verifier reading job records.

Assumptions:
- Today run.rs only starts the verifier when every check passed (`if all_passed`), so one failing check leaves all criteria unassessed; that is the behavior to change.
- 'Pre-existing' means the same-named check failed on the base commit; a check is pre-existing only if it also fails on the candidate.
- A new check failure still fails the job, but the verifier runs anyway so criteria are judged individually.
- The base run reuses the same check specs, environment, timeout and kill switch as candidate checks, in a snapshot of the base.
- Nested runs keep skipping checks.

## Acceptance criteria

1. [behavior] A new test shows a job whose base commit fails a check (for example fmt) still reaches the verifier, the record marks that check pre-existing, and the final verdict can be pass. Run `cargo test`.
2. [behavior] A new test shows a check that passes at the base and fails on the candidate is not marked pre-existing and still makes the final verdict fail. Run `cargo test`.
3. [behavior] A new test shows that when a non-pre-existing check fails, the verifier still runs and every acceptance criterion gets its own met or unmet result, instead of all being unassessed. Run `cargo test`.
4. [behavior] A test shows the base-commit check run happens before the producer starts and its failing check names are written to the job record. Run `cargo test`.
5. [behavior] A check that fails on the candidate with a different failure than at the base is not hidden. If the test check runs fewer or zero tests, it is not treated as pre-existing. `cargo test` covers this.
6. [content] The verification record shows each check's pre-existing status, and a stranger reading the verifier brief or the rejection reasons can see which failures were pre-existing and which were new.
7. [content] The CheckRun record change stays readable for existing stored records, with a default for the new field, and CHANGELOG.md describes the new behavior.
8. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass on the candidate. Any that fail at the base are reported as pre-existing.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- docs/ (modify)

## Out of scope

- Fixing the failing checks themselves
- Changing athena.toml check definitions or CI workflows
- Changing how the verifier is allowed to run commands
- Retry or thrash policy

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests and a clean cargo test, clippy and fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name the verdicts and records that must result.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The verdict logic is changed, not a single check special-cased.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change on the check and verdict path.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Records and rejection reasons state pre-existing versus new failures.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the verdict outcome, not that the wiring exists.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
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
