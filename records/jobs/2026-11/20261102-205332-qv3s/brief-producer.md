# Job 20261102-205332-qv3s: Hand verifier failing tests and log tail from a red reused CI run

## Request (verbatim)

> When the checks stage reuses a red GitHub CI run, hand the verifier the names of the failing tests and the end of the failing job's log, so it does not rerun the whole suite to find them.
> Today the build-inputs job's GitHub run failed one test; the verifier spent its whole 2400s budget running every test locally to find which one, and ended with no verdict.
> - When a reused CI run is red, Athena reads the failing shard job's log through the existing gh client and records, on the failed check, the failing test names (cargo's 'failures:' list) and the last lines of output, as a passing local check already records its output tail.
> - The verifier brief shows those names and that tail, and tells the verifier to run only the failing tests (cargo test -p <crate> <name>) unless it has a reason to run more.
> - A test with a fake gh runner returning a red run and a shard log containing a 'failures:' list shows the failed check record carries the names and the verifier input shows them.
> - Changes stay in crates/athena-core (ci_result, gh, run/checks, verification brief) and crates/athena-context briefs; CHANGELOG.md gets an entry.
> - Before planning, list what else reads the check record and the verifier brief, with a test or a reason for each.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

When the checks stage reuses a red GitHub CI run, Athena reads the failing shard job's log via the gh client and records the cargo 'failures:' test names and the log tail on the failed CheckRun (failed_tests, output_tail). The verifier brief then shows them and tells the verifier to run only those tests (cargo test -p <crate> <name>) unless it has reason to run more. The audience is the verifier, which today runs out of budget.

Assumptions:
- Code survey: read crates/athena-core/src/ci_result.rs (CiRecord/Step/CiRun carry only fixed fields, no log), crates/athena-core/src/routing.rs (run_routed_with_record: resolve() then runs_from_record for a Source::Ci record, so reuse happens before any local fallback; red CI gives Outcome::Failed), crates/athena-core/src/checks.rs (runs_from_record at ~306 builds CheckRun with empty failed_tests and output_tail; failed_tests/output_tail extraction for local runs at ~480-512), crates/athena-core/src/gh/client.rs (failing_test_name(log) at ~154, already used at ~559; CiBranchSource/ci_source ~750-800; parse_ci_run), crates/athena-core/src/run/checks.rs (SnapExec), crates/athena-core/src/run/mod.rs (CheckRun construction ~1537), crates/athena-core/src/verify.rs (failed_tests used for baseline comparison at ~138-160 and ~335), and crates/athena-context/src/briefs.rs (verifier brief, uses output_tail/check_runs).
- Existing: CheckRun in crates/athena-data/src/records.rs already has failed_tests: Vec<String> and output_tail: String, so no new record field or data file is expected; the records validation allow-list and AnyRecord round trip should be unaffected. If a field is added, the producer must cover it with the existing round-trip test.
- Setting failed_tests on a reused red check also feeds verify.rs baseline logic (failing tests that also failed on base), status.rs, reading.rs carried-check output, introspect.rs and run/send_back.rs; each needs a test or a stated reason it is unaffected. This list goes in the CHANGELOG or the producer's plan before planning.
- The shard log is fetched through the existing gh Runner/GhClient (a fake Runner in tests); a failure to fetch or parse the log degrades to today's behavior (failed record without names), never a crash or a pass.
- Log text is data: names and tail are redacted and clipped like local output tails before being recorded.

## Acceptance criteria

1. [behavior] A test in crates/athena-core using a fake gh Runner that returns a red run for the SHA and a failing shard job log containing a cargo 'failures:' list shows the failed CheckRun (via run_routed_with_record) has passed=false, failed_tests equal to the listed names, and a non-empty output_tail holding the last lines of the log. Run: cargo test -p athena-core.
2. [behavior] A test builds the verifier input/brief from that failed check record and asserts the brief text contains the failing test names and the log tail, and the instruction to run only the failing tests with cargo test -p <crate> <name> unless there is a reason to run more. Run: cargo test -p athena-context -p athena-core.
3. [behavior] A test shows that when the shard log cannot be read or has no 'failures:' list, the reused red check is still recorded as failed with empty failed_tests, and a green reused run is unchanged (no log fetch, no names).
4. [content] Changes are confined to crates/athena-core (ci_result, gh, run/checks, routing/checks, verification brief) and crates/athena-context briefs, plus CHANGELOG.md; git diff --stat shows no other paths.
5. [content] CHANGELOG.md has an entry describing that reused red CI runs now give the verifier failing test names and the log tail.
6. [content] The producer's change lists every other reader of the check record and the verifier brief (verify.rs baseline comparison, status.rs, reading.rs carried output, introspect.rs, run/send_back.rs, athena-context outputs.rs/lib.rs, athena-cli tests) each with a test that covers it or a one-line reason it is unaffected; Grep for each of those readers confirms the claimed handling.
7. [behavior] Existing tests for local check output tails and for failed_tests baseline comparison in verify.rs still pass, showing the new data does not change verdict logic for local runs. Run: cargo test -p athena-core verify.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fetching logs for green runs or for non-test failures such as fmt and clippy beyond recording their log tail where trivial
- Changing the verifier's time budget or the verdict rules
- Changing the CI workflow files or athena.toml
- Adding a new record type or data file

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the red-run record and the brief; existing tests stay green.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check names and tail appearing in the record and brief, not that wiring exists.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change inside the named crates.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Log text is redacted before recording.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The verifier gets the data at the source (the reused red run) instead of a longer budget.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier exercises the new tests with cargo test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Behavior is enforced by tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 40 cases in 40 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer stops, Athena itself runs fmt --check and clippy --all-targets -D warnings (and test) on the job's snapshot. If any fail, it sends the exact output back to the producer for one repair round within the same job. The verifier is only start...
- 20 cases in 18 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make the disk guard and build cleanup one ordered step. Before the free-space check, and again on any low-disk hold, Athena removes the regenerable builds of held, interrupted and finished jobs, then measures free space. Only the unfreeable shortfall holds a job. Held jobs resume automatically once ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

25 more qualifying classes are left out.

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

A test in crates/athena-core using a fake gh Runner that returns a red run for the SHA and a failing shard job log containing a cargo 'failures:' list shows the failed CheckRun (via run_routed_with_record) has passed=false, failed_tests equal to the listed names, and a non-empty output_tail holding the last lines of the log. Run: cargo test -p athena-core.
A test builds the verifier input/brief from that failed check record and asserts the brief text contains the failing test names and the log tail, and the instruction to run only the failing tests with cargo test -p <crate> <name> unless there is a reason to run more. Run: cargo test -p athena-context -p athena-core.
A test shows that when the shard log cannot be read or has no 'failures:' list, the reused red check is still recorded as failed with empty failed_tests, and a green reused run is unchanged (no log fetch, no names).
Changes are confined to crates/athena-core (ci_result, gh, run/checks, routing/checks, verification brief) and crates/athena-context briefs, plus CHANGELOG.md; git diff --stat shows no other paths.
CHANGELOG.md has an entry describing that reused red CI runs now give the verifier failing test names and the log tail.
The producer's change lists every other reader of the check record and the verifier brief (verify.rs baseline comparison, status.rs, reading.rs carried output, introspect.rs, run/send_back.rs, athena-context outputs.rs/lib.rs, athena-cli tests) each with a test that covers it or a one-line reason it is unaffected; Grep for each of those readers confirms the claimed handling.
Existing tests for local check output tails and for failed_tests baseline comparison in verify.rs still pass, showing the new data does not change verdict logic for local runs. Run: cargo test -p athena-core verify.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
