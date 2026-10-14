# Roles, the steward, and the requirements ladder

This note covers the first build of the `BRIEF.md` addendum "2026-10-14: A loop that steers Athena, roles, and
a ladder of requirements" (rows A9-1 and A10-1 of `docs/design/requirements-map.md`). It is small on purpose:
one role, one ladder. Both rows are Partial in the map: A9-1 because the steward never calls a model, and
A10-1 because cost and yield tracking, reshaping and retiring are not built.

## What was built

- **A role is a recorded definition** (`athena.role/1`, `crates/athena-data/src/roles.rs`): a name, a version,
  a goal, a short stable prompt, a cadence counted in finished tasks, and the highest rung of the ladder the
  role may change by itself. A change to a role is a new record with the next version.
- **One role, the steward.** It runs after every five finished tasks (a task that landed or was rejected),
  on its own thread, so the job that triggered it returns first. It reads Athena's state where it lives,
  looks for repeated failures, and queues work items. Its prompt is recorded with it. In this build the
  steward is code, not a model session, so the prompt states its job for the reader and for a later build
  that gives it a model.
- **A work item is a record** (`athena.work-item/1`). It names its source and cites its evidence by
  reference (a job id, a path, or a commit), never by copy. It has a kind (`blind-spot`,
  `repeated-failure`, `proposal`) and the rung it concerns.
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
2. The steward reads the records at the current commit through a `View`. It records only what would be lost:
   the work items it queues, and its own definition the first time it runs. It copies nothing else.
3. **Blind spots.** A source it cannot read becomes a `blind-spot` item whose `source` names it. So does a
   file under `records/` that does not parse as a record.
4. **Repeated failures.** Two or more jobs of one class of work that ended rejected or failed make a proposed
   change on the `defaults-conventions` rung. The ladder decides it:
   - at or below the role's limit, the change is applied by queueing a request (the same record
     `athena ask --later` writes). It is not run by the steward: it runs when someone runs `athena go`, under
     the same checks and the independent verifier as any job. The work item cites the queued request;
   - above the limit, only the work item is queued, as a proposal with its evidence. Nothing is applied.
5. Each finding has a key, and a key already on a work item is not queued again. A new failure in the class
   changes the key, so it is raised again with the new evidence.

The steward never edits `defaults.toml`, `athena.toml`, `BRIEF.md`, or the owner's goals.

## Alternatives considered

- **Copy Athena's state into a new store for the steward.** Rejected: it duplicates what the records already
  say and goes stale. The steward reads in place and records only findings.
- **Run the steward in a separate process.** Rejected for now: a thread needs no new process protocol. The
  agent driver is not shared between threads, so the steward holds only the repository and state directory
  (`Committer` in `crates/athena-core/src/commits.rs`), and needs no driver.
- **Count by time instead of tasks.** Rejected: the addendum says tasks, and an idle repository then costs
  nothing.
- **Let the steward edit files directly on the lower rungs.** Rejected: queueing a request keeps every change
  under the checks and the verifier.
- **Tie the ladder to a prompt rule.** Rejected: `may_change` is a function with tests, so the limit holds
  whatever a prompt says.

## Left for later

Further roles; cost and yield tracking, reshaping and retiring a role (the rest of A10-1 and A9-2); closing a
work item once it is done (today an item stays, and its key stops it being raised again); a second source
besides the records, such as the landing history; and a model-backed steward. If the steward fails, its error
is written to `steward-error.txt` in the state directory.
