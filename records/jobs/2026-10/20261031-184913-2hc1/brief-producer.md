# Job 20261031-184913-2hc1: Refuse readings that rely on repo paths absent from the snapshot

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Nothing checks a reading's references to earlier jobs against the current repo. A job id named in th
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-assumes-prior-job-deliverable-exists-unverified (20261026-050336-ykx8)
> Cause: Nothing checks a reading's references to earlier jobs against the current repo. A job id named in the request or notes is taken to mean its work has landed. The earlier job may be queued, unmerged or never produced. The critic is the first ...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261026-050325-8red (job 20261026-042558-5p65); prevention it proposed: Add a check at reading acceptance. For every earlier job id, role, prompt or file the reading calls existing, it confirms the artifact is in the snapshot (landed in main, with a path found). If not, the reading must either list it as a deli...
> Introspection 20261026-065033-9gdg (job 20261026-064950-38jx); prevention it proposed: Add a check before a reading is accepted. It lists the roles, prompts and files the reading depends on and confirms each exists in the current snapshot. When one is missing, the check requires the reading either to make any request it queue...
> Introspection 20261026-092111-01tx (job 20261026-065228-02zf); prevention it proposed: Before a reading is accepted, run a check that every field, state, role or function the reading relies on is found in the current snapshot (by grep of the named symbols). It should flag any dependency that is absent, such as a "resolved" st...
> Introspection 20261026-145400-8x0c (job 20261026-145124-ssmw); prevention it proposed: Add a required step to the reading stage: for every existing path, record type, role or function the reading says a feature will use, it must cite the file and line where it exists in the current snapshot, and for each transfer between comp...
> Introspection 20261026-145920-qjpb (job 20261025-001236-brde); prevention it proposed: Add a required step to reading and planning for any request that changes "all" readers, writers or call sites. Before the plan is accepted, the stage must list every real call site found by searching the snapshot, mark each as in scope or o...
> Introspection 20261026-150536-zmp5 (job 20261026-145955-s7e2); prevention it proposed: Require any plan that changes "all readers/callers" of something to include a call-site inventory produced by actually running the scan against the current tree. Each hit must be marked move, allow-listed or out of scope, with a reason and ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: reading acceptance (check_reading_acceptance in crates/athena-core/src/reading.rs) checks only deliverable paths against the base commit, so a reading that says an earlier job's file, role or prompt "already exists" is accepted unverified and the critic finds it later (failure class reading-assumes-prior-job-deliverable-exists-unverified, seen in jobs 5p65, 38jx, 02zf and others). Add a deterministic check, for the planner/critic's reader audience, that every repo path the reading's text relies on exists in the base snapshot or is created by this change.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: check_reading_acceptance (line ~2849) runs check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_governance_coverage, then check_boundary_and_repo_files, which checks only deliverable paths (path_exists_in) and banned words over reading_text_fields; repo_files is None in a replay with no repository. No record or data file is added under crates/, so check_record_file and AnyRecord are untouched.
- Goal and audience: the reader stage and the critic. A reading that relies on a path (file, prompt, role source) that does not exist at the base commit must be repaired by the reader in its one correction, not found by the critic after planning.
- Design: a new check_reading_references in reading.rs, called last from check_reading_acceptance (or folded into check_boundary_and_repo_files). It extracts repo-path tokens (crates/, docs/, xtask/, studies/, .github/ and root files) from the summary, assumptions and acceptance criteria, and refuses any that is not a file or directory in repo_files, not a create deliverable, and not in a sentence that states it is absent or to be created. With repo_files None it checks nothing. It is deterministic and calls no model.
- Earlier jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf are the instances that surfaced the class. Their code is not redone here: each was a failed or rejected attempt whose reading assumed a design-review role or a resolved finding state that did not exist. None of their candidate files is kept or changed by this job, and none of their findings is fixed here; a later retry of them would be checked by the new rule.
- Refusing more readings is intended. expected 3% of newly refused readings.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-core: a new unit test in crates/athena-core/src/reading.rs gives a reading whose assumption says a path such as crates/athena-core/src/design_review.rs already exists, with a repo_files set lacking it, and shows check_reading_acceptance refuses it naming the path; the same test fails when the check is removed.
2. [behavior] Run cargo test -p athena-core: tests show the same reading is accepted when the path is in repo_files, when it is a create deliverable, when the text says it does not exist, and when repo_files is None; the refusal is for the class (any path), not for one job id.
3. [content] Grep of crates/athena-core/src/reading.rs shows the new check called from check_reading_acceptance, and a doc comment naming the root cause (reading acceptance never compared the reading's references with the snapshot) and jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf.
4. [content] The refusal message tells the reader how to repair it: list the missing artifact as a create deliverable, or say it does not exist; CHANGELOG.md has an entry for the new check.
5. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked all pass; the verifier runs them.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing the code-health check or the post-mortem field work of jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf
- Checking job ids against landed state or queue position
- Changes to athena.toml, defaults.toml or prompt files
- Call-site inventory checks for plans (the other proposals in the failure class)

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is placed at reading acceptance, where the unverified reference originates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests plus fmt, clippy and test criteria.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A deterministic check, not prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests that show refusal and acceptance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file of logic plus changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 37 cases in 37 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a failure is attributed to the job or to the base. Add a plan-t...
- 11 cases in 10 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena a disk lifecycle. Before the checks stage, it should automatically delete the finished producer build folder and any other regenerable artifacts (verify-side builds, caches) once they are no longer needed. If free space is still below the guard, it should hold the job instead of failing ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

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

Run cargo test -p athena-core: a new unit test in crates/athena-core/src/reading.rs gives a reading whose assumption says a path such as crates/athena-core/src/design_review.rs already exists, with a repo_files set lacking it, and shows check_reading_acceptance refuses it naming the path; the same test fails when the check is removed.
Run cargo test -p athena-core: tests show the same reading is accepted when the path is in repo_files, when it is a create deliverable, when the text says it does not exist, and when repo_files is None; the refusal is for the class (any path), not for one job id.
Grep of crates/athena-core/src/reading.rs shows the new check called from check_reading_acceptance, and a doc comment naming the root cause (reading acceptance never compared the reading's references with the snapshot) and jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf.
The refusal message tells the reader how to repair it: list the missing artifact as a create deliverable, or say it does not exist; CHANGELOG.md has an entry for the new check.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked all pass; the verifier runs them.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
