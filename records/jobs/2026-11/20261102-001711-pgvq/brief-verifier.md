# Job 20261102-001711-pgvq: verify "Speed up the slowest test binary using newest green CI timings"

## Request (verbatim)

> Find the slowest tests from GitHub's CI timings and speed up the worst one.
> - Read per-test or per-binary times from the newest green CI run on main (the six test shards' logs list each binary's 'finished in' time); do not run the whole suite locally, which takes longer than the writer's time limit.
> - CHANGELOG.md lists the ten slowest test binaries with their times and the run they came from.
> - Change only the single slowest binary's tests, keeping what they cover, and show its time before and after with a local run of that one binary; if the cause cannot be fixed without losing coverage, say so and change no code.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

Read each test binary's "finished in" time from the six shard logs of the newest green main CI run, record the ten slowest in CHANGELOG.md with the run id, then speed up only the single slowest binary's tests without losing coverage, showing its local before/after time. If the cause cannot be fixed without losing coverage, change no code and say so. Audience: maintainers reading CHANGELOG.md.

Assumptions:
- Code survey: I read crates/athena-cli/tests/ (26 integration binaries, e.g. lifecycle.rs, live.rs, steward.rs), crates/athena-core/tests and crates/athena-data/tests listings, plus xtask/shard-timings.txt; that file holds only estimates (athena-cli --lib 40s and --test lifecycle 40s are its largest), so it is not the real CI data and the real slowest binary must come from CI logs.
- The slowest binary is unknown to me; it is likely one of crates/athena-cli/tests/*.rs or the athena-cli lib unit tests, decided by the CI logs.
- The producer can read CI logs via GitHub tooling (e.g. gh); the run id is recorded in CHANGELOG.md. Existing CHANGELOG.md has no ten-slowest list yet, so a new entry is added.
- Local before/after times come from running only that one binary (cargo test -p <pkg> --test <name>), reported in the CHANGELOG entry; the whole suite is not run.
- No governance files change; xtask/shard-timings.txt is not updated, since it is an estimates file maintained by a separate command.
- Alternative outcome: if no coverage-preserving speedup exists, only CHANGELOG.md changes and it says so.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry listing ten test binaries, slowest first, each with its 'finished in' seconds, and naming the GitHub run id (newest green main run) they came from.
2. [content] The CHANGELOG entry states the slowest binary's local time before and after the change, from running only that binary, or states that the cause cannot be fixed without losing coverage and no code changed.
3. [content] The diff touches test code of only the single slowest binary (plus CHANGELOG.md); no other test binary is modified.
4. [content] Every test function present in the changed binary before the change still exists afterwards with equivalent assertions (git diff shows no removed #[test] without a replacement and no weakened assertions).
5. [behavior] Run cargo test on only the changed binary (cargo test -p <package> --test <name>); it passes, and its reported 'finished in' time is below the 'before' time stated in CHANGELOG.md.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/live.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Running the whole test suite locally
- Changing any test binary other than the single slowest
- Updating xtask/shard-timings.txt or CI sharding
- Removing or weakening test coverage

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the slowest binary's measured time falls, not just that edits were made.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Times are cited from a named CI run id or a local run, not estimated.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the single slowest binary is changed.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier re-runs the one changed binary.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion checks the binary's timing and passing, not file existence.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry is written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 18b6a4f59e21 → candidate 4666d29b7885. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md | 6 ++++++
 1 file changed, 6 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41473513269 for 4666d29b78854273d4b6e828bfb58278614637d9)
- ci-result: `not a check` passed (Check results for 4666d29b78854273d4b6e828bfb58278614637d9 come from the CI run https://github.com/doodla/athena/actions/runs/41473513269 (434s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "CHANGELOG.md has a new entry listing ten test binaries, slowest first, each with its 'finished in' seconds, and naming the GitHub run id (newest green main run) they came from."; "The CHANGELOG entry states the slowest binary's local time before and after the change, from running only that binary, or states that the cause cannot be fixed without losing coverage and no code changed."; "Run cargo test on only the changed binary (cargo test -p <package> --test <name>); it passes, and its reported 'finished in' time is below the 'before' time stated in CHANGELOG.md.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
