# Job 20261015-031159-xx7n: Correct and complete the firstmate explainer page to reach 4 of 5

## Request (verbatim)

> The audience rated the explainer 3 of 5 and queued nothing; anything under 4 isn't good enough for me. Independent readers who also checked the page against firstmate's source rated it 2 of 5: they found facts the page gets wrong or leaves out, which a newcomer reading only the page can't notice.

## Reading (accepted by the user)

The firstmate explainer (built by `xtask site` from `studies/firstmate-work-handling`) rated 3 by its audience and 2 by readers who checked it against firstmate's source. I read this as: fix what the page gets wrong and fill in what it leaves out, mainly in `intro.json` and the study claims and model. Newcomers are the audience, and the page must stay cited and buildable.

Assumptions:
- The audience is newcomers to firstmate who read only the page; the reviewers who check against firstmate's source are the stricter judges, so factual accuracy is the main lever on the rating.
- The request does not list the specific errors. I could not see firstmate's source or the reviewers' notes, so the producer must re-read the pinned commit (pin.json) to find them. My gap list comes from reading the page's own claims and model and is not exhaustive.
- The study stays a single pinned commit and all content stays in the existing JSON files; no new rendering features, script, or diagram are added.
- Corrections that change claims or model must keep the study validators in crates/athena-cap-study passing; validation.json is regenerated only if the validator requires it.

## Acceptance criteria

1. [content] Every firstmate fact in intro.json, claims.json and model.json that changed is backed by a claim whose quote appears in the pinned firstmate source at the pinned commit; the summary of the change lists each factual correction and each addition, with the source path and lines.
2. [content] The page's Start here section and transition cards state what a newcomer needs and the current page omits or blurs. This includes the failure and limit behaviour already in the claims: lock contention exits 1 without waiting, an unreadable capacity exits 1, relaunch failure restores the record, no retry or relaunch bound was found, and a missing session start leaves a close unreplayed. It also corrects the over-simplified merge-authority wording, since yolo is a configured setting and the claim says the 'configured merge authority', and the glossary entry for 'ship'.
3. [content] Placeholder values the study model carries (guard 'none' with evaluator 'agent' on t5, t7, t8) are not presented to a reader as if the agent evaluates a guard; the page says no guard applies.
4. [behavior] Run `cargo run -p xtask -- site --out <dir>` from the repository root with a fresh out path inside the working directory. It exits 0 and writes index.html. Every cited claim id resolves and no intro element, glossary term or transition summary is uncited.
5. [behavior] `cargo test --workspace` passes, including the xtask site tests and the study-capability validation tests, `cargo clippy --workspace --all-targets` is clean, and `cargo fmt --check` passes.
6. [content] If the intro or lead-in structure changes, docs/design/explainer-site.md describes it; no governance file (athena.toml, defaults.toml, workflows) changes.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/rationales.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- xtask/src/site.rs (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Studying additional firstmate mechanisms beyond work handling, or a different pinned commit
- An expert track, replay view, drawn state diagram or custom domain
- Enabling GitHub Pages or changing .github/workflows/pages.yml
- Changing study schemas in crates/athena-cap-study

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every new or corrected statement on the page cites claims with source quotes from the pinned firstmate commit; unmeasured items are labeled.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Wording is rewritten for a newcomer who has never seen firstmate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass, with a site test added if the structure changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier builds the page with xtask site and checks the claim-to-source citations.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The accuracy and completeness of the study content is fixed, not just the few phrases reviewers named.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The missing specifics were recovered from the repository's study files and the design note.
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
