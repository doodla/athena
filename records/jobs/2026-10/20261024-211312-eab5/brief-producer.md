# Job 20261024-211312-eab5: Add legacy rule for pre-C0 commits, with legacy-exempt record builder

## Request (verbatim)

> Build job 5, part 2, of the records-history plan (docs/design/records-history.md section 4 and 10, decision D111): the legacy rule. For main's commits up to and including a cutover commit C0, the attribution check delegates to the existing history-check logic (athena-gate) unchanged, except that the exempt list is read from a legacy-exempt record on athena/records (the record type exists in athena-data history.rs) instead of athena.toml's [history] exempt; commits after C0 keep the records-only rule from part 1. Add a way to produce that legacy-exempt record from athena.toml's current list and comments (the six entries plus 02837e0, each with its reason), without writing it to the real branch yet. Tests: each of the seven named commits (49c6ac1, 64ecdc9, ee4b0dd, f2a8453, 5a73478, 39962ed, 02837e0) is judged as section 4 says when its legacy-exempt entry is present, and fails as today when it is absent; a legacy-exempt record that later gains an entry is refused (append-only). C0 is a parameter; nothing chooses it yet.

## Reading (accepted by the user)

Adds a legacy rule to the attribution check. Commits up to and including a parameter C0 go to the existing history check (athena-gate) with its exempt list read from a legacy-exempt record on athena/records. Commits after C0 keep the part 1 records-only rule. Also adds a function that builds that record from athena.toml's seven entries and their comment reasons, without writing it to the real branch. Audience: the owner and the verifier, who judge by the seven-commit tests.

Assumptions:
- Goal: pre-cutover commits are judged by today's history logic, with the exempt list taken from the records branch rather than athena.toml; audience is the owner and verifier.
- The history check's exempt source becomes a parameter (for example an Option<Vec<String>> override of tip_config.history.exempt in check_history, or a new variant that takes the list). Existing check_history behavior is unchanged when no override is given.
- The legacy rule is a new entry point in crates/athena-gate/src/attribution.rs or a new sibling module. It takes C0 as an argument, reads legacy-exempt from the records tip, and judges commits from the root through C0 with the old logic and C0+ with the existing walk. Nothing in the CLI or CI selects a C0.
- 'Judged as section 4 says' means: with the entry present, the commit is excused by the legacy list; with it absent, it fails exactly as the history check does today. Tests use small constructed repos in which commits play the seven shas, since the real shas cannot be used.
- The producer function takes athena.toml's current [history] exempt list plus a table of reasons. The reasons are copied from the athena.toml comments, which cover six entries plus 02837e0. It returns a LegacyExempt record (a record JSON string or a RecordsWriter Change) and writes nothing to the real branch. Path is legacy-exempt.json per section 4.
- Append-only refusal is checked at the records-writer level: a second write to legacy-exempt.json with different bytes (an added entry) is refused by RecordsWriter::append. The legacy rule also rejects, with a finding, a records history in which legacy-exempt.json was ever changed.
- athena.toml is not edited: the [history] exempt table stays until stage 2, so no governance file changes.
- No existing public signature is broken if avoidable, because part 1 and other callers use check_attribution and check_history.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-gate`. For each of the seven commit ids named in the request there is a test in which the commit is a pre-C0 hand change excused when its legacy-exempt entry is present. A second test fails with the same violation as today's check_history when the entry is absent. All pass.
2. [behavior] Run `cargo test` for the workspace. A test shows that a legacy-exempt record that later gains an entry is refused as append-only, and that the original record is unchanged.
3. [behavior] Run `cargo test -p athena-gate`. A test with C0 as a parameter shows that commits after C0 are judged only by the records-only rule from part 1 (an unattributed post-C0 commit is a finding, and a pre-C0 commit is never judged by attribution). A second test shows that a commit exactly at C0 is judged by the legacy rule.
4. [behavior] Run `cargo test`. A test builds the legacy-exempt record from the built-in or repository athena.toml's current [history] exempt list. The record has exactly seven entries, the six listed plus 02837e0, each with a non-empty reason taken from the athena.toml comments. It validates under LegacyExempt::validate, and nothing is written to any git ref.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
6. [content] The existing history-check logic is reused, not copied: the exempt list is the only thing that differs, and check_history's existing tests pass unchanged.
7. [content] athena.toml, defaults.toml and the workflows are unchanged, and no function chooses a value for C0. The module docs say C0 is a caller-supplied parameter and cite design section 4 and decision D111.
8. [content] The attribution.rs module doc comment, which says the legacy rule will come later, is updated to describe the rule as built.

## Expected deliverables

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/legacy.rs (create)
- crates/athena-data/src/history.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## Out of scope

- Writing legacy-exempt.json to the real athena/records branch
- Choosing a value for C0, or any CLI or CI wiring that selects one
- Removing or emptying [history] exempt in athena.toml (stage 2)
- Removing records/ from main's tree, or the import-manifest and checkpoint work (jobs 6 to 8)
- Changing the part 1 records-only rule for post-C0 commits

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust, in athena-gate and possibly athena-data.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for each of the seven commits, the append-only refusal, and C0 handling; cargo test and clippy must be clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test, clippy and fmt --check commands the verifier can run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check judged outcomes (excused versus failing, refused rewrite), not that types exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the legacy rule and its record builder; the CLI, CI and branch writes are excluded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The exempt source is parameterised in the one history check rather than duplicating its logic.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing LegacyExempt record type, RecordsWriter append-only enforcement and check_history; builds nothing new where they suffice.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Append-only is enforced by the writer and tested, not left to prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite design section 4 and D111 and give the reason for each exempt entry.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Reasons come from public athena.toml comments; no personal data is written.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the legacy rule's effect on the seven commits, not the work done.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A criterion requires the record to contain exactly seven entries, so an empty list cannot pass.
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

Run `cargo test -p athena-gate`. For each of the seven commit ids named in the request there is a test in which the commit is a pre-C0 hand change excused when its legacy-exempt entry is present. A second test fails with the same violation as today's check_history when the entry is absent. All pass.
Run `cargo test` for the workspace. A test shows that a legacy-exempt record that later gains an entry is refused as append-only, and that the original record is unchanged.
Run `cargo test -p athena-gate`. A test with C0 as a parameter shows that commits after C0 are judged only by the records-only rule from part 1 (an unattributed post-C0 commit is a finding, and a pre-C0 commit is never judged by attribution). A second test shows that a commit exactly at C0 is judged by the legacy rule.
Run `cargo test`. A test builds the legacy-exempt record from the built-in or repository athena.toml's current [history] exempt list. The record has exactly seven entries, the six listed plus 02837e0, each with a non-empty reason taken from the athena.toml comments. It validates under LegacyExempt::validate, and nothing is written to any git ref.
Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
The existing history-check logic is reused, not copied: the exempt list is the only thing that differs, and check_history's existing tests pass unchanged.
athena.toml, defaults.toml and the workflows are unchanged, and no function chooses a value for C0. The module docs say C0 is a caller-supplied parameter and cite design section 4 and decision D111.
The attribution.rs module doc comment, which says the legacy rule will come later, is updated to describe the rule as built.

## Report

End with the producer report as your structured output.
