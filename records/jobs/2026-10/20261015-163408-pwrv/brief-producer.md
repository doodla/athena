# Job 20261015-163408-pwrv: Make drain count only outcome-changing landings and check all page text

## Request (verbatim)

> try again on 20261015-152305-6h8k

## Reading (accepted by the user)

Fix `athena drain` so "target met" needs (1) a landing whose diff touches what the explainer is built from, and (2) a model-backed check of every page text input (claims.json, intro.json intro/transition/glossary, any other text file) against the pinned source, failing on contradicted or omitted statements. Audience: the person running drain who trusts "target met". The retry starts from base, so it redoes both rules and covers the intro text the last attempt missed.

Assumptions:
- The base has no source-vs-text check and no outcome-changing-landing rule; RealSteps::check_page in drain.rs only tests model.json for self-consistency (read).
- Page text lives in studies/<name>/claims.json and intro.json (intro, transition, glossary); other studies/*.json files are structural or evidence, and the producer must enumerate which are text-bearing.
- Landings count only if their changed paths include studies/, the site generator, or the page sources.
- The model-backed check is guarded by the budget and its cost is added to spent_usd; the reserve must scale with the number of studies checked, or all studies be checked in one capped call.
- The source-check agent is recorded under a role that matches its verifier model and cap, not Role::Audience.
- Existing self-consistency check stays as a floor.
- The real source-check path is tested end to end with a fake agent and a stub pinner.

## Acceptance criteria

1. [behavior] With a stubbed Steps, a drain whose only landing changed paths outside studies/, the site generator and the page sources, with the rating at the target, does not stop as "target met". The report says the landing did not change the outcome. Run `cargo test -p athena-cli --test drain`.
2. [behavior] A landing that changes a study file or the site generator, then a rating at the target and a passing source check, still stops as "target met". Run `cargo test -p athena-cli`.
3. [behavior] Tests inject one wrong statement into each of claims.json, the intro, the transition and a glossary entry (for example teardown closes instead of can reopen; yolo-on allows self-merge; wrong handoff frees a place), plus one omitted fact. With a fake agent and stub pinner, RealSteps::check_page fails each time and names the statement, and drain does not report "target met". Run `cargo test -p athena-cli`.
4. [behavior] A code check and test fail if any text-bearing file in a study directory is not covered by the source check. A page with correct text in every file passes. Run `cargo test`.
5. [behavior] The source check is budget-guarded: the reserve covers every model call it will make (one per study or a single call), drain stops as "budget spent" when it does not fit, and the cost is added to spent_usd. Run `cargo test -p athena-cli`.
6. [content] Stop::TargetMet docs, module docs, `athena drain --help`, a new decision in docs/decisions.md (with alternatives, covering all page text, and the self-consistency floor) and CHANGELOG.md state both rules.
7. [behavior] `cargo fmt --check`, `cargo clippy --workspace` and `cargo test --workspace` pass.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- README.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Rewriting the studies' content or fixing the wrong facts in the existing page
- Changing the audience rating rubric or target value
- Running the check against the live model as part of tests

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria inject a wrong statement into each page text input and assert drain does not stop as target met.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for drain rules and the source check; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check covers every text-bearing page input, not only claims.json.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A code check fails if a text-bearing page file is not covered by the source check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Add a decision in docs/decisions.md with alternatives and a CHANGELOG entry.
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
