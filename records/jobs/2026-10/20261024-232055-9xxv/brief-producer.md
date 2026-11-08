# Job 20261024-232055-9xxv: Add `athena check --host-rules` for the read-only host-rules check

## Request (verbatim)

> Add a command for the read-only host-rules check (crates/athena-core host_rules, added in 837b812): it reports for athena/records whether force pushes and deletion are forbidden and for main whether branches must be up to date before merging, and exits non-zero when a rule is not in force.
> Acceptance:
> - The command (for example athena check --host-rules) is listed in --help.
> - A test with fixture replies shows the exit code and text for rules in force and not in force.
> - It makes only GET requests.
> Files: crates/athena-cli/src/main.rs, crates/athena-cli/src/display.rs, crates/athena-core/src/host_rules.rs, CHANGELOG.md

## Reading (accepted by the user)

Wire the existing read-only host_rules check (athena-core) into a new `--host-rules` flag on `athena check`, printing per-branch rule states and exiting 2 when any required rule is not in force. Audience: the repo owner running it to confirm branch protections.

Assumptions:
- Code survey: crates/athena-cli/src/main.rs (Command::Check variant near line 463, dispatch near line 1520, exit-code 2 convention for findings), crates/athena-core/src/host_rules.rs (check(runner, owner, repo), evaluate(reply), render(results), RuleState, BranchRules, existing test only_get_calls_to_the_rules_view_are_made), crates/athena-core/src/lib.rs (pub mod host_rules). The property is measured directly by running the CLI handler against fixture replies via a fake Runner, not through a stand-in.
- Environment: the check calls `gh api repos/{owner}/{repo}/rules/branches/{branch}` (GET only, read permission on the repository host) for athena/records and main; tests use fixture replies and make no network calls.
- The flag is added to the existing `Check` subcommand as `--host-rules`, conflicting with the other check modes; no new record or data file is added, so no records validation allow-list change is needed.
- Owner/repo come from the existing origin-remote parsing helper if one exists in athena-cli/athena-core; otherwise parsed from `git remote get-url origin`.
- Exit codes follow the CLI convention: 0 when all rules in force, 2 when any is not in force.
- The change adds no commit-classification or record-validation rule, so no realistic-branch fixture is needed; the existing check modes are untouched.
- CHANGELOG.md gets one line under the unreleased section. No governance files change.

## Acceptance criteria

1. [content] `athena check --help` / `athena --help` source in crates/athena-cli/src/main.rs documents `--host-rules` in the Check doc comment and as a clap arg, so it appears in --help output.
2. [behavior] Run `cargo test -p athena-cli host_rules`: a test with fixture replies for rules in force asserts exit code 0 and the exact rendered text (athena/records: force pushes and deletion forbidden; main: up-to-date required before merge).
3. [behavior] Run `cargo test -p athena-cli host_rules`: a test with fixture replies where a rule is not in force asserts exit code 2 and the rendered text names the branch and the missing rule.
4. [behavior] Run `cargo test -p athena-core host_rules`: the fake runner records every call and the test asserts each is a GET `gh api` read (no -X, -f, --method other than GET) when invoked through the new command path.
5. [behavior] Run `cargo run -p athena-cli -- --help` (or `check --help`): output lists `--host-rules`.
6. [content] CHANGELOG.md has an entry describing `athena check --host-rules`; display.rs holds any new rendering helper used by main.rs.
7. [behavior] Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass (tested default).

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/host_rules.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any write or PATCH/POST call to the host, or changing the rules themselves
- Changes to the other check modes or their behavior
- Changes to athena.toml, defaults.toml, or CI workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-reply tests, cargo test and clippy criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check exit code and text from running tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One flag on the existing Check command reusing host_rules.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs --help and the tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Exit code reflects whether rules are in force.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 9 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs when a reading is accepted. For any request that names a size or line-count target, it measures the named files in the current snapshot and records the measured values in the reading. It refuses or flags the reading if the stated figure differs from the measurement. Requir...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261024-232055-9xxv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261024-232055-9xxv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

`athena check --help` / `athena --help` source in crates/athena-cli/src/main.rs documents `--host-rules` in the Check doc comment and as a clap arg, so it appears in --help output.
Run `cargo test -p athena-cli host_rules`: a test with fixture replies for rules in force asserts exit code 0 and the exact rendered text (athena/records: force pushes and deletion forbidden; main: up-to-date required before merge).
Run `cargo test -p athena-cli host_rules`: a test with fixture replies where a rule is not in force asserts exit code 2 and the rendered text names the branch and the missing rule.
Run `cargo test -p athena-core host_rules`: the fake runner records every call and the test asserts each is a GET `gh api` read (no -X, -f, --method other than GET) when invoked through the new command path.
Run `cargo run -p athena-cli -- --help` (or `check --help`): output lists `--host-rules`.
CHANGELOG.md has an entry describing `athena check --host-rules`; display.rs holds any new rendering helper used by main.rs.
Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass (tested default).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
