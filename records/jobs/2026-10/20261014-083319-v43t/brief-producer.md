# Job 20261014-083319-v43t: Publish firstmate explainer page to GitHub Pages via xtask and workflow

## Request (verbatim)

> try again on 20261014-082328-g6vy

## Reading (accepted by the user)

Render one self-contained HTML explainer from studies/firstmate-work-handling via a new Rust xtask `site` command, and add .github/workflows/pages.yml (a governance file, listed and named explicitly) that builds and deploys it to GitHub Pages. Enabling Pages in repo settings and verifying a live HTTP 200 are owner actions outside the change.

Assumptions:
- The live URL (HTTP 200) cannot be checked by the verifier or produced by a commit alone; it needs the owner to enable Pages with source GitHub Actions and the workflow to run after landing.
- The default branch is main.
- The renderer may add a small crate dependency only if needed; plain string building with an escape function is preferred.
- studies/firstmate-work-handling JSON files are the sole content source.

## Acceptance criteria

1. [content] New file .github/workflows/pages.yml (governance change, named here) runs on push to the default branch and workflow_dispatch, builds with `cargo run -p xtask -- site`, and deploys using actions/configure-pages, upload-pages-artifact and deploy-pages, with minimal permissions (contents: read, pages: write, id-token: write) and a concurrency group.
2. [content] Existing .github/workflows/ci.yml, athena.toml, defaults.toml, rust-toolchain.toml and Cargo lint tables are unchanged.
3. [content] An xtask `site` subcommand renders one self-contained HTML file (inline CSS, viewport meta, responsive layout, prefers-color-scheme dark mode) with text taken from the study JSON and HTML-escaped.
4. [content] Every factual statement on the page cites a claim id from claims.json, and the page states the pinned firstmate commit from pin.json.
5. [content] The output directory is gitignored; the build produces index.html at the site root so the Pages URL serves it.
6. [content] A design note in docs/design records alternatives and deferrals; CHANGELOG.md has an entry; README gives the local build command and the expected Pages URL, and notes the owner must set Pages source to GitHub Actions.
7. [behavior] Run `cargo run -p xtask -- site`; it exits 0 and writes index.html; `git status` shows no tracked changes from the output.
8. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`; all pass, including new renderer tests for escaping, claim-id citation and missing-data error.

## Expected deliverables

- .github/workflows/pages.yml (create) (a governance file: allowed because the reading lists it)
- xtask/src/site.rs (create)
- xtask/src/main.rs (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- .gitignore (modify)
- docs/design/explainer-site.md (create)
- CHANGELOG.md (modify)
- README.md (modify)
- xtask/tests/policy.rs (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Enabling Pages in repository settings
- Confirming the live URL returns HTTP 200
- Expert track, replay, custom domain
- Changes to ci.yml, athena.toml, lints or toolchain

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Renderer is an xtask Rust command.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New renderer tests; cargo test and clippy clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Page cites claim ids and the pinned commit.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design note with alternatives.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Governance file listed and named in a criterion.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets in workflow or page.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: README, changelog and design note written for outsiders.
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
