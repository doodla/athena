# Job 20261027-085452-nm2t: Judge a PR's candidate head as pending in pr-head and history checks

## Request (verbatim)

> Every PR's first CI run fails pr-head and history checks on the candidate commit (no landing yet); the checks cannot become required at cutover as designed
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261027-011309-3p55 (key pr-ci-history-checks-fail-on-candidate-push-before-landing), queued at 2026-10-27T01:13:09.228527024Z by the steward role.
> Rung of the requirements ladder: design-decisions.
> 
> ## Source
> 
> path .github/workflows/ci.yml
> ```
> # Athena's CI (docs/design/athena-v0.md §11.2): a thin call into Rust. Both jobs check out the full
> # history and start with `./setup.sh --ci`. Actions are pinned by commit SHA, the token is read-only, and
> # nothing is deployed. Tests that call real models are opt-in and never run here.
> #
> # Cache layers (docs/design/ci-routing.md §4.1): toolchain, registry and target, keyed on the content of
> # rust-toolchain.toml and Cargo.lock. Every run restores them; only a push to the default branch saves them, so
> # an unreviewed branch cannot fill or poison the shared entries. Each job reports exact, restore key or miss.
> #
> # The 'fmt, clippy, tests' job also reuses its own earlier pass (docs/decisions.md D121): before setup it looks up
> # `ci-pass-<governed fingerprint>`; a marker naming that fingerprint skips setup and the checks. The marker is
> # saved only after the checks pass, on pull-request branches too (entries stay on the branch that saved them).
> name: ci
> 
> on:
>   # Push runs only for main. A push to a pull request's branch ran the same checks a second time for the same
>   # commit, in the push cache scope, which misses the pull_request run's cached pass. On PR #185 the pull_request
>   # run passed from the cache in 26 s, while two push runs of the same commit each ran the full 16 minutes
>   # (timings from the request; docs/decisions.md D125). The pull_request run satisfies the required check.
>   push:
>     branches: [main]
>   pull_request:
>   workflow_dispatch:
>   schedule:
>     # Weekly, so that new gitleaks rules reach old commits, and so the default branch's cache entries stay fresh.
>     - cron: "17 6 * * 1"
> 
> permissions:
>   contents: read
> 
> jobs:
>   checks:
>     name: fmt, clippy, tests
>     runs-on: ubuntu-24.04
>     steps:
>       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
>         with:
>           fetch-depth: 0
>           persist-credentials: false
>           # For pull requests, check the commits that would land, not GitHub's merge commit.
>           ref: ${{ github.event.pull_request.head.sha || github.sha }}
>       # Reuse of an earlier pass (docs/decisions.md D121). The fingerprint is the governed content of the checked-out
>       # tree (records/ and owner paths left out). xtask is built with the runner's own stable cargo, into a
>       # separate target directory, so this step needs neither the pinned toolchain nor setup.
>       - name: Fingerprint the governed content
>         id: fingerprint
>         env:
>           CARGO_TARGET_DIR: ${{ runner.temp }}/xtask-target
>         run: |
>           cargo +stable build --locked -q -p xtask
>           xtask="$CARGO_TARGET_DIR/debug/xtask"
>           echo "XTASK=$xtask" >> "$GITHUB_ENV"
>           fingerprint="$("$xtask" fingerprint)"
>           echo "fingerprint=$fingerprint" >> "$GITHUB_OUTPUT"
>           echo "key=ci-pass-${fingerprint#sha256:}" >> "$GITHUB_OUTPUT"
>       # Restoring is allowed on every event. Saving a pass is also allowed on pull requests: GitHub scopes a cache
>       # to the branch that saved it, so a pull request's entries serve only that branch (and branches made from it)
>       # and the default branch's shared entries are written only from the default branch.
>       - name: Look up an earlier pass
>         id: ci-pass
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: ci-pass
>           key: ${{ steps.fingerprint.outputs.key }}
>       # A hit only counts if the restored marker names this very fingerprint.
>       - name: Decide whether the earlier pass can be reused
>         id: gate
>         env:
>           FINGERPRINT: ${{ steps.fingerprint.outputs.fingerprint }}
>         run: '"$XTASK" ci-cache check ci-pass "$FINGERPRINT" --output "$GITHUB_OUTPUT"'
>       - name: Report the reused pass (build and checks skipped)
>         if: steps.gate.outputs.skip == 'true'
>         env:
>           FINGERPRINT: ${{ steps.fingerprint.outputs.fingerprint }}
>           MATCHED_KEY: ${{ steps.ci-pass.outputs.cache-matched-key }}
>         run: '"$XTASK" ci-cache report "$FINGERPRINT" "$MATCHED_KEY" --summary "$GITHUB_STEP_SUMMARY"'
>       - name: Restore the toolchain cache
>         if: steps.gate.outputs.skip != 'true'
>         id: cache-toolchain
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: ~/.rustup/toolchains
>           key: rustup-${{ hashFiles('rust-toolchain.toml') }}
>       - name: Restore the registry cache
>         if: steps.gate.outputs.skip != 'true'
>         id: cache-registry
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: |
>             ~/.cargo/registry
>             ~/.cargo/git
>           key: registry-${{ hashFiles('Cargo.lock') }}
>           restore-keys: registry-
>       - name: Restore the target cache
>         if: steps.gate.outputs.skip != 'true'
>         id: cache-target
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: target
>           key: target-checks-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('Cargo.lock') }}
>           restore-keys: target-checks-${{ hashFiles('rust-toolchain.toml') }}-
>       - name: Report the cache outcome (exact, restore key, miss)
>         if: steps.gate.outputs.skip != 'true'
>         env:
>           TOOLCHAIN_HIT: ${{ steps.cache-toolchain.outputs.cache-hit }}
>           TOOLCHAIN_KEY: ${{ steps.cache-toolchain.outputs.cache-matched-key }}
>           REGISTRY_HIT: ${{ steps.cache-registry.outputs.cache-hit }}
>           REGISTRY_KEY: ${{ steps.cache-registry.outputs.cache-matched-key }}
>           TARGET_HIT: ${{ steps.cache-target.outputs.cache-hit }}
>           TARGET_KEY: ${{ steps.cache-target.outputs.cache-matched-key }}
>         run: |
>           outcome() {
>             if [ "$1" = "true" ]; then echo "exact"
>             elif [ -n "$2" ]; then echo "restore key"
>             else echo "miss"; fi
>           }
>           {
>             echo "cach
> [clipped: 10957 more characters]
> ```
> 
> ## Evidence
> 
> ### 1. path .github/workflows/ci.yml
> ```
> # Athena's CI (docs/design/athena-v0.md §11.2): a thin call into Rust. Both jobs check out the full
> # history and start with `./setup.sh --ci`. Actions are pinned by commit SHA, the token is read-only, and
> # nothing is deployed. Tests that call real models are opt-in and never run here.
> #
> # Cache layers (docs/design/ci-routing.md §4.1): toolchain, registry and target, keyed on the content of
> # rust-toolchain.toml and Cargo.lock. Every run restores them; only a push to the default branch saves them, so
> # an unreviewed branch cannot fill or poison the shared entries. Each job reports exact, restore key or miss.
> #
> # The 'fmt, clippy, tests' job also reuses its own earlier pass (docs/decisions.md D121): before setup it looks up
> # `ci-pass-<governed fingerprint>`; a marker naming that fingerprint skips setup and the checks. The marker is
> # saved only after the checks pass, on pull-request branches too (entries stay on the branch that saved them).
> name: ci
> 
> on:
>   # Push runs only for main. A push to a pull request's branch ran the same checks a second time for the same
>   # commit, in the push cache scope, which misses the pull_request run's cached pass. On PR #185 the pull_request
>   # run passed from the cache in 26 s, while two push runs of the same commit each ran the full 16 minutes
>   # (timings from the request; docs/decisions.md D125). The pull_request run satisfies the required check.
>   push:
>     branches: [main]
>   pull_request:
>   workflow_dispatch:
>   schedule:
>     # Weekly, so that new gitleaks rules reach old commits, and so the default branch's cache entries stay fresh.
>     - cron: "17 6 * * 1"
> 
> permissions:
>   contents: read
> 
> jobs:
>   checks:
>     name: fmt, clippy, tests
>     runs-on: ubuntu-24.04
>     steps:
>       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
>         with:
>           fetch-depth: 0
>           persist-credentials: false
>           # For pull requests, check the commits that would land, not GitHub's merge commit.
>           ref: ${{ github.event.pull_request.head.sha || github.sha }}
>       # Reuse of an earlier pass (docs/decisions.md D121). The fingerprint is the governed content of the checked-out
>       # tree (records/ and owner paths left out). xtask is built with the runner's own stable cargo, into a
>       # separate target directory, so this step needs neither the pinned toolchain nor setup.
>       - name: Fingerprint the governed content
>         id: fingerprint
>         env:
>           CARGO_TARGET_DIR: ${{ runner.temp }}/xtask-target
>         run: |
>           cargo +stable build --locked -q -p xtask
>           xtask="$CARGO_TARGET_DIR/debug/xtask"
>           echo "XTASK=$xtask" >> "$GITHUB_ENV"
>           fingerprint="$("$xtask" fingerprint)"
>           echo "fingerprint=$fingerprint" >> "$GITHUB_OUTPUT"
>           echo "key=ci-pass-${fingerprint#sha256:}" >> "$GITHUB_OUTPUT"
>       # Restoring is allowed on every event. Saving a pass is also allowed on pull requests: GitHub scopes a cache
>       # to the branch that saved it, so a pull request's entries serve only that branch (and branches made from it)
>       # and the default branch's shared entries are written only from the default branch.
>       - name: Look up an earlier pass
>         id: ci-pass
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: ci-pass
>           key: ${{ steps.fingerprint.outputs.key }}
>       # A hit only counts if the restored marker names this very fingerprint.
>       - name: Decide whether the earlier pass can be reused
>         id: gate
>         env:
>           FINGERPRINT: ${{ steps.fingerprint.outputs.fingerprint }}
>         run: '"$XTASK" ci-cache check ci-pass "$FINGERPRINT" --output "$GITHUB_OUTPUT"'
>       - name: Report the reused pass (build and checks skipped)
>         if: steps.gate.outputs.skip == 'true'
>         env:
>           FINGERPRINT: ${{ steps.fingerprint.outputs.fingerprint }}
>           MATCHED_KEY: ${{ steps.ci-pass.outputs.cache-matched-key }}
>         run: '"$XTASK" ci-cache report "$FINGERPRINT" "$MATCHED_KEY" --summary "$GITHUB_STEP_SUMMARY"'
>       - name: Restore the toolchain cache
>         if: steps.gate.outputs.skip != 'true'
>         id: cache-toolchain
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: ~/.rustup/toolchains
>           key: rustup-${{ hashFiles('rust-toolchain.toml') }}
>       - name: Restore the registry cache
>         if: steps.gate.outputs.skip != 'true'
>         id: cache-registry
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: |
>             ~/.cargo/registry
>             ~/.cargo/git
>           key: registry-${{ hashFiles('Cargo.lock') }}
>           restore-keys: registry-
>       - name: Restore the target cache
>         if: steps.gate.outputs.skip != 'true'
>         id: cache-target
>         uses: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684 # v4.2.3
>         with:
>           path: target
>           key: target-checks-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('Cargo.lock') }}
>           restore-keys: target-checks-${{ hashFiles('rust-toolchain.toml') }}-
>       - name: Report the cache outcome (exact, restore key, miss)
>         if: steps.gate.outputs.skip != 'true'
>         env:
>           TOOLCHAIN_HIT: ${{ steps.cache-toolchain.outputs.cache-hit }}
>           TOOLCHAIN_KEY: ${{ steps.cache-toolchain.outputs.cache-matched-key }}
>           REGISTRY_HIT: ${{ steps.cache-registry.outputs.cache-hit }}
>           REGISTRY_KEY: ${{ steps.cache-registry.outputs.cache-matched-key }}
>           TARGET_HIT: ${{ steps.cache-target.outputs.cache-hit }}
>           TARGET_KEY: ${{ steps.cache-target.outputs.cache-matched-key }}
>         run: |
>           outcome() {
>             if [ "$1" = "true" ]; then echo "exact"
>             elif [ -n "$2" ]; then echo "restore key"
>             else echo "miss"; fi
>           }
>           {
>             echo "cach
> [clipped: 10957 more characters]
> ```
> 
> ### 2. job 20261026-232420-ctt9
> Request:
> ```
> Priority: high
> Structure pass job R0 (docs/design/structure-pass.md, section 4). Add the one characterization test: a fake job through produce_and_verify asserting the order of ui.line output and the invocation numbers. Edits no other test. No behavior change. 
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run.rs, CHANGELOG.md
> ```
> Read as: Add produce_and_verify ordering characterization test (R0) (tests). Add one new test in crates/athena-core/src/run.rs that drives a fake job through produce_and_verify and asserts the order of ui.line output and the invocation sequence numbers. Add a CHANGELOG entry. No behavior change, no other test edited. The audience is the structure-pass owner, who needs R5-R8 guarded on ordering.
> Outcome: landed. verified and landed
> 
> 
> ### 3. job 20261026-232356-ds51
> Request:
> ```
> Priority: high
> Structure pass job G1 (docs/design/structure-pass.md, section 4). Make gh.rs the directory gh/ (rename to gh/mod.rs), update the D87/D88 paths in introspect.rs per section 5.2 with a test that a landing touching a moved file still selects D87 and D88, and change the external_calls_guard WRAPPER constant to the gh/ directory prefix (approved). Also mark the structure-pass decision in docs/decisions.md accepted, decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26): module names and order approved, R0 approved, all jobs queued now with dependencies, the guard constant may change to the prefix. No behavior change. 
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh.rs, crates/athena-core/src/gh/mod.rs, crates/athena-core/src/introspect.rs, crates/athena-core/tests/external_calls_guard.rs, docs/decisions.md, CHANGELOG.md
> ```
> Read as: Make gh.rs the gh/ directory (G1) and accept the structure-pass decision (code). Job G1 of the structure pass: rename gh.rs to gh/mod.rs with no code change. Point the D87/D88 `paths` in introspect.rs and the guard's WRAPPER constant at the `crates/athena-core/src/gh/` prefix. Mark D138 accepted in docs/decisions.md and add a CHANGELOG entry. The audience is the owner and the verifier, who check the section 5 proof. The reader found no matcher for `DecisionHeader.paths` in the code, which is the main uncertainty.
> Outcome: landed. verified and landed
> 
> 
> ### 4. job 20261025-171957-p5ns
> Request:
> ```
> Priority: high
> Label Jev's class answers from evidence, not opinion (BRIEF.md addendum of 2026-10-25, 'Evidence should label jev imo'). athena score-jev needs labeled introspections (at least 10 in its test part) and has none: 187 introspections carry a recorded Jev shadow answer, and the only labeling path today is a person's athena review. Derive labels from what happened afterwards, each with the records it rests on: (1) the retry that followed the introspection's proposed request and prevention landed, or failed for the same cause; (2) after a prevention for the class landed, the class stopped or kept recurring; (3) the critic's or verifier's findings on the same job name the same cause. Write each derived label as a class-label record marked evidence-derived with its basis, leave cases with no evidence unlabeled, and let athena review still override. Then score Jev with score-jev.
> Acceptance:
> - A test with fixture records shows each of the three evidence kinds producing a label with its basis, and a case with no evidence left unlabeled.
> - Labels are records (append-only); a person's review label takes precedence.
> - Run on this repository's records, score-jev reports how many labels were derived and, if enough, a score and verdict.
> Files: crates/athena-core/src/introspect.rs, crates/athena-core/src/jev.rs, crates/athena-data/src/records.rs, crates/athena-cli/src/main.rs, CHANGELOG.md, docs/decisions.md
> ```
> Read as: Derive evidence-based class labels for Jev scoring (code). Add an evidence-derived labeling path that writes append-only class-label records (labeler "evidence", with a basis citing record ids) from three kinds of later evidence: retry outcome after the proposed request/prevention, class recurrence after prevention, and critic/verifier findings naming the same cause. Cases without evidence stay unlabeled, a person's review label always wins, and score-jev reports the derived count and a score and verdict if at least 10 test rows exist. The audience is the maintainer running athena score-jev.
> Outcome: landed. verified and landed
> 
> 
> ### 5. job 20261026-083650-jhde
> Request:
> ```
> Priority: urgent
> The disk guard (9102f8f, default minimum 10 GB free) makes tests depend on the machine's real free space: athena-cli tests/binary.rs a_nested_run_works_in_a_scratch_copy_with_fake_agents failed in job 20261026-064950-38jx with 'environment: low disk: 9.0 GB free, 10.0 GB required', because the job's own build had used about 19 GB of the session's allowance. Tests must not depend on real free disk: let the minimum be set through the test fixtures' config (for example 0 in test configs) or inject the free-space reading, so the guard is still tested with simulated values and nested runs in tests are unaffected.
> Acceptance:
> - The nested-run test passes with free disk below 10 GB (shown by a test that simulates low free space for the nested run's environment and still passes because the test config sets the minimum to 0).
> - The disk guard's own tests still show a hold and a fail-fast with simulated low space.
> Files: crates/athena-core/src/disk.rs, crates/athena-cli/tests/common/mod.rs, crates/athena-cli/tests/binary.rs, CHANGELOG.md
> ```
> Read as: Make tests independent of real free disk by setting the minimum to 0 (tests). The shared test config (test_config in tests/common/mod.rs) gets [disk] min_free_gb = 0, so CLI tests, including the nested-run test, no longer depend on the machine's free space. A new test makes the nested run see low free space (a minimum above any real free space is not simulated; the reading is injected or the guard is exercised with simulated values) and shows it still passes. The disk guard's own tests keep showing a hold and a fail-fast with simulated values. The audience is the maintainers and verifier running cargo test on small-disk sandboxes.
> Outcome: landed. verified and landed
> 
> 
> ### 6. job 20261025-202447-xgk9
> Request:
> ```
> try again on 20261025-190606-eebk
> ```
> Read as: Retry test-suite speedup: keep prior changes, suite under 7 minutes (tests). Retry of job 20261025-190606-eebk, following the user's latest reply: keep the prior approach, including the secrets.rs scan memo and the shorter run_check poll schedule (both allowed, each with a test proving unchanged behavior). The target is the full `cargo test --locked` under 7 minutes on the job's machine, with per-binary before/after times measured the same way and recorded in docs/decisions.md. Audience: the user, who reads docs/decisions.md and later job times.
> Outcome: abandoned. closed: Superseded: the test-suite speedup landed narrowed as ph40 (8cab024) with a measured-improvement criterion.
> 
> 
> ### 7. job 20261024-191951-4pqw
> Request:
> ```
> Build job 4, part 1, of the records-history plan (docs/design/records-history.md sections 5.1 and 10, decision D111): a pre-merge check on a pull request's head, as a command (for example athena check --pr-head <base> <head>) that passes only when every commit between the base and the head classifies (via land.rs pr_commits) as a job landing whose verification passed and whose prediction precedes its outcome, a declared intervention, or records-only, and the head's content equals the last job landing's verified content plus only records-only commits after it; otherwise it fails naming each offending commit. Add it to .github/workflows/ci.yml as a new job on pull requests that does not block merging yet (it becomes required at the cutover, job 7), and list .github/workflows/ci.yml as a governance deliverable. Tests: a head with an unverified commit fails; a head shaped like a real Athena pull request passes; a head with a declared intervention after the job landing fails, since a squash of it is not a valid landing until the cutover.
> ```
> Read as: Add `athena check --pr-head` pre-merge check and non-blocking CI job (code). Adds a command (`athena check --pr-head <base> <head>`) that classifies every commit in base..head with `land::pr_commits` and passes only if each is a verified job landing with prediction before outcome, a declared intervention, or records-only, and the head's content equals the last job landing's verified content plus records-only commits. A new non-required pull_request job in ci.yml runs it. The audience is maintainers and CI reviewing Athena PRs before the job-7 cutover.
> Outcome: landed. verified and landed

## Reading (accepted by the user)

The first CI run of a PR sees the job's candidate commit before closing records exist, so `athena check --pr-head` and `--history` fail it. The change makes both judge a candidate-shaped head as pending (pass, with a visible note) and still fully judge the landing head, so both checks can become required at cutover. Audience: maintainers and CI reviewing Athena PRs.

Assumptions:
- Code survey: read crates/athena-core/src/pr_check.rs (check_pr_head, classification via land.rs pr_commits), crates/athena-core/src/reconcile.rs (history check), crates/athena-cli/src/main.rs (the `check --pr-head` and `--history` entry points), crates/athena-cli/src/display.rs (report printing), crates/athena-cli/tests/history_pr_head.rs and crates/athena-cli/tests/reconcile.rs (existing history and pr-head tests). The pending state is measured directly by calling check_pr_head and the CLI on fixture heads, not through a stand-in. No new record type or data file is added under crates/, so no record validation allow-list or AnyRecord change is needed.
- Previous jobs: 20261024-191951-4pqw added the `--pr-head` check and its non-blocking CI job; this job extends its rule for candidate heads and keeps the landing-head checks as they were. Jobs 20261026-232420-ctt9 (R0 characterization test), 20261026-232356-ds51 (G1 gh/ rename) and 20261025-171957-p5ns (evidence labels) are cited only as evidence and are already landed; their findings (duplicated test helpers in run.rs, stale gh.rs paths in docs, rerun label growth) do not touch this check and are not addressed here. Their changed files are kept as landed and excluded below.
- Job 20261026-232420-ctt9 and the others involve no change to ci.yml: the fix is in the Rust check, so .github/workflows/ci.yml, athena.toml and defaults.toml are not changed. The checks stay non-required until cutover.
- Governance: rust-toolchain.toml and setup.sh are named by the proposal's source only as part of CI setup. This job does not change either one; they are out of scope.
- The change makes the check accept more heads (candidate-shaped heads pass as pending) rather than refuse more, so no newly-refused-readings rate applies.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-core pr_check`: a test whose head is shaped like the early-PR candidate (governed work plus the job's candidate record, no verification or outcome) makes check_pr_head pass as pending, with the pending state in the report.
2. [behavior] Run `cargo test --locked -p athena-cli --test history_pr_head`: a test shows the history check on a candidate-shaped PR head, with main and the records branch present, passes as pending.
3. [behavior] Run `cargo test --locked`: non-candidate heads still fail naming each offending commit (governed change with no job records, failed verification, declared intervention after the job landing); existing pr_check and history_pr_head tests pass.
4. [behavior] Run `cargo test --locked`: once verification and outcome records are on top of the candidate, the landing head is fully judged and passes; a wrong verified fingerprint fails.
5. [content] `athena check --pr-head` prints a distinct pending line (Grep crates/athena-cli/src/display.rs or pr_check.rs) so a CI log reader can tell pending from verified.
6. [content] docs/design/records-history.md §5.1/§10 and CHANGELOG.md describe the pending treatment and why the landing head is still fully checked before merge.
7. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both pass.
8. [content] git diff shows rust-toolchain.toml, setup.sh, athena.toml, defaults.toml and .github/workflows/ci.yml unchanged.

## Expected deliverables

- crates/athena-core/src/pr_check.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/history_pr_head.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- rust-toolchain.toml and setup.sh: not changed by this job
- athena.toml, defaults.toml and .github/workflows/ci.yml: not changed; making the checks required is the cutover job
- Making the checks required at cutover
- Landed work of jobs 20261026-232420-ctt9, 20261026-232356-ds51 and 20261025-171957-p5ns: crates/athena-core/src/run.rs, crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh.rs, crates/athena-core/src/introspect.rs, crates/athena-core/tests/external_calls_guard.rs, crates/athena-core/src/evidence_labels.rs, crates/athena-core/src/labeled.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/review.rs, crates/athena-core/src/jev.rs, crates/athena-data/src/records.rs, crates/athena-cli/tests/labeled.rs, README.md, docs/records.md, docs/design/defaults-loop.md, docs/system-facts.md, docs/architecture.md are kept as landed and not edited
- Loosening checks for non-candidate heads

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check pending versus failing outcomes by running tests on fixture heads.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests plus fmt and clippy criteria.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named cargo test commands.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by the check itself and tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

26 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked -p athena-core pr_check`: a test whose head is shaped like the early-PR candidate (governed work plus the job's candidate record, no verification or outcome) makes check_pr_head pass as pending, with the pending state in the report.
Run `cargo test --locked -p athena-cli --test history_pr_head`: a test shows the history check on a candidate-shaped PR head, with main and the records branch present, passes as pending.
Run `cargo test --locked`: non-candidate heads still fail naming each offending commit (governed change with no job records, failed verification, declared intervention after the job landing); existing pr_check and history_pr_head tests pass.
Run `cargo test --locked`: once verification and outcome records are on top of the candidate, the landing head is fully judged and passes; a wrong verified fingerprint fails.
`athena check --pr-head` prints a distinct pending line (Grep crates/athena-cli/src/display.rs or pr_check.rs) so a CI log reader can tell pending from verified.
docs/design/records-history.md §5.1/§10 and CHANGELOG.md describe the pending treatment and why the landing head is still fully checked before merge.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both pass.
git diff shows rust-toolchain.toml, setup.sh, athena.toml, defaults.toml and .github/workflows/ci.yml unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
