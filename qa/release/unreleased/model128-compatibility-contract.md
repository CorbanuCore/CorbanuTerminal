# Retained model IDs: local compatibility contract

Prepared October 23, 2026 for current main, following the [PR #141 maintainer
request][retarget]. This regression-only change covers local compatibility for
[issue #128][issue] and the proposed [PR #140 specification][spec]. The adapted
tests have not yet run on this main-based tree. This record does not close all of
issue #128 or establish that a provider currently serves a retained model.

Change class: **Routine** tests and QA documentation under `AGENTS.md`. Product
basis: [Shipping MVP — LIVE][product], **Multi-provider inference**: “OpenAI,
Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten,
Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” No
production code, catalogue record, routing policy, credential, or output budget
changes; no product-initiative plan or sprint is introduced.

## Revision and policy boundary

The baseline is main `55ffd0277cba12bbe90dfa687a086809d14c4c10`.
The catalogue retains these exact IDs and output metadata:

| Provider | Retained ID | Catalogue `max_output_tokens` |
| --- | --- | --- |
| OpenRouter | `x-ai/grok-4.6` | 128,000 |
| OpenRouter | `openrouter/owl-alpha` | 262,144 |
| OpenRouter | `x-ai/grok-4.5` | 128,000 |
| OpenRouter | `deepseek/deepseek-v4-pro` | 384,000 |
| OpenRouter | `deepseek/deepseek-v4-flash-0731` | 384,000 |
| OpenAI | `gpt-5.5` | absent |

At this main pin, all six records are hidden from the picker and disabled for
automatic allocation. Five have explicit output metadata; `gpt-5.5` has none.
The regressions retain explicit-ID resolution without changing these catalogue
records. These are source facts and intended assertions, not new runtime proof,
upstream service availability or a historical retirement date. The original
release-based tests and QA history remain bound to commit
`0a636facef219106f6c13dd04fad86b95ba5574d`.

## What the tests exercise

The `catalogue_issue_128` model-manager tests extend the existing generic hidden
model/cache regression. They resolve each saved explicit-ID input through the
real model-manager methods and verify exact identity, provider, hidden/non-default
picker state, preserved disabled orchestration, the bundled row's output metadata,
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
For each model it first uses that row's current output metadata, then repeats
construction with metadata absent, zero, and 8,192. It asserts that the entire
serialized request remains equal, and none of `max_tokens`,
`max_completion_tokens`, or `max_output_tokens` is present—not even as JSON null.
The current builders and serialization types still omit these wire keys; adding
catalogue caps did not add fields to these two request formats. The real provider
constructors and wire API variants are used. Actual test execution remains pending.

The source-level contract on these paths remains wire-limit omission. A catalogue
value alone is not an enforced output budget in these request types. This record
makes no claim about a provider's current default or maximum, nor a new observed
runtime result.

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

Nextest fails when a filter matches zero tests. The two manager tests are intended
to cover 24 six-ID/state assertions; the core matrix is intended to build and
serialize 24 requests (including `gpt-5.5`'s repeated absent-metadata case). Record
the tested PR head and actual command outcomes in the PR, including tool or
baseline failures. A passing focused run does not imply a passing workspace.

Until those tests execute, this unit supplies prepared coverage rather than
qualification. Its intended scope is local resolution, cache transitions, and
builder/serializer behavior. It does **not** exercise reading saved profile/session files, a live TUI,
credentials, HTTP transport or endpoint acceptance, current provider serving
status, token-capacity boundaries, endpoint fallback, billing, or a release.
No live inference calls are required. Those limits are material: provider
retirement/migration decisions and the broader #129 output-budget policy need
separate evidence and authorization.

[issue]: https://github.com/CorbanuCore/CorbanuTerminal/issues/128
[spec]: https://github.com/CorbanuCore/CorbanuTerminal/blob/28490d22b53a99485eabbfe8f0ba125b7845e71d/MODEL-CATALOGUE-METADATA-REMEDIATION-SPEC.md
[product]: https://github.com/CorbanuCore/CorbanuTerminal/blob/55ffd0277cba12bbe90dfa687a086809d14c4c10/docs/corbanu-product-spec.md#shipping-mvp--live
[retarget]: https://github.com/CorbanuCore/CorbanuTerminal/pull/141#issuecomment-6312414499
