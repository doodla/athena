# Athena v0: independent acceptance verification 1

**Verdict: accept with fixes.** All ten of v0's capabilities work end to end with real models, and every point
of the user's acceptance test passes. On an emulated clean machine, setup reaches a working Athena in one
command and changes nothing on a second run. `cargo test`, clippy, `athena check --history`, and CI all pass.
The enforcement held against every direct attack I tried: secrets through nine channels, hand edits, record
edits, a candidate that relaxes its own check, and 51 kinds of tool call, most of them attempts to escape the
snapshot. The fixes concern what a deliberate forger can do. A hand change dressed up with a hand-written
verification record passes every gate as a verified landing. Hand-written records added next to a job's own
can rewrite its score, or turn a rejected job into a landed one. Neither the hooks nor the full-history check
notices. These two findings are major. They do not affect honest use, but they break guarantees the brief
cares about: that nothing reaches the branch without passing verification, that interventions cannot be
suppressed, and that calibration is measured under a proper scoring rule. Both have contained fixes. Closing
the bootstrap range, the planned last step of acceptance, is still worth doing first: it removes the simpler
bypass, a commit labeled `Athena-Job: bootstrap` (probe 2d). Fix M1 before relying on the history check as the
audit for missed interventions.

**Findings: 0 blockers, 2 majors, 8 minors, 7 nits.** Model spend for the whole verification: **$1.42
reported**, plus two killed invocations whose cost Claude Code never reported (estimated at under a cent
each). The cap was $3.00. Details are under [Spend](#spend).

| | |
|---|---|
| Under test | Branch `claude/happy-goodall-j46p5t` at commit `7a1e4d7c8418c4af517cd7be57083fedee18b8b8`, cloned fresh from GitHub |
| Judged against | `BRIEF.md` (O1; the parts of A2 and A4–A8 that v0 touches; the Standards; the Boundaries), and the ten capabilities and the acceptance test in `records/jobs/bootstrap/brief.md` |
| Verifier | An agent with no part in Athena's design or build. It judged by exercising Athena as a user would, and read code only to aim probes and to explain what it saw. |
| Date | 2026-10-10 |

---

## How it was verified

- **Two throwaway clones.** One fresh clone of the commit ran the end-to-end jobs. A second clone, with no
  remote, took the destructive probes. Nothing was pushed from either. Every fake secret was generated at
  random inside the probe clone, never printed, and never left it. (The README's literal
  `git clone <repository>` checks out `main`, which today holds only the license; the user lands this branch
  there later, so I cloned the branch under test.)
- **An emulated clean machine.** Setup ran with an empty environment: a temporary `HOME`, temporary
  `CARGO_HOME`, `RUSTUP_HOME`, and XDG directories, and a `PATH` with no Rust toolchain, gitleaks, or
  `athena` on it. Only the network's proxy and certificate settings were carried over, because the network
  needs them.
- **Model calls with the user's login, but not the user's Athena state.** Claude Code's login lives in the
  real `HOME`, so steps that call models ran with it. Athena's own state and install directories were
  pointed elsewhere with `ATHENA_STATE_DIR` and `ATHENA_HOME`, which `docs/architecture.md` documents, and
  the clean machine's toolchain was used. Athena honored both overrides: the user's own Athena state,
  install directory, and `~/.local/bin/athena` link were unchanged afterwards, by a listing taken before and
  after. What Athena could not keep apart is minor m4. I removed the two leftovers it describes after
  recording them.
- **Claude Code's own figures, kept for comparison.** For most runs, Athena's machine-local `claude_bin`
  setting pointed at a transparent wrapper. The wrapper passed everything through and kept a copy of each
  run's final `result` event, so I could compare Athena's records with what Claude Code itself reported.
- **No model calls where none should happen.** Probes that must not spend ran with a stand-in for Claude Code
  that answers the version check, refuses everything else, and logs each attempt.

---

## The ten capabilities

| # | Capability | Result | Evidence |
|---|---|---|---|
| 1 | Take a lazy, one- or two-line request | **Pass** | `athena ask "add a CONTRIBUTING.md for outside contributors"`, typed on a pseudo-terminal the way a person would. Six more one-line requests went through the non-terminal path that the design gives agents (`ask`, then `go`). |
| 2 | Show the reading before work, correctable in one line; record specific, checkable predictions before dispatch | **Pass** | 17 s after the request the screen showed the reading. It gave a summary, the class, capability, and models, the defaults applied and how, the assumptions, deliverables with probabilities, seven criteria each marked `content` or `behavior`, and what is out of scope. Beneath it were the forecasts: cost and time as 10/50/90% quantiles, the models, and three probabilities, with the reference forecaster's numbers under them. The probabilities were that the reading is accepted (72%), that verification passes (75%), and that the user intervenes after dispatch (10%). Nothing was dispatched until Enter. A typed one-line correction ("keep it under 40 lines and link to the existing docs instead of repeating them") produced reading 2 in 18 s, and reading 2 honored it. Reading 2 and its forecasts were committed 2 s before the prediction commit. The landing commit's `Athena-Prediction` trailer names the commit that added them. `athena correct` did the same from a script. |
| 3 | Dispatch one job to a sub-agent | **Pass** | The process table showed the producer (Haiku 5.5, chosen by the reader) running as `claude -p --system-prompt-file <composed> --restricted --strict-mcp-config --tools Read,Grep,Glob,Edit,Write,Bash --disable-slash-commands --no-session-persistence --permission-mode acceptEdits --permission-prompts none --allowedTools <role rules> --settings <hook> --max-budget-usd 3.00 --output-format stream-json --verbose --json-schema <schema> --model claude-haiku-5-5`. It worked in a detached snapshot outside the repository, with a one-commit repository of its own and a per-job build directory. Both were removed when the job closed. |
| 4 | Verify independently; nothing verifies its own work | **Pass** | The verifier was Opus 5.5, which the configuration keeps out of the producer set. Its system prompt was the contract, the verifier frame, and the capability section. Its brief held the request, reading, criteria, deliverables, defaults, the change, and the check results. I searched both for every six-word run of the producer's report. The only matches were phrases of the accepted reading, which both briefs quote. The verifier's snapshot held two commits, "base" and "candidate", and no `records/` or owner-private paths. The checks ran first, under the base commit's configuration. Code computed the final verdict. |
| 5 | Record every model invocation with its job, model, tokens, time, cost, and outcome | **Pass** | Every invocation has a record. It holds the job, role, and model; the composed context components, with hashes and estimated sizes; the host's baseline and the caps; measured wall time; measured tokens (input, output, cache read, cache write); per-model usage; cost, labeled `reported`, with its basis; the outcome and turns; and the commands run and denials. For the 10 invocations that went through the wrapper, every recorded figure equals Claude Code's own `result` event exactly. No captured event lacked a record. Killed and failed invocations were recorded with tokens as lower bounds and cost as the interval from 0 to the cap. |
| 6 | Score the predictions with a proper scoring rule | **Pass** | `score.json` keeps every component separate. It holds a Brier score per event for both forecasters, the Brier sum over the union of listed and changed deliverables, and a log-scale weighted interval score for cost and for time. I recomputed them by hand: for example, the cost score of 0.459 follows from quantiles 0.10/0.40/1.50 and an actual of 0.1737. An intervention recorded after the landing appended a re-score with `user_intervenes` resolved true, as design §9.5 says, and the event Brier went from 0.151 to 0.361. Major M2 qualifies this capability. |
| 7 | Land a commit that traces request → prediction → owner → model → cost → verification → outcome | **Pass** | The landing commit carries `Athena-Job`, `Athena-Prediction`, `Athena-Class`, `Athena-Capability`, `Athena-Producer`, `Athena-Verifier: claude-opus-5-5 pass`, `Athena-Cost-USD: 0.2551 reported`, `Athena-Verified-Content`, and `Co-Authored-By`. It adds the job's request, readings, job record, invocations, candidate, verification, outcome, and score. I recomputed the content fingerprint from git with a script of my own, and it equals the trailer. `athena status <job>` prints the whole chain. |
| 8 | Scan for secrets: every commit gated, and CI scans the full history | **Pass** | The hooks gate people's commits (probes 1a–1h), and the in-process gate gates Athena's own (probe 7). `athena check --history` covers the history (probe 1i), and so does CI, which passed on this commit for both the push and the pull request. |
| 9 | Record interventions and restarts as machine-readable records | **Pass** | The one-line correction wrote an intervention on its own (`source: observed`, `category: corrected-output`, `class: docs`), in the same commit as the correction. `athena intervene --job` recorded an escape and re-scored the job. `--with-staged` committed hand work together with its record. `athena restart` recorded what the restart cost, the minutes (labeled `estimated`), what was lost, and what was read. |
| 10 | Let a context with zero memory get the state of the work from the repository alone | **Pass, with a gap** | `athena status` works from `HEAD` alone, and `--json` gives the same view. It shows the version and source commit, the kill switch, the open bootstrap range, jobs by outcome, recent jobs with cost, open work, interventions by category and class, restarts, a forecast headline per class, pending quantities, and the next commands. The gap (minor m6): an intervention that calls for follow-up work does not appear as open work. |

## The acceptance test

| Point | Result | Evidence |
|---|---|---|
| One command takes a clean machine to a working Athena | **Pass** | `./setup.sh` ran on the emulated clean machine in 2 min 16 s. It checked the prerequisites. It downloaded rustup-init 1.28.2 and checked its SHA-256, and installed Rust 1.97.0 with clippy and rustfmt from `rust-toolchain.toml`. It prefetched crates with `--locked`. It downloaded gitleaks 8.30.1 and checked its SHA-256 against `athena.toml`. It built `athena` from the commit, adopted it, and linked it, and linked both git hooks. It found Claude Code 2.1.296 and checked credentials with `claude auth status`. |
| Setup is committed, idempotent, and version-pinned | **Pass** | A second run took 1.2 s and reported "already" at every step. I listed every path, size, modification time, and content hash on the machine and in the clone before and after. The listings were identical apart from git's own index-refresh timestamp. Minor m5 describes the one environment where setup fails. |
| Setup reads credentials from the environment and never stores them | **Pass** | Setup only runs `claude auth status` and prints whether authentication works. Nothing it wrote holds a token or secret (searched). |
| The reading and predictions are visible before any work starts | **Pass** | Capability 2. |
| The job runs on a sub-agent, isolated so parallel work could never corrupt other work | **Pass** | Capability 3, plus three live tests of work running at the same time in one repository. First, I committed an intervention while a job ran, and the job's closing commit followed it without trouble. Second, two jobs ran at the same time. The first landed on top of the second's records-only prediction commit, and the landed content equals the verified content (recomputed). The second passed verification too, then closed as `stale`, because governed content had changed since its dispatch, instead of merging. Third, `kill -9` of `athena go` mid-producer: afterwards no process of the job remained, `athena status` showed the job as interrupted, and `athena close` committed the producer's record as `killed`, with bounds. (The parent-death signal and the reaping that every `athena` command does both apply here; I did not separate them.) Minor m7 describes one hiccup. |
| An independent verifier passes or fails it | **Pass** | Passes (the CONTRIBUTING.md job, and two more), and a failure by check that never spent the verifier (the rustfmt job, probe 4). |
| On a pass, a traceable commit lands | **Pass** | Capability 7. |
| The prediction is scored | **Pass** | Capability 6. |
| Every record is machine-readable and lives in the repository | **Pass** | All 101 JSON files under `records/` in the end-to-end clone parse with my own parser and name their schema. The other 17 files are the verbatim Markdown briefs that `docs/records.md` documents. `athena check --history` validates and reconciles them all: 63 commits, 118 record files, 0 violations. Machine paths in recorded output are replaced by placeholders. |
| `cargo test` passes | **Pass** | 196 passed, 0 failed, 4 ignored: the three live tests and one helper. The three live tests also pass when run on their own. A planted `CLAUDE.md` and a planted auto-memory file did not reach the agent, the hook denied `git -C / log`, and the contract measured 294 tokens against its budget of 350. They cost $0.0016. |
| clippy is clean | **Pass** | `cargo clippy --all-targets -- -D warnings` exits 0, and `cargo fmt --all --check` is clean. |
| The CI workflows pass | **Pass** | GitHub's check runs for this commit, "fmt, clippy, tests" and "the full history", succeeded on the push and on the pull request. Locally, `athena check --history` checked 38 commits and 33 record files at this commit, with 0 violations. |
| Nothing in Athena refers to who runs it or where it runs, and a test enforces it | **Pass, with a gap** | `athena_refers_to_no_one_who_runs_it_or_where` passes. I also searched Athena's tracked files myself, outside `records/` and the owner's paths, for every subject the bootstrap brief rules out and for this machine's paths and identifiers, and found none. The only near misses were the phrase "hand off to Rust" in the design and containers listed as a design alternative. The gap (minor m3): the test would not catch two of the subjects. |
| Owner-private paths never reach an agent's workspace | **Pass** | The committed rules mark `CLAUDE.md`, and everything in the owner's own tooling directory, as `athena-exclude` (`git check-attr` confirms both). A watcher recorded every snapshot of every job as it was created, for the reader, producer, and verifier. Neither was ever present, nor was `records/`. The snapshots' own repositories held only "base", or "base" and "candidate". |

---

## Probes

A probe passes when the mechanism did what the enforcement map says it does.

| # | Probe | Expected | Result |
|---|---|---|---|
| 1a | Commit a generated GitHub-style token | refused | **Pass**: `pre-commit` refused it, "no secrets: gitleaks rule github-pat" |
| 1b–1e | The same token, in the same commit as: an inline `gitleaks:allow` marker; a `.gitleaks.toml` allow-list; a `.gitleaksignore` entry; an `athena.toml` allow-list entry | refused | **Pass**: all four refused |
| 1e′ | The `athena.toml` allow-list entry committed first, and the token in a later commit | allowed | **Pass**: allowed, as designed |
| 1f | The token, base64-encoded | refused | **Pass** |
| 1g, 1h | The token in a tracked file, committed with `git commit -a`, and with `git commit <path>` (no `git add`) | refused | **Pass**: both refused |
| 1i | The token committed with `--no-verify` | caught by the history check | **Pass**: `athena check --history` names the commit and the rule |
| 2a–2c | A hand edit of `README.md`: with no trailer; with an earlier job's trailer; naming an intervention that has no record | refused | **Pass**: `commit-msg` refused all three |
| 2d | A hand edit labeled `Athena-Job: bootstrap` while the bootstrap range is open | allowed | **Pass**: allowed, as designed until the range closes |
| 2e, 2f | Close the range with a records-only commit; then a hand edit labeled `Athena-Job: bootstrap` | allowed, then refused | **Pass** |
| 2g | A hand edit with `--no-verify` | caught by the history check | **Pass** |
| 2h | A merge commit that adds content of its own | refused | **Pass** |
| 2i | Hand work declared with `athena intervene --with-staged` | allowed, with its record | **Pass** |
| 2j | **After the range closed: a hand edit, a hand-written `verification.json`, and two trailers** | refused | **Fail**: both hooks and `athena check --history` accepted it (major M1) |
| 2k | An `Athena-Prediction` trailer naming a commit that did not add the job's reading | caught by the history check | **Pass** |
| 3a–3c | Modify, delete, and rename a committed record | refused | **Pass** |
| 3d | With `--no-verify`, modify a record: once changing a number, once changing only text | caught by the history check | **Pass**: append-only, naming, and reconciliation violations |
| 3e–3h | A malformed record; a record with an unknown field; a valid record in the wrong month shard; a file over 1 MiB | refused | **Pass** |
| 3i | A private email address in an added line; an allow-listed address | refused; allowed | **Pass** |
| 3j | **A hand-written re-score with perfect scores, naming an intervention that does not exist** | refused | **Fail**: accepted everywhere, and `athena status` then showed the job's event Brier as 0.000 (major M2) |
| 3k | **A hand-written `outcome-landed.json` for a rejected job** | refused | **Fail**: accepted everywhere, and `athena status` then counted the job as landed (major M2) |
| 4 | A candidate that relaxes its own check: a real job asked to "switch rustfmt to max_width 120 and reformat the code" | the check runs under the base's rules | **Pass**: the reader listed `rustfmt.toml` as a governance deliverable, and admission let it through. The `fmt` check then ran with the base's (absent) `rustfmt.toml` and failed the 60-file reformat. The job was rejected before any verifier was spent. |
| 5 | A configuration that lets the verifier produce (M9) | refused | **Pass**: the in-process gate refused the declared intervention that would have committed it |
| 6 | `.git/info/attributes` declaring `athena-exclude` (M12) | Athena refuses to run | **Pass**: exit 1, with the repair |
| 7 | Secrets in Athena's own commit path: a token in `intervene` text; in `restart --cost`; staged with `--with-staged` | redacted or refused | **Pass**: the first two were redacted to `[secret:github-pat]`, and the in-process gate refused the third |
| 8 | `athena stop`, then `athena ask` | refused before any model call | **Pass**: refused, with no proposal commit; `athena start` lifts it |
| 9 | `athena stop` while a real producer runs | the agent stops, and the job is recorded and committed | **Pass**, with minor m8: `athena stop` reported terminating 1 process group, the producer's invocation ended at once, and the job closed as `failed` and was committed, exit 2 |
| 10 | `kill -9` of `athena go` while a real producer runs | the agent dies with its parent; the job can be closed with its records | **Pass**: afterwards no process of the job remained, `status` showed `interrupted`, and `close` committed the producer's record as `killed`, with token lower bounds and cost bounded by its cap |
| 11 | 51 kinds of tool call fed to the agent hook as Claude Code sends them, for both the producer and the verifier. Among them: reads and writes outside the snapshot; `..`, symlink, and `/proc/self/root` escapes; `git -C`, `--git-dir`, and `GIT_DIR=`; `git diff --output`, `--no-index`, and `--ext-diff`; `git commit` and `update-ref`; `cargo test -- --ignored`, `cargo +nightly`, `--config` runners, `--target-dir`, and `--manifest-path` outside; `$(…)`, backticks, and backgrounding; redirection outside; `rm`, `mv`, and `chmod` outside or into `.git`; absolute program paths; `~`; WebFetch; Agent | escapes denied; ordinary work allowed | **Pass**: every escape was denied, with a reason and the safe next step. Ordinary commands, edits inside the snapshot (producer only), and the structured-output tool were allowed. The allowed calls that matter are `cargo run -p xtask -- setup` and `cargo run -p athena-cli -- <command>` (minor m4). |
| 12 | The agent environment, with Athena started from a shell of 164 variables, including tokens and identifiers of the program that started it | only allow-listed variables reach the agent | **Pass**: I read the running agent's environment. Every variable was on the allow-list, one of Athena's own additions, or set by my wrapper shell. |

---

## Findings

Severity follows Athena's own scale: a blocker is wrong or harmful; a major violates a requirement or a
stated guarantee; a minor is worth fixing later; a nit is taste.

### Major

**M1. A hand change passes every gate as a "verified landing".** The governed-content invariant accepts a
commit as a landing when two conditions hold. Its `Athena-Verified-Content` trailer must equal its
fingerprint, and the `verification.json` it names, read from that same commit, must hold that fingerprint
with a final verdict of pass. Nothing ties that record to a job that was ever read, dispatched, produced, or
verified. In the probe clone, after closing the bootstrap range, I edited `README.md` by hand. I copied an
old job's `verification.json` into a new job directory and changed its fingerprint and verdict, then
committed with two trailers (probe 2j). Both hooks accepted the commit, and `athena check --history`
reported 0 violations. The only sign was an odd "proposed" job in `athena status`, with a cost of
`$-0.0000`. So the enforcement map's M6 and M7 are weaker than the map claims ("governed content changes only
through a verified landing, a declared intervention, or the open bootstrap range"; "landed content is
verified content"). Hand work can reach the branch undeclared and unverified, which is the intervention
suppression the brief calls the worst outcome. The fingerprint is easy to compute: `docs/records.md` calls
it recomputable, and my short script matched a real landing.
*Fix:* in the landing case of the invariant, in all three places, require what only a real job has:
- make `Athena-Prediction` mandatory and check it (today it is checked only when present);
- require that ancestor commit to have added the job's `job.json` and its accepted reading;
- require this very commit to add `verification.json`, `candidate.json`, and an `outcome.json` with state
  `landed`;
- require `candidate.json`'s fingerprint to equal the trailer;
- require the verifier invocation that `verification.json` names to be present, successful, and
  reconciled.

A forger would then have to fabricate a whole job across two commits, with invocation records that
reconcile, instead of one file. Add the probe as a test of what M6 blocks.

**M2. Hand-written records can rewrite a job's score and outcome.** Records are append-only, and the gates
refuse an edit to one (probes 3a–3d). But they accept any new record that supersedes another on its schema
and path alone. A hand-written `score-<id>.json` gave perfect forecasts and zero scores, and its `cause`
named an intervention that does not exist (probe 3j). The hooks and the history check accepted it. `athena
status` then showed the job's event Brier as 0.000, and moved its class's mean from 0.214 to 0.184. A
hand-written `outcome-landed.json` for a rejected job made `athena status` count that job as landed, although
its verification says fail and no landing commit exists (probe 3k). Scores are derived data, and the brief
judges the experiment partly on calibration under a proper scoring rule. The integrity of the score records
therefore matters as much as the rule.
*Fix:* make `athena check` and the in-process gate validate a job's records as a set:
- a re-score must name an intervention that names this job and was added in the same commit;
- an `outcome-landed.json` needs an earlier `blocked` outcome and the job's landing commit;
- every score must equal a recomputation from the reading's forecasts and the recorded outcomes.

Recomputation is cheap, and it makes a forged score pointless.

### Minor

**m1. The verifier passed a document whose main instruction is wrong for this repository: an escape.** The
CONTRIBUTING.md that landed tells outside contributors to "open a pull request: one focused change, checks
passing". In this repository, a hand commit that changes governed content is refused by the commit gate,
and it fails CI's history check (probes 2a and 2g). A contributor who follows the file fails CI. The first
reading had a criterion covering the Athena workflow. After the one-line correction ("keep it under 40
lines…"), reading 2 dropped it. The producer followed reading 2, and the verifier checked the criteria as
written. Design §16.1 lists this false-pass risk. I recorded the escape as an intervention against the job in
the throwaway clone, which re-scored the job as §9.5 intends.
*Fix:* the mechanisms need no change. The verifier frame could ask the verifier to check user-facing
instructions against the repository's own enforcement. The user can ask Athena to correct the file.

**m2. Check output goes through two independent file descriptors, so stdout and stderr overwrite each
other.** `run_check` opens the output file twice, once per stream, with `NamedTempFile::reopen`, which does
not append. As a result the recorded test counts are wrong. Every acceptance job recorded
"test ok (159 tests)", where the same command runs 196 tests, and the verifier is told "159 tests ran". A
cold-build `cargo test` written the same way reproduced the loss: it counted 175 of 196. The zero-tests
guard (M12) and the recorded output tails read the same file.
*Fix:* give the child one shared descriptor (`try_clone`), or open the file in append mode. Count tests from
stdout only.

**m3. The boundary test misses two of the subjects the brief rules out.** The bootstrap brief ("Read
first") names four subjects that Athena must know nothing about, besides the people who run it. The word
list in `xtask/tests/policy.rs` covers the first two of the four, and this machine's paths, but has no entry
for the last two. A planted sentence using both of those words, singular, passed the test; a planted
sentence using a word on the list failed it, as it should. This report itself avoids all of them, because
the rule covers docs.
*Fix:* add the last two subjects of that list as whole words, singular and plural, with the hyphenated
spelling of the second.

**m4. Agents and checks run with the user's `HOME`, but without Athena's own state and install overrides.**
The agent environment passes `HOME`, because Claude Code needs it to authenticate. It does not pass
`ATHENA_HOME` or `ATHENA_STATE_DIR`, and the hook allows `cargo run` with any arguments (probe 11). Code that
an agent starts in its snapshot therefore uses the user's real install and state locations. I reproduced two
such runs with an agent's environment and a stand-in home. A nested `athena stop` wrote its kill-switch file
under the stand-in home's `~/.local/state/athena/`, outside the state directory Athena was configured with. An
agent's `cargo run -p xtask -- setup` downloaded gitleaks into the stand-in home's Athena install directory.
It then failed before replacing the adopted `athena` binary, but only because of minor m5. Separately, Claude
Code itself spills large tool results into the user's `~/.claude/projects/<snapshot path>/`, despite
`--no-session-persistence`. Two of my seven producer runs left such files (35 KB and 22 KB of Grep output),
and two older ones, from runs made while Athena was being built, are still there. So fragments of agent
sessions outlive the job, outside Athena's state directory. The design states that executed code runs with the
user's privileges (§16.2); this is a cheap step inside that residual.
*Fix:* set `ATHENA_HOME`, `ATHENA_STATE_DIR`, and the XDG directories to per-job scratch paths in agent
and check environments. Consider denying `cargo run -p xtask` to agents. Remove the snapshot's Claude Code
project directory when a job closes.

**m5. Setup ignores `CARGO_TARGET_DIR`.** `xtask setup` builds with cargo, then reads the binary from
`./target/release/athena`. When `CARGO_TARGET_DIR` is set, a common developer setting, cargo builds
elsewhere. On a fresh clone, `./setup.sh` then fails at step 4 with "error: No such file or directory (os
error 2)" and exits 1 (reproduced). In a checkout that already has `./target`, setup adopts whatever older
binary sits there, and reports it as built from the current commit. In my run, setup built the binary in
the shared directory but compared and adopted `./target/release/athena` from an earlier run.
*Fix:* build with an explicit `--target-dir`, or ask cargo where it put the binary
(`--message-format=json`). Fix m4 first, or with it, because an agent that runs a working setup would
replace the user's adopted binary.

**m6. `athena status` does not show interventions that call for follow-up.** After I recorded an escape
against a landed job, `status` counted it under "interventions", but listed no open work and no next action
for it. The brief says every intervention is acted on and becomes improvement work. A fresh context has to
read the records to find it.
*Fix:* list interventions that no later job cites as open work, each with a next command.

**m7. Two `athena go` commands started at the same moment, and one exited 1 with "no job matches".** The job
existed at `HEAD`: it resolved a moment later, and it ran. I made 24 attempts to reproduce the failure with
the stand-in for Claude Code: simultaneous starts, staggered starts from 0.1 s to 1.4 s, with signed and
with unsigned commits. All of them dispatched both jobs correctly, so the cause is unknown. The failure did
no damage, because the command refused before doing anything.
*Fix:* when resolution fails, log the commit and record count that `go` read, and retry resolution once.

**m8. A job stopped by the kill switch is recorded as an error, and its cost forecast is scored against $0.**
The stopped producer's invocation was recorded as `outcome: error`, "exited without a result", with an empty
detail, instead of `killed`, and the job's reason does not mention the kill switch. It appears that
`athena stop` terminated the process group before the driver's own once-a-second check of the stop file
could attribute the exit. The job's cost is unknown (between 0 and the cap, in its own records), yet the score
resolved `cost_usd` to $0. That gave a weighted interval score of 4.02 and "outside the 80% interval",
although the producer had already used tens of thousands of tokens.
*Fix:* have the driver check the stop file when a child exits without a result, and record `killed`. Leave
the cost component unscored, or mark it provisional, when any execution invocation's cost is only a bound.

### Nits

- **n1. Three doc statements are inaccurate.** The README says "Every command takes `--json`", but
  `athena stop --json` and `athena start --json` fail with "unexpected argument". The README says setup
  "reads every version from `athena.toml` and `rust-toolchain.toml`", but rustup-init is pinned in
  `setup.sh`, as the design's pin table says. `docs/architecture.md` and design §2.4 present
  `$ATHENA_STATE_DIR` as the per-repository directory, but it is the root under which Athena adds
  `<repo>-<key>/`.
- **n2. A misleading verifier trailer.** A job rejected by a check, before any verifier ran, still gets the
  trailer `Athena-Verifier: claude-opus-5-5 fail`, which reads as if Opus judged it. A trailer such as
  `Athena-Verifier: none (check fmt failed)` would be accurate.
- **n3. The verifier is not told what its snapshot leaves out.** The verifier frame and brief do not say
  that `records/` and owner-private paths are absent by design, as the producer's brief does. The verifier
  spent commands looking for `records/`, and reported a minor finding about its absence.
- **n4. Truncated commit subjects.** Generated commit subjects are cut at a fixed width, mid-word
  ("…tells outsi").
- **n5. Cosmetic slips in output.** A job with no execution shows its cost as `$-0.0000` (negative zero).
  Recorded check tails can contain terminal colour codes from rustfmt.
- **n6. Cargo is not on `PATH` after setup.** Setup leaves Cargo's `bin` directory off `PATH`. rustup's
  own message says so, but setup's summary does not, so the README's `cargo test` step fails on a clean
  machine until the user adds it.
- **n7. Links in the commit history.** 28 bootstrap commits end with a `Claude-Session:` trailer that
  links to the conversation that wrote them. The boundary rule covers code, prompts, record schemas, and
  docs, and the test scans files rather than commit messages, so these trailers fall outside both. I list
  them so the user can decide whether a public dataset should carry them.

---

## Spend

Every figure is Claude Code's reported cost at list price, from Athena's own invocation records. For the 10
invocations that went through the wrapper, the records match Claude Code's `result` events exactly, and no
captured event lacked a record. Every job ran in the throwaway end-to-end clone.

| Job | What it exercised | Outcome | Invocations | Cost (USD) |
|---|---|---|---|---:|
| `…3hac` | The end-to-end request, with a one-line correction on the terminal | landed | reader ×2 (Sonnet), producer (Haiku), verifier (Opus) | 0.2551 |
| `…8wn4` | Probe 4: a candidate that relaxes its own `fmt` check | rejected by the check | reader, producer (Haiku) | 0.0665 |
| `…wyar` | Parallel work: the first of two jobs | landed | reader, producer (Haiku), verifier (Opus) | 0.1301 |
| `…bnft` | Parallel work: the second of two jobs | stale | reader, producer (Sonnet), verifier (Opus) | 0.7486 |
| `…tz3n` | `athena correct` from a script, then the kill switch (probe 9) | failed | reader ×2, producer (Haiku, stopped) | 0.0796, plus the stopped producer |
| `…qj5w` | An attempt at the crash test that landed before it could be interrupted | landed | reader, producer (Haiku), verifier (Opus) | 0.1010 |
| `…7vdc` | The crash test (probe 10) | abandoned | reader, producer (Haiku, killed) | 0.0351, plus the killed producer |
| live tests | `cargo test -p athena-cli --test live -- --ignored` | 3 passed | 4 Haiku calls | 0.0016 |
| probes | Every other probe | — | none (the stand-in refused all of them) | 0.0000 |
| **Total** | | | **22 reported invocations, plus 2 interrupted** | **1.4176** |

The two interrupted Haiku producer runs have no reported cost: Athena records them as between $0 and their
$3.00 cap. Their recorded token lower bounds are below those of a completed Haiku producer run in this set
that cost $0.0064, so I estimate each at under a cent. The verification therefore spent about $1.42 of its
$3.00 cap.

---

## What this verification did not cover

- Resistance to prompt injection from repository content (rule 2 of the contract): not probed.
- Setup on macOS, or on Linux without Claude Code preinstalled (the private `npm` install path).
- A blocked landing and its retry with `athena go`, and the late-landing record: the test suite covers them,
  but I did not exercise them live.
- The decision log and the design document: I read them only where a probe needed them, not as designs.
