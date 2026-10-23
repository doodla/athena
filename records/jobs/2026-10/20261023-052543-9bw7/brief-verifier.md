# Job 20261023-052543-9bw7: verify "Design a separate history for Athena's records"

## Request (verbatim)

> Design (no code) moving Athena's records out of the code's commit history into a history of their own, per Sri's words in BRIEF.md's addendum of 2026-10-23 ("Wouldn't a db or a shadow repo solve this entire class of issues?", "Yeah. Also copy over whatever. We shouldn't depend on trailers in commit messages"). Deliver docs/design/records-history.md and a proposed (not accepted) entry in docs/decisions.md for Sri to decide.
> 
> Why: records ride in the same commits as code, and that keeps breaking. Six [history] exempt entries in athena.toml, four of them squash merges (ee4b0dd, f2a8453, 5a73478, 39962ed), and main's history check is red again on the squash 02837e0. A revert of code fails the check because it deletes records. A job's landed outcome is written before the squash exists, so it cannot name the commit that lands, hence the Athena-Verified-Content trailer and fingerprint rules. Every landing forces land to rebuild the work branch. Most of the history check's time (about 65 s locally) is spent validating about 4,000 records at the tip.
> 
> The design must:
> 1. Compare at least: a records branch in this repository (a separate root, never merged into main), a second repository, and a database; recommend one. Weigh the brief's A4 (Athena can be lifted out with its history intact) and A5 (records are machine-readable and live in the repo; every commit traces to its work; secrets scanned across the full history), and D72's records-outside-the-repository mode for repositories Athena does not own.
> 2. Migrate every existing record ("copy over whatever"), and say what happens to records/ on main afterwards and how the history check treats the commits before the cutover.
> 3. Attribute every commit on main without reading commit-message trailers: the records name the commits (for example a landed outcome naming the pull request's head and the commit the host made), written after the merge. Cover attribution at the pull-request level: a squash is valid when its content equals the head of a pull request whose own commits are each a verified landing or a declared intervention from the squash's parent, and say how those commits stay reachable without depending on the host.
> 4. Keep every guarantee the history check gives today (append-only records, nothing unverified reaches main, the prediction precedes the outcome, secrets and personal data) and say where each is now enforced.
> 5. Handle a crash between the host's merge and the record write (reconciliation), and parallel writers to the records history.
> 6. Keep the check's cost from growing with the number of commits: judge only what is new since the last passing state, with a periodic full replay.
> 7. Give a build plan in jobs each sized for one producer run, ordered so main is never left unattributed, and the reversal cost.

## Reading (accepted by the user)

Write docs/design/records-history.md, a design-only document covering all seven required points for moving Athena's records into their own history, and add a Proposed entry to docs/decisions.md for Sri to decide. CHANGELOG.md may also get an entry. No code or configuration changes.

Assumptions:
- Goal: a design Sri can decide on; audience is Sri and the verifier judging the document.
- Design only; no athena.toml change even though the [history] exemptions would later change, which the design describes.
- CHANGELOG.md may get an entry per the changelog default.
- Recommendation is the producer's judgment from the repository; the reading does not presuppose which option.

## Acceptance criteria

1. [content] docs/design/records-history.md compares at least three options (records branch with a separate root in this repo, a second repository, a database), each weighed against BRIEF.md A4, A5 and D72, and recommends exactly one with reasons.
2. [content] The design migrates every existing record, states what happens to records/ on main afterwards, and how the history check treats commits before the cutover (including the six [history] exempt entries and 02837e0).
3. [content] The design attributes each main commit from records alone, with no commit-message trailers: it names the records written after the merge (PR head, host-made commit), gives the pull-request-level squash rule, and says how PR commits stay reachable without the host.
4. [content] A table lists each current guarantee (append-only, nothing unverified reaches main, prediction precedes outcome, secrets and personal data) with where it is enforced after the change.
5. [content] The design covers crash reconciliation between host merge and record write, and parallel writers to the records history.
6. [content] The design makes the check's cost depend only on commits new since the last passing state, with a periodic full replay, and says how this is measured against today's ~65 s / ~4,000 records.
7. [content] The design gives an ordered build plan of jobs, each sized for one producer run, in which main is never unattributed, and states the reversal cost.
8. [content] docs/decisions.md has a new entry with status Proposed (not Accepted) summarising recommendation and alternatives, in the file's existing format and numbering; no existing entry is altered.
9. [behavior] No code or governance files change: run git diff --stat and confirm it lists only docs/design/records-history.md, docs/decisions.md and CHANGELOG.md (no athena.toml, defaults.toml, workflow or Rust files).

## Expected deliverables

- docs/design/records-history.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any code, tests, or implementation of the records history
- Changes to athena.toml, defaults.toml, workflows, or the [history] exempt entries
- Accepting the decision: the entry stays Proposed
- Moving or rewriting any existing records

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The deliverable is the written design with alternatives, proposed for review before anything is built.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite BRIEF.md, athena.toml and measured figures, with estimates labeled.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Two docs files plus the changelog entry; no code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the design's content and the diff, not file existence.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Grep the working-tree document for the three options and the A4/A5/D72 rows and a single recommendation heading. Run: `git status --porcelain`. Observes: Lists docs/design/records-history.md as new; the producer's file, read with Read/Grep (headings 'records branch', 'second repository', 'database', 'A4', 'A5', 'D72', 'Recommendation'), shows a comparison with each option weighed against all three and exactly one recommended option.
- Criterion 2: Grep the working-tree document for the migration section and each exempt sha. Run: `git diff --stat`. Observes: Document (read via Grep for 49c6ac1, 64ecdc9, ee4b0dd, f2a8453, 5a73478, 39962ed, 02837e0, 'manifest', 'records/ on main') has a copy-with-hash-manifest procedure, the state of records/ on main after, and the legacy treatment of the seven commits.
- Criterion 3: Grep the working-tree document for the attribution section, landing record fields, squash rule and reachability mechanism. Run: `git status --porcelain`. Observes: Document (Grep 'landing', 'PR head', 'squash', 'parent', 'trailer') shows records written after the merge naming PR head and host-made commit, the PR-level squash rule, parent-link reachability without the host, and no step that reads commit-message trailers.
- Criterion 4: Grep the working-tree document for the guarantees table rows. Run: `git status --porcelain`. Observes: A table with rows append-only, nothing unverified reaches main (pre-merge status on PR head, plus post-merge attribution check), prediction precedes outcome, secrets and personal data (full-history scan on both histories), each with an enforcement location.
- Criterion 5: Grep the working-tree document for reconciliation and parallel-writer text. Run: `git status --porcelain`. Observes: Document (Grep 'reconcil', 'pending', 'compare-and-swap', 'retry') shows the crash scenario, local-first matching with host fallback, the bounded pending state, and parallel-writer rules.
- Criterion 6: Grep the working-tree document for checkpoint, full replay and measurement plan. Run: `git status --porcelain`. Observes: Document (Grep 'checkpoint', 'full replay', '65 s', '4,000') shows cost bound by new commits, periodic replay, and a measurement plan comparing against the reported ~65 s / ~4,000 records, labelled reported or estimate.
- Criterion 7: Grep the working-tree document for the numbered job list and reversal cost. Run: `git status --porcelain`. Observes: Document (Grep 'Job', 'Reversal cost') shows numbered jobs each sized for one producer run, ordered so landing records and the attribution rule exist before records/ leaves the check, and a stated reversal cost.
- Criterion 8: Inspect the decisions diff. Run: `git diff docs/decisions.md`. Observes: Only added lines at the end: '## D112' with Decision beginning 'Proposed', in Case/Decision/Alternatives/Tradeoff/Reversal cost/Provenance format; no removed or changed lines.
- Criterion 9: List changed and untracked files. Run: `git status --porcelain`. Observes: Exactly CHANGELOG.md, docs/decisions.md modified and docs/design/records-history.md untracked; combined with git diff --stat showing only the two modified docs; no athena.toml, defaults.toml, workflow or .rs file.
  Fails without the behavior: If a config or Rust file were edited, it would appear in git status --porcelain and git diff --stat and the criterion would fail.

Planned files:

- docs/design/records-history.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base fe41321feff2 → candidate 2e86b73589c9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |   3 +
 docs/decisions.md              |  32 +++++
 docs/design/records-history.md | 318 +++++++++++++++++++++++++++++++++++++++++
 3 files changed, 353 insertions(+)
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
