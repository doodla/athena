# Job 20261024-232055-9xxv: verify "Add `athena check --host-rules` for the read-only host-rules check"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8c4227e8c147 → candidate 2c8354a1c630. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   3 +
 crates/athena-cli/src/display.rs     |  20 ++++++
 crates/athena-cli/src/main.rs        | 118 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/host_rules.rs |  62 ++++++++++++++++++
 4 files changed, 203 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261024-232055-9xxv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261024-232055-9xxv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42458286769 for 2c8354a1c63019bef6d7f6d26b6f6d6ce37137d3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42458286769 for 2c8354a1c63019bef6d7f6d26b6f6d6ce37137d3)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42458286769 for 2c8354a1c63019bef6d7f6d26b6f6d6ce37137d3)
- ci-result: `not a check` passed (Check results for 2c8354a1c63019bef6d7f6d26b6f6d6ce37137d3 come from the CI run https://github.com/doodla/athena/actions/runs/42458286769 (673s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass (tested default).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
