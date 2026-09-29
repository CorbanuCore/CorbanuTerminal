# Model Catalogue Metadata Remediation

**Status:** Proposed fix plan; no catalogue or runtime fix is claimed.
**Scope:** [#127](https://github.com/CorbanuCore/CorbanuTerminal/issues/127), [#128](https://github.com/CorbanuCore/CorbanuTerminal/issues/128), [#129](https://github.com/CorbanuCore/CorbanuTerminal/issues/129).
**Pinned sources:** `MAIN=c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b` (verified upstream `main` head on 2026-09-29); `RELEASE=64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39` (open PR #123 head on 2026-09-29). All repository file and field observations below refer to these exact commits. Validation commands record `FIX=$(git rev-parse HEAD)` and inspect that exact implementation commit.
**Change class:** Routine documentation of proposed bounded fixes. Product basis: [“Shipping MVP — LIVE” at MAIN](https://github.com/CorbanuCore/CorbanuTerminal/blob/c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b/docs/corbanu-product-spec.md), “Multi-provider inference” includes OpenRouter, OpenAI, and DeepSeek. Implementation must follow the repository's change process.

PR #123 introduces the Grok and GLM/DeepSeek rows; the six missing output limits already exist on MAIN. If PR #123 changes or is not merged, rebase this plan's implementation against the actual catalogue and rerun the checks. Provider feeds are mutable: record the UTC time, raw response, route, and units used to justify any value.

## #127 — Grok 4.7 long-prompt pricing

**Decision.** Should `x-ai/grok-4.7` remain eligible for automatic cost-based selection before tier-aware pricing exists? At [RELEASE catalogue](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/models.json), `orchestration.status` is `eligible` and `orchestration.billing` contains flat 1600/4800/400 milli-USD per million input/output/cache-read tokens. At [RELEASE `ModelBilling::Metered`](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/protocol/src/openai_models.rs) and [RELEASE allocator formatting](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/core/src/tools/handlers/multi_agents_spec.rs), the cost description cannot express the ≥200,000-prompt-token tier. The [OpenRouter model feed](https://openrouter.ai/api/v1/models) exposes that tier; on 2026-09-29 its base input/output rates were already $2/$6 per million, versus $1.60/$4.80 in the RELEASE row. For the issue's 2026-09-25 capture, 250,000 uncached input + 10,000 output tokens implied $0.448 at the flat rates versus $0.896 at the tier rates (illustrative arithmetic, not a bill).

**Recommendation.** Set this row's [RELEASE `orchestration.status`](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/models.json) to `disabled` with a tier-pricing reason; keep explicit selection available. Refresh or remove its numerical description when the feed disagrees. This is a bounded mitigation while tier-aware billing and prompt-length accounting are designed and tested. Do not encode a guessed invoice or rely on a user-visible prompt length that excludes serialized system/tool context.

**Acceptance.**

1. Automatic cost-based selection excludes `x-ai/grok-4.7`; explicit OpenRouter selection still resolves to the same model ID with its existing credentials.
2. No allocator economics describes the base price as valid for long prompts. A displayed price is either provider-current with its threshold and date or clearly directs users to live pricing.
3. If tier-aware auto-selection replaces the mitigation, tests cover serialized prompt lengths 199,999, 200,000, and 200,001, both cached and uncached input, output pricing, and the provider's exact threshold semantics.

**Validation (from the implementation repository root, after the release rows land):**

```bash
set -euo pipefail
RELEASE=64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39
FIX=$(git rev-parse HEAD)
printf 'Checking implementation commit %s\n' "$FIX"
git show "$RELEASE:codex-rs/models-manager/models.json" |
  jq -e '.models[] | select(.slug == "x-ai/grok-4.7") |
    .orchestration.status == "eligible"'
jq -e '[.models[] | select(.slug == "x-ai/grok-4.7")] |
  length == 1 and .[0].visibility == "list" and
  .[0].orchestration.status == "disabled"' <(git show "$FIX:codex-rs/models-manager/models.json")
date -u '+Feed checked %Y-%m-%d %H:%M:%S UTC'
curl -fsSL 'https://openrouter.ai/api/v1/models' |
  jq -e '.data[] | select(.id == "x-ai/grok-4.7") |
    {base: .pricing, tiers: .pricing.overrides}'
cd codex-rs
cargo test -p codex-models-manager --lib manager_tests
cargo test -p codex-core --lib multi_agents_spec_tests
```

The feed query is a dated observation, not a pass/fail price assertion. In the remediation PR, add regressions in the named suites for exclusion, explicit selection, and advertised economics; review the captured tool description and selector result. The `jq` guard covers the recommended manual-only route.

## #128 — Six retained IDs without output limits

**Decision.** Does each retained explicit ID require a fixed `max_output_tokens`, a documented backend default/discovery path, or retirement? The [MAIN catalogue](https://github.com/CorbanuCore/CorbanuTerminal/blob/c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b/codex-rs/models-manager/models.json) and [RELEASE catalogue](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/models.json) both omit `max_output_tokens` for `x-ai/grok-4.6`, `openrouter/owl-alpha`, `x-ai/grok-4.5`, `deepseek/deepseek-v4-pro`, `deepseek/deepseek-v4-flash-0731` (OpenRouter), and `gpt-5.5` (OpenAI). At RELEASE all six have `visibility: "hide"` and `orchestration.status: "disabled"`, yet the [RELEASE release notes](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/qa/release/0.1.44/RELEASE_NOTES.md) promise explicit-ID compatibility. The [RELEASE discovery overlay](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/src/manager.rs) only fills a missing remote output limit from the bundle when one exists; absence alone does not prove an OpenRouter/OpenAI request fails.

**Recommendation.** Decide separately for each of the six IDs: record whether the route is served, its provider/date/source, and the actual request limit. For a served fixed-cap route, store the verified cap; for a deliberately dynamic or omitted limit, document the provider contract and behavior without discovery. If an ID is no longer served, explicitly retire it and correct the compatibility claim. Do not copy a cap from a different model or invent one for a dead route.

**Acceptance.**

1. All six IDs have an explicit, source-backed disposition (verified fixed cap, intentional default/discovery/override, or retirement) and the [RELEASE release-note compatibility claim](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/qa/release/0.1.44/RELEASE_NOTES.md) matches it.
2. Tests exercise a saved explicit ID through the OpenRouter or OpenAI request path, with and without discovery metadata. The sent request includes a supported positive limit or deliberately omits it under the documented provider contract; it never silently uses zero or another model's cap. Retired IDs fail with a clear migration path.
3. Hidden/disabled selection remains intentional for retained legacy routes. A failing request with absent discovery cannot be called compatible.

**Validation (from the implementation repository root):**

```bash
set -euo pipefail
MAIN=c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b
RELEASE=64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39
FIX=$(git rev-parse HEAD)
printf 'Checking implementation commit %s\n' "$FIX"
for SHA in "$MAIN" "$RELEASE"; do
  git show "$SHA:codex-rs/models-manager/models.json" |
    jq -r '.models[] | select(.slug | IN("x-ai/grok-4.6",
      "openrouter/owl-alpha", "x-ai/grok-4.5", "deepseek/deepseek-v4-pro",
      "deepseek/deepseek-v4-flash-0731", "gpt-5.5")) |
      [.slug, has("max_output_tokens"), .visibility, .orchestration.status] | @tsv'
done
jq -r '.models[] | select(.slug | IN("x-ai/grok-4.6",
  "openrouter/owl-alpha", "x-ai/grok-4.5", "deepseek/deepseek-v4-pro",
  "deepseek/deepseek-v4-flash-0731", "gpt-5.5")) |
  [.slug, (.max_output_tokens // "provider default / discovery / retired"),
    .visibility, .orchestration.status] | @tsv' <(git show "$FIX:codex-rs/models-manager/models.json")
cd codex-rs
cargo test -p codex-models-manager --lib manager_tests
cargo test -p codex-core --lib client_tests
```

The printed rows require the per-ID disposition and request-capture tests specified above; a non-null number alone is insufficient. Add those regressions during implementation, including saved IDs with discovery off.

## #129 — DeepSeek output and GLM context scope

**Decision.** Should application limits be raised to match a feed maximum? At [RELEASE catalogue](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/models.json), `deepseek/deepseek-v4.1-flash.max_output_tokens` is 384,000 and `z-ai/glm-5.3-flash.context_window` / `max_context_window` are 1,048,576. The [OpenRouter model feed](https://openrouter.ai/api/v1/models) and [GLM endpoint feed](https://openrouter.ai/api/v1/models/z-ai/glm-5.3-flash/endpoints) distinguish model-wide maxima from routable endpoint limits. On 2026-09-29 14:33 UTC the model feed reported DeepSeek `top_provider.max_completion_tokens` 943,718 and GLM `context_length` 1,310,720; GLM `top_provider.context_length` was 1,048,575. These mutable observations do not identify the endpoint used for a request. The [DeepSeek endpoint feed](https://openrouter.ai/api/v1/models/deepseek/deepseek-v4.1-flash/endpoints) also lists providers with limits below 384,000.

**Recommendation.** Retain the conservative 384,000 output and 1,048,576 context application caps until a selected, pinned endpoint and fallback policy have been validated. Even these caps need route checks: a fallback with a smaller limit must be excluded or the request reduced/rejected explicitly. Raise only when request construction, provider acceptance, and fallback coverage support the new cap; keep this OpenRouter GLM route distinct from the native Z.AI route.

**Acceptance.**

1. For each [RELEASE catalogue limit](https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/models.json), record source, UTC observation, token units, and whether it is an app cap, model-wide maximum, or endpoint capability; explain any retained lower cap. The user-facing description agrees with the chosen policy.
2. Test prompt tokens + reserved output tokens at the selected endpoint's context boundary: one below, at, and one above. Do the same for the output cap. A request cannot fall back to an endpoint with smaller applicable limits without safe rejection or reduction.
3. A higher cap requires a recorded successful request on the intended route and equivalent fallback tests; a feed comparison alone does not qualify it.

**Validation (from the implementation repository root):**

```bash
set -euo pipefail
RELEASE=64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39
FIX=$(git rev-parse HEAD)
printf 'Checking implementation commit %s\n' "$FIX"
git show "$RELEASE:codex-rs/models-manager/models.json" |
  jq '.models[] | select(.slug == "deepseek/deepseek-v4.1-flash" or
    .slug == "z-ai/glm-5.3-flash") |
    {slug, context_window, max_context_window, max_output_tokens}'
jq -e '[.models[] | select(.slug == "deepseek/deepseek-v4.1-flash" or
  .slug == "z-ai/glm-5.3-flash")] |
  length == 2 and all(.[]; .context_window > 0 and
    .max_context_window > 0 and .max_output_tokens > 0)' \
  <(git show "$FIX:codex-rs/models-manager/models.json")
date -u '+Feed checked %Y-%m-%d %H:%M:%S UTC'
curl -fsSL 'https://openrouter.ai/api/v1/models' |
  jq '.data[] | select(.id == "deepseek/deepseek-v4.1-flash" or
    .id == "z-ai/glm-5.3-flash") | {id, context_length, top_provider}'
for SLUG in deepseek/deepseek-v4.1-flash z-ai/glm-5.3-flash; do
  curl -fsSL "https://openrouter.ai/api/v1/models/$SLUG/endpoints" |
    jq '.data.endpoints[] | {provider_name, context_length, max_completion_tokens}'
done
cd codex-rs
cargo test -p codex-models-manager --lib manager_tests
cargo test -p codex-core --lib client_tests
```

Record the feed capture time. The local guard only checks positive caps; the implementation PR must add boundary and fallback regressions and capture the selected route's actual request behavior before claiming the higher capacity works.
