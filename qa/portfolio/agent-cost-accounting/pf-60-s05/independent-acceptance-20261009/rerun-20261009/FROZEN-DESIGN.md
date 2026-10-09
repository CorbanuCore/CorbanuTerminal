# PF-60-S05 targeted re-run: frozen design (2026-10-09)

Frozen before any real request of this re-run. Read before freezing: the sprint record, the defaults table, the options
memo, `real-provider-checks-20261009.md`, the first run (README, FROZEN-DESIGN, tools, captures), `sec-common.md`,
`docs/features/claude-plan-authentication.md`, `docs/features/claude-headless-panes.md`, `docs/config.md`,
`docs/authentication.md`, `corbanu features list` and `corbanu debug models` of the built binary, and OpenAI's pricing
page. No implementation source, PR diffs, lane briefs, worker logs or review files.

Binary: developer-accounting debug build of origin/main `5d283fde18` on macOS (and on Linux for `claude-plan`).
Every built-binary run: disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`; keys resolved by the INSTALLED wrapper
without the test variables and substituted only on the consuming command.

Credentials: `provider/kimi_api_key` (KIMI_API_KEY), `claude-plan-test-token` (CLAUDE_CODE_OAUTH_TOKEN),
`openai-api-key` (CODEX_API_KEY in exec).

Independent channel: raw provider events traced before parsing (`RUST_LOG=codex_api=trace`), recomputed with Decimal
(`tools/recompute.py`) at the provider's published price read today. OpenAI `gpt-5.6-luna` Standard
$0.20 / $0.02 cached / $0.25 cache writes / $1.20 output per 1M; Fast (formerly Priority) $0.40 / $0.04 / $0.50 / $2.40.

| Case | AC | Action | Expected |
| --- | --- | --- | --- |
| R1 | AC5, Blocker 1 | One home. T1: built-in `openai`, API key, `gpt-5.6-luna`, one short turn; capture `/cost` day. T2: built-in `kimi-code`, model `k3`, real membership key, one short turn; capture `/cost` day and T2's conversation. | T2 is Subscription with $0 counted as spent; the day's spent total equals T1's estimate before and after T2; the overflow note appears on views with T2's work and not on a view with only T1's. |
| R2 | AC10, AC12, AC2 (kimi-code) | R1-T2, plus a custom provider at `https://api.kimi.com/coding/v1` with the same key. | Both subscription ("built-in" source); an env key does not make it pay per use. Kimi Open Platform: NOT VERIFIABLE (membership key is not a platform key). |
| R3 | AC10, AC12 (OpenAI API key) | R1-T1 plus a second turn; custom provider at `https://api.openai.com/v1` (Responses wire) with the same key. Observation: `OPENAI_API_KEY` alone in `exec`. | Pay per use; the estimate equals the recomputation at Standard rates to the micro-dollar; custom route pay per use. |
| R4 | AC10, AC12 (claude-plan) | Real `claude-plan` turn with the setup token (Linux: no keychain on that host). | Subscription, not spent, overflow note; no "Pay per use", "no price available" or "check the bill"; not in the "had no price" count. |
| R5 | AC4 | (a) `gpt-5.6-luna` with `service_tier="fast"` (Fast = Priority) on the API key; (b) image generation via the API key; (c) realtime via the API key if the build exposes it; (d) a Claude pane on the claude-plan token if reachable without the keychain. | (a) recorded with the priority tier, pay per use (API key), estimate = recomputation at Fast rates, or a stated limitation; (b)(c) recorded with the API-key basis; (d) Subscription. ChatGPT-login parts: NOT VERIFIABLE. |
| R6 | AC7 | OpenAI API key; ledger refuses new attempts (SQLite trigger, as in the first run); one web-search turn and one image-generation turn (smallest size). | Search results and the image are returned; one gap warning per turn; nothing recorded for the refused attempts. |
