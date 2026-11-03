# Job 20261103-092929-s7ts: verify "Render owner decision briefs from open proposals, with role-limit"

## Request (verbatim)

> try again on 20261103-075205-wbtm

## Reading (accepted by the user)

Retry of job 20261103-075205-wbtm (failed in planning, round 3): add the owner-style doc and a decisions page in xtask that renders one short brief per open Steward proposal for the owner. The recommendation now reads the raising role's limit from role records through athena-data, so in-limit proposals are not told 'Decide yourself'.

Assumptions:
- Goal and audience: the owner reviewing waiting Steward proposals on the published site; briefs are one screen, plain, and use only the proposal's own fields.
- Previous job 20261103-075205-wbtm produced no candidate (no files changed), so nothing is kept or dropped; every deliverable below is built fresh. Its findings are addressed as: (major) the recommendation reads the role limit from role records instead of guessing; (minor) the run command is plain `cargo test -p xtask` so every demonstrating test runs; (nit) the records-link pin is stated as Pin { base: ATHENA_URL, sha: records tip }, and the link test asserts that base.
- Code survey: read crates/athena-core/src/work_decisions.rs (split_open at line 73: open proposals within the raising role's limit are approved without a person), crates/athena-data/src/roles.rs (RoleDefinition, may_change at lines 57 and 106), crates/athena-git/src/records_branch.rs (resolve_tip, RecordsTip) and crates/athena-git/src/lib.rs (ls_tree_under, cat_blobs). Entry point is site::build in xtask; the records tip is resolved first, then records and role records are listed and read in one cat_blobs call, parsed with check_record_file into AnyRecord. No new record or file format is added, so the validation allow-list and the round-trip test are untouched. The recommendation is measured by asserting the exact rendered line per fixture.
- Existing code: crates/athena-core/src/work_decisions.rs, crates/athena-data/src/roles.rs, crates/athena-git/src/records_branch.rs, xtask/src/site.rs (source_url, ATHENA_URL). The recommendation reuses the same limit rule via athena-data's may_change and the role limit (newest version by name, then highest_rung), rather than a per-rung table.
- No new dependencies: xtask already depends on athena-data and athena-git, so xtask/Cargo.toml and Cargo.lock are listed only at low probability.
- Rung/limit sentence choice: role not recorded; rung above the role's limit (decide yourself); rung in the top two (no role may change it, decide yourself); rung within the limit (Athena approves it without you; check only if you disagree). Each brief keeps a first-evidence pointer and the approve-work/reject-work command.

## Acceptance criteria

1. [content] docs/conventions/owner-style.md exists and contains the owner's style paragraph verbatim, including 'Show me with big pictures and little words without witholding key details' and 'dont use weird metaphors'.
2. [content] CHANGELOG.md has a new entry describing the owner decision briefs page.
3. [content] Grep of the diff shows xtask reads WorkItem proposals and role records through athena-data (check_record_file/AnyRecord) and introduces no new file format and no new dependency.
4. [behavior] Run `cargo test -p xtask`: a test builds briefs from two fixture proposals with different title, rung, source and evidence and asserts the briefs differ, with no fixed cost, risk or 'Approve'.
5. [behavior] Run `cargo test -p xtask`: with role fixtures, one proposal within the role's limit and one above it at the same rung produce exact, different recommendation lines; the in-limit one does not say 'Decide yourself', and a proposal with an unrecorded role says the role is not recorded.
6. [behavior] Run `cargo test -p xtask`: a test asserts every record link in the briefs begins with the Athena GitHub base (ATHENA_URL) and contains the records tip sha, and no href is relative.
7. [behavior] Run `cargo test -p xtask`: a brief over about 120 words of prose is refused with an error, and a brief with no recommendation line is refused with an error. Expected 0% of newly refused readings outside the new renderer.
8. [behavior] Run `cargo test -p xtask`: a proposal with no evidence renders an explicit 'not recorded' statement, and a checkout without the records branch lists no proposals and the page says so rather than reporting success.
9. [behavior] Run `cargo test -p xtask`: the rendered index contains a link to the decisions page, build() writes the decisions page next to index.html, and existing study page tests still pass unchanged.

## Expected deliverables

- docs/conventions/owner-style.md (create)
- xtask/src/site.rs (modify)
- xtask/src/decisions.rs (create)
- xtask/src/main.rs (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Parts 2 and 3 of the owner decision briefs series
- Approving or rejecting proposals from the page
- Cost or risk figures not present in the proposal's fields
- Changes to athena.toml, defaults.toml or workflows
- Any new record or file format
- Changes to athena-core, such as new edges or edits to split_open
- Repository-wide fmt, clippy or test runs as their own criteria

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has tests under cargo test -p xtask.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert rendered lines, links and refusals.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The recommendation derives from the role limit read from records, not a guess.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A missing records branch gives an explicit empty-state page, tested.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Doc and briefs carry no personal data.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses athena-data, athena-git and source_url; no new dependency.
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

Base 1073f4bccfa6 → candidate 72f9226bf195. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                    |  12 +
 docs/conventions/owner-style.md |  15 ++
 xtask/src/decisions.rs          | 540 ++++++++++++++++++++++++++++++++++++++++
 xtask/src/main.rs               |   3 +-
 xtask/src/site.rs               |  48 +++-
 5 files changed, 609 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41615119228 for 72f9226bf1950b9b658d6c7867107e5ef38c187d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41615119228 for 72f9226bf1950b9b658d6c7867107e5ef38c187d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41615119228 for 72f9226bf1950b9b658d6c7867107e5ef38c187d)
- ci-result: `not a check` passed (Check results for 72f9226bf1950b9b658d6c7867107e5ef38c187d come from the CI run https://github.com/doodla/athena/actions/runs/41615119228 (602s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
