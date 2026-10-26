# Job 20261026-065228-02zf: Add post-mortem field and retry findings (context-engineer job 4)

## Request (verbatim)

> Priority: urgent
> Build job 4 of the context and knowledge engineer design (docs/design/context-engineer.md section 11): the post-mortem field and retry findings. Add to the introspection record the 'context that would have prevented it' field with its fixed outcome list and the planning-stopped marker, add that question to Jev's call, and make a retry's brief list every unresolved blocker and major finding of the previous attempt (failure class reading-omits-prior-verifier-finding, 7 jobs). Also mark the design's decision accepted in docs/decisions.md, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), noting that its job 1 (size and touch report) is covered by the code-health check (request 20261026-042558-5p65 and its retry).
> Acceptance:
> - A test retries a job rejected with two major findings and shows both quoted in the retry's producer brief, and resolved ones left out.
> - A test shows the new introspection field and the planning-stopped marker recorded, and the question in the Jev call.
> - The decision is marked accepted.
> Files: crates/athena-data/src/records.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/jev.rs, crates/athena-core/src/run.rs, crates/athena-context/src/briefs.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Implement job 4 of the context-engineer design: add the 'context that would have prevented it' field (fixed outcome list) and a planning-stopped marker to the introspection record, add the question to Jev's call, and make retry briefs list every unresolved blocker and major finding of the previous attempt. Mark D130 accepted under the brain's delegation, noting job 1 is covered by the code-health check.

Assumptions:
- Goal: build job 4 of docs/design/context-engineer.md section 11; audience is Sri and the verifier, who judge by the tests and the decision status.
- Existing Rejection and Failure introspection triggers cover the planning stop; the marker is added to the record, not a new event.
- The prior attempt's findings are available from stored verifier findings with a severity and a resolved state; the retry builder in briefs.rs gets them as input, and run.rs passes them.
- The request lists crates/athena-core/src/run.rs and briefs.rs; the existing planning-stop reason string in run.rs is reused to detect the marker.
- D130 is the decision to mark accepted; its 'six one-producer jobs' text may need a note that job 1 is covered by the code-health check.
- Docs such as docs/records.md may describe the introspection record; updated only if a doc test requires it.

## Acceptance criteria

1. [behavior] Run `cargo test` for athena-core, athena-data and athena-context: a test retries a job rejected with two major findings and asserts both are quoted in the retry's producer brief, while a finding marked resolved is absent; the test fails if the builder drops one.
2. [behavior] A test through the existing introspection path shows a job that ends failed with the 'planning stopped before any code was written:' reason yields a record carrying the planning-stopped marker and a 'context that would have prevented it' outcome from the fixed list (system-facts-line, failure-class, brief-assembly, vocabulary-term, spec-fix, structure, none).
3. [behavior] Tests show a rejected job and a job failed for another reason each yield the field, a landed job yields no record, an outcome outside the list is stored as an error and blocks nothing, and a failed Jev call stores an error in the field with the job outcome unchanged.
4. [behavior] A test shows the text of Jev's introspection call contains the question 'What one piece of context, if the agent had read it, would have prevented this?' and the fixed outcome list.
5. [content] docs/decisions.md D130 status reads accepted, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), and notes that job 1 (size and touch report) is covered by the code-health check (request 20261026-042558-5p65 and its retry).
6. [content] CHANGELOG.md has an entry for the post-mortem field, the planning-stopped marker and the retry findings change.
7. [behavior] cargo test, cargo clippy and cargo fmt --check pass for the workspace.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)

## Out of scope

- Other build jobs of the context engineer design (1, 2, 3, 5, 6); job 1 is covered by the code-health check.
- Changing outcomes of jobs from Jev's answer; the field stays in shadow.
- Adding a new event or a second post-mortem record.
- Changes to athena.toml or defaults.toml.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for each new behavior; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the brief quoting findings and the record field being written, not that types exist.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and changelog text written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the listed files.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The retry brief lists all unresolved blocker and major findings, fixing the whole failure class.
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

Run `cargo test` for athena-core, athena-data and athena-context: a test retries a job rejected with two major findings and asserts both are quoted in the retry's producer brief, while a finding marked resolved is absent; the test fails if the builder drops one.
A test through the existing introspection path shows a job that ends failed with the 'planning stopped before any code was written:' reason yields a record carrying the planning-stopped marker and a 'context that would have prevented it' outcome from the fixed list (system-facts-line, failure-class, brief-assembly, vocabulary-term, spec-fix, structure, none).
Tests show a rejected job and a job failed for another reason each yield the field, a landed job yields no record, an outcome outside the list is stored as an error and blocks nothing, and a failed Jev call stores an error in the field with the job outcome unchanged.
A test shows the text of Jev's introspection call contains the question 'What one piece of context, if the agent had read it, would have prevented this?' and the fixed outcome list.
docs/decisions.md D130 status reads accepted, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), and notes that job 1 (size and touch report) is covered by the code-health check (request 20261026-042558-5p65 and its retry).
CHANGELOG.md has an entry for the post-mortem field, the planning-stopped marker and the retry findings change.
cargo test, cargo clippy and cargo fmt --check pass for the workspace.

## Report

End with the producer report as your structured output.
