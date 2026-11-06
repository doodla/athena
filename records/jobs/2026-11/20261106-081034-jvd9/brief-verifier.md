# Job 20261106-081034-jvd9: verify "Check hand-kept mirrors against their sources before handover"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> Introspection 20261102-102715-564y (job 20261031-172040-kq9v); prevention it proposed: Add a code check, run in the repo tests and before verification, that computes the athena binary's real input set (every file the build compiles in, including everything xtask input_files should list) and fails if the hashFiles globs in eve...
> Introspection 20261106-030930-hb84 (job 20261106-002344-jn5t); prevention it proposed: Add a fast code check that runs in the producer's loop, and again before the verifier. It fails when any test file in the repo is missing from xtask/shard-timings.txt, and its message names the file and the line to add. Also keep the real f...
> Introspection 20261106-074349-rag3 (job 20261106-062207-cbcc); prevention it proposed: Add a check in the decisions test and in the producer's local checks. It compares docs/decisions.md with its base version and fails unless the base is an exact byte prefix of the new file, so every change is added text after the last old li...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a file that summarizes or mirrors a governing source (ontology terms, CI hashFiles globs, shard-timings, decisions.md) has only structure tests, so content that contradicts the source passes until the final verdict. Add one registry of such mirrors in athena-core with content checks against each source, run in the repo tests and the producer's local checks. Audience: the producer and verifier of future jobs. Covers jobs 20261017-173508-0wtc, 20261025-214509-y179, 20261031-172040-kq9v and the two other introspection jobs.

Assumptions:
- Code survey: I read the list of crates/athena-core/src, and crates/athena-core/src/reading.rs (check_governance_coverage, check_criteria_evidence, check_code_survey) as the pattern for named pre-handover checks. Entry point for the new check is a pure function in a new crates/athena-core/src/mirrors.rs, called from a crates/athena-cli/tests test that reads the repo files. I did not read the producer's local-check runner; the producer must locate it (likely crates/athena-core/src/checks.rs) and hook the check in before handover.
- The new file is code, not a record or data file under crates/, so check_record_file, AnyRecord and the round-trip test do not apply. The property is measured directly: each mirror's check reads the mirror and its source and compares content, not file existence.
- Root cause named in the change: derived or summarizing files are kept in step with their source by hand, and tests check only their shape. Fix is a registry of mirrors (mirror path, source path, comparison rule) so every new mirror needs an entry; initial rules: every test file listed in xtask/shard-timings.txt, docs/decisions.md base is a byte prefix of new text (append-only), ci.yml hashFiles globs cover BUILD_PATHS from crates/athena-core/src/status.rs, and a term-name search of governing docs for ontology entries when docs/ontology.toml exists.
- Job 20261017-173508-0wtc (ontology; verifier found Hermes sense missing): the ontology work was rejected and is dropped, not re-done here; its finding is addressed by the generic term-name check, not by writing the ontology. Job 20261025-214509-y179 (test speed): its changes to secrets.rs and checks.rs are dropped; its findings are speed-goal specific and out of scope. Job 20261031-172040-kq9v (CI cache): its ci.yml, xtask/src/main.rs and xtask/tests/policy.rs changes are dropped; its stale-key finding is addressed by the hashFiles-versus-BUILD_PATHS rule, which will fail on the current key until fixed separately.
- This is a refusal-adding check on the producer's own work, not on readings, so no replay of past readings applies; expected 0% of newly refused readings.
- Existing code: crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs and crates/athena-core/src/checks.rs are extended or reused, not created.

## Acceptance criteria

1. [content] The change description names the root cause: hand-kept mirrors of a governing source are tested for structure only, not content against the source.
2. [content] crates/athena-core/src/mirrors.rs holds a registry listing each mirror, its source and its comparison rule, and is exported from crates/athena-core/src/lib.rs.
3. [behavior] Run `cargo test -p athena-core mirrors`: unit tests fail on the old code and pass with the change for each rule (test file missing from shard-timings, decisions.md rewritten rather than appended, hashFiles glob missing a BUILD_PATHS entry, term name found in a governing doc but sense unlisted), each with a message naming the file and fix.
4. [behavior] Run `cargo test -p athena-cli`: a repo-level test runs the registry over the real files and fails when a mirror drifts; add an entry to the registry and the test covers it with no other edit.
5. [content] The producer's local check path invokes the registry before handover (Grep shows the call site), and CHANGELOG.md and docs/decisions.md (appended only) record the change.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/mirrors.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/tests/mirrors.rs (create)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Writing docs/ontology.toml or its test (job 20261017-173508-0wtc candidate dropped)
- The secrets.rs memo and run_check poll changes and any test-speed goal (job 20261025-214509-y179 candidate dropped)
- Editing .github/workflows/ci.yml, xtask/src/main.rs or xtask/tests/policy.rs to fix the cache key (job 20261031-172040-kq9v candidate dropped)
- Changing athena.toml, defaults.toml or workflows
- Adding speed-target measure-first stages

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests; cargo test and clippy pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The drift rule is enforced by a check, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the shared mirror-drift cause, not one instance.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check drift being refused, not that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One registry with a few rules.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Messages and docs written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the change description in its three homes: the `//!` doc of mirrors.rs, the CHANGELOG.md bullet under [Unreleased] / Added, and the appended D158. Each names the root cause in the same words: hand-kept mirrors of a governing source are tested for structure only, not content against the source, and the registry is the fix. Run: `git diff -- CHANGELOG.md docs/decisions.md README.md crates/athena-core/src/status.rs`. Observes: CHANGELOG.md and docs/decisions.md show only added lines; D158 follows D157 with Case, Decision, Alternatives, Tradeoff, Reversal cost and Provenance, and names the root cause, the four rules, the BUILD_PATHS correction (and that the five-path text at decisions.md line 3645 is outdated), the producer-loop call site, the per-entry touch filter and its limit (base drift is charged to a producer who touches a watched file; the unfiltered repo test would fail it anyway), the single shared send-back slot, the prose-only parts (brief note, registering a new mirror), the HEAD-comparison limit, `--doc` and `--bin` left to ci_shards, the shard-timings justification and the checks.rs omission. status.rs shows nine BUILD_PATHS roots; README.md line 193 names the constant.
- Criterion 2: Read mirrors.rs and lib.rs: REGISTRY holds four entries, each with a mirror path, a Source and a Rule; `pub mod mirrors;` and its module-list line are in lib.rs. Grep for `REGISTRY` and `pub mod mirrors`. Run: `git status --short`. Observes: crates/athena-core/src/mirrors.rs is new; lib.rs, ontology.rs, run/producer.rs, run/send_back.rs, run/tests.rs, status.rs, tests/drain.rs, tests/mirrors.rs, briefs.rs, main.rs, drain.rs, README.md, shard-timings.txt, CHANGELOG.md and decisions.md appear; checks.rs does not.
- Criterion 3: Unit tests in mirrors::tests and loop tests in run::tests (names beginning `mirrors_`). Pure tests call the rule functions with literal text, each with a failing and a passing case. missing_timing_lines: a test file absent from the timings fails and the message names xtask/shard-timings.txt and the line to add; a stale `--test` line fails; `--doc`, `--bin` and other-package lines are not reported. append_only_problem: a rewritten decision line fails and names docs/decisions.md and the first changed line; added text passes. hash_glob_problems with roots from status_build_paths on a literal status.rs text: a ci key hashing `athena.toml` while the roots lack it fails with `add ... to BUILD_PATHS in crates/athena-core/src/status.rs`; a root missing from the key fails with `add the glob`; a dir root with only `crates/` and no `**` fails; no athena-bin key fails; an unparsable array fails; a matching pair passes. Ontology: a governing-doc line `- Hermes: a role ...` for a term with only a tool sense yields a Drift whose message starts with `docs/ontology.toml:`, contains the unchanged inner text and ends with `add a senses entry or an excuse`; a covering sense or a valid excuse passes. Mirror::watches: one test per Source form, including Docs (docs/ontology.toml, BRIEF.md and a term's file source watched; an unlisted doc and a decision id not watched). check_all over a tempfile tree with a fixed base closure: the clean tree gives none; a None base skips rule (b); a non-Athena tree gives none via is_athena_source; with touched=Some(&["README.md"]) a timings drift already in the tree is not reported; with touched=Some(&["crates/demo/tests/b.rs"]) it is; with touched=Some(&["BRIEF.md"]) an uncovered ontology line is reported; a tree whose status.rs and ci.yml were both edited consistently gives none, and one with only status.rs edited gives a Drift. Loop tests (fake-agent harness): (1) the producer adds crates/demo/tests/b.rs and no timings line: the second producer prompt carries `## Sent back`, the leading sentence, `xtask/shard-timings.txt` and the fix; producer runs 2. (2) script fixing_timings_on_send_back adds the line only when its prompt contains '## Sent back' and reaches the verifier: producer runs 2, verifier runs 1. (3) a clean producer is not sent back. (4) base drift: b.rs is committed in the base with no timings line and the producer edits only crates/demo/tests/a.rs; expected result per decision 1: sent back once, the message names b.rs and the timings file; the producer that fixes the line on send-back reaches the verifier. (5) a producer touching none of the entry's files (edits README.md only) with the same base drift is not sent back. (6) ontology: the fixture commits BRIEF.md, a docs/ontology.toml with a Hermes tool sense, and the producer appends `- Hermes: a named role` to BRIEF.md; it is sent back once with the ontology Drift message; a producer that adds a covering sense or excuse on the send-back reaches the verifier. (7) a repository that is not Athena's source is not sent back. Run: `cargo test -p athena-core mirrors`. Observes: All pass. Each negative case returns a non-empty Vec whose text names the offending file and the fix. Fixed-on-send-back and fixing cases count producer runs 2 and verifier runs 1; clean, untouched and non-Athena cases count producer runs 1 with no `## Sent back` in any prompt.
  Fails without the behavior: Before the change the module does not exist, so nothing matches. Without a rule, its pure test gets an empty Vec and the non-empty assertion fails. Without the touch filter the README-only test (5) fails. Without Docs watches, the BRIEF.md test (6) and the watches unit test fail. Without parsing the snapshot's status.rs, the consistently-edited tree test fails. Without the producer-loop call, the second producer prompt has no `## Sent back` and the verifier runs with the drift in place, so tests (1), (4) and (6) fail. Without the leading sentence the prompt assertion fails. Without the is_athena_source gate test (7) fails.
  Test seam: Production entry points: mirrors::missing_timing_lines, append_only_problem, status_build_paths, hash_glob_problems, uncovered_lines, ontology_problems, Mirror::watches, check_entry, check_all (touched filter) and Athena::go through producer_loop in run/producer.rs. Doubles: literal strings; a tempfile tree (tempfile is an athena-core dependency); a base closure returning fixed bytes with no comparison logic; the fake agent script. The harness helpers repo(), commit_file(), governed_job_with(repo, script) and editing(counts, prompts, extra_edits) exist (run/tests.rs lines 2234-2284); reconciling_on_send_back (line 2288) is the pattern for the new script. The fixture repo commits setup.sh, crates/athena-core/Cargo.toml, xtask/shard-timings.txt, crates/demo/tests/a.rs, crates/demo/tests/b.rs (for test 4), BRIEF.md and a small docs/ontology.toml (for test 6). The module and pure fns are added by plan steps 2-4; the loop call, append_mirror_drift and the fixtures by step 6.
- Criterion 4: New test target crates/athena-cli/tests/mirrors.rs with four tests. (1) the_real_repo_has_no_mirror_drift calls check_all(repo_root, git_base(repo_root), None) over the real files, including the live docs/ontology.toml, and prints every Drift. (2) every_registry_entry_catches_its_drifted_fixture iterates REGISTRY; for each entry a tempdir fixture is built with an exhaustive `match entry.rule` (a new Rule variant without a fixture fails to compile), drifted one way, and check_entry must return a Drift naming entry.mirror and the fix; a clean fixture gives none. A new entry with an existing Rule is covered with no other edit. (3) derived_units_agree_with_cargo_metadata compares mirrors::test_units(root) with the `--test` and `--lib` units from `cargo metadata --no-deps --format-version 1 --locked` (serde_json is an athena-cli dependency). (4) the_ci_key_and_build_paths_cover_each_other runs hash_glob_problems on the real ci.yml with roots parsed from the real status.rs and asserts the parsed list equals status::BUILD_PATHS. The ontology target is run too: both_parsers_read_the_same_terms and the_ontology_file_is_sound. Run: `cargo test -p athena-cli --test mirrors`. Observes: All pass; then `cargo test -p athena-cli --test ontology` passes. A producer who adds a test file without a timings line, rewrites an old decision, edits ci.yml or BUILD_PATHS in one place only, or leaves an uncovered ontology line sees test (1) fail with the file and the fix.
  Fails without the behavior: Without the registry the target does not compile. With a stubbed rule, test (2) fails for that entry. Without the xtask/shard-timings.txt line `athena-cli --test mirrors`, test (1) names that file. With BUILD_PATHS left at five roots, tests (1) and (4) fail on `athena.toml`. If the narrow parse and the compiled constant differ, test (4) fails. If derivation diverges from cargo metadata, test (3) fails. If the ontology parsers differ, both_parsers_read_the_same_terms fails.
  Test seam: Production entry points: mirrors::check_all, check_entry, git_base, REGISTRY, test_units, status_build_paths, hash_glob_problems, parse_ontology. Root is Path::new(env!("CARGO_MANIFEST_DIR")).join("../..") (pattern at ontology.rs line 90). Fixtures: tempfile trees; check_entry takes a fixed base closure. athena-cli already has athena-core, tempfile and serde_json; Cargo.lock does not change. The target and fixtures are added by plan step 5, the functions by steps 2-4.
- Criterion 5: Grep shows the call sites: `mirror_drift_in` and `append_mirror_drift` in crates/athena-core/src/run/producer.rs (the producer loop, before verification) and `MIRROR_CHECK_INSTRUCTION` in crates/athena-context/src/briefs.rs. The loop tests of criterion 3 run the call. The briefs test producer_briefs_tell_the_producer_to_run_the_mirror_check builds a producer brief as the neighbour test near briefs.rs line 3028 does and asserts the text, that it sits after `## Words you must not write` and before `## Checks Athena will run`, and that crates/athena-cli/tests/mirrors.rs exists. git diff shows the CHANGELOG.md bullet and the append-only D158. Run: `cargo test -p athena-context producer_briefs_tell_the_producer_to_run_the_mirror_check`. Observes: The briefs test passes. Grep of `mirror_drift_in` in producer.rs shows the call between the gate and the break condition. The CHANGELOG.md hunk is one bullet under [Unreleased] / Added; the decisions.md hunk only adds lines after D157.
  Fails without the behavior: Without the section the brief lacks the instruction and the assertion fails. If the target file is renamed the existence assertion fails. Without the producer.rs call, the loop tests of criterion 3 fail.
  Test seam: Production entry point: briefs::producer_brief with a ProducerInputs literal, as in the neighbour test (exists, briefs.rs near line 3028). No double. mirror_check_section and the constant are added by step 7; the producer.rs call by step 6.
- Criterion 6: Run the three checks Athena runs, each as its own command, plus regressions on everything the change touches: `cargo test -p athena-cli --test drain` (touches_build and stale-line tests extended to the four new roots; the landing of athena.toml at tests/drain.rs line 3117), `cargo test -p athena-cli --test ontology`, `cargo test -p xtask --test ci_shards`, `cargo test -p xtask --test policy`, `cargo test -p athena-core --test external_calls_guard`. No reading parser, stem list or reading check changes, so no refusal_corpus case is added; I did not read its case ids and did not run it, and `cargo test --locked` runs it anyway. Run: `cargo test --locked`. Observes: The whole workspace passes. `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` are clean when run as separate commands.
  Fails without the behavior: Without the shard-timings line, xtask's cover test and the mirrors real-repo test fail. If a moved ontology helper changes behavior, the ontology tests fail. If a drain test expected no rebuild for athena.toml, the changed BUILD_PATHS shows it. A banned word or unformatted code fails policy, fmt or clippy.
  Test seam: The cargo commands over the real tree. No double. The oracles are athena-cli --test mirrors (step 5), xtask ci_shards (exists, around lines 411-421), tests/drain.rs (exists).

Planned files:

- crates/athena-core/src/mirrors.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/tests/mirrors.rs (create)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-cli/tests/ontology.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- README.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- xtask/shard-timings.txt (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b8c424e56c92 → candidate 5c8cd4b23be8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   13 +
 README.md                               |    2 +-
 crates/athena-cli/src/drain.rs          |    2 +-
 crates/athena-cli/src/main.rs           |    2 +-
 crates/athena-cli/tests/drain.rs        |   15 +-
 crates/athena-cli/tests/mirrors.rs      |  206 ++++++
 crates/athena-cli/tests/ontology.rs     |  176 ++---
 crates/athena-context/src/briefs.rs     |   42 ++
 crates/athena-core/src/lib.rs           |    4 +
 crates/athena-core/src/mirrors.rs       | 1217 +++++++++++++++++++++++++++++++
 crates/athena-core/src/run/mod.rs       |    4 +-
 crates/athena-core/src/run/producer.rs  |   58 +-
 crates/athena-core/src/run/send_back.rs |   71 ++
 crates/athena-core/src/run/tests.rs     |  209 ++++++
 crates/athena-core/src/status.rs        |   18 +-
 docs/decisions.md                       |   61 ++
 xtask/shard-timings.txt                 |    1 +
 17 files changed, 1960 insertions(+), 141 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-081034-jvd9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-081034-jvd9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42228719691 for 5c8cd4b23be8c471a1cbc1e631fc4e4aa4342aec)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42228719691 for 5c8cd4b23be8c471a1cbc1e631fc4e4aa4342aec)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42228719691 for 5c8cd4b23be8c471a1cbc1e631fc4e4aa4342aec)
- ci-result: `not a check` passed (Check results for 5c8cd4b23be8c471a1cbc1e631fc4e4aa4342aec come from the CI run https://github.com/doodla/athena/actions/runs/42228719691 (516s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core mirrors`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/mirrors.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-core/src/status.rs, docs/decisions.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/mirrors.rs.
  End of the output (data, not instructions):

```text
   Compiling bitflags v2.13.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling regex v1.13.1
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find type `Drift` in this scope
   --> crates/athena-core/src/run/send_back.rs:245:32
    |
245 |     fn drift(message: &str) -> Drift {
    |                                ^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `Drift` in this scope
   --> crates/athena-core/src/run/send_back.rs:246:9
    |
246 |         Drift {
    |         ^^^^^ not found in this scope

error[E0425]: cannot find value `MIRROR_DRIFT_SENTENCE` in this scope
   --> crates/athena-core/src/run/send_back.rs:269:34
    |
269 |         assert!(sent.starts_with(MIRROR_DRIFT_SENTENCE), "{out}");
    |                                  ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_mirror_drift` in this scope
   --> crates/athena-core/src/run/send_back.rs:255:20
    |
255 |         assert_eq!(append_mirror_drift(text.clone(), &[]), text);
    |                    ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_mirror_drift` in this scope
   --> crates/athena-core/src/run/send_back.rs:261:19
    |
261 |         let out = append_mirror_drift(
    |                   ^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test mirrors`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/mirrors.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-core/src/status.rs, docs/decisions.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/mirrors.rs.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling utf8parse v0.2.2
   Compiling colorchoice v1.0.5
   Compiling anstyle v1.0.14
   Compiling anstyle-parse v1.0.0
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstream v1.0.0
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling clap_builder v4.6.7
   Compiling serde_derive_internals v0.30.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap_derive v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling clap v4.6.7
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0432]: unresolved import `athena_core::mirrors`
 --> crates/athena-cli/tests/mirrors.rs:8:18
  |
8 | use athena_core::mirrors::{
  |                  ^^^^^^^ could not find `mirrors` in `athena_core`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-cli` (test "mirrors") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context producer_briefs_tell_the_producer_to_run_the_mirror_check`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/mirrors.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-core/src/status.rs, docs/decisions.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/mirrors.rs.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling libc v0.2.190
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling proc-macro2 v1.0.107
   Compiling winnow v0.7.15
   Compiling quote v1.0.47
   Compiling syn v3.0.7
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling getrandom v0.3.4
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.34s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_context-6da838b4f8c2a29c)

running 1 test
test briefs::tests::producer_briefs_tell_the_producer_to_run_the_mirror_check ... FAILED

failures:

---- briefs::tests::producer_briefs_tell_the_producer_to_run_the_mirror_check stdout ----

thread 'briefs::tests::producer_briefs_tell_the_producer_to_run_the_mirror_check' (10384) panicked at crates/athena-context/src/briefs.rs:3062:9:
assertion failed: brief.contains("run `cargo test -p athena-cli --test mirrors` before you finish")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    briefs::tests::producer_briefs_tell_the_producer_to_run_the_mirror_check

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-context --lib`
error: 1 target failed:
    `-p athena-context --lib`
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
