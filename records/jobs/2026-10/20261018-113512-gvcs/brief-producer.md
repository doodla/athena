# Job 20261018-113512-gvcs: Dispatch jobs from a stable ref so merges don't disturb running jobs

## Request (verbatim)

> try again on 20261018-095726-r3bn

## Reading (accepted by the user)

Retry of job 20261018-095726-r3bn for the maintainer running Athena: stop rewriting the local branch after a squash merge (the resync_onto_merge call in Athena::land_pr in gh.rs), so the next job can start while the earlier PR is in CI, and one merge still lands one job as one commit. gh.rs is now a deliverable, and the test seam is a crate-private inner function of Athena::land_pr that takes a Remote and host.

Assumptions:
- The typed request is a retry of the original dispatch-stability job; the proposed planning-stage change (seam-naming critic, hand-on at the round cap) is a separate job and is not done here.
- Goal: the next job starts while the previous PR is in CI. Audience: the maintainer running Athena.
- The fault is the resync_onto_merge call after a squash merge in Athena::land_pr (gh.rs, about line 1630). Removing it keeps running jobs' bases as ancestors of HEAD, so land.rs plan still lands them.
- A squash conflict against a moved remote tip is handled by cutting the PR head onto the remote tip with Git::replay, which skips commits that become empty. A PrLink source field stops reruns from pushing again, and the fingerprint check before any push stays.
- Test seam: Athena::land_pr becomes a thin wrapper over a crate-private inner function taking &dyn Remote and a host. Tests call the inner function with a FakeRemote and a local bare repo. The inner function keeps the former resync call site, so a test fails if the call is restored.
- Records commits may sit between landed commits on the local branch. Tests assert no merge commits, A's changes not applied twice, a matching tree fingerprint, and one commit per squash merge, not that B's head holds only B's own commits.
- GitRemote::cut_head uses self.git.signs_commits() directly.
- Whether run.rs, pr.rs and watcher.rs change depends on where the dispatch base is read. They are listed with lower probability.

## Acceptance criteria

1. [behavior] A new test calls the inner land_pr function with a FakeRemote and host. It squash-merges job A's PR while job B is already dispatched, then checks that B's base commit and worktree are unchanged and B's candidate lands cleanly. Run cargo test.
2. [behavior] A test dispatches two jobs back to back before the first merges. After each squash merge, main gains exactly one commit per job, with no merge commits and no rebuild commits. A's changes are not applied twice, and the tree fingerprint matches. Run cargo test.
3. [behavior] The test of the second behavior fails if the resync_onto_merge call is restored in the land path. The verifier can check this by reading the test and running cargo test.
4. [behavior] cargo test passes, cargo clippy is clean, and cargo fmt --check passes.
5. [content] No production path calls resync_onto_merge after a merge, and dispatch no longer depends on it having run. Nothing under crates/ refers to it except possibly a removed or renamed definition.
6. [content] docs/design/parallel-jobs.md, docs/design/native-pr-landing.md and docs/decisions.md state the new base rule, the alternatives rejected, and how a job stale against a moved base is handled by cutting onto the remote tip. A maintainer can read that the next job starts while the earlier PR is in CI and that one merge lands one job.
7. [content] CHANGELOG.md has an entry for the change.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- docs/design/parallel-jobs.md (modify)
- docs/design/native-pr-landing.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-git/src/lib.rs (modify)

## Out of scope

- Changing the planning stage, the critic, or the three-round cap (the earlier introspection's proposal is a separate job).
- Changes to CI workflows or governance files.
- Redesigning the Pages or checks handling in landing.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check are required criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check merge behavior through tests, not that wiring exists.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design docs and decisions.md record the rule and rejected alternatives.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the post-merge path; the planner-stage change is excluded.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Removes the resync call rather than working around it at dispatch.
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
