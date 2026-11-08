# Job 20261027-011638-zsz9: verify "Add real-call-site test convention to the change producer brief"

## Request (verbatim)

> Add a producer-brief convention: each behavior change needs at least one test that goes through the real call site (the existing fake agent driver or nested-run harness where needed) and would fail if the production call were deleted. The producer must state which test that is. Acceptance: the producer brief contains the convention and a policy test checks it is present.
> 
> Acceptance:
> - The change is made as described: Producer's new tests exercise helpers, not the changed call site; verifier finds every test would still pass if the production change were reverted
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/jobs/2026-10/20261026-170042-cgq9/verification.json

## Reading (accepted by the user)

Add a short convention to the change producer brief: every behavior change needs at least one test through the real call site (existing fake agent driver or nested-run harness) that fails if the production call is deleted, and the producer names that test in its report. A policy test in athena-cap-change checks the brief contains it. Job 20261026-170042-cgq9 (a records-branch reader change) is dropped, not continued.

Assumptions:
- Goal and audience: the change producer agent reads the brief; the verifier and maintainers judge whether the convention is present and enforced by a test.
- Code survey: read crates/athena-cap-change/src/lib.rs (producer prompt is include_str!("../prompts/producer.md") at line 21, exposed via producer_section; two existing unit tests parse athena.toml and defaults.toml) and the list of files referencing producer.md (crates/athena-context/src/briefs.rs, crates/athena-context/src/lib.rs). The new policy test goes in crates/athena-cap-change/src/lib.rs tests and asserts on the producer prompt text. No record or data file is added, so no records allow-list or AnyRecord round-trip is affected. The property is measured directly on the embedded prompt text.
- Job 20261026-170042-cgq9 (retry source, rejected, verdict fail): its findings concern reader tests that only called helpers (records_branch_readers.rs) and a decisions note about reads versus writes. They no longer apply because that records-branch reader work is dropped entirely; this job only supplies the convention that would have caught those findings. All 18 files its candidate changed are excluded in out_of_scope.
- The request's 'Files:' entry names records/jobs/.../verification.json; records/ never changes, so it is read context only.
- Token headroom: producer (change) has 146 tokens left, so the addition is kept to about 60 tokens in crates/athena-cap-change/prompts/producer.md, the only target whose sets have room (it is not composed into the verifier or reader).
- No governance files (athena.toml, defaults.toml, workflows) need to change.

## Acceptance criteria

1. [content] Grep of crates/athena-cap-change/prompts/producer.md finds text requiring, for each behavior change, at least one test through the real call site (naming the fake agent driver or nested-run harness) that would fail if the production call were deleted.
2. [content] Grep of crates/athena-cap-change/prompts/producer.md finds an instruction that the producer must name which test that is in its report.
3. [content] A test in crates/athena-cap-change/src/lib.rs asserts the embedded producer prompt contains the convention's key phrases (real call site, would fail if the production call were deleted, name the test); Grep finds it and it asserts on the producer prompt text, not a copy of the string.
4. [behavior] Run `cargo test -p athena-cap-change`: the new policy test passes, and its assertions would fail if the convention paragraph were removed from producer.md (checkable by reading the diff's assertions against the paragraph).
5. [behavior] Run `cargo test`: the whole suite passes, including any existing token-budget or prompt tests, so the producer (change) prompt stays within its 1300-token budget.
6. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets`: both are clean.
7. [content] `git diff --stat` shows changes only to crates/athena-cap-change/prompts/producer.md and crates/athena-cap-change/src/lib.rs (plus CHANGELOG.md if the repository convention requires it); no unrelated behavior changes.

## Expected deliverables

- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Everything job 20261026-170042-cgq9's candidate changed is dropped, not kept: CHANGELOG.md records-branch entry, crates/athena-cli/src/backtest.rs, crates/athena-cli/tests/records_branch_readers.rs, crates/athena-core/src/{auditor,gh,introspect,manage,proposals,reading,run,status,store}.rs, crates/athena-data/src/{lib,paths}.rs, crates/athena-gate/src/records.rs, docs/decisions.md D134, xtask/src/main.rs, xtask/src/walltime.rs.
- Reworking records-branch readers or writers, or the cutover.
- Changing the study producer or verifier prompts, or any checker that enforces the convention on future diffs.
- Editing anything under records/.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A policy test is added and cargo test, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the convention text and the test's failure when the text is removed, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt paragraph plus one test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The convention is enforced by a policy test on the prompt text.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a producer brief lacking the convention fails the test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 026e7d45f95b → candidate 930194aeac50. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |  3 +++
 crates/athena-cap-change/prompts/producer.md |  3 ++-
 crates/athena-cap-change/src/lib.rs          | 16 ++++++++++++++++
 docs/decisions.md                            | 17 +++++++++++++++++
 4 files changed, 38 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-011638-zsz9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-011638-zsz9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42476508922 for 930194aeac5045c5707f805832d637a07574283f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42476508922 for 930194aeac5045c5707f805832d637a07574283f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42476508922 for 930194aeac5045c5707f805832d637a07574283f)
- ci-result: `not a check` passed (Check results for 930194aeac5045c5707f805832d637a07574283f come from the CI run https://github.com/doodla/athena/actions/runs/42476508922 (769s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`: the whole suite passes, including any existing token-budget or prompt tests, so the producer (change) prompt stays within its 1300-token budget.; Run `cargo fmt --check` and `cargo clippy --all-targets`: both are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
