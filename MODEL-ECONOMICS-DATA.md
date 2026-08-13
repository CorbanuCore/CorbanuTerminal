# Model Economics Reference (researched 2026-07-26)

Research source for the spawn-time model catalogue. The canonical,
machine-readable runtime source is `codex-rs/models-manager/models.json` under
each model's typed `orchestration` field. Prices are USD per 1M tokens, list
rate, from vendor pricing pages and aggregator listings as of 2026-07-26.

There is deliberately no `unknown` billing variant. A model is either
`eligible` with exact `plan`, `plan_schedule`, `metered`, `auth_dependent`, or `local` billing,
or `disabled` with an explicit reason and no billing object. `auth_dependent`
contains both exact subscription burn and API-key prices; the live spawn tool
resolves the active side from authentication. Unverified rows in this document
remain disabled until their billing and capability metadata is verified.

Billing class:

- `plan` = drawn from a subscription pool. **NOT free.** Plan capacity is a
  finite shared weekly allowance. Models draw against it at different rates
  (Claude Fable 5 weighs ~2x an Opus session and is additionally capped at ~50%
  of the pool). When the pool is exhausted, usage can overflow to metered
  credits billed at API rates on the _same provider id_, which `provider_allowlist`
  cannot detect. Prefer plan routes, but still prefer the cheapest plan runtime
  that can do the job.
- `metered`= pay-per-token API key
- `local` = user-owned rented GPU, already paid for by the hour
- `plan_schedule` = subscription quota with documented time-dependent burn;
  temporary discounts carry an explicit expiry instead of replacing the normal
  schedule

| slug                                      | provider         | billing                                                                                  |  in $/M | out $/M | cached in $/M | vision |   ctx |
| ----------------------------------------- | ---------------- | ---------------------------------------------------------------------------------------- | ------: | ------: | ------------: | :----: | ----: |
| claude-opus-5-plan                        | claude-plan      | plan (burn 1.0x)                                                                         |       — |       — |             — |  yes   |  1.0M |
| claude-fable-5-plan                       | claude-plan      | plan (burn 2.0x, 50% cap)                                                                |       — |       — |             — |  yes   |  1.0M |
| gpt-5.6-sol                               | openai           | auth-dependent: plan 1.0x / API                                                          |    5.00 |   30.00 |          0.50 |  yes   |  372K |
| gpt-5.6-terra                             | openai           | auth-dependent: plan 0.5x / API                                                          |    2.50 |   15.00 |          0.25 |  yes   |  372K |
| gpt-5.6-luna                              | openai           | auth-dependent: plan 0.2x / API                                                          |    1.00 |    6.00 |          0.10 |  yes   |  372K |
| claude-opus-5                             | anthropic        | metered                                                                                  |    5.00 |   25.00 |          0.50 |  yes   |  1.0M |
| claude-fable-5                            | anthropic        | metered                                                                                  |   10.00 |   50.00 |          1.00 |  yes   |  1.0M |
| gpt-5.5                                   | openai (API key) | metered                                                                                  |    5.00 |   30.00 |          0.50 |  yes   |  272K |
| k3                                        | kimi-code        | metered                                                                                  |    3.00 |   15.00 |          0.30 |  yes   |  262K |
| moonshotai/kimi-k3                        | openrouter       | metered                                                                                  |    3.00 |   15.00 |          0.30 |  yes   | 1.05M |
| moonshotai/kimi-k2.7-code                 | ambient          | metered                                                                                  |    0.73 |    3.50 |          0.15 |  yes   |  262K |
| x-ai/grok-4.6                             | openrouter       | metered                                                                                  |    2.00 |    6.00 |          0.50 |  yes   |  500K |
| x-ai/grok-4.5                             | openrouter       | metered                                                                                  |    2.00 |    6.00 |          0.50 |  yes   |  500K |
| glm-5.2                                   | zai              | plan schedule: 3x peak / 2x normal off-peak / 1x promotional off-peak through 2026-09-30 |       — |       — |             — |   no   |  1.0M |
| z-ai/glm-5.2                              | ambient          | metered                                                                                  |    0.76 |    2.42 |          0.14 |   no   |  101K |
| zai/glm-5.2                               | vercel           | metered                                                                                  |    1.40 |    4.40 |          0.26 |   no   | 1.04M |
| zai/glm-5.2-fast                          | vercel           | metered                                                                                  |    2.10 |    6.60 |          0.21 |   no   |  1.0M |
| zai-org/GLM-5.2                           | baseten          | metered                                                                                  |    1.40 |    4.40 |          0.14 |   no   | 1.05M |
| deepseek-v4-pro                           | deepseek         | metered                                                                                  |   0.435 |    0.87 |      0.003625 |   no   | 1.05M |
| deepseek/deepseek-v4-pro-0813             | openrouter       | metered                                                                                  |   0.435 |    0.87 |      0.003625 |   no   | 1.05M |
| deepseek/deepseek-v4-pro                  | openrouter       | metered                                                                                  |   0.435 |    0.87 |             — |   no   | 1.05M |
| minimax/minimax-m3                        | openrouter       | metered                                                                                  |    0.60 |    2.40 |             — |  yes   | 1.05M |
| tencent/hy3:free                          | openrouter       | metered                                                                                  |    0.00 |    0.00 |             — |   no   |  262K |
| google/gemini-3.5-flash                   | openrouter       | metered                                                                                  |    1.50 |    9.00 |          0.15 |  yes   | 1.05M |
| openrouter/owl-alpha                      | openrouter       | metered                                                                                  |    0.00 |    0.00 |          0.00 |   no   | 1.05M |
| muse-spark-1.1                            | meta             | metered                                                                                  | UNKNOWN | UNKNOWN |             — |  yes   | 1.05M |
| deepseek-ai/DeepSeek-V4-Flash             | gpu-\*           | local                                                                                    |       0 |       0 |             — |   no   |     — |
| huihui-ai/Huihui-GLM-5.2-abliterated-GGUF | gpu-\*           | local                                                                                    |       0 |       0 |             — |   no   |     — |

## Cost tiers (for allocation guidance)

- `plan` — drawn from subscription capacity. Preferred, but finite; burn
  weight still matters. Local GPU is the only genuinely free tier.
- `low` — under $1/M input: deepseek-v4-pro, minimax-m3, hy3
- `medium` — $1-3/M input: glm-5.2, grok-4.6, grok-4.5, k3, gpt-5.6-terra/luna API
- `high` — $5+/M input: claude-opus-5, gpt-5.6-sol API, gpt-5.5
- `premium` — $10+/M input: claude-fable-5

## Notes that matter for allocation

- Output dominates agent cost. Claude Fable 5 output is $50/M; Grok 4.6 is $6/M.
- Grok 4.6 input, cached-input, and output rates double above 200K prompt tokens.
- Kimi K3 always reasons at max effort; every reasoning token bills at $15/M.
- Cached input is 10x cheaper on most providers. Long-lived agents with stable
  system prompts are much cheaper than the list rate implies.
- Vision is NOT universal: the GLM-5.2 family and DeepSeek V4 Pro are text-only.
  Kimi K3, Grok 4.6, Grok 4.5, MiniMax M3, all GPT-5.x, and all Claude models accept images.
- `-plan` model slugs route through `claude-plan` (subscription). The identical
  model without the suffix routes through `anthropic` (metered). This is the
  single most expensive naming footgun in the catalog.

## Correction history

An earlier revision listed plan routes with `—` for cost, which read as free.
That was wrong: plan capacity is finite, Fable draws roughly double an Opus
session against it, and exhaustion overflows into metered credits automatically.
Burn weights above replace that.

The initial 2026-07-26 orchestration patch also disabled Ambient, Vercel, and
Baseten GLM routes without first checking their live official catalogues. That
was wrong. The rows above now use:

- Ambient's public `/v1/models` catalogue:
  `https://api.ambient.xyz/v1/models`
- Vercel AI Gateway's public `/v1/models` catalogue and provider page:
  `https://ai-gateway.vercel.sh/v1/models` and
  `https://vercel.com/ai-gateway/models/glm-5.2/providers`
- Baseten's direct Model API pricing:
  `https://www.baseten.co/pricing/`
- Z.AI's Coding Plan endpoint and GLM-5.2 burn schedule:
  `https://docs.z.ai/devpack/quick-start` and `https://z.ai/blog/glm-5.2`
- OpenRouter's live catalogue and official model pages:
  `https://openrouter.ai/api/v1/models`,
  `https://openrouter.ai/openrouter/owl-alpha/pricing`, and
  `https://openrouter.ai/google/gemini-3.5-flash/api`

## Unverified

`muse-spark-1.1` remains unverified for the direct `meta` route. OpenRouter's
catalogue lists `meta/muse-spark-1.1`, but that is a different provider route
and its gateway price cannot be copied onto Meta's direct API.
