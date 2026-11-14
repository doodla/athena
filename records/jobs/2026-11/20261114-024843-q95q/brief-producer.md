# Job 20261114-024843-q95q: Add accessible core/expert tracks to site; keep links alive; spec

## Request (verbatim)

> try again on 20261030-060907-6pvy

## Reading (accepted by the user)

Retry of job 20261030-060907-6pvy: extend xtask/src/site.rs with skip link, CSS-only core/expert switch, learning path, reduced-motion and focus styles, but keep the claims list and glossary visible in every track so no in-page link is dead. Add a test that renders each track and fails on any dead or hidden anchor target, then record the mechanism and a `status: not shown` measure for OLY-B-D2-02 in docs/spec.toml.

Assumptions:
- Code survey: I read xtask/src/site.rs and crates/athena-core/src/status.rs. Entry point: site::render (site.rs:241) builds the header, then start_here, states, counters, transitions, rationales, glossary, findings, next_steps and claims_section, then finish (site.rs:628), which writes the about section, the footer and the html wrapper with the viewport meta and CSS. Citations come from cites() (site.rs:155) as `<a href="#claim-ID">`, and their targets are in claims_section. with_evidence (site.rs:889) later inserts the job-evidence section after `</header>`. decisions.rs imports site::CSS and builds its own wrapper. Validation: STATUS_MEASURE_LABELS (crates/athena-core/src/status.rs:312) is the allow-list. A measure must end with `status: <label>` or `status: not shown` naming a test; hestia::spec_gaps flags others (status.rs:520). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip tests are not touched. The property is measured directly by a unit test in xtask/src/site.rs that calls the real render() on the study fixture and inspects the HTML. It does not measure a stand-in.
- Prior job 20261030-060907-6pvy is addressed finding by finding. Major, dead claim links: the claims section and glossary stay in the core track (not marked expert), so every `#claim-` link resolves; the new test checks it. Minor, learning path matching only `<section><h2>`: build the list from all sections, including those with ids, e.g. by matching `<section` generally or by collecting section titles in one place. Minor, evidence section ahead of the reading nav: insert the nav so it comes before the evidence section, or have with_evidence insert after the nav. Minor, decisions page: either give decisions.rs the skip link and main id, or word the spec, design doc and changelog as the explainer page, with decisions getting only the shared CSS rules. The previous candidate's four changed files (xtask/src/site.rs, docs/spec.toml, CHANGELOG.md, docs/design/explainer-site.md) are all deliverables again. The retry starts from base, so that work is redone, not kept as is.
- The introspection's proposed request applies: no dead anchors in any track, and the header's claim that claims are listed at the end must hold. The new request text, `try again`, takes precedence for the spec/measure work. Both the spec work and the link-integrity test are in scope.
- Audience: the site's readers on mobile and with assistive tech, and the verifier checking the rendered HTML. Expert-only detail (marked class expert) may be hidden in the core track via CSS only, with no JavaScript.
- xtask has no library, so tests are unit tests in xtask/src/site.rs, run with `cargo test -p xtask`.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-D2-02 has a mechanism that names xtask/src/site.rs instead of "none", owner Athena, and a measure that ends `status: not shown` and names the test that shows it (a diff line in docs/spec.toml).
2. [content] xtask/src/site.rs page wrapper and CSS contain a skip link to the main content, a core/expert track control with expert content marked, a learning-path ordered list covering every section, including those that have an id, an `@media (prefers-reduced-motion:reduce)` rule and a `:focus-visible` style. The existing viewport meta and `prefers-color-scheme:dark` rules remain.
3. [content] The claims section and the glossary are not hidden by the track CSS. Only sections marked class expert are hidden in the core track, and the header sentence saying the claims are listed at the end stays true.
4. [behavior] A new unit test in xtask/src/site.rs renders the real page in each track (core default and expert ticked). It fails if any in-page `href="#..."` points to a missing id or to an element inside a hidden expert subtree, or if a section the header names is hidden. Run `cargo test -p xtask site`. It passes with the change and fails if claims_section is marked expert.
5. [behavior] A second assertion in that test checks each feature exists in the rendered HTML: skip link, track control, learning-path list, reduced-motion rule, focus-visible rule. Run `cargo test -p xtask site`. The existing site tests, including the job-evidence ordering test, still pass or are updated consistently.
6. [behavior] `cargo test` passes, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes (cite the commands that ran, or the Athena checks recorded on the final tree).
7. [content] CHANGELOG.md has a new entry for this change, and docs/design/explainer-site.md describes the tracks, the learning path and the rule that no track has dead links, without claiming features that the decisions page lacks.

## Expected deliverables

- xtask/src/site.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)
- xtask/src/decisions.rs (modify)

## Out of scope

- Client-side JavaScript or a build toolchain for the site
- Changing other spec requirements
- Editing docs/decisions.md or records/
- Changes to crates/ code, athena.toml, defaults.toml or workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check rendered links and features, not only that files exist
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test, clippy and fmt to pass
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test covers every track and every anchor, not only the claim links that failed
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261030-060907-6pvy raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (xtask/src/site.rs:673 (EXPERT_SECTIONS) and CSS line 962 (.expert{display:none})): In the default core track, every claim citation link goes nowhere, and the page header's promise that claims are listed at the end is false. cites() (site.rs:169) writes `<a href="#claim-{id}">[{id}]</a>` after statements in all the core sections. The targets are `<li id="claim-...">` inside "Claims and sources" (line 586). This change marks that section expert and hides it with `.expert{display:none}` unless the checkbox is ticked. An anchor whose target is inside a display:none subtree does not navigate, so on the default page every [cN] link is dead. The header (line 267-268) still says "the claims, with links to their sources, are listed at the end". Glossary is hidden the same way, although core cards use its terms. This breaks a working default to satisfy the track criterion. Simpler fixes: keep Claims and sources (and Glossary) in the core track and mark truly advanced material as expert instead, or reveal a hidden expert section when one of its children is the link target (`.expert:has(:target){display:block}`). Add a test that every href="#..." in a core section points to an element that is not under class="expert".

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-024843-q95q/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-024843-q95q/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml requirement OLY-B-D2-02 has a mechanism that names xtask/src/site.rs instead of "none", owner Athena, and a measure that ends `status: not shown` and names the test that shows it (a diff line in docs/spec.toml).
xtask/src/site.rs page wrapper and CSS contain a skip link to the main content, a core/expert track control with expert content marked, a learning-path ordered list covering every section, including those that have an id, an `@media (prefers-reduced-motion:reduce)` rule and a `:focus-visible` style. The existing viewport meta and `prefers-color-scheme:dark` rules remain.
The claims section and the glossary are not hidden by the track CSS. Only sections marked class expert are hidden in the core track, and the header sentence saying the claims are listed at the end stays true.
A new unit test in xtask/src/site.rs renders the real page in each track (core default and expert ticked). It fails if any in-page `href="#..."` points to a missing id or to an element inside a hidden expert subtree, or if a section the header names is hidden. Run `cargo test -p xtask site`. It passes with the change and fails if claims_section is marked expert.
A second assertion in that test checks each feature exists in the rendered HTML: skip link, track control, learning-path list, reduced-motion rule, focus-visible rule. Run `cargo test -p xtask site`. The existing site tests, including the job-evidence ordering test, still pass or are updated consistently.
`cargo test` passes, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes (cite the commands that ran, or the Athena checks recorded on the final tree).
CHANGELOG.md has a new entry for this change, and docs/design/explainer-site.md describes the tracks, the learning path and the rule that no track has dead links, without claiming features that the decisions page lacks.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
