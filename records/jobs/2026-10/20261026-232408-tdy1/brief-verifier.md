# Job 20261026-232408-tdy1: verify "Move the Host API client row to gh/client.rs"

## Request (verbatim)

> Priority: high
> Structure pass job G5 (docs/design/structure-pass.md, section 4). Move the Host API client row (caller-heavy: list the caller check for every moved name). No behavior change. Depends on 20261026-232359-msh3.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/client.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job G5: move the Host API client row of docs/design/structure-pass.md section 2.1 (Checks, NewPr, PrDraft, PrHost, CiRunState, MergeMethod(s), GhClient and its impls, parse_* helpers, classify_checks, the CommitHost impl) from gh/mod.rs into new gh/client.rs, with re-exports so public paths stay. Audience: maintainers and the verifier. Depends on job 20261026-232359-msh3 (G2, Transport row, gh/transport.rs); that job was dispatched with no candidate recorded and no findings, but transport.rs is already in the tree, so I treat it as landed and build on it.

Assumptions:
- Code survey: read crates/athena-core/src/gh/ (mod.rs, transport.rs, remote.rs, pages.rs, records.rs) and docs/design/structure-pass.md rows 42, 168, 269; the entry point is the gh module's public API, and no records, data files or validation allow-lists are added, so check_record_file and AnyRecord are not involved. Behavior is measured directly by running the existing test suite and a Grep of call sites.
- Job 20261026-232359-msh3 (G2) had no candidate changed files and no findings recorded; its scope (transport.rs, mod.rs, CHANGELOG.md) is not redone here. This job keeps whatever it left in the tree: gh/transport.rs is kept as is and not touched.
- The request names the new file gh/client.rs, while the design table names it gh/host.rs; the request wins.
- Moved names keep their visibility; only what the move forces (pub(super) or pub(crate) for items mod.rs still uses) may change, and any such change is visible in the diff.
- The verifier checks caller safety by running Grep itself over crates/ for each moved name, not from a producer report.

## Acceptance criteria

1. [content] crates/athena-core/src/gh/client.rs exists and holds the Host API client row (Checks, NewPr, PrDraft, PrHost, CiRunState, MergeMethod, MergeMethods, GhClient with its Host, PrHost and CiSource impls, the CommitHost impl, parse_ci_runs, parse_check_run, parse_ci_run, parse_merged_pr, parse_remote, classify_checks, failing_test_name); `git diff` shows these definitions removed from gh/mod.rs with the same text and no body edits.
2. [content] Public paths are unchanged: gh/mod.rs has `mod client;` and re-exports every previously public moved name, so a Grep of crates/ and xtask/ for `gh::(GhClient|PrHost|Checks|CiRunState|MergeMethod|parse_)` finds the same paths as before and each resolves; no caller outside gh/ has an edited import.
3. [content] Caller check by Grep: for every moved name, a Grep of crates/ and xtask/ for the name finds no use that fails to resolve; in particular GhClient, PagesHost, land_pr( and landed_commit sites in main.rs, display.rs, reconcile.rs, trailer_stop.rs and external_calls.rs are untouched in the diff.
4. [content] Existing tests pass with only import lines edited: `git diff` on test code shows only `use` line changes, and the tests the_client_runs_only_gh_api_with_rest_paths, the_merge_method_is_chosen_from_what_the_host_allows and checks_are_classified_from_the_two_replies still exist under the same names (test list unchanged).
5. [content] Any visibility widening in the diff (pub(super), pub(crate)) applies only to items mod.rs or sibling modules still use, and no signature or body changes appear.
6. [content] CHANGELOG.md has a new entry describing the Host API client move to gh/client.rs as a no-behavior-change refactor.
7. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass and the test count equals the count before the change.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/transport.rs (modify)
- crates/athena-core/src/gh/pages.rs (modify)
- crates/athena-core/src/gh/remote.rs (modify)

## Out of scope

- Moving any other row of structure-pass.md section 2.1 (G3, G4, G6, G7 onward)
- Updating docs paths such as docs/system-facts.md or docs/architecture.md
- Any behavior, signature or visibility change beyond what re-exports need
- Editing introspect.rs or external_calls_guard.rs
- Splitting the job by trait unless the producer reports it cannot finish
- Redoing job 20261026-232359-msh3 (gh/transport.rs stays as is)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust module move.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the checks and Greps callers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the one row moves.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria rest on the test run and Grep of call sites.
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

Base e83ccd588f4b → candidate e16eae6df5c6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   8 +
 crates/athena-core/src/gh/client.rs | 678 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/mod.rs    | 679 +-----------------------------------
 3 files changed, 697 insertions(+), 668 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
