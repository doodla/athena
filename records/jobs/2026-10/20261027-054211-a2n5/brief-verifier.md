# Job 20261027-054211-a2n5: verify "Refuse readings that leave out governance files the outcome needs"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Reading of the request happens without knowing which files the outcome will require changing. "Publi
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: governance-file-needed-but-not-in-reading (20261014-083307-xkby)
> Cause: Reading of the request happens without knowing which files the outcome will require changing. "Publish to GitHub Pages" implies a deploy workflow, but nothing at intake maps outcomes to governance files. The protection rule is only enforced...
> Seen 4 time(s) in 4 jobs. The introspections that record it:
> Introspection 20261014-083258-8afk (job 20261014-082328-g6vy); prevention it proposed: At reading time, run a check that predicts the files a job will change, for example from the repo's Pages configuration and existing workflows. If any are governance files, require the reading to list them as deliverables and name them in a...
> Introspection 20261015-015633-3wma (job 20261015-014012-akwq); prevention it proposed: Add a check at reading acceptance that scans the planned change for governance-protected paths (athena.toml, workflows, policy files). It would also check that new config keys exist in the schema. When the outcome implies one of these, such...
> Introspection 20261017-054034-fccj (job 20261017-044557-3htd); prevention it proposed: Add a check when the reading is drafted or accepted. It would work out which files the request's outcome is likely to touch, for example any request mentioning routing, checks, classes or budgets, and compare them with the governance-protec...
> Introspection 20261017-121053-b8ek (job 20261017-114155-ksq0); prevention it proposed: Add a check at the reading stage, before acceptance. It would take the files the reading's deliverables are expected to touch, using the request's keywords such as CI, gate, workflow and config plus a scan of the repo. It would compare them...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: reading acceptance (`check_reading_acceptance` in athena-core reading.rs) never maps the request's outcome to governance files, so the gap only surfaces at admission after the producer has finished (jobs g6vy, akwq, 3htd and others). Add a no-model-call acceptance check that predicts governance paths from the request keywords and the listed deliverables, and refuses a reading that omits them or lists them without naming them in a criterion.

Assumptions:
- Audience: the reader agent and the steward, who get a repairable refusal at reading time rather than a failed job at admission. The check reuses `is_governance` and `governance_change_is_listed` from admission.rs so there is one definition of governance.
- Job 20261014-082328-g6vy (Pages explainer, refused for .github/workflows/pages.yml): addressed because a request mentioning Pages, deploy or workflow now must list and name a workflow file or explicitly put it out of scope. Its deliverables and out_of_scope were quoted from the earlier job; this change does not redo the site work.
- Job 20261015-014012-akwq (audience role, refused for athena.toml and an unknown `audience` field): addressed because a reading that adds a config section, i.e. lists crates/athena-data/src/config.rs, must also list and name athena.toml. The email-address finding is a separate policy matter and is out of scope here.
- Job 20261017-044557-3htd (CI routing item 5, refused for athena.toml): addressed because requests that mention routing, checks, classes or budgets, or that list a governance file without a criterion naming it, are refused at acceptance. Its allowed scope (routing.rs etc.) is not rebuilt here.
- The mapping is a small table of keyword to governance path in reading.rs. A reading can satisfy it by listing and naming the file, or by naming the path in out_of_scope. The refusal text states which path and why. No prompt file changes, since the reader prompt has 3 tokens of headroom.
- The request is generic across jobs, so a keyword heuristic is accepted. It errs toward asking for an explicit decision, which costs one retry at most.

## Acceptance criteria

1. [content] The change description names the root cause: reading acceptance has no mapping from the request's outcome to governance files, so the omission is found only at admission. The fix is in crates/athena-core/src/reading.rs `check_reading_acceptance`, called for first attempts and retries alike, not in any one job's reading.
2. [behavior] Run `cargo test -p athena-core`: new tests in reading.rs fail without the change and pass with it. A reading for the request 'Publish to GitHub Pages' with no workflow deliverable is refused, naming .github/workflows. A reading that lists athena.toml but no criterion names it is refused. A reading that lists crates/athena-data/src/config.rs without athena.toml is refused. Each of these readings is accepted once it lists and names the file, or names it in out_of_scope. A request with no matching keywords and no governance deliverables is accepted unchanged.
3. [behavior] The new tests are table-driven over the keyword table, covering the class (several request phrasings such as CI, gate, workflow, routing, budgets, classes), not just the three recorded jobs. Run `cargo test -p athena-core` to check.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no warnings, and existing reading and admission tests still pass.
5. [content] CHANGELOG.md has an entry for the new acceptance check, and the refusal message names the missing path and says to list it as a deliverable and name it in a criterion.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing athena.toml, defaults.toml, workflows or other governance files themselves
- Changing prompt files (the reader prompt has 3 tokens of headroom)
- Changing the admission rules, the governance-file list, or the policy email allow-list
- Redoing the Pages site, audience role or CI routing work from the three earlier jobs
- Using a model call to predict files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests are added, and fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check sits at reading acceptance, where the gap originates, and covers the whole class.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check readings being refused and accepted, not that a function exists.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by code at acceptance rather than by prompt prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in reading.rs, reusing admission helpers.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior criterion is exercised with cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base e13eb7282ded → candidate 07bf4a9ba9da. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  14 ++
 crates/athena-core/src/admission.rs |   4 +
 crates/athena-core/src/reading.rs   | 311 +++++++++++++++++++++++++++++++++++-
 docs/architecture.md                |   9 ++
 4 files changed, 337 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1501 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no warnings, and existing reading and admission tests still pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
