# Job 20261018-060257-vmgy: verify "Fix directory crash in plan check and re-enable job planning"

## Request (verbatim)

> Fix a crash in job planning, then turn planning back on. The first planned job (20261017-204355-358k) crashed athena go: its plan listed the directory docs/, and Athena's own plan check (crates/athena-core/src/run.rs, the existence check that calls self.git.show_file(&p, path)) failed with 'git cat-file blob <base>:docs/ failed ... bad file'. Git::show_file (crates/athena-git/src/lib.rs) checks existence with cat-file -e, which succeeds for a tree, and then reads it as a blob. Plan: (1) show_file returns None for a path that is not a blob, so no caller can crash on a directory. (2) Athena's plan checks turn a planned path that is a directory, or is not a plain relative file path, into a major finding that sends the plan back, never an error. (3) In athena.toml set [job_plan] enabled = true again; it was turned off by hand after the crash. Acceptance: cargo test: show_file on a directory returns None with no error; cargo test: a fake plan that lists 'docs/' gets an Athena major finding and the job goes on to the next round, with no crash; cargo test --locked, clippy -D warnings, fmt --check and xtask's boundary test pass; athena.toml has [job_plan] enabled = true; CHANGELOG.md names the crash and the job.

## Reading (accepted by the user)

Git::show_file returns None for a non-blob path, so no caller can crash on a directory. Athena's plan checks turn a planned directory or non-plain-relative path into a major finding that sends the plan back. Then job_plan is re-enabled and the crash and job 20261017-204355-358k go in the changelog. Audience: the Athena maintainer running athena go.

Assumptions:
- Root cause is in Git::show_file, which uses cat-file -e (true for trees) and then cat-file blob. The fix checks the object type, for example with cat-file -t or by resolving to a blob, and returns None for non-blobs.
- The plan_findings function in crates/athena-core/src/admission.rs, or the exists closure passed to plan_record in run.rs, is where the directory and bad-path checks go. The path shape check must not need the filesystem.
- A directory has no blob, so after the show_file fix exists=false. For Modify or Delete that already gives a major finding, but 'docs/' with Create would slip through. So an explicit path-shape and is-directory check is needed, and the directory check can use the git tree.
- The fake-agent test harness in crates/athena-cli/tests/job_plan.rs can script a planner output with a directory path.
- athena.toml is a governance file, and the request names it explicitly, so it is a listed deliverable.

## Acceptance criteria

1. [behavior] A git test in crates/athena-git shows that show_file on a directory path (for example a tree such as 'docs/' or 'docs') returns Ok(None) with no error, and that it still returns the bytes for a real file. Run: cargo test -p athena-git.
2. [behavior] A test with a fake planner whose plan lists 'docs/' shows that Athena emits a major finding for that path, that the job moves on to the next plan round, and that nothing crashes or returns an error. Run: cargo test.
3. [behavior] Plan checks give a major finding, not an error, for any planned path that is a directory, is absolute, contains '..', is empty, or ends in '/'. A unit test covers each case. Run: cargo test.
4. [behavior] cargo test --locked, cargo clippy --all-targets -- -D warnings and cargo fmt --check all pass, as does xtask's boundary test.
5. [content] athena.toml has [job_plan] enabled = true, and no other setting in that file changes.
6. [content] CHANGELOG.md under [Unreleased] names the job-planning crash on a directory path and job 20261017-204355-358k.
7. [behavior] Outcome check: replaying the failing input, a plan that lists docs/, no longer produces the 'git cat-file blob <base>:docs/ failed' error. This is shown by the new tests failing on the old code and passing on the new.

## Expected deliverables

- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Changing the planner prompt or the critic stage
- Changing planner or critic budgets in athena.toml
- Fixing the other callers of show_file individually
- Pre-existing-check handling from job 20261017-204355-358k's own request

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and integration tests; cargo test --locked, clippy -D warnings, fmt --check pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix show_file itself so no caller can crash on a directory, and also harden the plan check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise a directory path and a docs/ plan.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by tests and the plan check, not by prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The changelog entry and doc comments are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior criteria name the cargo commands the verifier runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the git crate, the plan check, config and changelog.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust and TOML.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base bdea14224091 → candidate e592d9599a5a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  10 ++++
 athena.toml                         |   2 +-
 crates/athena-cli/tests/job_plan.rs |  40 ++++++++++++++
 crates/athena-core/src/admission.rs | 106 +++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/run.rs       |   1 +
 crates/athena-git/src/lib.rs        |  36 +++++++++++-
 6 files changed, 191 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (933 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
