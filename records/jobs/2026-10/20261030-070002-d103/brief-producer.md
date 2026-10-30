# Job 20261030-070002-d103: Give each job a read-only evidence snapshot and athena status --snapshot

## Request (verbatim)

> Priority: urgent
> The producer and verifier cannot check a change against this repository's real records or main's history: a job's working copy has no records/ directory and a single commit, by design. Two jobs failed on this: 20261029-233021-e5sg (its criterion to run athena status on the real records could not be met; producer report: 'records/ is absent from my working copy') and 20261029-210827-18fc (A1's build measurement was impossible from the producer's copy). The reader still accepts such criteria.
> Change: give each job a read-only evidence snapshot outside the working copy: a copy of records/ and a git log of main's first-parent history (hash, commit time, subject, trailers) taken at dispatch. Name its path in the producer and verifier briefs, and add an option to athena status that reads records and history from such a snapshot. The working copy stays without records/.
> Acceptance:
> - A test through the real dispatch path shows the snapshot exists, read-only, at the path the briefs name, holding the records and main history at dispatch.
> - A test shows athena status run against the snapshot gives the same throughput section as on the main checkout for the same records.
> - The producer and verifier briefs state the snapshot path and that real-records checks use it.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/status.rs, crates/athena-context/src/briefs.rs, crates/athena-cli/src/main.rs, CHANGELOG.md

## Reading (accepted by the user)

At dispatch, Athena writes a read-only snapshot outside the working copy holding records/ and main's first-parent log (hash, commit time, subject, trailers). The producer and verifier briefs name its path, and `athena status` gains an option to read records and history from it. The audience is producers and verifiers who currently cannot check real-records criteria.

Assumptions:
- Code survey: I only grepped crates/athena-core/src/run/mod.rs (dispatch at ~line 684), crates/athena-context/src/briefs.rs (producer_brief ~415, verifier_brief ~983) and crates/athena-cli/src/main.rs (Command::Status ~314, handler ~1211); the producer must read them and status.rs fully. The snapshot is written during dispatch before the producer starts, then the brief inputs carry its path. Status loads records and history from the main checkout today and must load them from the snapshot dir when the option is given. The snapshot holds a copy of records/ in the same layout plus a history file; it is plain files, not a record kind under crates/athena-data, so no new AnyRecord variant or check_record_file entry is expected. If the producer adds one, it must be covered by the round-trip test. Throughput is measured directly by comparing the rendered throughput section from both sources on the same records.
- Prior jobs: 20261029-233021-e5sg (rejected) and 20261029-210827-18fc (landed) are the motivating evidence only. This job does not redo the landing-stage fix or the build-measurement work; their landed or rejected files are neither kept nor changed here. The e5sg landing-matching findings and the 18fc sccache-isolation finding are left to their own follow-ups, since this job only supplies the evidence they needed.
- The snapshot lives under the job's directory in Athena's state area (outside the working copy), is created at dispatch, and is made read-only by file permissions (dirs 0555, files 0444). The working copy still has no records/.
- The option name is `--snapshot <path>` on `athena status`, and it takes the snapshot root. History is stored as a text or JSON file in a documented format that status parses.
- Governance files: athena.toml, defaults.toml and workflow files do not need to change. Briefs are composed in code, so prompt-token budgets are not touched.
- The history log is first-parent on main at dispatch, capped to a reasonable number of commits. Records are copied as of dispatch.

## Acceptance criteria

1. [behavior] A test through the real dispatch path (run `cargo test -p athena-core`) shows the snapshot directory exists at the path the briefs name, is read-only (write attempts fail or permissions are non-writable), contains the records copied at dispatch, and contains main's first-parent history (hash, commit time, subject, trailers) at dispatch.
2. [behavior] A test (run `cargo test -p athena-core status`) shows `athena status` run against the snapshot gives a throughput section identical to the one from the main checkout for the same records and history.
3. [behavior] A CLI test (run `cargo test -p athena-cli`) shows `athena status --snapshot <dir>` reads records and history from the snapshot and does not need a records/ directory in the current working directory.
4. [behavior] Tests in crates/athena-context/src/briefs.rs (run `cargo test -p athena-context`) show that both the producer and verifier briefs state the snapshot path and say that real-records checks use it. The working copy remains without records/.
5. [content] The change is limited to crates/athena-core/src/run/mod.rs, crates/athena-core/src/status.rs, crates/athena-context/src/briefs.rs, crates/athena-cli/src/main.rs and CHANGELOG.md, plus any test files or helper modules they require (checked with git diff --stat).
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has a new entry describing the evidence snapshot and the `athena status` snapshot option.
8. [content] No secrets or personal data are written into the snapshot code, tests or docs, and the snapshot is not added to git history.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/snapshot.rs (create)

## Out of scope

- Redoing the landing-stage matching fix from job 20261029-233021-e5sg (its store.rs and athena-git changes are dropped here)
- Isolating sccache statistics per measurement (finding from job 20261029-210827-18fc); its landed files are kept as landed
- Putting records/ into the producer's working copy
- Changing athena.toml, defaults.toml, workflow files or records/
- Changing how the throughput section is computed

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for dispatch snapshot, status parity, and briefs; fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run dispatch and status and check the snapshot's contents and output, not that files exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is that a job can check real-records criteria from the snapshot.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Read-only is enforced by file permissions and tested.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Criterion forbids secrets or personal data in the snapshot code and tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the listed files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Tests exercise the dispatch path and the status option.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Briefs and CHANGELOG written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

22 more qualifying classes are left out.

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

A test through the real dispatch path (run `cargo test -p athena-core`) shows the snapshot directory exists at the path the briefs name, is read-only (write attempts fail or permissions are non-writable), contains the records copied at dispatch, and contains main's first-parent history (hash, commit time, subject, trailers) at dispatch.
A test (run `cargo test -p athena-core status`) shows `athena status` run against the snapshot gives a throughput section identical to the one from the main checkout for the same records and history.
A CLI test (run `cargo test -p athena-cli`) shows `athena status --snapshot <dir>` reads records and history from the snapshot and does not need a records/ directory in the current working directory.
Tests in crates/athena-context/src/briefs.rs (run `cargo test -p athena-context`) show that both the producer and verifier briefs state the snapshot path and say that real-records checks use it. The working copy remains without records/.
The change is limited to crates/athena-core/src/run/mod.rs, crates/athena-core/src/status.rs, crates/athena-context/src/briefs.rs, crates/athena-cli/src/main.rs and CHANGELOG.md, plus any test files or helper modules they require (checked with git diff --stat).
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has a new entry describing the evidence snapshot and the `athena status` snapshot option.
No secrets or personal data are written into the snapshot code, tests or docs, and the snapshot is not added to git history.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
