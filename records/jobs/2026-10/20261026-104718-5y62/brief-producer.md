# Job 20261026-104718-5y62: Add post-mortem field, planning-stopped marker and retry findings

## Request (verbatim)

> try again on 20261026-065228-02zf

## Reading (accepted by the user)

Retry of the failed job: add the post-mortem field ('context that would have prevented it', fixed outcome list) and planning-stopped marker to the introspection record, add the question to Jev's call, and carry the previous attempt's blocker and major findings into retry briefs in both seeded and unseeded paths. No 'resolved' state exists, so none is invented; the real parse_response must tolerate a missing answer; D130 is marked accepted.

Assumptions:
- Goal: complete job 4 of the context-engineer design so retries stop omitting prior blocker/major findings; audience is Athena's operator (Sri) and later producers reading retry briefs.
- Findings have no resolved state today (records.rs:750), so 'resolved ones left out' is dropped: all blocker and major findings of the previous attempt's final verification are carried. This supersedes the earlier acceptance bullet about resolved findings; the proposed retry request already says not to invent one.
- The real parse_response (jev.rs:147) fails the whole reply on any missing answer, so the new question is made optional there, with a test against the real parser.
- Seeded retry path keeps its full finding list but labels blocker and major ones, so both paths tell the producer what it must address.
- Files the design touches (reading.rs, crates/athena-cli/tests/introspection.rs, docs/records.md) are listed as deliverables, as the prior planning failures required.
- Symbols relied on were grepped: parse_response, tree_retry_seed, Finding (records.rs:750, introspect.rs:666), D130 (decisions.md:3272, status proposed).

## Acceptance criteria

1. [behavior] Run `cargo test`: a retry-brief test rejects a job with two major findings and a minor one, and asserts both major findings (summary, evidence, location) appear in the retry's producer brief in BOTH the seeded (tree_retry_seed) and unseeded paths, listed the same way; it fails if the builder drops one.
2. [behavior] Run `cargo test`: a test calls the real jev.rs `parse_response` (not FakeJev) on a reply with no answer to the new context-prevention question and asserts the class and repeat answers still come back and the new field is empty; a malformed or out-of-list answer is stored as an error in the field and blocks nothing.
3. [behavior] Run `cargo test`: a test through the introspection path shows a job failed with 'planning stopped before any code was written:' yields a record with the planning-stopped marker and a 'context that would have prevented it' outcome from the fixed list (system-facts-line, failure-class, brief-assembly, vocabulary-term, spec-fix, structure, none); rejected and otherwise-failed jobs yield the field, a landed job yields no record, and a failed Jev call leaves the job outcome unchanged.
4. [behavior] Run `cargo test`: a test shows Jev's introspection call text contains 'What one piece of context, if the agent had read it, would have prevented this?' and the fixed outcome list, and that a record written with the field reads back with it.
5. [content] No 'resolved' state is invented on Finding: every blocker and major finding of the previous attempt's final verification is carried, and CHANGELOG.md and docs/records.md state this plainly.
6. [content] docs/decisions.md D130 status reads accepted, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), noting job 1 (size and touch report) is covered by the code-health check (request 20261026-042558-5p65 and its retry).
7. [content] CHANGELOG.md has an entry for the post-mortem field, the planning-stopped marker and the retry findings change.
8. [behavior] cargo test, cargo clippy and cargo fmt --check pass for the workspace.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)

## Out of scope

- Inventing a resolved state or flag on Finding
- Build jobs 2, 3 and 5 of the context-engineer design
- Changing the code-health check or job 1's size and touch report

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each new behavior; cargo test, clippy and fmt --check in criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check briefs, records and the real parser behaving, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the omission in both retry paths, not one instance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the listed files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs state the no-resolved-state choice for outside readers.
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

Run `cargo test`: a retry-brief test rejects a job with two major findings and a minor one, and asserts both major findings (summary, evidence, location) appear in the retry's producer brief in BOTH the seeded (tree_retry_seed) and unseeded paths, listed the same way; it fails if the builder drops one.
Run `cargo test`: a test calls the real jev.rs `parse_response` (not FakeJev) on a reply with no answer to the new context-prevention question and asserts the class and repeat answers still come back and the new field is empty; a malformed or out-of-list answer is stored as an error in the field and blocks nothing.
Run `cargo test`: a test through the introspection path shows a job failed with 'planning stopped before any code was written:' yields a record with the planning-stopped marker and a 'context that would have prevented it' outcome from the fixed list (system-facts-line, failure-class, brief-assembly, vocabulary-term, spec-fix, structure, none); rejected and otherwise-failed jobs yield the field, a landed job yields no record, and a failed Jev call leaves the job outcome unchanged.
Run `cargo test`: a test shows Jev's introspection call text contains 'What one piece of context, if the agent had read it, would have prevented this?' and the fixed outcome list, and that a record written with the field reads back with it.
No 'resolved' state is invented on Finding: every blocker and major finding of the previous attempt's final verification is carried, and CHANGELOG.md and docs/records.md state this plainly.
docs/decisions.md D130 status reads accepted, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), noting job 1 (size and touch report) is covered by the code-health check (request 20261026-042558-5p65 and its retry).
CHANGELOG.md has an entry for the post-mortem field, the planning-stopped marker and the retry findings change.
cargo test, cargo clippy and cargo fmt --check pass for the workspace.

## Report

End with the producer report as your structured output.
