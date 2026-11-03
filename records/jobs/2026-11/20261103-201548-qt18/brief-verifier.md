# Job 20261103-201548-qt18: verify "Rewrite decision cards to lead with a yes/no question and outcomes"

## Request (verbatim)

> Rewrite each card on the decisions page so the owner can tell what they are deciding in five seconds. The owner's words on 3 Nov: 'It's garbo. I don't have a sense of what I'm deciding.' Each card today leads with a problem statement and file paths. Instead each card leads with: the decision as one yes/no question in plain words (for example 'Add a design step before every job?'); one line on what happens if yes and one on what happens if no; cost and risk in a few words; Athena's recommendation and its one-line reason. Evidence paths and record ids go in a collapsed section below. No jargon (no rung names, role names or record kinds in the visible text). Acceptance: a test that renders a card from a sample proposal and checks the question, yes, no, cost and recommendation lines appear before any path; the audience role rates a sample page against the owner-style rules.
> Depends on 20261103-201526-t4qv

## Reading (accepted by the user)

Rewrite each card on decisions.html (xtask/src/decisions.rs) for the owner, who must tell in five seconds what they are deciding: a plain yes/no question, one line for yes, one for no, cost and risk in a few words, and Athena's recommendation with a one-line reason. Evidence paths, record ids and the record link move into a collapsed details section. Prior job 20261103-201526-t4qv (open-and-above-limit filter) has no candidate and no findings; this change builds on the current page and does not redo that filter.

Assumptions:
- Code survey: I read xtask/src/decisions.rs (entry: write -> render(&locate(root)); locate resolves the records branch tip via athena_git::records_branch::resolve_tip, then read_snapshot lists records/work-items, records/work-decisions and records/roles and parses each blob with athena_data::records::check_record_file; proposals decided by a work decision are dropped; Brief::new builds a card, Brief::check refuses over MAX_WORDS=120 or no 'Recommendation:' line; Brief::html emits the card). I read crates/athena-data/src/records.rs (WorkItem has only title, rung, role, source, evidence, key; validate() requires evidence for proposals) and xtask/src/site.rs (DECISIONS_PAGE, CSS, escape, source_url). The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected and the records validation allow-list needs no change. The property (order of lines in a card) is measured directly on the rendered HTML string of Brief::html/render by finding byte offsets of the question, yes, no, cost and recommendation lines versus the first path/record id, not through a stand-in.
- Job 20261103-201526-t4qv: it was only proposed, not verified, with no candidate files and no findings. Nothing from it is kept or dropped; this job edits the same file as it exists in the snapshot, and the filter behaviour (open, above-limit only; 'Nothing waiting for you') is left to that job and not changed here.
- Audience is the owner reading the published decisions page; the goal is a five-second grasp of the decision. Style rules are docs/conventions/owner-style.md.
- Proposal records carry no yes/no, cost or risk fields, so the card text is derived by code from the title, rung and the raising role's recorded limit, with plain-word templates (no schema change). Cost and risk are short plain phrases chosen from the rung, labelled as Athena's estimate, never invented numbers.
- The 'audience role rates a sample page' criterion is met by a test that renders a sample page and checks the owner-style rules mechanically (word limit, no jargon words), plus the existing owner-style check if one exists; no new role is created.
- Jargon banned in visible text (outside the collapsed details): rung names, role names, record kinds such as work item, proposal record, blind spot, job, commit, path.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a new test renders a card (Brief::html) from a sample WorkItem proposal and asserts the byte offsets of the question (ending in '?'), the 'If yes' line, the 'If no' line, the cost/risk line and the recommendation line are all smaller than the offset of the first evidence path, record id or record link.
2. [behavior] Run `cargo test -p xtask`: a test asserts the evidence list, record id and GitHub record link sit inside a collapsed `<details>` element (no `open` attribute) in the card HTML, and nothing visible outside it contains a path or record id.
3. [behavior] Run `cargo test -p xtask`: a test renders a sample page of several cards and checks owner-style rules mechanically: each card's visible words are within MAX_WORDS, the visible text contains none of the banned jargon (rung names, role names, 'work item', 'proposal', 'blind spot', 'job', 'commit', 'path'), and each card has exactly one question.
4. [behavior] Run `cargo test -p xtask`: the recommendation line states Recommend yes or Recommend no with a one-line reason, and a card without a recommendation is still refused by Brief::check.
5. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`: both are clean; and `cargo test --workspace` passes including the existing xtask site tests that read decisions.html.
6. [content] Grep xtask/src/decisions.rs: the old visible lines 'Rung:', 'Raised by:', 'Source:' no longer appear outside the details section, and the module doc comment describes the new card layout.
7. [content] The diff touches no file under records/, athena.toml, defaults.toml or .github/workflows, and adds no record or data file under crates/.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- xtask/src/site.rs (modify)
- docs/conventions/owner-style.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing which proposals are listed (open, above-limit filter, 'Nothing waiting for you'): that is job 20261103-201526-t4qv.
- Adding fields to WorkItem or any record schema, or any file under crates/.
- athena.toml, defaults.toml, .github/workflows, records/ and prompt files.
- Recording or applying decisions from the page; it stays display-only.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust in the xtask crate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered card order and text, not wiring.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments updated for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file's rendering changed, no schema change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test and clippy and inspects rendered output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the owner can tell the decision from the card top.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Order and jargon rules enforced by tests and Brief::check, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 76db3a265f1c → candidate c7777db10e32. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                    |   8 +
 docs/conventions/owner-style.md |  19 ++-
 xtask/src/decisions.rs          | 314 ++++++++++++++++++++++++++++++++--------
 xtask/src/site.rs               |   3 +-
 4 files changed, 273 insertions(+), 71 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41740584744 for c7777db10e324e9af83714d73893ccd8c219cbb9)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41740584744 for c7777db10e324e9af83714d73893ccd8c219cbb9)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41740584744 for c7777db10e324e9af83714d73893ccd8c219cbb9)
- ci-result: `not a check` passed (Check results for c7777db10e324e9af83714d73893ccd8c219cbb9 come from the CI run https://github.com/doodla/athena/actions/runs/41740584744 (611s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`: both are clean; and `cargo test --workspace` passes including the existing xtask site tests that read decisions.html.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
