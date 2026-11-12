# Job 20261112-135930-pczv: Guard Athena's pushes to origin only; give OLY-B-BND-01 a mechanism

## Request (verbatim)

> try again on 20261030-013721-4gwn

## Reading (accepted by the user)

Retry of job 20261030-013721-4gwn (failed in plan critique round 3, before any code). Add a guard so Athena's own publishing (push, replace, delete_branch, PR/GitHub writes) reaches only the repo's own `origin`, record it as OLY-B-BND-01's mechanism and measure in docs/spec.toml, with a test that fails without it. Goal: Themis sees the boundary enforced by code and a test. The "try again" request is read as a retry of the original job, not the introspection's proposed reading-time check.

Assumptions:
- Prior job 20261030-013721-4gwn: it produced no candidate (no files kept), so all its work is dropped and redone from the base. Its plan-critique findings are addressed as follows. (1) CHANGELOG.md contradiction: the old criterion 7 said CHANGELOG.md untouched, which admission (admission.rs changelog_required) refuses; it is dropped, and CHANGELOG.md is a deliverable with one Unreleased line. (2) origin_url must read `git remote get-url --all origin` and `git remote get-url --push --all origin` and refuse when either lists more than one URL or the URLs name different repositories; tests cover a doubled push URL and a doubled fetch URL with no push URL. (3) crates/athena-git/src/lib.rs is now a listed deliverable. (4) A land_pr test or text assertion shows the origin lookup comes before any write when origin is absent. (5) GitRemote's three write methods recheck origin's URL so the limit is not left to the spec text.
- Code survey: read crates/athena-git/src/lib.rs (push_if_remote_at at line 384, push_branch at 432) and searched crates/ for GitRemote push, replace and delete_branch in crates/athena-core/src/gh/remote.rs. The planning critique also cited landing.rs (land_pr, origin_client), early.rs, gh/tests.rs ScriptRunner and UnreachedHost, and hestia parse_spec, measure_problem and spec_gaps. The producer must re-read them. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly by calling the real push, replace and delete_branch against temp bare repos.
- Existing code: crates/athena-core/src/gh/remote.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/tests.rs, crates/athena-git/src/lib.rs and docs/spec.toml are extended; the guard reuses GitRemote and the existing test doubles.
- Environment: the tests use only local temp git repos with bare `origin` and `upstream` remotes and a fake runner. They make no network or GitHub call, need no credentials, and read nothing from the records branch.
- The measure must end with `status: not shown` plus the test name, since no `athena status` line exists for upstream writes.
- Out of scope items from the earlier job are kept as they were. Blocking pushes to the project's own origin stays out of scope because it is the allowed path.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-BND-01 has `mechanism` naming crates/athena-core/src/gh/remote.rs (not "none"), keeps owner Themis, and `measure` ends with `status: not shown` plus the name of the showing test (git diff docs/spec.toml).
2. [content] Grep finds the guard function in crates/athena-core/src/gh/remote.rs and its use in GitRemote push, replace and delete_branch; crates/athena-git/src/lib.rs push_branch/push_if_remote_at are guarded to origin.
3. [behavior] Run `cargo test -p athena-core`: a new test builds a temp repo with bare `origin` and `upstream` remotes, calls GitRemote push, replace and delete_branch with name `upstream`, asserts an error and unchanged upstream refs, and the same calls to `origin` succeed.
4. [behavior] Run `cargo test -p athena-core`: tests show no origin gives an error and no write, a local-path origin makes no GitHub call, a GitHub https origin targets only the parsed owner/repo, and a doubled push URL and a doubled fetch URL (no push URL) are refused.
5. [content] The new tests assert on the refusal (error and unchanged refs), so removing the guard call from GitRemote::push makes a test fail; the diff shows this and not only wiring.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] CHANGELOG.md has one new Unreleased line naming the OLY-B-BND-01 guard; athena.toml, defaults.toml and .github/workflows are untouched (git status).

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/gh/remote.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how agents' git/gh commands are policed in crates/athena-agent/src/policy.rs.
- Blocking Athena's own pushes and PRs to the project's `origin`.
- Adding an `athena status` line for upstream writes.
- Changes to athena.toml, defaults.toml or .github/workflows.
- The introspection's proposed reading-time check for criteria that contradict defaults; it is a separate job.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in crates/athena-core/src/gh/tests.rs; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert refusal of non-origin writes, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a guard in code and named as the mechanism in docs/spec.toml.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The measure ends with a status label naming the showing test.
- changelog: (not in the defaults model) Here: CHANGELOG.md gets one Unreleased entry, which admission requires.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 20 cases in 20 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs before the verifier, at reading acceptance and again after the producer finishes. It would (1) validate every measure label a job adds or edits in docs/spec.toml against STATUS_MEASURE_LABELS by running the real-spec test; (2) require each new behavior test to fail on the ...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261112-135930-pczv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-135930-pczv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml entry OLY-B-BND-01 has `mechanism` naming crates/athena-core/src/gh/remote.rs (not "none"), keeps owner Themis, and `measure` ends with `status: not shown` plus the name of the showing test (git diff docs/spec.toml).
Grep finds the guard function in crates/athena-core/src/gh/remote.rs and its use in GitRemote push, replace and delete_branch; crates/athena-git/src/lib.rs push_branch/push_if_remote_at are guarded to origin.
Run `cargo test -p athena-core`: a new test builds a temp repo with bare `origin` and `upstream` remotes, calls GitRemote push, replace and delete_branch with name `upstream`, asserts an error and unchanged upstream refs, and the same calls to `origin` succeed.
Run `cargo test -p athena-core`: tests show no origin gives an error and no write, a local-path origin makes no GitHub call, a GitHub https origin targets only the parsed owner/repo, and a doubled push URL and a doubled fetch URL (no push URL) are refused.
The new tests assert on the refusal (error and unchanged refs), so removing the guard call from GitRemote::push makes a test fail; the diff shows this and not only wiring.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
CHANGELOG.md has one new Unreleased line naming the OLY-B-BND-01 guard; athena.toml, defaults.toml and .github/workflows are untouched (git status).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
