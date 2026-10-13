# Job 20261013-232806-86ha: verify "Revise defaults-loop design: apply review 1, add introspection step"

## Request (verbatim)

> Revise docs/design/defaults-loop.md in two ways. First, make every change listed in docs/design/reviews/defaults-loop-review-1.md, so it reaches 'accept'. Second, add introspection as a first-class step, from the BRIEF.md addendum '2026-10-13: Introspection on every correction'. Every correction, intervention, rejection and failure triggers an introspection that Athena runs itself and records. The record covers: what happened, which class of failure it is and earlier cases of that class, the cause beyond this instance, what would have prevented it (a default, a check in code, or a design change), and a proposed next request. A retry starts from that record rather than from the user rewriting the request. The defaults loop consumes these records. Matching a new case to an earlier class is a typed decision suited to Jev, through /v1/evaluate. Use today's operator-found cases as evidence: CHANGELOG excluded by scope criteria (8qkc, m1bs); the reader budget failing four jobs in a row (69gf, t1y6, 3p3y, jk8v) while each retry only patched the last finding; 'checks not finished' on #47 with no diagnosis of a hung CI; a Haiku producer handing in untested work (69gf); a plan to edit the owner-only BRIEF.md (16zf); plans bundling design and build; Jev's interface assumed instead of checked. Design only.

## Reading (accepted by the user)

Revise docs/design/defaults-loop.md to apply every change in review 1 and add introspection as a first-class step, using the operator's cases as reported evidence. Per the user's reply, the design must say which part of Athena reads past records (code outside the agent snapshot or a filtered view) and what it passes on. Design only.

Assumptions:
- The review file lists seven changes (F1-F5, two Medium, Lows) as in the earlier reading; I did not re-read it.
- D90 in docs/decisions.md must be edited to stay consistent with the F1 change.
- Records are read by Athena code outside the agent snapshot, which hands the introspecting agent only a filtered summary; this is the stated assumption the doc will justify.
- Design only: no code for introspection is written.

## Acceptance criteria

1. [content] defaults-loop.md restates Jev's role so Jev only answers typed questions (cluster assignment or pairwise boolean) and a language model drafts statement, applies_to and title; D90 in docs/decisions.md matches (F1).
2. [content] Section 2 labels the R2 reader-budget overrun as a counter-example, states R1 is closed in code by D89, marks the loop's case as unproved, and gates any build on at least N distinct jobs a default would cover (F2).
3. [content] A section documents POST /v1/evaluate: request fields (model, state, questions with type, instructions, criteria), response fields (choice, probabilities, confidence), chat-route refusal as a build error, fake Jev following that shape, cost/tokens read from the response (F4).
4. [content] Top-probability and confidence signals are defined with what each controls, fallback vs explicit second call is chosen, and the answering model is stored in evidence (F5).
5. [content] Reader budget is stated by first sentence plus id/scope and entry count, says where the full statement lands, defines the intervention record as the single counted unit, and makes a render() measurement with the live counter a gate on the 1,500-token estimate.
6. [content] Promotion criterion computes the test-part size needed for a skill lower bound or drops the interval for user judgment (F3); D26 'eleven' and the RM-3 citation are corrected; Low items from the review are done.
7. [content] Introspection is a first-class step: triggered by every correction, intervention, rejection and failure, run and recorded by Athena, with fields what happened, failure class and earlier cases, cause beyond the instance, prevention (default, code check or design change), proposed next request; retries start from the record; the defaults loop consumes records; class matching is a typed /v1/evaluate decision for Jev.
8. [content] The design states which part of Athena reads past records/ (code outside the agent snapshot, or a filtered view given to the introspecting agent), picks one, and says exactly what it passes on to the agent and onward.
9. [content] The operator's cases (8qkc/m1bs CHANGELOG; 69gf/t1y6/3p3y/jk8v reader budget; #47 checks not finished; 69gf Haiku untested; 16zf BRIEF.md edit; design+build bundling; Jev interface assumed) are each labelled reported, with class and what would have prevented it.
10. [content] No Rust, config, defaults.toml, BRIEF.md, records/ or review file changes; CHANGELOG.md has an entry.
11. [behavior] Run `git status` and `git diff --stat`: only markdown files under docs/ and CHANGELOG.md changed; `cargo test` still passes.

## Expected deliverables

- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Any Rust code or tests for introspection
- Edits to BRIEF.md, defaults.toml, athena.toml, records/ or the review file
- Implementing /v1/evaluate clients

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Revision is a written design with alternatives (records reader choice, fallback choice).
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Operator cases labelled reported; numbers measured or labelled estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One document revision plus its decision-log and changelog entries; no code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Job ids only; no personal data.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Introspection design targets causes beyond the instance.

## The change

Base 986f1a2cb663 → candidate 64718a1445fe. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                 |   6 +
 docs/decisions.md            |  21 +-
 docs/design/defaults-loop.md | 658 ++++++++++++++++++++++++++++---------------
 3 files changed, 449 insertions(+), 236 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (382 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
