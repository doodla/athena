# Job 20261029-155632-pqgc: verify "Show a real job's models, verdict and costs on the homepage"

## Request (verbatim)

> On the site homepage that xtask/src/site.rs generates, back up the header's two claims with something the reader can check. The claims are that one model does the work while a different one checks it, and that each step's cost is recorded. For one real landed job, show: which model produced the work, which model verified it, the verdict, and the recorded cost of each stage. Generate this from that job's records rather than writing it by hand, and link to the source record. Acceptance: (1) the rendered page shows, for one named job, the producer model, the verifier model, the verdict, and the per-stage costs. (2) A test fails when the page is rebuilt without that section. (3) No other homepage content changes, except where this section is placed. This is separate from the queued items about install steps and the homepage being mostly the firstmate study.
> 
> Acceptance:
> - The change is made as described: 11 newcomer ratings from 2026-10-27 to 2026-10-29 say the same thing: the homepage claims a second model checks the work and every step's cost is recorded, but shows neither
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: xtask/src/site.rs

## Reading (accepted by the user)

Add a section to the homepage that xtask/src/site.rs renders. For one named landed job it shows the producer model, the verifier model, the verdict and each stage's recorded cost, taken from that job's records with a link to the source record. The audience is a newcomer judging the header claims "a different model checks" and "each step's cost is recorded". Nothing else on the page changes, except where the section is placed.

Assumptions:
- Code survey: read xtask/src/site.rs. `build(root,out)` calls `render(&load(root/STUDY_DIR))` and writes index.html, then calls `crate::decisions::write(root,out)`. `render` builds the header with the two claims, then calls start_here, states, counters, transitions, rationales, glossary, findings, next_steps and claims_section in that order, then `finish`. Also read xtask/src/decisions.rs: `locate(root)` finds the records branch and gives `Records::Missing` when it is absent, and `read_snapshot(git, tip)` reads records from the branch tip. Also read crates/athena-data/src/records.rs: `Reading` has producer_model and verifier_model, and the verification record holds `final_verdict` and `agent_verdict`. A cost struct (line ~416) has `cost_usd: f64`, and `check_record_file` (line 1402) with `AnyRecord` (line 1076) validates record files by path.
- Code survey: the producer and verifier models and the per-stage costs are measured directly from the job's own records. They are parsed with the athena-data record types (AnyRecord via check_record_file), not from prose, and the page shows those values. The change adds no record or data file under crates/. Any fixture is a test-local temp file, so the allow-list and round-trip test need no new entries.
- Design: the section reads a named job's records on the records branch, in the way decisions::locate does. The producer model comes from the job's reading record. The verdict and per-stage costs come from its verification and cost records, and the source record link follows the existing github_base/source_url link style. The section is a new function called from `render` next to the header, so existing sections keep their text and order. The records branch is not in the reader copy, so the producer must pick a real landed job id from it. If the records are missing, the section must say so and not invent values, and the test must fail on that state.
- Environment: the section reads the local git records branch of the checkout at `root` through the existing decisions::locate path. It needs read access only and writes nothing outside the output directory. It reaches no network and does no push.
- The test that fails when the section is missing belongs in xtask/src/site.rs unit tests, because xtask has no library target. It builds the page from a fixture job in a temp dir and asserts the producer model, verifier model, verdict, per-stage costs and a source link all appear. A second test asserts that a page rebuilt without the section fails that check.
- Out of scope per the request: the queued install-steps item and the item on the homepage being mostly the firstmate study. athena.toml, defaults.toml and workflows are not changed.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`. A new test builds the homepage from a fixture job's records and asserts the rendered HTML contains that job's id, its producer model, a different verifier model, the verdict, and a recorded cost for each stage, with a link to the source record.
2. [behavior] A new test calls the same check on a page rendered with the section removed, or with the job's records missing, and asserts the check fails (the page does not pass). Run `cargo test -p xtask` to see it pass.
3. [content] `git diff` shows the section's values (models, verdict, costs) are computed from record data in site.rs code and not typed into string literals. The diff contains no hand-written model names or dollar figures outside test fixtures.
4. [content] `git diff` touches only xtask/src/site.rs (plus xtask files the section strictly needs). The header text, the existing sections' text and the CSS are unchanged, and the only change to `render` is one call that places the new section.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, including every pre-existing site.rs test.
6. [content] Outcome check: the producer model and verifier model in the rendered section differ from each other, so the page backs the claim that a different model checks the work. This is asserted in the test.

## Expected deliverables

- xtask/src/site.rs (modify)
- xtask/src/decisions.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Install-steps content on the homepage
- Reducing how much of the homepage is the firstmate study
- Changes to athena.toml, defaults.toml or workflows
- Changes to record formats or crates/

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered page and a failing test, not that the code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in site.rs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The section is generated from records, so the page cannot drift from the evidence.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8900f14d06b2 → candidate f8daefb7daaf. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md           |   5 +
 xtask/src/decisions.rs |  20 +++-
 xtask/src/site.rs      | 310 ++++++++++++++++++++++++++++++++++++++++++++++++-
 3 files changed, 328 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-155632-pqgc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-155632-pqgc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42861581062 for f8daefb7daafc9c676e2c28d7d03a3ce84814713)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42861581062 for f8daefb7daafc9c676e2c28d7d03a3ce84814713)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42861581062 for f8daefb7daafc9c676e2c28d7d03a3ce84814713)
- ci-result: `not a check` passed (Check results for f8daefb7daafc9c676e2c28d7d03a3ce84814713 come from the CI run https://github.com/doodla/athena/actions/runs/42861581062 (606s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
