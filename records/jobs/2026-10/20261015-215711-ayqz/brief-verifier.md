# Job 20261015-215711-ayqz: verify "Pin study jobs to the commit the study's pin.json names"

## Request (verbatim)

> Study jobs read the wrong firstmate commit: job 20261015-201650-cyvb's pinned source resolved to fc8972cd while studies/firstmate-work-handling/pin.json says 6407052f, so the producer cited lines from the newer code under the old label and the verifier rejected it. A study job must read exactly the commit its study's pin.json names.

## Reading (accepted by the user)

The core resolves a study job's source from athena.toml's repo_ref ("main"), ignoring the pin in studies/<id>/pin.json, so a re-study reads newer code under the old label. Change the pinning step in athena-core so that when the studied repository already has a study with a pin.json, the job pins that exact sha, and add tests. The audience is study producers and verifiers, who must see the mounted tree match the pin.

Assumptions:
- Root cause, read from crates/athena-core/src/run.rs pin_repo and athena.toml [capabilities.study]: pin_repo uses only the global repo_url/repo_ref ("main") and never reads studies/*/pin.json, so any later study job gets the branch tip.
- The request names no study, so the study is found from the repository: the sole study whose pin.json repository_url equals the configured repo_url, else the one whose study_id appears in the request text; otherwise the configured ref applies (a new study).
- The fix is in the core pinning step, not in athena.toml: editing repo_ref to a sha would pin every future study to one commit.
- The existing firstmate-work-handling study is not repinned or re-cited here; the user can ask for that separately.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core (run_pin_tests.rs) dispatches a study job whose head holds studies/<id>/pin.json with a repository_url matching the configured repo_url and a sha differing from what repo_ref ("main") resolves to. It asserts the pinner is asked for that sha, and the job record's pin sha equals it. Run: cargo test -p athena-core.
2. [behavior] Tests cover the fallbacks: with no existing study for the repository the configured repo_ref is used as before; with several studies for one repository, the one whose id the request names is chosen, and if none is named the behavior is documented and never silently picks a different commit. Run: cargo test.
3. [behavior] A pin.json whose sha is not a full 40-hex id, or whose pin does not resolve to that same sha, stops the job before dispatch with a message naming the study, rather than falling back to main. Run: cargo test.
4. [behavior] cargo test --locked, cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check all pass.
5. [content] The doc comment on pin_repo and a CHANGELOG entry state the rule (an existing study's pin.json overrides repo_ref; athena.toml's repo_ref is used only for a new study) and name the cause: job 20261015-201650-cyvb mounted fc8972cd while the study pin said 6407052f.
6. [content] No file under studies/, athena.toml or defaults.toml is changed; re-pinning or re-citing the study is a separate user decision.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/pin.rs (modify)
- crates/athena-core/src/fake.rs (modify)

## Out of scope

- Re-pinning or re-citing studies/firstmate-work-handling to fc8972cd
- Fixing the t3 local-only/scout wording in model.json and intro.json
- The rules.rs STUDY_SHA test mapping
- Changing athena.toml repo_ref

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fix and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy, fmt criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the pinning step that ignores pin.json, not the single study's citations.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert which sha the pinner is asked for and the recorded pin.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code with tests rather than a prose rule.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in pin_repo and its tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands to run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and CHANGELOG explain the cause for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 374c29a9823b → candidate 4ecf19d80881. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |   8 +++
 crates/athena-core/src/fake.rs          |  21 +++++--
 crates/athena-core/src/run.rs           |  98 +++++++++++++++++++++++++++--
 crates/athena-core/src/run_pin_tests.rs | 107 ++++++++++++++++++++++++++++++++
 docs/decisions.md                       |  20 ++++++
 5 files changed, 245 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (713 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
