# Job 20261030-195941-prtf: Retry job d103: read-only evidence snapshot and athena status --snapshot

## Request (verbatim)

> try again on 20261030-070002-d103

## Reading (accepted by the user)

Retry of job 20261030-070002-d103, which failed only on low disk before the verifier (environment, not the work) and was never verified. Redo its change: at dispatch write a read-only snapshot of records/ and main's first-parent history outside the working copy, name it in producer and verifier briefs, and add `athena status --snapshot <dir>`. Audience: producers and verifiers who must check real-records criteria.

Assumptions:
- Previous result, job 20261030-070002-d103: it ended failed with no verdict because free disk was 9.3 GB against the 10.0 GB required before the verifier. Its one recorded reason is environmental, so there is no verifier finding to fix. Its candidate was discarded and this retry starts from base, redoing the same snapshot feature. Its 19 changed files are each addressed in deliverables (kept as the expected touch set) or out_of_scope.
- Disk hygiene (auto-cleanup, hold-as-waiting-for-disk) from introspection 20261030-195920-9eng is a separate follow-up and is out of scope here; this job only needs the disk to be free when it runs. That prevention was already tried in a prior case, so it is not re-proposed.
- Code survey: read crates/athena-core/src/status.rs (status body and throughput section), crates/athena-core/src/store.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/disk.rs (existing free-disk guard, untouched), crates/athena-core/src/run/ (dispatch writes the snapshot before the producer is spawned, briefs built afterwards), crates/athena-context/src/briefs.rs, crates/athena-cli/src/main.rs. The snapshot is runtime state under the state dir, not a record file under crates/, so check_record_file and AnyRecord do not apply. Throughput is measured by calling the same status function on the snapshot and on the main checkout and comparing the section text directly.
- The snapshot lives at <state>/jobs/<id>/evidence/ holding records/, landings/ and history.json (last 200 first-parent commits: hash, commit time, subject, trailers), locked read-only. Stored briefs use a <state> placeholder so no machine path is committed.
- Files outside the original scope (crates/athena-agent/src/lib.rs, policy.rs, claude.rs, crates/athena-core/src/reading.rs, run/planning.rs, run/verification.rs, admission.rs, tests/live.rs) are kept as likely needed: agent read policy must allow the evidence path, and the planner, verifier and reader briefs must carry it. They change only as needed for that; if the producer finds them unneeded they stay untouched.

## Acceptance criteria

1. [behavior] A test through the real dispatch path (run `cargo test -p athena-core`) shows the snapshot directory exists at the path the briefs name, is read-only (a write attempt fails), contains the records copied at dispatch, and contains main's first-parent history (hash, commit time, subject, trailers) at dispatch.
2. [behavior] A test (run `cargo test -p athena-core status`) shows `athena status` run against the snapshot gives a throughput section identical to the main checkout's for the same records and history.
3. [behavior] A CLI test (run `cargo test -p athena-cli`) shows `athena status --snapshot <dir>` reads records and history from the snapshot and works with no records/ directory in the current directory.
4. [behavior] Tests (run `cargo test -p athena-context`) show the producer and verifier briefs both state the snapshot path and say real-records checks use it; the working copy still has no records/.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has a new entry describing the evidence snapshot and the `athena status --snapshot` option (checked with git diff).
7. [content] git diff --stat shows no change to athena.toml, defaults.toml, .github/workflows, records/ or the throughput computation, and no machine-specific path or personal data in code, tests or docs.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/snapshot.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (create)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-cli/tests/live.rs (modify)
- crates/athena-git/src/lib.rs (modify)

## Out of scope

- Disk hygiene: automatic cleanup, waiting-for-disk holds, admission disk checks (separate follow-up from introspection 20261030-195920-9eng)
- Redoing the landing-stage matching fix from job 20261029-233021-e5sg
- Isolating sccache statistics per measurement (job 20261029-210827-18fc); its landed files are kept as landed
- Putting records/ into the producer's working copy
- Changing athena.toml, defaults.toml, workflow files or records/
- Changing how the throughput section is computed

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for snapshot, status and briefs; fmt, clippy -D warnings and cargo test criterion.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the snapshot being read-only and the throughput matching, not that files exist.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Stored briefs use a <state> placeholder; no machine paths or personal data committed.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Scope limited to the snapshot feature; disk hygiene excluded.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion that real-records checks work through the snapshot path.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

A test through the real dispatch path (run `cargo test -p athena-core`) shows the snapshot directory exists at the path the briefs name, is read-only (a write attempt fails), contains the records copied at dispatch, and contains main's first-parent history (hash, commit time, subject, trailers) at dispatch.
A test (run `cargo test -p athena-core status`) shows `athena status` run against the snapshot gives a throughput section identical to the main checkout's for the same records and history.
A CLI test (run `cargo test -p athena-cli`) shows `athena status --snapshot <dir>` reads records and history from the snapshot and works with no records/ directory in the current directory.
Tests (run `cargo test -p athena-context`) show the producer and verifier briefs both state the snapshot path and say real-records checks use it; the working copy still has no records/.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has a new entry describing the evidence snapshot and the `athena status --snapshot` option (checked with git diff).
git diff --stat shows no change to athena.toml, defaults.toml, .github/workflows, records/ or the throughput computation, and no machine-specific path or personal data in code, tests or docs.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
