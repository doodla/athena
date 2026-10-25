# Job 20261017-182608-jjkf: Make athena audit read landed jobs' files or refuse before paying

## Request (verbatim)

> Make `athena audit` read a landed job's real files, and refuse before paying when it cannot. Today it reads the job's changed files from the candidate commit, but candidate commits are not in a fresh clone (for 20261017-133559-1b3d, 20261017-121143-sg0j and 20261017-091546-05yx, `git cat-file -t <candidate_commit>` fails). `Git::show_file` returns Ok(None) for a missing commit just as for a missing path (crates/athena-git/src/lib.rs, show_file), so crates/athena-core/src/auditor.rs writes an empty outcome/ folder and still pays for and records an audit. Found by an independent spot-check of today's landings.
> Plan: (1) In auditor.rs, choose the commit to read from: the candidate commit if it exists as a commit; else, for a landed job, the commit on its branch's base (main) whose message has the trailer `Athena-Job: <job id>` (for 20261017-133559-1b3d that is 8e35286); else none. (2) If the job changed files and no commit resolves, or none of its changed files can be read, stop before any model call with an error naming the job and the commits tried; write no audit record. (3) Leave show_file's behavior for a missing path as it is; add a small helper in athena-git that tells whether a commit exists, if there is none.
> Acceptance:
> - cargo test: in a test repository where the candidate commit is absent but a squash commit with the `Athena-Job:` trailer exists, the auditor's outcome/ folder holds the changed files' contents from that squash commit (the test asserts on file content), and one audit is recorded.
> - cargo test: with neither commit present, `athena audit <job>` fails with a message naming the job, the fake driver records zero invocations, and no audit record is written.
> - The existing auditor tests pass unchanged; cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
> - CHANGELOG.md says what was wrong (audits of jobs whose candidate commit is missing read nothing) and what changed.
> Out of scope: the auditor's prompt, what it judges, and how candidate commits are stored.

## Reading (accepted by the user)

The auditor picks the candidate commit if it exists, else the squash commit on main carrying `Athena-Job: <id>`, else none. If the job changed files and none can be read, it errors naming the job and the commits tried before any model call, and writes no record. Audience: maintainers running `athena audit` on landed jobs in fresh clones.

Assumptions:
- The squash commit is found with the existing Git::log_grep on HEAD (the base branch) for the trailer `Athena-Job: <id>`; a new `commit_exists` helper in athena-git checks `<oid>^{commit}` via rev_parse/cat-file.
- Resolution happens in auditor::run before the workspace is created or agent_spec/invoke is called, so a refusal costs nothing and commits no records.
- A job whose candidate has no changed files (after filtering records/ and non-plain paths) is still audited as today; only 'changed files exist but nothing readable' is refused.
- Deleted files read as Ok(None) from show_file; the refusal fires only if none of the changed files could be read, not if some were deleted while others read.
- New tests go in crates/athena-cli/tests/auditor.rs using its existing fixture and fake driver.

## Acceptance criteria

1. [behavior] Run `cargo test`: in a test repository where the candidate commit is absent but a squash commit with the `Athena-Job: <id>` trailer exists, the auditor's outcome/ folder holds the changed files' contents from the squash commit (asserted on file content) and exactly one audit is recorded.
2. [behavior] Run `cargo test`: with neither commit present, `athena audit <job>` fails with a message naming the job and the commits tried, the fake driver records zero invocations, and no audit or invocation record is written.
3. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: existing auditor tests pass unchanged and all are clean.
4. [content] athena-git gains a small helper telling whether a commit exists; Git::show_file's behavior for a missing path is unchanged.
5. [content] auditor.rs resolves the commit (candidate, then the trailer-matched commit on the base, then none) and refuses before any model call when the job changed files and none can be read.
6. [content] CHANGELOG.md [Unreleased] states that audits of jobs whose candidate commit is missing read nothing, and what changed; bullets have no blank lines between them.

## Expected deliverables

- crates/athena-core/src/auditor.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-cli/tests/auditor.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The auditor's prompt and what it judges
- How candidate commits are stored or fetched
- Changing show_file's missing-path behavior

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests plus a unit test for the commit-exists helper; clippy and fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert outcome/ file contents and zero fake driver invocations, not just wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix resolves the commit explicitly and fails loudly instead of treating a missing commit as a missing path.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only auditor.rs, one helper in athena-git, tests and the changelog change.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Error message and changelog entry are written for an outside reader.
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
