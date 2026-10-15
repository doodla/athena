# Job 20261015-112611-kccy: Gate drain's 'target met' on a source check; fix places-held counter

## Request (verbatim)

> Drain stopped on 'target met' because the audience rated the explainer 4 of 5, but an independent reader who checked the page against firstmate's pinned source rated it 3: the body still states the work-slot counter wrongly (only a recorded PR or cleanup frees a place) and model.json contradicts itself on it. A newcomer reading only the page can't catch wrong facts, so the check that ends drain has to.

## Reading (accepted by the user)

Drain stops on 'target met' from the audience rating alone, which can't catch wrong facts. I read this as two fixes: correct the places-held counter in the firstmate study (a place frees only when a worker records its ready PR or is cleaned up, per claim c3) and make model.json consistent. Then make drain's stop on target met depend on a check of the page against the pinned source, not only on the rating. The audience is the drain operator and the newcomer who reads the page.

Assumptions:
- The wrong body text is in the firstmate-work-handling study (intro.json and model.json); I did not find a different page source.
- The 'independent check' is a mechanism inside drain, such as a model-free consistency check of model.json against claims or an independent reader step, and it may reuse the existing validation rules in studies/*/validation.json.
- The cheapest sound design is an extra gate on TargetMet; a full independent LLM reader inside drain is more than this change needs.
- No change to athena.toml, defaults.toml or workflows is needed.

## Acceptance criteria

1. [content] In studies/firstmate-work-handling/model.json the places-held counter's changed_by lists every transition that frees or takes a place (spawn t1, the ready-PR report t3, and cleanup t4), matching claim c3 and t2's retry.changes 'falls when another worker records its ready PR or is cleaned up'; the file no longer contradicts itself.
2. [content] No page text in intro.json says a place is freed by anything other than a recorded PR or a cleanup; each such sentence cites c3.
3. [behavior] Drain no longer reports Stop::TargetMet from a rating at or above the target alone: it also requires a passing check of the page against the pinned source. A test in crates/athena-cli/tests/drain.rs shows a 4 rating with a failing check does not stop drain, and a 4 rating with a passing check does. Run `cargo test -p athena-cli`.
4. [behavior] A test shows the check fails on the earlier contradiction (a counter's changed_by omitting a transition whose retry changes it). Run `cargo test`.
5. [behavior] The drain report says which check held drain back when it keeps running. Run `cargo test`; `cargo clippy` is clean and `cargo fmt --check` passes.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- studies/firstmate-work-handling/validation.json (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Re-running the study against firstmate's source
- Changing the audience role's rating scale or prompt
- Changing the other four drain stop reasons
- Editing records/

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is that the stop depends on a rating that cannot detect wrong facts, so the fix gates the stop, not only the one wrong sentence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain tests cover the gate; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The check is enforced in code, not left as prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are run with cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the data fix plus one gate.
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

## Report

End with the producer report as your structured output.
