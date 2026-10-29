# Job 20261026-232411-2zv3: verify "Move the Early pull request and CI wait row to gh/early.rs"

## Request (verbatim)

> Priority: high
> Structure pass job G6 (docs/design/structure-pass.md, section 4). Move the Early pull request and CI wait row. No behavior change. Depends on 20261026-232408-tdy1.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/early.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job G6: cut the Early pull request and CI wait row (EARLY_RUN_* constants, replace_early, wait_for_early_run, name_failure, Waited, wait_for_checks, EarlyPr, open_early_pr, https_github_repository, Athena::early_pr, early_pr_to) from gh/mod.rs into new gh/early.rs, text unchanged, re-exported so public paths hold. Follows job 20261026-232408-tdy1 (G5, landed, passed); its two findings were CHANGELOG inaccuracies, addressed here by writing this entry accurately.

Assumptions:
- Goal: a pure move for maintainers reviewing the structure pass; the verifier judges the diff as a move with no behavior change.
- Job 20261026-232408-tdy1 (G5): its client.rs, mod.rs and CHANGELOG.md work is kept as landed (client.rs is not edited here). Its findings were minor/nit CHANGELOG inaccuracies (visibility claim missed TEST_NAME_LIMIT and wrongly named mod.rs as a caller; CI_CHECK_STEP not listed). They do not recur in code; this job's CHANGELOG entry must list every moved name and every visibility widening exactly, including private constants.
- Code survey: read docs/design/structure-pass.md sections 2.1, 4 and 5 and listed crates/athena-core/src/gh/ (mod.rs, client.rs, transport.rs, pages.rs, records.rs, remote.rs). The row's items are named in section 2.1; early.rs may depend only on host, client/transport per the module table; no lower module may name early.
- land_pr and publish remain in mod.rs until G7, so items they call (wait_for_checks, wait_for_early_run, EarlyPr, open_early_pr, replace_early) may need pub(super); only such widening is allowed and must be listed.
- Existing tests stay in mod.rs; only their import lines may change.

## Acceptance criteria

1. [content] crates/athena-core/src/gh/early.rs exists and holds the Early row items (EARLY_RUN_* constants, replace_early, wait_for_early_run, name_failure, Waited, wait_for_checks, EarlyPr, open_early_pr, https_github_repository, Athena::early_pr, Athena::early_pr_to); git diff shows them removed from mod.rs with identical text (moved-text comparison).
2. [content] gh/mod.rs declares the early module and re-exports every previously public moved name, so Grep of `gh::(EarlyPr|open_early_pr|EARLY_RUN_WAIT)` across crates/ finds the same occurrences and no caller outside gh changed.
3. [content] Existing tests are unmoved and unedited except import lines in the diff; no test is added or removed.
4. [behavior] Run `cargo test --locked -- --list` and compare to the base: the test list is unchanged; `cargo test --locked` passes, including land_pr_waits_for_the_early_run_before_pushing and land_pr_replaces_the_early_head_pinned_and_reuses_the_early_pull_request.
5. [behavior] `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` pass.
6. [content] No lower-row module (transport, remote, client, pages, records) names early; early.rs uses only host/client/transport siblings; no circular use.
7. [content] CHANGELOG.md has a G6 entry listing every moved name (including private constants) and every visibility change (any pub(super) widenings) accurately; the final report lists the caller check for each moved name.

## Expected deliverables

- crates/athena-core/src/gh/early.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/client.rs (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving any other row (G7 Landing, G8, R jobs)
- Updating docs paths such as docs/system-facts.md or docs/architecture.md
- Any behavior or signature change beyond visibility needed for re-exports and cross-module calls
- Editing introspect.rs or external_calls_guard.rs
- Redoing G5 (client.rs) or earlier gh jobs; transport.rs, pages.rs, records.rs, remote.rs stay as is

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One row moved.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs fmt, clippy, test and list comparison.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Existing early/land_pr tests exercise moved code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7f205eef076a → candidate 75475189e821. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |  13 ++
 crates/athena-core/src/gh/early.rs | 365 +++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/mod.rs   | 356 +-----------------------------------
 3 files changed, 387 insertions(+), 347 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
