# Retained model IDs: local compatibility contract

Recorded September 30, 2026. This regression-only change protects the existing
release-candidate behavior requested by [issue #128][issue] and the proposed
[PR #140 specification][spec]. It does not close all of issue #128 or establish
that a provider currently serves a retained model.

Change class: **Routine** tests and QA documentation under `AGENTS.md`. Product
basis: [Shipping MVP — LIVE][product], **Multi-provider inference**: “OpenAI,
Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten,
Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” No
production code, catalogue record, routing policy, credential, or output budget
changes; no product-initiative plan or sprint is introduced.

## Revision and policy boundary

The baseline is PR #123 head `64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39`, on
`release/corbanu-0.1.44`. The catalogue retains these exact IDs:

| Provider | Retained IDs |
| --- | --- |
| OpenRouter | `x-ai/grok-4.6`, `openrouter/owl-alpha`, `x-ai/grok-4.5`, `deepseek/deepseek-v4-pro`, `deepseek/deepseek-v4-flash-0731` |
| OpenAI | `gpt-5.5` |

At this release pin, all six records are hidden from the picker, disabled for
automatic allocation, and have no catalogue `max_output_tokens`. They remain
available for explicit-ID metadata resolution. These statements describe the
pinned candidate, not upstream service availability or a historical retirement
date. Main `c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b` has different selection
policy; this change targets the release branch and does not copy that policy onto
main.

## What the tests exercise

The `catalogue_issue_128` model-manager tests extend the existing generic hidden
model/cache regression. They resolve each saved explicit-ID input through the
real model-manager methods and verify exact identity, provider, hidden/non-default
picker state, preserved disabled orchestration, absent inherited output metadata,
and absence of fallback metadata in four states:

1. Discovery is unavailable and no cache exists; no endpoint fetch occurs.
2. Live discovery tries to re-list hidden models and replace their provider and
   orchestration policy with an eligible route. A separate discovered model has
   a positive output limit, which must not leak into a retained ID.
3. A new manager reads the fresh persisted cache; no extra endpoint fetch occurs.
4. The persisted cache is aged past its TTL, then a new manager refetches an empty
   discovery result. Old unrelated discovery records disappear, while all six
   saved IDs still resolve from the bundle.

The core `catalogue_issue_128_saved_id_wire_requests_omit_output_limits` test
passes each exact saved ID through the static model manager, then calls the real
OpenRouter Chat Completions or OpenAI Responses request builder with a user
message. It serializes the returned request and verifies its exact model ID.
For each model it repeats construction with output metadata absent, zero, and
8,192. The entire serialized request remains equal, and none of `max_tokens`,
`max_completion_tokens`, or `max_output_tokens` is present—not even as JSON null.
The real provider constructors and wire API variants are used.

The observed local contract is therefore deliberate wire-limit omission. A
catalogue value alone is not an enforced output budget on these paths. The
provider determines the effective output behavior; this record makes no claim
about its current default or maximum. All six share this disposition, with the
provider-specific request path shown above.

## Reproduction and qualification

Use the pinned repository toolchain and installed `just`/`cargo-nextest`. From
the repository root, after formatting the final tree:

```sh
just fmt
just test --locked -p codex-models-manager -E 'test(catalogue_issue_128)'
just test --locked -p codex-core --lib -E 'test(catalogue_issue_128)'
just test --locked -p codex-models-manager
just test --locked -p codex-core
```

Nextest fails when a filter matches zero tests. The two manager tests cover 24
six-ID/state assertions; the core test builds and serializes 18 requests. Record
the tested PR head and actual command outcomes in the PR, including tool or
baseline failures. A passing focused run does not imply a passing workspace.

This unit qualifies local resolution, cache transitions, and builder/serializer
behavior. It does **not** exercise reading saved profile/session files, a live TUI,
credentials, HTTP transport or endpoint acceptance, current provider serving
status, token-capacity boundaries, endpoint fallback, billing, or a release.
No live inference calls are required. Those limits are material: provider
retirement/migration decisions and the broader #129 output-budget policy need
separate evidence and authorization.

[issue]: https://github.com/CorbanuCore/CorbanuTerminal/issues/128
[spec]: https://github.com/CorbanuCore/CorbanuTerminal/blob/28490d22b53a99485eabbfe8f0ba125b7845e71d/MODEL-CATALOGUE-METADATA-REMEDIATION-SPEC.md
[product]: https://github.com/CorbanuCore/CorbanuTerminal/blob/64ffe30dd94f986bf5a9e7ccc5e9f25c958d4c39/docs/corbanu-product-spec.md#shipping-mvp--live
