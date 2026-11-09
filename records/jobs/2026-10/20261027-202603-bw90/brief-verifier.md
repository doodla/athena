# Job 20261027-202603-bw90: verify "Require claim-support check for study-text edits in producer brief"

## Request (verbatim)

> Add a convention for jobs that edit study text (studies/*/model.json, intro.json and similar): for every new or reworded statement, the producer must open each claim id it cites, check that the claim's text states what the sentence says (including who does the action), and list each changed statement with its supporting claim text in the change description. If no claim supports the statement, the producer must add a sourced claim to claims.json or drop the statement. The changelog must not say claims support a wording unless they do. Put this in the producer brief that is used when a job's files are under studies/.
> 
> Acceptance:
> - The change is made as described: Study-page edits add or reword statements that their cited claims do not support; verifier finds it on 5 jobs
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: studies/*/model.json, intro.json, claims.json, studies/.

## Reading (accepted by the user)

Add a convention to the change producer prompt: when a job's files are under studies/, for every new or reworded statement the producer opens each cited claim id, checks the claim text supports the sentence (including who acts), and lists each changed statement with its claim text in the change description; unsupported statements get a sourced claim in claims.json or are dropped; the changelog never says claims support wording unless they do. Audience: producers and verifiers of study-page jobs.

Assumptions:
- Goal: producers editing study text stop adding statements their cited claims do not support; audience is the change producer reading its brief and the verifier checking study-page jobs.
- The 'producer brief used when files are under studies/' is the change capability producer prompt (crates/athena-cap-change/prompts/producer.md), as that is where the existing rendered-page convention is carried; no path-conditional brief mechanism exists, so the sentence is written conditionally ('when your files are under studies/').
- Study-capability producers (crates/athena-cap-study/prompts/producer.md) are left alone; it has 182 tokens of headroom and the request concerns edit jobs.
- Consumers: none found (no config deliverable).
- Not a refusal rule, so no replay percentage applies; the convention is prose-only, marked as such, with verification by the verifier reading the cited claims.
- Environment: none beyond the repository copy.

## Acceptance criteria

1. [content] crates/athena-cap-change/prompts/producer.md contains a sentence, conditioned on the job's files being under studies/, that requires the producer to open each cited claim id, check the claim text states what the sentence says including who does the action, and list each changed statement with its supporting claim text in the change description (checked by Grep/Read of the file).
2. [content] The same prompt says that a statement with no supporting claim must get a sourced claim added to claims.json or be dropped, and that the changelog must not say claims support a wording unless they do.
3. [content] The prompt text fits the producer (change) token budget (889 of 1200 used); the repository's prompt budget tests pass with the added text.
4. [content] The convention is written for a stranger in docs/conventions/study-claim-support.md (rule, why, enforcing mechanism or marked prose-only), and the prompt points to it.
5. [content] No files under studies/ and no Rust source other than test expectations tied to prompt text are changed; CHANGELOG.md has an entry describing the convention.
6. [behavior] cargo fmt --check, cargo clippy and cargo test all pass, including prompt structure/mirror checks over the edited prompt and docs.

## Expected deliverables

- crates/athena-cap-change/prompts/producer.md (modify)
- docs/conventions/study-claim-support.md (create)
- CHANGELOG.md (modify)
- docs/spec.toml (modify)

## Out of scope

- Editing any studies/*/model.json, intro.json or claims.json content
- Adding a code check that validates claim support
- Changing the study capability prompts or the verifier prompts

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the prompt text and passing checks, not activity.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Convention marked prose-only in the docs file.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt sentence plus a short convention doc and changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4e32fb95f4cb → candidate 45986a4afe68. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |  4 ++++
 crates/athena-cap-change/prompts/producer.md |  2 +-
 docs/conventions/study-claim-support.md      | 25 +++++++++++++++++++++++++
 docs/spec.toml                               |  2 +-
 docs/system-facts.md                         |  2 +-
 5 files changed, 32 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-202603-bw90/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-202603-bw90/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42599423502 for 45986a4afe68c9d5e31fac7b819db721d50c3fc8)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42599423502 for 45986a4afe68c9d5e31fac7b819db721d50c3fc8)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42599423502 for 45986a4afe68c9d5e31fac7b819db721d50c3fc8)
- ci-result: `not a check` passed (Check results for 45986a4afe68c9d5e31fac7b819db721d50c3fc8 come from the CI run https://github.com/doodla/athena/actions/runs/42599423502 (788s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy and cargo test all pass, including prompt structure/mirror checks over the edited prompt and docs.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
