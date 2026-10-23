# Job 20261023-155649-fy1x: verify "Accept D112: records branch athena/records, readers section, open points"

## Request (verbatim)

> Revise docs/design/records-history.md and accept D112 in docs/decisions.md, with these three changes decided under Sri's delegation (BRIEF.md addenda of 2026-10-23): (1) the records history is a branch named athena/records with its own root, not the custom ref refs/athena/records, so the host's branch rules can forbid force-pushes and deletion and a plain clone carries it; update every place the design names the ref, and the A4 row of the options table. (2) Add a section on readers: every part of Athena that reads records today from records/ in the working tree (status, forecasting and the reference forecaster, scoring, retry carry, audit, introspection, the study site generator) and how each reads from the records branch after the cutover (for example a worktree of athena/records kept at a fixed path), with the job in the build plan that moves them. (3) State that the pre-merge status depends on the host requiring branches to be up to date before merging, so a squash's parent equals the head's base. Fill the open points: pending window 15 minutes; cutover after build job 6; two weeks between job 7 and job 8. D112's status becomes accepted, citing those addenda. Only docs/design/records-history.md, docs/decisions.md and CHANGELOG.md change.

## Reading (accepted by the user)

Revise the records-history design so the records history is a branch named athena/records, add a readers section and a pre-merge up-to-date requirement, and fill the three open points. Then mark D112 accepted, citing the BRIEF.md addenda of 2026-10-23, and add a CHANGELOG entry. Audience: Sri, who decides, and anyone checking the design.

Assumptions:
- The addenda in BRIEF.md of 2026-10-23 exist and authorise these decisions; the producer cites them without editing BRIEF.md.
- 'Cutover after build job 6' matches the existing plan, where job 7 is the cutover; the wording is made explicit.
- Reader locations come from the repository's code. The producer greps for reads of records/; the section describes them and no code changes.
- Goal: an accepted, internally consistent design that later build jobs follow. Nothing is built.

## Acceptance criteria

1. [content] No occurrence of `refs/athena/records` or `refs/athena/*` remains in docs/design/records-history.md or the D112 entry, except where explaining the change from the earlier custom ref. Every place that named the ref (§3 recommendation and assumption, §6.1, §7 table, §10 job 1, §12) now names the branch `athena/records` with its own root. The A4 row says a plain clone carries it.
2. [content] The design states why a branch: the host's branch rules can forbid force-pushes and deletion, and a plain clone carries it. The host-accepts-`refs/athena/*` assumption and its open point are replaced by the host-rules requirement.
3. [content] A new readers section lists each reader that reads records/ from the working tree today: status, forecasting and the reference forecaster, scoring, retry carry, audit, introspection, and the study site generator. For each it gives where it reads now (a code path found in the repository) and how it reads after the cutover, for example a worktree of athena/records at a fixed path. It names the build-plan job that moves it. The build plan in §10 is updated so some job owns this work.
4. [content] The design states that the pre-merge status depends on the host requiring branches to be up to date before merging, so the squash's parent equals the head's base. The text names what happens if the host does not require it.
5. [content] Open points are filled: pending window 15 minutes (§5.2 states it as the value, not an example), cutover after build job 6, and two weeks between job 7 and job 8. §10 and §12 are consistent with these; §4 and §12 no longer leave these points open.
6. [content] The design's Status line and D112 in docs/decisions.md say accepted, citing the BRIEF.md addenda of 2026-10-23. D112's Decision, Tradeoff and Provenance text uses the branch name and no longer says 'proposed' or 'not accepted'.
7. [content] CHANGELOG.md has an entry recording the acceptance and the three changes. The existing entry that says 'Awaiting Sri's decision' is updated or superseded. git diff shows changes only in docs/design/records-history.md, docs/decisions.md and CHANGELOG.md.
8. [behavior] Run `git status` and `git diff --stat`: only those three paths changed. Run `cargo test` and confirm it passes, in case a test validates decisions.md format.

## Expected deliverables

- docs/design/records-history.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any code change, including moving the readers
- Creating the athena/records branch or configuring host branch rules
- Changes to BRIEF.md, athena.toml, or other decisions
- Measuring the check time

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: Prose is written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The readers section cites the code paths read; unmeasured figures stay labelled reported or estimate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the three named files change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A verifier checks the diff against the criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the reader table content and the stated rules, not just that a section exists.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design keeps its alternatives and reversal cost.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff and search the final files with Grep for `refs/athena`. Run: `git diff docs/design/records-history.md docs/decisions.md`. Observes: Remaining matches only in the sentence explaining the change from the earlier custom ref; §3 (recommendation, assumption), §6.1, §7 table, §10 job 1, §12 and the A4 row name `athena/records` with its own root; the A4 row says a plain clone carries it.
- Criterion 2: Read the diff for the why-a-branch text. Run: `git diff docs/design/records-history.md`. Observes: Text says branch rules can forbid force-pushes and deletion and a plain clone carries it; the refs/athena/* assumption and its §12 open point are gone, replaced by the host-rules requirement.
- Criterion 3: Check each reader row against the code with Grep (store.rs:239, introspect.rs:316, scoring.rs:14, reading.rs:1058, auditor.rs:236, status.rs:503, xtask/src/walltime.rs:83, xtask/src/site.rs:17). Run: `git diff docs/design/records-history.md`. Observes: New section has a row for status, forecasting and the reference forecaster, scoring, retry carry, audit, introspection and the site generator, each with a real current path, an after-cutover read and an owning job; §10 names the job that does this work; the site row states what the code actually reads.
- Criterion 4: Read §5.1 and the host-rules requirement. Run: `git diff docs/design/records-history.md`. Observes: States the pre-merge status depends on the host requiring branches to be up to date, so the squash's parent equals the head's base, and says what happens if not.
- Criterion 5: Search the final file for `15 minutes`, `after build job 6`, `two weeks`, and for 'example' and 'Open points'. Run: `git diff docs/design/records-history.md`. Observes: §5.2 states 15 minutes as the value; §4 and §10 state the cutover after job 6 and the two weeks between jobs 7 and 8; §12 no longer lists these as open.
- Criterion 6: Read the Status line and the D112 entry; search for 'proposed' and 'not accepted'. Run: `git diff docs/design/records-history.md docs/decisions.md`. Observes: Status and D112 say accepted, citing the BRIEF.md addenda of 2026-10-23; Decision, Tradeoff and Provenance use `athena/records`, with no 'proposed' or 'not accepted' wording.
- Criterion 7: Read the diff and the Unreleased section. Run: `git diff CHANGELOG.md`. Observes: A new entry records the acceptance and the three changes; the 'Awaiting Sri's decision' line is updated or superseded.
- Criterion 8: Run git status, then git diff --stat, then cargo test --locked. Run: `git status`. Observes: Only docs/design/records-history.md, docs/decisions.md and CHANGELOG.md are changed; cargo test passes, including any test of decisions.md format.
  Fails without the behavior: If decisions.md format were broken, a test under xtask/tests would fail; if another file were edited, git status would list it.

Planned files:

- docs/design/records-history.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base f75bc6245165 → candidate 50aa3c19afb2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |  12 +++-
 docs/decisions.md              |  28 +++++---
 docs/design/records-history.md | 142 ++++++++++++++++++++++++++++++-----------
 3 files changed, 131 insertions(+), 51 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (976 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
