# Job 20261027-054214-dn47: verify "Check config edits against the schema at plan time, not only at landing"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The config file and the code that parses it are changed separately, and nothing checks that a new co
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: config-edit-ahead-of-schema-breaks-admission (20261016-185439-64ke)
> Cause: The config file and the code that parses it are changed separately, and nothing checks that a new config section is known to the parser. The new `[parallel]` table was added to athena.toml (by the job's plan or by hand) without a matching s...
> Seen 4 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261016-185428-8vdd (job 20261016-071202-q0xy); prevention it proposed: Run a parse check of athena.toml against the schema as part of the edit and before admission. It should run when a config change is planned, and again on the job's resulting diff. A job whose outcome needs a new config section must deliver ...
> Introspection 20261016-191018-b44f (job 20261016-185517-qcrz); prevention it proposed: Add two code checks. (1) A config-parse check at submit and enqueue time, and again before a job starts. If athena.toml fails to parse under the current schema, refuse the request at once with the parse error and say it is the same cause as...
> Introspection 20261016-191109-w348 (job 20261016-185517-qcrz); prevention it proposed: Change admission so it can accept a config change that comes with the schema change in the same job. For example, validate the changed athena.toml against the schema from the job's own proposed code, or treat config sections the binary does...
> Introspection 20261016-205039-fqgx (job 20261016-200605-xh54); prevention it proposed: Add a code check that runs before any commit that touches athena.toml or the config schema, and before landing. It builds the parser from the working tree being committed and parses the athena.toml in that same tree, and it refuses the comm...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: athena.toml and its parser (crates/athena-data/src/config.rs, strict on unknown fields) are changed separately, and only admission and landing check them. The gate already defers the parse when a change touches crates/athena-data/ (`defers_to_schema_test`), but a plan can still list athena.toml or defaults.toml without any schema file, or name the wrong crate (jobs 20261016-071202-q0xy and 20261016-185517-qcrz did). The fix adds one shared plan-time rule on that same predicate and tests it for the class. It addresses jobs 20261016-071202-q0xy, 20261016-185517-qcrz and 20261016-200605-xh54.

Assumptions:
- Goal: Athena's maintainers need a config edit that is not covered by a schema change to be refused or flagged at plan time, so jobs stop dying at admission or landing with 'unknown field'.
- Job 20261016-071202-q0xy: its `[parallel]` refusal came from the schema being edited in the wrong crate (athena-core) and no check connecting the two. The plan-time rule now catches that case; the parallel feature itself has since landed (athena-data/src/config.rs has `Parallel`, and athena.toml has `[parallel]`).
- Job 20261016-185517-qcrz: the same refusal. The schema deferral in athena_gate (SCHEMA_DIR = crates/athena-data/) already handles the case where the schema file is in the change. The gap that remains is a plan that edits config without a schema file, which this change closes.
- Job 20261016-200605-xh54: it passed verification but the landing gate refused; its four findings (extra files, panic isolation in drain, the display label, gate refusal) concern the parallel feature, not this cause. The gate refusal is covered by the existing deferral rule and the new rule; the other findings no longer apply to this change.
- The rule reuses `athena_gate::defers_to_schema_test` and `SCHEMA_DIR` so plan, admission and gate share one definition. It lives in athena-core (plan or admission module), since athena-core already depends on athena-gate.
- Rule: a plan item whose deliverables modify athena.toml or defaults.toml must also list a modified path under crates/athena-data/, or the plan is flagged. The check cannot see new sections at plan time, so the producer picks the narrowest sound form and documents it.
- The exact location of plan validation was not read in full. The producer should find it in crates/athena-core/src/plan.rs or admission.rs.

## Acceptance criteria

1. [content] The change description names the root cause: config and parser change separately, and nothing connects a new athena.toml section to the schema that must accept it before plan time. It cites jobs 20261016-071202-q0xy, 20261016-185517-qcrz and 20261016-200605-xh54.
2. [behavior] A new test in crates/athena-core (run with `cargo test -p athena-core`) shows that a plan or change which modifies athena.toml without any crates/athena-data/ path is flagged with a message that names the schema file to change. The same test shows that a change that also modifies crates/athena-data/src/config.rs is accepted. The test fails on the base commit and passes with the change.
3. [behavior] The plan-time rule calls `athena_gate::defers_to_schema_test`/SCHEMA_DIR rather than duplicating the path, and a test confirms that the gate, admission and the plan rule agree on the same inputs (`cargo test`).
4. [behavior] Existing gate and admission tests (unknown-field refusal in content.rs, deferral tests) and crates/athena-data/tests/repo_config.rs still pass under `cargo test`.
5. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry for the new check. No changes to athena.toml, defaults.toml or .github/workflows.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/content.rs (modify)

## Out of scope

- Changes to the parallel-jobs feature itself (drain.rs, [parallel] settings)
- Submit/enqueue-time config parse refusal for a config that already fails to parse
- Letting admission build and run candidate code
- Changes to athena.toml, defaults.toml or .github/workflows
- The drain panic-isolation and display-label findings from job 20261016-200605-xh54
- Threads-in-one-process spike, daemon or async runtime, automatic merging of stale jobs, run-level spend cap
- Multi-job `athena go` and the --pr landing phase

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix goes where the config and schema are connected, using the shared SCHEMA_DIR rule, not at one job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the class, and fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the new test and check that it fails on the base.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A code rule enforces this, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused plan-time check on the existing shared predicate.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 1df9a0183a57 → candidate 566b6c8cb2ea. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  12 +++++
 crates/athena-core/src/admission.rs |  38 +++++++++++++
 crates/athena-core/src/plan.rs      | 105 +++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/reading.rs   |  77 +++++++++++++++++++++++++-
 crates/athena-gate/src/content.rs   |  34 ++++++++++++
 crates/athena-gate/src/lib.rs       |  24 +++++++++
 6 files changed, 287 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1505 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
