# Job 20261103-101228-2485: verify "Retry owner decision briefs: distinct fixture sources, fetch records in"

## Request (verbatim)

> try again on 20261103-092929-s7ts
> The last attempt met 8 of 9 criteria. Fix the two remaining points and keep everything else:
> - Give the two fixture proposals different sources (and different titles, rungs and evidence), and assert each brief shows its own proposal's source, so a hard-coded source line would fail the test.
> - The Pages workflow in .github/workflows/ checks out only main, so the published decisions page would always say the records branch is missing. Fetch the records branch in that workflow before the site is rendered, and add a test or workflow check that the fetch is present.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

Retry of job 20261103-092929-s7ts for the owner, who reviews Steward proposals on the published site. Redo the owner-style doc and xtask decisions page, and fix the two findings: fixture proposals now differ in source with per-brief source assertions, and pages.yml fetches the records branch before rendering, guarded by a test.

Assumptions:
- Job 20261103-092929-s7ts is retried from base; its work is kept as the approach and re-created: docs/conventions/owner-style.md, xtask/src/decisions.rs, xtask/src/site.rs, xtask/src/main.rs and CHANGELOG.md all remain deliverables, with the original decisions-page criteria (verbatim style doc, GitHub source_url links, 120-word and no-recommendation refusals, not-recorded evidence, role-limit recommendations, index link) kept unchanged.
- Finding 1 (major, same fixture source) is fixed by giving each fixture a different source (e.g. Reference::path and Reference::job) and asserting each brief shows its own source line. Finding 2 (Pages checks out only main) is fixed by editing .github/workflows/pages.yml to fetch athena/records into refs/heads/athena/records (RECORDS_REF in crates/athena-git/src/records_branch.rs) before 'cargo run -p xtask -- site', plus an xtask test reading the workflow file and asserting the fetch precedes the render step.
- Other findings of 20261103-092929-s7ts (duplicated role_limit, build() failing whole site on one bad proposal, hand-built record path, word count excluding Record/command lines) are minor/nit; the producer should fix the cheap ones (derive path from canonical_path or the tree entry, count all displayed prose) and need not change athena-core, which stays out of scope.
- Code survey: read crates/athena-git/src/records_branch.rs (resolve_tip returns Missing when neither refs/heads/athena/records nor refs/remotes/origin/athena/records exists), .github/workflows/pages.yml (default checkout, then xtask site), and xtask/src/ (site.rs, main.rs). Records are read through athena-data check_record_file/AnyRecord; no new record format or data file is added, so no allow-list change.
- The new refusals (over-long brief, missing recommendation) apply only inside the new decisions renderer: expected 0% of newly refused readings.
- pages.yml is a workflow file and is a deliverable named in a criterion; athena.toml and defaults.toml do not change.

## Acceptance criteria

1. [content] docs/conventions/owner-style.md exists and contains the owner's style paragraph verbatim, including 'Show me with big pictures and little words without witholding key details' and 'dont use weird metaphors'.
2. [content] CHANGELOG.md has a new entry describing the owner decision briefs page and the Pages records-branch fetch.
3. [content] The diff shows xtask reads WorkItem proposals and role records through athena-data (check_record_file/AnyRecord) with no new file format and no new dependency beyond what xtask already has.
4. [behavior] Run `cargo test -p xtask`: a test builds briefs from two fixture proposals that differ in title, rung, source (different kinds or paths) and evidence, and asserts each brief's title, rung, source line and evidence line match its own proposal and differ from the other brief, so a hard-coded source line would fail; briefs contain no fixed cost, risk or 'Approve'.
5. [behavior] Run `cargo test -p xtask`: with role fixtures, an in-limit and an above-limit proposal at the same rung give exact, different recommendation lines (the in-limit one not 'Decide yourself'), and a proposal with an unrecorded role says the role is not recorded.
6. [behavior] Run `cargo test -p xtask`: every record link begins with the Athena GitHub base (ATHENA_URL) and contains the records tip sha, and no href is relative.
7. [behavior] Run `cargo test -p xtask`: a brief over about 120 words of prose (counting every displayed prose line) is refused with an error, and one with no recommendation line is refused; expected 0% of newly refused readings outside the new renderer.
8. [behavior] Run `cargo test -p xtask`: a proposal with no evidence renders an explicit 'not recorded' statement, and a checkout without the records branch lists no proposals and the page says so.
9. [behavior] Run `cargo test -p xtask`: the index links to the decisions page, build() writes decisions.html next to index.html, and the existing study page tests pass unchanged.
10. [content] .github/workflows/pages.yml fetches the athena/records branch (into refs/heads/athena/records or the tracking ref) in a step before 'cargo run --locked -p xtask -- site', with the existing checkout pins and permissions otherwise unchanged.
11. [behavior] Run `cargo test -p xtask`: a test reads .github/workflows/pages.yml and fails if the records-branch fetch is absent or comes after the site render step.

## Expected deliverables

- docs/conventions/owner-style.md (create)
- xtask/src/decisions.rs (create)
- xtask/src/site.rs (modify)
- xtask/src/main.rs (modify)
- CHANGELOG.md (modify)
- .github/workflows/pages.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Parts 2 and 3 of the owner decision briefs series
- Approving or rejecting proposals from the page
- Cost or risk figures not present in the proposal's fields
- Changes to athena.toml or defaults.toml
- Any new record or file format
- Changes to athena-core, such as moving role_limit into athena-data or edits to split_open
- Repository-wide fmt, clippy or test runs as their own criteria

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust in xtask.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, changelog and comments written for an outside reader.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has tests; fmt, clippy and test stay green but are not listed as separate criteria, per the request.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered briefs and the workflow content by test, not file existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The published decisions page is made able to list proposals, checked by the workflow-fetch test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the previous deliverables plus pages.yml change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 6c04a6a8ae23 → candidate 8dc0ccf4a681. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/pages.yml     |   8 +-
 CHANGELOG.md                    |  14 +
 docs/conventions/owner-style.md |  29 ++
 docs/design/explainer-site.md   |   6 +
 xtask/src/decisions.rs          | 659 ++++++++++++++++++++++++++++++++++++++++
 xtask/src/main.rs               |   3 +-
 xtask/src/site.rs               |  29 +-
 7 files changed, 735 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (2075 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
