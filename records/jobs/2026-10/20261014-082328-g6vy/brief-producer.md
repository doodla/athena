# Job 20261014-082328-g6vy: Publish the first firstmate explainer page to GitHub Pages

## Request (verbatim)

> Publish a first explainer page on how firstmate handles work.

## Reading (accepted by the user)

Render one self-contained HTML page on how firstmate handles work from studies/firstmate-work-handling via a Rust xtask command, and add a GitHub Pages deploy workflow so it opens at a URL. Enabling Pages in repository settings is outside the change and is flagged for the owner.

Assumptions:
- One static page is enough for 'first explainer'; no expert track, replay, or multi-page navigation.
- The deploy workflow uses the official GitHub Pages actions, triggered on push to the default branch, and runs the xtask renderer then uploads the output directory.
- The Pages source setting (GitHub Actions) must be enabled by the owner in repository settings; the change cannot do that, and the final URL is only confirmed after the first deploy.
- The page is rendered only from the validated study JSON (claims, findings, model, pin); no new facts are invented.
- The renderer lives in xtask (Rust) with no new heavy dependencies; serde_json is used if already available.

## Acceptance criteria

1. [content] An xtask subcommand (e.g. `site`) renders a single self-contained HTML page from studies/firstmate-work-handling JSON, with a viewport meta tag, responsive CSS, and prefers-color-scheme dark mode.
2. [content] Every factual statement on the page cites a claim id from claims.json, and the page states the pinned firstmate commit from pin.json.
3. [content] Page text comes from the study data, not hard-coded, and is HTML-escaped.
4. [content] A new workflow under .github/workflows (named explicitly as a governance change) builds the page with the xtask command and deploys it to GitHub Pages using the official Pages actions with minimal permissions; existing ci.yml is unchanged.
5. [content] A design note in docs/design records alternatives considered and deferrals (expert track, replay, custom domain); CHANGELOG.md has an entry; README states the one command to build the site locally and where it is published.
6. [content] The output directory is gitignored or otherwise not committed as build artifacts, and athena.toml, Cargo lints, and rust-toolchain.toml are unchanged.
7. [behavior] Run `cargo run -p xtask -- site`; it exits 0 and writes the HTML file; `git status` shows only ignored output.
8. [behavior] Run `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check`; all pass, including new renderer tests (escaping, claim-id citation, missing-data error).

## Expected deliverables

- xtask/src/main.rs (modify)
- xtask/src/site.rs (create)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- .github/workflows/pages.yml (create) (a governance file: allowed because the reading lists it)
- docs/design/explainer-site.md (create)
- CHANGELOG.md (modify)
- README.md (modify)
- .gitignore (modify)
- xtask/tests/policy.rs (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Enabling Pages in repository settings or configuring a custom domain
- Expert track, replay, quiz, or final report pages
- Studying other systems (Hermes, Athena)
- Changing ci.yml, athena.toml, Cargo lints, or toolchain files
- Editing the study JSON records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The renderer is Rust in xtask; the page is generated HTML/CSS with no JS toolchain.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each page statement cites a claim id and the pinned commit is shown.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Renderer tests added; test, clippy, and fmt must pass.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: A short design note records alternatives and deferrals.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The deploy is by workflow; citation and escaping are enforced by tests.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Page and workflow contain no secrets or personal data.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One page and one workflow only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Page and docs are written for an outside reader.
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
