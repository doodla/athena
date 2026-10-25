# Job 20261024-115923-02gf: Write landing-intent and landing records in athena land --pr

## Request (verbatim)

> try again on 20261024-115052-bzw6

## Reading (accepted by the user)

Retry job 3 part 1 of records-history: gh::land_pr writes and pushes a LandingIntent to athena/records before merging, then a Landing record (second parent = PR head) after. A small constructor builds the real records sink from a Git and its remote; Athena::land_pr (and so drain.rs) calls it, and a test covers that constructor. Audience: the maintainer and verifier checking via cargo test.

Assumptions:
- The remote is the source of truth for athena/records: commit built on the remote tip, pushed with a lease, local ref moves only after the push succeeds; rejected push fetches, rebuilds and retries a bounded number of times.
- A local records ref ahead of or diverged from the remote is reset to the remote before writing.
- No merge-requested marker. If the merge cannot be confirmed, no landing record is written, a PrReport warning names the intent, exit codes stay as today, and reconcile (job 3 part 2) repairs it.
- No fake-runner rig for Athena::land_pr; wiring is proven by testing the constructor that builds the real sink.
- display.rs and main.rs change only if the warning needs one line.
- Records writing sits in gh::land_pr behind the records sink it already takes; existing trailers, records/ and resync_onto_merge behavior are unchanged.

## Acceptance criteria

1. [behavior] A free-function gh::land_pr test with a fake host reads the records branch and finds a landing-intent record then a landing record, in that order. Intent: PR number, head sha, head tree, expected base, job ids. Landing: PR number, head sha, host commit, merge method, commits with tree/job/verification, intent id. Run: cargo test -p athena-core.
2. [behavior] A test shows the records commit carrying the landing record has the PR head sha as its second parent. Run: cargo test.
3. [behavior] A test with a fake host that refuses the merge shows the local and remote records branch hold the intent and no landing record, and land_pr returns its existing error. Run: cargo test.
4. [behavior] A call-order test with a bare remote shows the intent is on the remote before PrHost::merge is called and the landing record after. Run: cargo test.
5. [behavior] Rejected-push test: the remote records branch moves between build and push; the commit is rebuilt on the fetched remote tip and retried (bounded), succeeds, and local and remote tips match; the local ref moves only after a successful push. Run: cargo test.
6. [behavior] Leftover-local-intent test: a local records ref ahead of or diverged from the remote is reset to the remote before writing; the next land_pr succeeds with no divergence error. Run: cargo test.
7. [behavior] Merged-but-unconfirmed test: no landing record is written, the PrReport warning names the intent, exit codes are unchanged, and a rerun creates no duplicate records. Run: cargo test.
8. [behavior] A test of the constructor that builds the real records sink from a Git and its remote shows it returns a working sink writing to the remote records branch; Athena::land_pr calls this constructor (used by drain.rs), so a call passing no sink is not the production path. Run: cargo test.
9. [behavior] A commit between the remote base and the PR head with no landing trailer or verification gives an error naming its sha and saying it lies between the remote base and the PR head. Run: cargo test.
10. [behavior] Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged; cargo test, cargo clippy --all-targets and cargo fmt --check are clean.
11. [content] Records use athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod), are validated before writing, and are written through the athena-git RecordsWriter, not a new writer.
12. [content] docs/design/records-history.md states how sync_from changed (or the distinct recover path) and has a failure-path table (starting state, function, cited behavior); new functions carry doc comments citing the design section; CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/external_calls.rs (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Reconcile of merged-but-unconfirmed landings (job 3 part 2)
- A merge-requested marker
- A fake-runner rig for Athena::land_pr
- Changes to existing trailers, records under records/, or resync_onto_merge
- Changing exit codes in main.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each new path; cargo test, clippy --all-targets and fmt --check clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check records on the branch, order, parents and failure paths, not mere existence.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite design sections; the design doc gets a failure-path table.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing RecordsWriter and history.rs types; no new writer.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Remote-tip build with lease fixes divergence at its cause.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
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

A free-function gh::land_pr test with a fake host reads the records branch and finds a landing-intent record then a landing record, in that order. Intent: PR number, head sha, head tree, expected base, job ids. Landing: PR number, head sha, host commit, merge method, commits with tree/job/verification, intent id. Run: cargo test -p athena-core.
A test shows the records commit carrying the landing record has the PR head sha as its second parent. Run: cargo test.
A test with a fake host that refuses the merge shows the local and remote records branch hold the intent and no landing record, and land_pr returns its existing error. Run: cargo test.
A call-order test with a bare remote shows the intent is on the remote before PrHost::merge is called and the landing record after. Run: cargo test.
Rejected-push test: the remote records branch moves between build and push; the commit is rebuilt on the fetched remote tip and retried (bounded), succeeds, and local and remote tips match; the local ref moves only after a successful push. Run: cargo test.
Leftover-local-intent test: a local records ref ahead of or diverged from the remote is reset to the remote before writing; the next land_pr succeeds with no divergence error. Run: cargo test.
Merged-but-unconfirmed test: no landing record is written, the PrReport warning names the intent, exit codes are unchanged, and a rerun creates no duplicate records. Run: cargo test.
A test of the constructor that builds the real records sink from a Git and its remote shows it returns a working sink writing to the remote records branch; Athena::land_pr calls this constructor (used by drain.rs), so a call passing no sink is not the production path. Run: cargo test.
A commit between the remote base and the PR head with no landing trailer or verification gives an error naming its sha and saying it lies between the remote base and the PR head. Run: cargo test.
Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged; cargo test, cargo clippy --all-targets and cargo fmt --check are clean.
Records use athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod), are validated before writing, and are written through the athena-git RecordsWriter, not a new writer.
docs/design/records-history.md states how sync_from changed (or the distinct recover path) and has a failure-path table (starting state, function, cited behavior); new functions carry doc comments citing the design section; CHANGELOG.md has an entry.

## Report

End with the producer report as your structured output.
