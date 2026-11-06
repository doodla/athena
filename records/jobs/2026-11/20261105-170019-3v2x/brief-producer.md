# Job 20261105-170019-3v2x: Count docs-only skipped or CI-reused checks as seen for 'still passes'

## Request (verbatim)

> Count a check skipped for a docs-only change as evidence. The many-agents design note (a docs-only change) was rejected only because its criterion 'cargo test and cargo clippy still pass' could not cite a command seen running: Athena's record says clippy was skipped locally because only docs changed, and the test result was reused from GitHub. For a change that touches only docs, a check Athena skipped for that reason, or reused from a green GitHub run, should satisfy a 'still passes' criterion that cites it.
> 
> Files changed: the verdict's seen-running rule in crates/athena-core/src/verify.rs and its tests.
> 
> Acceptance:
> - a test where a docs-only change has clippy skipped for that reason and test reused from GitHub, and a criterion citing both is met
> - a test where a code change has clippy skipped and the criterion is still unmet

## Reading (accepted by the user)

The verdict's seen-running rule (D43, `judge` in verify.rs) now accepts a check Athena skipped by the docs-only path rule (`skip_when_only` note) as seen, when the change touches only docs. A passing CI-reused check already counts. This unblocks criteria like 'cargo test and cargo clippy still pass' on docs-only changes such as the many-agents design note. Audience: Athena's verifier and the user reading verdicts.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (`judge`: `ran` is built from checks with exit 0 or reused-without-exit on matching candidate/checks tree, lines ~370-395; unseen commands make a behavior criterion unmet at ~523-554; test helpers `check`, `reused_from_ci`, `judge_recorded`, and the existing test `a_skipped_check_that_was_not_reused_does_not_count_as_run` ~1991), crates/athena-core/src/checks.rs (`skip_note` writes 'not run: every changed path matches skip_when_only (...)' and `skipped_run` yields exit_code None, passed true, reused false), crates/athena-core/src/routing.rs (docs/design kinds skip fmt and clippy when only `docs/` or `*.md` changed), and the caller crates/athena-core/src/run/verification.rs:389 plus send_back.rs:264. The property is measured directly by calling `judge` and reading `criteria[i].met` and `commands_verified`; no reading/refusal rule or record file under crates/ is added, so records validation is not touched.
- `judge` has no changed-path input today. The producer should either add a field such as `changed_paths` to `VerdictInputs` (and set it in verification.rs, send_back.rs and the verify.rs test constructors) and treat a check as seen only when every changed path is docs-only, or rely on the skip note. Preferring the path input, because a skipped record on a code change must not count. Docs-only means the same `docs/` and `*.md` patterns routing uses, ideally via the shared `path_matches` rather than a copy.
- Only a skipped record whose note comes from the `skip_when_only` rule counts; a skip for another reason (for example 'not run: CI was not green') or a failed record still does not. The existing candidate-tree and checks-tree match condition stays.
- The user said the files changed are verify.rs and its tests, so the run caller edits are minimal field plumbing only. No change to athena.toml, defaults.toml or workflows is needed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/verify.rs has a docs-only change (changed paths only under docs/ or *.md) with clippy skipped by the skip_when_only rule and test reused from GitHub CI. A criterion citing `cargo clippy` and `cargo test` is met, with commands_verified true and no D43 'cited but not seen running' finding. Run `cargo test -p athena-core verify` to check.
2. [behavior] A new test has a code change (a .rs path changed) with clippy recorded as skipped. A criterion citing `cargo clippy` is still unmet, with commands_verified false and the 'cited but not seen running' finding. Run `cargo test -p athena-core verify` to check.
3. [behavior] Existing tests still pass, including a_skipped_check_that_was_not_reused_does_not_count_as_run, a_failed_check_reused_from_ci_does_not_count_as_run and the other-tree tests; a skip for a reason other than the docs-only path rule, or a failed check, still does not count. Run `cargo test` and `cargo clippy --all-targets`.
4. [content] The doc comments on `VerdictInputs`/`judge` in verify.rs state the new rule: a check skipped by the docs-only path rule on a docs-only change counts as seen, and name the docs-only definition used.
5. [behavior] The change does not widen what counts as seen for non-docs changes. Run `cargo test -p athena-core` and read the diff with `git diff` to confirm the only new counting path is gated on docs-only changed paths.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing which checks routing skips or the skip_when_only patterns
- Changing how CI results are reused (ci_result.rs, checks.rs)
- Re-running or re-landing the many-agents design note
- Any change to athena.toml, defaults.toml, or .github/workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests plus cargo test and clippy must stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria call `judge` and check met and commands_verified.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule is fixed in the judge's seen-running rule, not by special-casing one job.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test on the new tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in verify.rs, with minimal plumbing at callers.
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
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-170019-3v2x/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-170019-3v2x/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-core/src/verify.rs has a docs-only change (changed paths only under docs/ or *.md) with clippy skipped by the skip_when_only rule and test reused from GitHub CI. A criterion citing `cargo clippy` and `cargo test` is met, with commands_verified true and no D43 'cited but not seen running' finding. Run `cargo test -p athena-core verify` to check.
A new test has a code change (a .rs path changed) with clippy recorded as skipped. A criterion citing `cargo clippy` is still unmet, with commands_verified false and the 'cited but not seen running' finding. Run `cargo test -p athena-core verify` to check.
Existing tests still pass, including a_skipped_check_that_was_not_reused_does_not_count_as_run, a_failed_check_reused_from_ci_does_not_count_as_run and the other-tree tests; a skip for a reason other than the docs-only path rule, or a failed check, still does not count. Run `cargo test` and `cargo clippy --all-targets`.
The doc comments on `VerdictInputs`/`judge` in verify.rs state the new rule: a check skipped by the docs-only path rule on a docs-only change counts as seen, and name the docs-only definition used.
The change does not widen what counts as seen for non-docs changes. Run `cargo test -p athena-core` and read the diff with `git diff` to confirm the only new counting path is gated on docs-only changed paths.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
