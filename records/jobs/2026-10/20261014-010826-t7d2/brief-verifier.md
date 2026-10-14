# Job 20261014-010826-t7d2: verify "Replace the non-Rust file ledger check with a code-in-other-languages ch"

## Request (verbatim)

> Simplify the non-Rust file check, from the BRIEF.md addendum '2026-10-14: Common sense over rigid rules'. The intent of BRIEF.md A4 is that all code is Rust. Today xtask's every_tracked_file_is_rust_or_a_listed_kind refuses any file kind not listed in docs/non-rust-files.md, so a JSON test fixture (data, not code) failed job pazf. Replace it with a check aimed at the intent: flag code in another language (by extension, such as .py, .js, .ts, .sh, .rb, .go, or by a shebang line), and allow data and documents (such as json, toml, yaml, md, txt) anywhere without a ledger entry. Keep a short named allow-list only for the non-Rust code that must exist (setup.sh and the CI workflow), each with its reason. Update docs/non-rust-files.md to describe the new rule, and keep a test that a stray script is still caught.

## Reading (accepted by the user)

Replace the xtask M20 test, which rejects any tracked file kind not listed in docs/non-rust-files.md, with a check that flags code in other languages (by extension or shebang). Data and documents pass anywhere. A short named allow-list covers setup.sh and the CI workflow, each with its reason, and docs/non-rust-files.md describes the new rule.

Assumptions:
- The allow-list lives in xtask/tests/policy.rs as a small named const, each entry with a reason string, and docs/non-rust-files.md repeats it as a table.
- The CI workflow (.github/workflows/ci.yml) is YAML, which is allowed as data, but is still named in the allow-list with its reason because the request asks for it.
- Code extensions flagged: py, js, ts, sh, rb, go, plus similar ones (mjs, cjs, jsx, tsx, pl, php, bash, zsh, java, c, cpp, h, lua, ps1). Any text file whose first line starts with #! is also flagged, unless allow-listed.
- The existing test names are kept (every_tracked_file_is_rust_or_a_listed_kind may be renamed, with docs/enforcement.md row M20 updated to match) so the M24 check that every named test exists still passes.
- Files under owner-private paths stay excluded via the existing athena_files() helper; records/ and the owner's files are not edited.
- The new docs text avoids the M19 boundary-scan words.

## Acceptance criteria

1. [content] xtask/tests/policy.rs no longer parses the docs/non-rust-files.md table to decide which files are allowed. The M20 check instead flags tracked files with a non-Rust code extension (at least .py, .js, .ts, .sh, .rb, .go) or a shebang first line.
2. [content] Data and document files (json, toml, yaml, yml, md, txt) pass anywhere in the tree with no ledger entry, including a JSON fixture under a test directory.
3. [content] A short named allow-list in the test has exactly setup.sh and the CI workflow (.github/workflows/ci.yml), each with a reason string.
4. [content] docs/non-rust-files.md describes the new rule (code in other languages is flagged, data and documents are allowed, the allow-list has setup.sh and the CI workflow with reasons) and no longer lists per-kind rows for data files. Its notes on the git hooks and the absent schemas and gitleaks files stay accurate.
5. [content] docs/enforcement.md row M20 names tests that exist in xtask/tests/policy.rs and describes the new rule, and the header comment in policy.rs matches.
6. [behavior] A test in xtask/tests/policy.rs shows that a stray script (for example tools/build.py, scripts/run.sh, or an extensionless file with a #!/bin/sh first line) is flagged. It also shows that a JSON fixture, a .md file and a .toml file are not flagged, and that setup.sh is allowed. Run `cargo test -p xtask` and confirm they pass.
7. [behavior] `cargo test` passes for the workspace (including M19 and M24 in policy.rs), `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.

## Expected deliverables

- xtask/tests/policy.rs (modify)
- docs/non-rust-files.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Editing BRIEF.md or its addendum.
- Changing athena.toml, defaults.toml, CI workflows or .gitattributes.
- Rewriting or removing any existing tracked file, or editing records/.
- Changing the M19 boundary scan or the M21 dependency-graph check.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check stays a Rust test in xtask, and its purpose is to enforce that all code is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new rule gets passing tests, including one that a stray script is still caught. Clippy and fmt must be clean.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule stays enforced by a test, not by prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is rewritten to match the intent (code versus data), not patched with another ledger row for JSON fixtures.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the test and its docs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The docs explain the rule and the reason for each allow-list entry for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 84952f547565 → candidate a8c950117d2f. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                    |   5 ++
 docs/decisions.md               |  16 ++++--
 docs/design/athena-v0.md        |  17 +++---
 docs/design/requirements-map.md |   2 +-
 docs/enforcement.md             |   2 +-
 docs/non-rust-files.md          |  37 +++++++------
 xtask/tests/policy.rs           | 114 +++++++++++++++++++++++++---------------
 7 files changed, 118 insertions(+), 75 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (394 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
