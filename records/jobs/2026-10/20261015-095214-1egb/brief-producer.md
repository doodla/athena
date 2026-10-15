# Job 20261015-095214-1egb: Add outcome-check, no-empty-success and thrash-stop standing defaults

## Request (verbatim)

> Add standing defaults that the reading step applies without asking the user, unless the request says otherwise. (1) The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. (2) An empty queue or finished list never counts as success on its own. (3) If the same item fails twice in a loop, the loop stops and reports thrashing. Record these defaults as acceptance criteria in the reading. Show that a request for an explainer page gets them with no question to the user.

## Reading (accepted by the user)

Add three cross-cutting standing defaults (outcome check with a fresh-reader rating of at least 4/5, no success from an empty list, stop on thrashing after two failures) to defaults.toml. Make the reader record them as acceptance criteria without asking, and show with tests that an explainer-page request receives them. The audience is the user and the reader role.

Assumptions:
- Audience: the user and the reader role, who read the rendered defaults list; the explainer-page demonstration is a test, since the verifier cannot call a model.
- An explainer page is class docs, so the new defaults must apply to docs; '*' is chosen because the rules are cross-cutting.
- Enforcement of the 4-of-5 rating and the thrash stop in the loop code is not built; these are recorded as defaults and reader criteria, and marked prose-only for the loop mechanism.
- defaults.toml version stays at 1 unless the parser or tests require a bump.
- The three defaults come from the user's standing spec as stated in this request; source cites the request.

## Acceptance criteria

1. [content] defaults.toml gains three [[default]] entries with unique ids (outcome-check, no-empty-success, thrash-stop), each applies_to ["*"] (or classes that include docs), each with a source citing this request, and each statement's first sentence states the core rule, since render shows only that sentence.
2. [content] The outcome-check statement says: the outcome is the check behind the work, not the activity; for audience-facing quality a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. no-empty-success says an empty queue or finished list never counts as success alone. thrash-stop says that if the same item fails twice in a loop, the loop stops and reports thrashing.
3. [content] The reader prompt (crates/athena-context/prompts/reader.md) tells the reader to record applicable standing defaults of this kind as acceptance criteria and not to ask the user about them unless the request says otherwise.
4. [behavior] Run `cargo test -p athena-data`: a new test loads the repository defaults.toml, selects defaults for class docs (an explainer page request), and shows the three defaults appear in the rendered default list; the existing parse, duplicate and render tests still pass.
5. [behavior] Run `cargo test` for the workspace: a new test builds the reader brief for the request 'write an explainer page' and shows it contains the three defaults and the instruction to turn them into acceptance criteria, with no question field required; a rival check shows a request that says otherwise is not forced to carry them if the reader prompt supports that wording.
6. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Implementing loop code that actually computes the 4-of-5 rating or detects thrashing
- Changing existing defaults
- Editing records/ or owner files
- Calling a live model in tests

## User defaults that apply

- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Defaults are tested by parsing and rendering; loop enforcement is explicitly marked prose-only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show the explainer request receives the defaults, not that entries merely exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in athena-data; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change: three defaults, one prompt line, tests.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the defaults format and render behaviour from the repository without asking.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the tests that exercise the explainer case.
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
