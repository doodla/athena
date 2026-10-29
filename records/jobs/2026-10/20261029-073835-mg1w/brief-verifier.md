# Job 20261029-073835-mg1w: verify "Scan only the request's own text at admission, not named files"

## Request (verbatim)

> Priority: urgent
> The boundary-word scan at request admission also scans the contents of files a request names, so any request that names the owner's brief file is refused: that file is the owner's own words and legitimately uses boundary words. On 2026-10-29 a request to map the brief's requirements was refused for words on brief lines 1067 to 1083 it did not contain. Scan only the request's own text at admission; the files it names are scanned when a candidate changes them, as today.
> Acceptance:
> - A test shows a request that names the brief file but whose own text has no boundary word is admitted, and a request whose own text has one is still refused.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/admission.rs, CHANGELOG.md

## Reading (accepted by the user)

The admission boundary-word scan (`scan_request_input` in crates/athena-core/src/reading.rs) also scans every file a request names, so naming BRIEF.md refuses the request. Change it to scan only the request text. Named files are still scanned when a candidate changes them, as today. The audience is the owner, who names BRIEF.md in requests.

Assumptions:
- Code survey: I read crates/athena-core/src/reading.rs. `scan_request_input` (line ~2030) scans the request text, then each file found by `request_text_touches`. `refuse_banned_input` calls it from the two ask entry points (lines ~715 and ~740). I also read crates/athena-core/src/run/mod.rs (test near line 2440-2484, which asserts that a dirty named file is refused, and calls `scan_request_input` directly) and crates/athena-core/src/admission.rs. admission.rs holds no boundary scan. It is named in the request, but the scan lives in reading.rs.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test do not apply.
- The existing run/mod.rs test expects a named dirty file to be refused at admission. It must be updated to the new behavior: the named file is admitted, the request's own banned word is still refused.
- The scan of files a candidate changes (the xtask policy test and the check on a candidate's changed files) stays untouched. `check_boundary_and_repo_files` still scans reading text.
- The `scan_request_input` signature may stay the same or drop its git parameters; the producer decides. It keeps the request-text findings and the `InputFinding` type. Any other caller, such as the one in run/mod.rs, is updated.

## Acceptance criteria

1. [behavior] A test in crates/athena-core shows that a request naming an existing file with a boundary word (such as BRIEF.md or a docs file) but whose own text has none is admitted. Run `cargo test -p athena-core`.
2. [behavior] A test shows that a request whose own text holds a boundary word is still refused with a 'not admitted' error naming source `request`. Run `cargo test -p athena-core`.
3. [content] `scan_request_input` in crates/athena-core/src/reading.rs no longer reads named files at admission, and its doc comment and the `refuse_banned_input` message describe request-only scanning. The old test in run/mod.rs that expected a named dirty file to be refused is replaced.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.
5. [content] CHANGELOG.md has an entry saying admission scans only the request's own text, not the files it names, and says why: the owner's brief legitimately uses boundary words.
6. [content] The change refuses fewer readings, not more; no new refusal rule is added. Expected 0% of newly refused readings.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/admission.rs (modify)

## Out of scope

- Changing the boundary word lists or `boundary_scan` in athena-context.
- Changing the scan of files a candidate changes (xtask policy test, verification).
- Editing BRIEF.md or any file under records/.
- Changes to athena.toml, defaults.toml or workflows.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added or updated; fmt, clippy -D warnings and cargo test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise admission of a request naming a file with a boundary word, and refusal of a request with one in its own text.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the scan scope at its source in scan_request_input, not just for BRIEF.md.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the scan plus its tests and the changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion checks that a request naming the brief file is admitted.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in run/mod.rs tests module. It uses the existing fixtures `repo()`, `athena(&repo,&state)` and `commit_file`, all present today at ~2255, 2359 and 2403. It commits BRIEF.md and docs/dirty.md, each with a banned word built with `["sand","box"].concat()` so the source stays clean. First it proves the old reach is gone by calling `crate::reading::scan_request_input("map the requirements in BRIEF.md and docs/dirty.md")` and asserting no findings. Then it calls the production entry point `athena.ask("map the requirements in BRIEF.md and docs/dirty.md", &Quiet)`, with the usual fake-model setup of the neighbouring tests. Run: `cargo test -p athena-core a_request_naming_a_file_with_a_boundary_word_is_admitted`. Observes: `ask` returns Ok with `proposal.reading.is_some()`, and `athena.head()` differs from the head before (a job was recorded).
  Fails without the behavior: With the old file loop, `ask` returns the 'not admitted' error listing `BRIEF.md:N: word` and `docs/dirty.md:N: word`, so the unwrap fails.
- Criterion 2: Test in run/mod.rs, kept from the first half of the old test and the existing banned-in-request test (~2400-2441). It calls `athena.ask` and `athena.queue` with request text holding a banned word on line 2, built by concatenation. It also names BRIEF.md, a clean file, to show the naming does not matter. It asserts on the error. It also calls `scan_request_input` directly, as in the old test. Run: `cargo test -p athena-core a_request_whose_own_text_has_a_boundary_word_is_refused`. Observes: The error string contains 'not admitted' and `request:2: <word>`, and `athena.head()` is unchanged (nothing recorded). The direct call returns exactly one `InputFinding{source:"request",line:2,term}`.
  Fails without the behavior: If the request-text scan were removed or broken when the file loop was dropped, `ask` would return Ok and the `unwrap_err` would fail.
- Criterion 3: Content criterion. The test above calls `scan_request_input` with one argument, which compiles only if the git and treeish parameters are gone, so no file read can happen. The old test `a_named_file_with_a_banned_word_is_flagged_with_path_and_line_and_a_clean_one_is_not` is deleted and its replacements are named differently. A reviewer can also run `git diff` to read the doc comments and the message. Run: `cargo test -p athena-core a_request_naming_a_file_with_a_boundary_word_is_admitted`. Observes: The tests compile and pass against the one-argument signature. The `git diff` of reading.rs shows doc comments that say 'the request's own text' and a bail message such as 'the request was not admitted: its own text holds words Athena's boundary scan rejects (source:line: word)', with no 'or a file it names'.
- Criterion 4: Run the three checks Athena runs, in order. Whole commands are run separately by the verifier if chaining is not allowed. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. This includes the xtask policy test, which scans the new test source and CHANGELOG for banned words; the test strings must be built with concat and the changelog must not name the banned words.
  Fails without the behavior: An unused import or parameter fails clippy, a leftover caller of the old three-argument form fails compile, and a literal banned word in the new test or changelog fails the policy test.
- Criterion 5: Content check by reading the diff. Run: `git diff -- CHANGELOG.md`. Observes: One new bullet under `## [Unreleased]` > `### Changed` says that admission (`athena ask`, including `--later`) scans only the request's own text, not the files it names; why (the owner's brief legitimately uses boundary words, and a request to map it was refused on 2026-10-29); and that files are still scanned when a candidate changes them. It does not spell out the banned words.
- Criterion 6: Review of the diff plus the tests: the change only deletes the file loop. `boundary_scan` and the word lists are untouched, and no new `bail!` or refusal rule is added. The criterion-2 test proves the request-text refusal still works with the same term list. Run: `cargo test -p athena-core a_request_whose_own_text_has_a_boundary_word_is_refused`. Observes: The diff has no added refusal condition. The refusal set is a strict subset of the old one: every request refused now was refused before.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 769bdf9a4d5c → candidate ae134c645d8e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  6 ++++
 crates/athena-core/src/reading.rs | 54 ++++++++---------------------
 crates/athena-core/src/run/mod.rs | 72 ++++++++++++++++++++++++---------------
 docs/enforcement.md               |  2 +-
 xtask/tests/policy.rs             |  2 +-
 5 files changed, 68 insertions(+), 68 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1646 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
