# Job 20261027-055334-x7yx: verify "Run error-review backfill and evidence derivation at drain start and"

## Request (verbatim)

> Priority: urgent
> Make the error review's backlog pass and Jev's evidence labeling run on their own, with no person running a command (Sri, 2026-10-27: running the backfill 'should be Athena's job'). At the start of every athena drain and at each Hestia sweep, run the idempotent error-review backfill (athena hestia review --backfill) and the evidence-label derivation (athena score-jev --derive), and print a one-line summary of what each did. Supersedes request 20261026-224929-6erm (close it, citing this job); include its fix that recurrence-after-prevention labels are stable so reruns write nothing new.
> Acceptance:
> - A drain test shows both passes run at drain start, and a second drain with no new records writes nothing.
> - A test shows a recurrence-after-prevention label is not rewritten on a rerun with unchanged evidence.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/evidence_labels.rs, CHANGELOG.md

## Reading (accepted by the user)

At the start of every drain and in each Hestia sweep, automatically run the idempotent error-review backfill and the evidence-label derivation, and print a one-line summary of each. Also make recurrence-after-prevention label bases stable by dropping the changing closed-job count from the basis detail, so reruns write nothing. Close request 20261026-224929-6erm, citing this job. Audience: Sri and the verifier.

Assumptions:
- Cause: evidence_labels.rs recurrence_vote puts the count of closed jobs in LabelBasis.detail, and derive() skips rewriting only when basis equals the existing basis, so the count changing as jobs close causes fresh copies. Fix: state the threshold (STOPPED_MIN_JOBS) rather than the live count, so the basis is stable while the evidence is unchanged.
- The drain hook is a new Steps method with a default no-op (RealSteps implements it by calling Athena::error_review_backfill and derive_labels), called in drain_with before the queue is read; the one-line summaries go through ui or the report notes.
- hestia_sweep calls the same two passes; a pass failure is reported in the summary line and does not abort the drain or sweep.
- Closing request 20261026-224929-6erm is done by the landing process citing this job, not by a repo file edit.
- Its asked-for status display (scored-set size, last derived time) is not part of this request's Files list, so it is left out.
- Athena::error_review_backfill's existing idempotence (a second run changes nothing) is relied on, not rebuilt.

## Acceptance criteria

1. [behavior] A drain test in crates/athena-cli/src/drain.rs shows both passes (backfill and derivation) run at drain start and each prints a one-line summary; a second drain with no new records writes nothing (no new commit or records). Run: cargo test -p athena-cli drain.
2. [behavior] A test in crates/athena-core/src/evidence_labels.rs shows a recurrence-after-prevention label is not rewritten on a rerun after more jobs have closed with otherwise unchanged evidence (derive returns no labels). Run: cargo test -p athena-core evidence_labels.
3. [behavior] A hestia.rs test shows a sweep runs the backfill and derivation and a repeat sweep over unchanged records changes nothing. Run: cargo test -p athena-core hestia.
4. [content] The recurrence-after-prevention basis detail no longer contains the live count of closed jobs, so it is the same text whenever the evidence is the same.
5. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
6. [content] CHANGELOG.md has an entry describing automatic backfill and derivation at drain start and sweep, and the stable recurrence label, and noting that it supersedes request 20261026-224929-6erm.
7. [content] Doc comments in hestia.rs and drain.rs that say a person runs athena hestia review --backfill or athena score-jev --derive are updated to say Athena runs them; the CLI commands stay available.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/evidence_labels.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Showing scored-set size or last-derived time in athena status (from the superseded request)
- Changing how the backfill ranks or queues causes
- Changing the manual CLI commands
- Adding a Jev answer or making Jev scorable

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and the full test suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the passes ran and that reruns write nothing, not that wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The label instability is fixed at its cause, the changing count in the basis, not by deduplicating afterward.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Second-drain and rerun tests check that nothing new is written.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the four named files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The passes run in code at drain start and sweep, not by a documented manual step.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 7a59958d3bd9 → candidate 4cf861f110d7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   7 ++
 crates/athena-cli/src/drain.rs            |  91 ++++++++++++++++++++++
 crates/athena-cli/src/main.rs             |   3 +-
 crates/athena-cli/tests/drain.rs          | 121 ++++++++++++++++++++++++++++++
 crates/athena-core/src/evidence_labels.rs |  51 ++++++++++++-
 crates/athena-core/src/hestia.rs          |  98 +++++++++++++++++++++++-
 crates/athena-core/src/status.rs          |   4 +-
 7 files changed, 366 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1426 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
