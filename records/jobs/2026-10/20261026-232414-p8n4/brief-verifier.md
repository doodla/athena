# Job 20261026-232414-p8n4: verify "Move the Landing row from gh/mod.rs into gh/landing.rs"

## Request (verbatim)

> Priority: high
> Structure pass job G7 (docs/design/structure-pass.md, section 4). Move the Landing row; its diff is only a move. No behavior change. Depends on 20261026-232405-wqeg and 20261026-232408-tdy1 and 20261026-232411-2zv3.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/landing.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass G7: cut the Landing row (work_branch, landed_commit*, squash_text_for, Poll, LandPr, PrReport, land_pr, confirm_existing_merge, refusal_text, publish, Athena::land_pr, Athena::after_merge) out of gh/mod.rs into new gh/landing.rs with identical text, re-exporting public names so gh:: paths are unchanged. Audience: the owner and verifier judging a pure-move diff. Builds on landed G4 (20261026-232405-wqeg), G5 (20261026-232408-tdy1) and the rejected G6 (20261026-232411-2zv3).

Assumptions:
- Code survey: read crates/athena-core/src/gh/mod.rs (mod declarations at lines 31-36, pub use re-exports, LandPr at 157, land_pr at 215, publish at 411), gh/early.rs (imports LandPr, GitRemote, Remote via super), gh/client.rs (EARLY_RUN_WAIT now lives at client.rs:18) and gh/records.rs. The entry point is the public gh:: API used by main.rs, drain.rs, run.rs, display.rs; there is no records validation or new data file, so check_record_file and AnyRecord do not apply.
- Prior job 20261026-232405-wqeg (G4, landed, pass): its two findings (stale test count 1397 vs 1617; reconcile.rs listed but unchanged) are addressed by not citing a fixed test count (compare the test list before and after) and not listing reconcile.rs. Its files records.rs, mod.rs, CHANGELOG.md are kept as landed; only mod.rs and CHANGELOG.md are touched again.
- Prior job 20261026-232408-tdy1 (G5, landed, pass): its CHANGELOG inaccuracies (api/repo_path visibility, TEST_NAME_LIMIT, CI_CHECK_STEP) concern the G5 entry; they are kept as is and not rewritten here. The G7 entry must itself list every moved name and every visibility change precisely, avoiding the same mistakes. client.rs is kept as landed.
- Prior job 20261026-232411-2zv3 (G6, rejected): its major finding (client<->early cycle through EARLY_RUN_WAIT) no longer applies in this base because EARLY_RUN_WAIT is now defined in client.rs and re-exported from mod.rs. Its minor finding (early.rs uses LandPr, so early->landing plus landing->early) applies: G7 must keep early.rs importing LandPr through super (the mod.rs re-export) with text unchanged, and the cycle must be noted in the CHANGELOG entry; if landing.rs cannot avoid a cycle with early.rs beyond that, report it. Its CHANGELOG nit (cfg(test) pr imports kept) is addressed by describing mod.rs imports accurately.
- Visibility widening is allowed only where a moved name is called from a sibling or from tests left in mod.rs (pub(super)); each such change is listed in the CHANGELOG and the report.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [content] A new crates/athena-core/src/gh/landing.rs holds the Landing row items (work_branch, landed_commit*, squash_text_for, Poll, LandPr, PrReport, land_pr, confirm_existing_merge, refusal_text, publish, Athena::land_pr, Athena::after_merge); git diff shows these as removed from gh/mod.rs and added to landing.rs with identical text apart from import lines and listed pub(super) visibility changes.
2. [content] gh/mod.rs declares `mod landing;` and re-exports every public moved name (land_pr, LandPr, PrReport, Poll, landed_commit*, etc.), so a Grep for `gh::(land_pr|LandPr|PrReport|Poll|landed_commit)` across crates/ and xtask/ shows callers unchanged and none edited.
3. [content] The diff changes no test function names or bodies: only `use` lines in test modules differ, and the set of #[test] fn names in the crate is unchanged (Grep count of `#[test]` before/after equal).
4. [content] gh/landing.rs imports only from host (client), early, records, remote, pages and transport siblings as in structure-pass.md 3.1; no lower-row module imports landing except via the mod.rs re-export, and the early.rs LandPr dependency is noted in CHANGELOG.md.
5. [content] CHANGELOG.md has a G7 entry that accurately lists every moved name, every visibility change and every import change kept in mod.rs, and the caller check (which files use each moved name) for each moved name.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all exit 0, including every land_pr_* and early test, with the same number of tests as the recorded base run (1617).
7. [behavior] Run `cargo test --locked -- --list` and confirm the list of gh:: test names is identical to the base (git show of base is not needed: the diff touches no test names).

## Expected deliverables

- crates/athena-core/src/gh/landing.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving any other row (G8 test modules, R jobs)
- Redoing G4, G5 or G6 or rewriting their CHANGELOG entries
- Updating docs that cite gh.rs (system-facts.md, architecture.md)
- Editing introspect.rs, external_calls_guard.rs, athena.toml, defaults.toml or workflows
- Any behavior change or test edits beyond import lines
- Edits to callers in main.rs, drain.rs, run.rs, display.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust module move.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and full cargo test criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single row move, no other changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs fmt/clippy/test and diffs the moved text.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check unchanged test list and passing land_pr tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Proof of section 5 criteria.
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

Base 8b0ca6f886bf → candidate b4eaeb2376a0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  33 ++
 crates/athena-core/src/gh/landing.rs | 569 ++++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/mod.rs     | 572 +----------------------------------
 docs/design/structure-pass.md        |   5 +
 4 files changed, 623 insertions(+), 556 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all exit 0, including every land_pr_* and early test, with the same number of tests as the recorded base run (1617).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
