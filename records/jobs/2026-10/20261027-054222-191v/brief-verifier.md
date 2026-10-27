# Job 20261027-054222-191v: verify "Show readers the token headroom of every prompt set in their brief"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has le
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-edits-prompt-without-checking-token-budget (20261024-002642-bm4c)
> Cause: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has left. The shared verifier prompt is composed into several sets, so an addition there costs every set that includes it, and one set had 4 token...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261024-002725-eqs1 (job 20261024-002644-1as8); prevention it proposed: Add a pre-dispatch check for any job whose reading touches prompt files. It computes the token headroom of every prompt set that composes each target file, the same figure the budgets test prints, and adds it to the reading. If the planned ...
> Introspection 20261024-104248-8p9c (job 20261024-093833-vava); prevention it proposed: Before a reading is accepted, run a check that computes the remaining token room for every composed prompt (producer per capability, verifier, and so on) and puts the table into the reading stage's context. Add a check that any reading or p...
> Introspection 20261025-090447-2zm3 (job 20261025-034452-wqp8); prevention it proposed: Add a check at reading admission. It takes each prompt file the reading or plan says it will grow, and compares the planned growth with the remaining budget the budgets test computes. It rejects the reading, naming the file and the room lef...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (failure class reading-edits-prompt-without-checking-token-budget, seen in jobs 20261024-002644-1as8, 20261024-093833-vava and 20261025-034452-wqp8): the reader's brief never says how many tokens each composed prompt set has left. The shared verifier.md, reader.md and producer.md therefore get planned for growth that overruns a set's budget. The fix is to compute the headroom per set in code and put it in the reader brief, with the files each set composes, so the reading picks a target file that fits and lists no overrun.

Assumptions:
- Goal: readings stop planning prompt edits that overflow a budget. The audience is the reader agent, and the maintainers who judge its readings.
- Job 20261024-002644-1as8 (landed, pass, 0 findings): it kept its change inside crates/athena-cap-change/prompts/verifier.md because the shared verifier prompt was full. Nothing is left to fix; this change makes that headroom visible up front. Its scope stays untouched.
- Job 20261024-093833-vava (landed, 2 findings: run_pin_tests.rs changed outside the listed deliverables; a nit on D116 wording). Neither finding applies here, since this change touches neither file. The same outcome-check applies: its out-of-scope items 'adding to the change verifier prompt (at 1298 of 1300 tokens)' and 'growing the shared producer.md beyond its budget' stay out of scope, and this change grows no prompt.
- Job 20261025-034452-wqp8 (rejected, 5 findings: D122 inserted inside D121; no test for urgent_reason parse path; unlisted files changed). Its decisions.md finding applies to this change: the new decision is appended as a fresh heading at the end and no existing decision is edited. The other findings concern the urgent priority feature, which is not redone here. Its out-of-scope items 'growing reader.md beyond the token budget' and 'changing athena.toml or defaults.toml' stay out of scope.
- The headroom is computed from the same compose and check_budget logic that crates/athena-cli/tests/budgets.rs uses, so the figures match what that test prints. Sets: contract, reader, planner, critic, and producer and verifier per enabled capability. Each row names its component ids (for example frame.verifier, capability.change.verifier) so the reader can see which sets a shared file feeds.
- The figures go in the reader's user brief (ReaderInputs), not in reader.md, which is near its own budget. The reader.md wording is left unchanged, and the brief heading says plainly that an edit to a listed file costs every set that includes it.
- Pre-dispatch rejection of over-budget readings, as two of the introspections suggested, is not built here. The reader is shown the numbers and the existing budgets test is the backstop, which keeps the change small.

## Acceptance criteria

1. [content] The reader brief has a 'Prompt token headroom' section listing, for each budgeted set (contract, reader, planner, critic, and producer and verifier per enabled capability), the component ids it composes, the estimated tokens used, the budget and the tokens left. It says an edit to a shared file costs every set that includes it.
2. [content] The headroom is computed in code by one function in crates/athena-context/src/lib.rs, using compose and the budget values already there. reading.rs passes the rows into ReaderInputs, and no budget number is hard-coded in the brief.
3. [behavior] Run `cargo test -p athena-context`. A new briefs test builds a ReaderInputs whose rows show a set with 2 tokens left, and asserts the brief names that set, its component ids and '2'. Another asserts the section is absent when no rows are given. Both tests fail without the change.
4. [behavior] Run `cargo test -p athena-cli --test budgets`. A new test asserts the computed headroom rows equal the figures behind computed_rows(), so the brief cannot drift from the budgets test. It also asserts a shared-file row, such as the verifier frame, appears in every verifier set that composes it.
5. [behavior] Run `cargo test -p athena-core reading`. A test shows the reader invocation's user prompt (via the fake driver) contains the headroom section with the real verifier and reader figures.
6. [content] CHANGELOG.md has an entry. docs/decisions.md gets one new decision appended as its own heading, naming the root cause, citing the three introspections and the alternatives (pre-dispatch rejection, a note in reader.md), without editing any existing decision.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, including the system-facts budgets test, which shows no prompt file was grown.

## Expected deliverables

- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Any change to the job 20261024-002644-1as8 deliverables: crates/athena-cap-change/prompts/verifier.md and the shared verifier prompt
- Growing or editing any prompt file, including reader.md, producer.md and verifier.md, and the change verifier at 1298 of 1300 tokens
- Rejecting readings automatically at admission for exceeding a budget
- Changing budget values or the token estimator
- Re-doing the urgent-priority feature of job 20261025-034452-wqp8
- Editing existing entries in docs/decisions.md, records/, athena.toml or defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in athena-context, athena-cli budgets and athena-core reading; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the source: the reader's brief lacks the headroom data, so the data is added there for the whole class of prompt edits.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered brief and the reader's actual prompt, not that types exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is a reader that sees headroom per set. Criteria check that and not the activity.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change. Pre-dispatch rejection is left out.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test ties the shown figures to the budgets computation, so they cannot drift.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the decision record are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the named cargo commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 8a012199572a → candidate 8a2e605b1cb1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   6 ++
 crates/athena-cli/tests/budgets.rs      |  53 ++++++++++++++++
 crates/athena-context/src/briefs.rs     |  45 +++++++++++++
 crates/athena-context/src/lib.rs        | 109 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs       |  27 ++++++++
 crates/athena-core/src/run_pin_tests.rs |  67 ++++++++++++++++++++
 docs/decisions.md                       |  27 ++++++++
 7 files changed, 334 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1465 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, including the system-facts budgets test, which shows no prompt file was grown.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
