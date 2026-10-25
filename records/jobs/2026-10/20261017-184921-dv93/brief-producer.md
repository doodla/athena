# Job 20261017-184921-dv93: Review docs/design/job-planning.md against the code

## Request (verbatim)

> Write an independent review of docs/design/job-planning.md as docs/design/reviews/job-planning-review-1.md, in the format of the existing reviews in docs/design/reviews/ (severity: High blocks the build, Medium must be settled in the design or the first build job, Low). It was written by hand at Sri's direction (BRIEF.md addenda of 2026-10-17); review it as you would any design. Do not name who runs Athena or where in the review: the boundary test in xtask/tests/policy.rs fails on those words. Check it against the code as it is: that the functions, files and records it names exist and behave as it says (run.rs dispatch and produce_and_verify, manage.rs recover_interrupted, store.rs job_of, admission::is_governance and governance_change_is_listed, athena_agent::policy::evaluate, Config::validate, verify::judge); that the five build jobs in section 8 are ordered so no config change lands before its schema; that each job fits one producer run; that the keep rule in section 9 can be computed from Athena's records; and what the design misses. Do not change the design or any code.
> Acceptance:
> - docs/design/reviews/job-planning-review-1.md exists, names the commit it reviewed, and for each of the design's sections 3 to 9 gives its findings or says it has none; each finding has a severity, evidence (file and line, or a record), and a proposed fix.
> - Each code claim marked (read) in the design that the review checked is listed as confirmed or contradicted, with the line checked.
> - The review ends with a verdict: build as designed, build after the listed fixes, or redesign.
> - Only the review file and CHANGELOG.md change; cargo fmt --check and cargo test pass.

## Reading (accepted by the user)

Write an independent review of docs/design/job-planning.md in the existing review format, checking its (read) code claims against the code, the section 8 job ordering and sizing, and the section 9 keep rule. Only the review file and CHANGELOG.md change; it ends with a verdict.

Assumptions:
- Audience is Sri and the build-job verifier; goal is an independent, evidence-cited review ahead of building the design.
- The reviewed commit is the one checked out when the producer runs; it records its hash.
- Existing reviews in docs/design/reviews/ define the format; producer reads them first.
- Design is docs-class work; no code or governance files change.

## Acceptance criteria

1. [content] docs/design/reviews/job-planning-review-1.md exists, names the reviewed commit, and follows the format of existing reviews in docs/design/reviews/ (severity High/Medium/Low, evidence, proposed fix per finding).
2. [content] For each of the design's sections 3 to 9 the review lists findings or states it has none; every finding has severity, evidence (file:line or record) and a proposed fix.
3. [content] Each (read) code claim in the design that was checked (run.rs dispatch and produce_and_verify, manage.rs recover_interrupted, store.rs job_of, admission::is_governance, governance_change_is_listed, athena_agent::policy::evaluate, Config::validate, verify::judge) is listed as confirmed or contradicted with the line checked.
4. [content] The review assesses section 8's five build jobs for ordering (no config change before its schema) and single-producer-run fit, and whether section 9's keep rule is computable from Athena's records.
5. [content] The review ends with a verdict: build as designed, build after the listed fixes, or redesign.
6. [content] The review does not name who runs Athena or where (boundary test in xtask/tests/policy.rs); the design file and code are unchanged.
7. [behavior] Run cargo fmt --check and cargo test: both pass, including the xtask policy boundary test; git status shows only the review file and CHANGELOG.md changed.
8. [content] CHANGELOG.md has an entry for the review.

## Expected deliverables

- docs/design/reviews/job-planning-review-1.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Editing docs/design/job-planning.md
- Changing any code, tests, or configuration
- Implementing the five build jobs

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every finding cites file and line or a record; unmeasured numbers labeled estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review written for an outside reader, without naming the operator or host.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data; avoids words the boundary test rejects.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Code claims checked against actual lines, not existence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo fmt --check and cargo test run by verifier.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One review file plus changelog.
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

## Report

End with the producer report as your structured output.
