# Job 20261102-171907-ab85: Say who Athena is for and what it solves on homepage and README

## Request (verbatim)

> The homepage never says who Athena is for or what problem it solves; 11 audience ratings flag this, and README.md does not say it either
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261102-132626-18vs (key homepage-never-says-who-athena-is-for-flagged-repeatedly), queued at 2026-11-02T13:26:26.303354781Z by the steward role.
> Rung of the requirements ladder: brief-requirements.
> 
> ## Source
> 
> path records/audience-ratings/2026-11/20261102-113512-jpxc.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261102-113512-jpxc",
>   "at": "2026-11-02T11:34:44.598013844Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 3,
>   "findings": [
>     {
>       "text": "The opening paragraph says what Athena is in one plain sentence. A newcomer can name the product without scrolling.",
>       "source": "<p class=\"lede\">Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository."
>     },
>     {
>       "text": "This is Athena's own site, but almost the whole page is about another project, firstmate. Athena gets about three sentences. A newcomer who came to learn about Athena has to work out that the rest of the page is a sample of its output, not documentation about Athena.",
>       "source": "<title>Athena explains how firstmate handles work</title>"
>     },
>     {
>       "text": "The page says why the study exists, but never says why a newcomer should care about Athena: who it is for, or what problem it solves for them. The demo only proves traceability.",
>       "source": "Athena made this study of firstmate to show that it can read an unfamiliar project and explain it with every statement traced to a claim, and every claim to lines in the project's own files."
>     },
>     {
>       "text": "The page promises these features, but nothing on it shows them. No checker model, cost record or request reading appears, so a newcomer has to take the headline features on trust.",
>       "source": "It shows how it read the request, has one AI model do the work and a different one check it, and records what each step cost."
>     },
>     {
>       "text": "There is a clear next-step link near the top and a Next steps section. But the install step gives no clone command, no prerequisites (for example a Rust toolchain or AI model access) and no hint of what `athena ask` will do to a repository. A newcomer would have to leave the page to start.",
>       "source": "Next steps — \"Install Athena from a checkout of its repository: ./setup.sh\" and \"athena ask \"add a changelog\"\""
>     },
>     {
>       "text": "The firstmate sections assume a lot of background: exit codes, places-held, pending-close records, captain calls. A newcomer gets lost quickly. The 'Start here' section and the glossary help, but the transition cards are written for someone already studying firstmate's internals.",
>       "source": "Transitions — \"Every worker place is held, so the spawn is deferred with exit 75 and the item stays queued.\""
>     },
>     {
>       "text": "The same terms (yolo, red, green, in scope) are defined in full both in 'Words used first' and in the Glossary, and restated again in the bullets and cards. This makes a long page longer without adding anything.",
>       "source": "Start here — \"Words used first\" and Glossary entries for yolo, red, green"
>     },
>     {
>       "text": "Every statement links to a claim, and every claim links to exact lines at a pinned commit. This is a convincing, checkable demonstration of what the page says Athena can do.",
>       "source": "Claims and sources — e.g. \"bin/fm-spawn.sh:194-197\" links at the pinned commit"
>     },
>     {
>       "text": "It is good that the page admits the study's limits. But a note about an earlier draft missing paths is internal process detail, and it may lower a newcomer's confidence instead of informing them.",
>       "source": "Limits of this study — f3: \"A first reading of the pinned tree missed these paths\""
>     },
>     {
>       "text": "This section doesn't say which repository the `studies/` folder belongs to, so a newcomer cannot follow it.",
>       "source": "About this page — \"is the name of the folder under studies/ that holds the data this page is generated from.\""
>     }
>   ]
> }
> ```
> 
> ## Evidence
> 
> ### 1. path records/audience-ratings/2026-11/20261102-113512-jpxc.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261102-113512-jpxc",
>   "at": "2026-11-02T11:34:44.598013844Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 3,
>   "findings": [
>     {
>       "text": "The opening paragraph says what Athena is in one plain sentence. A newcomer can name the product without scrolling.",
>       "source": "<p class=\"lede\">Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository."
>     },
>     {
>       "text": "This is Athena's own site, but almost the whole page is about another project, firstmate. Athena gets about three sentences. A newcomer who came to learn about Athena has to work out that the rest of the page is a sample of its output, not documentation about Athena.",
>       "source": "<title>Athena explains how firstmate handles work</title>"
>     },
>     {
>       "text": "The page says why the study exists, but never says why a newcomer should care about Athena: who it is for, or what problem it solves for them. The demo only proves traceability.",
>       "source": "Athena made this study of firstmate to show that it can read an unfamiliar project and explain it with every statement traced to a claim, and every claim to lines in the project's own files."
>     },
>     {
>       "text": "The page promises these features, but nothing on it shows them. No checker model, cost record or request reading appears, so a newcomer has to take the headline features on trust.",
>       "source": "It shows how it read the request, has one AI model do the work and a different one check it, and records what each step cost."
>     },
>     {
>       "text": "There is a clear next-step link near the top and a Next steps section. But the install step gives no clone command, no prerequisites (for example a Rust toolchain or AI model access) and no hint of what `athena ask` will do to a repository. A newcomer would have to leave the page to start.",
>       "source": "Next steps — \"Install Athena from a checkout of its repository: ./setup.sh\" and \"athena ask \"add a changelog\"\""
>     },
>     {
>       "text": "The firstmate sections assume a lot of background: exit codes, places-held, pending-close records, captain calls. A newcomer gets lost quickly. The 'Start here' section and the glossary help, but the transition cards are written for someone already studying firstmate's internals.",
>       "source": "Transitions — \"Every worker place is held, so the spawn is deferred with exit 75 and the item stays queued.\""
>     },
>     {
>       "text": "The same terms (yolo, red, green, in scope) are defined in full both in 'Words used first' and in the Glossary, and restated again in the bullets and cards. This makes a long page longer without adding anything.",
>       "source": "Start here — \"Words used first\" and Glossary entries for yolo, red, green"
>     },
>     {
>       "text": "Every statement links to a claim, and every claim links to exact lines at a pinned commit. This is a convincing, checkable demonstration of what the page says Athena can do.",
>       "source": "Claims and sources — e.g. \"bin/fm-spawn.sh:194-197\" links at the pinned commit"
>     },
>     {
>       "text": "It is good that the page admits the study's limits. But a note about an earlier draft missing paths is internal process detail, and it may lower a newcomer's confidence instead of informing them.",
>       "source": "Limits of this study — f3: \"A first reading of the pinned tree missed these paths\""
>     },
>     {
>       "text": "This section doesn't say which repository the `studies/` folder belongs to, so a newcomer cannot follow it.",
>       "source": "About this page — \"is the name of the folder under studies/ that holds the data this page is generated from.\""
>     }
>   ]
> }
> ```
> 
> ### 2. path records/audience-ratings/2026-10/20261031-213352-hnwb.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261031-213352-hnwb",
>   "at": "2026-10-31T21:33:23.402631464Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 3,
>   "findings": [
>     {
>       "text": "The first lede tells a newcomer what Athena is in plain words. After that, the page says almost nothing more about Athena itself, and the only demo of it is this study.",
>       "source": "\"Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository. It shows how it read the request, has one AI model do the work and a different one check it, and records what each step cost.\""
>     },
>     {
>       "text": "This is the root URL of Athena's site, but the title and almost all of the content are about a different project, firstmate. A newcomer who came to learn about Athena may be confused about which project the page is for.",
>       "source": "<title>Athena explains how firstmate handles work</title> / h1 of the same text"
>     },
>     {
>       "text": "This gives a clear reason the page exists: every statement can be traced to a source. The claim ids, which link to line ranges at a fixed commit, back that up throughout the page.",
>       "source": "\"Athena made this study of firstmate to show that it can read an unfamiliar project and explain it with every statement traced to a claim, and every claim to lines in the project's own files.\""
>     },
>     {
>       "text": "The page links to next steps right at the top. The Next steps section gives the repo link, an install command and a sample request, so the reader knows what to do.",
>       "source": "\"Next step: install Athena or read firstmate itself\" and the Next steps section"
>     },
>     {
>       "text": "The install step assumes the reader already has a checkout. There is no clone command, and the page does not say what setup.sh needs or what you get after running it.",
>       "source": "\"Install Athena from a checkout of its repository: ./setup.sh\""
>     },
>     {
>       "text": "The walkthrough meant for newcomers uses words it hasn't explained yet. 'Unheld' is never defined, and exit codes show up in step 2. This makes the start-to-finish story hard to follow on a first read.",
>       "source": "Start here > One piece of work: \"A spawn is accepted only for an unheld, unblocked item.\" / \"the spawn exits 75\""
>     },
>     {
>       "text": "Someone who came for firstmate gets a clear answer about what it is and what problem it solves, and it comes before the detailed sections.",
>       "source": "Start here > What firstmate is: \"Firstmate is a tool for people who direct AI coding agents... so that several tasks can run side by side without the workers overwriting each other's files\""
>     },
>     {
>       "text": "Many definitions appear two or three times: in the 'Words used first' list, again in the Glossary, and again inside the cards. This makes the page longer and denser than a first-time reader needs.",
>       "source": "Words used first, Glossary, and Transitions cards (e.g. \"in scope\" defined identically in both lists)"
>     },
>     {
>       "text": "The page says plainly that the study only read firstmate's files and never ran it. This helps a newcomer judge how far to trust what the page says.",
>       "source": "Limits of this study: \"every claim is marked read\" / \"no run confirmed that a script behaves as its header says\""
>     }
>   ]
> }
> ```
> 
> ### 3. path records/audience-ratings/2026-10/20261030-125228-0v0t.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261030-125228-0v0t",
>   "at": "2026-10-30T12:51:56.871266638Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 3,
>   "findings": [
>     {
>       "text": "The first lines tell a newcomer what Athena is in plain words. That is all the page says about Athena itself. Who it is for, what it needs to run (models, keys, platforms), and why to pick it over other tools are never said.",
>       "source": "\"Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository. It shows how it read the request, has one AI model do the work and a different one check it, and records what each step cost.\""
>     },
>     {
>       "text": "This is the Athena site, but nearly all of the page is a deep study of a different project, firstmate. A newcomer who came to learn about Athena has to work out from the third lede paragraph that the page is a demonstration, not Athena's documentation.",
>       "source": "<h1>Athena explains how firstmate handles work</h1>"
>     },
>     {
>       "text": "The page does say why it matters and then delivers on it. Every statement carries claim ids, and each claim links to exact file lines at a pinned commit, so a reader can check the selling point for themselves.",
>       "source": "\"Athena made this study of firstmate to show that it can read an unfamiliar project and explain it with every statement traced to a claim, and every claim to lines in the project's own files.\""
>     },
>     {
>       "text": "The next step is easy to find: the header links to it and a section at the end lists it. But the install instructions leave out how to get the checkout and what to have installed first. A newcomer has to go to the GitHub repository to actually start.",
>       "source": "\"Next step: install Athena or read firstmate itself.\"; Next steps: \"Install Athena from a checkout of its repository: ./setup.sh\""
>     },
>     {
>       "text": "The 'Start here' section is the most readable part for a newcomer. It defines the key terms before using them, gives a one-sentence summary of firstmate, and walks through one numbered task from start to finish.",
>       "source": "Start here: \"The three parts below say what firstmate is, follow one piece of work, and say what happens when something fails or has no limit.\""
>     },
>     {
>       "text": "Several terms are used without being defined, even though the glossary is long: endpoint, durable record, heartbeat (claim c9), crewmate (c44), and harness pin. A newcomer stalls on these in the transition cards.",
>       "source": "t8 card: \"a secondmate re-reads its pin and needs none, and a secondmate with no endpoint is recovered by the spawn script instead\"; t1 summary: \"the prior durable record is restored\" (from 'When things go wrong')"
>     },
>     {
>       "text": "The state, counter and transition sections are written for someone reasoning about firstmate's internals, such as exit codes and lock behaviour. They are thorough but far too detailed for a first visit, and nothing says a newcomer can skip them.",
>       "source": "Transitions: \"the spawn script finds every capacity place held (decided by code)\" ... \"capacity full (exit 75)\""
>     },
>     {
>       "text": "The page is open about what it does not know: it flags gaps in the source and says the study read the code but never ran it. A newcomer trying to judge whether Athena can be trusted gets an honest picture.",
>       "source": "in scope: \"The cited instruction does not say what makes work in scope, so this page gives no test for it.\"; Limits of this study: \"no failure or retry edge was observed; every claim is marked read.\""
>     }
>   ]
> }
> ```
> 
> ### 4. path records/audience-ratings/2026-10/20261029-155924-5c5w.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261029-155924-5c5w",
>   "at": "2026-10-29T15:58:54.537689497Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 3,
>   "findings": [
>     {
>       "text": "The first sentence plainly says what Athena is, so a newcomer can name it right away.",
>       "source": "Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository."
>     },
>     {
>       "text": "This one line is the only explanation of why Athena matters. It never says what problem it solves or how it differs from other AI coding tools, so the 'why' is thin.",
>       "source": "It shows how it read the request, has one AI model do the work and a different one check it, and records what each step cost."
>     },
>     {
>       "text": "The page is at Athena's own address, but nearly everything below the header describes firstmate, another project. A newcomer who came to learn about Athena gets one paragraph on it and many screens on something else.",
>       "source": "This page is that study of firstmate."
>     },
>     {
>       "text": "A next-step link sits at the top and jumps to a clear Next steps section, so the newcomer knows where to go without scrolling through the study.",
>       "source": "Next step: install Athena or read firstmate itself"
>     },
>     {
>       "text": "The install step assumes you already have a checkout and gives no clone command. It also lists no requirements, such as a Rust toolchain or AI model access.",
>       "source": "Install Athena from a checkout of its repository: ./setup.sh"
>     },
>     {
>       "text": "A concrete first command is given, which helps. It doesn't say what the command will do to your repository or what you'll see afterwards.",
>       "source": "Try it with a one-line request: athena ask \"add a changelog\""
>     },
>     {
>       "text": "Key terms (captain, yolo, green, red) are defined before they are used. This makes the firstmate sections easier to follow, though the definition of 'in scope' admits that it has no test for what counts.",
>       "source": "Words used first"
>     },
>     {
>       "text": "Eleven cards (t1 to t11), plus the exit codes 75 and 1, the counter name 'places-held' and claim ids on every sentence, make most of the page reference material. It is too dense for a first visit.",
>       "source": "Transitions"
>     },
>     {
>       "text": "Every statement can be traced to a claim and its source lines in firstmate, and the page explains how to follow these links. That shows a newcomer the care Athena takes over evidence.",
>       "source": "Each statement below ends with the id of the claim that supports it; the claims, with links to their sources, are listed at the end."
>     },
>     {
>       "text": "The page openly says nothing was run and that a first reading missed several paths. That honesty helps trust, but it also tells the newcomer the demo is not proven by running it.",
>       "source": "Limits of this study"
>     }
>   ]
> }
> ```
> 
> ### 5. path records/audience-ratings/2026-10/20261015-031143-p561.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261015-031143-p561",
>   "at": "2026-10-15T03:11:21.294603401Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 3,
>   "findings": [
>     {
>       "text": "Someone arriving at the Athena site URL never learns what Athena is. The page names only \"the study\" and a footer path, and nothing says who or what produced the study, or why.",
>       "source": "<title>How firstmate handles work</title>; \"A reading of firstmate at one pinned commit, from the study firstmate-work-handling.\""
>     },
>     {
>       "text": "This part works. Three short bullets and a six-step walk-through let a newcomer say what firstmate does and follow one item from Queued to Done.",
>       "source": "Start here > What firstmate is: \"Firstmate takes an item from a backlog and gives it to a worker, which does the work in its own isolated worktree\""
>     },
>     {
>       "text": "The page never says why firstmate or this study matters, or who should care. It never says what a reader might use firstmate for, or what problem the study answers.",
>       "source": "Start here: \"The three parts below say what it is, follow one piece of work, and define the words the rest of the page uses.\""
>     },
>     {
>       "text": "There is no next step. The repository appears only as plain code text, not a link, and nothing says to install, read the source, or see other studies. The claim sources such as \"AGENTS.md:206-206\" are not linked either.",
>       "source": "Pinned firstmate commit: <code>https://github.com/kunchenguid/firstmate.git</code>"
>     },
>     {
>       "text": "Some key terms stay unclear. The glossary never says whether the captain is a human user. \"Home\" is used in the glossary and in claims c1 and c15 but is never defined.",
>       "source": "Glossary: \"captain — The one who approves merges and local-only landings when yolo is off.\"; c1: \"refuses when this home has no item for the work\""
>     },
>     {
>       "text": "Guards marked \"none (checked by agent)\" in t5, t7 and t8 contradict themselves. The explanation comes only at the end, in Limits finding f2, and uses internal field names such as \"guard_evaluated_by\" that mean nothing to a newcomer.",
>       "source": "Transitions, t5: \"Guard: none (checked by agent)\"; Limits f2: \"These transitions carry agent there as a placeholder\""
>     },
>     {
>       "text": "This sentence is awkward and describes the page's layout instead of the content. Glossary entries tagged \"about this page\" (state, transition, claim, pin, mark) add more page-about-page material before the content.",
>       "source": "Transitions lead-in: \"The two deciders are explained below the cards' heading.\""
>     },
>     {
>       "text": "The page is honest that the study comes only from reading the code. That helps a careful reader judge how much to trust it.",
>       "source": "Limits of this study, f1: \"no failure or retry edge was observed; every claim is marked read\""
>     }
>   ]
> }
> ```
> 
> ### 6. path README.md
> ````
> # Athena
> 
> Athena is an agent orchestration system written in Rust. You give it a lazy, one- or two-line request.
> It shows how it read the request and what it predicts, before any work starts. It runs the job on a
> sub-agent in an isolated snapshot, has a different model verify the result independently, and lands a
> commit you can trace back to the request, the prediction, the models, the cost, and the verdict. It records
> every model call, scores its own predictions with proper scoring rules, and gates every commit with a
> secret scan.
> 
> This is v0, the walking skeleton. Its design is [`docs/design/athena-v0.md`](docs/design/athena-v0.md).
> 
> ## Quick start
> 
> ```sh
> ./setup.sh                        # prerequisites, Rust, gitleaks, athena, git hooks, Claude Code, credentials
> athena ask "add a changelog"      # shows the reading and predictions; Enter runs it
> ```
> 
> `./setup.sh` is idempotent and pinned: it reads every version from `athena.toml` and `rust-toolchain.toml`
> (rustup-init, which runs before any Rust can, is pinned in `setup.sh` itself), checks downloads against pinned
> SHA-256 sums, and reads credentials only to report whether Claude Code can
> authenticate. It never stores them. `athena.toml` currently pins no `[tools.sccache]` (D84), so setup skips
> that step; if a pin is added, setup builds it with `cargo install --locked` into the install directory (no
> digest; this can take minutes). It needs git 2.40 or later, a C linker, curl, and either Claude Code
> 2.1.299 or npm to install it. `./setup.sh --ci` also skips any pinned sccache (CI does not use it) and stops
> after building Athena.
> 
> `athena --version` prints the package version and the commit the binary was built from, for example
> `athena 0.1.0 (commit 0123abc…, dirty)`. A build that names no commit shows `unknown`, and any build that
> is not a clean commit shows `dirty`.
> 
> ## What happens when you ask
> 
> ```text
> athena ask "<request>"
>   reader (Sonnet, read-only snapshot)  → a reading: summary, criteria, deliverables, forecasts
>                                          committed before any work (you can correct it in one line)
> Enter, or athena go <job>
>   prediction commit                    → the accepted reading's job record and the producer's brief
>   producer (Haiku or Sonnet)           → edits in a detached snapshot with its own throwaway repository
>   candidate                            → built from the change list; admitted or refused
>   checks                               → fmt, clippy, tests, run by Athena under the base's configuration
>   verifier (Opus, never a producer)    → judges the exact candidate; never sees the producer's report
>   landing                              → the change plus every record, one commit, fast-forwarded
> ```
> 
> To retry a rejected or failed job, ask `athena ask "try again on <job>"` (the full id or a unique suffix; the form is
> matched case-insensitively at the start of the request). The reader then sees that job's request and its final
> verification's findings, redacted, in an "Earlier attempt" section. That section also shows the job's final
> reading (summary, acceptance criteria, deliverables) and the corrections and answers the user gave, and it says the
> attempt's changes are not in what the reader sees. The retry's producer starts from the base, except when the
> attempt was rejected with every check passed and only content criteria unmet: then it starts from that attempt's
> candidate (the base's files with the attempt's changes applied) and is sent the verifier's findings. The new candidate
> is always checked and verified in full and compared with the base, and a producer that changes nothing fails as an
> empty candidate. If other work has landed since, the retry starts from the base. A failed job (for example one refused at
> admission, which has no verification) also has its recorded reason carried there. Jobs that are landed, stale,
> in flight, or unknown are refused.
> 
> `athena retry <job>` is the short form: it does the same, and when the job has an introspection record it also shows
> the reader that record's cause, prevention and proposed next request, and the earlier cases of the same class with
> the preventions already tried, so a repeat is seen as a repeat. It takes `--yes` and `--json` like `ask`, and runs
> nothing until the reading is accepted.
> 
> Any request that names a job by its full id (like `20261012-075824-1x28`) anywhere in its text also shows the
> reader that job's request and its final verification's findings, redacted, in a "Referenced jobs" section, whatever
> the job's state. At most three jobs are carried; an id that matches no job is ignored. Unique suffixes are resolved
> only in the "try again on" form.
> 
> Exit codes: 0 when the job landed, 2 when it did not (rejected, failed, stale, or blocked), 1 when Athena
> itself erred.
> 
> | Command | What it does |
> |---|---|
> | `athena ask "<request>" [--yes \| --later]` | Read a request; on a terminal, Enter runs it, a typed line corrects it, `q` leaves it proposed. `--later` only queues the request, with no model call |
> | `athena retry <job> [--yes]` | Try a rejected or failed job again from its introspection record (proposed next request, cause, prevention, earlier cases of the same class); shown as `ask` shows a reading. Without an introspection record it is `athena ask "try again on <job>"` |
> | `athena propose` | Write a pending defaults proposal for each failure class whose introspections span at least 3 distinct jobs. Proposals are records; `athena status` lists the open ones with their sources, and `defaults.toml` is never edited by Athena |
> | `athena score-jev [--cut <time>] [--derive] [--json]` | Score Jev's recorded shadow answers against introspections that have a class label (`athena.class-label/1` records, from a person or derived from evidence): Brier skill over two references, sample size, interval or "no bound computed", and the promotion verdict for the items it can compute. It reports how many labels came from evidence, per kind. With too few lab
> [clipped: 18372 more characters]
> ````

## Reading (accepted by the user)

Add a plain "who it is for and what problem it solves" statement to the homepage (generated by xtask/src/site.rs from studies/firstmate-work-handling/intro.json) and to the opening of README.md, and make the Next steps list usable (clone, prerequisites, what athena ask does). Audience is newcomers judging the site; it addresses the 11 ratings flagging this gap.

Assumptions:
- Code survey: xtask/src/site.rs `render` builds the header ledes from intro.json `about.athena` and `about.study`, and the `next-steps` section from fixed strings; tests are in xtask/tests/site_page.rs (Next steps checked near line 189) and unit tests in site.rs. No record or data file is added under crates/, so check_record_file/AnyRecord are not touched; intro.json is an existing study data file whose edit must still pass the study validation tests.
- Existing code: xtask/src/site.rs, xtask/tests/site_page.rs and studies/firstmate-work-handling/intro.json are modified, not created.
- Audience: a newcomer (and the audience raters) arriving at https://doodla.github.io/athena/. Goal: they learn in the first screen who Athena is for (people who want a lazy request turned into a verified, traceable commit in their own repository) and what problem it solves (unreviewed, unaccountable AI edits), drawn only from README.md's own wording.
- Only the who/problem gap and the install step are fixed; the other rating findings (page length, duplicated glossary, firstmate jargon, limits wording) are left for later jobs per the small default.
- Governance files are not changed: athena.toml, defaults.toml, rust-toolchain.toml and setup.sh are named only because the README quick-start/prerequisite text refers to them; the job reads them for facts and edits none. No workflow under .github/workflows/ needs a change.
- No prior-job files are carried: job 20261012-075824-1x28 (landed, pass) is referenced only as context; its findings (missing test for the three-job cap) concern reading.rs and do not apply to this docs change, and its changed files (CHANGELOG.md, README.md, lifecycle.rs, briefs.rs, reading.rs) are kept as landed; only README.md and optionally CHANGELOG.md are touched again here.

## Acceptance criteria

1. [content] studies/firstmate-work-handling/intro.json `about.athena` (or a new `about` field rendered in the header) states in plain words who Athena is for and what problem it solves, within the first lede paragraphs of the page.
2. [content] README.md's opening paragraph (before 'Quick start') contains a sentence naming who Athena is for and the problem it solves.
3. [content] The Next steps section in xtask/src/site.rs gives a git clone command for the Athena repository, names prerequisites (git 2.40+, a C linker, curl, Claude Code or npm, as README.md states), and says what `athena ask` does (shows a reading and predictions; Enter runs it on the repository).
4. [behavior] Running `cargo test --locked` (including xtask/tests/site_page.rs, updated to assert the who-it-is-for text appears in the header before 'Start here' and the clone command and prerequisites appear in Next steps) passes.
5. [behavior] `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --check` pass.
6. [content] athena.toml, defaults.toml, rust-toolchain.toml, setup.sh and .github/workflows/ are unchanged in the diff (`git diff --stat`).
7. [content] CHANGELOG.md has an entry describing the homepage and README audience statement, following its existing format.
8. [content] Outcome check: a newcomer reading only the rendered page's header can answer who Athena is for and what problem it solves, which is exactly what the 11 ratings found missing; no secrets or personal data are added.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- xtask/src/site.rs (modify)
- xtask/tests/site_page.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- athena.toml, defaults.toml, rust-toolchain.toml and setup.sh: read for facts, not changed
- Shortening the page, deduplicating glossary terms, or simplifying the firstmate transition cards
- Rewording the Limits of this study section or the About this page section
- Adding a live demo of the checker model, cost record or request reading to the page
- Changes to reading.rs, briefs.rs, run.rs or lifecycle.rs from job 20261012-075824-1x28 (kept as landed)

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: Text is written for a newcomer outside the project.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Wording comes from README.md claims; prerequisites cite README facts.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Site tests updated; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the audience statement and install step only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks a newcomer can answer who/what-problem from the header.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert rendered text, not file existence.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data added.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

26 more qualifying classes are left out.

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

studies/firstmate-work-handling/intro.json `about.athena` (or a new `about` field rendered in the header) states in plain words who Athena is for and what problem it solves, within the first lede paragraphs of the page.
README.md's opening paragraph (before 'Quick start') contains a sentence naming who Athena is for and the problem it solves.
The Next steps section in xtask/src/site.rs gives a git clone command for the Athena repository, names prerequisites (git 2.40+, a C linker, curl, Claude Code or npm, as README.md states), and says what `athena ask` does (shows a reading and predictions; Enter runs it on the repository).
Running `cargo test --locked` (including xtask/tests/site_page.rs, updated to assert the who-it-is-for text appears in the header before 'Start here' and the clone command and prerequisites appear in Next steps) passes.
`cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --check` pass.
athena.toml, defaults.toml, rust-toolchain.toml, setup.sh and .github/workflows/ are unchanged in the diff (`git diff --stat`).
CHANGELOG.md has an entry describing the homepage and README audience statement, following its existing format.
Outcome check: a newcomer reading only the rendered page's header can answer who Athena is for and what problem it solves, which is exactly what the 11 ratings found missing; no secrets or personal data are added.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
