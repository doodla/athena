# Preflight and sandbox blockers

Sandbox-side facts for the operator. Athena never reads this file. Each check names how it was
made, so a later session can re-run it. Re-check every open blocker at each restart.

## Preflight of 2026-10-10 (session `cse_01BK4zqW5z6Xh8mk2bEuC9W1`)

| Capability | Result | Evidence |
|---|---|---|
| Push access to `doodla/athena` | Available | `gh api repos/doodla/athena` reports `push: true, admin: true` for the session; first push of this branch succeeded (see the commit that added this file). |
| GitHub Pages, Actions as source | **Blocked** | `gh api repos/doodla/athena` reports `has_pages: false`. The Pages API (`repos/.../pages`) is refused by the sandbox's GitHub proxy (HTTP 403, "Access to this GitHub API path is not permitted"). |
| Start a new Claude cloud session | Available | `create_session` started probe session `session_01MYgnitwNvqtwBsG6AsXAmZ` (Haiku 5.5), which answered "preflight probe ok"; it was archived right after. One environment exists: `env_011CUwXrgx63fmfaxEf2GKvb`. |
| Coding-agent harness for firstmate | Available | Claude Code 2.1.296 runs headless: `claude -p --model claude-haiku-5-5 --output-format json` returned a result with measured usage and cost. `tmux` is installed. |
| LLM provider for Hermes | **Blocked** | The Vercel AI Gateway (`ai-gateway.vercel.sh`) answers HTTP 401 "Authentication failed" on `/v1/models`, `/v1/chat/completions`, `/v1/messages`, `/v1/ai/language-model`, and `/v1/credits`, with no auth header, with `AI_GATEWAY_API_KEY` as a bearer token or `x-api-key`, and with the proxy placeholder. The proxy is documented to inject the gateway credential, but authentication fails either way. |
| Jev through the gateway | **Blocked** | Same 401 for `typesafe-ai/jev`. |
| Iris credentials | Available | `OPENAI_API_KEY` and `GEMINI_API_KEY` are set; free model-list calls returned HTTP 200 from both providers, listing the GPT Image 2.5 and Nano Banana models Iris supports. No paid call was made. |

Other facts found on the way:

- GitHub through the sandbox proxy: repository-scoped REST paths work (`gh api repos/doodla/athena/...`).
  GraphQL is refused, so `gh pr ...` subcommands fail; pull requests must use REST
  (`gh api repos/doodla/athena/pulls`). Writes to branch protection and rulesets are refused
  (branch protection: "Resource not accessible by integration"; rulesets: proxy 403). The Actions
  permissions path is refused.
- Downloads: GitHub release assets download (HTTP 200); `api.github.com` unauthenticated release
  lookups are refused (403); `raw.githubusercontent.com` and `index.crates.io` work.
- Toolchain present in this image (not to be relied on; setup must pin its own): Rust 1.97.0,
  Node 22, Python 3.13, uv 0.11, Docker 29, Playwright's Chromium in `/opt/pw-browsers`.
- `claude -p --setting-sources user` does not load the project `CLAUDE.md` (canary test: the
  default run repeated a canary word planted in a project `CLAUDE.md`; the run with
  `--setting-sources user` did not). `claude --bare` fails authentication here, because it reads
  only `ANTHROPIC_API_KEY` or an `apiKeyHelper`, and this sandbox authenticates otherwise.

## Blockers and workaround plans

### B1. GitHub Pages is not enabled

- Needed: Sri enables Pages with "GitHub Actions" as the source (repository Settings → Pages).
- Workaround: Athena's deploy workflow is built regardless. It tries `actions/configure-pages`
  with `enablement: true`, which may fail with the default token. Until Pages is on, the site is
  verified locally and in CI against the built artifact, and live-site checks are listed as
  pending in the report.
- Re-check: `gh api repos/doodla/athena --jq .has_pages`.

### B2. The Vercel AI Gateway rejects authentication (blocks Jev and Hermes's provider)

- Needed: a working gateway credential for this environment (Sri, environment settings).
- Workaround for Jev: Athena builds its Jev client against the gateway's documented
  OpenAI-compatible API and tests it offline. Until the gateway answers, typed decisions run on a
  fallback tier (Haiku 5.5 with structured output), recorded as the fallback, never as Jev.
- Workaround for Hermes: study it statically, and run it partially where a scripted or local model
  can stand in; findings without real runs are marked read-only. No other paid credential in the
  sandbox is repurposed for it: the OpenAI and Gemini keys belong to Iris and its $15 cap.
- Re-check: `curl -sS -o /dev/null -w '%{http_code}' https://ai-gateway.vercel.sh/v1/models`.

### B3. Server-side branch protection cannot be configured

- Needed: Sri may add a ruleset on `main` that requires pull requests and the CI checks.
- Workaround: work reaches `main` only through pull requests merged after CI passes, by tooling
  that checks the CI result before merging, and CI re-verifies `main` after every merge.
