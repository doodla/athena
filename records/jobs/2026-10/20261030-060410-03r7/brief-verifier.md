# Job 20261030-060410-03r7: verify "Complete homepage Next steps with prerequisites list and ask scope"

## Request (verbatim)

> In the homepage generator (xtask/src/site.rs), make the Next steps install text complete for a first-time reader: give the command that clones the Athena repository, list the prerequisites that setup.sh and the code actually need (check them in the repo; do not guess), and say that `athena ask` acts on the git repository in the current directory, if the code confirms that. Acceptance: a test in xtask/tests/site_page.rs asserts the rendered page contains the clone command and a prerequisites list; fmt, clippy -D warnings and the tests pass.
> 
> Acceptance:
> - The change is made as described: The homepage's install step leaves out the clone command, the prerequisites and what `athena ask` acts on; newcomer ratings have flagged this in 27 ratings
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: xtask/src/site.rs, xtask/tests/site_page.rs

## Reading (accepted by the user)

Audience: first-time readers of the generated homepage. next_steps in xtask/src/site.rs already has the clone command and a prerequisites sentence. I'll turn the prerequisites into a list and add a line that `athena ask` acts on the git repository in the current directory. I'll extend the test in xtask/tests/site_page.rs to assert the clone command and the list.

Assumptions:
- Code survey: read xtask/src/site.rs next_steps (lines 541-565), xtask/tests/site_page.rs (lines 180-202), xtask/src/main.rs setup() (lines 181-230) and crates/athena-cli/src/wiring.rs:176 (Git::discover(cwd)). No records, AnyRecord or refusal rules are involved. No crates/ data files are added.
- The clone command and a prerequisites sentence already exist, so the remaining work is the list form, the current-directory statement, and strengthened tests.
- I could not read setup.sh (blocked by the path policy). Its only visible dependency is curl, from a grep that returned a single curl line, so I rely on main.rs setup() for the prerequisites. The producer should re-check setup.sh and claude/npm needs in the repo.
- Existing unit tests in xtask/src/site.rs (around lines 1536-1538) may assert the old sentence wording and need keeping in step.

## Acceptance criteria

1. [content] In xtask/src/site.rs, next_steps renders the prerequisites as an HTML list (nested <ul>/<li>) naming git 2.40 or later, a C linker (cc), curl, and Claude Code (or npm to install it). These match the checks in xtask/src/main.rs setup().
2. [content] The rendered Next steps include a sentence that `athena ask` acts on the git repository in the current directory. This is backed by Git::discover(cwd) in crates/athena-cli/src/wiring.rs, which errors with 'Athena runs inside a git repository'.
3. [content] xtask/tests/site_page.rs asserts the rendered Next steps section contains `<code>git clone https://github.com/doodla/athena.git</code>`, the prerequisites list items, and the current-directory sentence.
4. [behavior] Run `cargo test -p xtask`. The site_page tests and the unit tests in xtask/src/site.rs all pass, including the existing assertions on setup.sh and `athena ask "add a changelog"`.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`. Both pass.
6. [content] The diff touches only the Next steps text and its tests, plus the CHANGELOG.md entry. No other page section or behavior changes.

## Expected deliverables

- xtask/src/site.rs (modify)
- xtask/tests/site_page.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changes to setup.sh or to athena ask behavior
- Other homepage sections
- README edits

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered page text and the test, fmt and clippy runs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, fmt and clippy are required to pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e60a4e8f9688 → candidate c27aa65eee4f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md             |  1 +
 xtask/src/site.rs        | 12 ++++++++----
 xtask/tests/site_page.rs | 17 +++++++++++++++--
 3 files changed, 24 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-060410-03r7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060410-03r7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (3161 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
