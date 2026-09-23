# Independent code-blind test design

Frozen before final candidate testing. Designer: `/root/model_picker_test_design`; received user intent and essential constraints only, no implementation or results. Scope is a local debug handoff, not a public release.

## P0

1. Record existing stable version and non-secret settings. Launch `corbanu-debug --yolo`; distinguish debug identity; stable executable/profile unchanged.
2. With existing Claude Plan authentication, open picker. Opus 5.5 Plan must be explicit and distinct from Anthropic API. GPT-6 Sol must also appear with its intended provider.
3. Select Plan Opus 5.5 and send harmless prompt. UI and sanitized request metadata must identify the exact provider/model and subscription auth, without requesting an API key. Model self-description is not proof.
4. Exit/relaunch and send another prompt. Exact model/provider persist and are used by the next request.
5. With unavailable/rejected model, show an actionable error without older-model, provider, or API-billing fallback.
6. Inspect visible auth/request/restart output for secrets. Do not copy credentials into plaintext locations.

## P1

7. Select GPT-6 Sol, send/restart/send; exact provider/model persist.
8. With both routes configured, switch Plan/API Opus 5.5; each uses only selected auth mechanism.
9. Missing/expired Plan authentication must request subscription recovery, not use API keys.
10. After testing, stable install retains original version/profile/model/auth behavior.

## Ambiguities preserved

Canonical model IDs, intended GPT-6 Sol provider, reuse vs isolation of existing Plan auth, safe request-metadata observation, actual subscription eligibility, and whether `--yolo` affects profile isolation require explicit evidence. These proposals are not test results or release approval.
