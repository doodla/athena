# Job 20261103-202651-j4w5: Add one xtask command to repin Claude Code and point refusals at it

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

## Findings you must address

The verifier of job 20261016-005805-p806 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 5 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`
> blocker (docs/decisions.md:4222 (D154); docs/decision-links.toml): The new D154 entry breaks the workspace test suite: its 'amends D03' link is not in the confirmed decision-links list. `cargo test -p athena-cli --test backtest the_real_log_has_no_condition_a_finding_against_the_confirmed_list` fails with 'hit not in the confirmed list: D03 amended by D154 (line 4222, in D154)'. CI run 41710043053 failed on the same test. Fix: add a [[link]] line=4222 older="D03" newer="D154" status="amended" to docs/decision-links.toml (outside the deliverables list), or reword the entry. The header of docs/decision-links.toml says each listed hit is for the user to confirm.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 42 cases in 42 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer may finish, Athena runs the cheap mechanical gate on the working copy: the decision-link consistency test (every "amends/supersedes Dxx" link in docs/decisions.md is in docs/decision-links.toml), fmt, clippy, and a scan that each behavior criterion has evidence citing a command t...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

26 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

A stand-in claude reporting a version different from the pin is still refused by check_pinned, and the error names both the found and pinned versions and tells the reader to run the new xtask repin command. Run `cargo test -p athena-agent`.
The new xtask command, run against a temporary athena.toml and a stand-in claude reporting 2.1.305, rewrites claude_version to "2.1.305" and leaves every other line byte-identical; a second run changes nothing; an empty or unparseable version leaves the file untouched and errors. Run `cargo test -p xtask`.
The existing exact-match behaviour is unchanged: a binary whose version equals the pin is accepted and the reported version recorded. Run `cargo test -p athena-agent`.
`cargo test --locked --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check` pass, including the decision-links backtest test.
docs/decisions.md has an entry recording the choice (exact pin kept, one repin command, alternatives: accept newer patches, auto-repin; cost: a manual command after each update). Any link to an earlier decision in it is registered in docs/decision-links.toml, and CHANGELOG.md and README.md describe the command.
athena.toml, defaults.toml, setup.sh and .github/workflows are unchanged (Grep/git diff shows no edits), and no secrets or personal data are committed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
