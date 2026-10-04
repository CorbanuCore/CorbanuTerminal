# Sol access: OpenAI compatibility negotiation repair

Bounded fix; product authority **Shipping MVP — LIVE / Multi-provider inference**: “OpenAI, Anthropic/Claude Plan … Z.AI, DeepSeek, OpenRouter … and custom providers.” Same existing worktree and base as CHANGE.md. No account, credential, billing, model-ID, or persistent-default changes. No public release.

## Diagnosis

Official installed Codex 0.156.0, existing ~/.codex ChatGPT login, exact gpt-6-sol: SOL_READY succeeded. Installed Corbanu debug, existing ~/.corbanu profile with its normal legacy ~/.codex auth fallback: HTTP400 unsupported model. Both ran with user configuration ignored, no model fallback, and generic no-tool prompts.

Controlled direct requests used the same existing access token, account ID, exact gpt-6-sol request body, endpoint, originator, and user-agent. Only compatibility version changed:

| Version | Model discovery | Inference |
| --- | --- | --- |
| 0.153.0 | HTTP200, Sol absent | HTTP400, model unsupported for ChatGPT account |
| 0.156.0 | HTTP200, Sol present | HTTP200, response.completed model gpt-6-sol, SOL_READY |

Credentials were consumed in memory, not printed or copied into evidence. This establishes version-dependent behavior, not a Pro-plan exclusion. Discovery client_version and inference version header already share OPENAI_CODEX_COMPAT_VERSION; update that one constant from 0.153.0 to 0.156.0. Preserve product version and branded user-agent.

Intermediate custom-provider probes were not valid substitutes for built-in OpenAI behavior: reserved provider overrides were correctly rejected; a differently named custom provider did not activate OpenAI's encrypted collaboration schema, yielding a reserved-tool schema error. No such custom provider was saved. Final validation must use the normal built-in OpenAI route.

Evidence directory: /home/pfrpc/corbanu-debug-evidence. Initial runs: sol-official-codex-diagnostic.txt, sol-corbanu-diagnostic.txt. Controlled version-only result recorded above from executed diagnostics. No global credential tracing.

## Frozen independent design

Fresh-context agent /root/sol_access_test_design (Newton), intent-only packet, instruction-only isolation; no tool calls. Original proposed cases:

1. P0 Non-destructive launch: existing nonempty profile, all providers/credentials/defaults retained, stable installation unchanged; package and before/after evidence.
2. P0 Exact model/existing ChatGPT login: real TUI OpenAI gpt-6-sol selection, complete streamed answer, no HTTP400/account/key/model substitution; request evidence, not self-identification alone.
3. P0 Real tool round trip: inspect harmless freshly created fixture using tool; final answer matches actual tool output.
4. P0 Cancel and recover: cancel active response, next prompt succeeds in same session, no unintended execution.
5. P0 Restart: normal debug relaunch, credentials/providers retained, Sol request succeeds without login or unintended default change.
6. P1 Regression: previously working OpenAI and non-OpenAI provider, return to Sol, no credential replacement/default changes.
7. P1 Honest controlled failure: actionable error, retry recovery, no silent model/account/provider/auth fallback.

Designer requires each case disposition; mocks/build alone insufficient.

## Execution and independent evidence review

Installed debug wrapper now uses package `0f18b63404-sol-compat156` with matching code-mode host. Product version remains 0.1.42. Package/source hashes: sol-compat-manifest.md and sol-compat-binaries.sha256 in evidence directory. Formatting/build/diff checks passed; 419/419 protocol/model/provider tests passed.

F01/F02/F03/F05/F07: normal-profile launch, exact Sol TUI request, real fresh-fixture shell read, restart, and explicit recovery after a controlled invalid-model HTTP400 all succeeded. Sanitized response/auth metadata: sol-compat-routing-summary.json. Original config/provider eligibility/stable launcher/stable executable hashes remain unchanged. Tool check used requested YOLO mode, not approval-dialog coverage.

F04 FAILED: Escape cancelled a streaming 1..10000 output, but the next simple prompt stayed Working for 1m32s. A second cancel/retry also stalled; fresh sessions succeed. This remains unresolved; do not claim complete acceptance or release readiness.

F06 partial: Astra, Luna and existing Z.AI provider succeeded in separate exact-package CLI runs, followed by successful normal Sol TUI restart. Same-session TUI cross-provider switching not executed in this repair pass.

Independent reviewer /root/sol_access_test_design supports the narrow Sol-access repair claim and explicitly rejects complete acceptance because of F04. Review artifact sol-compat-review.md. No public release, benchmark, or human sign-off claimed. Only our QA sessions were stopped; user sessions were not interrupted. Restart is necessary to pick up the new binary; interrupted-session recovery is a known remaining limitation.
