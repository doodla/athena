# Job 20261026-232431-dpem: verify "Move the Close and land row of run/mod.rs into run/close.rs"

## Request (verbatim)

> Priority: high
> Structure pass job R4 (docs/design/structure-pass.md, section 4). Move the Close and land row (after G7, so the caller check is made against one layout). No behavior change. Depends on 20261026-232423-hvyb and 20261026-232429-r147 and 20261026-232414-p8n4.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/close.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job R4: move close_dispatched, land_or_block, retry_landing, closing_subject, landing_body, build_text and the small formatters from run/mod.rs into new run/close.rs, text unchanged, public paths kept by re-exports. The audience is the maintainers and verifier judging the structure pass. Depends on R1 (20261026-232423-hvyb, abandoned and superseded; R1's result is in the base through its retry, so nothing is redone), R3 (20261026-232429-r147, landed; its planning.rs, pin.rs and mod.rs changes are kept as they are) and G7 (20261026-232414-p8n4, landed; its landing.rs and gh re-exports are kept and not touched). G7's findings concern gh CHANGELOG entries and don't apply here. R3's nit, that the per-name caller check is missing from the report, is addressed by listing it per name in the CHANGELOG entry.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs (close_dispatched at about line 2205, land_or_block 2403, retry_landing 2586, closing_subject 2731, landing_body 2764, build_text 2786), the run/ directory listing (mod.rs, pin.rs, planning.rs, send_back.rs) and docs/design/structure-pass.md sections 2.2, 3.2 and 4. The change is a pure move, so no resolution or probing order, record allow-list or new data file is involved. The behavior is measured directly by the existing tests and the three cargo checks.
- The row's current line range in mod.rs is found by name, since earlier moves shifted lines; items between them that belong to other rows stay put.
- Tests for moved functions stay in mod.rs or move with their function as R3 did, import lines only; helper dedup is not done.
- The retry_landing caller in main.rs resolves through the existing path, so main.rs is expected to need no edit; the earlier refused deliverable of a main.rs path is dropped.

## Acceptance criteria

1. [content] crates/athena-core/src/run/close.rs exists and holds close_dispatched, land_or_block, retry_landing, closing_subject, landing_body, build_text and the small formatters of the Close and land row, with bodies textually identical to the removed text in run/mod.rs (git diff shows only deletions from mod.rs matched by additions in close.rs, plus mod declaration, use lines, re-exports and pub(super) or pub(crate) visibility changes).
2. [content] Public paths are unchanged: closing_subject and build_text stay reachable at their old paths under run (re-exported from mod.rs), and Grep over crates/ and xtask/ finds no caller path that needed editing beyond import lines.
3. [content] Existing tests pass with only import lines edited: git diff shows no test function added, removed, renamed or changed in body, and the test list from cargo test --locked -- --list holds the same names, apart from the module path of any test module that moved with its function.
4. [content] CHANGELOG.md has an R4 entry under Unreleased saying no behavior change and listing, for each moved name, the callers found by search (including main.rs for retry_landing and the gh::land_pr use), so a reader can check each name.
5. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked from the repository root; all exit 0.
6. [content] Changes to athena.toml, defaults.toml and .github/workflows are absent from the diff; they are not needed.

## Expected deliverables

- crates/athena-core/src/run/close.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving any other row of run/mod.rs (producer, verifier, checks, candidate, verification)
- Splitting produce_and_verify
- Any behavior change, or test edits beyond import lines
- Deduplicating test helpers
- Editing athena.toml, defaults.toml or workflows
- Redoing R1, R2, R3 or G7, or rewriting their CHANGELOG entries
- Editing gh/ files and callers in main.rs or drain.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test checks are required criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One row moved, in one focused change.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test-list and text-identity criteria check that behavior is unchanged.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the cargo checks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry is written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d0f77767d31c → candidate 88c6af4535b7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  14 +
 crates/athena-core/src/run/close.rs | 591 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/mod.rs   | 578 +----------------------------------
 docs/design/structure-pass.md       |   5 +
 4 files changed, 612 insertions(+), 576 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked from the repository root; all exit 0.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
