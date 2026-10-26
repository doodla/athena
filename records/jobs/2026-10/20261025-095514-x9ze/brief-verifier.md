# Job 20261025-095514-x9ze: verify "Retry from candidate when only docs findings block a passing rejection"

## Request (verbatim)

> Priority: high
> Extend retry-from-candidate (3449219) to rejections whose checks all passed and whose only blocking findings are about documentation or other non-code content, even when no criterion is marked unmet: job 20261025-034452-wqp8 passed every check and was rejected for one major finding (decision D122 inserted inside D121 in docs/decisions.md), but its retry started from the base and redid 30 minutes of code. Classify a finding as content-only when every path it cites is a non-code file (Markdown, CHANGELOG, docs/), and use that in the qualifying rule.
> Acceptance:
> - A test shows a rejection with passing checks and only a docs-located major finding retries from the candidate.
> - A finding citing a code file, or a failed check, still retries from the base.
> Files: crates/athena-core/src/run.rs, crates/athena-core/src/verify.rs, CHANGELOG.md

## Reading (accepted by the user)

Extend qualifies_for_tree_retry so a rejected job whose checks all passed, with no behavior-criterion trouble and no reading mismatch, also qualifies when no criterion is unmet but every blocking (blocker/major) finding cites only non-code paths (Markdown, CHANGELOG, docs/). A new content-only classifier on findings lives in verify.rs. Audience: the operator running `athena retry`, who stops losing ~30 minutes to a redo like job wqp8's.

Assumptions:
- The qualifying rule lives in reading.rs (qualifies_for_tree_retry, line ~1059) though the request lists run.rs and verify.rs; reading.rs must change with its tests, and run.rs only if its doc comment or seed wiring needs it.
- A finding's paths come from Finding.location (e.g. 'docs/decisions.md:3005-3053', possibly several paths) and, if location is None, from paths named in summary/evidence; a finding with no recognizable path is NOT content-only (conservative: retry from base).
- Blocking means severity blocker or major; minor/nit findings do not block and are ignored by the rule.
- Non-code means extension .md/.markdown/.txt, a file named CHANGELOG*, or a path under docs/; Cargo.toml, athena.toml, prompts, .rs and workflow files count as code/config, so they retry from base.
- The existing rule (unmet content criteria only) stays; the new branch applies when there are no unmet criteria and at least one blocking finding, all content-only. A rejection with no unmet criterion and no blocking finding still has unknown cause and does not qualify.
- The tree-retry seed's findings text already carries the findings, so the retry producer is told the docs fix needed; no new record schema.

## Acceptance criteria

1. [behavior] A test in athena-core (run with `cargo test -p athena-core`) builds a rejected job with candidate, all checks passed, no unmet criterion, and one major finding whose location is docs/decisions.md:3005-3053; qualifies_for_tree_retry returns true for it.
2. [behavior] Tests show retry still starts from the base (qualifies_for_tree_retry false) when: a major finding cites a .rs file; a finding cites both a docs path and a code path; a check failed; a behavior criterion is unmet; the verifier reported a request/reading mismatch; the candidate has admission violations; or no finding and no unmet criterion exists.
3. [behavior] A test of the content-only classifier (in verify.rs) shows docs/…, *.md, and CHANGELOG.md locations classify as content-only, and src files, Cargo.toml, a missing location with no path, and mixed paths do not; minor findings do not block qualification.
4. [behavior] A test through tree_retry_seed shows a docs-only-finding rejection yields a seed whose findings list includes the docs finding text; `cargo test` and `cargo clippy --all-targets` pass and `cargo fmt --check` is clean.
5. [content] The doc comment on qualifies_for_tree_retry states the new condition (no unmet criterion, all blocking findings cite only non-code files) and that the unknown-cause exclusion still applies otherwise.
6. [content] CHANGELOG.md has an entry under the unreleased section describing that docs-only rejections now retry from the candidate, and that code-file findings or failed checks still retry from the base.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Fixing D122/D121 in docs/decisions.md or re-running job wqp8.
- Changing which verifier findings are raised or their severities.
- Changing the base-commit-unchanged check or the seeding mechanics of a tree retry.
- Adding a new record field or schema version.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the classifier and the qualifying rule; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rule's decisions on constructed rejections, not only that a function exists.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment on the rule updated; CHANGELOG entry written for outside readers.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Generalizes the rule to all content-only blocking findings rather than special-casing D122.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test runs the verifier can execute.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the retry qualification rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in reading.rs tests: take `content_only_rejection()`, set all criteria met, set findings to one Major with location `docs/decisions.md:3005-3053`, assert `qualifies(&job)`. Run: `cargo test -p athena-core docs_only_blocking_finding`. Observes: qualifies_for_tree_retry returns true for a passing-check rejection with no unmet criterion and only a docs major finding.
  Fails without the behavior: Without the new branch, no unmet criterion means false and the assertion fails.
- Criterion 2: Table test over constructed docs-only rejections, each with one change, asserting false: major finding at `crates/athena-core/src/run.rs:10`; one finding with location `docs/a.md crates/x/src/lib.rs`; a check failed; a behavior criterion unmet; request_reading_mismatch set; a candidate violation; no findings and no unmet criterion. Also update the existing case 'no criterion is unmet' in `only_a_rejection_with_every_check_passed_and_content_criteria_unmet_qualifies`, because its fixture has a CHANGELOG.md major finding and would now qualify; clear findings in that case. Run: `cargo test -p athena-core docs_only_blocking_finding`. Observes: Every case returns false, so the retry starts from the base.
  Fails without the behavior: If the rule ignored the finding paths or the other gates, one of these cases would return true.
- Criterion 3: Unit tests in verify.rs: locations `docs/x.md:3-9`, `notes.md`, `CHANGELOG.md` are content-only; `src/lib.rs`, `Cargo.toml`, `athena.toml`, None with prose naming no path, and `docs/a.md src/b.rs` are not. A test that a Minor code finding is excluded by `finding_blocks` and a Minor finding alongside a docs Major still qualifies (in reading.rs). Run: `cargo test -p athena-core finding_is_content_only`. Observes: Classifier returns the stated booleans; minor findings do not block.
  Fails without the behavior: A classifier that accepted any location or ignored severity would fail the mixed, code and minor cases.
- Criterion 4: Test in reading.rs: records_with a docs-only-finding rejection, call tree_retry_seed with 'try again on <id>', assert Some, unmet empty, and findings[0] contains `major (docs/decisions.md:3005-3053):` and the summary. Then run the full checks. Run: `cargo test -p athena-core tree_retry_seed_docs_only && cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`. Observes: The seed carries the docs finding text; all tests pass, clippy and fmt are clean.
  Fails without the behavior: Without the rule extension, tree_retry_seed returns None and the expect fails.
- Criterion 5: Inspect the doc comment on qualifies_for_tree_retry in the diff. Run: `git diff crates/athena-core/src/reading.rs`. Observes: It says: with no unmet criterion, the rule holds when at least one blocker/major finding exists and all blocking findings cite only non-code files; otherwise a rejection with no unmet criterion has an unknown cause and does not qualify.
- Criterion 6: Inspect the diff under `## [Unreleased]`. Run: `git diff CHANGELOG.md`. Observes: An entry (under Changed) says a rejection whose checks passed and whose only blocking findings cite documentation or other non-code files now retries from the candidate; findings that cite code files, or a failed check, still retry from the base. It is written for outside readers.

Planned files:

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- docs/decisions.md (modify)

## The change

Base 92da550e746b → candidate e3b6b70a5032. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   6 ++
 crates/athena-cli/src/drain.rs    |   3 +-
 crates/athena-core/src/reading.rs | 149 ++++++++++++++++++++++++++++++++++++--
 crates/athena-core/src/run.rs     |   3 +-
 crates/athena-core/src/verify.rs  | 134 ++++++++++++++++++++++++++++++++++
 docs/decisions.md                 |  25 +++++++
 6 files changed, 311 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1274 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
