# Job 20261016-205137-wm0p: verify "Judge athena.toml by the candidate's schema without host builds"

## Request (verbatim)

> Parallel jobs failed a third time (20261016-200605-xh54): the landing gate (athena-gate content.rs) parses a changed athena.toml with the running schema, the same flaw #118 fixed only in admission, as #118's verifier noted. #118's verifier also noted admission now compiles candidate-supplied code on the host. Fix both checks at the root, without building candidate code outside the agent sandbox.

## Reading (accepted by the user)

The landing gate (content.rs) parses a changed athena.toml/defaults.toml with the running schema, so a commit that adds a section together with its schema is refused; admission fixed this by building the candidate's athena-data on the host. Fix both at the root: add a test in the candidate's own athena-data that parses the repo's athena.toml and defaults.toml, which runs only in the agent sandbox. When the commit changes crates/athena-data/, gate and admission rely on that sandboxed test and never compile candidate code. Otherwise they parse with the running schema. Audience: Athena operators and the verifier of parallel jobs.

Assumptions:
- The agent sandbox runs cargo test on the candidate, so a repo test is the way to judge the candidate's schema without host compilation.
- Reading at base: gate content.rs lines 84-104 use Config::parse and DefaultsModel::parse (running schema); admission.rs check_with_candidate_schema builds candidate code with cargo run on the host.
- The host-side builder (check_with_candidate_schema, CHECK_MAIN) is removed, not kept as a fallback.
- When crates/athena-data/ is unchanged, the running schema equals the parent's, so host parsing is safe and still catches bad files.
- A schema-changing commit whose sandboxed test does not run or fails is caught by the existing check/test gate; the decision is recorded in docs/design and CHANGELOG.
- Governance files athena.toml, defaults.toml and workflows need no change.

## Acceptance criteria

1. [behavior] Gate regression test: a commit that adds an athena.toml section and the matching athena-data schema change in one commit produces no Configuration violation from athena-gate content::check, while the same athena.toml change without a schema change is still refused with the parse error. Run: cargo test -p athena-gate.
2. [behavior] Admission test: a candidate changing both crates/athena-data/ and athena.toml is admitted without any process being spawned to build candidate code, and an unknown section with no schema change is still a violation. Run: cargo test -p athena-core.
3. [content] admission.rs no longer contains check_with_candidate_schema, CHECK_MAIN, or any Command::new(cargo) use on candidate files; grep shows none in athena-core or athena-gate.
4. [behavior] A test in crates/athena-data parses the repository's athena.toml and defaults.toml with that crate's own schema and fails if either does not parse, so a candidate's sandboxed cargo test judges its own schema. Run: cargo test -p athena-data.
5. [behavior] Both checks use one shared rule for deciding when to defer to the sandboxed test, so gate and admission cannot diverge. Run: cargo test, cargo clippy --all-targets, cargo fmt --check all pass.
6. [content] A short design note records the decision with alternatives (host build, sandbox build, lenient parse) and states why candidate code is never built on the host; CHANGELOG notes the change.

## Expected deliverables

- crates/athena-gate/src/content.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-data/tests/repo_config.rs (create)
- CHANGELOG.md (modify)
- docs/design/roles-and-ladder.md (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Parallel drain behavior, panic isolation, and the '(could run in parallel)' label from job xh54.
- Re-running job qcrz or landing the [parallel] section.
- Changing the schema fields themselves.
- Sandboxing or changing the agent sandbox.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Regression tests for gate and admission plus an athena-data config test; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the gate and admission with one shared rule instead of patching the gate alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that schema-plus-config commits pass and bad configs still fail, by running tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design note and CHANGELOG written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a test in the repo, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the two checks and their shared helper.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 11068b5b16f9 → candidate 9f9c8a9413e0. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  12 +-
 crates/athena-core/src/admission.rs     | 213 +++++---------------------------
 crates/athena-data/src/lib.rs           |   3 +
 crates/athena-data/tests/repo_config.rs |  35 ++++++
 crates/athena-gate/src/content.rs       |  70 ++++++++++-
 crates/athena-gate/src/lib.rs           |  18 +++
 docs/design/roles-and-ladder.md         |  26 ++++
 7 files changed, 189 insertions(+), 188 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (772 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
