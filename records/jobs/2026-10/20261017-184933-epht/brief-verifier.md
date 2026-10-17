# Job 20261017-184933-epht: verify "Add sourced ontology file with validating test and decision entry"

## Request (verbatim)

> try again on 20261017-182453-a5sf

## Reading (accepted by the user)

Retry of the rejected ontology job (Olympus decision 5) for Sri and the verifier: add docs/ontology.toml with the seven seeded misread terms, a Rust structural test, a decision entry and a changelog line. The earlier failure was the repo vocabulary test (athena_refers_to_no_one_who_runs_it_or_where) flagging "operator" in docs/design/job-planning.md. That word is absent at base, so the new text must simply avoid the banned words.

Assumptions:
- Typed request 'try again' takes precedence: this is the ontology retry, with the earlier user corrections (seven seeded terms, no per-occurrence BRIEF.md check) still applying.
- The proposed baseline-comparison feature for job verification is a separate, larger change and is not part of this job.
- At base, grep finds no 'operator' (any case) under docs/, so job-planning.md needs no edit unless the producer's own edits introduce the word; the producer must run cargo test (including xtask policy tests) before handover and avoid the banned vocabulary in all new text.
- Audience: Sri, other agents reading the file, and the independent verifier running cargo test.
- Hermes entry covers both senses (Hermes Agent, second studied system, Part 2 S1; Hermes the Orchestration god in the Olympus Pantheon in BRIEF.md). System is Olympus; Athena is its strategy and learning role; Sri owns Olympus.

## Acceptance criteria

1. [content] docs/ontology.toml defines the seven terms (planning, job, Athena, Hermes, product versus system, landed, Sri). Each has a definition, the misread that caused it, a sources field naming existing paths or decision ids, and relations whose targets are all defined terms.
2. [content] The Hermes entry names both senses (Hermes Agent, Part 2 S1, not part of Athena; Hermes as Orchestration god in the Olympus Pantheon, BRIEF.md), says which is part of Olympus, and cites a source for each 'not part of' claim.
3. [content] Relations match Olympus: the system is Olympus, Athena is the strategy and learning role within it, Sri owns Olympus. No relation makes Athena the whole system.
4. [content] The file states a term is added only when it causes a misread and how a check reads it (parse path and fields). Landed means merged on main through a verified pull request. The Sri term says use the name or 'they', never he or she, and cites job 20261017-173504-91ys. No per-occurrence BRIEF.md cross-check exists.
5. [content] docs/decisions.md has a new entry keeping the ontology, citing the Olympus 2026-10-17 decision sheet (decision 5), listing alternatives (no ontology; per-study glossaries only), and stating studies keep their own concept model so firstmate, Hermes Agent and Athena are compared in the same terms. docs/decision-links.toml is updated if required and CHANGELOG.md has an entry.
6. [behavior] Run `cargo test --locked`: the whole suite passes, including the new ontology test on docs/ontology.toml, in-memory negative tests (missing sources field, non-existent source, dangling relation target, duplicate term), and xtask's athena_refers_to_no_one_who_runs_it_or_where. Run `cargo clippy` (clean) and `cargo fmt --check` (passes).
7. [content] No secrets or personal data, including the user's email address, appear in any new or changed file.

## Expected deliverables

- docs/ontology.toml (create)
- crates/athena-cli/tests/ontology.rs (create)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Baseline-versus-main comparison of check results in job verification (separate job).
- Per-occurrence cross-checks of term names in BRIEF.md.
- Changes to athena.toml, defaults.toml, workflows, or the xtask policy test's banned word list.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The structural test is Rust; the data file is TOML read by it.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test with negative cases; cargo test, clippy and fmt must be clean.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Ontology and docs written for outside readers, avoiding banned vocabulary.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every term carries sources that the test checks exist.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email or personal data in any file.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the test running and content of the terms, not just file existence.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer runs the full suite, including policy tests, before handover.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Baseline-check feature left out as a separate change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo test, clippy and fmt --check are the independent checks.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 23f10735dd01 → candidate f307bcccb9db. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   7 +
 crates/athena-cli/tests/ontology.rs | 280 ++++++++++++++++++++++++++++++++++++
 docs/architecture.md                |   2 +-
 docs/decisions.md                   |  22 +++
 docs/ontology.toml                  |  95 ++++++++++++
 5 files changed, 405 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (877 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
