# Job 20261015-223418-qb41: Re-study firstmate work handling and add pin-consistency check

## Request (verbatim)

> try again on 20261015-214235-be8x

## Reading (accepted by the user)

Retry of job be8x. Re-derive the firstmate work-handling study from the mounted source so a newcomer reading index.html finds teardown reopening a backlog item, yolo self-merge, and places freed only by a recorded PR or cleanup. Add a pin-consistency check in rules.rs with tests.

Assumptions:
- Audience: an outside reader or independent reviewer checking the explainer against the pinned firstmate source.
- The mounted source commit is the single pin; if pin.json or labels differ, they are re-pinned to the mounted commit and CHANGELOG records it.
- Gate commands (test, run site, clippy, fmt) are run by the producer and their results reported; they are not content criteria.
- The earlier plan's deliverable list is reused; xtask/src/site.rs changes only if the rebuild needs it.

## Acceptance criteria

1. [content] rules.rs (or its caller) has a check comparing the mounted source commit, pin.json sha, and the sha label on every source in claims.json and rationales.json; on mismatch it fails with a message naming the differing values.
2. [content] pin.json and every citation label name one identical commit, the one the quotes were read from; CHANGELOG states the pin decision and any re-pin.
3. [content] claims.json has cited claims (path, quote, line) that teardown can reopen a backlog item instead of closing it and under which condition; that with yolo on firstmate merges green work itself and what happens with yolo off; and that only a recorded PR or cleanup frees a place, with relaunches and secondmates never counted.
4. [content] model.json, rationales.json, findings.json and intro.json agree with the claims, reference only existing claim ids, and say local-only ships and scouts hold their place until cleanup rather than being freed by a recorded PR.
5. [content] The rebuilt index.html shows the three corrected behaviors, each citing the pinned commit, with quoted lines that exist at that commit.
6. [content] The final report includes a table mapping each requirement to evidence (file and line or command output).
7. [behavior] Run `cargo test`: all pass, including the new pin-consistency test and its mismatched-label rejection case, and the xtask site tests.
8. [behavior] Run `cargo run -p xtask -- site`: succeeds with no unknown claim id error and renders index.html. Run `cargo clippy --all-targets` and `cargo fmt --check`: clean.
9. [content] No secrets or personal data are written.

## Expected deliverables

- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/rationales.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/pin.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- crates/athena-cap-study/src/rules.rs (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)

## Out of scope

- Changing the harness completion gate or report format
- Studying systems other than firstmate work handling
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Check and tests are written in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New pin-consistency test including a mismatch case; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check quotes exist at the pinned commit and the page content, not file existence.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim cites path, quote and line at the pinned commit.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data written.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Study re-derived from source, with a code check against pin drift.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Site build and tests run.
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
