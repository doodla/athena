# Job 20261103-202651-j4w5: verify "Add one xtask command to repin Claude Code and point refusals at it"

## Request (verbatim)

> try again on 20261016-005805-p806

## Reading (accepted by the user)

Retry of job 20261016-005805-p806 under the user's corrections. The pin stays exact; a new `cargo xtask` command sets `claude_version` in athena.toml to the installed claude's reported version, and the version-mismatch refusal tells the person running Athena to run it. The earlier patch-tolerant rule is dropped.

Assumptions:
- Prior job 20261016-005805-p806 is addressed by dropping its patch-tolerant rule: its D03 amendment, the second copy of the compatibility table in xtask, and its edits to crates/athena-cli/src/wiring.rs and crates/athena-cli/tests/live.rs are all discarded (out_of_scope). Its findings (the unregistered 'amends D03' link, the duplicated rule, the unrun full test suite) no longer apply, except that any new decisions.md link must be registered in docs/decision-links.toml, and the producer must run `cargo test --locked` and cite it.
- Code survey: crates/athena-agent/src/claude.rs check_pinned (line 61) runs `claude --version` and requires an exact match with the pin, called from the driver constructor (line 103). xtask/src/main.rs ensure_claude (line 621) probes PATH then a private npm install for the pin, and installs with npm when neither matches. The pin is `agents.claude_version` in athena.toml (read via crates/athena-data/src/config.rs:357). No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly with a stand-in claude script that reports a chosen version.
- Audience: whoever runs Athena and currently edits the pin by hand after a Claude Code self-update. The refusal is worded for them, not for an operator.
- The command reads the version from the claude on PATH (or the one named in the machine config) using the same `--version` probe and rewrites only the claude_version line of athena.toml, preserving the rest of the file. This job does not itself change athena.toml's value.
- setup.sh does not mention claude (grep found no match), so it needs no change and is out of scope.

## Acceptance criteria

1. [behavior] A stand-in claude reporting a version different from the pin is still refused by check_pinned, and the error names both the found and pinned versions and tells the reader to run the new xtask repin command. Run `cargo test -p athena-agent`.
2. [behavior] The new xtask command, run against a temporary athena.toml and a stand-in claude reporting 2.1.305, rewrites claude_version to "2.1.305" and leaves every other line byte-identical; a second run changes nothing; an empty or unparseable version leaves the file untouched and errors. Run `cargo test -p xtask`.
3. [behavior] The existing exact-match behaviour is unchanged: a binary whose version equals the pin is accepted and the reported version recorded. Run `cargo test -p athena-agent`.
4. [behavior] `cargo test --locked --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check` pass, including the decision-links backtest test.
5. [content] docs/decisions.md has an entry recording the choice (exact pin kept, one repin command, alternatives: accept newer patches, auto-repin; cost: a manual command after each update). Any link to an earlier decision in it is registered in docs/decision-links.toml, and CHANGELOG.md and README.md describe the command.
6. [content] athena.toml, defaults.toml, setup.sh and .github/workflows are unchanged (Grep/git diff shows no edits), and no secrets or personal data are committed.

## Expected deliverables

- xtask/src/main.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Accepting any version other than the exact pin, including newer patches (the earlier patch-tolerant rule is dropped)
- Automatic repinning at run time
- Editing athena.toml, defaults.toml, setup.sh or .github/workflows
- crates/athena-cli/src/wiring.rs and crates/athena-cli/tests/live.rs (the earlier candidate's edits are dropped)
- Changing how Claude Code is invoked or its flags

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The command and tests are Rust in xtask and the agent crate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the repin command and the refusal message; workspace test, clippy and fmt required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands that exercise a stand-in claude.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command and one message change, with the earlier broader rule dropped.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry, README and CHANGELOG written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria run through the verifier's allowed cargo commands.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data committed.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2751d82eee9a → candidate 4e20bce5007a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   3 +
 README.md                         |  11 ++++
 crates/athena-agent/src/claude.rs |  21 ++++--
 docs/decisions.md                 |  22 +++++++
 xtask/src/main.rs                 | 133 +++++++++++++++++++++++++++++++++++++-
 5 files changed, 184 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41718612201 for 4e20bce5007ae51afe93f9cd09a140d03133481d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41718612201 for 4e20bce5007ae51afe93f9cd09a140d03133481d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41718612201 for 4e20bce5007ae51afe93f9cd09a140d03133481d)
- ci-result: `not a check` passed (Check results for 4e20bce5007ae51afe93f9cd09a140d03133481d come from the CI run https://github.com/doodla/athena/actions/runs/41718612201 (547s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test --locked --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check` pass, including the decision-links backtest test.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
