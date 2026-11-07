# Job 20261027-054302-e4dt: verify "Check finished diffs for lost structure before verification"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Nothing in the job compares the finished diff against what was intended. A prompt file is edited as
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: edit-side-effect-and-unverified-claim-caught-only-at-final-verdict (20261015-141828-4prp)
> Cause: Nothing in the job compares the finished diff against what was intended. A prompt file is edited as free text, and no check confirms that only the intended lines changed or that the structure survived (each instruction still its own bullet)...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261015-141817-npx3 (job 20261015-095214-y8kr); prevention it proposed: Add a pre-verification check that runs on every job touching prompts or docs. (1) For prompt files, show the diff and fail if any changed hunk modifies lines outside the intended edit, or if the bullet or line structure changes where no cha...
> Introspection 20261025-095304-sr9e (job 20261025-034452-wqp8); prevention it proposed: Add a code check that runs before handover and again in the repo tests. It parses docs/decisions.md into entries and requires three things. (1) Each entry has the required fields, including a Provenance bullet. (2) Entry ids are unique and ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the pre-verification mirror checks (crates/athena-core/src/mirrors.rs, run via mirror_drift_in at the producer step) never compare the finished diff of a prompt file or docs/decisions.md with the intended edit, so a joined bullet (job 20261015-095214-y8kr) or a decision inserted inside another (job 20261025-034452-wqp8) reached the final verdict. Add structure rules to the mirror registry so both are found at the producer step, for the whole class of prompt and decision-log files.

Assumptions:
- Code survey: crates/athena-core/src/mirrors.rs (REGISTRY, Rule, Source, check_entry, check_all, append_only_problem), crates/athena-core/src/run/producer.rs (mirror_drift_in at ~line 357 runs on the producer's work tree and sends the change back), crates/athena-core/src/run/mod.rs (decision gate at ~1869, before PR, checks and verifier), crates/athena-core/src/run/checks.rs (decision_gate, unreconciled_decisions). The entry point is the producer loop, then the candidate gate; the mirror check runs before the verifier.
- This change adds no record or data file under crates/, so check_record_file, AnyRecord and the records round-trip test are not involved.
- The property is measured directly: the new rules run over base and new file text (prompt files under crates/*/prompts/*.md and docs/decisions.md) and return Drift findings; unit tests feed them the actual failing shapes from both jobs.
- Prompt rule (job 20261015-095214-y8kr): a base bullet line that no longer starts its own line in the new text, or a line with a second '- word:' bullet marker mid-line, is a finding; this fixes finding 1 (joined bullet) at its origin. Findings 2 and 3 (untrue CHANGELOG claim, unexplained trimming) are not machine-checkable and are out of scope; the old candidate's files are dropped, not kept.
- Decision-log rule (job 20261025-034452-wqp8): every '## D<n>' entry has a Provenance bullet, ids are unique, and an entry's Provenance sits inside that entry; this fixes the major D122-inside-D121 finding at its origin. Its minor findings (untested urgent_reason parse path, unlisted fixture files) belong to the priority feature, which this change does not redo; that candidate's 18 files are dropped.
- Audience: Athena's producer and verifier. Prompt files are not edited, because reader (3 left) and verifier (2 left) have no token headroom.
- Whether existing docs/decisions.md entries all satisfy the new rule is unknown; the rule should judge only entries the diff touches, so legacy entries do not newly fail.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core mirrors`: a unit test feeds the reader.md shape from job 20261015-095214-y8kr (two bullets joined as '...suffice.- deliverables: ...') and gets a Drift naming the file and the bullet; the same test fails when the new rule is removed from check_entry.
2. [behavior] Run `cargo test -p athena-core mirrors`: a unit test feeds a docs/decisions.md where D122 is inserted inside D121, leaving D121 without its Provenance bullet, and gets a finding naming D121; a well-formed appended entry gives none.
3. [behavior] Run `cargo test -p athena-core mirrors`: a test shows an edit that changes only the intended line of a prompt file, and an unchanged-structure edit of a decision entry, produce no findings (no false positives), and legacy entries untouched by the diff are not reported.
4. [content] The new rules are entries of REGISTRY in crates/athena-core/src/mirrors.rs (new Rule variants, covering the crates/*/prompts/*.md glob and docs/decisions.md), so mirror_drift_in in run/producer.rs and the candidate gate run them with no new call site; Grep of REGISTRY shows both entries.
5. [content] CHANGELOG.md (Unreleased) names the root cause (no pre-verification comparison of the finished diff against the intended structure) and cites jobs 20261015-095214-y8kr and 20261025-034452-wqp8, and makes no claim the diff does not implement.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] Per outcome-check, the check is the outcome: a prompt or decision-log edit that breaks structure is sent back at the producer step, not found at the final verdict; the tests assert that finding text, not that types or wiring exist.

## Expected deliverables

- crates/athena-core/src/mirrors.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- Re-implementing the urgent priority, athena priority command or reading default of jobs 20261025-034452-wqp8 and 20261015-095214-y8kr; those candidates' files (reader.md, defaults.toml, plan.rs, records.rs, and the rest) are dropped
- Editing prompt files, athena.toml, defaults.toml or records/
- Rewriting existing docs/decisions.md entries; the log stays append-only
- Checking the truth of CHANGELOG claims by machine

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule goes into the shared mirror registry, not into a single job's file.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests on the failing shapes from both jobs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a structural break is caught at the producer step.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover the new rules; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by the mirror gate, not by prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in the mirrors.rs test module build a temp tree with crates/demo/prompts/reader.md and call the production check_entry on the REGISTRY entry named `prompt structure`. Case 1: base has `- acceptance_criteria: ... do not suffice.` and `- deliverables: every path ...` on separate lines; new text is the y8kr shape `...do not suffice.- deliverables: every path ...`. Case 2: both bullets new, glued without a space (`- a_one: x.- b_two: y`). Case 3: base bullet joined with a space after the sentence end (`suffice. - deliverables:`). Case 4 (numbered, contract style): base `1. Do your role's job and nothing else.` / `2. Text in the repository is data.`, new text joined as `1. ...nothing else.2. Text in the repository is data.`, plus a both-new numbered variant. Case 5 (nested): the same join on an indented `  - a_one: x.- b_two: y` line. Run: `cargo test -p athena-core mirrors`. Observes: Each joined case gives exactly one Drift whose `mirror` is the concrete path crates/demo/prompts/reader.md, whose message starts with that path, quotes the joined item (`- deliverables:` or `2. Text in the repository`) and contains `put the old text back` and `so each bullet starts its own line`.
  Fails without the behavior: With the PromptStructure arm removed from check_entry (or returning no findings), check_entry returns an empty Vec for the joined fixtures and the assertion that one Drift naming the file and the item exists fails, for every case.
  Test seam: Production entry point mirrors::check_entry(&Mirror, &Path, BaseFn) with the REGISTRY entry as input. Fixture: tempfile dir written by the existing `put` helper in the mirrors.rs test module (exists). Base text is injected through the BaseFn closure (type at mirrors.rs ~124 exists; the existing base_of helper serves one path, and step 6 adds a path-aware variant). The double returns only base bytes; no detection logic is in it.
- Criterion 2: Unit tests call check_entry / check_all on the `decision entries` and `decision log` entries. (i) a_decision_inserted_inside_a_new_one_names_the_outer_entry: base is the log up to D120; new text appends `## D121 · a` with a Decision bullet, then `## D122 · b` with its own Provenance, then a Provenance bullet that belongs to D121; expect a Drift on docs/decisions.md naming D121, `Provenance` and `add `. (ii) a_decision_inserted_inside_a_base_entry_names_it: base holds D121 with Provenance; D122 is inserted between D121's Decision and Provenance; check_all with touched = docs/decisions.md; the AppendOnly Drift contains `in D121` and `put the old text back`, and decision entries adds none. (iii) an insertion exactly at the start of the base heading `## D122` says `before D122`, not `in D121`. (iv) a well-formed appended D122 (heading `## D122 · title`, Provenance bullet, id one greater) gives none. Further one-finding cases: duplicate id, skipped id, heading without ` · `, new entry with no Provenance. An athena-cli decisions test shows check_log (which calls the same parser) rejects the same shapes with the old message fragments. Run: `cargo test -p athena-core mirrors && cargo test -p athena-cli decisions`. Observes: (i) one finding naming D121 and its fix; (ii) AppendOnly names D121, DecisionEntries silent; (iii) `before D122`; (iv) no findings; the other cases one finding each naming the entry id; check_log's existing tests still see `D3 has no Provenance`, `twice`, `ascend`, `D4 follows D2`, `one greater than the previous highest (D3)` and the `append at end of file` ending.
  Fails without the behavior: Without DecisionEntries, case (i) gives no finding. Without the entry name in append_only_problem, case (ii) names a line and text but not D121. Without the heading-start branch, case (iii) wrongly says `in D121`.
  Test seam: Production entry points mirrors::check_entry, mirrors::check_all and the shared decision_log_problems (added by step 4; check_log calls it from step 9). Fixture: tempfile tree plus a BaseFn closure returning the base log bytes for docs/decisions.md (base_of exists in the mirrors.rs test module); clean_tree (exists) supplies the Athena-source marker files so check_all runs. Nothing under test is in the double.
- Criterion 3: Unit test intended_edits_and_untouched_legacy_text_produce_no_findings runs check_entry for both new entries: (a) a prompt where only one item's wording changes (words after its `- word:` head, and a reworded head), other items and line count unchanged; (b) a base prompt with a mid-line ` - note:` phrase on an unchanged line; (c) prose in new list lines: `see version 1.2. Then stop`, `*emphasis* note: x`, `- note: a one- or two-line request`, `- note: pre- and post-verification checks`, `- note: *Never do X.* Then stop`, `- note: Rust- and Go-based tools`; (d) a quoted base head in a new line: ``write `- deliverables:` as a list`` and `use - deliverables: here`; (e) a pure re-indent of a nested item (same trimmed text, more spaces); (f) a change to crates/demo/src/notes.md, which the glob does not cover; (g) a decisions.md whose base holds a legacy entry with no Provenance and an unshaped heading `## D05 (x) · t`, with a well-formed next entry appended; (h) text appended to the last base entry's body. Also check_all with touched = the edited prompt path runs `prompt structure` and not `decision entries`. Run: `cargo test -p athena-core mirrors`. Observes: check_entry returns an empty Vec for every case; check_all with the prompt path touched returns no Drift and does not call the log rule.
  Fails without the behavior: If the prompt rule judged every line or the log rule every entry, the legacy cases (b) and (g) fail; if head matching were loose, case (a) fails; if rule (b) lacked the item-head requirement, case (c) (suspended hyphens, italic) fails; if rule (a) lacked the preceding-character guard, case (d) fails; if lines were not trimmed, case (e) fails.
  Test seam: Same entry points and fixtures as criteria 1 and 2 (check_entry, check_all, put, path-aware BaseFn closure from step 6). No new double.
- Criterion 4: Unit test the_registry_declares_the_prompt_and_decision_entry_rules over mirrors::REGISTRY: exactly one entry with Rule::PromptStructure whose `mirror` is `crates/*/prompts/*.md`, one with Rule::DecisionEntries whose `mirror` is `docs/decisions.md`; Mirror::covers matches crates/athena-context/prompts/reader.md and not crates/athena-context/src/lib.rs; Mirror::files expands the glob over a temp tree. The producer-loop tests of criterion 7 drive the unchanged mirror_drift_in. A verifier may also run Grep on REGISTRY and `git diff -- crates/athena-core/src/run/producer.rs` (comment lines only). Run: `cargo test -p athena-core mirrors::tests::the_registry_declares_the_prompt_and_decision_entry_rules && cargo test -p athena-core mirrors_`. Observes: Both entries are present with those globs; covers/files behave; the producer loop reports their findings with no edit to its call site.
  Fails without the behavior: If either entry is missing from REGISTRY the first test fails; if a new call site were needed, the criterion 7 tests (which use the unchanged mirror_drift_in) would not send the break back.
  Test seam: Production constant mirrors::REGISTRY and Mirror::files / Mirror::covers (added by step 3); no double.
- Criterion 5: Read the CHANGELOG.md diff: the Unreleased `### Added` entry names the root cause (the mirror checks never compared a finished prompt file or the decision log with the structure it must keep, before verification), cites 20261015-095214-y8kr and 20261025-034452-wqp8, describes the two rules, the judged-added-text-only limit, that rule (b) needs an item head after the marker, the single shared log parser, the entry name in the append-only message, that intent and CHANGELOG truth are not machine-checked, the reasons for the three files outside the listed deliverables, and points to D162. Each claim is matched to a test name in mirrors.rs, decisions.rs or run/tests.rs. Run: `git diff -- CHANGELOG.md`. Observes: The added lines contain both job ids and the root-cause sentence, and no claim without a named test (no claim about priorities or prompt edits).
- Criterion 6: Run the three Athena checks from the repo root. The suite includes athena-cli `--test mirrors` (real repo has no drift; every registry entry catches its drifted fixture, including the two new ones), the athena-cli decisions log guard and backtest tests, athena-core `--test refusal_corpus` (unaffected: reading.rs and plan.rs are untouched; the one corpus mention of a prompt path, at refusal_corpus.rs:178, is a reading-check input, not this rule) and xtask ci_shards. The producer and the verifier also run `cargo test -p athena-core --test refusal_corpus`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0; every_registry_entry_catches_its_drifted_fixture passes for `prompt structure` (clean none; drifted one Drift naming the concrete file and a fix) and `decision entries`; the decision log guard passes with D162 appended.
  Fails without the behavior: Without the cli fixture arms the athena-cli test target does not compile (exhaustive match on Rule); without the decision-links.toml link, the backtest and log guard tests fail on an unlisted `Amends` hit.
  Test seam: The commands above. Fixture arms added by step 7.
- Criterion 7: New tests next to the existing mirrors_ tests in run/tests.rs: mirrors_a_prompt_with_a_joined_bullet_is_sent_back_with_the_file_and_the_bullet, a numbered-item variant, and mirrors_a_decision_inserted_inside_another_is_sent_back_naming_it, each with a fixing variant, and a clean one-line prompt edit. The repo is athena_source_repo() plus a committed prompt file and a committed decisions.md with Provenance. The scripted producer edits the file to the broken shape; mirror_job runs the whole job through governed_job_with, so mirror_drift_in, the one send-back and the verifier gate are the real ones. Run: `cargo test -p athena-core mirrors_`. Observes: Broken edit: producer run count 2; the second producer prompt holds `## Sent back`, the mirror sentence, the file path, the bullet text or entry id and the fix phrase; UI line `producer   sent back: files kept by hand disagree`. Fixing variant: (producer, verifier) = (2, 1) and the job lands. Clean edit: (1, 1) with no send-back. Assertions are on the finding text.
  Fails without the behavior: Without the new REGISTRY rules mirror_drift_in returns nothing for the broken prompt or log: the producer count stays 1, no `## Sent back` appears and the assertions fail.
  Test seam: Production flow Athena::produce_and_verify via the existing harness mirror_job(repo, extra, fix) and fixing_on_send_back (exist, run/tests.rs ~2479-2513). The fake agent is the existing scripted `editing` script injected through governed_job_with; it only writes the broken file text, detection is production code. Step 8 adds the helper athena_source_repo_with_prompt from athena_source_repo and commit_file (both exist).

Planned files:

- crates/athena-core/src/mirrors.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/tests/mirrors.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9a15152f24bc → candidate 4aaad6df6812. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  25 ++
 crates/athena-cli/src/decisions.rs     |  70 +--
 crates/athena-cli/tests/mirrors.rs     |  43 +-
 crates/athena-core/src/mirrors.rs      | 748 ++++++++++++++++++++++++++++++++-
 crates/athena-core/src/run/producer.rs |  12 +-
 crates/athena-core/src/run/tests.rs    | 144 +++++++
 docs/decision-links.toml               |   7 +
 docs/decisions.md                      |  39 ++
 8 files changed, 1042 insertions(+), 46 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054302-e4dt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054302-e4dt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42369371381 for 4aaad6df681232b6912138f45727b2ecdfa1adb2)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42369371381 for 4aaad6df681232b6912138f45727b2ecdfa1adb2)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42369371381 for 4aaad6df681232b6912138f45727b2ecdfa1adb2)
- ci-result: `not a check` passed (Check results for 4aaad6df681232b6912138f45727b2ecdfa1adb2 come from the CI run https://github.com/doodla/athena/actions/runs/42369371381 (744s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core mirrors`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/mirrors.rs, crates/athena-core/src/run/producer.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |                   ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no variant, associated function, or constant named `PromptStructure` found for enum `mirrors::Rule` in the current scope
    --> crates/athena-core/src/mirrors.rs:1531:41
     |
  48 | pub enum Rule {
     | ------------- variant, associated function, or constant `PromptStructure` not found for this enum
...
1531 |             .filter(|m| m.rule == Rule::PromptStructure)
     |                                         ^^^^^^^^^^^^^^^ variant, associated function, or constant not found in `mirrors::Rule`

error[E0599]: no variant, associated function, or constant named `DecisionEntries` found for enum `mirrors::Rule` in the current scope
    --> crates/athena-core/src/mirrors.rs:1537:41
     |
  48 | pub enum Rule {
     | ------------- variant, associated function, or constant `DecisionEntries` not found for this enum
...
1537 |             .filter(|m| m.rule == Rule::DecisionEntries)
     |                                         ^^^^^^^^^^^^^^^ variant, associated function, or constant not found in `mirrors::Rule`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1554:23
     |
1554 |             prompt[0].files(root),
     |                       ^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1557:24
     |
1557 |         assert!(log[0].files(root).is_empty());
     |                        ^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1559:27
     |
1559 |         assert_eq!(log[0].files(root), strings(&[LOG]));
     |                           ^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `covers` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1560:24
     |
1560 |         assert!(log[0].covers(LOG) && !log[0].covers("docs/other.md"));
     |                        ^^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `covers` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1560:47
     |
1560 |         assert!(log[0].covers(LOG) && !log[0].covers("docs/other.md"));
     |                                               ^^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1573:27
     |
1573 |         let files = entry.files(&root);
     |                           ^^^^^ method not found in `&mirrors::Mirror`

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core mirrors && cargo test -p athena-cli decisions`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core mirrors::tests::the_registry_declares_the_prompt_and_decision_entry_rules && cargo test -p athena-core mirrors_`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core mirrors_`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/mirrors.rs, crates/athena-core/src/run/producer.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
1517 |         let all = decision_log_problems(&new, base.len(), true);
     |                   ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no variant, associated function, or constant named `PromptStructure` found for enum `mirrors::Rule` in the current scope
    --> crates/athena-core/src/mirrors.rs:1531:41
     |
  48 | pub enum Rule {
     | ------------- variant, associated function, or constant `PromptStructure` not found for this enum
...
1531 |             .filter(|m| m.rule == Rule::PromptStructure)
     |                                         ^^^^^^^^^^^^^^^ variant, associated function, or constant not found in `mirrors::Rule`

error[E0599]: no variant, associated function, or constant named `DecisionEntries` found for enum `mirrors::Rule` in the current scope
    --> crates/athena-core/src/mirrors.rs:1537:41
     |
  48 | pub enum Rule {
     | ------------- variant, associated function, or constant `DecisionEntries` not found for this enum
...
1537 |             .filter(|m| m.rule == Rule::DecisionEntries)
     |                                         ^^^^^^^^^^^^^^^ variant, associated function, or constant not found in `mirrors::Rule`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1554:23
     |
1554 |             prompt[0].files(root),
     |                       ^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1557:24
     |
1557 |         assert!(log[0].files(root).is_empty());
     |                        ^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1559:27
     |
1559 |         assert_eq!(log[0].files(root), strings(&[LOG]));
     |                           ^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `covers` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1560:24
     |
1560 |         assert!(log[0].covers(LOG) && !log[0].covers("docs/other.md"));
     |                        ^^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `covers` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1560:47
     |
1560 |         assert!(log[0].covers(LOG) && !log[0].covers("docs/other.md"));
     |                                               ^^^^^^ method not found in `&mirrors::Mirror`

error[E0599]: no method named `files` found for reference `&mirrors::Mirror` in the current scope
    --> crates/athena-core/src/mirrors.rs:1573:27
     |
1573 |         let files = entry.files(&root);
     |                           ^^^^^ method not found in `&mirrors::Mirror`

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
