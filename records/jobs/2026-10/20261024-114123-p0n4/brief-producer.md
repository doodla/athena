# Job 20261024-114123-p0n4: Write landing intent and landing records from athena land --pr

## Request (verbatim)

> Build job 3, part 1, of the records-history plan (docs/design/records-history.md sections 5.2, 5.3, 6.1 and 10, decision D111): athena land --pr writes a landing-intent record (PR number, head sha, head tree, expected base, job ids) to the athena/records branch before it asks the host to merge, and a landing record (PR number, head sha, the commit the host made, merge method, the pull request's commits with their trees and the job and verification covering each, and the intent it fulfils) after the merge, in a records-branch commit that takes the pull request's head as an extra parent so its commits stay reachable. Use the record types in athena-data history.rs and the writer in athena-git records_branch.rs, and push the records branch to the remote. Everything land does today (trailers, records under records/, the rebuild of the work branch) stays as it is. Tests with the fake host: a landing writes both records in order with the right fields; the landing commit's second parent is the PR head; a merge the host refuses leaves an intent and no landing record.

## Reading (accepted by the user)

For the maintainers who run athena land --pr: before the host is asked to merge, land_pr writes a LandingIntent to the athena/records branch. After the merge it writes a Landing record in a records commit whose second parent is the PR head. Both are pushed to the remote, and existing landing behavior is unchanged.

Assumptions:
- Goal: land --pr leaves an auditable intent and landing record pair on athena/records (D111 job 3 part 1). The audience is the maintainer and the later history check.
- The hook points are gh.rs land_pr (before job.host.merge) and after observe_merge reports Landed. Athena::land_pr in gh.rs likely supplies the git handle, since the LandPr struct has none.
- The merge method is read from the host or inferred from the merged commit's shape. The PR's commits come from git between the base and the head. Each commit's job and verification come from its Athena-Job trailer and the job's verification record.
- A failed records write or push before the merge blocks the merge. After the merge it is reported as a warning and never undoes the merge. This is my judgment, not stated in the request.
- Fake-host tests use the existing FakeRunner/PrHost fakes in gh.rs and a real temporary git repo with a bare remote.
- I did not read docs/design/records-history.md sections 5.2, 5.3, 6.1 and 10 in full, so details may differ.

## Acceptance criteria

1. [behavior] A fake-host test of a successful landing through land_pr reads the records branch and finds a landing-intent record and then a landing record, in that order. The intent has the PR number, head sha, head tree, expected base and job ids. The landing has the PR number, head sha, host commit, merge method, commits with tree, job and verification, and the intent's id. Run: cargo test -p athena-core.
2. [behavior] A test shows the records-branch commit that carries the landing record has the pull request's head sha as its second parent, so the PR commits stay reachable from athena/records.
3. [behavior] A test with a fake host that refuses the merge shows the records branch holds the intent record and no landing record, and land_pr still returns its existing error. Run: cargo test.
4. [behavior] The intent is on the records branch, and pushed to the remote, before PrHost::merge is called. A test records the call order. The records branch is pushed to the remote after each write; a test with a bare remote checks it.
5. [behavior] Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged. cargo test, cargo clippy --all-targets and cargo fmt --check are clean.
6. [content] The records use the athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod) and are written through the athena-git RecordsWriter, not a new writer. Each record is validated before it is written.
7. [content] Any new function or module has a doc comment that cites the design doc section it implements.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-cli/tests/external_calls.rs (modify)

## Out of scope

- The history check (athena-gate) accepting or enforcing the new records, and the squash-refusal text in refusal_text (job 3 part 2 or later)
- Declared interventions, checkpoints, legacy exemptions, and import manifests
- Changing trailers, records under records/, or resync_onto_merge
- Changing athena.toml, defaults.toml, or the CI workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in the existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New behavior has tests; cargo test passes and clippy is clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the record contents, their order, and the parent link by running tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing RecordsWriter and history types; no new crates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the landing path.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite the design sections.
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

A fake-host test of a successful landing through land_pr reads the records branch and finds a landing-intent record and then a landing record, in that order. The intent has the PR number, head sha, head tree, expected base and job ids. The landing has the PR number, head sha, host commit, merge method, commits with tree, job and verification, and the intent's id. Run: cargo test -p athena-core.
A test shows the records-branch commit that carries the landing record has the pull request's head sha as its second parent, so the PR commits stay reachable from athena/records.
A test with a fake host that refuses the merge shows the records branch holds the intent record and no landing record, and land_pr still returns its existing error. Run: cargo test.
The intent is on the records branch, and pushed to the remote, before PrHost::merge is called. A test records the call order. The records branch is pushed to the remote after each write; a test with a bare remote checks it.
Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged. cargo test, cargo clippy --all-targets and cargo fmt --check are clean.
The records use the athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod) and are written through the athena-git RecordsWriter, not a new writer. Each record is validated before it is written.
Any new function or module has a doc comment that cites the design doc section it implements.

## Report

End with the producer report as your structured output.
