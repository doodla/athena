# Job 20261102-102740-qx57: Cache gitleaks and athena binary in CI with complete input keys

## Request (verbatim)

> try again on 20261031-172040-kq9v

## Reading (accepted by the user)

Retry of job 20261031-172040-kq9v: redo the CI caches for gitleaks and the athena binary, with the athena-bin key and the setup --ci skip digest covering every compiled-in file, derived from one shared list that a repo test checks. Audience: CI maintainers and the throughput review.

Assumptions:
- Job 20261031-172040-kq9v is rejected; its candidate is discarded and this job starts from the base. Its files .github/workflows/ci.yml, CHANGELOG.md, xtask/src/main.rs and xtask/tests/policy.rs are kept as deliverables, redone from scratch. xtask/tests/ci_shards.rs and setup.sh are dropped unless a test there needs updating (setup.sh's --ci path calls xtask setup, so it is likely unchanged).
- Findings of 20261031-172040-kq9v addressed: (major) the key and skip digest omitted prompts/*.md, docs/system-facts.md and other include_str! inputs, so a restored binary could be stale. Fixed by hashing crates/** and xtask/** plus rust-toolchain.toml, Cargo.toml, Cargo.lock, athena.toml, defaults.toml, setup.sh and docs/system-facts.md, in the key and in the digest. (minor) no test tied the hashFiles list to input_files: now a policy test compares them. (minor) stamp written only on --ci builds: now written on every build, or removed on a non-CI build. (minor) restored binary reports the older source commit: stated in the ci.yml comment and CHANGELOG, accepted since the CI jobs only run read-only checks. (nit) skip path bypassed the dirty-tree refusal: the skip applies only with --ci, and this is documented.
- Code survey: I read xtask/src/main.rs (main dispatches to setup(); setup runs numbered steps, step 4 is sccache, skipped under --ci; the build and adopt step is later), crates/athena-core/src/status.rs (BUILD_PATHS = crates/, xtask/, Cargo.toml, Cargo.lock, setup.sh), and the claim that crates/athena-context/src/lib.rs and briefs.rs include prompts and docs/system-facts.md with include_str!. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property (a compiled-in file change changes the key and the digest) is measured directly: the test hashes a temp tree, edits one file, and compares digests, and it parses the hashFiles globs from ci.yml.
- The shared list is one const in xtask (the athena build input globs). The digest function and a policy test both read it, and the test parses every athena-bin hashFiles in ci.yml and fails if any input path is not covered by a glob.
- The test enumerates compiled-in files independently by scanning include_str!/include_bytes! targets under crates/ and xtask/ plus the manifests, so a new compiled-in file outside the list fails the test.
- The gitleaks cache and the restore-before-setup ordering test from the original request are kept. Restore steps come before 'Set up (pinned toolchain, gitleaks, athena)' in fmt, clippy, test, history, attribution and pr-head jobs. Saves run on the default branch only, as today. All uses: lines stay pinned by a 40-character SHA.
- No rule is made stricter on readings, so no refusal-rate criterion applies.

## Acceptance criteria

1. [content] Grep of .github/workflows/ci.yml: every athena-bin cache key (all restore and save steps) uses hashFiles covering crates/**, xtask/**, rust-toolchain.toml, Cargo.toml, Cargo.lock, athena.toml, defaults.toml, setup.sh and docs/system-facts.md. In each of the fmt, clippy and test jobs, actions/cache/restore steps for gitleaks and the athena binary come before the step named 'Set up (pinned toolchain, gitleaks, athena)'.
2. [content] ci.yml has matching actions/cache/save steps with the existing condition (not a pull request, ref equal to the default branch, no exact cache hit). Every uses: line is pinned by a 40-character SHA. History, attribution and pr-head also restore before setup, or a comment states why they do not. No other workflow behavior changes.
3. [content] xtask has one shared list of athena build inputs that both the skip digest and the policy test read. The comment in ci.yml and the CHANGELOG do not claim more than the key covers.
4. [behavior] Run `cargo test -p xtask`. A test edits one compiled-in file (a prompt .md under a temp copy of the tree) and asserts the inputs digest changes. Another asserts the skip happens with a matching digest and cached binary and does not happen with a mismatched digest or no binary, and a third asserts the stamp is not left stale after a non-CI build.
5. [behavior] Run `cargo test -p xtask --test policy`. The new tests derive the compiled-in file set from include_str!/include_bytes! targets and the manifests, and fail if any athena-bin hashFiles key in ci.yml misses one. They also fail on a ci.yml variant with a restore step removed or placed after the setup step.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass, including xtask/tests/ci_shards.rs and the existing policy tests.
7. [content] CHANGELOG.md has a new entry describing the gitleaks and athena binary caches, their keys, and that the speed-up on GitHub Actions is not measured here.

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/src/main.rs (modify)
- xtask/tests/policy.rs (modify)
- CHANGELOG.md (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- xtask/tests/ci_shards.rs (modify)

## Out of scope

- Measuring the real time saved on GitHub Actions; the later throughput review does that.
- Changing the toolchain, registry or target caches, the gate and pass logic, or the shard layout.
- Changing athena.toml, defaults.toml, tool pins or action SHAs (they are only hashed, not edited).
- Changing the build commit that a restored binary reports.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask tests; cargo test and clippy -D warnings must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A policy test enforces key coverage and restore-before-setup order.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The key and digest derive from one shared list, not a patched list.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests edit a compiled-in file and check the digest changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the cargo commands.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments and the CHANGELOG are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261031-172040-kq9v

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> Grep of .github/workflows/ci.yml: in each of the fmt, clippy and test jobs, an actions/cache/restore step for gitleaks and one for the athena binary come before the step named 'Set up (pinned toolchain, gitleaks, athena)'. Each key uses hashFiles over the files that decide the install. Every `uses:` line is still pinned by a 40-character SHA.: Restore order and pinning are met. The policy test every_job_restores_the_tool_caches_before_it_runs_the_setup_step passes, and workflow_actions_are_pinned in ci_shards.rs passes. A grep found all 44 `uses: actions/cache*` lines pinned to 5a3ec84eff668545956fd18022155c47e93e2684. The gitleaks key (athena.toml plus OS and arch) is right. The athena key is not over 'the files that decide the install': it hashes only 'crates/**/*.rs', 'crates/**/Cargo.toml', the xtask equivalents and five root files. The binary also embeds crates/athena-context/prompts/*.md (lib.rs:35-65), crates/athena-cap-change/prompts and crates/athena-cap-study/prompts (lib.rs:21-31) and docs/system-facts.md (briefs.rs:1230), none of which are in the key.

## Findings you must address

The verifier of job 20261031-172040-kq9v raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (.github/workflows/ci.yml athena-bin keys (all 6 restore and 3 save steps); xtask/src/main.rs input_files): The athena binary cache key and the skip digest leave out files that the binary compiles in, so CI can restore and use an out-of-date binary The key and input_files() keep only *.rs files, Cargo.toml files and five root files. The binary also embeds crates/athena-context/prompts/{contract,reader,producer,verifier,planner,critic}.md (crates/athena-context/src/lib.rs:35-65), crates/athena-cap-change/prompts/*.md, crates/athena-cap-study/prompts/*.md and docs/system-facts.md (crates/athena-context/src/briefs.rs:1230), all through include_str!. A commit that changes only a prompt leaves the key and the stamp unchanged, so the cache hits and setup --ci skips the build. History, attribution and pr-head then run `athena check` with a binary not built from the commit under test. The ci.yml comment and the CHANGELOG say 'a stale binary is never restored', which is false. The repo already defines the build inputs: BUILD_PATHS = [crates/, xtask/, Cargo.toml, Cargo.lock, setup.sh] in crates/athena-core/src/status.rs:20. Fix: hash 'crates/**' and 'xtask/**' plus rust-toolchain.toml, Cargo.toml, Cargo.lock, athena.toml, defaults.toml, setup.sh and docs/system-facts.md in both the key and inputs_digest, ideally from one shared list.

All of the verification's findings:

> major (.github/workflows/ci.yml athena-bin keys (all 6 restore and 3 save steps); xtask/src/main.rs input_files): The athena binary cache key and the skip digest leave out files that the binary compiles in, so CI can restore and use an out-of-date binary The key and input_files() keep only *.rs files, Cargo.toml files and five root files. The binary also embeds crates/athena-context/prompts/{contract,reader,producer,verifier,planner,critic}.md (crates/athena-context/src/lib.rs:35-65), crates/athena-cap-change/prompts/*.md, crates/athena-cap-study/prompts/*.md and docs/system-facts.md (crates/athena-context/src/briefs.rs:1230), all through include_str!. A commit that changes only a prompt leaves the key and the stamp unchanged, so the cache hits and setup --ci skips the build. History, attribution and pr-head then run `athena check` with a binary not built from the commit under test. The ci.yml comment and the CHANGELOG say 'a stale binary is never restored', which is false. The repo already defines the build inputs: BUILD_PATHS = [crates/, xtask/, Cargo.toml, Cargo.lock, setup.sh] in crates/athena-core/src/status.rs:20. Fix: hash 'crates/**' and 'xtask/**' plus rust-toolchain.toml, Cargo.toml, Cargo.lock, athena.toml, defaults.toml, setup.sh and docs/system-facts.md in both the key and inputs_digest, ideally from one shared list.
> minor (xtask/src/main.rs input_files doc comment; xtask/tests/policy.rs): The ci.yml hashFiles list and the Rust input_files list are kept in step by hand and no test checks them The doc comment says 'The CI cache key in .github/workflows/ci.yml hashes the same set', but nothing enforces it. A policy test could compare the patterns in the athena-bin key with the input set. The defaults model asks for rules enforced by mechanism.
> minor (xtask/src/main.rs setup(), step 5): Only a --ci build writes the stamp, so a later non-CI build can leave an out-of-date stamp next to a new binary write_stamp runs only when flags.ci is set. On a developer machine, run setup --ci, change the sources, then run a plain setup: the binary is rebuilt but athena.inputs still holds the old digest. If the sources go back to the old state and setup --ci runs again, it skips the build and keeps a binary built from different sources. Writing the stamp on every build_and_adopt, or deleting it on a non-CI build, removes this. CI runners start fresh, so this does not affect CI.
> minor (xtask/src/main.rs setup(); crates/athena-cli/src/wiring.rs:52): A restored binary reports the source commit of the run that built it build_info() embeds ATHENA_BUILD_COMMIT and is passed into Athena (wiring.rs:140) and `athena --version`. The candidate states this in ci.yml and the CHANGELOG. The CI jobs only run read-only checks, so the effect looks small, but any CI output that names the build commit will show an older commit.
> nit (xtask/src/main.rs setup()): The skip path never runs build_and_adopt's dirty-tree refusal cached_athena returns early before the `dirty && !flags.allow_dirty` check in build_and_adopt. This is harmless on CI checkouts but changes behaviour for a local `setup --ci` on a dirty tree.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 16 cases in 14 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena one disk lifecycle. Before the low-disk check and before any hold, the writer's regenerable build is removed once the writer has finished and its output is committed or recorded. The guard then measures free space after that removal. Where checks come from GitHub, the guard does not coun...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

24 more qualifying classes are left out.

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

The work starts from the rejected candidate of job 20261031-172040-kq9v: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Grep of .github/workflows/ci.yml: every athena-bin cache key (all restore and save steps) uses hashFiles covering crates/**, xtask/**, rust-toolchain.toml, Cargo.toml, Cargo.lock, athena.toml, defaults.toml, setup.sh and docs/system-facts.md. In each of the fmt, clippy and test jobs, actions/cache/restore steps for gitleaks and the athena binary come before the step named 'Set up (pinned toolchain, gitleaks, athena)'.
ci.yml has matching actions/cache/save steps with the existing condition (not a pull request, ref equal to the default branch, no exact cache hit). Every uses: line is pinned by a 40-character SHA. History, attribution and pr-head also restore before setup, or a comment states why they do not. No other workflow behavior changes.
xtask has one shared list of athena build inputs that both the skip digest and the policy test read. The comment in ci.yml and the CHANGELOG do not claim more than the key covers.
Run `cargo test -p xtask`. A test edits one compiled-in file (a prompt .md under a temp copy of the tree) and asserts the inputs digest changes. Another asserts the skip happens with a matching digest and cached binary and does not happen with a mismatched digest or no binary, and a third asserts the stamp is not left stale after a non-CI build.
Run `cargo test -p xtask --test policy`. The new tests derive the compiled-in file set from include_str!/include_bytes! targets and the manifests, and fail if any athena-bin hashFiles key in ci.yml misses one. They also fail on a ci.yml variant with a restore step removed or placed after the setup step.
`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass, including xtask/tests/ci_shards.rs and the existing policy tests.
CHANGELOG.md has a new entry describing the gitleaks and athena binary caches, their keys, and that the speed-up on GitHub Actions is not measured here.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
