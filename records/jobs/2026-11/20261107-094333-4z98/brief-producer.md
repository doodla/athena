# Job 20261107-094333-4z98: Add honest delivery-verb outcome check with replay over real jobs

## Request (verbatim)

> try again on 20261027-054230-8nz8

## Reading (accepted by the user)

Retry of job 20261027-054230-8nz8: add `check_outcome_state` to reading acceptance so delivery-verb requests (publish, deploy, share, release) must name an observable state. Claim only what a replay test over the recorded cited jobs shows (1 of 3 real delivery-verb misreads), fix the verifier's findings, and correct CHANGELOG and docs to match. Audience: the Athena operator and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (refusing_check at ~line 3225 runs one `step!(check_...)` line per check, ending with check_governance_coverage, check_boundary_and_repo_files, check_reading_references) and crates/athena-core/tests/refusal_corpus.rs (case lists, run(), and a test that every step! has a refusing case). No record or data file under crates/ is added, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly by calling refusing_check on real request text.
- Job 20261027-054230-8nz8 is addressed finding by finding. (a) Major, 'changelog claims 3 of 3 but only 1 caught': the CHANGELOG and the check_outcome_state doc will say the check covers 1 of the 3 cited jobs (20261014-082328-g6vy, a real publish misread). 20261014-174815-7bf8 (ladder of requirements) and 20261017-173508-0wtc (ontology decision) belong to another class, figurative or structural terms and glossary, and are stated as not covered. (b) Major, 'reading does not match request': decisions.md and CHANGELOG state the narrowed scope and the count instead of claiming the whole class. (c) Minor, substring match in the out_of_scope exit: use whole-word has_phrase matching, including the -ing form, so 'shared helpers' or 'Release notes' do not satisfy it. (d) Minor, common words in OBSERVABLE_STATE such as tag and url: drop or tighten ambiguous words so a criterion must name a reachable or served state. (e) Minor, verbs only found at sentence start: also detect the verb after 'and', 'can you', 'I want to' and similar, with tests for 'Build the docs and publish them', 'Can you deploy the site' and 'I want to release v1'. (f) Nit, step placement after check_governance_coverage is kept and explained in decisions.md.
- The replay test uses the recorded request text of 20261014-082328-g6vy, written as a fixture string in the test, and asserts the check refuses it when the reading's criteria only say a file exists. It also asserts the check does not fire for the other two cited requests and states the count 1 of 3 in the test. The CHANGELOG count must equal it.
- Past accepted readings are replayed as accepting cases: ordinary requests that name a user-visible outcome, and requests with no delivery verb such as 'Add a timeout setting', stay accepted.
- The refusal-rate estimate of 5% is a guess, not a measurement. The check applies only to requests with a delivery verb whose reading names no observable state.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A unit test in crates/athena-core/src/reading.rs shows check_outcome_state refusing a reading for the real g6vy request 'Publish a first explainer page on how firstmate handles work.' whose criteria only say a file exists. The error names the missing observable state (reachable URL or served page). The test fails when the check is removed.
2. [behavior] Run `cargo test -p athena-core`. A replay test runs refusing_check over the recorded request text of all three cited jobs (20261014-082328-g6vy, 20261014-174815-7bf8, 20261017-173508-0wtc), asserts the check refuses exactly the real delivery-verb misread (g6vy) and not the other two, and asserts the caught count is 1 of 3.
3. [behavior] Run `cargo test -p athena-core`. Unit tests show acceptance when a criterion names a reachable URL or served state, for deploy, share and release as well as publish, and show no refusal for 'Add a timeout setting'. Tests show detection of 'Build the docs and publish them', 'Can you deploy the site' and 'I want to release v1'.
4. [behavior] Run `cargo test -p athena-core`. Tests show an out_of_scope item such as 'Changes to shared helpers' or 'Release notes are unchanged' does not satisfy the check for a Share or Release request, and a criterion containing only 'tag' or 'URL' as incidental words (for example 'the <title> tag names the page') does not satisfy it for a publish request.
5. [content] Grep crates/athena-core/src/reading.rs shows a `step!(check_outcome_state(` line inside refusing_check, with a doc comment naming the root cause (a delivery verb read as a narrow deliverable, with no check against the state the user observes) and stating it covers 1 of the 3 cited jobs (20261014-082328-g6vy), naming the other two as outside the check.
6. [behavior] Run `cargo test -p athena-core --test refusal_corpus`. The corpus has a case in which check_outcome_state refuses and a matching accepting case, and the rule that every step! has a refusing case still passes.
7. [behavior] The change refuses more readings. Expected 5% of newly refused readings. A replay of past readings is expected to hold the job only above 10%. The higher refusal rate is intended: the check applies only to requests with a delivery verb whose reading names no observable state.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.
9. [content] CHANGELOG.md has an entry naming the root cause and the new check, with the count 1 of 3 equal to the replay test, and not claiming the other two jobs are caught. docs/decisions.md records the scope, the alternatives and the placement after check_governance_coverage. No governance file (athena.toml, defaults.toml, workflows, toolchain or lint config) changes.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing job 20261014-082328-g6vy: xtask site, .github/workflows/pages.yml, docs/design/explainer-site.md, README.md, .gitignore, xtask/tests/policy.rs and requirements-map.md edits are dropped
- Job 20261014-174815-7bf8's landed files (steward, roles, ladder) are kept and not touched
- Job 20261017-173508-0wtc's docs/ontology.toml, ontology test and decisions entry are dropped; no glossary or ontology is built here
- Changing the reader prompt or READING_RULES in crates/athena-context/src/briefs.rs, which has 3 tokens of headroom
- Defining figurative terms such as ladder, layer or tier, beyond delivery verbs; those two cited jobs are another class
- Changes to athena.toml, defaults.toml, .github/workflows or toolchain and lint configuration
- Enabling GitHub Pages or any deployment
- The files job 20261027-054230-8nz8 changed (CHANGELOG.md, reading.rs, refusal_corpus.rs, decisions.md) are kept as deliverables but its candidate is dropped and rewritten from the base

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit, replay and corpus tests; fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check lives in reading acceptance; the claim is limited to what the replay shows.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that refuse and accept real request text.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The changelog count is tied to the replay test result.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a step! check in refusing_check with a corpus case.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, four files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria use commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-054230-8nz8 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (CHANGELOG.md (new Added entry); crates/athena-core/src/reading.rs doc of check_outcome_state): The changelog says all three cited jobs were delivery-verb misreads. Only one was. The check does not fire for the other two, so the class is prevented for 1 of 3 recorded instances. From the evidence snapshot's request.json files: 20261014-082328-g6vy is 'Publish a first explainer page...', which delivery_verb detects. 20261014-174815-7bf8 ('Build what the BRIEF.md addendum ... a ladder of requirements ...') and 20261017-173508-0wtc ('From Sri's decision sheet ... Should Athena keep an ontology? ...') have no sentence-initial publish, deploy, share or release. check_outcome_state returns Ok for both, and so does a retry of either through attempt.request. Yet CHANGELOG says 'reading acceptance turned a delivery verb (publish, deploy, share, release) into a narrow deliverable ...; jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc were read that way.' The request asked for the whole class to be prevented and for a test 'for the cause and not only for one job'. The only real-request test case is the g6vy request.
> major: The reading does not match the request, so passing criteria do not show the request was met The request asked for the root cause fixed where it originates, so that the whole class is prevented, with a test 'for the cause and not only for one job'. The class is seen in 3 jobs. The reading narrowed the fix to four delivery verbs at the start of a sentence. It also claimed that job 20261017-173508-0wtc's 'shared cause ... is covered by the delivery-verb check'. That is false: from the evidence snapshot, that request (the ontology decision) and 20261014-174815-7bf8's request (ladder of requirements) contain no delivery verb, so check_outcome_state never fires for them or their retries. Only 20261014-082328-g6vy is prevented. The reading's general goal, 'a reading states what the user will see when the work is done', is enforced only for requests that open a sentence with publish, deploy, share or release.

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
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

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

- path: <state>/jobs/20261107-094333-4z98/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-094333-4z98/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`. A unit test in crates/athena-core/src/reading.rs shows check_outcome_state refusing a reading for the real g6vy request 'Publish a first explainer page on how firstmate handles work.' whose criteria only say a file exists. The error names the missing observable state (reachable URL or served page). The test fails when the check is removed.
Run `cargo test -p athena-core`. A replay test runs refusing_check over the recorded request text of all three cited jobs (20261014-082328-g6vy, 20261014-174815-7bf8, 20261017-173508-0wtc), asserts the check refuses exactly the real delivery-verb misread (g6vy) and not the other two, and asserts the caught count is 1 of 3.
Run `cargo test -p athena-core`. Unit tests show acceptance when a criterion names a reachable URL or served state, for deploy, share and release as well as publish, and show no refusal for 'Add a timeout setting'. Tests show detection of 'Build the docs and publish them', 'Can you deploy the site' and 'I want to release v1'.
Run `cargo test -p athena-core`. Tests show an out_of_scope item such as 'Changes to shared helpers' or 'Release notes are unchanged' does not satisfy the check for a Share or Release request, and a criterion containing only 'tag' or 'URL' as incidental words (for example 'the <title> tag names the page') does not satisfy it for a publish request.
Grep crates/athena-core/src/reading.rs shows a `step!(check_outcome_state(` line inside refusing_check, with a doc comment naming the root cause (a delivery verb read as a narrow deliverable, with no check against the state the user observes) and stating it covers 1 of the 3 cited jobs (20261014-082328-g6vy), naming the other two as outside the check.
Run `cargo test -p athena-core --test refusal_corpus`. The corpus has a case in which check_outcome_state refuses and a matching accepting case, and the rule that every step! has a refusing case still passes.
The change refuses more readings. Expected 5% of newly refused readings. A replay of past readings is expected to hold the job only above 10%. The higher refusal rate is intended: the check applies only to requests with a delivery verb whose reading names no observable state.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.
CHANGELOG.md has an entry naming the root cause and the new check, with the count 1 of 3 equal to the replay test, and not claiming the other two jobs are caught. docs/decisions.md records the scope, the alternatives and the placement after check_governance_coverage. No governance file (athena.toml, defaults.toml, workflows, toolchain or lint config) changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
