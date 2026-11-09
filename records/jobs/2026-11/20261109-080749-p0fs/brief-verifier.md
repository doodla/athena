# Job 20261109-080749-p0fs: verify "Document that all of Athena's design is changeable"

## Request (verbatim)

> Dunno if Athena knows that literally everything of it's design can be changed

## Reading (accepted by the user)

The user worries Athena does not know its own design (rules, roles, records, docs) can be changed by jobs. Read as a docs-only change: add a short, sourced statement in README.md (for engineers reading the repo) that no part of the design is fixed, naming the real exceptions, plus a CHANGELOG entry.

Assumptions:
- Goal: make it plain to Athena's readers (engineers, and the agents that read the repo) that no part of its design is fixed. Audience: the user and engineers reading README.md.
- Ambiguity: the user may instead want agent prompts to say this; I chose docs because reader/planner/verifier prompts have only 67-95 tokens of headroom. A prompt edit is out of scope unless the user asks.
- Existing exceptions (BRIEF.md edits only from Sri; records/ immutable; owner files) are real per BRIEF.md and docs/decisions.md D06; the text must keep them rather than say 'everything'.
- Existing code: none needed; docs only.

## Acceptance criteria

1. [content] README.md has a new short section (Grep for a heading containing 'change') stating that every part of Athena's design (code, rules, roles, prompts, docs) can be changed by a job.
2. [content] The section names the exceptions that actually exist, with a link to where each is stated: BRIEF.md (changes only from Sri) and records/ (immutable), checkable by Grep for 'BRIEF.md' and 'records/' in the new section, and the links resolve to existing files such as docs/decisions.md.
3. [content] The section says how a design change is made (a request through the normal change pipeline, with a decision recorded in docs/decisions.md), and does not claim any mechanism that does not exist; each claim cites a file.
4. [content] CHANGELOG.md has an entry for the README addition, in the file's existing format.
5. [content] Only README.md, CHANGELOG.md and optionally docs/architecture.md changed per git diff; no prompt, athena.toml, defaults.toml or workflow file changed (prompt token budgets are nearly full).
6. [behavior] Run cargo test and cargo clippy: both still pass, including any tests that check docs links or changelog format.

## Expected deliverables

- README.md (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing prompt files under the token budgets
- Changing athena.toml, defaults.toml, or workflows
- Changing BRIEF.md or records/
- Any code or rule change

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: Wrote the section for an outside reader with links.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each claim must cite a file.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused docs change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8b027a450249 → candidate 780ee6184620. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md |  3 +++
 README.md    | 15 +++++++++++++++
 2 files changed, 18 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-080749-p0fs/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-080749-p0fs/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42591970933 for 780ee61846209aaa6ffd31777615904cecd4b6b9)
- ci-result: `not a check` passed (Check results for 780ee61846209aaa6ffd31777615904cecd4b6b9 come from the CI run https://github.com/doodla/athena/actions/runs/42591970933 (692s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo test and cargo clippy: both still pass, including any tests that check docs links or changelog format.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
