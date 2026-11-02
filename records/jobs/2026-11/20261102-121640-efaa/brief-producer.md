# Job 20261102-121640-efaa: Check ontology terms against sources with bounded, reasoned excuses

## Request (verbatim)

> try again on 20261027-054251-33v7

## Reading (accepted by the user)

Retry of job 20261027-054251-33v7 (rejected: file-wide source-only excuses disabled the content check for 6 of 7 terms). The ontology test will compare every term with its governing sources, accept excuses only per named marker with a reason, and require every term to have checked claims.

Assumptions:
- Code survey: read crates/athena-cli/tests/ontology.rs (problems, real_text, term helpers, scratch_root, the unit tests) and docs/ontology.toml (seven terms with sources fields). The ontology file is under docs/, not crates/, so no record file or AnyRecord round-trip is added. The property is measured directly by scanning BRIEF.md and the term's own file sources for each term name.
- Job 20261027-054251-33v7 is addressed by keeping its approach (senses and excused fields, scan of BRIEF.md and file sources) and fixing each finding: (major) source-only or marker-less excuses are removed, and every excuse must name a marker and give a reason; BRIEF.md can never be excused file-wide; the real terms get senses/markers so each of the seven is genuinely checked, including Athena's BRIEF.md:11 hit and its third 'studied instance' use. (minor) markers match as whole words/phrases, not substrings, and describe the sense rather than copying a line. (nit) an excuse that matches no line is flagged as stale. Its four changed files are kept as deliverables and re-edited from base.
- Per the introspection 20261102-121613-7j4a (kdwg): the test also fails if any term has no claim checked against a source, and a deliberately altered definition for each of the seven terms is flagged.
- Goal: stop seeded term content from contradicting governing sources unnoticed. Audience: the verifier and maintainers who rely on docs/ontology.toml. Where a definition really disagrees with its source, it is corrected in docs/ontology.toml and listed in the decision/changelog.
- Source-wide excuses are dropped entirely; a per-file excuse is allowed only with a marker, a reason, and exactly one file.

## Acceptance criteria

1. [content] The change description in docs/decisions.md and CHANGELOG.md names the root cause (content checked for structure, not against sources; and an unbounded file-wide excuse let the check be met by excusing), citing jobs 20261017-173508-0wtc, 20261025-214509-y179 and 20261027-054251-33v7, with alternatives (prose-only rule, measure-first stage) and why the check lives in the ontology test.
2. [behavior] Run `cargo test -p athena-cli --test ontology`: a new test scans BRIEF.md and each term's file sources for every term name in docs/ontology.toml and passes on the real file.
3. [behavior] Run `cargo test -p athena-cli --test ontology`: for each of the seven terms (Hermes, planning, job, Athena, product versus system, landed, Sri) a test using a deliberately altered definition or missing sense is flagged; the Hermes case (tool-only entry while the source names a role) fails, and passes with both senses listed.
4. [behavior] Run `cargo test -p athena-cli --test ontology`: tests show that an excuse with no reason, no marker, or covering more than one file is flagged, that an excuse matching no line is flagged as stale, and that a term with no checked claim is flagged.
5. [content] Grep of docs/ontology.toml shows no excuse without a marker and reason, and no file-wide excuse; the check in crates/athena-cli/tests/ontology.rs is generic over all terms, matches markers as whole words, and the rule and optional fields are described in the test header or docs/ontology.toml [about].
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`: all pass.
7. [content] `git diff --stat` shows only crates/athena-cli/tests/ontology.rs, docs/ontology.toml, docs/decisions.md and CHANGELOG.md changed; no athena.toml, defaults.toml, workflows, studies/, or speed-work files.

## Expected deliverables

- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Speed-target work from job 20261025-214509-y179 (measure-first stage, secrets.rs memo, run_check polling, crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs)
- Adding new ontology terms
- Changing any study's files under studies/
- Changes to athena.toml, defaults.toml or workflows
- Production code outside the ontology test
- Wiring the ontology into agent contexts or prompt budgets

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The test change is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests with fmt, clippy and test passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the ontology tests and check flagged altered definitions.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The excuse design and the content-vs-structure gap are fixed in the check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check every term is really compared with its sources.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Four files, one focused change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by test, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-054251-33v7 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/ontology.toml (excused entries on planning, job, Athena, product versus system, landed, Sri); crates/athena-cli/tests/ontology.rs sense_problems excused predicate): A source-only excuse turns off the content check for a whole file, and 6 of the 7 terms use one on every file that gets scanned. In practice only Hermes, the case already fixed, is checked against its sources. In sense_problems, an excuse with `source` set and no `marker` matches every line of that file (`e.marker.as_deref().is_none_or(..)`). Decision-id sources are skipped, so these terms have every scanned file excused: planning (BRIEF.md, job-planning.md), job (BRIEF.md, records.md), Athena (BRIEF.md; D01 is skipped, so its `senses` list is never consulted), product versus system (both of its files), landed (BRIEF.md and native-pr-landing.md; its other source, D10, is skipped) and Sri (BRIEF.md). The excuse hides real candidate senses. BRIEF.md:11 says 'Athena: the orchestration system you build', which the definition contradicts ('Athena is not the whole system'). BRIEF.md:22 uses `athena` as the repo name. The Athena excuse itself mentions a third use ('the studied instance in Part 2') and waves it through instead of listing it. BRIEF.md is append-only and is where new senses arrive, and for every term except Hermes it is fully excused, so a new second sense there passes silently. That is the same failure class the request says to prevent. Any new term can pass the same way by adding one blanket excuse, which brings back the prose-only guard that D152 rejects. Simpler, more correct alternative: excuses must name a marker (per hit or phrase, as the introspection proposed, 'records why a hit is not one'), and source-wide excuses are refused for BRIEF.md or dropped altogether. The real terms then get the senses and markers they need (Athena already has senses that would do most of the work).

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 16 cases in 14 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena one disk lifecycle. Before the low-disk check and before any hold, the writer's regenerable build is removed once the writer has finished and its output is committed or recorded. The guard then measures free space after that removal. Where checks come from GitHub, the guard does not coun...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

24 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change description in docs/decisions.md and CHANGELOG.md names the root cause (content checked for structure, not against sources; and an unbounded file-wide excuse let the check be met by excusing), citing jobs 20261017-173508-0wtc, 20261025-214509-y179 and 20261027-054251-33v7, with alternatives (prose-only rule, measure-first stage) and why the check lives in the ontology test.
Run `cargo test -p athena-cli --test ontology`: a new test scans BRIEF.md and each term's file sources for every term name in docs/ontology.toml and passes on the real file.
Run `cargo test -p athena-cli --test ontology`: for each of the seven terms (Hermes, planning, job, Athena, product versus system, landed, Sri) a test using a deliberately altered definition or missing sense is flagged; the Hermes case (tool-only entry while the source names a role) fails, and passes with both senses listed.
Run `cargo test -p athena-cli --test ontology`: tests show that an excuse with no reason, no marker, or covering more than one file is flagged, that an excuse matching no line is flagged as stale, and that a term with no checked claim is flagged.
Grep of docs/ontology.toml shows no excuse without a marker and reason, and no file-wide excuse; the check in crates/athena-cli/tests/ontology.rs is generic over all terms, matches markers as whole words, and the rule and optional fields are described in the test header or docs/ontology.toml [about].
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`: all pass.
`git diff --stat` shows only crates/athena-cli/tests/ontology.rs, docs/ontology.toml, docs/decisions.md and CHANGELOG.md changed; no athena.toml, defaults.toml, workflows, studies/, or speed-work files.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
