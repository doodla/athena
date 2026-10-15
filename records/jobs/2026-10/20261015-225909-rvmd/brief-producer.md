# Job 20261015-225909-rvmd: Complete the firstmate study and rebuild the explainer page

## Request (verbatim)

> Study firstmate's work handling again against the study's pinned source so the study is complete and right, then rebuild the explainer page from it. An independent reader who checked the page against that source found the study misses: teardown can reopen a backlog item instead of closing it; with yolo on firstmate merges green work itself; only a recorded PR or cleanup frees a place, and local-only ships, scouts, relaunches and secondmates are handled differently.

## Reading (accepted by the user)

Re-study firstmate at the pinned commit 6407052f to add the missed behaviors: teardown reopening a backlog item, yolo self-merge, which events free a place, and the local-only, scout, relaunch and secondmate paths. Update the study records in studies/firstmate-work-handling, then regenerate the page with `cargo run -p xtask -- site`. The audience is a newcomer reading the page and the independent reader checking it against the source.

Assumptions:
- The pinned commit stays 6407052f4c521e39401ce2305dfa45d201249827; the pin is not changed.
- The page is generated into target/site by xtask from the study files and is not committed, so rebuilding means the study changes render correctly.
- The reader's findings are leads to be checked against the source, not facts; each is added only if the source supports it, and corrected if it does not.
- xtask/src/site.rs changes only if the new records need rendering the page lacks.

## Acceptance criteria

1. [content] The study records state that teardown can reopen a backlog item rather than close it, with a citation to the pinned source path and quote.
2. [content] The study records state that with yolo on, firstmate merges green work itself, and that otherwise it does not, each with a source citation.
3. [content] The study records state that only a recorded PR or a cleanup frees a place, with citations.
4. [content] The study records separate handling of local-only ships, scouts, relaunches and secondmates, each a cited claim.
5. [content] Each new claim is linked from findings and the model, and validation.json is updated to match the records.
6. [behavior] Run `cargo test` and `cargo clippy`: the kept study test in crates/athena-cap-study/src/rules.rs passes, so every cited path and quote is checked against the pinned source, and clippy is clean.
7. [behavior] Run `cargo run -p xtask -- site`: it succeeds, and the generated page shows the new behaviors in terms a newcomer can follow. Each claim on the page traces to a cited study record.

## Expected deliverables

- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/rationales.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the pinned commit
- Changing Athena's study rules or athena.toml
- Studying firstmate behavior beyond work handling
- Redesigning the explainer page layout

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every new claim cites a pinned-source path and quote.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Cargo test and the site build are run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the stated behaviors in the records and on the rendered page.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the study records and the page they feed change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The study is re-derived from the source, not just patched for the five listed gaps.

## Pinned source

Athena resolved the studied repository to one commit before this job began:

- url: https://github.com/kunchenguid/firstmate.git
- ref: 6407052f4c521e39401ce2305dfa45d201249827
- sha: 6407052f4c521e39401ce2305dfa45d201249827

Cite this url, ref and full SHA exactly as given; they are real, not placeholders. The tree is mounted read-only in your working directory.

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
