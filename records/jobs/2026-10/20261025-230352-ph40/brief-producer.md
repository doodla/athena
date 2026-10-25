# Job 20261025-230352-ph40: Memoize secret scans and shorten run_check polling; record timings

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

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

A test in secrets.rs shows scan_text on identical text twice returns findings equal to a fresh uncached scan, and different text never returns another text's findings (including secret-bearing then clean text). Run: cargo test -p athena-gate.
A test in checks.rs shows the new poll schedule still detects a fast completion, a slow completion, and a timeout of a long-running check with the correct status. Run: cargo test -p athena-core checks.
docs/decisions.md has a new decision recording full cargo test --locked wall time before and after, labelled as measured on the job's machine with the method stated, and nextest (584 s vs 673 s, prior measurement) as a rejected alternative with the reason.
cargo test --locked passes in full, cargo clippy is clean and cargo fmt --check passes; no test is removed, ignored, weakened or filtered (git diff shows only added tests).
CHANGELOG.md has an entry for both changes. Only the four listed files change, and no governance files change.

## Report

End with the producer report as your structured output.
