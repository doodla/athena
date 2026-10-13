# Job 20261013-203122-td2v: verify "Revise parallel-jobs design to answer review findings F1-F8"

## Request (verbatim)

> Revise docs/design/parallel-jobs.md to answer its review (docs/design/reviews/parallel-jobs-review-1.md), findings F1 to F8. F1 is now addressed by native PR landing, which shipped today: each job lands through its own pull request by squash, and after each landing Athena rebuilds the local branch on the merged commit (docs/design/native-pr-landing.md). Build on that. Prefer running each job as its own process over sharing Athena across threads, unless the design shows threads are safe (F2). Say how CHANGELOG.md, docs/decisions.md numbering and other shared append-only files stop colliding (F3). Design only; the build is a later job.

## Reading (accepted by the user)

Rewrite docs/design/parallel-jobs.md to answer review 1 (F1-F8). It builds on native PR landing (one PR per job, squash, local branch rebuilt after each landing), recommends one process per job over shared threads, and adds a shared-append-only-files section. The change is design only; no code.

Assumptions:
- The review file's line numbers may have drifted after native PR landing shipped, so the producer re-reads the cited source before correcting them.
- docs/design/native-pr-landing.md describes the shipped mechanism, and the revision follows it rather than redesigning it.
- The design stays at 'proposed, review pending', ready for a second review; the revision does not mark itself accepted.
- The roadmap and CHANGELOG are changed only if the repository's convention requires it for a design-only revision.

## Acceptance criteria

1. [content] F1: the design drops the claim that disjoint change jobs land through local replay. It states that each job lands through its own pull request by squash, and that Athena rebuilds the local branch on the merged commit after each landing. It cites docs/design/native-pr-landing.md. It says what happens to a job whose base is behind the new HEAD, and §7, the open question on plan, and the build order (§10) are rewritten to match.
2. [content] F2: the design recommends one child process per job as the default. It lists why threads are not chosen (AgentDriver and Ui have no Send/Sync bound, a caught panic is not enough isolation) and says what a spike would have to show to restore threads. The process model covers how the launcher spawns workers, collects results and exit codes, and handles the kill switch.
3. [content] F3: a section on shared append-only files (CHANGELOG.md, docs/decisions.md numbering, README.md) states concrete rules. These cover which job writes them, how number allocation avoids two jobs taking the same number, how the pool serializes jobs that name the same file, and what a collision test must check. It says which of these the landing check enforces and which are only hints.
4. [content] F4-F8 are each answered. Duplicate job refs are rejected up front after resolving to ids. The kill-switch check comes before the job lock. The design prints the worst-case spend (N times the per-job cap) at the start of a run. Line references are corrected (run.rs 698, land_or_block 1165-1307, retry_landing 1424-1433), or the design says how the references now stand after native PR landing. The seven open questions are settled or carried forward with a stated decision.
5. [content] The design status line and any review-response table map every finding F1-F8 to the section that answers it. Claims cite their sources, and numbers are labeled as estimates. The only files changed are docs, with no source code or governance files.
6. [behavior] Run `git diff --stat` and `git status`: only Markdown files under docs/ (and optionally CHANGELOG.md) changed. Run `cargo fmt --check` and `cargo test`: both still pass.

## Expected deliverables

- docs/design/parallel-jobs.md (modify)
- docs/design/roadmap.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any code, tests, or build of the parallel-jobs feature
- Changing native-pr-landing.md or the review file
- Running the spike on thread safety
- Editing governance files, records/, or athena.toml

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The change revises a design document in response to review, and the build stays a later job.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite the source files and lines read; numbers are labeled as estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused edit to one design document.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The document is written for an outside reader, with the review findings summarized in it.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Collisions on shared files are handled by a rule on who writes them, not by a fix for one instance.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The review and the native PR landing design supply the missing detail.

## The change

Base 43661fb8d76f → candidate 313ce0491a5e. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                 |   8 +
 docs/design/parallel-jobs.md | 661 ++++++++++++++++++++++++++-----------------
 docs/design/roadmap.md       |   8 +-
 3 files changed, 416 insertions(+), 261 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (376 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
