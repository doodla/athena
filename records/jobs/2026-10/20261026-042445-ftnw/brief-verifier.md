# Job 20261026-042445-ftnw: verify "Design a no-behavior-change structure pass for gh.rs and run.rs"

## Request (verbatim)

> Priority: high
> Design (no code yet) a structural pass on Athena's two most-changed files, with no behavior change. crates/athena-core/src/gh.rs is 5,276 lines (about 2,600 production, 251 functions) mixing the host API, the landing flow, early pull requests and CI waiting, and records-branch writes; crates/athena-core/src/run.rs has produce_and_verify at 724 lines plus five other functions over 180 lines. Both changed in about 10 of the last week's landings, and this week's breakages (land refusing every pull request, the cutover breaking pull-request checks, the early pull request's timing) were in them; critics repeatedly found missed callers there. Propose module boundaries (for example host client, landing, early pull request and CI wait, records writes; and producer, checks, verification and send-back stages for run.rs), the order of moves as jobs each sized for one producer run, and how each move is shown to change nothing (the existing tests pass unchanged; no test edited except imports). This also serves Sri's request to reconsider the design every few jobs (20261017-205558-dn2e).
> Acceptance:
> - docs/design/structure-pass.md names each new module, what moves into it, and the order of jobs.
> - A proposed entry in docs/decisions.md.
> Files: docs/design/structure-pass.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Write docs/design/structure-pass.md proposing module boundaries for gh.rs (host client, landing, early PR and CI wait, records writes) and run.rs (producer, checks, verification, send-back), an ordered list of single-producer-sized move jobs, and a per-move proof of no behavior change. Add a proposed (not accepted) entry to docs/decisions.md and a CHANGELOG.md line. Audience is Sri, who will approve the jobs; no code changes.

Assumptions:
- Design only: no Rust code is moved in this job; the moves become later jobs.
- Sri is the audience and approves the plan before jobs are queued.
- Line counts in the request are taken as stated but the doc re-measures them.
- Decision entry is marked proposed since the owner has not accepted it.

## Acceptance criteria

1. [content] docs/design/structure-pass.md names each proposed module for gh.rs and run.rs, lists which functions/types move into each (cited from the current files with measured line/function counts), and gives alternatives considered.
2. [content] The doc orders the moves as numbered jobs, each sized for one producer run, with dependencies and the riskiest moves (caller-heavy functions) identified, and splits produce_and_verify into stages by named seams.
3. [content] For each move the doc states how no behavior change is shown: existing tests pass unchanged, only import/use lines edited in tests, public paths re-exported so callers do not change, cargo clippy and fmt --check clean, and a caller check (grep of old paths) listed.
4. [content] docs/decisions.md has a new entry marked proposed, with context, decision, alternatives and consequences, linking the design doc and the design-review cadence job 20261017-205558-dn2e.
5. [content] CHANGELOG.md has an entry for the design doc; no source, test, athena.toml, defaults.toml or workflow file is changed.
6. [behavior] Run git diff --stat / git status: only the three listed docs files changed; cargo test and cargo fmt --check still pass.

## Expected deliverables

- docs/design/structure-pass.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any code or editing tests
- Changing behavior or public APIs
- Changing the design-review cadence mechanism

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design written with alternatives before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Counts and callers cited from the files, measured or labeled estimates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Each move's no-change proof is a test/grep outcome, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Moves sized as separate single-producer jobs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 1eaf15d522fe → candidate e08e54c482d8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                  |   5 +
 docs/decisions.md             |  35 +++++
 docs/design/structure-pass.md | 301 ++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 341 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (1394 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
