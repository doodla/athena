# Owner decision briefs

The owner decides what Athena cannot: Steward proposals above its limit, paused lines of work, design
choices. Today those arrive as long log lines. This note describes the first slice: one-screen visual
briefs, checked before they are shown. The owner's style rules are in
`docs/conventions/owner-style.md`.

## What was built

`xtask/src/site.rs` (the same renderer as the explainer page) gains a `Brief`: a decision, two or more
options with a cost, a risk and a risk level (1 to 3), Athena's recommendation, and a link to a record for
every claim. `render_brief` writes one self-contained page: inline style, one inline SVG bar chart of risk
by option, no script and no external reference.

`cargo run -p xtask -- site` also writes `decisions/index.html` and `decisions/<id>.html` next to the
explainer's `index.html`, and the index links to the decisions page. Waiting items are read from
`records/decisions-waiting/*.json` (absent means nothing is waiting). `Brief::from_proposal` builds the brief
for a waiting Steward proposal.

## Refusals, enforced by the renderer

`Brief::check` runs before any page is written, and a refused brief fails the build:

- no recommendation, or a recommendation that is not one of the options: error names the missing part;
- more than 120 words of prose (decision, option names, costs, risks, recommendation): error gives the count;
- fewer than two options, a missing cost or risk, a risk level outside 1 to 3, or any claim without a record link.

## Alternatives considered

- A second renderer for briefs: rejected; the explainer already has the escaping, CSS and build command.
- Rendering from the Steward's Rust types directly: rejected for now; `xtask` does not depend on
  `athena-core`, and a plain input struct keeps the renderer testable. `crates/athena-core/src/steward.rs`
  is unchanged; a later change will write the JSON files from `ProposedChange`.
- Counting every word on the page: rejected; labels and links are not prose the owner must read.

## What else reads the explainer site, the audience role and the status output

| Reader | Effect of this change | Test or reason |
| --- | --- | --- |
| `xtask/tests/site_page.rs` builds the page with `xtask site` and checks it against the study data | The index gains one link; study content is unchanged | Existing tests still run; the new `build_writes_decisions_page_linked_from_the_index` covers the link |
| Site renderer unit tests in `site.rs` | `render` is untouched; the link is added in `build` | Existing tests pass unchanged |
| `xtask/tests/policy.rs` (boundary scan, workflow rule that CI renders the page with the xtask command) | The command is the same | Reason: no workflow or tool configuration changed |
| Audience role (`crates/athena-core/src/audience.rs`, `Target::explainer`) rates the live explainer page as a newcomer | Rates the index page, which gains one link; briefs are not a target yet | Reason: rating briefs is follow-up 1; the existing audience tests do not read the page body |
| `athena status` (`crates/athena-core/src/status.rs`) | Unchanged | Reason: status lines with brief links are follow-up 2; no status test is affected |

## Follow-ups (not in this slice)

1. The audience role reads each brief against `owner-style.md`, rates it, rewrites it once when the rating
   is 3 or lower, and records the rating.
2. `athena status` shows each waiting item as one line with the brief's link.
3. Briefs for paused lines of work and design choices, and writing the waiting files from Steward proposals.
