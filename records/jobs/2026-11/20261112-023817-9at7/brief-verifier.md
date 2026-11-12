# Job 20261112-023817-9at7: verify "Enforce Opus/Sonnet/Haiku roles in config and Jev gateway for SET-02"

## Request (verbatim)

> try again on 20261030-001211-6mzd

## Reading (accepted by the user)

Retry of 20261030-001211-6mzd (rejected: the hard-coded ROSTER had no reader). Instead of a copied list, Config::validate in athena-data now refuses an athena.toml whose [models] roles lack an Opus, a Sonnet or a Haiku model. A test checks the real athena.toml plus the Jev gateway constants, and OLY-B-SET-02 records that as mechanism and measure. Audience: Athena's owner and the spec gap check.

Assumptions:
- Prior job 20261030-001211-6mzd: finding 1 (major, ROSTER in jev.rs read by nothing, test checks only constants) is fixed by dropping ROSTER and jev.rs edits entirely and enforcing the roles through Config::validate, which every Config::parse calls. Finding 2 (minor, xtask/shard-timings.txt) is kept as a deliverable because the new test file needs a line there (the estimate is 5.00 s). Finding 3 (nit, lib.rs unchanged) no longer applies: lib.rs is dropped from scope.
- Prior candidate files: CHANGELOG.md kept (rewritten for the new mechanism); crates/athena-core/src/jev.rs dropped (reverted); crates/athena-core/tests/model_roster.rs kept but rewritten to load the repository athena.toml; docs/spec.toml kept (re-edited); xtask/shard-timings.txt kept.
- Code survey: read crates/athena-data/src/config.rs (Models struct at ~314, ConfigError at ~567, Config::parse and validate at ~616-680, which already check verifier not in producer_allowed and default producer allowed), crates/athena-core/src/reading.rs (reads config.models.producer_allowed at ~617, ~797 to choose and check the producer model), and crates/athena-core/src/jev.rs (ENDPOINT, MODEL constants at lines 31-33). Consumers of the roles: reading.rs and the briefs. No record or data file is added under crates/, so check_record_file and AnyRecord do not apply. The producer must check existing config tests and fixtures that build Models with other model names (e.g. crates/athena-context/src/briefs.rs ~3550, config.rs tests ~1329) still pass, and adjust fixtures if the new rule refuses them.
- Existing code: crates/athena-data/src/config.rs (Config::validate, ConfigError), crates/athena-core/src/jev.rs (ENDPOINT, MODEL, reused not duplicated), docs/spec.toml (OLY-B-SET-02 entry), athena.toml (read only, current [models] already has claude-sonnet-5-5, claude-haiku-5-5 and verifier claude-opus-5-5).
- Family match is by substring on the model name (opus, sonnet, haiku) across reader, producer_default, producer_allowed and verifier. athena.toml is not changed because it already satisfies the rule; Consumers: none found for any config file change.
- Jev stays shadow-only; the requirement's Jev clause is shown by a test that the Jev endpoint is on ai-gateway.vercel.sh and the model is typesafe-ai/jev, not by making Jev decide real work.
- The measure ends with `status: not shown` plus the test command, since no `athena status` label covers this.

## Acceptance criteria

1. [content] docs/spec.toml OLY-B-SET-02 has a mechanism other than "none" naming Config::validate in crates/athena-data/src/config.rs and the test; owner Athena; and a measure ending with `status: not shown` and the test command.
2. [content] Consumer named: crates/athena-data/src/config.rs Config::validate (called by Config::parse for every load) is the non-test reader. It returns a new ConfigError variant when [models] roles lack an Opus, a Sonnet or a Haiku model. No hard-coded ROSTER list is added, and jev.rs is unchanged.
3. [behavior] Run `cargo test -p athena-data`: new unit tests build a Config with the Haiku, Sonnet or Opus model removed from the roles and assert Config::parse returns the new error for each. The same text with all three present parses. These tests fail if the validation is removed.
4. [behavior] Run `cargo test -p athena-core --test model_roster`: it parses the repository athena.toml through the real loader and asserts the roles cover Opus, Sonnet and Haiku. It also asserts jev::ENDPOINT contains ai-gateway.vercel.sh and jev::MODEL is the Jev model, using the existing constants.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including `cargo test -p xtask --test spec` and xtask ci_shards (xtask/shard-timings.txt has a model_roster line).
6. [content] CHANGELOG.md has an entry for the model-family check; no secrets are written; athena.toml, defaults.toml and workflows are unchanged.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/tests/model_roster.rs (create)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Making Jev decide real work (it stays shadow-only)
- Changing athena.toml, defaults.toml or workflows
- Other requirements such as OLY-B-SET-04
- Adding a ROSTER constant or any change to crates/athena-core/src/jev.rs and lib.rs (prior candidate dropped)

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the real config loader and show the refusal when a family is removed.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that configured roles cover the three families, not that a list exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read docs/spec.toml OLY-B-SET-02 (starts at :2019): the mechanism names Config::validate in crates/athena-data/src/config.rs and the tests, owner is Athena, and the measure ends `status: not shown` with the test commands. The test spec_entry_has_a_real_mechanism in model_roster.rs reads docs/spec.toml via Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."), slices the OLY-B-SET-02 block, and asserts no `mechanism = "none"`, the text `Config::validate`, `owner = "Athena"`, and a measure value (prefix stripped, closing quote trimmed) ending `status: not shown`. A second command, `cargo test -p xtask --test spec`, shows the spec format tests still accept the entry (a format check only). Run: `cargo test -p athena-core --test model_roster`. Observes: spec_entry_has_a_real_mechanism passes and the spec file read shows the new mechanism text; xtask spec tests pass.
  Fails without the behavior: If the entry still reads mechanism = "none", or does not name Config::validate, or the measure does not end `status: not shown`, spec_entry_has_a_real_mechanism fails.
  Test seam: Entry point: a plain file read of docs/spec.toml in the integration test. No fake. Added by plan step 4 (test) and step 6 (spec edit).
- Criterion 2: Read config.rs: Config::validate (called by Config::parse at :616-619) returns ConfigError::ModelFamily when a family is missing. Grep `ROSTER` under crates/ finds no match. `git status` and `git diff` list no change to crates/athena-core/src/jev.rs or lib.rs. The criterion 3 tests drive the variant through Config::parse, which shows validate is its caller. Non-test callers of validate through parse: config_at (athena-gate/src/lib.rs:184), JobConfig::at (capability.rs:41), admission.rs:933, content.rs:99 and :133, invariant.rs:832, gh/records.rs:314, xtask/src/main.rs:184, athena-cli wiring.rs:206. Run: `cargo test -p athena-data`. Observes: The crate builds with the new variant; the ModelFamily tests pass through Config::parse; grep finds no ROSTER; the diff does not list jev.rs or lib.rs.
  Fails without the behavior: If validate does not return ModelFamily, the criterion 3 tests fail, so the variant cannot sit unread.
  Test seam: Entry point Config::parse (config.rs:616). No fake. The seam exists (read in config.rs).
- Criterion 3: New unit tests in the config.rs tests module (it imports super::*; REPO_CONFIG = include_str!(athena.toml) at config.rs:888). A helper applies (from, to) replacen edits to REPO_CONFIG and asserts the text changed, so a pattern that misses cannot pass silently. Cases: (haiku) producer_allowed becomes ["claude-sonnet-5-5"], so no role names haiku; (sonnet) reader, producer_default, hard_producer_model and producer_allowed all become claude-haiku-5-5; (opus) verifier becomes claude-other-5-5, outside producer_allowed, so VerifierCanProduce cannot fire first. [models.names] still lists all three, which shows the check does not read it. Each case asserts matches!(Config::parse(&text), Err(ConfigError::ModelFamily("haiku" | "sonnet" | "opus"))) with the exact family, and that the message names the family and says every athena.toml. Controls: the unchanged REPO_CONFIG parses; uppercase names such as Claude-Haiku still count. Run: `cargo test -p athena-data`. Observes: Each removal returns the matching ModelFamily error; both controls parse.
  Fails without the behavior: If the family check is deleted from validate, the three removal cases parse and the matches! assertions fail. If a replacen misses, the text-changed assertion fails.
  Test seam: Entry point Config::parse, which calls validate. Fixture: the real athena.toml text edited in memory. No fake. The seam exists; the helper and tests are added by plan step 3.
- Criterion 4: The integration test reads athena.toml from Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."), as crates/athena-data/tests/repo_config.rs:10-15 does, and panics with a readable message if the read fails (never skips). It calls athena_data::Config::parse (the real loader and validate) and asserts the lowercased roles contain opus, sonnet and haiku separately. A mutation test removes the haiku id from producer_allowed in the text and asserts ConfigError::ModelFamily("haiku"). It asserts athena_core::jev::ENDPOINT contains "ai-gateway.vercel.sh" and athena_core::jev::MODEL == "typesafe-ai/jev". Run: `cargo test -p athena-core --test model_roster`. Observes: All assertions pass on the repository files; the mutated copy is refused.
  Fails without the behavior: If a family is removed from the real athena.toml or the validate check is removed, the parse assertion or the mutation assertion fails. If ENDPOINT or MODEL change, the Jev assertions fail.
  Test seam: Entry points: athena_data::Config::parse and the pub consts athena_core::jev::ENDPOINT and MODEL (jev.rs:31-33). athena-data is a normal dependency of athena-core, so an integration test can use it. No fake. The consts and loader exist; the test file is added by plan step 4.
- Criterion 5: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. Also run `cargo test -p xtask --test spec`, `cargo test -p xtask --test ci_shards`, `cargo test -p xtask --test policy`, `cargo test -p athena-core --test external_calls_guard`, `cargo test -p athena-gate`, `cargo test -p athena-core` (gh/tests.rs cutover tests, admission tests), `cargo test -p athena-cli` (cutover_fixture, trailer_stop) and `cargo test -p athena-data --test repo_config`. The CI job `athena check --history` is the check that covers past configs; it runs in CI and cannot be run here. The rule is a config check on structured TOML, not a request parser, so no refusal_corpus case applies; `cargo test -p athena-core --test refusal_corpus` is run only as a regression check and I read no corpus case for it. Run: `cargo fmt --all --check`. Observes: No formatting diff, no clippy warnings, a full green run. ci_shards' checked_in_timings_cover_every_workspace_test_unit sees the model_roster line. external_calls_guard passes.
  Fails without the behavior: Without the shard-timings line, checked_in_timings_cover_every_workspace_test_unit fails. An HTTP-client word in the new files fails external_calls_guard.
  Test seam: Entry points are the cargo commands. The ci_shards oracle lists targets from the crates' tests/ directories, so the new file is picked up automatically. The seam exists.
- Criterion 6: Read CHANGELOG.md and confirm a new entry under [Unreleased] / Added, in the style of the entries at CHANGELOG.md:12-20. It says OLY-B-SET-02 now has a mechanism, that Config::validate refuses a [models] section whose roles lack an Opus, a Sonnet or a Haiku model for every athena.toml Athena loads, names the tests, and says Jev stays shadow-only. Run `git status` and `git diff` to show no change to athena.toml, defaults.toml, .github/workflows, jev.rs or lib.rs. The policy test runs the boundary-word and personal-data scan. Run: `cargo test -p xtask --test policy`. Observes: The policy scan passes. The diff lists docs/spec.toml, crates/athena-data/src/config.rs, crates/athena-core/tests/model_roster.rs, CHANGELOG.md and xtask/shard-timings.txt. briefs.rs is absent on purpose (see Files).
  Fails without the behavior: The changelog read fails if the entry is missing. The policy test fails if an email-like string or a banned word is added.
  Test seam: Entry points: a file read, git diff and the xtask policy test. No fake. The seam exists.

Planned files:

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/tests/model_roster.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-context/src/briefs.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bf8d29ecef55 → candidate 6a318b001dc9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   9 +++
 crates/athena-core/tests/model_roster.rs |  78 ++++++++++++++++++++++
 crates/athena-data/src/config.rs         | 110 +++++++++++++++++++++++++++++++
 docs/spec.toml                           |   4 +-
 xtask/shard-timings.txt                  |   1 +
 5 files changed, 200 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261112-023817-9at7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-023817-9at7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43146745950 for 6a318b001dc901e723c1f13ce6c3c94f554e756b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43146745950 for 6a318b001dc901e723c1f13ce6c3c94f554e756b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43146745950 for 6a318b001dc901e723c1f13ce6c3c94f554e756b)
- ci-result: `not a check` passed (Check results for 6a318b001dc901e723c1f13ce6c3c94f554e756b come from the CI run https://github.com/doodla/athena/actions/runs/43146745950 (946s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test model_roster`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-data/src/config.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling getrandom v0.3.4
   Compiling zmij v1.0.23
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling serde_json v1.0.151
   Compiling winnow v1.0.4
   Compiling thiserror v2.0.21
   Compiling serde v1.0.229
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling winnow v0.7.15
   Compiling cpufeatures v0.2.17
   Compiling itoa v1.0.18
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling ref-cast v1.0.27
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling bitflags v2.13.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling equivalent v1.0.2
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
error[E0599]: no variant, associated function, or constant named `ModelFamily` found for enum `ConfigError` in the current scope
  --> crates/athena-core/tests/model_roster.rs:46:22
   |
46 |         ConfigError::ModelFamily("haiku")
   |                      ^^^^^^^^^^^ variant, associated function, or constant not found in `ConfigError`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-core` (test "model_roster") due to 1 previous error
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-data/src/config.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling libc v0.2.190
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling proc-macro2 v1.0.107
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling quote v1.0.47
   Compiling syn v3.0.7
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling getrandom v0.3.4
   Compiling toml v0.9.12+spec-1.1.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling thiserror v2.0.21
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
error[E0599]: no variant, associated function, or constant named `ModelFamily` found for enum `config::ConfigError` in the current scope
    --> crates/athena-data/src/config.rs:1355:43
     |
 567 | pub enum ConfigError {
     | -------------------- variant, associated function, or constant `ModelFamily` not found for this enum
...
1355 |             matches!(&error, ConfigError::ModelFamily(f) if *f == family),
     |                                           ^^^^^^^^^^^ variant, associated function, or constant not found in `config::ConfigError`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-data` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p xtask --test policy`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-data/src/config.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling serde v1.0.229
   Compiling crc32fast v1.5.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling adler2 v2.0.1
   Compiling simd-adler32 v0.3.10
   Compiling ref-cast v1.0.27
   Compiling miniz_oxide v0.9.1
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling xattr v1.6.1
   Compiling schemars_derive v1.2.2
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.32s
     Running tests/policy.rs (<job>/target/verify/debug/deps/policy-8766ce0d1330490f)

running 22 tests
test a_machine_path_fragment_is_flagged_even_inside_a_claims_quote ... ok
test a_banned_word_in_a_claims_quote_of_studied_source_is_not_flagged ... ok
test a_quote_with_escaped_characters_is_still_exempt ... ok
test a_stray_script_is_flagged ... ok
test a_missing_test_name_is_flagged ... ok
test an_undocumented_edge_is_flagged ... ok
test athena_bin_keys_cover_every_compiled_in_file ... ok
test dependency_direction_matches_the_architecture ... ok
test a_stale_or_incomplete_ci_yml_is_flagged ... ok
test every_action_is_pinned_by_a_full_commit_sha ... ok
test every_job_restores_the_tool_caches_before_it_runs_the_setup_step ... ok
test every_tracked_file_is_rust_or_data_or_an_allowed_exception ... ok
test only_a_source_quote_of_a_claim_is_blanked ... ok
test a_banned_word_in_a_study_s_own_prose_or_outside_studies_is_flagged ... ok
test the_admission_scan_finds_what_the_policy_scan_finds ... ok
test the_checks_job_looks_up_its_earlier_pass_before_setup_and_skips_on_a_hit ... ok
test the_pass_is_marked_and_saved_only_after_the_checks_and_the_shared_caches_stay_default_branch_only ... ok
test the_producer_brief_lists_exactly_the_scanners_terms ... ok
test every_test_named_in_the_enforcement_map_exists ... ok
test the_tool_cache_check_fails_when_a_restore_is_missing_or_after_setup ... ok
test the_boundary_scanner_flags_a_planted_term ... ok
test athena_refers_to_no_one_who_runs_it_or_where ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.37s

```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
