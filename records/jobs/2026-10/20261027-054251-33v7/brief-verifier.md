# Job 20261027-054251-33v7: verify "Check ontology terms against the governing sources they summarize"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (jobs 20261017-173508-0wtc and 20261025-214509-y179): checks covered structure or activity, not whether seeded or claimed content is right against the sources it summarizes. Fix it in the ontology test: for each term, search the governing docs for its name and fail unless the entry lists every sense found or records why a hit is not one. This is a retry of 0wtc's lesson on its landed files; y179's speed work is out of scope.

Assumptions:
- Code survey: read crates/athena-cli/tests/ontology.rs (problems() checks parse, empty fields, source existence, relation targets, not_part_of sources; it never reads the sources' content) and the term list in docs/ontology.toml (planning, job, Athena, Hermes, product versus system, landed, Sri). No records/ or check_record_file/AnyRecord is touched: docs/ontology.toml is not under crates/, and no new file is added under crates/.
- Job 20261017-173508-0wtc: its Hermes finding (BRIEF.md names Hermes as Orchestration god) is already fixed in the tree: the Hermes term has both senses and the test asserts them. Its files (docs/ontology.toml, crates/athena-cli/tests/ontology.rs, docs/decisions.md, CHANGELOG.md) are kept and extended, not dropped. What remains is the class: nothing checks a term against the source text, so the next multi-sense term would again fail only at the final verdict.
- Job 20261025-214509-y179: its findings were a missed speed goal, unmade test cuts and causes inferred rather than traced. It shares the class of a check that covers activity rather than the outcome. Its changed files (secrets.rs, checks.rs, its decisions.md and CHANGELOG entries) are kept untouched; the speed work and the measure-first proposal are out of scope here and need their own job.
- The new check is a test in crates/athena-cli/tests/ontology.rs. Governing sources are BRIEF.md plus each term's own listed source files. A term name is matched as a whole word or phrase. The term entry gains an optional field (for example senses or ignored_hits) with a reason for each hit it does not treat as a sense.
- Audience: Athena agents and Sri, who read the ontology; the verifier runs cargo test.

## Acceptance criteria

1. [content] The change description names the root cause: term content was checked for structure, not against the governing sources it summarizes, so a multi-sense term was defined from one source. It cites jobs 20261017-173508-0wtc and 20261025-214509-y179.
2. [behavior] Run `cargo test -p athena-cli --test ontology`: a new test scans BRIEF.md and the term's sources for each term name in docs/ontology.toml and passes on the real file. It fails if a sense found there is neither listed on the entry nor excused with a recorded reason.
3. [behavior] Run `cargo test -p athena-cli --test ontology`: a unit test on in-memory input and a scratch source (the Hermes case: one entry defining only the tool while the source also names a role) is flagged. The same input with both senses listed passes. The test fails when the new check is removed.
4. [content] The check is generic over all terms, not special-cased to Hermes. The test's header or docs/ontology.toml's [about] states the new rule and the optional field for senses or excused hits.
5. [content] docs/decisions.md has a new decision stating the cause, the alternatives considered (a rule only in prose; a measure-first stage for speed jobs) and why the check was placed in the ontology test. CHANGELOG.md has an entry.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] No change to athena.toml, defaults.toml, workflows or the speed-work files (crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs). Verify with `git diff --stat`.

## Expected deliverables

- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Speed-target work from job 20261025-214509-y179 (measure-first stage, test-side cuts, the secrets.rs memo, run_check polling); those files are kept as landed and not changed
- Adding new ontology terms not caused by a misread
- Changing any study's concept model or files under studies/
- Changes to athena.toml, defaults.toml or workflows
- Wiring the ontology into agent contexts or prompt budgets

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check is a Rust test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests, with cargo test, clippy and fmt passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a check on content against sources, not a patch to one term.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the test and a failing in-memory case.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a test, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One test file, the data file and the notes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision and test header are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d733fa8de2ad → candidate 8a691448a993. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   6 ++
 crates/athena-cli/tests/ontology.rs | 197 +++++++++++++++++++++++++++++++++++-
 docs/decisions.md                   |  27 +++++
 docs/ontology.toml                  |  41 ++++++++
 4 files changed, 269 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41441535129 for 8a691448a99328900f46b3d4e78f6ea65fce0a90)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41441535129 for 8a691448a99328900f46b3d4e78f6ea65fce0a90)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41441535129 for 8a691448a99328900f46b3d4e78f6ea65fce0a90)
- ci-result: `not a check` passed (Check results for 8a691448a99328900f46b3d4e78f6ea65fce0a90 come from the CI run https://github.com/doodla/athena/actions/runs/41441535129 (502s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
