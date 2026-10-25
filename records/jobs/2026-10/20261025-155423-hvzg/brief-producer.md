# Job 20261025-155423-hvzg: Keep work and progress note when producer hits deadline; resume on retry

## Request (verbatim)

> try again on 20261017-044405-4zcq

## Reading (accepted by the user)

When the producer stage hits its deadline, keep the job's worktree edits and write a progress note listing changed files. Record the job as timed out, not failed. Make `athena retry` resume from the kept work with the note, and make status show timed out versus failed. Producer stage only; the audience is the Athena operator who loses 20-30 minutes per timeout.

Assumptions:
- Scope is producer stage only, per the user's replies: no verifier-stage timeouts, no planning split, no deadline length change.
- Existing record formats must stay readable.
- Deliverable paths are carried over from the earlier attempt and not re-verified in the repository.

## Acceptance criteria

1. [behavior] A test with the fake agent exceeds a short producer deadline after editing files. Afterwards the edits remain in the job worktree, a progress note listing the changed files exists, and the job is recorded as timed out, not failed. Run: cargo test.
2. [behavior] A test retries a timed-out job. The new producer starts from the kept worktree (not a clean checkout) and is given the progress note naming the earlier changed files. Run: cargo test.
3. [behavior] Status output distinguishes a timed-out job from a failed one, and the deadline reason (e.g. 'deadline of 1800s reached') is kept in the record. Run: cargo test.
4. [behavior] A test shows existing records with killed or failed outcomes still deserialize unchanged, and a job that finishes in time behaves as before. Run: cargo test.
5. [content] The new timed-out outcome is documented and CHANGELOG.md mentions the change.
6. [behavior] cargo test, cargo clippy and cargo fmt --check all pass with no new warnings. Run each.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/invocation.rs (modify)

## Out of scope

- Verifier-stage timeouts
- Splitting multi-part requests in planning
- Changing the deadline length
- Pre-dispatch size checks

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy, fmt to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check worktree edits, note, and recorded outcome in tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the first step: keep and resume.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A test with the fake agent exceeds a short producer deadline after editing files. Afterwards the edits remain in the job worktree, a progress note listing the changed files exists, and the job is recorded as timed out, not failed. Run: cargo test.
A test retries a timed-out job. The new producer starts from the kept worktree (not a clean checkout) and is given the progress note naming the earlier changed files. Run: cargo test.
Status output distinguishes a timed-out job from a failed one, and the deadline reason (e.g. 'deadline of 1800s reached') is kept in the record. Run: cargo test.
A test shows existing records with killed or failed outcomes still deserialize unchanged, and a job that finishes in time behaves as before. Run: cargo test.
The new timed-out outcome is documented and CHANGELOG.md mentions the change.
cargo test, cargo clippy and cargo fmt --check all pass with no new warnings. Run each.

## Report

End with the producer report as your structured output.
