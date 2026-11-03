# Job 20261102-134406-gznq: verify "Check declared build-input sets against the real compiled-in inputs"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> Introspection 20261102-102715-564y (job 20261031-172040-kq9v); prevention it proposed: Add a code check, run in the repo tests and before verification, that computes the athena binary's real input set (every file the build compiles in, including everything xtask input_files should list) and fails if the hashFiles globs in eve...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a declared list (CI cache globs, input_files, BUILD_PATHS, ontology terms) is checked for shape but never against the source it summarizes, so errors surface only at the final verdict. Add a code check that derives the athena binary's real input set (every include_str!/include_bytes! target under crates/ plus BUILD_PATHS) and fails if the ci.yml athena-bin hashFiles globs or xtask input_files miss any. Audience: Athena maintainers and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (BUILD_PATHS at line 20 is the repo's definition of build inputs), crates/athena-context/src/lib.rs and briefs.rs (include_str! of prompts and docs/system-facts.md), and grep of include_str! across crates/*/src. Entry point is a new test in crates/athena-core (cargo test), run before verification; it reads files from the workspace root, scans sources for include_str!/include_bytes! targets, resolves them relative to the source file, and compares with BUILD_PATHS and the hashFiles globs parsed from .github/workflows/ci.yml. No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected; the allow-list is untouched. The property is measured directly: the real include set versus the declared globs, not a stand-in.
- Prior job 20261031-172040-kq9v: its finding (athena-bin key omitted prompts and docs/system-facts.md) already appears fixed in the repo's ci.yml (globs crates/**, xtask/**, docs/system-facts.md). This change keeps that work as is and adds the mechanism the finding's minor item asked for, so the hand-kept list cannot drift again. The stamp-on-non-CI-build, restored build-commit and dirty-tree findings are not addressed here; they are minor, local, and listed out of scope.
- Prior job 20261017-173508-0wtc (ontology term contradicts BRIEF.md): addressed at the class level only through the shared mechanism of checking a declared list against its governing source; the ontology content itself is left as is and not touched here. Prior job 20261025-214509-y179 (speed target not met, no baseline): its code changes (secrets.rs memo, checks.rs poll schedule) are kept untouched; the measure-first stage is out of scope because it is a pipeline change needing its own job.
- The check parses hashFiles patterns with a simple glob match (prefix/** and exact file names), enough for the current ci.yml; a pattern form it cannot parse fails loudly rather than passing.

## Acceptance criteria

1. [content] The change description (CHANGELOG.md entry) names the root cause: declared input lists were checked for structure but not against the real compiled-in inputs, citing jobs 20261031-172040-kq9v, 20261017-173508-0wtc and 20261025-214509-y179 and saying how each prior result is addressed or why it is out of scope.
2. [content] A new test under crates/athena-core derives the input set from every include_str!/include_bytes! target under crates/*/src plus BUILD_PATHS, and asserts each is covered by the athena-bin hashFiles globs in .github/workflows/ci.yml (all jobs) and by BUILD_PATHS; it reads ci.yml rather than a copy.
3. [behavior] Negative test: run `cargo test -p athena-core` on in-memory fixtures where a glob list omits a prompt .md file or docs/system-facts.md; the check returns a failure naming the missing path. The same fixtures with the full list pass. The test fails against the pre-fix key from job kq9v (only *.rs and Cargo.toml globs).
4. [behavior] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: all pass on the real repository, so the current ci.yml satisfies the new check.
5. [content] The check is a refusal rule over repository files, not over readings; no replay of past readings is affected (expected 0% of newly refused readings).

## Expected deliverables

- crates/athena-core/tests/build_inputs.rs (create)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Changing docs/ontology.toml, crates/athena-cli/tests/ontology.rs or any study (job 0wtc work stays dropped/unchanged)
- Changing crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs or the test-speed goal (job y179 work kept as is)
- Changing xtask/src/main.rs, xtask/tests/policy.rs or the setup.sh stamp and dirty-tree behavior (job kq9v work kept as is)
- A measure-first stage for speed-target jobs
- Changing athena.toml, defaults.toml, tool pins or action SHAs

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the shared cause: declared lists never checked against their real source.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a test, not prose.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Test plus fmt, clippy and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Negative fixture test checks the failure happening.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check; ontology and pipeline changes excluded.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Check written in Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG names cause and jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Inspect the new Unreleased entry in CHANGELOG.md by Read, and run the small test `unreleased_changelog_names_the_cause_and_jobs`, which reads CHANGELOG.md, takes the text after `## [Unreleased]` up to the next `## [`, and asserts it contains 20261031-172040-kq9v, 20261017-173508-0wtc, 20261025-214509-y179 and the word `compiled-in`. Run: `cargo test -p athena-core --test build_inputs changelog`. Observes: The entry states the cause (declared input lists checked for structure, not against the real compiled-in inputs) and how each job is handled: kq9v by adding the BUILD_PATHS check and the list-taking matcher on top of the existing ci.yml key check; 0wtc by the shared mechanism only, ontology untouched; y179 kept as is and out of scope (measure-first stage needs its own job). The test passes only if the three ids and the keyword are present.
  Fails without the behavior: Without the entry, the test finds no job id in the Unreleased section and fails; a changelog that omits the cause leaves the verifier's content check failing.
  Test seam: Reads the real CHANGELOG.md through std::fs from the workspace root (CARGO_MANIFEST_DIR/../..). No double. The same split on `## [Unreleased]` exists in reading.rs:7824; the test is added by step 3.
- Criterion 2: Test `declared_inputs_cover_the_real_compiled_in_inputs`: calls `production_includes()` plus the manifest set to build `required`, asserts the known-target floor and that every path exists on disk, then calls `check_build_paths(&BUILD_PATHS, &required)` and `check_keys(&real_ci_yml, &required)`. A second test counts the `key: athena-bin-` lines in the real ci.yml. Run: `cargo test -p athena-core --test build_inputs`. Observes: The scan returns the prompts of athena-context, athena-cap-change and athena-cap-study, docs/system-facts.md and athena.toml. Each is covered by BUILD_PATHS and by every one of the nine athena-bin key lines (six setup jobs plus three save steps, the count policy.rs asserts at line 970), all read from the real .github/workflows/ci.yml, not a copy. The floor equals the one at xtask/tests/policy.rs:942-951.
  Fails without the behavior: Not a behavior criterion; the content is checked by the verifier reading the test. The test also fails if the scan finds nothing, if a scanned path does not exist, or if ci.yml has no athena-bin key.
  Test seam: Not required for this content criterion; the real repository files are the input.
- Criterion 3: Negative tests that call the same functions as the real-repository test, with edited text or lists. A: `check_keys` on the real ci.yml with `, 'docs/system-facts.md'` removed once (replacen; assert the text changed) returns Err containing docs/system-facts.md. B: `check_keys` on ci.yml with the key globs replaced by the pre-fix kq9v key (`crates/**/*.rs`, `Cargo.toml` only) returns Err naming crates/athena-context/prompts/producer.md and docs/system-facts.md. C: `check_keys` with a glob form the matcher does not support (e.g. `crates/*/prompts/*.md`) returns Err naming that pattern. D: `check_keys` with a required set plus `crates/x/prompts/new.md` returns Ok (covered by `crates/**`), plus `docs/new-compiled-in.md` returns Err naming only that path. E: `check_build_paths(&OLD_FIVE, &required)` where OLD_FIVE is `["crates/","xtask/","Cargo.toml","Cargo.lock","setup.sh"]` returns Err naming athena.toml and docs/system-facts.md and rust-toolchain.toml. Each is paired with the unmodified real input returning Ok (the real `BUILD_PATHS` for E). Run: `cargo test -p athena-core --test build_inputs`. Observes: Every failure names the missing path; every paired unmodified or full list passes. The pre-fix kq9v key fails by naming a prompt .md file, not by a parse error.
  Fails without the behavior: If `check_keys` ignored a missing path, the unwrap_err assertions in A, B and D fail. If `check_build_paths` always returned Ok, E fails. Without the matcher's `dir/**/*.ext` handling, B would fail on the unsupported-glob error and the assertion naming the prompt path would fail. Without the BUILD_PATHS extension in status.rs, the real-repository pairing of E fails on athena.toml and docs/system-facts.md.
  Test seam: Production entry points: `athena_core::status::BUILD_PATHS` (exists, status.rs:20) and the new `athena_core::status::list_touches(list: &[&str], paths: &[String]) -> bool` (added by step 2; `touches_build` at status.rs:23 calls it, so production and the check share one matcher). Check functions `check_keys(yml: &str, required: &BTreeSet<String>) -> Result<(), String>` and `check_build_paths(list: &[&str], required: &BTreeSet<String>) -> Result<(), String>` are added by step 3 in build_inputs.rs. Fixtures: the real ci.yml string edited with replacen in the test, and a literal five-entry list; no double and no logic in the fixtures. Guard: each edit asserts the text changed, so a no-op replacen cannot give a false pass.
- Criterion 4: Run the three commands on the real repository. Also run `cargo test -p xtask --test policy` and `cargo test -p athena-cli --test drain`, and grep for tests that assert on the `stale build` text or BUILD_PATHS length (e.g. Grep `stale build` and `BUILD_PATHS` under crates/) so the extension breaks none. Run: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --check`. Observes: All three exit 0. The new test passes against the current ci.yml, so the workflow satisfies the check. policy.rs still passes after the ci.yml header-comment edit, because the edit adds no line starting `key: athena-bin-`.
  Fails without the behavior: Without the BUILD_PATHS extension, `cargo test --workspace` fails in build_inputs on athena.toml, docs/system-facts.md and rust-toolchain.toml. Without the CHANGELOG entry, the changelog test fails.
  Test seam: The real repository tree. The seam `list_touches` is added by step 2; the drain test (`only_changes_to_code_and_the_files_that_build_it_touch_the_build`) calls `touches_build`, which exists and keeps its signature.
- Criterion 5: Show by `git diff --stat` that the change touches only build_inputs.rs, status.rs, CHANGELOG.md, docs/decisions.md and the ci.yml header comment, and run the unchanged refusal corpus test as insurance. D154 states that the check refuses repository files and that no replay of past readings is affected. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: The corpus test passes unchanged and no file under reading.rs or the corpus is in the diff. I read no corpus case ids and cite none. Expected newly refused readings: 0.
  Fails without the behavior: Not a behavior criterion; a content check on the diff and on D154.
  Test seam: Not required for this content criterion.

Planned files:

- crates/athena-core/tests/build_inputs.rs (create)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- .github/workflows/ci.yml (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base 5a4ae212c79a → candidate b66afedf184b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml                 |   4 +-
 CHANGELOG.md                             |  19 ++
 crates/athena-cli/src/drain.rs           |   2 +-
 crates/athena-core/src/status.rs         |  33 ++-
 crates/athena-core/tests/build_inputs.rs | 363 +++++++++++++++++++++++++++++++
 docs/decisions.md                        |  50 +++++
 6 files changed, 462 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41516044363 for b66afedf184bf4b4fe5cf2adf5852fcb6fe067d4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41516044363 for b66afedf184bf4b4fe5cf2adf5852fcb6fe067d4)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41516044363 for b66afedf184bf4b4fe5cf2adf5852fcb6fe067d4)
- ci-result: `not a check` passed (Check results for b66afedf184bf4b4fe5cf2adf5852fcb6fe067d4 come from the CI run https://github.com/doodla/athena/actions/runs/41516044363 (431s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: all pass on the real repository, so the current ci.yml satisfies the new check.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
