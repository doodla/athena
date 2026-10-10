# Preflight and sandbox blockers

Sandbox-side facts for the operator. Athena never reads this file. Each check names how it was
made, so a later session can re-run it. Re-check every open blocker at each restart.

## Preflight of 2026-10-10 (session `cse_01BK4zqW5z6Xh8mk2bEuC9W1`)

| Capability | Result | Evidence |
|---|---|---|
| Push access to `doodla/athena` | Available | `gh api repos/doodla/athena` reports `push: true, admin: true` for the session; first push of this branch succeeded (see the commit that added this file). |
| GitHub Pages, Actions as source | **Blocked** at first check; resolved at about 10:09Z (see B1) | `gh api repos/doodla/athena` reports `has_pages: false`. The Pages API (`repos/.../pages`) is refused by the sandbox's GitHub proxy (HTTP 403, "Access to this GitHub API path is not permitted"). |
| Start a new Claude cloud session | Available | `create_session` started probe session `session_01MYgnitwNvqtwBsG6AsXAmZ` (Haiku 5.5), which answered "preflight probe ok"; it was archived right after. One environment exists: `env_011CUwXrgx63fmfaxEf2GKvb`. |
| Coding-agent harness for firstmate | Available | Claude Code 2.1.296 runs headless: `claude -p --model claude-haiku-5-5 --output-format json` returned a result with measured usage and cost. `tmux` is installed. |
| LLM provider for Hermes | **Blocked** at first check; resolved at 09:46Z (see B2) | The Vercel AI Gateway (`ai-gateway.vercel.sh`) answers HTTP 401 "Authentication failed" on `/v1/models`, `/v1/chat/completions`, `/v1/messages`, `/v1/ai/language-model`, and `/v1/credits`, with no auth header, with `AI_GATEWAY_API_KEY` as a bearer token or `x-api-key`, and with the proxy placeholder. The proxy is documented to inject the gateway credential, but authentication fails either way. |
| Jev through the gateway | **Blocked** at first check; resolved at 09:46Z (see B2) | Same 401 for `typesafe-ai/jev` at first. |
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

### B1. GitHub Pages is not enabled: resolved

- Resolution: at about 10:09Z the same day, Sri set Pages' source to "GitHub Actions" (screenshot
  of Settings → Pages: "Pages source saved"); `gh api repos/doodla/athena --jq .has_pages` now
  answers `true`. The site will be served at `https://doodla.github.io/athena/`.
- Needed (originally): Sri enables Pages with "GitHub Actions" as the source.
- Workaround: Athena's deploy workflow is built regardless. It tries `actions/configure-pages`
  with `enablement: true`, which may fail with the default token. Until Pages is on, the site is
  verified locally and in CI against the built artifact, and live-site checks are listed as
  pending in the report.
- Re-check: `gh api repos/doodla/athena --jq .has_pages`.

### B2. The Vercel AI Gateway rejects authentication (blocks Jev and Hermes's provider): resolved

- Needed: a working gateway credential for this environment (Sri, environment settings).
- Resolution: Sri reported that the Jev key should work from the next session. A re-probe at
  09:46Z the same day already succeeded in this session:
  - Jev is a decision model, not a language model. Chat completions answer 400
    `ModelTypeMismatchError`. The OpenAI-compatible Decisions API works:
    `POST https://ai-gateway.vercel.sh/v1/decisions` with `model: typesafe-ai/jev`, an `input`,
    and typed `questions` (`predicate`, `choice`, `score`), answering with probabilities. A
    two-question call cost $0.0000134 (319 input tokens). Reference:
    https://vercel.com/docs/ai-gateway/sdks-and-apis/openai-decisions
  - `POST /v1/chat/completions` with `anthropic/claude-haiku-5.5` answered, so the gateway can
    serve as Hermes's OpenAI-compatible provider (base URL `https://ai-gateway.vercel.sh/v1`).
  - Both work with `Authorization: Bearer $AI_GATEWAY_API_KEY` and with no auth header (the
    sandbox proxy injects the credential).
- Original workaround, kept in case it regresses: Jev's calls fall back to a recorded fallback
  tier; Hermes is studied statically with partial runs marked read-only. The OpenAI and Gemini keys
  belong to Iris and its $15 cap and are never repurposed.
- Re-check at every restart: `curl -sS -o /dev/null -w '%{http_code}' -X POST
  https://ai-gateway.vercel.sh/v1/decisions -H 'Content-Type: application/json' -d
  '{"model":"typesafe-ai/jev","input":"x","questions":[{"type":"predicate","instructions":"Is x a letter?"}]}'`.

### B3. Server-side branch protection cannot be configured

- Needed: Sri may add a ruleset on `main` that requires pull requests and the CI checks.
- Workaround: work reaches `main` only through pull requests merged after CI passes, by tooling
  that checks the CI result before merging, and CI re-verifies `main` after every merge.
