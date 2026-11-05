# Job 20261103-231808-3gkq: verify "Fix decisions page to list only open, above-limit proposals"

## Request (verbatim)

> try again on 20261103-201526-t4qv: its reading was refused because the reader assumed a design file (docs/design/outcome-check-overrates.md) that does not exist at the base commit. The request needs no design file: it changes only which steward proposals the decisions page lists (open and above the steward's limit), read from the approve-work and reject-work decision records.

## Reading (accepted by the user)

Retry of job 20261103-201526-t4qv: the owner reading decisions.html must see only proposals that are open (no approve-work or reject-work record names them) and above the raising role's limit; with none, one line 'Nothing waiting for you'. No design file is needed or assumed. The producer must trace and name why decided proposals reached the published page, and prove the fix with a test that fails on the base.

Assumptions:
- Goal and audience: the owner reading https://doodla.github.io/athena/decisions.html, who should be asked only about proposals that are theirs to decide.
- Previous job 20261103-201526-t4qv (rejected, verdict fail): its candidate changes to CHANGELOG.md and xtask/src/decisions.rs are DROPPED; this retry starts from the base. Its findings are addressed as follows. Major (root cause not named; only an existing filter was moved): the producer must trace and state the cause and fix it there, and the changelog line must say only what the diff does. Minor (in_limit copied into xtask): reuse one definition, or state in the code why xtask keeps its own rule. Minor (records-branch test also passes on old code): the new tests must fail on the base, with the failing run shown in the check output.
- Code survey: crates/athena-core/src/work_decisions.rs (role_limit, in_limit, split_open, approve_work, reject_work) and crates/athena-core/src/commits.rs (commit_records, Committer::advance), plus xtask/src/decisions.rs. read_snapshot() reads work items, work decisions and roles from the records branch tip, then also reads records/work-decisions from HEAD (MAIN_TREE), and drops decided ids with proposals.retain(). render() applies NO limit check, so a within-limit proposal such as ne91 is listed (a certain defect at base). The empty state is currently 'No proposals waiting'. approve_work and reject_work commit decisions through commit_records onto the records branch. xtask does not depend on athena-core. .github/workflows/pages.yml fetches athena/records with --depth=1 only on a push to main or manual dispatch.
- Root-cause leads for the producer to check (not confirmed; records/ is absent from the snapshot): (a) in-limit proposals are never filtered at base; (b) decisions are committed to the records branch, which may not be pushed or may differ in the fetched copy, while read_snapshot's HEAD read of MAIN_TREE assumes main (its comment says so); (c) a decision tied to a dropped job such as 20261030-081751-edsg may exist only where the page never reads; (d) the page is stale because pages.yml runs only on a push to main. The producer must pick the supported cause by reading code and building fixtures, and report it. If the cause lies in workflow or push behavior outside scope, name it in the changelog and report, and still make the renderer correct for every data source it reads.
- Code check: records validation uses check_record_file and AnyRecord in athena-data; the change adds no record or data file, so no allow-list change is needed. The rule is measured directly by rendering Snapshot/Records values and by running locate() on a temp git repo that holds real WorkItem and WorkDecision records.
- A role with no recorded definition counts as above the limit, matching split_open in athena-core.
- This change adds no refusal rule. It removes entries from a rendered page only.

## Acceptance criteria

1. [behavior] A new test in xtask/src/decisions.rs builds an approved, a rejected, and a within-limit open proposal, using the real WorkItem and WorkDecision shapes. It asserts render() contains 'Nothing waiting for you', contains no '<article', and has no 'waiting</h2>' count heading. Run: cargo test -p xtask decisions.
2. [behavior] A new test with approved, rejected, open-above-limit and open-below-limit proposals together asserts render() shows exactly one '<article' card, for the open above-limit one, with heading '1 waiting'. Run: cargo test -p xtask decisions.
3. [behavior] A test through locate() on a temp git repo (records branch plus decision records in the place(s) where approve-work and reject-work really write them) shows the decided proposals absent from the page. The recorded check output shows this test failing on the old code and passing on the new. Run: cargo test -p xtask decisions.
4. [content] The CHANGELOG.md entry names where decided or within-limit proposals entered the page and why they were not excluded, and says only what the diff changes; nearby 'Root cause:' entries are the model. Check with git diff CHANGELOG.md.
5. [content] The empty state in render() of xtask/src/decisions.rs is one line reading 'Nothing waiting for you', replacing 'No proposals waiting'. The missing-records-branch message ('No proposals listed') stays distinct. The module doc comment states that only open, above-limit proposals are listed. Conflicting existing tests are updated. Check with Grep and git diff.
6. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass at the repository root.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing records/ files, or recording decisions for work items 20261016-055247-qyd9, 20261017-204324-jtd2 or 20261018-000648-ne91
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows (including pages.yml; if the cause is there, name it in the report)
- Job 20261030-081751-edsg and its docs and design work (docs/design/outcome-check-overrates.md, docs/decisions.md, docs/design/audience-role.md): not continued, nothing kept
- Changing approve-work or reject-work behavior or the record schemas in crates/athena-data and crates/athena-core
- Redesigning the card layout or the 160-word brief limit
- Kept from job 20261103-201526-t4qv: nothing; its candidate edits to xtask/src/decisions.rs and CHANGELOG.md are dropped and redone

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The reading requires naming where decided and within-limit proposals enter the page and fixing it there, not moving an existing filter.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria use fixtures of real record shapes, require tests that fail on the base, and check the rendered output.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy --all-targets and fmt --check must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that no decided or within-limit proposal appears on the rendered page.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The changelog and doc comment are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the renderer, its tests and the changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the named cargo commands and reads the diff.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 700edebd6cd0 → candidate 060841effb95. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md           |  14 ++++++
 xtask/src/decisions.rs | 133 ++++++++++++++++++++++++++++++++++++++++++++++---
 xtask/src/site.rs      |   5 +-
 3 files changed, 143 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261103-231808-3gkq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261103-231808-3gkq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41971057550 for 060841effb95822173749939572e9c46317eab58)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41971057550 for 060841effb95822173749939572e9c46317eab58)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41971057550 for 060841effb95822173749939572e9c46317eab58)
- ci-result: `not a check` passed (Check results for 060841effb95822173749939572e9c46317eab58 come from the CI run https://github.com/doodla/athena/actions/runs/41971057550 (619s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy --all-targets and cargo fmt --check all pass at the repository root.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
