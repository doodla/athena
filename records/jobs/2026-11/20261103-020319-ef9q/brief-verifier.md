# Job 20261103-020319-ef9q: verify "Add decision-brief renderer with refusal checks to xtask site"

## Request (verbatim)

> Give the owner one-screen visual decision briefs instead of text, and check them before they are shown.
> The owner decides things Athena cannot (Steward proposals above their limit, paused lines of work, design choices) but cannot read long text. Today those arrive as long log lines in athena status.
> - Store the owner's style rules verbatim in docs/conventions/owner-style.md (new file): 'Use HTML to help me understand what we're discussing. Show me with big pictures and little words without witholding key details. Use your own taste, avoid lazy design, integrate real-world pictures/icons/visuals as needed but only selectively when it makes sense. When you do use prose/detail: Be clear, concise/terse, and avoid jargon. Explain things as if the viewer is new here, but dont use weird metaphors.'
> - For each item that waits for a person, render one self-contained HTML brief with the same xtask site renderer: what is being decided, the options, what each costs and risks, and Athena's recommendation, in at most about 120 words of prose plus one diagram or chart; every claim links to the record it came from. Publish them on the existing explainer site under a decisions page.
> - The audience role reads each brief against owner-style.md and rates it; a brief rated 3 or lower is rewritten once before it is published, and the rating is recorded.
> - athena status shows each waiting item as one line with the brief's link.
> - Tests: a waiting Steward proposal produces a brief with the required parts and a link from status; a brief with no recommendation or over the word limit is refused by the renderer.
> - CHANGELOG.md gets an entry; docs/design/ gets a short design note.
> - Before planning, list what else reads the explainer site, the audience role and the status output, with a test or a reason for each.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

First slice of owner decision briefs: store the owner's style rules verbatim, add a renderer in the xtask site that produces one self-contained HTML brief per waiting item and refuses briefs without a recommendation or over ~120 words, and publish a decisions page. The audience-role rating and rewrite, and the status line links, are follow-up requests.

Assumptions:
- Goal: the owner (a non-reader of long text) gets one-screen visual briefs; the audience is the owner and the verifier judging against owner-style.md.
- Code survey: read xtask/src/site.rs and xtask/src/main.rs (site entry point) and crates/athena-core/src/steward.rs and status.rs (where waiting proposals and status lines come from); the briefs add no records or data files under crates/, so check_record_file/AnyRecord are not affected.
- Scope cut to fit a single run: renderer, refusal rules, decisions page, style file, design note, changelog. Waiting items are limited to Steward proposals in this slice.
- The brief input is a plain struct (title, options with cost/risk, recommendation, record links); word count covers prose fields only, limit 120 with small tolerance.
- Refusal rules apply only to new briefs, not existing readings, so the replay refusal-rate criterion does not apply.

## Acceptance criteria

1. [content] docs/conventions/owner-style.md exists and contains the owner's quoted style rules verbatim, including 'Show me with big pictures and little words without witholding key details.' (checked by Grep).
2. [behavior] Run `cargo test -p xtask`: a test builds a brief from a waiting Steward proposal and asserts the HTML has the decision, options, cost and risk per option, a recommendation, one diagram (inline SVG), and a link to the source record for each claim.
3. [behavior] Run `cargo test -p xtask`: a brief with no recommendation is refused with an error naming the missing part, and a brief with prose over the word limit (about 120 words) is refused with an error naming the count.
4. [behavior] Run `cargo run -p xtask -- site` (or the existing site command) and `ls` the output: a decisions page exists under the explainer site, is linked from the site index, and each brief is self-contained (no external script or stylesheet references, checked by Grep).
5. [content] CHANGELOG.md has an entry for decision briefs, and docs/design/ has a short design note with alternatives and a section listing what else reads the explainer site, the audience role and the status output, each with a test or a reason; it states that rating/rewrite and status links are follow-ups.
6. [content] Existing site renderer tests still pass because the change extends xtask/src/site.rs rather than adding a second renderer; no change to athena.toml, defaults.toml or workflows.

## Expected deliverables

- docs/conventions/owner-style.md (create)
- xtask/src/site.rs (modify)
- xtask/src/main.rs (modify)
- docs/design/owner-decision-briefs.md (create)
- CHANGELOG.md (modify)
- crates/athena-core/src/steward.rs (modify)

## Out of scope

- Audience-role rating of briefs against owner-style.md, rewrite-once for ratings of 3 or lower, and recording the rating (follow-up request 1)
- athena status one-line-per-waiting-item with brief links (follow-up request 2)
- Briefs for paused lines of work and design choices (follow-up request 3)
- Repository-wide fmt, clippy or test run as its own criterion

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Renderer and tests in Rust within xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests for brief parts and refusals.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design note and docs written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Cut to renderer slice; rest named as follow-ups.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests and the site build.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Refusals enforced in the renderer, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c62f663326e1 → candidate dc43f2d4e3f1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   9 +
 docs/conventions/owner-style.md      |   8 +
 docs/design/owner-decision-briefs.md |  51 ++++
 xtask/src/site.rs                    | 452 ++++++++++++++++++++++++++++++++++-
 4 files changed, 518 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41597307099 for dc43f2d4e3f1288097d4087a8eae28f8c621ff85)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41597307099 for dc43f2d4e3f1288097d4087a8eae28f8c621ff85)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41597307099 for dc43f2d4e3f1288097d4087a8eae28f8c621ff85)
- ci-result: `not a check` passed (Check results for dc43f2d4e3f1288097d4087a8eae28f8c621ff85 come from the CI run https://github.com/doodla/athena/actions/runs/41597307099 (583s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
