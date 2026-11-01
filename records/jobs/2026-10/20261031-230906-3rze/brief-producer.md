# Job 20261031-230906-3rze: Push each candidate to athena/ci/<job id> and delete it at close

## Request (verbatim)

> Priority: urgent
> Push each job's candidate to a CI branch on GitHub, and delete it when the job closes
> 
> First of two small jobs replacing 20261031-150034-17c3, which failed twice: planning grew to 28 items and the writer hit its 3000s limit. This part only pushes; nothing reads the CI result yet.
> 
> 1. Right after the candidate commit is admitted, push it to branch athena/ci/<job id> on the origin remote (through the existing gh::Remote push), and make .github/workflows/ci.yml run on pushes to athena/ci/**.
> 2. Delete athena/ci/<job id> from the remote when the job closes, whatever the outcome.
> 3. With no GitHub origin, do nothing and say so in one log line.
> 
> Tests must not reach the network: fake the remote, and for any test with an https github.com origin also install early_seam with recording fakes, as run_pin_tests.rs does.
> 
> Acceptance:
> - A test through Athena::go shows the CI branch is pushed after admission and deleted at close, through recording fakes.
> - A test shows a repository with no GitHub origin pushes nothing.
> - ci.yml has a push trigger for athena/ci/**, and the required-check job keeps its name 'fmt, clippy, tests'.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/gh/early.rs, .github/workflows/ci.yml, crates/athena-core/src/run/tests.rs, CHANGELOG.md

## Reading (accepted by the user)

Reads job 20261031-150034-17c3's first half: after the candidate is admitted, push it to origin branch athena/ci/<job id> through gh::Remote, and delete that branch when the job closes. ci.yml also runs on pushes to athena/ci/**. Nothing reads the CI result yet. With no GitHub origin, one log line says so and nothing is pushed. The audience is the maintainer, who judges it by the tests and CI.

Assumptions:
- Code survey: I read crates/athena-core/src/gh/remote.rs (the Remote trait has remote_head, push, replace and commits_after, but no delete), crates/athena-core/src/gh/early.rs (early_pr skips non-https-github origins silently, and under cfg(test) early_pr_to uses early_seam::current(); check_and_verify calls early_pr), and crates/athena-core/src/run_pin_tests.rs (the existing early_seam install pattern; this path exists, there is no run/run_pin_tests.rs). The entry point is Athena::go in crates/athena-core/src/run/mod.rs. No record or data file is added, so no records validation or AnyRecord change is needed. The behavior is measured directly by recording fakes that log push and delete calls.
- Prior job 20261031-150034-17c3 is addressed by splitting. This job keeps only its push and delete parts (items 1, 2 and 5 of that request, without resolve, athena.toml wait bound, verify::reuse_statement or checks-stage wiring). Its findings about the CI result and reuse statement no longer apply because nothing reads the CI result here. The major finding (real network via early_pr) is fixed: every test with an https github.com origin also installs early_seam with recording fakes, as run_pin_tests.rs does. The ci.yml trigger comment finding is fixed by updating the comment. The history job finding does not apply because nothing reads the result. That job's candidate recorded no changed files, so no earlier work is kept. Its other planned files (ci.rs, ci_tests.rs, checks.rs, verification.rs, close.rs, routing.rs, verify.rs, athena-git, config.rs, athena.toml, docs/design/ci-routing.md) are dropped.
- Add a Remote::delete_branch method with a default that returns an error, as replace does, implemented for GitRemote with git push origin --delete. This needs crates/athena-core/src/gh/remote.rs, which the request did not list. The fake remote in tests records it.
- The push is a forced push (replace-style or a plain refspec), so a branch left by a killed job does not block it. A push or delete failure is one warning line and does not change the job's outcome. Delete runs in a close path that covers every outcome. The skip log line is emitted once per job when origin is missing or not an https github.com URL, for both push and delete.
- athena.toml, defaults.toml and Cargo/clippy config do not change. .github/workflows/ci.yml changes only the push trigger and its comment; the job named 'fmt, clippy, tests' keeps its name.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a test in crates/athena-core/src/run/tests.rs drives Athena::go with a recording fake Remote and early_seam installed with recording fakes. It shows branch athena/ci/<job id> is pushed with the admitted candidate commit after admission and deleted at close, in that order, and that the early pull request fake was also called, so the two pushes are separate.
2. [behavior] Run `cargo test -p athena-core`: a test through Athena::go with no origin, or a non-GitHub origin, records zero pushes and zero deletes on the fake remote, and the UI output has exactly one line saying CI branch push is skipped because there is no GitHub origin.
3. [behavior] Run `cargo test -p athena-core`: a test shows the CI branch is deleted at close when the job ends in failure or rejection, not only on success.
4. [content] Grep .github/workflows/ci.yml: the push trigger lists athena/ci/** as well as main, the comment above it describes these as Athena candidate CI runs, and the job name 'fmt, clippy, tests' is unchanged.
5. [content] Grep the new tests: every test with an https://github.com origin installs early_seam with recording fakes, and no test reaches the network.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
7. [content] CHANGELOG.md has an entry for the CI branch push and delete, saying that nothing reads the CI result yet.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/remote.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/mod.rs (modify)

## Out of scope

- Reading or using the CI result in the checks stage, calling resolve, the athena.toml wait bound, or the verifier reuse statement (the second job).
- Changing what the CI workflow checks beyond its trigger and comment.
- Real network calls in tests, or giving the producer GitHub access.
- Changing ci_result.rs, gh/client.rs, the local check implementation, athena.toml, or docs/design/ci-routing.md.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus one workflow YAML.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for push/delete and the no-origin case, with fmt, clippy -D warnings and cargo test passing.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only push and delete; the result reading is left to the second job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check recorded push and delete calls through Athena::go.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Branch cleanup is enforced in code at close and checked by test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and the code comments are written for an outside reader.
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

Run `cargo test -p athena-core`: a test in crates/athena-core/src/run/tests.rs drives Athena::go with a recording fake Remote and early_seam installed with recording fakes. It shows branch athena/ci/<job id> is pushed with the admitted candidate commit after admission and deleted at close, in that order, and that the early pull request fake was also called, so the two pushes are separate.
Run `cargo test -p athena-core`: a test through Athena::go with no origin, or a non-GitHub origin, records zero pushes and zero deletes on the fake remote, and the UI output has exactly one line saying CI branch push is skipped because there is no GitHub origin.
Run `cargo test -p athena-core`: a test shows the CI branch is deleted at close when the job ends in failure or rejection, not only on success.
Grep .github/workflows/ci.yml: the push trigger lists athena/ci/** as well as main, the comment above it describes these as Athena candidate CI runs, and the job name 'fmt, clippy, tests' is unchanged.
Grep the new tests: every test with an https://github.com origin installs early_seam with recording fakes, and no test reaches the network.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
CHANGELOG.md has an entry for the CI branch push and delete, saying that nothing reads the CI result yet.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
