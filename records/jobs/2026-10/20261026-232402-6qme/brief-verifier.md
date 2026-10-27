# Job 20261026-232402-6qme: verify "Move Git remote and Pages rows out of gh/mod.rs into submodules"

## Request (verbatim)

> Priority: high
> Structure pass job G3 (docs/design/structure-pass.md, section 4). Move the Git remote and Pages rows. No behavior change. Depends on 20261026-232356-ds51.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/remote.rs, crates/athena-core/src/gh/pages.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure-pass job G3: move Remote trait, GitRemote and helpers into gh/remote.rs, and the Pages items (PAGES_WORKFLOW, PagesRun, PagesHost and its GhClient impl, PagesReport, wait constants, wait_pages_run, watch_pages) into gh/pages.rs. Text is moved verbatim, gh/mod.rs re-exports so public paths are unchanged, and a CHANGELOG entry is added. The audience is the verifier checking the section 5 proof.

Assumptions:
- Job G1 has landed so crates/athena-core/src/gh/mod.rs exists.
- Visibility of moved private items may widen to pub(super)/pub(crate) only as needed for the split.
- Docs still citing gh.rs are left to a later job (G1 verification noted this; section 5 rule 7).

## Acceptance criteria

1. [content] gh/remote.rs holds the Remote trait, GitRemote, its impl and helpers used only by them; gh/pages.rs holds PAGES_WORKFLOW, PagesRun, PagesHost, the GhClient impl of PagesHost, PagesReport, the PAGES_* wait constants, wait_pages_run and watch_pages. Moved text is identical to the old text apart from visibility and import lines (git diff shows moved blocks unchanged).
2. [content] gh/mod.rs declares the remote and pages modules and re-exports every previously public name with pub use, so gh::Remote, gh::GitRemote, gh::PagesHost, gh::PagesReport, gh::watch_pages and other public paths are unchanged.
3. [content] Existing tests, including after_a_merge_the_branch_is_resynced_then_pages_watched_then_code_health_checked, are edited only on import lines; no test is added, removed or renamed.
4. [content] No file outside crates/athena-core/src/gh/ names a moved item by an old module path (gh::remote::... or gh::pages::... would be new internal paths only); the verifier confirms with a Grep over crates/ for those paths and for moved names, finding every caller still uses the gh:: re-export.
5. [content] CHANGELOG.md has an entry for the G3 move stating behavior is unchanged.
6. [behavior] Run cargo test --locked -- --list before (git stash-free: compare against the base via git show/diff of test names) and after; the lists of test names are identical.
7. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked; all three pass.

## Expected deliverables

- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/remote.rs (create)
- crates/athena-core/src/gh/pages.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Updating docs that cite gh.rs
- Moving other gh rows (other G jobs)
- Any behavior change
- Editing introspect.rs or the external_calls_guard

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria
- small: Prefer one focused change; size is a design choice, not a wish list. Here: only the two rows move
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: verifier greps and compares test lists
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: test-list comparison and grep of callers
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: n/a pure move; re-exports keep paths
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 406b84571071 → candidate 96faa8ef1a27. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   6 +
 crates/athena-core/src/gh/mod.rs    | 277 +-----------------------------------
 crates/athena-core/src/gh/pages.rs  | 198 ++++++++++++++++++++++++++
 crates/athena-core/src/gh/remote.rs |  83 +++++++++++
 4 files changed, 294 insertions(+), 270 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1397 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked; all three pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
