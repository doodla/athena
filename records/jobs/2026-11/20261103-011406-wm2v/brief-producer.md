# Job 20261103-011406-wm2v: Split athena-core --lib into filtered parts across CI test shards

## Request (verbatim)

> try again on 20261103-005128-nfvz

## Reading (accepted by the user)

Retry of job 20261103-005128-nfvz (dispatched, never verified, no findings, no candidate files): teach `xtask test-shard` to split one test binary into parts by test-name module prefix, and split athena-core's `--lib` (821 tests, 255.11s) into parts that land on at least two of the six shards, each test running exactly once. Audience: CI maintainers and the verifier judging shard balance and exactly-once coverage.

Assumptions:
- Retry of job 20261103-005128-nfvz: it left no findings and no candidate files, so there is nothing to fix or keep; the work starts from the base and its six deliverables and out_of_scope items are carried over unchanged.
- Code Survey: Code survey: crates/athena-core/src/lib.rs (the --lib binary whose test names are module paths such as admission::tests::..., gh::tests::records_flow::..., run_pin_tests::...; run_pin_tests and truncate_tests are top-level modules). Entry point is xtask/src/shard.rs: plan_units builds Units from cargo metadata, assign balances them by timings, command runs `cargo test --locked -p <pkg> <selector>`. No record or data file is added under crates/, so check_record_file/AnyRecord are not touched.
- Design: a Unit gains an optional filter (module prefixes, or a catch-all that skips the other parts' prefixes via `--skip`). The filter is part of Unit::line() so timings keys, --dry-run and the run command all agree; parts are sorted deterministically. The part list for athena-core --lib is declared in xtask/src/shard.rs, and plan_units expands that one unit into parts.
- Prefix grouping is chosen so that part estimates (from the given 255.11s total, apportioned by test count per module group) are balanced; all estimates are labeled as estimates from run 41469096430. Largest modules by test count (gh, reading, run, hestia, verify, plan, disk, checks) go in separate parts; a skip-based catch-all part covers everything else including future modules.
- The coverage test in xtask/tests/shard_parts.rs obtains names from `cargo test -p athena-core --lib -- --list` (or an embedded snapshot of that list) and checks each name matches exactly one part's filter under libtest substring/--skip semantics; a negative test mutates the part set (drops or duplicates a prefix) and expects rejection.
- Other xtask/tests/ci_shards.rs checks are updated so they accept filtered units. No workflow file or athena.toml change is needed because the number of shards stays six.

## Acceptance criteria

1. [content] xtask/src/shard.rs lets a unit carry a test-name filter (module prefixes plus a skip-based catch-all), prints it in the --dry-run line, and runs it as `cargo test --locked -p athena-core --lib -- <filter>`; Grep of xtask/shard-timings.txt shows athena-core --lib as at least two part lines, not one.
2. [behavior] Run `cargo run -p xtask -- test-shard K 6 --dry-run` for K=0..5: athena-core --lib parts appear in at least two different shards, no part appears twice, and every other target appears exactly once across the shards.
3. [behavior] Run `cargo test -p xtask --test shard_parts` and `cargo test -p xtask --test ci_shards`: a test checks every name from `cargo test -p athena-core --lib -- --list` matches exactly one part's filter, and a negative test shows a dropped or duplicated prefix is rejected.
4. [content] xtask/shard-timings.txt lists the athena-core parts with estimated seconds summing to about 255s, notes they are estimates from run 41469096430, and no shard's estimated total exceeds roughly the mean plus the largest remaining single unit (lifecycle, 143.22s).
5. [content] CHANGELOG.md has an entry with the before numbers: athena-core --lib one binary of 821 tests at 255.11s (97% of shard 0's test time), shard 0 test step 5m45s vs shard 4 1m45s, and the next slowest binaries lifecycle 143.22s, drain 79.32s, retry 49.45s, job_plan 46.35s.
6. [behavior] Run `cargo test -p xtask`: unit tests in xtask/src/shard.rs for filtered units (assignment keeps every part exactly once, deterministic part ordering, timings round-trip with a filtered selector) pass.

## Expected deliverables

- xtask/src/shard.rs (modify)
- xtask/shard-timings.txt (modify)
- xtask/tests/ci_shards.rs (modify)
- CHANGELOG.md (modify)
- xtask/tests/shard_parts.rs (create)
- xtask/src/main.rs (modify)

## Out of scope

- Changing the number of shards or .github/workflows/ci.yml
- Adopting cargo-nextest
- Speeding up or editing the athena-core tests themselves
- Fetching anything from GitHub or re-measuring timings
- Splitting other binaries such as lifecycle.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests in shard.rs and the new shard_parts.rs test cover the change.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run test-shard --dry-run and the coverage test rather than checking that wiring exists.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Timings are labeled estimates from run 41469096430.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the sharding tool; other binaries are not split.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the CHANGELOG entry explain the filters for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 20 cases in 18 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make the disk guard and build cleanup one ordered step. Before the free-space check, and again on any low-disk hold, Athena removes the regenerable builds of held, interrupted and finished jobs, then measures free space. Only the unfreeable shortfall holds a job. Held jobs resume automatically once ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

25 more qualifying classes are left out.

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

xtask/src/shard.rs lets a unit carry a test-name filter (module prefixes plus a skip-based catch-all), prints it in the --dry-run line, and runs it as `cargo test --locked -p athena-core --lib -- <filter>`; Grep of xtask/shard-timings.txt shows athena-core --lib as at least two part lines, not one.
Run `cargo run -p xtask -- test-shard K 6 --dry-run` for K=0..5: athena-core --lib parts appear in at least two different shards, no part appears twice, and every other target appears exactly once across the shards.
Run `cargo test -p xtask --test shard_parts` and `cargo test -p xtask --test ci_shards`: a test checks every name from `cargo test -p athena-core --lib -- --list` matches exactly one part's filter, and a negative test shows a dropped or duplicated prefix is rejected.
xtask/shard-timings.txt lists the athena-core parts with estimated seconds summing to about 255s, notes they are estimates from run 41469096430, and no shard's estimated total exceeds roughly the mean plus the largest remaining single unit (lifecycle, 143.22s).
CHANGELOG.md has an entry with the before numbers: athena-core --lib one binary of 821 tests at 255.11s (97% of shard 0's test time), shard 0 test step 5m45s vs shard 4 1m45s, and the next slowest binaries lifecycle 143.22s, drain 79.32s, retry 49.45s, job_plan 46.35s.
Run `cargo test -p xtask`: unit tests in xtask/src/shard.rs for filtered units (assignment keeps every part exactly once, deterministic part ordering, timings round-trip with a filtered selector) pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
