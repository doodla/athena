# Job 20261016-071159-s6tr: verify "Add CI cache layers and a wall-time-per-result baseline measurement"

## Request (verbatim)

> Build the CI compute routing designed in docs/design/ci-routing.md (#114), and measure wall time per verified result before and after.

## Reading (accepted by the user)

Reads "build the routing" as the first buildable slice of docs/design/ci-routing.md section 10: the baseline measurement (item 2) plus the CI cache layers (item 3) in ci.yml, for the owner and the design reviewers. It records a measured "before" and a repeatable computation for "after". The SHA-keyed result reuse and per-kind routing (items 4-5) are left to later jobs.

Assumptions:
- The request authorizes building despite the pending review in docs/design/reviews/ci-routing-review-1.md; the producer must not edit that review or mark the design accepted.
- 'Small' default applies: only build-plan items 2 (baseline) and 3 (cache) are built; items 4-7 are later jobs.
- 'Before' is computed from existing job records or, if per-phase timestamps are absent, reported as unavailable with the requester's ~25 min median labeled given, not measured.
- 'After' cannot be measured in this job; the change provides the repeatable computation and a stated procedure, and the doc says so.
- Cache actions are pinned by SHA like the existing workflow actions; only the default branch saves the target cache.

## Acceptance criteria

1. [behavior] Run cargo test --locked: all tests pass, including new tests showing the wall-time computation gives the median and p90 of dispatch-to-verified time on sample records, reports failed/rejected jobs separately, and states when fewer than 20 jobs exist.
2. [behavior] Run cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check: both clean.
3. [behavior] Run the new xtask command via cargo run --locked -p xtask on the repository's job records or sample fixture: it prints wall time per verified result by job kind, or says explicitly that the records lack the timestamps instead of printing zero or an empty success.
4. [content] A baseline document gives the before figures with each number marked measured, given or estimate, and states how to compute the after figures with the same command and N=20.
5. [content] ci.yml adds toolchain, registry and target cache layers keyed on rust-toolchain.toml and Cargo.lock as in design 4.1, with the cache action pinned by SHA, target saved only on the default branch, and the cache outcome (exact, restore key, miss) reported.
6. [content] ci.yml still runs the same checks (xtask ci and the history job) with unchanged permissions; no check is removed or skipped. ci.yml is named as a changed governance path in the change notes.
7. [content] docs/design/ci-routing.md is not marked accepted, the review file is unchanged, and the CHANGELOG records what was built and what remains (build-plan items 4-7).

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/src/main.rs (modify)
- xtask/src/walltime.rs (create)
- xtask/tests/walltime.rs (create)
- docs/design/ci-routing.md (modify)
- docs/design/ci-routing-baseline.md (create)
- CHANGELOG.md (modify)
- Cargo.lock (modify)

## Out of scope

- SHA-keyed CI result reuse and local fallback (build-plan item 4)
- Per-job-kind routing from the plan (item 5)
- Recording the after comparison (item 6)
- README and decisions docs job (item 7)
- Editing athena.toml, defaults.toml, or the review file
- Changing the verifier's command set

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Measurement code is a Rust xtask subcommand, not a script.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The measurement code has unit tests; cargo test and clippy pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers are measured or labeled estimate/given.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the baseline and cache slices are built.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check wall time per verified result is computed, not that a workflow exists.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise the computation on sample records.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Policy tests, if any exist for workflows, are run and kept passing.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test and the new xtask command.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 29ae07484481 → candidate 565f50d8743a. In your working directory, `git diff HEAD~1` shows it.

```text
 .github/workflows/ci.yml           | 130 ++++++++++++++-
 CHANGELOG.md                       |  14 ++
 Cargo.lock                         |   1 +
 docs/design/ci-routing-baseline.md |  65 ++++++++
 docs/design/ci-routing.md          |  15 +-
 xtask/Cargo.toml                   |   1 +
 xtask/src/main.rs                  |   6 +-
 xtask/src/walltime.rs              | 315 +++++++++++++++++++++++++++++++++++++
 xtask/tests/walltime.rs            | 177 +++++++++++++++++++++
 9 files changed, 719 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (765 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
