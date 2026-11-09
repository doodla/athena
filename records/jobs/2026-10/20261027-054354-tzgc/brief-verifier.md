# Job 20261027-054354-tzgc: verify "Skip Priority lines when taking a request's title"

## Request (verbatim)

> Request titles in athena plan and athena status take the request's first line, so every request that starts with a 'Priority:' line shows the title 'Priority: high' or 'Priority: urgent'. Take the title from the first line that is not a Priority line.
> Acceptance:
> - A test shows a request whose first line is 'Priority: urgent' gets its title from the next non-empty line in plan and status output.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/plan.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

A request's title comes from its first line even when that is 'Priority: urgent'. The title will come from the first non-empty line that is not a Priority line, so plan and status show a useful title. The fix goes in one shared helper in plan.rs, used by the title code in store.rs that both outputs call.

Assumptions:
- Goal/audience: operators reading `athena plan` and `athena status` need distinct request titles instead of 'Priority: high'.
- Code survey: crates/athena-core/src/store.rs `title()` (line ~150) falls back to `req.text.lines().next()` when no reading exists; status.rs:711 calls job.title(); reading/topics.rs:374 builds the PlanEntry title from job.title(). plan.rs `Priority::from_text` (line ~97) already detects Priority lines (trimmed, any case, `Priority:` prefix). reading.rs:1360 also truncates the first line, which is probably the subject for the reading fallback. No record or data file is added, so check_record_file and AnyRecord are not touched.
- The shared helper lives in plan.rs (e.g. `pub fn title_line(text: &str) -> &str`), and store.rs title() calls it. store.rs is therefore edited even though the request's file list omits it. A reading-derived title is unchanged.
- If every line is blank or a Priority line, the title falls back to the first line, as before.
- Existing code: crates/athena-core/src/plan.rs Priority::from_text and crates/athena-core/src/store.rs title() are reused and extended.
- Consumers: none found (no config deliverable).

## Acceptance criteria

1. [behavior] A new unit test, run with `cargo test -p athena-core`, builds a request whose text starts with 'Priority: urgent', then a blank line, then 'Fix the widget'. It asserts the title is 'Fix the widget' in both the plan entry (topics/plan path) and the status output (status.rs).
2. [behavior] Tests cover a lowercase/indented 'priority: high' line, a request with no Priority line (the title is still its first line), and a request made only of Priority lines (falls back to the first line). `cargo test` passes.
3. [behavior] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
4. [content] The diff to crates/athena-core/src/plan.rs adds the shared title helper, and the diff to crates/athena-core/src/status.rs adds or updates a status test; the title logic in store.rs uses the helper instead of `lines().next()`.
5. [content] CHANGELOG.md has a new entry under the unreleased section describing the title fix.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Changing how priority itself is parsed or ranked
- Changes to titles that come from a reading
- Governance files such as athena.toml and defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests; cargo test, fmt and clippy -D warnings pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the shared title source in store.rs so plan and status both change, not just the two outputs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the tests on the resulting titles.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One helper plus one call site and tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 39bef27a84a1 → candidate 406e3f13bf60. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  7 +++++++
 crates/athena-core/src/plan.rs    | 37 +++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs | 19 +++++++++++++++++++
 crates/athena-core/src/status.rs  | 21 +++++++++++++++++++++
 crates/athena-core/src/store.rs   |  5 +++--
 5 files changed, 87 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054354-tzgc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054354-tzgc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42537184620 for 406e3f13bf60f748344f3dedf18946c532a1ae28)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42537184620 for 406e3f13bf60f748344f3dedf18946c532a1ae28)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42537184620 for 406e3f13bf60f748344f3dedf18946c532a1ae28)
- ci-result: `not a check` passed (Check results for 406e3f13bf60f748344f3dedf18946c532a1ae28 come from the CI run https://github.com/doodla/athena/actions/runs/42537184620 (543s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
