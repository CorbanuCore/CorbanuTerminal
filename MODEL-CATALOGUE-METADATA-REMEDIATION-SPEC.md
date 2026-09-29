# Model Catalogue Metadata Remediation

**Status:** Proposed specification for [#127](https://github.com/CorbanuCore/CorbanuTerminal/issues/127), [#128](https://github.com/CorbanuCore/CorbanuTerminal/issues/128), and [#129](https://github.com/CorbanuCore/CorbanuTerminal/issues/129). This PR adds the specification; implementation follows separately.

## Baseline and validation setup

- **MAIN:** `c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b`, verified upstream main head on 2026-09-29.
- **RELEASE:** `64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39`, verified head of open [PR #123](https://github.com/CorbanuCore/CorbanuTerminal/pull/123) on 2026-09-29.
- **Provider evidence:** [immutable 2026-09-25 capture][capture]. Its prices and limits are historical observations; refresh and preserve the source, UTC time, route, and units before implementation.
- **Change class:** Routine documentation. Product basis: [“Shipping MVP — LIVE” at MAIN][product], “Multi-provider inference.” Classify implementation separately under the repository's change process.

Every source link pins the relevant repository file and fields to MAIN or RELEASE. The Grok, DeepSeek V4.1, and GLM 5.3 Flash OpenRouter rows exist only at RELEASE; the six missing limits also exist at MAIN. Reconcile this plan with the landed catalogue if PR #123 changes.

Run the following setup and each issue's commands in one Bash session, from a clean implementation checkout. `FIX` records the tested commit. The metadata checks cover the recommended resolutions; an alternative decision requires corresponding assertions.

Implementation PRs must add regressions with the `catalogue_issue_127`, `catalogue_issue_128`, and `catalogue_issue_129` prefixes in the indicated suites. These tests are required future work. The runner rejects a filter matching zero tests.

```bash
set -euo pipefail
ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
test -z "$(git status --porcelain)"
FIX=$(git rev-parse HEAD)
printf 'Checking implementation commit %s\n' "$FIX"
run_tests() (
  cd "$ROOT/codex-rs"
  listing=$(cargo test -p "$1" --lib "$2" -- --list) || return
  if ! rg -q ': test$' <<< "$listing"; then
    printf 'No matching tests: %s\n' "$2" >&2
    return 1
  fi
  cargo test -p "$1" --lib "$2"
)
```

The test modules are [manager tests at RELEASE][manager-tests], [client tests at RELEASE][client-tests], and [allocator tests at RELEASE][allocator-tests]. Cargo uses module paths, not their source filenames. Run the pinned [workspace manifest][manifest] with its [Rust toolchain][toolchain], plus Bash, jq 1.6+, curl, and ripgrep.

## #127 — Grok 4.7 long-prompt pricing

**Decision.** Allow automatic cost-based allocation with incomplete pricing, or restrict the route to explicit selection?

At the [RELEASE catalogue][catalogue-release], `x-ai/grok-4.7` has `orchestration.status: "eligible"` and flat `orchestration.billing` rates of 1600/4800/400 milli-USD per million input/output/cache-read tokens. [RELEASE billing metadata][billing] and [allocator formatting][allocator] cannot represent the captured provider tier keyed by `min_prompt_tokens: 200000`. The [capture][capture] gives $1.60/$4.80/$0.40 below the tier and $3.20/$9.60/$0.80 for the tier, in USD per million tokens. At 250,000 uncached input + 10,000 output tokens, those schedules imply $0.448 versus $0.896; this is arithmetic, not an invoice.

**Recommended resolution.** Change the [RELEASE row's orchestration metadata][catalogue-release] to `status: "disabled"`, retain `provider_id: "openrouter"`, give a tier-pricing `reason`, and remove flat `billing` from that metadata. Keep explicit selection available. Refresh or remove numerical prices in its description. This uses the existing explicit-choice policy while avoiding an unqualified allocation estimate.

**Acceptance criteria.**

1. Automatic cost-based allocation excludes this route. Explicit selection and parent inheritance retain the same model/provider; the advertised allocator entry says explicit-choice only and exposes no flat economics.
2. Discovery/cache overlays cannot restore automatic eligibility. The selector and saved explicit-ID path still resolve with the existing credentials.
3. Any displayed prices identify their tier and source date. Re-enabling automatic allocation requires verified threshold semantics and tests at 199,999, 200,000, and 200,001 serialized prompt tokens, covering cached input, uncached input, and output. Count system and tool context.

**Validation.** Add catalogue/overlay and allocator-description regressions for criteria 1–2, then run:

```bash
git show "$FIX:codex-rs/models-manager/models.json" |
  jq -e '[.models[] | select(.slug == "x-ai/grok-4.7")] |
    length == 1 and .[0].visibility == "list" and
    (.[0].orchestration | .status == "disabled" and
      .provider_id == "openrouter" and
      (.reason | type == "string" and length > 0) and
      (has("billing") | not))'
run_tests codex-models-manager 'manager::tests::catalogue_issue_127'
run_tests codex-core 'tools::handlers::multi_agents_spec::tests::catalogue_issue_127'
```

## #128 — Six retained IDs without output limits

**Decision.** Populate verified output limits, document deliberate backend defaults, or retire an unsupported ID?

The [MAIN][catalogue-main] and [RELEASE][catalogue-release] catalogues omit `max_output_tokens` for these six IDs:

| Provider   | IDs                                                                                                                     |
| ---------- | ----------------------------------------------------------------------------------------------------------------------- |
| OpenRouter | `x-ai/grok-4.6`, `openrouter/owl-alpha`, `x-ai/grok-4.5`, `deepseek/deepseek-v4-pro`, `deepseek/deepseek-v4-flash-0731` |
| OpenAI     | `gpt-5.5`                                                                                                               |

At [RELEASE][catalogue-release], all six have `visibility: "hide"` and `orchestration.status: "disabled"`, while the [release notes][release-notes] retain explicit-ID compatibility. The [discovery overlay at RELEASE][overlay] fills a missing remote limit only when the bundle supplies one. The RELEASE [OpenRouter Chat Completions][chat-request] and [OpenAI Responses][responses-request] request types do not serialize an output-token ceiling. Adding catalogue values alone would not cap these requests.

**Recommended resolution.** For served IDs, prefer documenting and testing the existing deliberate omission of a request limit, including behavior without discovery. Add a fixed catalogue limit only with a verified source and a clear account of how that metadata is used. For an ID no longer served, record retirement and migration behavior and narrow the compatibility claim. This preserves supported explicit-ID behavior without inventing limits or changing request semantics merely to fill a field.

**Acceptance criteria.**

1. Every listed ID has a disposition with provider, observation date, source, effective output-limit policy, and behavior without discovery. Absence from a feed alone is insufficient evidence of retirement.
2. For each served ID, test saved-ID resolution with discovery present, absent, and stale, then inspect the serialized OpenRouter/OpenAI request. It deliberately omits the limit or sends the supported positive value specified by its disposition; no zero, unrelated model's cap, or silent model substitution is allowed.
3. Preserve hidden/disabled selection for retained records. If an ID is retired, verify the reported failure and migration guidance. The [RELEASE compatibility statement][release-notes] must match the tested behavior.

**Validation.** Inventory all six IDs, including removed or duplicate records. Add disposition/overlay tests and actual request-serialization regressions; the inventory is not a completion check.

```bash
git show "$FIX:codex-rs/models-manager/models.json" |
  jq --argjson ids '["x-ai/grok-4.6","openrouter/owl-alpha","x-ai/grok-4.5",
    "deepseek/deepseek-v4-pro","deepseek/deepseek-v4-flash-0731","gpt-5.5"]' '
    .models as $models | $ids[] as $id |
    [$models[] | select(.slug == $id)] |
    {slug: $id, rows: length, max_output_tokens: .[0].max_output_tokens,
     visibility: .[0].visibility, allocation: .[0].orchestration.status}'
run_tests codex-models-manager 'manager::tests::catalogue_issue_128'
run_tests codex-core 'client::tests::catalogue_issue_128'
```

## #129 — DeepSeek output and GLM context limits

**Decision.** Retain and explain the stored ceilings, or raise them after route validation?

The [RELEASE catalogue][catalogue-release] and [2026-09-25 provider capture][capture] differ as follows (tokens):

| Route / catalogue field at RELEASE                           |         Stored | Captured provider comparison                                                     |
| ------------------------------------------------------------ | -------------: | -------------------------------------------------------------------------------- |
| `deepseek/deepseek-v4.1-flash.max_output_tokens`             |        384,000 | `top_provider.max_completion_tokens`: 393,216                                    |
| `z-ai/glm-5.3-flash.context_window` and `max_context_window` | 1,048,576 each | Model-wide `context_length`: 1,310,720; `top_provider.context_length`: 1,048,576 |

The capture includes GLM endpoints both above and below its stored context ceiling. Provider metadata does not prove which endpoint serves a request. As the [RELEASE OpenRouter request type][chat-request] shows, the DeepSeek output value is catalogue metadata, not an enforced OpenRouter request cap.

**Recommended resolution.** Retain 384,000 and 1,048,576 as documented catalogue ceilings pending route qualification. Explain the lower values as a conservative application policy, without claiming universal endpoint support or enforced output limits. Raise them only after proving the intended request and fallback behavior. Keep the OpenRouter GLM policy separate from native Z.AI.

**Acceptance criteria.**

1. Each [RELEASE catalogue field][catalogue-release] has a dated source, token units, explicit scope (catalogue policy, model-wide maximum, or endpoint capability), and rationale. Document the effective wire limit or provider-default behavior; user-visible capacity claims must agree.
2. Test total serialized prompt + reserved output at one below, exactly at, and one above the applicable context boundary, plus the same output-limit boundaries. Include discovery overlays and a fallback endpoint with smaller limits. Incompatible requests must be rejected or explicitly reduced; the stored number alone is not proof.
3. A raised ceiling requires recorded request acceptance on the intended route and equivalent fallback checks. A retained ceiling still needs the documented scope and boundary evidence in criteria 1–2.

**Validation.** The metadata guard checks the recommended retained values. Add regressions proving their scope and request/fallback behavior, then run:

```bash
git show "$FIX:codex-rs/models-manager/models.json" |
  jq -e 'def one($slug):
    [.models[] | select(.slug == $slug)] |
    if length == 1 then .[0] else error("missing or duplicate row") end;
    (one("deepseek/deepseek-v4.1-flash").max_output_tokens == 384000) and
    (one("z-ai/glm-5.3-flash") |
      .context_window == 1048576 and .max_context_window == 1048576)'
run_tests codex-models-manager 'manager::tests::catalogue_issue_129'
run_tests codex-core 'client::tests::catalogue_issue_129'
```

## Refresh provider evidence

These are read-only metadata queries for #127 and #129. Preserve the raw responses and UTC capture time in implementation evidence. They do not establish request success or a particular routed endpoint.

```bash
EVIDENCE=$(mktemp -d)
date -u '+Feed checked %Y-%m-%d %H:%M:%S UTC' | tee "$EVIDENCE/captured-at.txt"
curl -fsSL 'https://openrouter.ai/api/v1/models' |
  tee "$EVIDENCE/models.json" |
  jq '.data[] | select(.id == "x-ai/grok-4.7" or
    .id == "deepseek/deepseek-v4.1-flash" or .id == "z-ai/glm-5.3-flash") |
    {id, pricing, context_length, top_provider}'
for SLUG in deepseek/deepseek-v4.1-flash z-ai/glm-5.3-flash; do
  curl -fsSL "https://openrouter.ai/api/v1/models/$SLUG/endpoints" |
    tee "$EVIDENCE/$(basename "$SLUG").endpoints.json" |
    jq '.data.endpoints[] | {provider_name, context_length, max_completion_tokens}'
done
printf 'Provider captures: %s\n' "$EVIDENCE"
```

[catalogue-main]: https://github.com/CorbanuCore/CorbanuTerminal/blob/c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b/codex-rs/models-manager/models.json
[catalogue-release]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/models.json
[product]: https://github.com/CorbanuCore/CorbanuTerminal/blob/c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b/docs/corbanu-product-spec.md#shipping-mvp--live
[billing]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/protocol/src/openai_models.rs#L358
[allocator]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/core/src/tools/handlers/multi_agents_spec.rs#L1172
[overlay]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/src/manager.rs#L452
[responses-request]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/codex-api/src/common.rs#L336
[chat-request]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/codex-api/src/common.rs#L595
[release-notes]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/qa/release/0.1.44/RELEASE_NOTES.md
[manager-tests]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/models-manager/src/manager.rs#L674
[client-tests]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/core/src/client.rs#L5617
[allocator-tests]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/core/src/tools/handlers/multi_agents_spec.rs#L1343
[manifest]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/Cargo.toml
[toolchain]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/codex-rs/rust-toolchain.toml
[capture]: https://gist.githubusercontent.com/GeorgL0ngGamma/9761521d217f71f4f5f00ccd27b2e2ff/raw/dbe4f24c6f4b53fdbfe04a2251b5388c08bbfd6b/corbanu-pr123-evidence.json
