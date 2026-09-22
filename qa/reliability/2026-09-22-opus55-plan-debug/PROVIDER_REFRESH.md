# Z.AI and OpenRouter catalogue refresh — 2026-09-22

Bounded existing-provider catalogue fix. Product authority: **Shipping MVP — LIVE**, **Multi-provider inference** — “OpenAI, Anthropic/Claude Plan … Z.AI, DeepSeek, OpenRouter … and custom providers.” Same local worktree, branch and main base as CHANGE.md. No release or user-default/credential changes.

User requested Z.AI GLM 5.3 Flash and FlashX, OpenRouter Grok 4.7, DeepSeek V4.1 Flash, GLM 5.3 Flash, and exact NVIDIA Nemotron 3 Ultra free ID. Retire OpenRouter legacy V4 Pro (including 0813), Flash0731, Grok4.5/4.6, OwlAlpha from picker and automatic allocation; retain explicit-ID metadata for existing sessions. Standard crew's new Grok slot moves from4.6 to4.7; running/saved crews are not rewritten.

Sources: https://docs.z.ai/guides/vlm/glm-5.3-flash and https://openrouter.ai/api/v1/models (retrieved2026-09-22). Public exact metadata saved in `/home/pfrpc/corbanu-debug-evidence/provider-refresh-openrouter-metadata.json`.

Z.AI docs explicitly distinguish Flash (Coding Plan supported) and FlashX (API only, not in Coding Plan). Asked user whether to add a separate metered API route; no silent endpoint/billing switch is authorized. Regular Flash uses exact glm-5.3-flash, 1M context,128Koutput, vision and required preserved thinking low/high/max. Its plan economics remain manual-only because current point/promotion rules differ from the existing GLM5.3 schedule. FlashX API integration is pending the billing-route decision, not claimed supported by Plan.

OpenRouter routes use verified exact IDs, current provider-compatible context/output limits, modalities and advertised reasoning efforts. DeepSeek4.1 automatic economics remain manual-only because its current price includes time-of-day/weekday overrides not represented by a flat rate. Nemotron remains exact :free (never substitute paid variant); data-use/rate-limit terms noted in its selector description.

Live upstream preflight: existing encrypted-vault credentials, no copied credentials, generic `Reply with only READY` prompt. All five confirmed routes returned HTTP200, exact requested model ID, and READY (including exact NVIDIA :free at zero reported cost). OpenRouter total reported cost across four probes: $0.00082413. These are upstream checks, not substitutes for final Corbanu TUI tests.

Code inspection found an existing config/resume defect: the legacy Z.AI effort normalizer replaced low/high/max with xhigh, unsupported by exact GLM5.3 routes. Config now respects a declared exact catalogue reasoning dialect, retaining old normalization for legacy routes. Added catalogue-driven config-load coverage and request-serialization checks for every supported effort of the new models.

## Frozen independent test design

Agent provider_refresh_test_design, code-blind before implementation:
- P0: actual normal-profile Z.AI/OpenRouter picker entries and exact requests for each new ID; report honest service failures, no fallback.
- P0: retired IDs absent from picker and automatic choices; explicit compatibility may remain; unrelated providers/credentials unchanged.
- P1: selection/restart/cancel persistence using existing nonempty user profile; no empty-profile handoff.
- P1: provider-specific failure/retry behavior truthfully surfaced; preserve selected exact ID.
- P2: no duplicate rows, deterministic ordering, unrelated-provider regression coverage.

FlashX case is explicitly pending user confirmation of separate paid API use, because vendor docs exclude it from the existing Plan route. Existing-profile UI checks and no-secret config hashes will accompany final installed-package evidence.
