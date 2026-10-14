# Job 20261014-083319-v43t: verify "Publish firstmate explainer page to GitHub Pages via xtask and workflow"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 33ac4eec7560 → candidate 31a7ac577c3a. In your working directory, `git diff HEAD~1` shows it.

```text
 .github/workflows/pages.yml     |  44 ++++
 .gitignore                      |   2 +
 CHANGELOG.md                    |   6 +
 README.md                       |  15 +-
 docs/design/explainer-site.md   |  53 +++++
 docs/design/requirements-map.md |   2 +-
 docs/non-rust-files.md          |   1 +
 xtask/Cargo.toml                |   2 +-
 xtask/src/main.rs               |  16 +-
 xtask/src/site.rs               | 447 ++++++++++++++++++++++++++++++++++++++++
 xtask/tests/policy.rs           |   5 +
 11 files changed, 589 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (542 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
