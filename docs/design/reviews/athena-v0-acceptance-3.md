# Athena v0: independent acceptance verification 3 (N1 re-check)

**Verdict: N1 is resolved.** A `[history] exempt` entry can no longer hide a hand change from the history
check.

| | |
|---|---|
| Under test | Branch `claude/happy-goodall-j46p5t` at `01f9eb52a7c942f90369f28befda164044f597ae`, with the fix in `b50b5eb`, `615efc1`, and `01f9eb5` (D60), cloned fresh from GitHub |
| Scope | Only N1 of [verification 2](athena-v0-acceptance-2.md) |
| Verifier | The author of verifications 1 and 2: an agent with no part in Athena's design or build |
| Method | Probes in a throwaway clone, using the hooks and the binary that setup built from this commit, after closing the bootstrap range there. No model calls. |
| Spend | $0.00 |

| # | Probe | Outcome |
|---|---|---|
| 1 | Round 2's probe: a declared intervention (`athena intervene --with-staged`) adds `"0"` to `[history] exempt` | **Refused**: "exempt entry "0" is not a full commit id" |
| 1b | The same entry and intervention record committed with `--no-verify`, then a hand edit of `README.md` committed with `--no-verify` and retried until its id began with `0` (51 tries) | **Caught**: `athena check --history` reports the configuration that does not parse (three violations) and the hand edit (one) |
| 2 | A full-length id that names no commit | **Refused**: "not a commit already in this history" |
| 3 | The full id of a commit that exists only on another branch | **Refused**, with the same reason |
| 4 | Exempting a hand change after the fact: the edit committed with `--no-verify`, then a declared intervention adding that commit's full id | The entry is accepted, since the commit is in history, but **has no effect**: the history check still flags the edit, because exemptions are matched against the configuration of the exempted commit's parent |
| 5 | A legitimate entry: the full id of an earlier commit, declared | **Accepted**; the history check stays clean. At the head, the existing entry (the commit that predates Athena) still parses, and `athena check --history` reports 73 commits and 0 violations |

One observation, below major severity. Probes 4 and 5 show that an entry added in a commit never exempts
anything. It can only name an earlier commit, and that commit is judged by its parent's configuration, which
predates the entry. Only entries already in the built-in configuration take effect, which today means only
this repository's first commit. So an adopter could not exempt history from before Athena by adding entries.
That was already so before the fix, because matching has always used the parent's configuration. It does not
weaken N1's fix.
