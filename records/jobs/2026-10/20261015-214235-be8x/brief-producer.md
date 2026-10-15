# Job 20261015-214235-be8x: Re-study firstmate work handling; add pin-consistency check; rebuild pag

## Request (verbatim)

> try again on 20261015-201650-cyvb

## Reading (accepted by the user)

Goal: a newcomer reading the explainer page can verify every citation at one pinned firstmate commit. Re-study work handling (including teardown reopening a backlog item, yolo self-merge, places freed only by recorded PR or cleanup), rebuild the page, and add a code check that the mounted source commit, pin.json and every citation label agree, failing early.

Assumptions:
- Reading restated for confirmation: re-derive the study from one pinned commit, rebuild the page from it, and make a mismatch between mount, pin.json and citation labels fail early.
- The mounted source's commit cannot be read from git metadata (the mount has none), so the check uses a commit marker supplied by the mount or harness; if none exists, the producer must stop and report it, not silently re-cite.
- If the mounted commit differs from 6407052f, the study is re-pinned to the mounted commit in pin.json and all labels, and the CHANGELOG says so; no citation is relabelled without that.
- Audience is an outside reader who checks quotes at the pinned commit; places-held wording must be accurate for local-only ships and scouts, which keep their place until cleanup.

## Acceptance criteria

1. [content] A check in crates/athena-cap-study/src/rules.rs (or its caller) compares the mounted source commit, pin.json sha, and the sha label on every source in claims.json and rationales.json, and fails with a message naming the differing values.
2. [content] pin.json and every citation label name one identical commit, which is the commit of the mounted source the quotes were read from; CHANGELOG states the pin decision and any re-pin.
3. [content] claims.json has cited claims (path, quote, line) that teardown can reopen a backlog item instead of closing it and under which condition; that with yolo on firstmate merges green work itself and what happens with yolo off; and that only a recorded PR or cleanup frees a place, with relaunches and secondmates never counted.
4. [content] model.json, rationales.json, findings.json and intro.json match the claims, reference only existing claim ids, and do not say a recorded PR frees a place for local-only ships or scouts, which hold it until cleanup.
5. [content] An outside reader opening the rebuilt index.html finds the three corrected behaviors, each citing the pinned commit, and the quoted lines exist at that commit.
6. [behavior] Run `cargo test` from the repository root: the study rules tests, including the new pin-consistency test (which also has a case showing a mismatched label is rejected), and the xtask site tests pass.
7. [behavior] Run `cargo run -p xtask -- site`: it succeeds with no unknown claim id error and renders index.html. `cargo clippy` and `cargo fmt --check` are clean.
8. [content] No secrets or personal data are written; the study is re-derived from the source, not patched with only the three reviewer points.

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

- Changing athena.toml, defaults.toml or workflows
- Studying anything in firstmate beyond work handling
- Changing the job pipeline outside the study rules check

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Check and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy, fmt must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim cites path and quote at the pinned commit.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the citations resolve at the pinned commit and mismatches are rejected.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Add a mechanism check for the mismatch, not just fix labels.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Pin consistency is enforced by test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Page and changelog written for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test and xtask site.
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
