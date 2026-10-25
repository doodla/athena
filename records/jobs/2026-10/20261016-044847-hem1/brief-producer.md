# Job 20261016-044847-hem1: Make planning a real per-item pass that flags underspecified requests

## Request (verbatim)

> Planning is still not fleshed out properly: the verifier of 20261016-035342-px6g found the refusal gate rarely fires because every job gets a default rung and a request's first line counts as acceptance. Make the plan fields come from an actual planning pass that reads each open item against the brief, the records and the code, so goal, rung, acceptance, open questions, dependencies and touched paths are real, flag items that are underspecified instead of defaulting them, and let me see the coordinated plan.

## Reading (accepted by the user)

Replace the default-filled plan fields (rung defaulting to Implementation, first line as acceptance) with a planning pass that reads each open item against the brief, records and repository code. Items it can't specify are flagged and held back. The coordinated plan, with groups, order, flags and open questions, is viewable on demand. The audience is the user and the verifier, who judge whether the refusal gate now fires on real, vague requests.

Assumptions:
- Goal and audience: the user wants the planning gate to be meaningful, so vague items are held back and flagged rather than run on defaults. The audience is the user, who sees the plan, and the verifier of this job.
- 'Actual planning pass' is read as a deterministic pass, with no new model role, over each item's request text, the brief, the job and proposal records, and file existence in the repository. A model-backed planner would be a new agent role and is out of this change. If the user wants one, they should say so.
- Rung comes from the steward proposal if present, else is inferred from the request, reading and brief. If inference is not confident the item is flagged, not defaulted.
- The view of the plan is a CLI command that prints text and JSON like other commands, plus the drain report. It does not need a web page.
- The wiring of plan::trace and GapLog to the records is optional follow-up unless it is cheap. Tests for open_questions (the nit) are included.
- The earlier verifier's note about display.rs being off-list is handled by listing display.rs here.

## Acceptance criteria

1. [behavior] Run `cargo test`: a new test shows an unread request with no bullets and no path-like words (for example 'make it better') is flagged underspecified. The flag names the missing field, and the item is not in Plan::order.
2. [behavior] Run `cargo test`: a new test shows a request's first line is no longer copied into acceptance. Only explicit acceptance bullets, or a reading's criteria, fill it.
3. [behavior] Run `cargo test`: a new test shows a non-proposal job's rung is inferred from its request or reading (for example docs/design work vs code), or flagged as missing. It is never silently set to Implementation, and an item with no determinable rung is refused.
4. [behavior] Run `cargo test`: tests assert that open_questions are filled and checked for a read and an unread request. Questions include ambiguity found by the pass, such as a touched path that does not exist in the repository and an unresolved referenced job.
5. [behavior] Run `cargo test`: a test shows two open items that name the same path get a dependency or are put in different groups. Items with disjoint touches are grouped together.
6. [behavior] Run `cargo test`: a drain test using RealSteps over a records fixture with one specified and one vague queued item shows the vague one in Report::refused with a reason, and not run as a job. The specified one runs.
7. [behavior] A user-invokable command (a new `athena plan` or equivalent, exercised with `cargo run`) prints the coordinated plan: order, parallel groups, each item's goal, rung, acceptance, open questions, dependencies and touches, and the flagged items with reasons. When nothing is open it says so explicitly instead of printing an empty success.
8. [content] display.rs, main.rs and any other files touched outside plan.rs, reading.rs and drain.rs are named in the change's docs or decision entry. The tradeoff text in docs/decisions.md about the gate rarely firing is updated to describe the new behavior.
9. [content] The change stays on the rust toolchain: `cargo clippy` is clean and `cargo fmt --check` passes. It uses no new secrets or personal data.
10. [content] The docs say plainly whether the planning pass is deterministic code or calls a model, and whether the plan::trace and GapLog audit is wired to the records or still follow-up.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Running plan groups concurrently; the drain stays one job at a time.
- Moving compute to CI or any design work after the planning change.
- A model-backed planner role or new prompts.
- Changes to athena.toml, defaults.toml, workflows or other governance files.
- Editing records/ or owner files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new passing tests, clean clippy and fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that vague items are refused and specified ones run, not that types exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The plan command must say explicitly when nothing is open or everything is refused.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix removes the defaults that disabled the gate (rung default, first-line acceptance), not just one instance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision entry are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the planning pass, the gate and the plan view.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The plan command is exercised through cargo run.
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
