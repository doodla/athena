# Job 20261029-023338-pt6z: verify "Move Early PR and CI wait row to gh/early.rs without a cycle"

## Request (verbatim)

> try again on 20261026-232411-2zv3

## Reading (accepted by the user)

Retry of job 20261026-232411-2zv3 (G6): move the Early pull request and CI wait row from gh/mod.rs into gh/early.rs with text unchanged, but keep EARLY_RUN_WAIT in client.rs beside PrHost::early_run_wait so no client↔early cycle exists. Audience: the maintainers and the verifier judging the structure pass.

Assumptions:
- Code survey: read crates/athena-core/src/gh/mod.rs (EARLY_RUN_WAIT at line 62, wait_for_early_run at ~437 uses job.host.early_run_wait()) and crates/athena-core/src/gh/client.rs (line 5 imports super::EARLY_RUN_WAIT; line 73-74 PrHost impl returns it). Siblings: transport.rs, remote.rs, records.rs, pages.rs, client.rs. The behavior is measured by cargo test --locked and --list, not through a stand-in.
- Job 20261026-232411-2zv3 finding major (client.rs:5/74 ↔ early.rs cycle) is fixed by keeping EARLY_RUN_WAIT defined in client.rs (the host row), re-exported from gh/mod.rs so the public path gh::EARLY_RUN_WAIT holds; early.rs imports it from client if needed. This deviation is recorded in docs/design/structure-pass.md.
- Finding minor (early.rs depends on remote.rs and LandPr, which the section 3.1 table omits) is addressed by a note in docs/design/structure-pass.md that early uses remote and LandPr until G7, so G7 must not make landing use early in the opposite direction in a way that cycles.
- Finding nit (CHANGELOG says mod.rs drops crate::pr imports, though two are kept under cfg(test)) is fixed by an accurate CHANGELOG G6 entry.
- The candidate's three changed files (CHANGELOG.md, new gh/early.rs, gh/mod.rs) are dropped as such; the retry starts from base and redoes them, keeping the same approach with the cycle fix.
- Depends on G5 (20261026-232408-tdy1), already landed. The earlier proposed dependency-direction test was a suggestion for prevention; the user's request is only to retry, so a new test is out of scope (the test list must stay unchanged).

## Acceptance criteria

1. [content] gh/early.rs exists and holds replace_early, wait_for_early_run, name_failure, Waited, wait_for_checks, EarlyPr, open_early_pr, https_github_repository, Athena::early_pr, early_pr_to and the EARLY_RUN_* constants other than EARLY_RUN_WAIT; git diff shows them removed from mod.rs with identical text apart from visibility keywords.
2. [content] Grep of crates/athena-core/src/gh/client.rs, transport.rs, remote.rs, pages.rs, records.rs finds no `early` reference (no `super::early`, no import of a name defined in early.rs); early.rs imports nothing from a higher row, and no two sibling modules use each other circularly.
3. [content] gh/mod.rs declares `mod early;` and re-exports every previously public moved name (EarlyPr, open_early_pr, EARLY_RUN_WAIT), so Grep of `gh::(EarlyPr|open_early_pr|EARLY_RUN_WAIT)` across crates/ finds the same occurrences as at base and no caller outside gh changed.
4. [content] Existing tests are unmoved and unedited except import lines; no test is added or removed.
5. [behavior] Run `cargo test --locked -- --list` and compare to base: the test list is unchanged; `cargo test --locked` passes, including land_pr_waits_for_the_early_run_before_pushing and land_pr_replaces_the_early_head_pinned_and_reuses_the_early_pull_request.
6. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`; both pass.
7. [content] CHANGELOG.md has an accurate G6 entry listing every moved name (including private constants), every visibility change, that EARLY_RUN_WAIT stays in client.rs, and the retained cfg(test) crate::pr imports in mod.rs.
8. [content] docs/design/structure-pass.md records the EARLY_RUN_WAIT placement deviation and the early→remote/LandPr dependency note for G7.
9. [content] Each moved name has a caller check (a Grep of its uses) shown in the diff-verifiable form: the same occurrences resolve through gh re-exports or inside gh.

## Expected deliverables

- crates/athena-core/src/gh/early.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/client.rs (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving any other row (G7 Landing, G8, R jobs)
- Adding a dependency-direction test or xtask check (the test list must stay unchanged)
- Updating docs paths such as docs/system-facts.md or docs/architecture.md
- Any behavior or signature change beyond visibility needed for re-exports and cross-module calls
- Editing introspect.rs or external_calls_guard.rs
- Redoing G5 or earlier gh jobs; transport.rs, pages.rs, records.rs, remote.rs stay as is

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Full cargo test, clippy and fmt required.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs test list comparison and checks.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Cycle fixed by placing the shared constant in the lower row.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check test results and grep outcomes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and design note written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base db95acc7a162 → candidate c0d79eea1c2a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  17 ++
 crates/athena-core/src/gh/client.rs |   6 +-
 crates/athena-core/src/gh/early.rs  | 363 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/mod.rs    | 362 ++---------------------------------
 docs/design/structure-pass.md       |   7 +
 5 files changed, 404 insertions(+), 351 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked -- --list` and compare to base: the test list is unchanged; `cargo test --locked` passes, including land_pr_waits_for_the_early_run_before_pushing and land_pr_replaces_the_early_head_pinned_and_reuses_the_early_pull_request.; Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`; both pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
