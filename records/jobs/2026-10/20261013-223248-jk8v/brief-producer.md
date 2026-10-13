# Job 20261013-223248-jk8v: Make the deliverables list the single statement of file scope

## Request (verbatim)

> Retry the goal of 20261013-221358-r2d0 with a different approach; the verifier showed that text heuristics over criteria reject ordinary readings. Make the reading's deliverables list the single statement of file scope: (1) the reader prompt says scope is given only by the deliverables list and acceptance criteria never restate which files may change; (2) for change readings, code adds CHANGELOG.md to the deliverables when it is missing, with a 'changelog' default citing BRIEF.md's 'tagged releases and a changelog'; (3) the verifier's brief says a file outside the deliverables list is the scope finding, judged against the list, not against criterion wording. No text-matching of criteria. Stay within the reader's token budget without changing the meaning of existing prompt lines.

## Reading (accepted by the user)

Retry of r2d0 without any text-matching of criteria. The reader prompt says scope comes only from the deliverables list; code adds CHANGELOG.md to change readings when missing, with a 'changelog' default; the verifier's brief says a file outside the list is the scope finding, judged against the list.

Assumptions:
- The repository is at the state before r2d0: no check_scope_consistency exists in reading.rs.
- 'The verifier's brief' means the brief text built in briefs.rs. If the text actually lives in verifier.md, the sentence goes there.
- The added CHANGELOG.md deliverable is for readings whose capability is change, including declined ones only if they have capability change; study and null-capability readings are untouched.
- The 'changelog' default is added by code as a defaults_applied entry with its own text citing BRIEF.md. It does not need an entry in defaults.toml, which is governance and stays unchanged.
- The reader prompt gains one line and stays under budget without trimming other lines. If it does not fit, the lib.rs budget check will show it and the producer must pay with removals that keep every line's meaning.

## Acceptance criteria

1. [content] crates/athena-context/prompts/reader.md adds one line saying file scope is given only by the deliverables list and acceptance criteria never restate which files may change. Every existing line keeps its meaning and wording, and the added text fits the reader prompt budget enforced in crates/athena-context/src/lib.rs.
2. [content] In crates/athena-core/src/reading.rs, build_reading appends CHANGELOG.md (action modify, with a stated probability) to the deliverables of a reading with a capability of change when it is missing, and adds a defaults_applied entry with id 'changelog' whose text cites BRIEF.md's 'tagged releases and a changelog'. It leaves a reading that already lists CHANGELOG.md, and any non-change reading, untouched.
3. [content] No code inspects the wording of acceptance criteria to judge file scope. There are no keyword or window matches on criterion text, and no new rejection of readings based on their criteria.
4. [content] The verifier's brief (crates/athena-context/src/briefs.rs, or the verifier prompt if that is where the brief text lives) says a changed file outside the deliverables list is the scope finding, judged against the list and not against criterion wording.
5. [behavior] New unit tests cover: CHANGELOG.md added when missing; no duplicate when present; no addition for a non-change capability; the 'changelog' default added once; a reading whose criteria mention CHANGELOG.md, 'exclud' or 'not' is accepted unchanged; the verifier brief contains the scope sentence. Run: cargo test, cargo clippy --all-targets, cargo fmt --check. All pass.
6. [content] CHANGELOG.md gains an entry for this change, and docs/decisions.md records the call, including why the criteria-text heuristic was dropped.

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/verifier.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/src/lib.rs (modify)
- defaults.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Any text-matching or heuristic check of acceptance-criteria wording
- Rejecting readings at parse or admission time because of scope wording
- Rewording or shortening existing reader prompt lines
- Changing the producer prompt
- Changes to athena.toml, CI workflows, or other governance files

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The deliverables list becomes the single statement of scope, so criteria cannot contradict it, instead of patching each contradiction.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests cover the CHANGELOG addition and the false-positive cases that sank r2d0.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the prompt, the reading builder and the verifier brief.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The CHANGELOG default is enforced in code when the reading is built; the scope sentences are prompt text, marked as prose-only.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Details came from the repository: BRIEF.md line 337, reader.md, reading.rs build_reading, and the prompt token budget in lib.rs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments, changelog and decision entries are written for an outside reader.

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
