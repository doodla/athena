# Job 20261016-005805-p806: Accept newer Claude Code patch versions instead of stopping

## Request (verbatim)

> Claude Code updates itself on this machine (2.1.296 to 2.1.299 to 2.1.301 in two days), and each update stops every agent against the pin until a person repins by hand, because repinning is itself a job that cannot start. Athena should not stop because its harness updated.

## Reading (accepted by the user)

Athena stops when the self-updating claude moves past the exact pin. I read the goal as accepting an installed claude with the same major.minor and a patch at or above the pin, recording the version actually reported. The audience is whoever runs Athena and currently repins by hand.

Assumptions:
- Code survey: read crates/athena-agent/src/claude.rs. check_pinned (line 61) runs inside ClaudeCodeDriver::new, after refuse_nested, and compares claude_version(bin) with == to the pin. The result is stored as the driver version. Tests at about lines 612 and 855 use a stand-in binary. crates/athena-data/src/config.rs holds claude_version; crates/athena-cli/src/wiring.rs builds a DeferredDriver from it. xtask/src/main.rs ensure_claude (line 621) also requires an exact match before installing the pin privately with npm. No record or data file is added, so check_record_file and AnyRecord are not affected. The version is measured by running the stand-in binary's --version, the same path the driver uses.
- The request is read as goal: Athena keeps running across Claude Code patch updates. Audience: whoever runs Athena. The pin in athena.toml stays and acts as the tested minimum.
- The earlier readings are not replies from the user, so no user reply overrides them. I chose Reading 1 because repinning cannot run as a job, so a repin command does not remove the stop.
- athena.toml is not edited. Only the check, the setup lookup that has to agree with it, and their docs change.
- This loosens a refusal rule, so it refuses fewer readings, not more. Expected 0% of newly refused readings.

## Acceptance criteria

1. [behavior] A stand-in claude reporting a newer patch than the pin (pin 2.1.299, binary 2.1.301) is accepted by check_pinned, and the returned version is 2.1.301. Run `cargo test -p athena-agent`.
2. [behavior] Tests show a binary with an older patch than the pin, a different major or minor, or an unparseable or empty version is still refused, with an error naming both the found and pinned versions. Run `cargo test -p athena-agent`.
3. [behavior] ClaudeCodeDriver::new built with a pin and a binary of a different accepted patch records the binary's reported version, not the pin, and a test asserts this. Run `cargo test -p athena-agent`.
4. [behavior] xtask ensure_claude uses the same compatibility rule, so a newer-patch claude on PATH is used rather than triggering an npm install. A test covers the comparison. Run `cargo test -p xtask`.
5. [behavior] `cargo test --workspace` and `cargo clippy --workspace --all-targets` pass, and `cargo fmt --check` is clean.
6. [content] docs/decisions.md gains an entry amending D03 that states the rule, the alternatives (any 2.x, auto-repin, keep exact), what is lost (an identical version on every run), and the reversal cost. CHANGELOG.md has an entry.
7. [content] The refusal message and README or setup docs tell the reader that a newer patch of the pinned major.minor needs no repin. athena.toml, defaults.toml and .github/workflows are unchanged, and no secrets or personal data are committed.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- xtask/src/main.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-cli/tests/live.rs (modify)

## Out of scope

- An xtask repin command that rewrites athena.toml
- Accepting a different major or minor version
- Editing athena.toml, defaults.toml or workflows
- Changing how Claude Code is invoked or its flags

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in the agent and xtask crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests with stand-in binaries; cargo test and clippy must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that a newer patch is accepted and others are still refused.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run the stand-in tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision entry and error text are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the version check and its docs.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data are committed.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The compatibility rule is enforced in check_pinned and tested.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
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

A stand-in claude reporting a newer patch than the pin (pin 2.1.299, binary 2.1.301) is accepted by check_pinned, and the returned version is 2.1.301. Run `cargo test -p athena-agent`.
Tests show a binary with an older patch than the pin, a different major or minor, or an unparseable or empty version is still refused, with an error naming both the found and pinned versions. Run `cargo test -p athena-agent`.
ClaudeCodeDriver::new built with a pin and a binary of a different accepted patch records the binary's reported version, not the pin, and a test asserts this. Run `cargo test -p athena-agent`.
xtask ensure_claude uses the same compatibility rule, so a newer-patch claude on PATH is used rather than triggering an npm install. A test covers the comparison. Run `cargo test -p xtask`.
`cargo test --workspace` and `cargo clippy --workspace --all-targets` pass, and `cargo fmt --check` is clean.
docs/decisions.md gains an entry amending D03 that states the rule, the alternatives (any 2.x, auto-repin, keep exact), what is lost (an identical version on every run), and the reversal cost. CHANGELOG.md has an entry.
The refusal message and README or setup docs tell the reader that a newer patch of the pinned major.minor needs no repin. athena.toml, defaults.toml and .github/workflows are unchanged, and no secrets or personal data are committed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
