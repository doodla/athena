# Job 20261025-031612-6239: Stop writing Athena trailers in commits after C0 (D111)

## Request (verbatim)

> Priority: high
> Stop writing Athena commit-message trailers in new commits after C0 (decision D111, job 7 part 1b), using the fixture from 20261025-031609-k0tt. Depends on 20261025-031609-k0tt. Retries the design of job 20261025-022144-emm8, rejected for its fixture.
> Acceptance:
> - On the fixture, a new job branch holding an intervention commit, a records-only commit and the job's landing is landed through land_pr with a squash merge; the squash commit on main and the three pull-request commits (reached from the records branch) carry no Athena- lines, the landing record classifies them, and athena check --history and --attribution pass.
> - A test covers retry_landing for a blocked job after C0 (no trailer search, no Athena-Verified-Content).
> - athena status's trace and the auditor read a post-C0 job through their real entry points.
> - With no cutover_job, trailers are written as before and existing tests pass.
> Files: crates/athena-core/src/commits.rs, crates/athena-core/src/gh.rs, crates/athena-core/src/run.rs, crates/athena-core/src/status.rs, crates/athena-core/src/auditor.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

When [history] cutover_job is set, new commits after C0 (intervention, records-only, landing, squash and pull-request commits) carry no Athena- trailers. Tests use the k0tt cutover fixture (test_support.rs) and cover a landing through land_pr, retry_landing after C0, and the status trace and auditor entry points. With no cutover_job, behavior is unchanged. The audience is the maintainer and the verifier, who run the tests.

Assumptions:
- Job k0tt has landed, so test_support.rs and its CutoverFixture exist. I confirmed they are in the repository.
- Decision D111 defines C0 as the host commit of the cutover job's landing record. Whether a commit is post-C0 is read from the config at the relevant commit.
- The athena-git helpers that strip lines (squash_text_relaxed) may need a change, as the earlier emm8 attempt showed.
- The fixture's fake host squash message has no '(#n)' suffix. Tests must account for that.

## Acceptance criteria

1. [behavior] A test on the cutover fixture lands a new job branch (intervention commit, records-only commit, job landing) through land_pr with a squash merge. The squash commit on main and the three pull-request commits reached from the records branch have no Athena- lines. The landing record classifies them. check --history and check --attribution report no violations. Run: cargo test -p athena-core.
2. [behavior] A test covers retry_landing for a blocked job after C0: no trailer search, no 'closing commit not found' bail, and no Athena-Verified-Content written. Run: cargo test -p athena-core retry_landing.
3. [behavior] Tests call athena status's trace and the auditor through their real entry points on a post-C0 job and get correct results. Run: cargo test.
4. [behavior] With no cutover_job configured, trailers are written as before. All existing tests pass, cargo clippy is clean and cargo fmt --check passes. Run: cargo test, cargo clippy, cargo fmt --check.
5. [content] CHANGELOG.md has an entry, and docs/decisions.md records the D111 trailer-stop design, the alternatives and the retry_landing post-C0 behavior. It removes the stale 'retry_landing has no test' statement.
6. [content] The fixture is the k0tt helper, with a job landed the old way through the real writers. No hand-built records are substituted.

## Expected deliverables

- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-git/src/lib.rs (modify)

## Out of scope

- Job 7 part 2.
- Rewriting history or changing pre-C0 commits.
- Changes to athena.toml or defaults.toml.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Each changed reader and writer has a test. cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise real entry points: land_pr, retry_landing, the status trace and the auditor.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Trailer writing is gated at the commit writers, not patched per call site.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the trailer writers and their readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

A test on the cutover fixture lands a new job branch (intervention commit, records-only commit, job landing) through land_pr with a squash merge. The squash commit on main and the three pull-request commits reached from the records branch have no Athena- lines. The landing record classifies them. check --history and check --attribution report no violations. Run: cargo test -p athena-core.
A test covers retry_landing for a blocked job after C0: no trailer search, no 'closing commit not found' bail, and no Athena-Verified-Content written. Run: cargo test -p athena-core retry_landing.
Tests call athena status's trace and the auditor through their real entry points on a post-C0 job and get correct results. Run: cargo test.
With no cutover_job configured, trailers are written as before. All existing tests pass, cargo clippy is clean and cargo fmt --check passes. Run: cargo test, cargo clippy, cargo fmt --check.
CHANGELOG.md has an entry, and docs/decisions.md records the D111 trailer-stop design, the alternatives and the retry_landing post-C0 behavior. It removes the stale 'retry_landing has no test' statement.
The fixture is the k0tt helper, with a job landed the old way through the real writers. No hand-built records are substituted.

## Report

End with the producer report as your structured output.
