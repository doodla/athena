# Job 20261024-033933-ta60: Add athena/records branch writer with CAS and bounded retry

## Request (verbatim)

> Build the records-branch writer of the records-history plan (docs/design/records-history.md section 8.2, decision D111, job 1 part 2): in athena-git, a writer that commits opaque record files to the branch athena/records (its own root, never merged into main) with compare-and-swap and bounded retry. Tests: two simultaneous writers both land, each as its own path, fast-forward only; a rejected update refetches, replays and succeeds; a write that would modify, delete or rename an existing path is refused and the tip is unchanged; the branch has no parent in common with main. Nothing in the job pipeline calls it yet.

## Reading (accepted by the user)

Add a new module in athena-git that appends opaque record files to the orphan branch athena/records. It uses a compare-and-swap ref update, fast-forward only, and retries a bounded number of times by refetching and replaying. It refuses any write that would modify, delete or rename an existing path. Nothing in the job pipeline calls it. The audience is the maintainer and the verifier, who check it by running the tests.

Assumptions:
- The writer works on a local repository's ref, and optionally on a remote through the existing guarded push (push_if_remote_at). The local ref update uses `git update-ref <ref> <new> <old>`, which takes git's lock and acts as the compare-and-swap. The 'simultaneous writers' test uses two threads on one repository, or two clones pushing to a bare remote.
- Whether to also implement the remote variant is the producer's call. Local plus a bare-remote test is the minimum that matches §8.2 'Local repositories use the same ref update'.
- Records are opaque bytes: the writer does not validate schemas, because the athena-data record types are job 1 part 1 and are out of this change.
- The retry count defaults to 5 (the design's example) and can be set as a parameter.
- The first write creates a root commit with no parents, using an empty base tree. The commit uses a fixed Athena identity, like Git::isolated, with signing configurable through the existing `sign` flag.
- The branch name is the constant `athena/records`.
- The goal is the writer plus its tests, and the maintainer will build the later jobs (importer, landing) on it.
- Existing-path refusal is checked against the latest fetched tip on each attempt, so a replay after rejection also refuses if the winner took the same path.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-git`. A test starts two writers at once (threads) against one repository, with different record paths. Both land, each path is present on the final tip, every update was a fast-forward (each tip descends from the previous one), and no commit was lost.
2. [behavior] Run `cargo test -p athena-git`. A test makes the ref move between the writer's read and its update (a stale expected tip). The first update is rejected, the writer refetches and replays its new files on the new tip, and the write succeeds. The earlier commit and the new files are both in the final tree.
3. [behavior] Run `cargo test -p athena-git`. Tests show that a write that modifies an existing path, deletes one, or renames one (a delete plus an add of the same content) is refused with an error. In each case the branch tip after the call equals the tip before. A write with identical bytes at an existing path is also tested, and its result is documented.
4. [behavior] Run `cargo test -p athena-git`. A test shows that `athena/records` shares no history with main: `git merge-base main athena/records` finds nothing, and the first records commit has no parent. A second test shows the retry bound is enforced. When every attempt is rejected, the writer returns a clear error after the bounded count and does not loop.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean. `cargo test` passes for the workspace.
6. [content] A writer API exists in athena-git (a new public module exported from lib.rs). It takes opaque bytes keyed by path, with doc comments that cite §8.2 and D111. It uses the existing Git wrapper, temp index and commit_tree helpers, with no new crate. A repository search finds no caller of it outside athena-git, so the pipeline is untouched.
7. [content] No governance file is changed (athena.toml, defaults.toml, workflows, lint tables). No file under records/ is touched.

## Expected deliverables

- crates/athena-git/src/records_branch.rs (create)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-git/Cargo.toml (modify)

## Out of scope

- New record types in athena-data
- The importer, landing intent and reconcile, and the pre-merge status (jobs 2 to 4)
- Wiring the writer into the job pipeline
- Confirming host branch rules (no force-push, no deletion) on the real host
- The history check that rejects commits that modify records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only, inside the athena-git crate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for each requested behavior; cargo test and clippy must pass.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing Git wrapper, TempIndex and commit_tree helpers and the git CLI's atomic update-ref; add no crate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module plus its export; no pipeline changes.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the concurrency, retry, refusal and no-common-ancestor behaviors instead of checking that the code exists.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite the design section and decision.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Tests use synthetic identities and no personal data.
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
