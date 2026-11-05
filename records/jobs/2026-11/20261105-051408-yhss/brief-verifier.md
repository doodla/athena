# Job 20261105-051408-yhss: verify "Show only open, above-limit proposals on the decisions page"

## Request (verbatim)

> try again on 20261103-231808-3gkq

## Reading (accepted by the user)

Retry of rejected job 20261103-231808-3gkq. The owner reading decisions.html must see only proposals that are open and above the raising role's limit; with none, one line 'Nothing waiting for you'. The producer first reproduces on the base, names the real cause with record evidence, and the changelog states only what the records show.

Assumptions:
- Code survey: crates/athena-core/src/work_decisions.rs (in_limit at line 66 uses role_limit and may_change; approve_in_limit; commit of decisions) and crates/athena-data/src/roles.rs (may_change at line 57) define the limit rule; xtask/src/decisions.rs has render(), locate() and read_snapshot() for the page, and xtask does not depend on athena-core. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by calling render() and locate() on a temp git repo.
- Goal and audience: the owner reading the published decisions page at https://doodla.github.io/athena/decisions.html should see only items needing their decision.
- Previous job 20261103-231808-3gkq: its candidate edits to xtask/src/decisions.rs, CHANGELOG.md and xtask/src/site.rs are dropped and redone from the base. Its findings are addressed as follows. (1) Guessed cause plus out-of-scope schedule trigger: the changelog instead names the evidenced cause, that the 3 Nov page was built by code reading decisions only from the records branch, fixed by the later commit 'Make the decisions page read approve/reject decisions from main' (3b1f8c1). No pages.yml change. (2) The ne91 example: ne91 has approval record 20261027-074524-5j98, so it is never called undecided; the within-limit filter is described as a separate guard. (3) No recorded failing run on the base: the producer first runs the new tests on the unchanged base and reports which fail. Since the base already drops decided proposals, only the within-limit assertion can fail there. Decided-proposal tests are labelled regression guards, not fail-before tests. (4) Rejection fixture: reject-work writes to the current branch (main) through commit_records, and the fixture matches that. (5) Over-long module doc line 6 is rewrapped.
- The earlier-failed prevention (a criterion that the fix test fails on old code) is met only for the within-limit part, which the verifier can see through the base's behavior in the diff. The changelog may state only what the records and history in the repository show.
- xtask/src/site.rs and docs/architecture.md are kept as optional deliverables, changed only if the page wording or the described behavior requires it.

## Acceptance criteria

1. [behavior] A test in xtask/src/decisions.rs builds an approved, a rejected and a within-limit open proposal using the real WorkItem and WorkDecision shapes. It asserts render() contains 'Nothing waiting for you', contains no '<article', and has no 'waiting</h2>' count heading. Run: cargo test -p xtask decisions.
2. [behavior] A test with approved, rejected, open above-limit and open below-limit proposals asserts render() shows exactly one '<article' card, for the open above-limit one, under the heading '1 waiting'. Run: cargo test -p xtask decisions.
3. [behavior] A test through locate() on a temp git repo writes the approval where approve-work writes it (records branch) and the rejection where reject-work writes it (current branch, main, via commit_records), and shows both decided proposals absent from the page. Run: cargo test -p xtask decisions.
4. [content] The recorded check output shows the new tests run on the unchanged base. The within-limit test fails there. Tests for decided proposals are described as regression guards where they pass on the base, and no criterion or comment claims they failed on the base.
5. [content] The CHANGELOG.md entry says the 3 Nov page was built by code that read work decisions only from the records branch. It names decisions 20261027-074524-5j98, 20261030-081855-kh40 and 20261030-081746-a1he and the fixing commit 3b1f8c1. It does not call ne91 undecided, proposes no workflow change, and says only what the diff changes. Check with git diff CHANGELOG.md and git log.
6. [content] The empty state in render() is one line reading 'Nothing waiting for you', replacing 'No proposals waiting'. The missing-records-branch message 'No proposals listed' stays distinct. The module doc comment states only open, above-limit proposals are listed, with wrapped lines of consistent width. Conflicting tests are updated. Check with Grep and git diff.
7. [behavior] At the repository root, cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing records/ files, or recording decisions for work items 20261016-055247-qyd9, 20261017-204324-jtd2 or 20261018-000648-ne91
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows including pages.yml
- Job 20261030-081751-edsg and its docs and design work: not continued, nothing kept
- Changing approve-work or reject-work behavior or the record schemas in crates/athena-data and crates/athena-core
- Redesigning the card layout or the 160-word brief limit
- Kept from earlier candidates (jobs 20261103-201526-t4qv and 20261103-231808-3gkq): nothing; their edits to xtask/src/decisions.rs, CHANGELOG.md and xtask/src/site.rs are dropped and redone

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test, clippy and fmt --check to pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer names the cause with record evidence; the changelog states it.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert rendered output, not wiring.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test -p xtask decisions.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and changelog written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the decisions page module.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 755cc6df8979 → candidate e4770beb268d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md           |   9 +++
 xtask/src/decisions.rs | 150 +++++++++++++++++++++++++++++++++++++++++++++----
 xtask/src/site.rs      |   5 +-
 3 files changed, 151 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-051408-yhss/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-051408-yhss/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41973048321 for e4770beb268d3220003c13f0f16c4b2c6142e71c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41973048321 for e4770beb268d3220003c13f0f16c4b2c6142e71c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41973048321 for e4770beb268d3220003c13f0f16c4b2c6142e71c)
- ci-result: `not a check` passed (Check results for e4770beb268d3220003c13f0f16c4b2c6142e71c come from the CI run https://github.com/doodla/athena/actions/runs/41973048321 (501s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: At the repository root, cargo test, cargo clippy --all-targets and cargo fmt --check all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
