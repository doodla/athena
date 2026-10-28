# Job 20261027-160842-avd0: verify "Refuse readings whose criteria rest on the producer's report"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Criteria are written without a defined list of what the verifier can observe (the diff, the repo sna
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: criterion-requires-evidence-verifier-cannot-observe (20261027-011728-xrka)
> Cause: Criteria are written without a defined list of what the verifier can observe (the diff, the repo snapshot, check output). Nothing compares a criterion's required evidence with that list. A criterion that only the producer's report could sat...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261027-011714-9qse (job 20261027-004517-gk9e); prevention it proposed: Add a check at reading or request acceptance that scans each criterion for evidence sources the verifier lacks, such as "producer's report", "the report lists" or "the producer states". The check flags the criterion unless it is restated as...
> Introspection 20261027-011812-anp5 (job 20261027-011733-vrsk); prevention it proposed: Add a reading-acceptance check. Each criterion must name its evidence as a command or Grep the verifier can run, or a file in the snapshot. A criterion whose evidence is the producer's report, summary or claims is refused. The reading and c...
> Introspection 20261027-013024-v1gj (job 20261027-012938-yckz); prevention it proposed: Add a reading-acceptance check. Every criterion must name an evidence source the verifier can observe: a Grep or command it runs, or a file in the snapshot. A criterion that cites the producer's report, or "the producer confirms", is reject...
> Introspection 20261027-023348-vrd5 (job 20261026-232402-6qme); prevention it proposed: Add two things. (1) A rule in the reader brief: every criterion must name evidence the verifier can observe itself, the producer's report is never evidence, and for code-move jobs the standard criteria are a Grep for old module paths outsid...
> Introspection 20261027-030613-vk78 (job 20261026-232423-hvyb); prevention it proposed: Two parts. (1) Add a rule to the reading brief that every criterion must name evidence the verifier can observe itself (the snapshot, commands it runs, its own tools), and that the producer's report is never evidence. (2) Add a code check o...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: nothing compares a criterion's required evidence with what the verifier can observe, so criteria like "the report lists the Grep" (jobs gk9e, vrsk, yckz and two others) are accepted and then cannot be judged. Add a mechanism check in crates/athena-core/src/reading.rs, run next to check_prior_result_and_scope when a reading is built, that refuses any criterion citing the producer's report, summary, claims or statement as its evidence. The error names the criterion and says to restate it as a command or Grep the verifier can run, or a file in the snapshot. Audience: Athena's operator and the verifier.

Assumptions:
- The cause originates at reading acceptance, where criteria are first fixed; the code check is the mechanism, matching what all five introspections proposed.
- The reader prompt has 3 tokens of headroom, so no prompt text is added; the refusal message teaches the reader on retry.
- The refusal is reported the same way as check_prior_result_and_scope (a reading-refused message), applied to every reading with a capability, not only retries.
- The named jobs 20261027-004517-gk9e and 20261027-011733-vrsk are cited as evidence of the failure class, not retried here; their earlier findings (unobservable report criterion) are the class this change prevents, and the gh transport move itself already landed in 20261027-012938-yckz.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-core reading`: a new test shows a reading with the criterion "the producer's report lists the caller-check Grep" is refused with an error naming that criterion, and the same test fails when the new check is removed.
2. [behavior] Run `cargo test --locked -p athena-core reading`: a test covers each phrase family ("the report lists", "the producer states", "the producer confirms", "producer's summary", "producer's claims") and each is refused.
3. [behavior] Run `cargo test --locked -p athena-core reading`: a test shows criteria naming a command, a Grep, or a file path (e.g. "a Grep of crates/ for `gh::` outside gh/ returns no hits", "cargo test --locked passes") are accepted, so ordinary readings are not refused.
4. [behavior] Run `cargo test --locked -p athena-core reading`: a test shows the check is wired into the reading path (the reader's output becomes a refusal message, not a Reading) for a first attempt as well as a retry, not only in the check function.
5. [content] The doc comment on the new check and the CHANGELOG.md entry name the root cause: criteria were written without a defined list of what the verifier can observe, and nothing compared required evidence with that list; jobs 20261027-004517-gk9e, 20261027-011733-vrsk, 20261027-012938-yckz, 20261026-232402-6qme and 20261026-232423-hvyb are cited.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Editing prompt files (reader has 3 tokens of headroom)
- Changing the verifier's handover or giving the verifier the producer's report
- Rewriting criteria of existing jobs
- Changes to athena.toml, defaults.toml or workflows
- Reverting or redoing the gh transport move

## User defaults that apply

- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a code check, not prose in a prompt.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at reading acceptance, covering every criterion, not one job's wording.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show refusal and acceptance behavior.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Check is written in Rust in athena-core.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check plus tests and a changelog entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base b73571aaed30 → candidate 5162d4a7c77c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  12 ++
 crates/athena-context/src/briefs.rs     |  13 +-
 crates/athena-core/src/reading.rs       | 253 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/run_pin_tests.rs | 113 ++++++++++++++
 4 files changed, 387 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1495 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
