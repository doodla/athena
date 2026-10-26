# Job 20261025-230352-ph40: verify "Memoize secret scans and shorten run_check polling; record timings"

## Request (verbatim)

> Priority: urgent
> Land the test-suite speedup that three attempts already achieved (jobs 20261025-190606-eebk, 20261025-202447-xgk9, 20261025-214509-y179, each rejected only for missing a fixed time target): two behavior-preserving changes, each with a test proving behavior is unchanged: (1) the secret scanner remembers its result for identical text within one process (crates/athena-gate/src/secrets.rs), since the same text is scanned many times per run; (2) run_check polls a running check on a shorter schedule (crates/athena-core/src/checks.rs). No fixed time target: measure the full cargo test --locked before and after on the job's machine and record both, measured, in docs/decisions.md (the earlier attempts measured Athena's test check at 444 to 466 s against about 720 s before), with nextest (584 s vs 673 s) as a rejected alternative.
> Acceptance:
> - A test shows the memo returns the same findings as a fresh scan for identical text and never for different text.
> - A test shows the poll schedule still detects completion and timeouts correctly.
> - docs/decisions.md records the before and after times, measured; no test is removed, weakened or filtered.
> Files: crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Land two behavior-preserving speedups: a per-process memo in Gitleaks::scan_text (crates/athena-gate/src/secrets.rs) and a shorter poll schedule in run_check (crates/athena-core/src/checks.rs, currently a fixed 200 ms sleep). Measure full cargo test --locked before and after, and record both in docs/decisions.md with nextest as a rejected alternative. The audience is the maintainers, who judge whether the change is safe and the timings are honest.

Assumptions:
- Poll schedule: start with short sleeps (e.g. 5 ms) that back off to the existing 200 ms cap; completion and timeout semantics are unchanged.
- Memo is bounded in entry count and text size, keyed by the exact text, and holds findings only (or a hash key) so text is not retained needlessly.
- Before/after timings are measured by running cargo test --locked on the job's machine; if the producer cannot run it, it must say so and not invent numbers.
- Earlier jobs' figures (444-466 s vs about 720 s; nextest 584 s vs 673 s) are cited as prior measurements, labelled as such, not as this job's measurements.
- No fixed time target; the earlier rejections for missing one do not apply.

## Acceptance criteria

1. [behavior] A test in secrets.rs shows scan_text on identical text twice returns findings equal to a fresh uncached scan, and different text never returns another text's findings (including secret-bearing then clean text). Run: cargo test -p athena-gate.
2. [behavior] A test in checks.rs shows the new poll schedule still detects a fast completion, a slow completion, and a timeout of a long-running check with the correct status. Run: cargo test -p athena-core checks.
3. [content] docs/decisions.md has a new decision recording full cargo test --locked wall time before and after, labelled as measured on the job's machine with the method stated, and nextest (584 s vs 673 s, prior measurement) as a rejected alternative with the reason.
4. [behavior] cargo test --locked passes in full, cargo clippy is clean and cargo fmt --check passes; no test is removed, ignored, weakened or filtered (git diff shows only added tests).
5. [content] CHANGELOG.md has an entry for both changes. Only the four listed files change, and no governance files change.

## Expected deliverables

- crates/athena-gate/src/secrets.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Per-binary test restructuring in athena-cli tests, common/mod.rs, or run.rs
- Adopting nextest
- Meeting any fixed time target
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the memo and the poll schedule; full cargo test and clippy pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Timings in docs/decisions.md are measured, or labelled as prior estimates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check unchanged behavior, not just that code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the four named files change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: nextest is recorded as a rejected alternative.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that findings and timeouts are unchanged, not that code was written.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A new unit test in secrets.rs clears the memo. It calls scan_text twice on a token text (fake_token) and compares both results to scan_text_uncached. It then scans clean text, which must be empty, and scans the token text again, which must match again. It also checks that a different config does not share entries. Run: `cargo test -p athena-gate`. Observes: The memoized and fresh findings are equal, and the clean text returns no findings after the secret-bearing text.
  Fails without the behavior: If the key ignored the text or the config, the clean text would return the token findings and the test would fail.
- Criterion 2: Three new tests in checks.rs use run_check. They run 'exit 0' (fast completion, passed, no deadline note). They run 'sleep 1' with a 30 s timeout (slow completion, passed). They run 'sleep 30' with a 300 ms timeout (exit_code None, not passed, note contains 'deadline', and elapsed is well under 30 s). Run: `cargo test -p athena-core checks`. Observes: Status, exit code and note are correct for each case.
  Fails without the behavior: If the backoff skipped try_wait or the deadline check, the completion or timeout assertions would fail or hang.
- Criterion 3: Time cargo test --locked on the base tree before editing code, then on the final tree, on the same machine. Record the wall times, command, and the machine and date in the new decisions.md entry. Cite the earlier jobs' figures and nextest as labelled prior measurements. Read the entry with git diff. Run: `git diff docs/decisions.md`. Observes: The entry has before and after wall times labelled measured, a stated method, and nextest 584 s vs 673 s as a prior measurement under rejected alternatives with its reason.
- Criterion 4: Run the full checks and inspect the diff for removed or altered test lines. Run: `cargo test --locked`. Observes: All tests pass. cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check are clean. git diff shows only added test lines.
  Fails without the behavior: If the memo or the poll changed behavior, existing tests such as the baseline timeout test or the gate tests would fail.
- Criterion 5: Check the changed file list and the CHANGELOG diff. Run: `git status`. Observes: Only the four listed files are modified, and CHANGELOG.md has an Unreleased entry for both changes.

Planned files:

- crates/athena-gate/src/secrets.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base d20ec166cd38 → candidate 63a005dff177. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  9 ++++
 crates/athena-core/src/checks.rs  | 45 +++++++++++++++++++-
 crates/athena-gate/src/secrets.rs | 89 +++++++++++++++++++++++++++++++++++++++
 docs/decisions.md                 | 35 +++++++++++++++
 4 files changed, 177 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1241 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
