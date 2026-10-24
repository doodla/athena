# Job 20261024-055549-0qcy: Add records-branch writer to athena-git (retry of 20261024-033933-ta60)

## Request (verbatim)

> try again on 20261024-033933-ta60

## Reading (accepted by the user)

Retry the rejected job: add a writer in athena-git that appends opaque record files to the orphan branch athena/records using compare-and-swap, fast-forward only, bounded retry (§8.2, D111). The prior attempt was rejected only because the gate criterion had no cited commands; this attempt must run and cite the gate commands.

Assumptions:
- Goal: a tested, uncalled library writer; audience is the maintainer and the verifier, who run the tests.
- The retry takes the original records-branch request as typed; the introspection's proposed pipeline pre-handover step is a separate job and is out of scope.
- The earlier design and test list were accepted; only the evidence gap (criterion 5) and the weak contention nit need fixing.
- The existing Git wrapper, temp index and commit_tree helpers in athena-git suffice; no new crate, so Cargo.toml likely stays unchanged.
- Writing identical bytes at an existing path is refused (or documented as a no-op); the producer picks one and documents it.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-git`. A test starts two writers on threads against one repository, with different paths. Both land, both paths are on the final tip, each update is a fast-forward of the previous tip, and no commit is lost.
2. [behavior] Run `cargo test -p athena-git`. A deterministic test makes the expected tip stale. The first update is rejected, the writer refetches and replays its files on the new tip, and the write succeeds with both the earlier and new files in the final tree. The two-thread test needs no forced contention; this test covers the reject path.
3. [behavior] Run `cargo test -p athena-git`. Tests show that modifying, deleting or renaming an existing path is refused with an error and the tip is unchanged. A same-bytes write at an existing path is tested and its result documented.
4. [behavior] Run `cargo test -p athena-git`. A test shows `git merge-base main athena/records` finds nothing and the first records commit has no parent. A second test shows that when every attempt is rejected, the writer returns a clear error after the bounded count.
5. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` from the repository root. All pass and are clean. The producer's final report quotes each command line and its result so the verifier can cite them (D43).
6. [content] A new public module in athena-git, exported from lib.rs, takes opaque bytes keyed by path. Its doc comments cite §8.2 and D111. It uses the existing Git wrapper and helpers with no new crate. No code outside athena-git calls it.
7. [content] The CHANGELOG [Unreleased] section has an entry, and docs/design/records-history.md notes the writer as built. No governance file (athena.toml, defaults.toml, workflows, lint tables) and nothing under records/ is changed.

## Expected deliverables

- crates/athena-git/src/records_branch.rs (create)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-git/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Any pre-handover evidence step or other change to the job pipeline
- Calling the writer from the pipeline
- Pushing or publishing athena/records to a remote
- Changes to governance files or records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The writer and its tests are Rust in athena-git.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Every new module has passing tests; cargo test and clippy are clean.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing helpers; add no crate.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check writers landing, replay, refusal and orphan history by running tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module plus export, changelog and design note.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite §8.2 and D111 for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The tests check the outcome: the branch state after each call.
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
