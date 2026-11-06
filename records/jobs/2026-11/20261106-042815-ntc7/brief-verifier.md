# Job 20261106-042815-ntc7: verify "Amend D152 to match the widened ontology claim-check scan scope"

## Request (verbatim)

> try again on 20261102-223344-sqs9

## Reading (accepted by the user)

Retry of job 20261102-223344-sqs9. Its code (scanned_files now scans BRIEF.md, README.md and docs/*.md) is kept, and the rejection finding is fixed: decision D152 still says the check scans only BRIEF.md and the term's file sources. The audience is the verifier and the next producer.

Assumptions:
- Job 20261102-223344-sqs9 is addressed finding by finding. (1) Major, D152 Decision (1) contradicting the code: fixed by amending D152 in docs/decisions.md (or adding a decision that supersedes it). (2) Minor, two error messages in crates/athena-cli/tests/ontology.rs (about lines 338 and 397) still say 'not BRIEF.md or one of the term's file sources': fixed to name BRIEF.md, README.md, docs/*.md and the term's file sources. (3) Minor, docs/ subfolders not scanned: the reading accepted by the user named docs/*.md and the new request asks only for documentation edits. This is not changed in code. D152's Tradeoff records that docs/ subfolders are scanned only when a term lists them as a source.
- The candidate's two changed files, CHANGELOG.md and crates/athena-cli/tests/ontology.rs, are kept as deliverables. The candidate is not visible in the repository I read: scanned_files there still scans only BRIEF.md plus the term's file sources (line 249). The producer must therefore start from the candidate; if it is absent, it must redo the scanned_files change, the scratch-root test and the header comment before amending the decision.
- Code survey: I read crates/athena-cli/tests/ontology.rs (scanned_files at line 249, term_hits at 266, the two message strings at about 338 and 397) and docs/decisions.md D152 (lines 4111-4157). D152 Decision (1) says 'it scans `BRIEF.md` and the term's file sources'. No record file under crates/ is added, so check_record_file and AnyRecord do not apply. The scan scope is measured directly by running the ontology test against a scratch root.
- Existing decisions that state the old scope must be amended or superseded. Grep docs/decisions.md for 'BRIEF.md and the term' and 'file sources' to find any besides D152. D152's wording is amended in place, with a dated note saying the scope widened and why.
- The widened scan is non-recursive over docs/. D152 should say so, and say that reaching subfolders needs the term to list the file.

## Acceptance criteria

1. [content] docs/decisions.md D152 Decision (1) no longer says the check scans only BRIEF.md and the term's file sources. It states that the check scans BRIEF.md, README.md, every docs/*.md and the term's file sources. Check by reading D152 (grep '^## D152').
2. [content] No decision in docs/decisions.md states the old narrower scope. Grep for 'BRIEF.md` and the term' and 'file sources' and confirm every hit is amended or is the superseding text.
3. [content] D152, or a decision that supersedes it, names the root cause, which is checks of shape and of listed sources and not of every governing source. It also states the tradeoff that docs/ subfolders are scanned only if a term lists them.
4. [content] The two error messages in crates/athena-cli/tests/ontology.rs now name BRIEF.md, README.md, docs/*.md and the term's file sources. Grep for 'is not BRIEF.md or one of' returns nothing.
5. [content] CHANGELOG.md has an entry for the scan-scope change that names the root cause. The header comment in crates/athena-cli/tests/ontology.rs also names it.
6. [behavior] The scratch-root unit test fails against the old scanned_files and passes with the change. A governing doc outside the term's listed sources names a term in a sense its definition lacks, and the test requires it to be flagged. Run: cargo test -p athena-cli --test ontology.
7. [behavior] The real-file ontology test passes on docs/ontology.toml with no new excuses. Run: cargo test -p athena-cli --test ontology. Also run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)

## Out of scope

- Redoing the y179 speed work, or changing secrets.rs, checks.rs or run.rs
- Changing ci.yml, xtask or setup.sh
- Adding ontology terms not caused by a misread
- Changes to athena.toml, defaults.toml or workflows
- Changing any study's concept model or files under studies/
- Recursive scanning of docs/ subfolders (job sqs9's third finding, recorded as a tradeoff instead)
- Building a code check that links decisions to the behavior they describe (the introspection's proposed prevention)

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The decision names the root cause and states the scan scope where the check is defined.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The scratch-root test and the real-file test are run, not just checked for existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the prior job's files plus decisions.md change. Subfolder recursion is left out.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision text is written for an outside reader and cites its sources.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by running the ontology tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2de0882d5c47 → candidate 1e7ca37ba718. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  6 ++++
 crates/athena-cli/tests/ontology.rs | 64 ++++++++++++++++++++++++++++++++++---
 docs/decisions.md                   | 11 +++++--
 docs/ontology.toml                  |  4 +--
 4 files changed, 75 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-042815-ntc7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-042815-ntc7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42163114507 for 1e7ca37ba7181deb3c8c0239586af6ad088649b4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42163114507 for 1e7ca37ba7181deb3c8c0239586af6ad088649b4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42163114507 for 1e7ca37ba7181deb3c8c0239586af6ad088649b4)
- ci-result: `not a check` passed (Check results for 1e7ca37ba7181deb3c8c0239586af6ad088649b4 come from the CI run https://github.com/doodla/athena/actions/runs/42163114507 (697s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
