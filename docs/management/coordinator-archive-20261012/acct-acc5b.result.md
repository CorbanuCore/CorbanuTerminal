The re-run is done and merged as **PR #362**. Blocker 1 is fixed on real Kimi Code requests and every route records the billing basis it declares. One item fails: OpenAI API-key cost estimates don't match OpenAI's published prices, filed as **#361**.

All runs used origin/main `5d283fde18` with developer accounting, on macOS (M) and Linux (L).

| Item | Verdict | Evidence |
|---|---|---|
| AC5 Kimi Code real (Blocker 1) | **PASS** (M, L) | Real `k3` work shows "Covered by your subscription" with $0 spent. The day's estimate stays at $0.009627 before and after the Kimi turn. The overflow note appears only after it. Tokens (10,428) match the provider's own count. |
| AC10 `kimi-code` | **PASS** (M, L) | Its request page says "Billing basis: subscription". |
| AC10 `claude-plan` | **PASS** (L) | Subscription, nothing spent, overflow note shown. The API-price figure ($0.167180) is labelled as what it would cost, not as spending. 8,355 tokens match the provider's count. |
| AC10 OpenAI API key | **Basis PASS, estimate FAIL** (#361) | Basis is pay per use on every model. For `gpt-5.6-luna`, `/cost` says "at least $0.009627" but the true cost at published prices is $0.00625965. Its rates are about 5x too high and cache writes have no rate. `gpt-5.6-terra` is shown 2.6x below its true cost. Other OpenAI models show "no price". |
| AC12 Kimi Code routes | **PASS**; Kimi Open Platform **NOT VERIFIABLE** | The built-in route and a custom route at the Kimi Code URL are both subscription. The membership key isn't an Open Platform key. |
| AC12 OpenAI API-key routes | **PASS** | The built-in provider and a custom route at `api.openai.com/v1` are both pay per use. |
| AC12 overflow note | **PASS** | It appears on exactly the 9 of 18 captured screens that include subscription work. |
| AC2 (these routes) | **PASS**; ChatGPT and mutation tests **NOT VERIFIABLE** | A Kimi key in the environment still gives subscription. Mutation tests need source edits, which a code-blind run can't make. |
| AC4 Claude pane | **PASS** (L) | `/cost` in the pane shows "Covered by your subscription", 26,041 tokens. No "pay per use", "no price" or "check the bill". |
| AC4 priority tier | ChatGPT part **NOT VERIFIABLE** | On the API key, OpenAI served the priority tier and `/cost` recorded pay per use with "no price available". That's correct for an API key. |
| AC4 image generation and realtime | **NOT VERIFIABLE** | No image tool is offered on the API-key route for any of the 5 models tried. Realtime is off by default and voice-only. |
| AC7 web search | **PASS** (M) | The turn finished with an answer after a completed search, with one gap warning, when accounting refused the write before the request. The standalone search tool also passed with the failure after the paid response. |
| AC7 first run's failing modes | **PASS** (M) | Read-only database, and another process holding the database for about 2 minutes: compaction ran and the turn finished. Fixed by #351, which landed after the first run. |
| AC7 image generation | **NOT VERIFIABLE** | No image tool on the API-key route. |
| ChatGPT-login items | **NOT VERIFIABLE** | You won't swap the ChatGPT login. |

**Spend:** about $0.41 on the OpenAI key at list prices. Kimi Code used about 24k tokens of membership and Claude about 34k tokens of subscription. No image was generated.

**Rotate `openai-api-key` as a precaution.** One probe run with full trace logging wrote the OpenAI key to a local scratch log. I deleted it unread and it was never committed or sent anywhere. Every later log and the committed folder were scanned for all three keys: 0 hits.

**Other notes:**
- I ran `claude-plan` only on Linux so nothing could reach the macOS keychain.
- The independent reviewer's verdict was "Supported with corrections"; all corrections are applied in the record.
- Scratch homes, the build target and the worktree are deleted on both machines.

**Recommendation: accept PF-60-S05, but only if you explicitly waive two things:**
- **(a) AC10's exact-estimate requirement for OpenAI API keys** until #361 is fixed. S05 deliberately didn't touch prices and the basis is right, but OpenAI API-key cost estimates are wrong in both directions until then.
- **(b) AC11's isolated execution gate.** Like the first run, this ran on the normal machines, not in the enforced sandbox. One effect: on the two image prompts the model read a skill from your real `~/.local/share`. Its script found no key and made no request.

Without those waivers, AC10 fails as written.

Four things stay open for later:
- the ChatGPT-login checks;
- image generation and realtime;
- Kimi Open Platform and BigModel, for which there are no keys;
- the mutation tests.

**Things I noticed but didn't file:**
- **OpenAI key variable:** in `exec`, setting only `OPENAI_API_KEY` gets a 401 at the handshake; `CODEX_API_KEY` works. The defaults table names `OPENAI_API_KEY`.
- **Claude token in the TUI:** with only the environment token, the TUI flagged the Claude credential as needing attention. Claude panes wouldn't run until I saved the token through the masked entry in the scratch home. `exec` used the environment token directly.
- **Pane label:** the "Opus 5 Claude Plan" pane was actually served Claude Opus 5.5.