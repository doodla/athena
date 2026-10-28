# Explainer site

This note records how the explainer page is built and published, what else was considered, and what is
left for later. It answers requirement D2-1 in [`requirements-map.md`](requirements-map.md): the explainer is
published to GitHub Pages from this repository and runs locally with one command.

## What is built

- `cargo run -p xtask -- site [--out <dir>]` renders one self-contained file, `index.html`, into `site/` in
  the cargo target directory (`$CARGO_TARGET_DIR`, else `target/`), or into `<dir>`. The default is outside
  the work tree on purpose: a verifier builds the page to check it, and a write into its work tree would
  change the snapshot it is verifying. The xtask enforces this: an `--out` inside the repository root,
  other than under the target directory, is refused with an error. Unit tests cover both rules.
- The only content source is `studies/firstmate-work-handling`: `pin.json`, `claims.json`, `model.json`,
  `rationales.json`, `findings.json` and `intro.json`. The renderer lives in `xtask/src/site.rs`.
- The page has inline CSS, a viewport meta tag, a responsive grid, and a dark theme through
  `prefers-color-scheme`. It loads no script, font, image or stylesheet from elsewhere.
- Every piece of study text is HTML-escaped. Every state, counter, transition and rationale links to the
  claim ids that support it, and the claims with their source paths and lines are listed at the end. A
  citation of a claim that does not exist, an element with no claim, or a missing or malformed study file is
  an error; no page is written from partial data.
- The page is for a newcomer who knows neither Athena nor firstmate. Its title and heading name Athena, and
  the opening paragraphs (`intro.json` `about`) say what Athena is, what a study is, and who made this one
  and why, before firstmate appears. A "Start here" section then says what firstmate is (AI coding agents
  as workers, who uses it, what problem it solves) and walks one piece of work from start to finish. The
  glossary is its own section after the main sections (it is reference, not a prerequisite), followed by
  "Next steps" with links to Athena and firstmate. When the pinned repository is on GitHub, the repository
  URL and every claim source link to the pinned commit's file and lines (`blob/<sha>/<path>#Lstart-Lend`);
  otherwise they are shown as text. Citation ids are small and muted so they do not break up the prose.
  The text lives in `intro.json` beside the other study files. Every sentence and every glossary term about firstmate cites claim ids from `claims.json`; a
  glossary term about the page's own structure (state, transition, trigger, guard, counter, claim, pin,
  mark) is marked "about this page" and cites nothing. An entry with no claims and no such mark is an error.
- Reading order for a newcomer: the header carries no study id (it is explained under "About this page",
  after the claims). Under "Start here", one cited sentence (`intro.json` `what_is`) says what firstmate
  is, before "Words used first". "Next steps" gives the install command and an example request from the
  README Quick start. The finding ids under "Limits of this study" are consecutive (f1, f2, f3); a test in
  `xtask/src/site.rs` checks this.
- Between the walk-through and the glossary, a list headed "When things go wrong, and what is not limited"
  comes from `intro.json` `limits`: lock contention and an unreadable capacity both exit 1, a failed relaunch
  restores the record, no retry or relaunch bound was found, and a close is replayed only at a session start.
  Each item cites claim ids, and an uncited item is an error. A transition whose guard is the placeholder
  `none` is shown as "No guard applies", with no checker named.
- Under each of States, Counters, Transitions and Why it is built this way, a lead-in says what the section
  shows and how to read it. The Transitions lead-in is followed by a short explanation of the two deciders,
  agent and code, and each transition card opens with a one-line plain summary. All of this is in
  `intro.json` (`leadins`, `deciders`, `summaries`), each element citing claim ids. It is kept out of
  `model.json` because the study model schema (`crates/athena-cap-study`) rejects unknown fields. A
  transition with no summary, or a lead-in, decider or summary with no claim, is an error.
- The page states the pinned firstmate commit from `pin.json` at the top and in the footer. The study's
  findings are shown under their own finding ids, in a section that says they are not claims about firstmate.
- `.github/workflows/pages.yml` runs on a push to `main` and on manual dispatch. It builds with
  `cargo run --locked -p xtask -- site`, then deploys with `actions/configure-pages`,
  `actions/upload-pages-artifact` and `actions/deploy-pages`. Permissions are `contents: read`,
  `pages: write` and `id-token: write`, and a `pages` concurrency group stops two deploys overlapping.
  The workflow uploads `target/site`, the xtask default. Because `index.html` sits at the root of the uploaded directory, the Pages URL serves it directly.

## Alternatives considered

| Alternative | Why not |
|---|---|
| A template or Markdown crate (for example `tera`, `pulldown-cmark`) | The page is one fixed layout over a handful of JSON shapes. Plain string building with one escape function adds no dependency and no new lockfile entries. |
| A static site generator (mdBook, Zola) | Needs a second tool in CI and a second source format; the content already exists as JSON. |
| Committing the rendered HTML | It would drift from the study files. Building it in the workflow keeps one source. |
| Publishing from a `gh-pages` branch | Needs write access to a branch and a bot commit per build. The Actions deploy needs only the three permissions above. |
| Adding the Pages steps to `ci.yml` | `ci.yml` stays read-only with no deploy permissions; a separate workflow keeps the broader token scope to one job file. |
| Client-side rendering of the JSON in the browser | It would need JavaScript, which the repository's all-Rust rule disallows, and the page would not work without script. |

## Watching the deploy

After `athena land --pr` lands a pull request, it looks at the `pages` workflow run for the merged commit
(D97). It waits up to about ten minutes for the run to finish. A run that `timed_out` or was `cancelled` is
re-run once through the `gh` wrapper, and the re-run gets the same wait; a second failure is reported and not
re-run. A commit with no Pages run is reported and is not an error. Every outcome is shown to the user, and none
of them undoes the landing.

## Deferred

- Enabling Pages in the repository settings (source: GitHub Actions) is an owner action. Until it is done the
  deploy job fails. Checking that the live URL answers HTTP 200 can only be done after that and a workflow run.
- The Pages actions are referenced by major version tag, not by commit SHA as `ci.yml` does. Pinning them is
  worth doing once the SHAs are looked up from the official releases.
- A tag-triggered or pull-request preview build.
- An expert track, a replay view, and a custom domain.
- The state graph is shown as lists and cards, not a drawn diagram; a drawing would need generated SVG.
- Studies beyond firstmate-work-handling. The study directory is a constant in `site.rs` for now.

## Checks

Unit tests in `xtask/src/site.rs` cover escaping, claim-id citation, the pinned commit, an unknown claim id,
missing study data, the output location, the intro appearing before the states and the glossary after them, Athena being named
before firstmate, the next-steps links, GitHub source links, and an uncited
intro element being rejected, the lead-ins, deciders and transition summaries rendering, and a transition
without a summary being rejected. `xtask/tests/policy.rs` and `docs/non-rust-files.md` list
`pages.yml` as an allowed YAML workflow.
