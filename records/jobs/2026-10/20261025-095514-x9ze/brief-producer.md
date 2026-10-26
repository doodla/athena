# Job 20261025-095514-x9ze: Retry from candidate when only docs findings block a passing rejection

## Request (verbatim)

> Priority: high
> Extend retry-from-candidate (3449219) to rejections whose checks all passed and whose only blocking findings are about documentation or other non-code content, even when no criterion is marked unmet: job 20261025-034452-wqp8 passed every check and was rejected for one major finding (decision D122 inserted inside D121 in docs/decisions.md), but its retry started from the base and redid 30 minutes of code. Classify a finding as content-only when every path it cites is a non-code file (Markdown, CHANGELOG, docs/), and use that in the qualifying rule.
> Acceptance:
> - A test shows a rejection with passing checks and only a docs-located major finding retries from the candidate.
> - A finding citing a code file, or a failed check, still retries from the base.
> Files: crates/athena-core/src/run.rs, crates/athena-core/src/verify.rs, CHANGELOG.md

## Reading (accepted by the user)

Extend qualifies_for_tree_retry so a rejected job whose checks all passed, with no behavior-criterion trouble and no reading mismatch, also qualifies when no criterion is unmet but every blocking (blocker/major) finding cites only non-code paths (Markdown, CHANGELOG, docs/). A new content-only classifier on findings lives in verify.rs. Audience: the operator running `athena retry`, who stops losing ~30 minutes to a redo like job wqp8's.

Assumptions:
- The qualifying rule lives in reading.rs (qualifies_for_tree_retry, line ~1059) though the request lists run.rs and verify.rs; reading.rs must change with its tests, and run.rs only if its doc comment or seed wiring needs it.
- A finding's paths come from Finding.location (e.g. 'docs/decisions.md:3005-3053', possibly several paths) and, if location is None, from paths named in summary/evidence; a finding with no recognizable path is NOT content-only (conservative: retry from base).
- Blocking means severity blocker or major; minor/nit findings do not block and are ignored by the rule.
- Non-code means extension .md/.markdown/.txt, a file named CHANGELOG*, or a path under docs/; Cargo.toml, athena.toml, prompts, .rs and workflow files count as code/config, so they retry from base.
- The existing rule (unmet content criteria only) stays; the new branch applies when there are no unmet criteria and at least one blocking finding, all content-only. A rejection with no unmet criterion and no blocking finding still has unknown cause and does not qualify.
- The tree-retry seed's findings text already carries the findings, so the retry producer is told the docs fix needed; no new record schema.

## Acceptance criteria

1. [behavior] A test in athena-core (run with `cargo test -p athena-core`) builds a rejected job with candidate, all checks passed, no unmet criterion, and one major finding whose location is docs/decisions.md:3005-3053; qualifies_for_tree_retry returns true for it.
2. [behavior] Tests show retry still starts from the base (qualifies_for_tree_retry false) when: a major finding cites a .rs file; a finding cites both a docs path and a code path; a check failed; a behavior criterion is unmet; the verifier reported a request/reading mismatch; the candidate has admission violations; or no finding and no unmet criterion exists.
3. [behavior] A test of the content-only classifier (in verify.rs) shows docs/…, *.md, and CHANGELOG.md locations classify as content-only, and src files, Cargo.toml, a missing location with no path, and mixed paths do not; minor findings do not block qualification.
4. [behavior] A test through tree_retry_seed shows a docs-only-finding rejection yields a seed whose findings list includes the docs finding text; `cargo test` and `cargo clippy --all-targets` pass and `cargo fmt --check` is clean.
5. [content] The doc comment on qualifies_for_tree_retry states the new condition (no unmet criterion, all blocking findings cite only non-code files) and that the unknown-cause exclusion still applies otherwise.
6. [content] CHANGELOG.md has an entry under the unreleased section describing that docs-only rejections now retry from the candidate, and that code-file findings or failed checks still retry from the base.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Fixing D122/D121 in docs/decisions.md or re-running job wqp8.
- Changing which verifier findings are raised or their severities.
- Changing the base-commit-unchanged check or the seeding mechanics of a tree retry.
- Adding a new record field or schema version.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the classifier and the qualifying rule; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rule's decisions on constructed rejections, not only that a function exists.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment on the rule updated; CHANGELOG entry written for outside readers.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Generalizes the rule to all content-only blocking findings rather than special-casing D122.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test runs the verifier can execute.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the retry qualification rule.
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

A test in athena-core (run with `cargo test -p athena-core`) builds a rejected job with candidate, all checks passed, no unmet criterion, and one major finding whose location is docs/decisions.md:3005-3053; qualifies_for_tree_retry returns true for it.
Tests show retry still starts from the base (qualifies_for_tree_retry false) when: a major finding cites a .rs file; a finding cites both a docs path and a code path; a check failed; a behavior criterion is unmet; the verifier reported a request/reading mismatch; the candidate has admission violations; or no finding and no unmet criterion exists.
A test of the content-only classifier (in verify.rs) shows docs/…, *.md, and CHANGELOG.md locations classify as content-only, and src files, Cargo.toml, a missing location with no path, and mixed paths do not; minor findings do not block qualification.
A test through tree_retry_seed shows a docs-only-finding rejection yields a seed whose findings list includes the docs finding text; `cargo test` and `cargo clippy --all-targets` pass and `cargo fmt --check` is clean.
The doc comment on qualifies_for_tree_retry states the new condition (no unmet criterion, all blocking findings cite only non-code files) and that the unknown-cause exclusion still applies otherwise.
CHANGELOG.md has an entry under the unreleased section describing that docs-only rejections now retry from the candidate, and that code-file findings or failed checks still retry from the base.

## Report

End with the producer report as your structured output.
