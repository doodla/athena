# Job 20261023-052543-9bw7: Design a separate history for Athena's records

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

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
