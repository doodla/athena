# Job 20261110-163344-3gaa: verify "Print recorded deliverable counts in the reader brief for carried jobs"

## Request (verbatim)

> try again on 20261110-151108-70nx

## Reading (accepted by the user)

Retry of job 20261110-151108-70nx. Its stated cause (ids cited only by introspection were never resolved) was contradicted by the records: 7zqw named its jobs in its own request, the base brief already showed their deliverables, and the reader still wrote 12 for a list of 11. The narrowed fix gives the reader the count the record holds, so it copies the number and does not count.

Assumptions:
- Job 20261110-151108-70nx is addressed by retraction, not by repair: finding 'root cause contradicted by records' (major) and 'reading does not match request' (major) are fixed by dropping the introspection-id resolution premise, which the records disprove (7zqw was not a retry; base referenced_jobs and prior_result_section already carry request-named ids and print deliverables). The finding about deliverables printed twice no longer applies: no new list is printed, only a count is added to existing lines. The retry.rs finding no longer applies: that file's work is dropped. The nit about compile-error fails-without no longer applies: no signature change.
- Root cause stated for this job: the reader is shown cited jobs' deliverables as a bare list and must count it, so it wrote 'zaj2 (12 deliverables)' where the record has 11 (evidence: the verification of earlier attempt 20261110-124254-zegf, as quoted in the 70nx findings). The information the stage lacked is the recorded count; give it in the brief instead of adding any refusal rule.
- Code survey: read crates/athena-context/src/briefs/reader.rs (prior_result_section at line 135, lines 160-189 print 'Job N: end state ...' then 'Allowed change scope, deliverables:' as a quoted list; referenced_jobs_section at line 436) and crates/athena-core/tests/refusal_corpus.rs plus the list of files mentioning replay (crates/athena-core/src/reading/replay.rs exists already). Entry point is the reader brief builder in briefs.rs; no records validation allow-list or data file under crates/ is added, so check_record_file and AnyRecord are untouched.
- Kept from the 70nx candidate: nothing is kept. Its seven changed files (CHANGELOG.md, crates/athena-cli/tests/retry.rs, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/topics.rs, crates/athena-core/tests/refusal_corpus.rs) are re-based; of these only CHANGELOG.md, briefs.rs and briefs/reader.rs are deliverables again, with new content. reading.rs, topics.rs, retry.rs and refusal_corpus.rs are excluded (dropped).
- The audience is the reader agent (which must quote counts correctly) and the verifier who checks a rendered brief.
- No new refusal rule is added, so no newly-refused share applies. The reader prompt set has 2 tokens of headroom, so no prompt file changes.
- Jobs 7zqw, fzx0 and bn3c are not claimed as instances of this gap; only the count-miscopy seen in 7zqw's reader is.

## Acceptance criteria

1. [behavior] A new test in crates/athena-context/src/briefs.rs builds a reader brief for a follow-up whose carried job records 11 deliverables and asserts the text contains the count 11 next to 'Allowed change scope, deliverables' for that job, and a second job with 3 deliverables shows 3. Run `cargo test -p athena-context brief`; it fails on base, where no count is printed.
2. [behavior] The count is computed from reading_deliverables.len() for every job in prior_result_section and in referenced_jobs_section, not typed per job: a test with differing lengths (0, 1, 11) shows each count, and 0 still prints the existing 'none recorded' line. Run `cargo test -p athena-context`.
3. [content] Grep of the diff shows only crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/briefs.rs and CHANGELOG.md changed; crates/athena-core/src/reading.rs, reading/topics.rs, reading/checks.rs, crates/athena-cli/tests/retry.rs, refusal_corpus.rs and all prompt files are unchanged, and no new step! or check function appears in refusing_check.
4. [content] CHANGELOG.md Unreleased has an entry that names the cause (reader had to count a bare deliverable list and miscounted, e.g. 12 for 11 in job 7zqw) and says the fix prints the recorded count; it does not list fzx0 or bn3c as instances. Check with Grep.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, and existing tests such as a_follow_up_brief_carries_the_result_and_scope_of_each_named_job still pass.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Resolving ids cited only by a retry's introspection (dropped from the 70nx candidate): crates/athena-core/src/reading/topics.rs, crates/athena-core/src/reading.rs, crates/athena-cli/tests/retry.rs, crates/athena-core/tests/refusal_corpus.rs are not changed.
- Any new refusal rule, replay harness, size check or bullet-coverage check; the dropped candidates of 7zqw, fzx0 and bn3c stay discarded.
- Reader prompts, athena.toml, defaults.toml, .github/workflows.
- Verifying root-cause claims before dispatch (the proposed next request from introspection jskc); a separate request.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered brief shows the correct count, not that code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the brief text.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b5b8b45d3400 → candidate 84679e321087. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  7 ++++++
 crates/athena-context/src/briefs.rs        | 39 ++++++++++++++++++++++++++++++
 crates/athena-context/src/briefs/reader.rs |  6 ++++-
 3 files changed, 51 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-163344-3gaa/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-163344-3gaa/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42856396651 for 84679e3210878b419082b5a758b353eda86aff8b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42856396651 for 84679e3210878b419082b5a758b353eda86aff8b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42856396651 for 84679e3210878b419082b5a758b353eda86aff8b)
- ci-result: `not a check` passed (Check results for 84679e3210878b419082b5a758b353eda86aff8b come from the CI run https://github.com/doodla/athena/actions/runs/42856396651 (826s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, and existing tests such as a_follow_up_brief_carries_the_result_and_scope_of_each_named_job still pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
