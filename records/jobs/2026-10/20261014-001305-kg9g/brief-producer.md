# Job 20261014-001305-kg9g: Revise decision-sets and defaults-loop designs per review 1 section 6

## Request (verbatim)

> Revise docs/design/decision-sets.md and docs/design/defaults-loop.md to make every change listed in section 6 of docs/design/reviews/decision-sets-review-1.md, settling F1 to F4 first, so the pair reaches 'accept' and the first build in the review's section 5 can start. Design only; keep the review file unchanged.

## Reading (accepted by the user)

Edit the two design docs to apply all eight section-6 changes of review 1, settling F1–F4 (reader cap and runtime budget check, store for typed default fields, host-error record and landing trigger, one evidence standard and D87 enforcement fix) before the rest. The review file stays unchanged; no code is touched.

Assumptions:
- The pair 'reaches accept' by addressing every section 6 item in the two designs; a fresh review is out of scope.
- F2 resolves to a separate typed record keyed by default id, or to new optional defaults.toml keys deferred to a governance job; the producer picks one and records alternatives.
- D90's stale 'section 9' reference lives in docs/decisions.md, which is an accepted log; it is noted in the designs as a correction needed rather than edited, unless editing is allowed.
- Items needing probes of the gateway (multi-question call) are written as pre-build requirements, not performed.

## Acceptance criteria

1. [content] F1 is settled in decision-sets.md: the reader gets sets only through code-side class/phase narrowing with a stated cap, not the whole set; a new runtime per-job set-size check is named as a build item with its own test; the per-job 'User defaults that apply' brief section is given a budget or declared unbudgeted; defaults are stated not to count against producer/verifier budgets today.
2. [content] Both designs replace 'refused by code' for over-budget sets (decision-sets.md §3, defaults-loop.md §2 R2) with wording that says the refusal is a test, not a job-time check.
3. [content] F2: one store is chosen for assumptions and signals of defaults (defaults.toml keys via a governance job, or a separate typed record keyed by default id), and athena.defaults-proposal/1 in defaults-loop.md §8.1 gains the fields.
4. [content] F3: a host-error record is specified with a status-code field, operation and job, written by the client; landing and host errors are added as watcher and introspection triggers (defaults-loop.md §3.1); the sentence saying rejections have no record is corrected to cite OutcomeState::Rejected.
5. [content] F4: one evidence rule is stated (a single event puts a decision under review; N=3 distinct jobs governs adding a default), the 'repeats' sentence in decision-sets.md §5 is made consistent, and the D87 worked example's enforcement is changed from code-check to a brief/design kind.
6. [content] F5/J2–J5: the designs state which matching parts are shared and which differ; choose one question shape for set membership; define the boolean threshold as the probability of the 'match/apply' option, not p_top; extend the defaults-loop.md §5 stored-field list for host-error and check records (no stderr from other jobs); name the 'request summary' field; require probing a multi-question call before the client is written.
7. [content] J6: defaults-loop.md §10 extends the labeled set, scoring and promotion criteria to set membership and signal match.
8. [content] F6/F7/F8: the class vocabulary is closed (or landing is stated as phase-only); decisions_matched is added to athena.introspection/1; a no-recursion rule says a firing's introspection is not watched, and host-error/check-failure observed interventions do not count toward N; the under-review store is named and recorded as a versioned, hashed brief component.
9. [content] F9 corrections in the two designs: the D89/changelog wording and the '12 entries' migration claim, the D01–D90 vs D91 count, §7 item 8's citation of the log header, and the 1,500-token estimate for defaults render(); both documents' status lines are updated to reflect the revision.
10. [content] docs/design/reviews/decision-sets-review-1.md is byte-for-byte unchanged and no file under crates/, defaults.toml, or records/ is modified (git diff --stat shows only design docs).
11. [behavior] Run `git status` and `git diff --stat`: only files under docs/ (and optionally CHANGELOG.md) appear, and the review file is absent from the diff.

## Expected deliverables

- docs/design/decision-sets.md (modify)
- docs/design/defaults-loop.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any code, tests or defaults.toml changes
- Editing the review file
- Running the first build of review section 5
- Calls to the Jev gateway
- Editing records/

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Changes are design revisions with alternatives, for later review.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: New claims cite files read or are labeled estimate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the two design docs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: F1–F4 settled first as the root issues before dependent edits.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Plain text written for an outside reader.

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

## Report

End with the producer report as your structured output.
