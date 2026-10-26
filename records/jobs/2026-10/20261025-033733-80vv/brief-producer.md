# Job 20261025-033733-80vv: Write a git-history file into the planner and critic snapshot

## Request (verbatim)

> Priority: high
> Give the planner and critic read-only evidence beyond a plain copy of the code (BRIEF.md addendum of 2026-10-25): a git snapshot with history (so they can run git log, git show and git diff, read-only) and the budget printout (cargo test --locked -p athena-cli --test budgets -- --nocapture), with no writes. Planning stops and post-plan rejections came from facts only history shows, such as the shape of real pull-request ranges (job 20261024-115923-02gf broke land on every pull request).
> Acceptance:
> - A test shows the planner and critic grants allow only those read-only git commands and the budgets command, and no writes.
> - A planning brief test shows they are told the commands exist.
> - A planning run on a fixture can run git log in its snapshot.
> Files: crates/athena-core/src/run.rs, crates/athena-git/src/snapshot.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

The planner and critic get a generated history file in their read-only snapshot. It holds the last 200 base commits (subject, date, changed paths), without records/ or private paths, and comes from plain git log at snapshot time. Both briefs name the file and say what it holds. No git repository goes in the snapshot, no history is rewritten, and the grants stay unwidened. The audience is the planner and critic agents, and the maintainer who judges whether they stop planning less often.

Assumptions:
- The history file lives at a fixed path in the snapshot root, such as HISTORY.txt; the producer picks the name and the briefs cite the same constant.
- Private paths are excluded with the same filter the snapshot already uses for its file copy.
- Commits whose changed paths are all excluded are omitted or listed without paths; the producer picks one and documents it.
- The 200 commits are counted on the base commit after exclusion, not before.
- The earlier proposals (git log/show/diff grants, budgets command grant) are dropped per the user's reply. The budgets printout is out of scope.

## Acceptance criteria

1. [behavior] A test in athena-git builds a planning snapshot of a fixture repository with several commits, including one touching records/ and one touching a private path. It checks that the history file exists with one entry per commit (subject, date, changed paths), is capped at 200 commits, and contains no records/ or private path. Run: cargo test -p athena-git snapshot.
2. [behavior] The snapshot contains no .git directory and the user's repository is unchanged by snapshotting. A test checks this. Run: cargo test -p athena-git snapshot.
3. [behavior] A briefs test shows planner_brief and critic_brief each name the history file's path and say what it holds (last 200 commits, subject, date, changed paths, records/ and private excluded) and that Read and Grep are used on it. Run: cargo test -p athena-context briefs.
4. [behavior] A test in athena-core asserts the planner and critic grants are unchanged: no Bash or git commands, writes false. Run: cargo test -p athena-core.
5. [content] The time the history file adds to snapshotting is measured, and the figure (labeled as a measurement, with the repository and method) is recorded in docs/decisions.md.
6. [content] CHANGELOG.md has an entry, and docs/decisions.md records the decision. It lists the alternatives (git repository in the snapshot, widened git grants, plain copy), the choice, and cites job 20261024-115923-02gf.
7. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-git/src/lib.rs (modify)

## Out of scope

- Giving the planner or critic Bash or git command grants
- Putting a .git directory or any repository in the snapshot
- Providing the budgets test output to the planner and critic
- Changing the producer or verifier snapshots

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the file, the briefs and the grants, plus a clean cargo test and clippy.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The snapshot time added is measured and recorded, citing job 20261024-115923-02gf.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the file's contents and the brief text, not just that the code exists.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The history file excludes records/ and private paths.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the listed files.
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

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A test in athena-git builds a planning snapshot of a fixture repository with several commits, including one touching records/ and one touching a private path. It checks that the history file exists with one entry per commit (subject, date, changed paths), is capped at 200 commits, and contains no records/ or private path. Run: cargo test -p athena-git snapshot.
The snapshot contains no .git directory and the user's repository is unchanged by snapshotting. A test checks this. Run: cargo test -p athena-git snapshot.
A briefs test shows planner_brief and critic_brief each name the history file's path and say what it holds (last 200 commits, subject, date, changed paths, records/ and private excluded) and that Read and Grep are used on it. Run: cargo test -p athena-context briefs.
A test in athena-core asserts the planner and critic grants are unchanged: no Bash or git commands, writes false. Run: cargo test -p athena-core.
The time the history file adds to snapshotting is measured, and the figure (labeled as a measurement, with the repository and method) is recorded in docs/decisions.md.
CHANGELOG.md has an entry, and docs/decisions.md records the decision. It lists the alternatives (git repository in the snapshot, widened git grants, plain copy), the choice, and cites job 20261024-115923-02gf.
cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Report

End with the producer report as your structured output.
