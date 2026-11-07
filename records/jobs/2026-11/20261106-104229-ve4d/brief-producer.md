# Job 20261106-104229-ve4d: Finish ETXTBSY fix: widen exec-write scan, retry on busy, reconcile D08

## Request (verbatim)

> Finish the ETXTBSY fix by designing it from the root cause, using the proposed_request in records/introspections/2026-11/20261106-104005-m047.json. First state the root cause in the CHANGELOG entry: an executable is written and run in the same process while a write handle or forked child still holds it. Search for set_permissions, from_mode, set_mode, set_executable and chmod, and list every place in tests, xtask and athena-core that writes a stand-in script or marks it executable. Send all of them through the shared athena-testutil helper, which creates the file and sets its mode in a child process. Where production code runs a stand-in, make it retry briefly on ETXTBSY. Widen crates/athena-testkit/tests/no_in_process_exec_writes.rs so it flags wrappers and an external chmod outside the helper. Add fixtures proving the scan catches both. If the change touches files that D08 covers, reconcile D08 in docs/decisions.md and docs/decision-links.toml. Acceptance: the scan fails on the wrapper and chmod fixtures and passes on the helper, and the full test suite passes.
> 
> Acceptance:
> - The change is made as described: Four jobs in a row failed to fix the ETXTBSY ("text file busy") test flake; each one guarded only the last symptom it saw
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-11/20261106-104005-m047.json., crates/athena-testkit/tests/no_in_process_exec_writes.rs, docs/decisions.md, docs/decision-links.toml.

## Reading (accepted by the user)

Goal: stop the recurring "Text file busy" test flake at its root for the maintainers and verifier. Widen the `no_in_process_exec_writes` scan to flag wrappers and external chmod outside the helper, add fixtures, route any remaining stand-in writes through the shared helper, add a short ETXTBSY retry where production code runs a stand-in, and state the root cause in CHANGELOG. The shared helper already exists as `athena_testkit::write_executable`; the request's "athena-testutil" is read as that crate.

Assumptions:
- Code survey: read crates/athena-testkit/src/lib.rs (write_executable runs sh in a child: cat, chmod 755, mv), crates/athena-testkit/tests/no_in_process_exec_writes.rs (pure `violations(rel, text)` over test regions, EXEMPT list, tree scan of crates/*/{src,tests} and xtask), crates/athena-core/src/toolconfig.rs, crates/athena-git/src/snapshot.rs, crates/athena-cli/src/slot_exec.rs, xtask/src/main.rs, crates/athena-agent/src/claude.rs (existing ETXTBSY retry, 20 attempts). No records or data files are added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly by calling `violations` on fixture strings and on the real tree, not through a stand-in.
- Existing code: extends crates/athena-testkit/src/lib.rs, crates/athena-testkit/tests/no_in_process_exec_writes.rs and crates/athena-agent/src/claude.rs (its ETXTBSY retry); reads docs/decisions.md D08, D157, D160 and docs/decision-links.toml.
- Survey result: set_permissions/from_mode/set_mode/set_executable sites are in crates/athena-core/src/{toolconfig,pin,snapshot}.rs, crates/athena-git/src/snapshot.rs, crates/athena-cli/src/slot_exec.rs, crates/athena-testkit/tests/stand_in_stress.rs and xtask/src/main.rs (set_executable at lines 415, 490, 550, 807). Several are production code that is not a test stand-in (pin and snapshot read-only toggles), so only sites that write and run a stand-in are converted. The producer must re-run the search and list each site in the D-entry.
- The scan currently flags direct calls but not a wrapper function defined in test code that hides set_permissions, nor a chmod run through `sh -c`, a shell script string, or `Command::new("chmod")` on a differently spelled line. Widening adds those shapes. The exemption list is kept honest by the existing every_exemption_still_exists test.
- Production sites that run a stand-in (xtask downloaded/adopted tools at xtask/src/main.rs, toolconfig.rs materialisation) get a brief bounded retry on libc ETXTBSY, modelled on claude.rs. Pure permission toggles are left alone.
- If the change touches crates/athena-git/src/snapshot.rs, a D08-governed path, a new decision entry in docs/decisions.md and a [[link]] with older D08 in docs/decision-links.toml are added; line numbers in existing links must stay correct. This is the same-repo local process; no CI, push or PR step is involved.
- The decision-log numbering continues after the latest entry; the producer picks the next free D-number. athena.toml and defaults.toml are unchanged.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry whose first sentence states the root cause: an executable is written and run in the same process while a write handle or forked child still holds it.
2. [content] docs/decisions.md has a new entry listing every searched site (set_permissions, from_mode, set_mode, set_executable, chmod) in tests, xtask and athena-core, and saying for each whether it was converted to athena_testkit::write_executable, given a retry, or left as a non-stand-in.
3. [content] No test, xtask or athena-core code outside athena_testkit writes a stand-in script or marks it executable in-process or via an external chmod; Grep for set_permissions, from_mode, set_mode, set_executable and chmod over crates/*/{src,tests} and xtask shows only the helper, the documented exemptions, and non-stand-in production uses.
4. [content] Production code that runs a stand-in retries briefly (bounded attempts, short sleep) on ETXTBSY; Grep shows the retry at each such run site, and the retry is bounded.
5. [content] crates/athena-testkit/tests/no_in_process_exec_writes.rs contains fixtures for a wrapper function that hides set_permissions and for an external chmod (shell string or Command) outside the helper, and a fixture for the helper path that is not flagged.
6. [behavior] Run `cargo test -p athena-testkit`: the scan's fixture tests show the wrapper and chmod fixtures are flagged, the helper fixture passes, and the real-tree scan test passes.
7. [behavior] Run `cargo test`: the full workspace suite passes, including the stress test and the exemption-staleness test.
8. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.
9. [content] If any D08-governed path (crates/athena-git/src/snapshot.rs, crates/athena-core/src/run/candidate.rs) is touched, docs/decisions.md has a new entry saying D08 stands or is amended, and docs/decision-links.toml has a [[link]] with older = "D08" and newer = that entry, with a correct line number.
10. [content] The scan change refuses more test code than before by design: the tree currently has no flagged lines outside EXEMPT, so expected 0% of newly refused readings; no past readings are replayed through this scan.
11. [content] No unrelated behavior changes; no edits to athena.toml, defaults.toml, workflows, or records/.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-testkit/tests/no_in_process_exec_writes.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-core/src/toolconfig.rs (modify)
- crates/athena-testkit/src/lib.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-cli/src/slot_exec.rs (modify)
- crates/athena-testkit/tests/stand_in_stress.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml, workflows or records/.
- Converting read-only/writable permission toggles in crates/athena-core/src/pin.rs and snapshot.rs that do not write or run stand-ins.
- Editing the introspection record itself (it is read as input only).
- Reversing D08.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture tests for the scan; cargo test, clippy and fmt must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: CHANGELOG states the cause; every write site is routed through one helper rather than guarding the last symptom.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by the scan test, not prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the scan on fixtures and the tree.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry and CHANGELOG written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the scan fails on the bad fixtures and passes on the helper.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing helper and the claude.rs retry pattern rather than a new crate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-104229-ve4d/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-104229-ve4d/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md has a new entry whose first sentence states the root cause: an executable is written and run in the same process while a write handle or forked child still holds it.
docs/decisions.md has a new entry listing every searched site (set_permissions, from_mode, set_mode, set_executable, chmod) in tests, xtask and athena-core, and saying for each whether it was converted to athena_testkit::write_executable, given a retry, or left as a non-stand-in.
No test, xtask or athena-core code outside athena_testkit writes a stand-in script or marks it executable in-process or via an external chmod; Grep for set_permissions, from_mode, set_mode, set_executable and chmod over crates/*/{src,tests} and xtask shows only the helper, the documented exemptions, and non-stand-in production uses.
Production code that runs a stand-in retries briefly (bounded attempts, short sleep) on ETXTBSY; Grep shows the retry at each such run site, and the retry is bounded.
crates/athena-testkit/tests/no_in_process_exec_writes.rs contains fixtures for a wrapper function that hides set_permissions and for an external chmod (shell string or Command) outside the helper, and a fixture for the helper path that is not flagged.
Run `cargo test -p athena-testkit`: the scan's fixture tests show the wrapper and chmod fixtures are flagged, the helper fixture passes, and the real-tree scan test passes.
Run `cargo test`: the full workspace suite passes, including the stress test and the exemption-staleness test.
Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.
If any D08-governed path (crates/athena-git/src/snapshot.rs, crates/athena-core/src/run/candidate.rs) is touched, docs/decisions.md has a new entry saying D08 stands or is amended, and docs/decision-links.toml has a [[link]] with older = "D08" and newer = that entry, with a correct line number.
The scan change refuses more test code than before by design: the tree currently has no flagged lines outside EXEMPT, so expected 0% of newly refused readings; no past readings are replayed through this scan.
No unrelated behavior changes; no edits to athena.toml, defaults.toml, workflows, or records/.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
