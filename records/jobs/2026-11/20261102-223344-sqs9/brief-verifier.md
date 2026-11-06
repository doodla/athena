# Job 20261102-223344-sqs9: verify "Check seeded ontology terms against every governing source, not a listed"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> Introspection 20261102-102715-564y (job 20261031-172040-kq9v); prevention it proposed: Add a code check, run in the repo tests and before verification, that computes the athena binary's real input set (every file the build compiles in, including everything xtask input_files should list) and fails if the hashFiles globs in eve...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a seeded data file is checked for shape and for the sources the author chose to list, so a sense of a term that lives in another governing source goes unseen until the final verdict. The fix is in crates/athena-cli/tests/ontology.rs, where the claim check should scan every governing doc for each term. This also covers the 0wtc, y179 and kq9v retries; the audience is the verifier and the next producer.

Assumptions:
- Code survey: crates/athena-cli/tests/ontology.rs. scanned_files() scans only BRIEF.md plus the sources the term itself lists, so a sense found in another governing doc (README.md, docs/*.md) escapes the check unless the author listed that doc. The file also holds claim_problems and the excuse mechanism. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly: each term name is searched in the governing files, and the test fails when a hit is neither a listed sense nor an excused hit.
- Job 20261017-173508-0wtc: its Hermes (tool vs Olympus role) finding is already fixed in the repo (docs/ontology.toml has senses and the test has claim checks). The remaining gap is the scan scope, which this change closes. Its files are kept: CHANGELOG.md, docs/decisions.md, docs/ontology.toml and crates/athena-cli/tests/ontology.rs; the last is modified, the others only if a newly found hit needs a sense or an excuse.
- Job 20261025-214509-y179: a speed-target job whose goal was missed. It is a different instance of the same cause, a claim never measured before handover. Its code changes (secrets.rs memo, checks.rs poll schedule, D126, CHANGELOG) are kept untouched. The speed work itself is not redone here, and the 7-minute goal is a separate request.
- Job 20261031-172040-kq9v: the stale-binary finding is already addressed in the repo. xtask BUILD_INPUT_GLOBS is the single list, xtask/tests/policy.rs requires each athena-bin key in ci.yml to match it, and ci.yml and xtask/src/main.rs are kept as they are. The remaining minor findings (stamp on non-CI builds, commit reporting, dirty-tree skip) are not re-opened.
- Existing code: crates/athena-cli/tests/ontology.rs and docs/ontology.toml are extended, not replaced.
- Scan scope becomes BRIEF.md, README.md and docs/*.md except docs/ontology.toml, plus the listed sources. Decision ids and CHANGELOG.md are excluded.

## Acceptance criteria

1. [content] The change description names the root cause: checks verified shape and listed sources, not each term's content against every governing source. Evidence: the CHANGELOG.md entry and the header comment in crates/athena-cli/tests/ontology.rs.
2. [content] scanned_files in crates/athena-cli/tests/ontology.rs scans BRIEF.md, README.md and docs/*.md for every term, not only the term's listed sources. Grep the function to check.
3. [behavior] A new unit test builds a scratch root where a governing doc outside the term's listed sources names a term in a sense its definition lacks. It fails against the old scanned_files and passes with the change. Run: cargo test -p athena-cli --test ontology.
4. [behavior] The real-file test passes on docs/ontology.toml with any new senses or well-formed excuses for newly found hits. Run: cargo test -p athena-cli --test ontology.
5. [content] CHANGELOG.md has an entry for the change. docs/decisions.md records the call only if the scan scope is a design choice.
6. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.

## Expected deliverables

- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing the y179 speed work, or changing secrets.rs, checks.rs or run.rs
- Changing ci.yml, xtask or setup.sh (the kq9v input-set check already exists)
- Adding terms not caused by a misread
- Changes to athena.toml, defaults.toml or workflows
- Changing any study's concept model or files under studies/

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the scan scope of the check, not at one term.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new failing-then-passing test plus fmt, clippy and test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A scratch-root test shows a missed sense being flagged.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One test file, with the data file and changelog only if needed.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust test code only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 00f51dbfcd76 → candidate 2c8af8aeda53. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  6 ++++
 crates/athena-cli/tests/ontology.rs | 60 +++++++++++++++++++++++++++++++++++--
 2 files changed, 63 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261102-223344-sqs9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261102-223344-sqs9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42161284919 for 2c8af8aeda533080c14c8d050913fb329d4bcf0f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42161284919 for 2c8af8aeda533080c14c8d050913fb329d4bcf0f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42161284919 for 2c8af8aeda533080c14c8d050913fb329d4bcf0f)
- ci-result: `not a check` passed (Check results for 2c8af8aeda533080c14c8d050913fb329d4bcf0f come from the CI run https://github.com/doodla/athena/actions/runs/42161284919 (671s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
