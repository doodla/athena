# Roles, the steward, and the requirements ladder

This note covers the first build of the `BRIEF.md` addendum "2026-10-14: A loop that steers Athena, roles, and
a ladder of requirements" (rows A9-1 and A10-1 of `docs/design/requirements-map.md`). It is small on purpose:
one role, one ladder. Both rows are Partial in the map: A9-1 because the model-backed steward has not been
run live and has the limits listed below, and A10-1 because cost and yield tracking, reshaping and retiring
are not built.

## What was built

- **A role is a recorded definition** (`athena.role/1`, `crates/athena-data/src/roles.rs`): a name, a version,
  a goal, a short stable prompt, a cadence counted in finished tasks, and the highest rung of the ladder the
  role may change by itself. A change to a role is a new record with the next version.
- **One role, the steward.** It runs after every five finished tasks (a task that landed or was rejected),
  on its own thread, so the job that triggered it returns first. **The steward calls a model.** Its stored
  prompt (`STEWARD_PROMPT`, recorded with the role; version 2 is the first that calls a model) is the system
  prompt of an agent run through the same driver as the other roles. The agent reads and reports work items;
  Athena validates them and queues them. The invocation is recorded (role, usage, cost) like any other.
- **Read-only, by the tool policy.** The agent gets Read, Grep, Glob and Bash. Bash allows `git status`,
  `diff`, `log`, `show`, `ls`, and `gh` reads only: `gh run list|view`, `gh pr list|view|status|checks|diff`,
  `gh workflow list|view`, and `gh api` with GET. The `gh` flags are an allow-list in
  `crates/athena-agent/src/policy.rs`: any word starting `-f` or `-F` (so `-ftitle=x` and `-Fx=@f` too),
  `--field`, `--raw-field`, `--input`, a method other than GET, graphql, headers, and every flag not listed are
  denied. Mutating `git` and `gh` commands are denied. Tests cover each denied form and the allowed reads.
- **A work item is a record** (`athena.work-item/1`). It names its source and cites its evidence by
  reference (a job id, a path, or a commit), never by copy. It has a kind (`blind-spot`,
  `repeated-failure`, `proposal`) and the rung it concerns.
- **Deciding a proposal.** A work item of kind `proposal` is open until a person decides it, by its id,
  which `athena status` lists. `athena approve-work <id>` submits an ordinary job through the usual reading.
  Readers cannot see `records/`, so a request that only named the work item would be read blind; the request
  is built from the work item instead: its title, rung, source and key, and the content of the source and of
  each cited reference copied in as text (a job's request, reading and outcome; a file's text, clipped at
  6,000 characters; a directory's file list; a commit's message). `athena reject-work <id> --why "..."`
  records the reason. Work items are never edited, so each decision is an appended
  `athena.work-decision/1` record (`records/work-decisions/<YYYY-MM>/<id>.json`) naming the work item and
  either the approving job or the rejection reason. An item with a decision is no longer open; a second
  decision, an unknown id and an empty reason are refused. Only proposals are decided this way; blind spots
  and repeated failures are not.
- **What `athena drain` decides.** `drain` approves an open proposal itself only when the role that raised it
  may change its rung (the role's newest recorded definition gives the limit, and the top two rungs are never
  changed by a role). It approves as `approve-work` does, then runs the job that the decision records, and it
  also runs any job a work decision records as approved that is still in state proposed (read once, never
  read again; a closed job is not picked up). Rejecting a proposal, and approving one above its role's limit
  or raised by a role with no recorded definition, stay with people: `drain` lists those as waiting.
- **Planning comes first, and the drain follows one plan** (`athena_core::plan`, D105). Before design or code,
  each open request is fleshed out as a plan entry: goal, ladder rung, acceptance, open questions, dependencies
  and touched paths. The rung is the proposal's, else what the request serves in BRIEF.md and its addenda
  (a request that shares at least two significant words with a requirement heading or an addendum ranks as a
  brief requirement; plain code work with a touched path is Implementation; a word such as "fix" is no
  evidence), so the owner's newest priorities order ahead of older plain work (D107). Once a request is read, these come from its reading (summary, criteria, question,
  deliverables); before that, the request's own words stand in (first line as goal; bullet lines, else the first
  line, as acceptance; path-like words as touches; full job ids it names as dependencies). All open entries
  are planned together: items whose dependencies are done form a wave; a wave is ordered by rung (top first),
  then age; within it, items with no shared dependency and disjoint touched paths form a parallel group (an item
  that names no path is never grouped). Items that wait on each other in a loop are not scheduled. An entry
  without a goal, a rung or acceptance is refused for design or code and reported. `drain` works the items in
  plan order; the items of one group run at once, up to `[parallel] max_concurrency` (default 1, one job at a
  time), and the next group starts when the group before it is done.
- **Stopping a drain.** `drain` asks before every job whether a stop was requested (`athena drain-stop` creates
  a file in the repository's state directory). A job that has started is finished, landed and rated; no further
  job starts. The stop reason is `stop requested`, distinct from the kill switch (`athena stop`), which ends
  running work.
- **The plan-stage audit (first slice).** Every logged gap (an intervention, an introspection, a rejection, a
  rating) is traced to the stage that let it through: reading, decomposition, assumptions, criteria or outcome
  check (`plan::trace`, by the words of the gap's account; a person can overrule it). `plan::GapLog::fell`
  then compares gaps of a stage per job before and after a change to that stage, so a change counts as working
  only if later gaps of that kind fell. Follow-up: reading the gaps and their accounts from the records
  automatically, a report on `athena status`, and a role that proposes the stage changes; none of that is built
  yet.
- **The ladder** (`Ladder`) has five rungs, highest first:

  | Rung | What it holds |
  |---|---|
  | `owner-goals` | what the owner wants Athena to be for |
  | `brief-requirements` | what `BRIEF.md` requires |
  | `design-decisions` | the recorded design decisions |
  | `defaults-conventions` | the user's defaults and the repository's conventions |
  | `implementation` | code, tests and docs |

  `may_change(limit, rung)` is true only when the rung is at or below the role's limit and is not one of
  the top two. A role's limit can never lift it above design decisions. The steward's limit is
  `defaults-conventions`, so in this build it changes the lowest two rungs and only proposes the others.

## How a run goes

1. `go` counts a landed or rejected job. The count is a number in the state directory. At five the count
   starts again and the steward is spawned. `go` does not wait for it, and the command binary waits only
   after its own work is done, so the run is not cut off at exit.
2. Athena builds the agent a workspace through a `View` and starts it. The workspace holds the code as the
   producer sees it (no private paths, no `records/`), a copy of `records/`, and `GIT-LOG.txt`. The brief
   names the sources: `jobs`, `events` (interventions, restarts, host errors, resolutions), `introspections`,
   `class-registry`, `decisions`, `git-log`, `code`, and `github` (read with `gh`). Athena also passes what
   it counted: the classes with two or more rejected or failed jobs (the old repeated-failure count, now an
   input and not a finding), and the keys already queued.
3. The agent reports `items`: kind, key, title, rung, source, evidence (job ids, paths, commits), and a request
   text. Athena validates each one: the sources and evidence must be well formed, a cited job must exist, a
   key has a fixed alphabet, and an item on a rung it may change needs a request. An invalid item is dropped
   and reported, and does not stop the others.
4. **Blind spots.** A source the agent could not read (GitHub unavailable, say), a source Athena could not
   write to the workspace, and a file under `records/` that does not parse as a record each become a
   `blind-spot` item. Its key is `blind-spot:<source name>`, made by Athena from the stable name and never
   from error text, so the same failure on the next run has the same key.
5. **Every item goes through `decide()`.** The ladder decides it:
   - at or below the role's limit, the change is applied by queueing a request (the same record
     `athena ask --later` writes, on the channel `steward` instead of `cli`). It is not run by the steward: it
     runs when someone runs `athena go` or `athena drain`, under
     the same checks and the independent verifier as any job. The work item cites the queued request. A
     blind spot is on the lowest rung, so its request asks for the source to be made readable;
   - above the limit, only the work item is queued, as a proposal with its evidence. Nothing is applied.
6. Each finding has a key, and a key already on a work item is not queued again (the brief also lists the
   keys, so the agent need not repeat them). A run that queues nothing makes no commit; its invocation record
   waits in the state directory (`steward-pending`) and is committed with the next run's items.
7. A nested run never starts the steward and `run` does nothing there, so no model call happens outside a job.

The steward never edits `defaults.toml`, `athena.toml`, `BRIEF.md`, or the owner's goals.

## Alternatives considered

- **Copy Athena's state into a new store for the steward.** Rejected: it duplicates what the records already
  say and goes stale. The steward's workspace is a fresh copy made for each run and removed after it, and
  only its findings are recorded.
- **Run the steward in a separate process.** Rejected for now: a thread needs no new process protocol. The
  agent driver is now shared between threads (`AgentDriver: Send + Sync`, `Athena.driver` is an `Arc`), and
  the thread builds its own `Athena` handle on the repository, the state directory, and that driver.
- **Keep counting repeated failures in code and let the model confirm.** Rejected: the count is a heuristic.
  It is passed to the model as an input, and the model decides what repeats.
- **Count by time instead of tasks.** Rejected: the addendum says tasks, and an idle repository then costs
  nothing.
- **Let the steward edit files directly on the lower rungs.** Rejected: queueing a request keeps every change
  under the checks and the verifier.
- **Tie the ladder to a prompt rule.** Rejected: `may_change` is a function with tests, so the limit holds
  whatever a prompt says.

## Judging configuration by the candidate's schema

A commit may add a section to `athena.toml` together with the schema field that accepts it. The running
Athena knows only the parent's schema, so parsing the new file with it refuses the commit. The landing gate
(`athena-gate` `content.rs`) and admission (`athena-core` `admission.rs`) both had this flaw.

Decision: when a commit changes `crates/athena-data/` (and does not only delete files there), neither check
parses `athena.toml` or `defaults.toml`. The test `crates/athena-data/tests/repo_config.rs` does, with the
schema of the commit's own tree, and it runs in the candidate's normal `cargo test`, where candidate code is
built. When that test does not run or fails, the existing check and test gate refuses the candidate. One
function, `athena_gate::defers_to_schema_test`, decides this for both checks. When the schema is unchanged,
the running schema is the parent's, and both checks parse the file directly.

Alternatives:

- **Build the candidate's `athena-data` on the host** (the earlier admission fix). Rejected: it compiles
  candidate-supplied code, including build scripts, outside the isolation that agent code runs in, and it
  left the landing gate unfixed.
- **Build it in an isolated environment from the checks.** Rejected: the isolated run of `cargo test` already
  exists; a second build only duplicates it.
- **Parse leniently** (ignore unknown fields). Rejected: a misspelled field would then pass silently, and
  the schema's strictness is deliberate.

Candidate code is never built on the host because whoever writes the candidate controls what that build
runs. A schema check must not be a way to run it.

## The auditor

The verifier judges a change against the acceptance criteria the reader wrote for that job, so a job is partly
graded by a yardstick its own reading set. The auditor is a second grader whose yardstick a job cannot set.
`athena audit [JOB]` (default `latest`) judges a job's real outcome against `BRIEF.md` and the owner's goals.

- **What it is given.** `BRIEF.md` in its brief, `OUTCOME.md` (the request and how the job ended), and the
  files the job changed, as they stood in its commit, under `outcome/`. It is not given the job's acceptance
  criteria, its verification, its findings or any other record about it, so it looks at what the job did, not
  at what records say about it.
- **What it may do.** Read, with `Read`, `Grep` and `Glob`; no commands; it changes nothing. It borrows the
  verifier's model and budget, as the steward and the audience do, and its run is charged to no job.
- **What it returns.** A summary and findings, each with `text` and `evidence`. A finding with no evidence
  refuses the whole report, and the invocation is still recorded because it cost money. A good report becomes
  an `athena.audit/1` record (`records/audits/<YYYY-MM>/<id>.json`), committed with the auditor's invocation
  and, the first time, its role definition. `athena audit` prints the findings and the path of the recorded
  invocation; `athena status` shows the latest audit.
- **How its prompt is protected.** Only by being a fixed constant (`AUDITOR_PROMPT` in
  `crates/athena-data/src/roles.rs`), stored with the role as a versioned record. Nothing stops a job from
  editing that file and so changing the yardstick it is judged by. The warden request will enforce this by
  mechanism. Until then the protection is **prose-only**: written here and in the module comment, and checked
  by no code.

## Limits

- **Git history.** The agent sees `GIT-LOG.txt`, the hash, date and subject of the last 200 commits, and a
  one-commit snapshot of the code. It does not get the full object history, because private paths must stay
  hidden. A subject line that names a private path would appear in the log file.
- **Model and budget.** The steward borrows the verifier's model and its per-invocation cost cap and timeout
  (`[models] verifier`, `[budget] verifier_usd` and `verifier_timeout_s` in `athena.toml`). It is not charged
  to any job. A steward-specific model and budget is future work.
- **GitHub.** `gh` needs a token the agent's environment carries. `athena.toml` `[agents] env_pass_if_set`
  lists what is passed, and the steward is only as able to read GitHub as that list allows. It passes
  `GH_TOKEN`, `GITHUB_TOKEN`, `GH_HOST` and `GH_ENTERPRISE_TOKEN`, each only when set (D96). The owner should
  export a read-only token before running Athena; the token reaches every agent role, so it must not have
  write scope. Without a token the steward reports the `github` blind spot.
- **Not run live.** The tests drive the steward with the fake driver. The real driver and the real
  Claude Code hook have not run it.

## Left for later

Further roles; cost and yield tracking, reshaping and retiring a role (the rest of A10-1 and A9-2); closing a
work item once it is done (today an item stays, and its key stops it being raised again); a second source
besides the records, such as the landing history; and a steward-specific model and budget. If the steward
fails, its error is written to `steward-error.txt` in the state directory; a run that fails after the model
answered still keeps its invocation record.
