# Job 20261026-144245-t93c: verify "Split site::render into per-section helper functions"

## Request (verbatim)

> Structural pass: xtask/src/site.rs
> 
> Code health found xtask/src/site.rs over a limit: function render is 333 lines, over the limit of 200.
> 
> Measured: 637 production lines (test modules excluded); longest function render at 333 lines; 0 landings in the last 7 days; 13 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function render is 333 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: xtask/src/site.rs

## Reading (accepted by the user)

Break the 333-line `render` in xtask/src/site.rs into small private section functions (header, start-here, states, counters, transitions, rationales, glossary, findings, next steps, claims, footer/wrapper) so every function is under 200 lines and the file stays under its limits. The HTML output must be byte-identical; the audience is the code-health check and maintainers.

Assumptions:
- Code survey: read xtask/src/site.rs (entry point build -> load -> render; render validates claims.json/pin.json/intro.json in order, then appends sections to one `body` String in a fixed order: header, Start here, States, Counters, Transitions, Why, Glossary, Limits, Next steps, Claims, About, footer) and its callers in xtask/src/main.rs and xtask/src/decisions.rs; read crates/athena-core/src/code_health.rs as the place code health measures per-function lines (limit 200) and production lines, so the property is measured by that check on render's own line count, not through a stand-in. No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not involved, and no refusal rule is added or tightened.
- Goal: reduce render below 200 lines with no behavior change; helpers take `&mut String` body plus the needed study/known/github references and return Result<()>, preserving error order and messages.
- Single file stays: 637 production lines is well under the 2000 file limit, so no module split is required.
- No governance file (athena.toml, defaults.toml, workflows) needs to change; xtask has no library target so tests stay as unit tests in xtask/src/site.rs.
- Environment: none needed; no CI, push or history read.

## Acceptance criteria

1. [content] In xtask/src/site.rs, `pub fn render(study: &Study) -> Result<String>` keeps its signature but is no longer than 200 lines; Grep of the file shows new private `fn` helpers (one per section) called from render, and no function in the file exceeds 200 lines.
2. [content] The diff touches only xtask/src/site.rs; the `mod tests` block is not edited except possibly imports, and no test is deleted or weakened (git diff shows no removed #[test] functions).
3. [behavior] Run `cargo test -p xtask`: all existing site tests pass unchanged, including the_real_study_renders_with_an_intro_and_glossary and the_page_cites_claim_ids_and_the_pinned_commit, showing the rendered HTML behavior is unchanged.
4. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; both exit clean.
5. [content] Public items used by xtask/src/decisions.rs and main.rs (ATHENA_URL, CSS, DECISIONS_PAGE, escape, source_url, target_dir, default_out, check_out, load, build, render) keep their names, visibility and signatures.

## Expected deliverables

- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing HTML output, CSS, or error messages
- Splitting site.rs into multiple files
- Editing decisions.rs, main.rs or other files
- Adding new tests beyond what the refactor needs

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused refactor in a single file.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt required as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Existing HTML-content tests are run to show unchanged output.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f8dbae68a18e → candidate 37d847e5f618. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md      |   4 ++
 xtask/src/site.rs | 120 ++++++++++++++++++++++++++++++++++++++++--------------
 2 files changed, 93 insertions(+), 31 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-144245-t93c/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144245-t93c/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42676456439 for 37d847e5f61892cb01dc7d699d384c699b8a2518)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42676456439 for 37d847e5f61892cb01dc7d699d384c699b8a2518)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42676456439 for 37d847e5f61892cb01dc7d699d384c699b8a2518)
- ci-result: `not a check` passed (Check results for 37d847e5f61892cb01dc7d699d384c699b8a2518 come from the CI run https://github.com/doodla/athena/actions/runs/42676456439 (605s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; both exit clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
