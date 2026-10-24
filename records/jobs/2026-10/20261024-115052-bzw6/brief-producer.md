# Job 20261024-115052-bzw6: Write landing intent and landing records in land --pr, with push

## Request (verbatim)

> try again on 20261024-114123-p0n4

## Reading (accepted by the user)

Retry job 3 part 1: land_pr writes a LandingIntent to athena/records and pushes it before the host merges, then a Landing record (second parent = PR head) after. Push-failure recovery follows the user's decisions: the remote is the source of truth, the commit is built on the remote tip, pushed with a lease, and the local ref moves only after the push succeeds. The drain.rs caller and the unconfirmed-merge warning are covered.

Assumptions:
- Audience: maintainers who run athena land --pr and the verifier judging it; goal is job 3 part 1 of the records-history plan plus working push-failure recovery.
- Per the user's decisions, the remote is the source of truth: records commit is built on the remote tip, pushed with a lease, local ref moved only after success.
- Merged-but-unconfirmed: no landing record, warning naming the intent; reconcile (job 3 part 2) repairs it and is out of scope.
- Deliverable paths are inferred from the earlier attempt plus drain.rs; I did not re-read the repository.

## Acceptance criteria

1. [behavior] Fake-host test of a successful land_pr reads the records branch and finds a landing-intent record then a landing record, in that order. Intent: PR number, head sha, head tree, expected base, job ids. Landing: PR number, head sha, host commit, merge method, commits with tree/job/verification, intent id. Run: cargo test -p athena-core.
2. [behavior] A test shows the records commit carrying the landing record has the PR head sha as its second parent. Run: cargo test.
3. [behavior] A test with a fake host that refuses the merge shows the records branch (and remote) holds the intent and no landing record, and land_pr returns its existing error. Run: cargo test.
4. [behavior] A call-order test with a bare remote shows the intent is pushed to the remote before PrHost::merge is called, and the landing record is pushed after the merge.
5. [behavior] Rejected-push test: the remote records branch moves between build and push; the commit is rebuilt on the fetched remote tip and retried (bounded attempts), succeeds, and local and remote tips match. The commit is built on the remote tip, pushed with a lease on that tip, and the local ref moves only after the push succeeds. Fails without the recovery code. Run: cargo test.
6. [behavior] Leftover-local-intent test: a local records ref ahead of or diverged from the remote is reset to the remote before writing; after a leftover unpushed local intent plus a remote write, the next land_pr succeeds with no divergence error, and the merge never happens without its intent on the remote.
7. [behavior] Merged-but-unconfirmed test: when the host merged but the merge cannot be confirmed, no landing record is written and a PrReport warning names the intent; a rerun in that state is tested and creates no duplicate records.
8. [behavior] The drain.rs caller of Athena::land_pr behaves the same: a test (or an existing drain test shown passing with the real ledger) shows the same records written.
9. [behavior] A commit between the remote base and the PR head with no landing trailer or verification gives an error naming its sha and saying it lies between the remote base and the PR head. Tested.
10. [behavior] Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged; cargo test, cargo clippy --all-targets and cargo fmt --check are clean.
11. [content] Records use the athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod), are validated before writing, and are written through the athena-git RecordsWriter, not a new writer.
12. [content] docs/design/records-history.md states how sync_from changed (or which distinct recover path was added) and includes a failure-path table: starting state, function called, its cited behavior in that state. New functions carry doc comments citing the design section; CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/external_calls.rs (modify)

## Out of scope

- Job 3 part 2: the reconcile command that repairs merged-but-unconfirmed landings
- Changing the existing land trailers, records under records/, or the work-branch rebuild
- Planning-stage changes to Athena itself (failure-path table in the planner, fallback after round 3)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each new path; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check records, order, parents and recovery happening via fake-host tests.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design doc updated with the recovery decision and failure-path table.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for outside readers.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Recovery fixes the ref-moves-before-push cause, not one instance.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria use commands the verifier can run.
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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

Fake-host test of a successful land_pr reads the records branch and finds a landing-intent record then a landing record, in that order. Intent: PR number, head sha, head tree, expected base, job ids. Landing: PR number, head sha, host commit, merge method, commits with tree/job/verification, intent id. Run: cargo test -p athena-core.
A test shows the records commit carrying the landing record has the PR head sha as its second parent. Run: cargo test.
A test with a fake host that refuses the merge shows the records branch (and remote) holds the intent and no landing record, and land_pr returns its existing error. Run: cargo test.
A call-order test with a bare remote shows the intent is pushed to the remote before PrHost::merge is called, and the landing record is pushed after the merge.
Rejected-push test: the remote records branch moves between build and push; the commit is rebuilt on the fetched remote tip and retried (bounded attempts), succeeds, and local and remote tips match. The commit is built on the remote tip, pushed with a lease on that tip, and the local ref moves only after the push succeeds. Fails without the recovery code. Run: cargo test.
Leftover-local-intent test: a local records ref ahead of or diverged from the remote is reset to the remote before writing; after a leftover unpushed local intent plus a remote write, the next land_pr succeeds with no divergence error, and the merge never happens without its intent on the remote.
Merged-but-unconfirmed test: when the host merged but the merge cannot be confirmed, no landing record is written and a PrReport warning names the intent; a rerun in that state is tested and creates no duplicate records.
The drain.rs caller of Athena::land_pr behaves the same: a test (or an existing drain test shown passing with the real ledger) shows the same records written.
A commit between the remote base and the PR head with no landing trailer or verification gives an error naming its sha and saying it lies between the remote base and the PR head. Tested.
Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged; cargo test, cargo clippy --all-targets and cargo fmt --check are clean.
Records use the athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod), are validated before writing, and are written through the athena-git RecordsWriter, not a new writer.
docs/design/records-history.md states how sync_from changed (or which distinct recover path was added) and includes a failure-path table: starting state, function called, its cited behavior in that state. New functions carry doc comments citing the design section; CHANGELOG.md has an entry.

## Report

End with the producer report as your structured output.
