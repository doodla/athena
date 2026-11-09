# Job 20261109-093015-m0rq: verify "Add README section: what a job may and may not change"

## Request (verbatim)

> try again on 20261109-080749-p0fs

## Reading (accepted by the user)

Retry of 20261109-080749-p0fs: add a short README section (for engineers) saying every part of Athena's design can be changed by a job, with the exception list derived from admission.rs and .gitattributes, plus a CHANGELOG entry.

Assumptions:
- Retry of job 20261109-080749-p0fs. Its candidate (README.md, CHANGELOG.md) is dropped and redone from base. The finding that it listed only two exceptions is fixed by deriving the list from crates/athena-core/src/admission.rs (line 5 and line 9) and .gitattributes; the minor finding is fixed by citing admission and .gitattributes for BRIEF.md instead of its header.
- Audience: engineers reading the repo; goal: Athena's docs say that no part of its design is fixed, and state exactly what a job cannot change.
- Exceptions read from source: paths under records/ (admission.rs:5, D06); athena-readonly paths, i.e. /BRIEF.md and /LICENSE (.gitattributes); athena-exclude paths, i.e. /CLAUDE.md, private to the owner (.gitattributes); a candidate that changes how any path is classified (admission.rs:9, so .gitattributes is effectively locked). Cite docs/decisions.md D09 for the unchangeable paths.
- Other admission refusals (submodules, size limit, emails/secrets, unparseable config, unlisted governance files) are conditions on a change, not unchangeable parts; the section may mention governance files need listing in the reading, citing admission.rs:17.
- The section gives no count such as 'two' unless it matches the source list. Existing-style README headings are '## ...'; the new heading contains 'change'. CHANGELOG entry goes under [Unreleased] / Changed in the existing Keep a Changelog format.
- No prompt, athena.toml, defaults.toml or workflow file changes, so no governance file is touched and prompt budgets are unaffected.

## Acceptance criteria

1. [content] README.md has a new section whose heading contains 'change' (Grep '^## .*[Cc]hange') stating that every part of Athena's design (code, rules, roles, prompts, docs) can be changed by a job.
2. [content] The section lists every path class admission refuses, each mapped to its source: records/ and athena-readonly/athena-exclude paths (admission.rs line 5), BRIEF.md and LICENSE (readonly) and CLAUDE.md (exclude) per .gitattributes, and changes to path classification (admission.rs line 9); it cites D09 in docs/decisions.md. Checkable by Grep for 'records/', 'BRIEF.md', 'CLAUDE.md', 'athena-readonly', 'athena-exclude', 'classif' in the section.
3. [content] The section gives no count ('two', 'only N') that disagrees with the admission.rs and .gitattributes list, and does not describe BRIEF.md's protection as a header statement; every link in it resolves to an existing file.
4. [content] The section says a design change is an ordinary request through the change pipeline, recorded in docs/decisions.md, and claims no mechanism absent from the repository.
5. [content] CHANGELOG.md has an [Unreleased] / Changed entry for the README addition that matches the README's list of exceptions.
6. [content] git diff shows only README.md, CHANGELOG.md (and optionally docs/architecture.md) changed; no prompt, athena.toml, defaults.toml, .gitattributes or workflow file. Not a new refusal rule, so no refusal-rate change.
7. [behavior] Run cargo test and cargo clippy: both pass, including any tests that check docs links or changelog format.

## Expected deliverables

- README.md (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing prompt files (token budgets nearly full)
- Changing athena.toml, defaults.toml, .gitattributes or workflows
- Changing BRIEF.md, LICENSE, CLAUDE.md or records/
- Any code or admission rule change

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the README list matches the admission source and links resolve.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each claim in the section cites a file.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only change to two files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo test and clippy run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e27875006dbf → candidate eb49e9cf06a2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md |  5 +++++
 README.md    | 25 +++++++++++++++++++++++++
 2 files changed, 30 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-093015-m0rq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-093015-m0rq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42594292508 for eb49e9cf06a2cd25bfe7f5876cd1fb028680e5a4)
- ci-result: `not a check` passed (Check results for eb49e9cf06a2cd25bfe7f5876cd1fb028680e5a4 come from the CI run https://github.com/doodla/athena/actions/runs/42594292508 (687s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo test and cargo clippy: both pass, including any tests that check docs links or changelog format.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
