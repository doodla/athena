# Job 20261025-033731-5hac: Add docs/system-facts.md to planner and critic briefs with drift test

## Request (verbatim)

> Priority: high
> Add a system-facts file that the planner and critic always read (BRIEF.md addendum of 2026-10-25): docs/system-facts.md, short, stating the facts about how Athena runs that reading the code cannot reveal, each with its source. Start with: the host squash-merges and main requires linear history and up-to-date branches; a producer works in a one-commit snapshot with no remote and no other branches; the reader, planner and critic get a plain copy without git or records/; prompt sets and their budgets with current sizes; the words the boundary scan bans; the request format athena plan parses (acceptance as bullets, dependencies as job ids, Priority: high); land lands only a job's pull request; records live on athena/records after C0. Add it to the planner and critic briefs, and add a test that fails when a prompt budget in the file differs from the budgets test's numbers.
> Acceptance:
> - The planner and critic briefs include the file, and a brief test shows it.
> - A test fails when the file's recorded budget sizes drift from the computed ones.
> - The planner and critic budgets still pass.
> Files: docs/system-facts.md, crates/athena-context/src/briefs.rs, crates/athena-cli/tests/budgets.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Create a short sourced docs/system-facts.md of run-time facts the code can't reveal (including the user's added facts), embed it in planner_brief and critic_brief, and add a budgets test failing when recorded budgets/sizes drift from computed ones. Audience: planner and critic agents, and maintainers editing the file.

Assumptions:
- The file is embedded via include_str! in briefs.rs so both briefs carry it.
- Budget numbers are recorded in a parseable form (e.g. 'reader: 1997 of 2000') so the test can extract them.
- The facts file may need to fit within the planner/critic budgets; if it does not, budgets in budgets.rs are not raised without need and the file is kept short.
- Boundary words are not listed; the file points to boundary_words() in xtask/tests/policy.rs as the source.
- No athena.toml or workflow change is needed; the [personal_data] allow-list is only referenced.

## Acceptance criteria

1. [content] docs/system-facts.md is short and states every requested and added fact (squash-merge/linear history/up-to-date branches; producer one-commit snapshot; plain copy without git or records/ for reader, planner, critic; prompt sets with budgets and current sizes; plan request format; land lands only a job's PR; records on athena/records after C0; SSH remote refused at admission; per-PR and per-branch caches; early PR and land waiting for CI), each with a source.
2. [content] The file does not list the banned boundary words and names boundary_words() in xtask/tests/policy.rs as the source; no email-like text appears in it.
3. [behavior] A briefs.rs unit test asserts planner_brief and critic_brief output contains the facts file text; run `cargo test -p athena-context`.
4. [behavior] A budgets.rs test fails when the budgets or current sizes recorded in the file differ from the computed ones, and a negative case with a doctored value is rejected; run `cargo test -p athena-cli --test budgets`.
5. [behavior] The existing planner and critic budget tests still pass with the file included; run `cargo test -p athena-cli --test budgets`.
6. [content] CHANGELOG.md has an entry and docs/decisions.md records the call (file, brief inclusion, drift test).
7. [behavior] cargo test, cargo clippy and cargo fmt --check pass at the repository root.

## Expected deliverables

- docs/system-facts.md (create)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing prompt budgets
- Listing the banned words in the file
- Changes to athena.toml or workflows
- Editing BRIEF.md

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit test in briefs.rs and drift test in budgets.rs; cargo test and clippy clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every fact in the file cites a source.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check brief output contains the file and that a doctored budget is rejected.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Plain wording for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No emails or secrets; SSH remote example uses a placeholder.
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

docs/system-facts.md is short and states every requested and added fact (squash-merge/linear history/up-to-date branches; producer one-commit snapshot; plain copy without git or records/ for reader, planner, critic; prompt sets with budgets and current sizes; plan request format; land lands only a job's PR; records on athena/records after C0; SSH remote refused at admission; per-PR and per-branch caches; early PR and land waiting for CI), each with a source.
The file does not list the banned boundary words and names boundary_words() in xtask/tests/policy.rs as the source; no email-like text appears in it.
A briefs.rs unit test asserts planner_brief and critic_brief output contains the facts file text; run `cargo test -p athena-context`.
A budgets.rs test fails when the budgets or current sizes recorded in the file differ from the computed ones, and a negative case with a doctored value is rejected; run `cargo test -p athena-cli --test budgets`.
The existing planner and critic budget tests still pass with the file included; run `cargo test -p athena-cli --test budgets`.
CHANGELOG.md has an entry and docs/decisions.md records the call (file, brief inclusion, drift test).
cargo test, cargo clippy and cargo fmt --check pass at the repository root.

## Report

End with the producer report as your structured output.
