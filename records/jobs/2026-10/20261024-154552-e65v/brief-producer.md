# Job 20261024-154552-e65v: Classify PR commits by content in pr_commits, not trailers

## Request (verbatim)

> try again on 20261024-153908-1rve

## Reading (accepted by the user)

Rewrite pr_commits in athena-core land.rs to classify each commit in remote_base..head by content (records_only, declared_intervention, job_landing) so `athena land --pr` stops refusing real PRs. Job landing reuses the gate's closing_jobs logic, so a blocked job later landed via outcome-landed.json is accepted. Audience: Sri and the verifier running the athena-core land and gh tests.

Assumptions:
- Per the user's reply, a job_landing is recognised exactly as the gate's closing_jobs does: the commit adds any one of the job's closing files and the job's outcome in that commit's tree is landed; the function is reused (made pub) rather than a new rule written.
- operator/ is an owner path via a nested operator/.gitattributes containing '* athena-exclude'; attrs.rs uses git check-attr --source, which honours nested files, so a lease commit under operator/ is records_only. The fixture mirrors this with a nested operator/.gitattributes.
- A job_landing requires the job's verification record to be present in the commit's tree so the verification id can be set; missing verification or non-landed outcome is refused with the sha.
- The previous reading's deliverables (land.rs, gh.rs, invariant.rs, records-history.md, CHANGELOG.md) still apply; invariant.rs changes only to expose reusable closing-job logic.
- Only athena-core land and gh tests are run during work; a narrow athena-gate invariant run is allowed only if invariant.rs changes, and the reason is stated.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core land::`. A realistic-range test builds: reading, dispatch and introspection commits, a lease commit under operator/ (fixture has a nested operator/.gitattributes with '* athena-exclude', named in the test's doc comment), an intervention commit adding records/interventions/<month>/<id>.json, then a job's landing adding candidate, verification and outcome. No commit has an Athena-Job trailer. Kinds asserted in order: records_only x4, declared_intervention, job_landing with that job and verification id.
2. [behavior] Run `cargo test -p athena-core land::`. A blocked-then-landed test: a commit adding candidate, verification and a blocked outcome yields records_only; a following commit adding only outcome-landed.json plus governed work yields job_landing for that job with its verification id.
3. [behavior] Run `cargo test -p athena-core land::`. Refusal tests: a commit changing governed content that is neither an intervention nor a job's closing records, a landing with no verification record, and a landing whose outcome is not landed each make pr_commits return an error containing that commit's sha. A companion assertion shows an operator/ commit without the exclusion is refused with its sha.
4. [behavior] Run `cargo test -p athena-core gh::`. The gh tests pass; commits built for a Landing record carry the kind (and job/verification) from pr_commits instead of a hard-coded JobLanding and pass LandingCommit::validate.
5. [content] pr_commits and PrCommit no longer read trailers or mention Athena-Job; PrCommit carries kind plus optional job and verification. Doc comments state the content rules: records_only when fingerprint equals the parent's using the gate's fingerprint function; job_landing via the gate's closing_jobs/not_landed logic (single source of truth). An empty range is still an error.
6. [content] The old trailer-count tests are removed, not left alongside the new ones. The change description lists every writer of landing records (including run.rs retry_landing) and the test covering each, and states why any athena-gate invariant test run was done.
7. [content] docs/design/records-history.md and CHANGELOG.md describe the content-based classification and no longer describe the trailer rule.
8. [behavior] Run `cargo clippy -p athena-core --all-targets` and `cargo fmt --check`; both are clean.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing run.rs retry_landing or other landing-record writers
- Changing .gitattributes or attrs.rs semantics
- Running the full workspace test suite
- Changing the gate's invariant semantics beyond exposing reusable logic

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New land tests replace trailer tests; clippy and fmt criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests asserting commit kinds and sha-bearing errors.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Trailer dependence is removed rather than excepted.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to pr_commits and its caller plus docs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog explain the content rules for outside readers.
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

Run `cargo test -p athena-core land::`. A realistic-range test builds: reading, dispatch and introspection commits, a lease commit under operator/ (fixture has a nested operator/.gitattributes with '* athena-exclude', named in the test's doc comment), an intervention commit adding records/interventions/<month>/<id>.json, then a job's landing adding candidate, verification and outcome. No commit has an Athena-Job trailer. Kinds asserted in order: records_only x4, declared_intervention, job_landing with that job and verification id.
Run `cargo test -p athena-core land::`. A blocked-then-landed test: a commit adding candidate, verification and a blocked outcome yields records_only; a following commit adding only outcome-landed.json plus governed work yields job_landing for that job with its verification id.
Run `cargo test -p athena-core land::`. Refusal tests: a commit changing governed content that is neither an intervention nor a job's closing records, a landing with no verification record, and a landing whose outcome is not landed each make pr_commits return an error containing that commit's sha. A companion assertion shows an operator/ commit without the exclusion is refused with its sha.
Run `cargo test -p athena-core gh::`. The gh tests pass; commits built for a Landing record carry the kind (and job/verification) from pr_commits instead of a hard-coded JobLanding and pass LandingCommit::validate.
pr_commits and PrCommit no longer read trailers or mention Athena-Job; PrCommit carries kind plus optional job and verification. Doc comments state the content rules: records_only when fingerprint equals the parent's using the gate's fingerprint function; job_landing via the gate's closing_jobs/not_landed logic (single source of truth). An empty range is still an error.
The old trailer-count tests are removed, not left alongside the new ones. The change description lists every writer of landing records (including run.rs retry_landing) and the test covering each, and states why any athena-gate invariant test run was done.
docs/design/records-history.md and CHANGELOG.md describe the content-based classification and no longer describe the trailer rule.
Run `cargo clippy -p athena-core --all-targets` and `cargo fmt --check`; both are clean.

## Report

End with the producer report as your structured output.
