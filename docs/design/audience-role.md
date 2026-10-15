# The audience role

The audience is a role beside the steward (`roles-and-ladder.md`). The steward reads what Athena has done.
The audience reads what Athena shipped, as the people it was shipped for would meet it, and says whether it
works for them. Its first target is the live explainer page (`explainer-site.md`), read as a newcomer.

## What it does

- It reads and records only. It changes no code, no docs and no page. Its tools are `Read`, `Grep` and `Glob`
  over a workspace that holds one file, `PAGE.html`, the page as fetched. It has no commands and no write
  access, enforced by the same tool policy as every other role.
- It gives one rating from 1 (the reader cannot use the page) to 5 (the reader got what they came for) and a
  list of findings. Each finding cites its evidence: a passage quoted from the page, or the heading where it
  was seen.
- Athena validates the report before recording it. A rating outside 1 to 5 is refused, and so is a finding
  with no source. A refused report records no rating; the invocation record is kept, because it cost money.

## The definition

`athena_data::roles::audience` defines it: name `audience`, version 1, a goal, a fixed prompt
(`AUDIENCE_PROMPT`), a cadence of 20 finished tasks, and the highest rung it may change, `implementation`.
Its record is `records/roles/audience-1.json`; it is written with the first rating. The prompt is stored, so a
change to it is a new version. The cadence is recorded for a later scheduler; today the role runs on request.

## Running it

```text
athena audience                              # rate the live explainer as a newcomer
athena audience --url <page> --reader <who>  # rate another page, read as someone else
athena audience --json
```

The command fetches the page with a plain GET (an `http` or `https` URL only) through `gh.rs`, the one place
Athena makes external calls, which a test guards. It then calls the model once with the
verifier's model and budget, and commits one records commit. It needs network access and the model's
credentials; the tests need neither, because they give the run a stub page and the fake agent.

## Where its records go

| Record | Path |
|---|---|
| Rating (`athena.audience-rating/1`) | `records/audience-ratings/<YYYY-MM>/<id>.json` |
| The role definition | `records/roles/audience-1.json` |
| The model invocation | `records/jobs/<YYYY-MM>/<run id>/invocations/01-audience.json` |
| Work item for a low rating (`athena.work-item/1`) | `records/work-items/<YYYY-MM>/<id>.json` |

## What reads the rating

- **The steward's brief** has an "Audience rating" section with the latest rating, its findings and their
  evidence, so the steward does not have to find it. When that rating raised a work item, the section names its
  key so the steward does not report it again.
- **`athena status`** lists the latest rating and findings (`--json` has them as `audience`).
- **A rating of 2 or lower** queues a work item of kind `proposal`, role `audience`, rung `implementation`,
  key `audience:<rating id>`. Its source and evidence are the rating's path. It appears in `athena status`
  with the other open proposals, and a person decides it with `athena approve-work` or `athena reject-work`.
  Nothing is queued as a request on its own, so the audience never starts a change.

## Adding a target beyond the explainer

The explainer's URL is one constant, `athena_core::audience::EXPLAINER_URL`, and `Target::explainer()` pairs it
with the reader `newcomer`. For a one-off, give `--url` and `--reader`. For a standing target, add a
constructor beside `Target::explainer()` and a command or flag that selects it. The record needs no change: it
stores the URL and the reader. The brief asks whether the reader can say what the page is, why it matters and
what to do next; a target that needs other questions needs its own brief function in `audience.rs`.

## Not done

- The role is not started by a cadence yet. A person runs `athena audience`.
- Only pages reachable by a plain `curl` can be read; a page that needs a login or runs script to render is
  rated as fetched.
