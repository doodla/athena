# Job 20261017-044405-4zcq: Keep work and resume on producer deadline; record as timed out

## Request (verbatim)

> A producer stage that reaches its deadline loses all its work (qw38 at 1800s, vc5k at 1800s; jsdj's verifier at 1200s). Keep the work and a progress note on a timeout, record it as timed out rather than failed, and let a retry resume from it; planning should split a request with several separate parts into one item per part.

## Reading (accepted by the user)

When a producer stage hits its deadline, keep the job's worktree edits and write a progress note. Record the job as timed out rather than failed, and have `athena retry` resume from the kept work with the note. Status output tells timed out from failed. The user narrowed this to the producer stage only: no verifier-stage timeouts and no multi-part planning. The audience is the Athena operator, who loses 20–30 minutes of work per timeout.

Assumptions:
- Scope is the producer stage only, per the user's reply. Verifier-stage timeouts and multi-part planning are excluded.
- A deadline kill is currently recorded as killed or failed and the worktree is discarded; I did not confirm this by reading the code.
- The progress note is a file in the job directory or worktree, passed into the retry producer's prompt.
- Deliverable paths are guesses from a grep for 'deadline' and 'Killed'; I did not read the files.

## Acceptance criteria

1. [behavior] A test with the fake agent hits a producer deadline after editing files. Afterwards the edits remain in the job worktree, a progress note exists, and the job is recorded as timed out, not failed. Run: cargo test.
2. [behavior] A test retries a timed-out job. The new producer starts from the kept worktree (not a clean checkout) and is given the progress note. Run: cargo test.
3. [behavior] Status output distinguishes a timed-out job from a failed one, and the deadline reason (e.g. 'deadline of 1800s reached') is kept in the record. Run: cargo test.
4. [behavior] A test shows existing records with killed or failed outcomes still deserialize unchanged. Run: cargo test.
5. [content] The new timed-out outcome is documented and CHANGELOG.md mentions the change.
6. [behavior] cargo test, cargo clippy and cargo fmt --check all pass with no new warnings.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/invocation.rs (modify)

## Out of scope

- Verifier-stage deadline handling
- Splitting multi-part requests into one plan item per part
- Changing the deadline values

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that edits are kept and the retry resumes from them, not that types exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Producer stage only, per the user's narrowing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise the deadline and retry flow with the fake agent.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Keep the work at the point where the timeout discards it, rather than adding a workaround.
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

A test with the fake agent hits a producer deadline after editing files. Afterwards the edits remain in the job worktree, a progress note exists, and the job is recorded as timed out, not failed. Run: cargo test.
A test retries a timed-out job. The new producer starts from the kept worktree (not a clean checkout) and is given the progress note. Run: cargo test.
Status output distinguishes a timed-out job from a failed one, and the deadline reason (e.g. 'deadline of 1800s reached') is kept in the record. Run: cargo test.
A test shows existing records with killed or failed outcomes still deserialize unchanged. Run: cargo test.
The new timed-out outcome is documented and CHANGELOG.md mentions the change.
cargo test, cargo clippy and cargo fmt --check all pass with no new warnings.

## Report

End with the producer report as your structured output.
