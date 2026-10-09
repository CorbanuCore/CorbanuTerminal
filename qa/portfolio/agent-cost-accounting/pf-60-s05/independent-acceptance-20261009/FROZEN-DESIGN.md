# PF-60-S05 independent acceptance: frozen functional design

Frozen 2026-10-09 ~12:25Z, before PR #346's results (real-provider-checks-20261009.md, demo index) were read and
before any S05 case was run. Sources read before freezing: the sprint record, the plan header, billing-basis-defaults.md,
both-behaviour-options.md, ledger-format-check.md, acct-coverage-audit-20261009.md, the S03 acceptance README and tools,
the S03 body review, sec-common.md, docs/ (claude-plan auth, zai integration), `/cost help`. Grep of
`codex-rs/core/config.schema.json` for the `approvals_reviewer` key name only (disclosed deviation).

Binaries: developer-accounting debug build of merged origin/main on macOS (arm64) and Linux (RTX box); a default
(no feature) build on macOS for config/absence checks. Every run: disposable home, CORBANU_TEST_NO_NATIVE_KEYRING=1,
keys only via the installed `corbanu vault auth-helper provider/<label>` on the consuming command.

Independent usage channel: provider SSE chunks traced by `RUST_LOG=codex_api=trace` (exec stderr / TUI log DB) and an
external stdlib pass-through proxy for custom routes. Totals recomputed with Decimal at prices read from the provider's
own pricing page on the day of the run. Cost views captured from the TUI in tmux.

Credentials present under provider/*: zai_api_key, deepseek_api_key, openrouter_api_key, ambient_api_key,
ai_gateway_api_key. Absent: Kimi Code, Kimi Open Platform, OpenAI API key, ChatGPT login, Anthropic API key, BigModel,
Meta, Baseten, Corbanu API, Bedrock. `provider/claude-code-oauth-token` exists but is provider-integration-only and
cannot be handed to a built binary in a disposable home.

| Case | AC | Action | Expected |
| --- | --- | --- | --- |
| C1 | AC1 | Every one of the 21 built-in providers: one `exec` turn in a fresh home with a placeholder credential (or a local mock for ollama/lmstudio); open the attempt in `/cost`. Run the provider-table unit tests and record names/results. | Basis shown matches the defaults table for every provider; no provider undeclared. |
| C2 | AC2 | `amazon-bedrock` with a command `auth` (placeholder command); `kimi-code` with `KIMI_API_KEY` env; `openai` with `OPENAI_API_KEY`. ChatGPT login if any credential exists. | Bedrock pay per use; kimi-code subscription; openai API key pay per use. Mutation tests: not executable code-blind. |
| C3 | AC3 | Real zai turn T1; then `model_providers.zai.billing = "subscription"`, turn T2; then remove. Invalid value (`"free"`) on both builds. Custom provider via local proxy with no `billing`. | T2 subscription and not spent; T1 keeps its pay-per-use estimate (not re-priced); invalid value is a config error naming the key; custom provider "billing basis not declared" with a next step naming `model_providers.<id>.billing`, tokens recorded, not counted as spent or subscription. |
| C4 | AC4 | ChatGPT/priority/image/realtime/pane-bridge need ChatGPT login or Claude panes. Substitute observable: real subscription work with no catalogue price (Z.AI coding route, zai-anthropic). | NV for the named paths unless a credential appears; substitute shows Subscription, never "Pay per use"/"no price available"/"check the bill", not in the "had no price" count. |
| C5 | AC5 | Real kimi-code turn. | NV without a membership key; placeholder attempt recorded as partial evidence. |
| C6 | AC6 | (a) run the committed-fixture test; (b) build the binary at `5bae03414e^` with developer accounting, make real zai turns, open that ledger with the new build: `/cost` totals equal the recomputation and new turns still record; (c) search for surviving real old ledgers; (d) future format: append an unknown migration row to a ledger copy, run turns. | (a)(b) validates and reads; (c) recorded or absence recorded; (d) turns succeed, exactly one warning, nothing new recorded. |
| C7 | AC7 | Inject store failures (busy lock held past the budget, a corrupted stored row so validation fails, read-only DB file) while a real zai turn forces auto-compaction (low `model_auto_compact_token_limit`); web search and image generation where a credential allows. | Compaction completes and the turn finishes; web search returns results; image returned; one gap warning per turn. NV where no credential. |
| C8 | AC8 | `approvals_reviewer = "auto_review"`, approval on-request, real zai turn that needs an escalated command. | Guardian requests appear under the parent conversation, labelled as review; conversation totals reconcile with the provider-reported usage. |
| C9 | AC9 | `exec --ephemeral` real zai run with several requests; non-ephemeral control. | Exactly one `accounting.excluded` warning for the ephemeral run, zero for the control; the 2026-10-09 audit lists `exec --ephemeral`. |
| C10 | AC10 | Real turns: zai GLM 5.2, deepseek, openrouter (plus ambient, vercel). Recompute each at published prices. claude-plan, ChatGPT, kimi-code. | Pay-per-use views equal the recomputation to the micro-dollar; subscriptions shown as subscription (NV where no credential). |
| C11 | AC11 | This record + one code-blind Opus 5.5 High review. | — |
| C12 | AC12 | Real requests on each route with a credential: Z.AI general (zai), Z.AI coding URL (custom provider), zai-anthropic. Placeholder attempts on Kimi Code / Moonshot / BigModel coding & general / OpenAI API key. Unknown route (local proxy). Overflow note checked on every page type. | Each route shows its declared basis; unknown route "not declared"; overflow note only on subscription work; day spent total excludes subscription work. |

Linux repeats C3, C6(b,d), C9, C10 (zai) and C12 (Z.AI routes) on its own binary.
