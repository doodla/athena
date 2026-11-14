# Job 20261114-041134-qwtn: verify "Add self-queued fix backlog line to athena status"

## Request (verbatim)

> Requested by Sri (the owner). Add a line to athena status that counts fixes Athena queued for itself (steward and Hestia requests) and how many of them have run, over the last 7 days and in total, so a stuck backlog of fixes is visible.

## Reading (accepted by the user)

Add one line to `athena status` counting requests Athena queued for itself (steward channel, which Hestia requests also use) and how many have run, for the last 7 days and in total. The owner uses it to spot a stuck backlog.

Assumptions:
- Code survey: crates/athena-core/src/status.rs (StatusView, Athena::status at ~1187, scan_jobs, state_name, text rendering) and crates/athena-core/src/steward.rs (REQUEST_CHANNEL = "steward" at line 335, DRAINED_CHANNELS, open-request helper near line 373) and crates/athena-data/src/records.rs (Request has job, text, received_at, channel). The count is measured directly from Request.channel and received_at, and from each job's state.
- Self-queued means a Request whose channel is REQUEST_CHANNEL ("steward"). I did not find a separate Hestia channel, so I assume Hestia requests use the same channel. The producer must confirm this in crates/athena-core/src/hestia*; if Hestia uses another channel, include it.
- Has run means the job has left the queued state: it is running, or it reached a terminal outcome such as landed, rejected or closed. Queued-but-untouched jobs are the backlog. Terminal state is judged with the existing state_name and is_terminal helpers.
- Last 7 days is measured by Request.received_at against the current time already used by status. Status stays read-only, and no new record type is added, so records validation needs no change.
- Field meaning: Request.channel is a free string. Its documented use is routing the drain (steward.rs REQUEST_CHANNEL and DRAINED_CHANNELS), and it already marks steward-queued requests, so it is reused without change.
- Goal and audience: the owner reads athena status to see whether self-queued fixes are piling up.
- Standing defaults apply: tests pass, clippy is clean, and a CHANGELOG.md entry is added.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/status.rs builds records with steward-channel and cli-channel requests, some inside and some outside 7 days, some run and some still queued. The test asserts the rendered line shows the right counts for the last 7 days and in total, and that cli-channel requests are not counted. Verify by running `cargo test -p athena-core status`.
2. [behavior] With no steward-queued requests, the line still prints with zeros (for example 0 queued, 0 run), so an empty backlog shows as an explicit count and is not hidden. A unit test covers this and is run with `cargo test -p athena-core status`.
3. [content] The diff to crates/athena-core/src/status.rs adds a line to the `athena status` text output. It reads like `self-queued fixes  last 7d: N queued, M run · total: N queued, M run` and reports both the queued and run counts for both windows. It counts only requests whose channel is steward::REQUEST_CHANNEL, not a duplicated literal.
4. [behavior] `cargo test`, `cargo clippy` and `cargo fmt --check` pass. Existing status tests and the xtask spec tests that check status labels still pass, with the Hestia spec_gaps label check updated if it applies.
5. [content] CHANGELOG.md has an entry recording the new status line.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/steward.rs (modify)
- docs/design/hestia.md (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Alerting or automatic action on a stuck backlog
- Changing how the drain queues or runs steward requests
- Adding new record types or fields

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and cargo test, clippy and fmt criteria
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered counts, not that wiring exists
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A zero-backlog case prints explicit zeros
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3741fed4edd6 → candidate 9adc5cf37f41. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   4 +
 crates/athena-cli/src/display.rs |   1 +
 crates/athena-core/src/status.rs | 163 +++++++++++++++++++++++++++++++++++++++
 3 files changed, 168 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-041134-qwtn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-041134-qwtn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43583069040 for 9adc5cf37f419076c319f39fa7f4d3e64a1a3da4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43583069040 for 9adc5cf37f419076c319f39fa7f4d3e64a1a3da4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43583069040 for 9adc5cf37f419076c319f39fa7f4d3e64a1a3da4)
- ci-result: `not a check` passed (Check results for 9adc5cf37f419076c319f39fa7f4d3e64a1a3da4 come from the CI run https://github.com/doodla/athena/actions/runs/43583069040 (690s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy` and `cargo fmt --check` pass. Existing status tests and the xtask spec tests that check status labels still pass, with the Hestia spec_gaps label check updated if it applies.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
