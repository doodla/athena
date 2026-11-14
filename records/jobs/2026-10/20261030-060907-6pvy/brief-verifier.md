# Job 20261030-060907-6pvy: verify "Give OLY-B-D2-02 a mechanism: site accessibility, tracks, learning path"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-D2-02 a mechanism
> 
> Requirement OLY-B-D2-02 in docs/spec.toml is active and has no mechanism.
> Requirement: The site is first-class on mobile, reads in a core track and an expert track, and offers a learning path, dark and light modes, reduced motion and accessibility basics.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-D2-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-D2-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Extend the explainer site generator (xtask/src/site.rs) so every page has a skip link, a core/expert track switch, a learning-path list, a reduced-motion rule and focus styles, on top of the existing viewport and dark mode. Then point OLY-B-D2-02 in docs/spec.toml at that code, with a measure ending `status: not shown` and the test that shows it.

Assumptions:
- Code survey: read xtask/src/site.rs: finish() builds the page wrapper with viewport meta; the CSS const (lines ~893-911) has prefers-color-scheme dark and a max-width:40rem rule but no reduced-motion, focus, skip link, tracks or learning path; tests at ~1608 assert viewport and prefers-color-scheme. Entry point is site::build -> render -> finish. The site is in xtask, not crates/; docs/spec.toml mechanisms cite paths like crates/athena-core/src/introspect.rs. No record or data file is added, so check_record_file/AnyRecord are not involved.
- Existing code: extends xtask/src/site.rs (CSS, finish, render) and docs/design/explainer-site.md; reuses existing page tests.
- Audience: the site's readers and the verifier checking the page output; goal is a page that visibly offers the listed features. 'Dark and light modes' is satisfied by the existing prefers-color-scheme plus color-scheme; a manual toggle is optional and not required.
- The track control is CSS/HTML only (no JavaScript) to keep the static page simple; core is default and expert sections are shown via a checkbox or details element.
- No governance files (athena.toml, defaults.toml, workflows) need changing.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-D2-02 has mechanism naming xtask/src/site.rs (a file that exists), owner Athena, and a measure ending `status: not shown` that names the test, replacing mechanism = "none".
2. [content] xtask/src/site.rs page wrapper and CSS contain: a skip link to main content, a core/expert track control with expert content marked, a learning-path ordered list of sections, an `@media (prefers-reduced-motion:reduce)` rule, a `:focus-visible` style, and keep the viewport meta and prefers-color-scheme rules.
3. [behavior] A new unit test in xtask/src/site.rs renders the real study page and asserts each feature above; run `cargo test -p xtask site`. The test fails when the new markup/CSS is removed (it asserts on strings the old page lacks) and passes with it.
4. [behavior] Existing site tests still pass: `cargo test` passes.
5. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
6. [content] CHANGELOG.md has an entry for this change.

## Expected deliverables

- xtask/src/site.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)

## Out of scope

- Client-side JavaScript or a build toolchain for the site
- Changing other spec requirements
- Editing docs/decisions.md or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test, clippy, fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts rendered page output, not just wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in site.rs and spec.toml.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 495de1a0d3cc → candidate 97a480a8114c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                  |  1 +
 docs/design/explainer-site.md |  7 ++++
 docs/spec.toml                |  4 +--
 xtask/src/site.rs             | 76 ++++++++++++++++++++++++++++++++++++++++++-
 4 files changed, 85 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-060907-6pvy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060907-6pvy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43546973364 for 97a480a8114ce6777184e6e533662668205d2d2b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43546973364 for 97a480a8114ce6777184e6e533662668205d2d2b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43546973364 for 97a480a8114ce6777184e6e533662668205d2d2b)
- ci-result: `not a check` passed (Check results for 97a480a8114ce6777184e6e533662668205d2d2b come from the CI run https://github.com/doodla/athena/actions/runs/43546973364 (674s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
