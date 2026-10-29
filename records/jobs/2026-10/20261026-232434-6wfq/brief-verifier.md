# Job 20261026-232434-6wfq: verify "Move produce_and_verify stages S1-S3 into run/producer.rs as methods"

## Request (verbatim)

> Priority: high
> Structure pass job R5 (docs/design/structure-pass.md, section 4). Split produce_and_verify step 1: stages S1-S3 into run/producer.rs as stage methods, bodies unchanged; early returns become Stop. No behavior change. Depends on 20261026-232426-91cn and 20261026-232431-dpem and 20261026-232420-ctt9.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, CHANGELOG.md

## Reading (accepted by the user)

Job R5 of the structure pass: the baseline (S1), producer snapshot (S2) and producer send-back loop (S3) at the start of `produce_and_verify` in run/mod.rs become stage methods in a new run/producer.rs. Each body is unchanged; the early `return Ok(closing)` paths become a `Stop` value the driver acts on. Audience: maintainers and the verifier, who need proof that nothing changed. Prior jobs 20261026-232426-91cn (R2), 20261026-232431-dpem (R4) and 20261026-232420-ctt9 (R0) are kept as landed; their findings are addressed below.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs. `produce_and_verify` starts at line 1112. S1 (take_baseline and the `baseline` ui line) is at 1126-1147. S2 (work dir, Snapshot::create, mount, grant, compose, env) is at 1149-1176. S3 (the send-back loop, with the early returns at 1223, 1236, 1246 and 1256) is at 1177-1328. The candidate stage S4 follows at 1330, and its idle-producer return at 1364 stays in mod.rs. Also read crates/athena-core/src/run/send_back.rs and docs/design/structure-pass.md sections 3.3, 4 and 5. Line numbers in the doc predate R1-R4.
- Line numbers are from the current tree. Producer-stage code calls `self.invoke_and_record`, `self.failure`, `self.keep_timed_out_work`, `cost_text` and `agent_spec`. Those are `Athena` methods or items in mod.rs, so producer.rs needs `impl Athena` blocks and `pub(super)` visibility on the new methods, as R4 did for close.rs.
- Values that cross the seam (`seq`, `closing`, `report_gaps`, `report_kept_in`, `left_criteria`, `snapshot`, `report`) are carried by `&mut` parameters and a small `Reported`-style struct. Only local names change; a statement that would need a change beyond a name is reported, not guessed at.
- Prior jobs: 20261026-232426-91cn (R2), 20261026-232431-dpem (R4) and 20261026-232420-ctt9 (R0) are landed and kept. Their files send_back.rs, close.rs and the R0 test in run/mod.rs, and their CHANGELOG entries, are not touched. R2's finding (the design doc contradicts itself about test modules) is out of scope, since this job keeps tests in mod.rs and does not move any. R4's findings are addressed by making this job's CHANGELOG entry state what that entry omitted: the test count, the file counts before and after for each moved name, and any rewrapped lines. R0's findings (duplicated fake-job helpers, a misplaced comment) concern test code and are left alone, because this job edits no test beyond import lines.
- Moving the stage code into `Athena` stage methods is a mechanical split and adds no crate. The buy-first default does not apply.
- The R0 characterization test (`produce_and_verify_order_tests`) is the behavior proof for this job. Its ui-line order and invocation numbers must pass unchanged, and no new test is added.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-core -- --list`. The list of test names is identical to the base commit's (compare counts and names), and `produce_and_verify_order_tests::a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order` passes. The CHANGELOG entry states the test count.
2. [behavior] Run `cargo test --locked`, `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. All pass, and no `#[allow]` is added (check with `git diff`).
3. [content] `git diff` shows the S1-S3 body text in crates/athena-core/src/run/producer.rs as the same text as the removed lines of run/mod.rs (`git diff --color-moved` marks them moved). Lines not marked moved are limited to `mod`/`use`/visibility lines, the stage method signatures, the `Stop`/`Reported` types, the local-to-field renames and the rustfmt rewraps, and each is listed with its reason in the CHANGELOG entry.
4. [content] Each of the early `return Ok(closing)` paths that were in S1-S3 (the producer could not run, timed out, did not complete, returned no valid report) is paired in the CHANGELOG entry with the `Stop` that replaced it. The old and new line numbers are given, and the driver in run/mod.rs returns `closing` when it sees `Stop`. The `return Ok(closing)` at the idle-producer check (S4) is unchanged.
5. [content] The CHANGELOG entry lists every local that crossed the S1-S3 seam and the field or parameter that carries it now.
6. [content] Public paths are unchanged. `git diff --stat` lists only crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, CHANGELOG.md and optionally docs/design/structure-pass.md. No test line changes except `use` lines. The CHANGELOG entry gives, for each moved or new name, the Grep run over crates/ with the file count before and after, and the counts are equal.
7. [content] `produce_and_verify` in run/mod.rs now calls the three stage methods in the order baseline, snapshot, producer loop. Grep shows the `baseline   ` ui line and the `producer   sent back:` line only in producer.rs, and shows one definition of each stage method.
8. [content] CHANGELOG.md has an R5 entry under Unreleased saying there is no behavior change, in the style of the R2 and R4 entries.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving S4 and later stages (candidate, admission, checks, verification, assembly); jobs R6-R8
- Any behavior change, or test edits beyond import lines
- Fixing the R2 design-doc contradiction about test modules, deduplicating the R0 test helpers, or fixing the R0 comment
- Reverting or rewriting the landed files run/send_back.rs, run/close.rs or the R0 test, or their CHANGELOG entries
- Editing athena.toml, defaults.toml, workflows, gh/ files, or callers in main.rs or drain.rs
- Rewriting `produce_and_verify` as a state machine

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Existing tests and the R0 ordering test must pass, and fmt and clippy must be clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier compares the test list and runs cargo test, fmt and clippy.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move of S1-S3, with no other stage touched.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The R0 ordering test exercises the moved stages through the real entry point.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is an unchanged test list and unchanged run behavior, not the new file existing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry explains the move and its checks for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Compiler, tests, fmt and clippy enforce the no-change proof.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Before editing, the producer runs the --list command and saves the sorted output outside the repo (for example /tmp/tests-before.txt) with its count. After the change it runs the same command, diffs the sorted lists and compares counts. The count goes in the CHANGELOG. Entry point: `Athena::produce_and_verify`, reached by the R0 test through `go_job`/`dispatch`. Preconditions: a git repo fixture and the fake agent driver. Seam: the existing `FakeAgentDriver` and `FakeTurn` injected by the R0 test, which exists today (run/mod.rs near lines 2293-2310 and 2611). No seam is added; the stages are not inside the fake. The verifier confirms no test line changed by reading `git diff -U0` hunk positions against the start of the test modules. Run: `cargo test --locked -p athena-core -- --list`. Observes: The --list output is identical in names and count to the base list (no `producer::` test names). Then `cargo test --locked -p athena-core produce_and_verify_order_tests` passes, including `a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order`.
  Fails without the behavior: The ordering test asserts the ui lines in order (`baseline   probe ok`, then the `producer   ...` line, checks, replay), `baseline.before_invocation == 2` and the stages `["dispatched","landed"]`, all through the real `produce_and_verify`. It fails if the stages are called in a different order, if `seq` is not carried back through `Reported`, or if the baseline is recorded after the producer. Existing early-return tests fail if the `Flow::Stop` mapping is wrong.
- Criterion 2: Run the three Athena check commands on the whole workspace (real commands, no seam). Then check that no `#[allow]` was added: read `git diff -U0` for mod.rs and CHANGELOG.md and look for added lines containing `allow(`, and use the Grep tool (not a shell command) with pattern `allow\(` on crates/athena-core/src/run/producer.rs, which is a new file so every line is added. Run: `cargo test --locked`. Observes: `cargo test --locked`, `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` each exit 0. The Grep tool finds no `allow(` in producer.rs, and `git diff -U0 -- crates/athena-core/src/run/mod.rs` shows no added line with `allow(`.
  Fails without the behavior: A non-compiling split, a visibility mistake (a private method not visible to the driver), an unapplied rustfmt rewrap, or a clippy `large_enum_variant` or `needless_borrow` finding fails one of the three commands. If clippy trips, the producer boxes the payload or edits the single argument instead of adding `#[allow]`, and the allow check would catch a shortcut.
- Criterion 3: Read the colour-moved diff of mod.rs and producer.rs. All of the old S1-S3 body lines (other than the prelude `let` lines) must be marked moved. Each line not marked moved must be on the CHANGELOG list with its reason. Expected set: `mod producer;` and `use` lines; the three method signatures; the `Flow`, `Reported` and `ProducerSnapshot` types; the prelude `let id`/`jc`/`job_dir`/`p` lines; the destructuring line in `run_producer`; the four `return Ok(Flow::Stop);` lines; the `Reported`/`ProducerSnapshot` construction lines at the stage ends; the driver's calls; and any rustfmt rewrap. Entry point is the diff itself; no seam. Run: `git diff --color-moved=zebra --color-moved-ws=ignore-all-space -- crates/athena-core/src/run/mod.rs crates/athena-core/src/run/producer.rs`. Observes: All S1-S3 body text appears as moved; every non-moved line is in the CHANGELOG list.
  Fails without the behavior: A reworded, reordered or dropped statement would show as a plain added or removed line instead of a moved one, and the verifier would find an unlisted line. This is the mechanism that proves the text is the same, since the compiler alone would accept a reworded body.
- Criterion 4: Read the removed return lines at old mod.rs 1223, 1236, 1246 and 1256 in the diff and the four `return Ok(Flow::Stop);` lines in producer.rs. The CHANGELOG pairs each, with old and new line numbers, in the order: could not run, timed out, did not complete, returned no valid report. Check the driver has `let Flow::Go(reported) = ... else { return Ok(closing); };` and that the idle-producer `return Ok(closing)` (old 1364) is still in mod.rs with unchanged text, only shifted by a known offset. Behavior is covered by the full test run of demonstration 2. Run: `git diff -U0 -- crates/athena-core/src/run/mod.rs crates/athena-core/src/run/producer.rs CHANGELOG.md`. Observes: Four pairs, with numbers that match the diff. The driver returns `closing` on Stop. The S4 idle return is untouched.
  Fails without the behavior: If a path were left as a value the driver ignores, or a Stop turned into a continue, the existing producer failure tests and the pairing check would fail. If the driver stopped returning `closing` on Stop, the `closing.happened` set in the stage would be lost and a failing-producer job would close with the wrong state.
- Criterion 5: Read the CHANGELOG list against the code. Every local declared in S1-S3 and read after the stage must be listed with its new carrier: `Reported.seq`, `.snapshot`, `.work`, `.report`, `.report_gaps`, `.report_kept_in`, `.left_criteria`; `ProducerSnapshot.*` for S2 to S3; the `&mut Closing` parameter; `brief` and `accepted` as parameters; `job_dir` as dropped from the driver. Cross-check by reading which names the driver uses after the three calls. Run: `git diff -- CHANGELOG.md crates/athena-core/src/run/mod.rs crates/athena-core/src/run/producer.rs`. Observes: The listed locals equal the locals the driver uses after the calls (`snapshot`, `report`, `work`, `report_gaps`, `report_kept_in`, `left_criteria`, `seq`, `closing`), and each has a carrier.
  Fails without the behavior: A crossing local missing from the list would compile but leave the entry incomplete; the check fails when the list does not match the used-after set.
- Criterion 6: Check `git diff --stat` lists only the four allowed paths. Read `git diff -U0` for mod.rs and confirm every hunk header lies inside `produce_and_verify` or the `mod` lines at the top, with none inside `mod critic_soundness_tests` or `mod produce_and_verify_order_tests` (R0 test module starts near line 2293) and no changed line outside `use` lines in test modules. The verifier re-runs, with the Grep tool over crates/, the patterns listed in the entry (`take_baseline`, `agent_spec`, `invoke_and_record`, `keep_timed_out_work`, `cost_text`, `failure(`, `unmet_criteria`, `left_to_athena`, `send_back_text`, `planned_paths`, `produce_and_verify`, `run_baseline`, `prepare_producer`, `run_producer`, `ProducerSnapshot`, `Reported`, `Flow`) and compares file and occurrence counts with the entry. Entry point is the diff and Grep; no seam. Run: `git diff --stat`. Observes: Four paths in the stat. mod.rs hunks fall outside the test modules. Grep counts match the entry: occurrence counts unchanged for existing names, file counts one higher where a call moved to producer.rs (explained in the entry), new names show before 0.
  Fails without the behavior: A changed path outside the allowed set or an edit to a test body shows in the stat or hunks; a path change or missed caller breaks `cargo test --locked`. Without the counts in the entry, there is no record of the caller check.
- Criterion 7: Read the new `produce_and_verify` body in mod.rs: the calls are `run_baseline`, then `prepare_producer`, then `run_producer`. Use the Grep tool over crates/athena-core/src/run with the patterns `baseline   `, `producer   sent back:` and `fn (run_baseline|prepare_producer|run_producer)`. The ordering behavior is shown by the R0 test in demonstration 1. Entry point `produce_and_verify`; no new seam. Run: `git diff -- crates/athena-core/src/run/mod.rs`. Observes: `baseline   ` appears once, in producer.rs. `producer   sent back:` appears once, in producer.rs. Each of the three `fn` names has exactly one definition. The driver's call order is baseline, snapshot, producer loop.
  Fails without the behavior: If S1-S3 text were copied rather than moved, the two ui strings would also appear in mod.rs and the Grep would report two files. If the driver called the stages out of order, the R0 ordering test would fail, since the baseline line must come first and the baseline's `before_invocation` must be 2.
- Criterion 8: Content check: read the diff of CHANGELOG.md for a bullet under `## [Unreleased]` / `### Changed`, above the R4 bullet, starting "Structure pass R5:" in the style of R2 and R4. It covers: S1-S3 moved to `run/producer.rs` as `run_baseline`, `prepare_producer`, `run_producer`; no behavior change; the test count and that the list is unchanged; the early-return pairings with old and new lines; seam locals and carriers; non-moved lines with reasons, including rewraps; caller-check counts before and after; the note that `take_baseline` and the S4 idle return stay in mod.rs; and what the R4 entry left out, now stated. Run: `git diff -- CHANGELOG.md`. Observes: The R5 bullet exists in the right place and states each required fact.
  Fails without the behavior: A missing or misplaced entry, one that does not say "No behavior change", or one that omits the test count, pairings, seam locals, rewraps or counts.

Planned files:

- crates/athena-core/src/run/producer.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/structure-pass.md (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base e38e697e0178 → candidate fc16c84b5e12. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  52 ++++++
 crates/athena-core/src/run/mod.rs      | 220 ++----------------------
 crates/athena-core/src/run/producer.rs | 304 +++++++++++++++++++++++++++++++++
 docs/design/structure-pass.md          |   5 +
 4 files changed, 377 insertions(+), 204 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` failed (killed at the deadline of 900s)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked`, `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. All pass, and no `#[allow]` is added (check with `git diff`).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
