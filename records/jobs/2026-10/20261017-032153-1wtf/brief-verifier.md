# Job 20261017-032153-1wtf: verify "Tell the verifier records/ and private paths are absent by design"

## Request (verbatim)

> try again on 20261010-154006-vc5k

## Reading (accepted by the user)

Retry of job 20261010-154006-vc5k as typed: add the "records/, private files, and history are absent by design; not a finding" statement to the verifier prompt and verifier_brief, as producer and reader already carry. The watchdog from the introspection is not part of this retry; the job stays small and focused. Audience: the verifier model and maintainers reading its findings.

Assumptions:
- The typed request 'try again' repeats the original job, not the watchdog proposed in the introspection.
- The earlier attempt failed by deadline, not because its reading was wrong, so the earlier criteria stand.
- Wording follows producer.md line 3 and reader.md line 3; producer and reader prompts and briefs stay unchanged.
- A CHANGELOG.md entry is expected, per the earlier deliverables.

## Acceptance criteria

1. [content] crates/athena-context/prompts/verifier.md states that records/, private files, and history are absent from the verifier's snapshot by design, and that their absence is not a finding, worded like producer.md and reader.md.
2. [content] verifier_brief in crates/athena-context/src/briefs.rs emits a sentence saying records/, the owner's private files, and history are absent by design and their absence is not a finding. Producer and reader briefs and prompts are unchanged.
3. [behavior] A new unit test in briefs.rs asserts verifier_brief output contains the 'absent by design' statement naming records/ and private files. Run `cargo test -p athena-context` and see it pass.
4. [behavior] `cargo test --workspace`, `cargo clippy --workspace --all-targets`, and `cargo fmt --check` pass, including any tests that pin prompt text.
5. [content] CHANGELOG.md has an entry describing the verifier statement.

## Expected deliverables

- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Producer progress watchdog, automatic retry, and deadline changes
- Changes to producer or reader prompts and briefs
- Changes to athena.toml, defaults.toml, or workflows

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the verifier prompt, brief, test, and changelog change; the watchdog is left out.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new unit test plus workspace test, clippy, and fmt checks.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the emitted brief contains the statement.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy, and fmt --check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix goes into both the prompt and the brief, the two places the statement is missing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The wording is plain, for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base d022782c87d2 → candidate 80a9c00b8524. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |  6 ++++++
 crates/athena-context/prompts/verifier.md |  7 ++++---
 crates/athena-context/src/briefs.rs       | 26 +++++++++++++++++++++++++-
 3 files changed, 35 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (793 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
