# Job 20261016-191121-g5yk: verify "Admit athena.toml changes by the candidate's own Config schema"

## Request (verbatim)

> Parallel jobs (20261016-071202-q0xy, 20261016-185517-qcrz) failed twice at admission: admission.rs parses a changed athena.toml with the running Athena's Config schema, so a job that adds a config section and its schema field together can never be admitted. Check the candidate's config against the candidate's own schema.

## Reading (accepted by the user)

Admission parses a changed athena.toml with the running Athena's compiled `Config`, so a job that adds a section and its schema field together is always refused (jobs q0xy and qcrz hit this on `[parallel]`). Change admission so the candidate's config is checked against the candidate's own schema, while a config with a genuinely unknown field is still refused. The audience is Athena's operator and future jobs that extend the config.

Assumptions:
- Read from crates/athena-core/src/admission.rs lines 215-238: the 'configuration' rule calls Config::parse (the base binary's athena_data::Config, which uses deny_unknown_fields in config.rs) on the candidate's athena.toml.
- Candidate commit C is a tree in the user's git repo, not built before admission (run.rs ~846), so the candidate's schema must be exercised by checking out/building the candidate's athena-data crate, e.g. in a temporary tree, and running a small check against it.
- Where the candidate does not change the schema (crates/athena-data/src/config.rs), behavior stays as today: parse with the running Config.
- The candidate's schema check must fail closed: if the candidate's own check cannot be built or run, or reports an error, admission yields a 'configuration' violation with the real message, never silent success.
- defaults.toml (DefaultsModel) has the same shape of problem; fix it the same way only if it falls out of the same mechanism, otherwise leave it.
- Landing still re-parses the config with the base Config where it does today; if landing has the same flaw it is noted rather than silently left. Check crates/athena-gate/src/lib.rs line 140 for that.

## Acceptance criteria

1. [behavior] A new admission test shows a candidate that adds a new `[section]` to athena.toml together with the matching field in athena-data's Config is admitted with no 'configuration' violation. Run `cargo test -p athena-core admission`.
2. [behavior] A new admission test shows a candidate that adds an unknown `[section]` to athena.toml without changing the schema is still refused with a 'configuration' violation naming the unknown field. Run `cargo test -p athena-core admission`.
3. [behavior] A test shows that when the candidate's own schema check cannot run or fails, admission reports a 'configuration' violation instead of passing. Run `cargo test -p athena-core admission`.
4. [content] admission.rs documents (module comment and rule comment) that a changed athena.toml is judged by the candidate's schema, and states how the candidate's schema is obtained.
5. [behavior] The existing admission tests (bad athena.toml, defaults.toml, docs/athena.toml) still pass; `cargo test` passes and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
6. [content] Outcome check: the failure text from jobs 20261016-071202-q0xy and 20261016-185517-qcrz (`unknown field parallel` when athena.toml and the schema change together) is reproduced by a test and no longer occurs.
7. [content] CHANGELOG.md has an entry describing the admission change, if the repository's CHANGELOG convention covers behavior changes.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- docs/design/parallel-jobs.md (modify)

## Out of scope

- Building parallel jobs themselves (job q0xy's request) and the [parallel] config section.
- Changing athena.toml or defaults.toml.
- Relaxing deny_unknown_fields in Config.
- Retrying jobs q0xy or qcrz.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New admission tests cover the admitted, still-refused, and fail-closed cases; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the rule that checks the config against the wrong schema, not just the `[parallel]` instance.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run admission on a config-plus-schema change and check the outcome.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion reproduces the exact failure of the two named jobs.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The fail-closed criterion prevents an unrunnable candidate check from counting as a pass.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in admission.rs; defaults.toml handled only if it falls out of the same mechanism.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by admission tests, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base cc93ff92a3d0 → candidate dfdd7e7847f7. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |   9 ++
 crates/athena-core/src/admission.rs | 213 +++++++++++++++++++++++++++++++++++-
 2 files changed, 218 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (768 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "admission.rs documents (module comment and rule comment) that a changed athena.toml is judged by the candidate's schema, and states how the candidate's schema is obtained."; "The existing admission tests (bad athena.toml, defaults.toml, docs/athena.toml) still pass; `cargo test` passes and `cargo clippy --all-targets` and `cargo fmt --check` are clean."; "Outcome check: the failure text from jobs 20261016-071202-q0xy and 20261016-185517-qcrz (`unknown field parallel` when athena.toml and the schema change together) is reproduced by a test and no longer occurs."; "CHANGELOG.md has an entry describing the admission change, if the repository's CHANGELOG convention covers behavior changes.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
