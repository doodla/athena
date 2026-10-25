# Job 20261014-012431-w4cj: Add typed decision view and confirmed supersession list to backtest

## Request (verbatim)

> Build step 2 of the build order in docs/design/decision-sets.md section 8: the typed decision view and the supersession parse (section 6, steps 2-3), with a user-confirmed list of supersession and amend hits; no briefs change. I confirm D11's partial revision by D39 (line 322) as a known hit, so the list includes it. Make 'athena backtest' use the confirmed list, so condition (a) passes on the real log.

## Reading (accepted by the user)

Build §8 step 2 of decision-sets.md: a typed read-only view of D01–D90 built from docs/decisions.md and the supersession/amend parse, checked against a committed user-confirmed list that includes D11 amended_by D39 (line 322). `athena backtest` condition (a) then compares its hits to that list instead of the hard-coded two pairs. No briefs, defaults, or decisions.md prose change.

Assumptions:
- Step 2 means all D01–D90 get a derived view (id, status, superseded_by, amended_by) only; kind, phase, paths, enforcement are not assigned per entry, since §6 step 2's per-entry judgement is a separate job.
- The confirmed list is a committed data file read at runtime (docs/decision-links.toml) or an equivalent embedded constant; I could not read records/ so I rely on the 24 hits reported in §9 and the grep of decisions.md.
- Per §6 step 3, D11 and every partial revision is amended_by; only wholly replaced decisions are superseded. D46/D79 is treated as full per its text, unless the producer finds otherwise, and I do not decide other 'Amends' extents beyond the log's wording.
- 'Condition (a) passes on the real log' means the parse/list check; conditions (b)-(d) depend on records/ and may still fail, and that is out of scope.
- The 'user-confirmed' list is built from the parse and shown in the change for the user to confirm; the only hit the user has explicitly confirmed is D11/D39.

## Acceptance criteria

1. [content] A typed decision view exists in athena-cli: one record per entry heading of docs/decisions.md with id, status (active/superseded), superseded_by and amended_by derived from the parse plus the confirmed list; the log's prose is untouched.
2. [content] A committed confirmed list names every supersession and amend hit with its line, older id, newer id, and confirmed status; it includes D11 amended_by D39 at line 322 (status stays active) and D46→D79 and D87→D88 (D87 amended_by, not superseded), and prose-only hits from D48 on.
3. [content] The backtest's condition (a) compares parsed hits with the confirmed list (hits missing from the list or listed but not found are findings) and no longer uses the hard-coded EXPECTED_SUPERSESSIONS pair list.
4. [content] No brief, defaults.toml, BRIEF.md, or docs/decisions.md prose is modified; no governance file is changed.
5. [content] decision-sets.md §8 step 2 is marked built and says what the confirmed list is; CHANGELOG notes the change.
6. [behavior] Run `cargo test -p athena-cli`: new tests pass, including one that parses the repo's docs/decisions.md and finds no condition (a) finding, one that D11 is active with amended_by D39, and one that an unlisted hit is reported.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.

## Expected deliverables

- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/decisions.rs (create)
- crates/athena-cli/src/lib.rs (modify)
- docs/decision-links.toml (create)
- crates/athena-cli/tests/backtest.rs (modify)
- docs/design/decision-sets.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/Cargo.toml (modify)

## Out of scope

- Changing any brief or set selection (step 3)
- Editing docs/decisions.md prose or adding headers to it
- Host-error record, watcher, Jev, defaults.toml or DefaultsModel changes
- Fixing backtest conditions (b)-(d) or D87's signal
- Assigning kind, phase, enforcement per entry

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New view and list check get unit tests; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the view, the list, and the backtest wiring.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing serde and toml dependencies and the existing parser rather than a new crate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the design doc update are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Replace the hard-coded expected pairs that caused the D11 finding with a confirmed list, rather than adding one more special case.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior verified by running cargo test on the real decisions.md.
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
