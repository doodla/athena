# Job 20261010-154007-jbt9: Cut generated commit subjects at a word boundary

## Request (verbatim)

> Cut generated commit subjects at a word boundary, not mid-word (n4, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

Replace the three bare `.chars().take(72)` cuts of generated subjects/titles (manage.rs, reading.rs, store.rs) with one shared word-boundary truncation helper, with tests and a CHANGELOG note. Audience: Athena maintainers reading git log.

Assumptions:
- Goal: review finding n4 (mid-word cut like '…tells outsi'); audience is Athena maintainers reading git log and job titles.
- The helper lives in athena-core (e.g. lib.rs) since all three sites are in that crate.
- The take(60) at reading.rs:572 is unrelated and left alone.

## Acceptance criteria

1. [behavior] Unit tests for the new helper pass under `cargo test`: over-limit text is cut at the last whitespace within 72 chars so no word is split; text at or under the limit is unchanged; a single over-long word is hard-cut at 72 chars; multibyte text does not panic.
2. [behavior] A `cargo test` test builds a long intervention subject (manage.rs) and a long reading title (reading.rs) from multi-word text and shows neither ends mid-word and both are at most 72 chars.
3. [content] The three `.chars().take(72)` sites (manage.rs subject, reading.rs title, store.rs title fallback) call the shared helper instead.
4. [behavior] `cargo clippy`, `cargo fmt --check` and the full `cargo test` all pass.
5. [content] CHANGELOG.md notes that generated commit subjects and titles are cut at word boundaries.

## Expected deliverables

- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Other truncations (take(60), take(300), take(12)) that are not commit subjects or titles
- Changing the 72-character limit

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Helper unit tests plus subject/title tests; cargo test and clippy clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared helper replaces all three cut sites rather than fixing one.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change to truncation only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check outputs do not end mid-word.
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
