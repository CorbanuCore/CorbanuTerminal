# PF-60-S05 independent acceptance: targeted re-run (2026-10-09)

**Result.**

- Re-runs only the items the [first run](../README.md) left NOT VERIFIABLE for lack of credentials, now that the vault
  holds a Kimi Code membership key, a Claude subscription setup token and an OpenAI project API key.
- **Blocker 1 is fixed on real requests.** Real Kimi Code `k3` work records as subscription, adds $0 to the day's
  spent total and carries the overflow note (macOS and Linux).
- **Every route records its declared basis:** `kimi-code` subscription, `claude-plan` subscription, Claude pane work
  subscription, OpenAI API key pay per use (built-in and custom route).
- **AC10 FAILS as written for the OpenAI API key.** The estimate does not match OpenAI's published price ([#361]):
  - **Rates:** the only priced models use rates about 5x (`gpt-5.6-luna`) and 25% (`gpt-5.6-terra`) above the
    published ones.
  - **Cache writes:** they have no rate at all.
  - **So the totals are wrong both ways:** luna's "at least" lower bound is *above* the true cost, and terra's total
    is about 2.6x *below* it.
  - This is the price catalogue, not the basis; the basis is right.
- **AC7, web search:** after injected accounting failures, the turn completes with an answer after a completed
  `web_search` call.
  - **Built-in search:** tested with the refusal before the request.
  - **Standalone search tool:** tested before and after the paid response.
  - **Image generation:** NOT VERIFIABLE, because no image tool is offered on the API-key route.
- **The first run's two AC7 turn-level failures now pass on this binary:** a read-only DB, and a DB another writer
  holds for about 2 minutes. #351 was merged in between.
- ChatGPT-login items stay **NOT VERIFIABLE**: Travis won't swap his ChatGPT account login.

Executor: an independent Codex worker, code-blind, as in the first run. One code-blind reviewer audited the first
draft: [REVIEW.md](REVIEW.md) (conclusion "Supported with corrections"). The corrections are listed at the end.

**Which binary.** The first run's verdicts were recorded on `d7846e29d5`. This re-run's verdicts are on `5d283fde18`.
AC1's "one recorded binary" therefore holds per record, not across the two.

## Verdicts

Platform: **M** = macOS, **L** = Linux. Captures are under `captures/{mac,rtx}/`, exec JSON under
`captures/exec-json/`.

| Item | Verdict | Platforms | Evidence |
| --- | --- | --- | --- |
| **AC5** Kimi Code real (Blocker 1) | **PASS** | M, L | **Day spent unchanged:** home `h-r1b`, T1 OpenAI API key, then T2 real `kimi-code` `k3`. The day's estimate is "at least $0.009627" before T2 (`mac-r1-day-after-t1`) and after T2 (`mac-r1-day-after-t2`). **Kimi line:** "Covered by your subscription (not billed per request). 2 requests, 10,428 tokens". **Conversation:** "No recorded attempt here was billed per token", "Known subtotal exact USD: none" (`mac-r1-t2-kimi-conv`). **Tokens:** 10,307 in / 121 out, equal to the raw provider usage (`data/recompute-mac-subscription-tokens.txt`). **Overflow note:** absent before T2, present after. **L:** same (`rtx-lx-day`, 6,527 tokens). |
| **AC10** `kimi-code` | **PASS** | M, L | As AC5. The technical page says "Billing basis: subscription" (`mac-r1-t2-kimi-request1-technical`). |
| **AC10** `claude-plan` | **PASS** | L | **Real turn:** one `claude-fable-5-1-plan` turn with the setup token. "Covered by your subscription", "Known subtotal exact USD: none". **Not spent:** "Same work at API prices: $0.167180" is labelled as what it would cost (`rtx-r4-claude-conv`). **Tokens:** 8,355 = the raw SSE usage (4 in + 8,347 cache write + 4 out). **Overflow note:** present. **Platform:** Linux only; see Safety. |
| **AC10** OpenAI API key | **Basis PASS; estimate FAIL** ([#361]) | M, L | **Basis:** every request is "Pay per use" (`gpt-5.6-luna`, `-terra`, `gpt-5.4-mini`, `gpt-5.2`, `gpt-6-luna`). **`gpt-5.6-luna` (M T1):** `/cost` says "at least $0.009627 (1 attempt had no price)", at rates of $1 / $0.10 / $6 with no cache-write rate (`mac-r1-t1-request1-technical`). The recompute at the published $0.20 / $0.02 / $0.25 cache write / $1.20 is **$0.00625965**: the lower bound is 1.54x the true cost (L: 2.0x, against $0.00480615). **`gpt-5.6-terra`:** `/cost` says at least $0.024068, at $2.50 / $15; the recompute at the published $2.00 / $12 plus $2.50 cache writes is **$0.062594**, 2.6x higher. **Other models:** `gpt-5.4-mini`, `gpt-5.2` and `gpt-6-luna` have "no price available". All prices: `data/openai-pricing-20261009.txt`. |
| **AC10** ChatGPT login | **NOT VERIFIABLE** | — | Travis won't swap his ChatGPT login. |
| **AC12** Kimi Code routes | **PASS** (Kimi Code); **NOT VERIFIABLE** (Kimi Open Platform) | M | **Built-in `kimi-code`:** subscription (`mac-r1-day-after-t2`, `mac-r1-t2-kimi-conv`). **Custom provider at `https://api.kimi.com/coding/v1`** with the same real key: subscription (`mac-r3-day`). **Kimi Open Platform:** the membership key is not an Open Platform key. |
| **AC12** OpenAI routes | **PASS** (API key, built-in and custom); **NOT VERIFIABLE** (ChatGPT) | M, L | **API key:** built-in `openai` and a custom provider at `https://api.openai.com/v1` (Responses wire) with the real key: both pay per use (`mac-r3-day`). |
| **AC12** overflow note | **PASS** | M, L | The note appears on exactly the 9 of 18 captured views that contain subscription work, and on none of the other 9. That includes the Claude conversation and the Claude pane view. |
| **AC2** (these routes) | **PASS** (observable); **NOT VERIFIABLE**: ChatGPT, mutation tests | M, L | **Kimi:** `kimi-code` with an environment key is subscription. **OpenAI:** with an API key it is pay per use. **Claude:** `claude-plan` with the token in the environment is subscription. **Mutation tests:** they need source edits, which a code-blind executor can't make. |
| **AC4** pane-bridge Claude work | **PASS** | L | **Setup:** a Claude pane (`/panes` → Claude Pane → "Opus 5 Claude Plan") ran one real turn on the setup token. **`/cost` in the pane:** "Claude Plan · Claude Opus 5.5 — Covered by your subscription … 1 request, 26,041 tokens". **Absent:** no "Pay per use", "no price available" or "check the bill", and no "had no price" count (`rtx-r4p-pane-cost`). **Tokens:** 2 + 26,035 cache write + 4 = the usage report Claude Code itself prints at the end of the pane turn (same capture). That is Claude Code's own record, not a wire trace. |
| **AC4** priority tier | ChatGPT part **NOT VERIFIABLE**; API-key observation consistent | M | **Run:** `gpt-5.6-luna` on the API key with `-c service_tier='"priority"'` (run `r5-prio`). The served tier was `"service_tier":"priority"` in both `response.completed` events (`data/provider-usage-mac.jsonl`). **Recorded:** pay per use, "no price available", not the Standard rate (`mac-r5-prio-request2-technical`). That is correct for an API key; AC4's case is a ChatGPT priority turn. **Recompute at the published Fast rates:** $0.0125224. |
| **AC4** image generation (API key) | **NOT VERIFIABLE** | M | **No image tool:** none is offered on the API-key route. Tool lists were checked in the `response.created` events for `gpt-5.6-luna`, `gpt-5.6-terra`, `gpt-6-luna`, `gpt-5.4-mini` and `gpt-5.2`. `features list` says `image_generation stable true`. **Two attempts:** the model drew the circle with a local script instead; no image API call was made. |
| **AC4** realtime (API key) | **NOT VERIFIABLE** | — | `realtime_conversation` is "under development, false" in `features list`. It is voice-only, so it can't be driven in tmux. |
| **AC4** ChatGPT unpriced model, ChatGPT image | **NOT VERIFIABLE** | — | ChatGPT login. |
| **AC7** web search, refusal before the request | **PASS** | M | **Injection:** a SQLite trigger refused inserts into `draft_accounting_attempts` (`data/r7-injections.txt`). **Built-in search (`r7-search-inj`) and standalone search (`--enable standalone_web_search`, `r7-sws-inj`):** each turn completed with an answer after a completed `web_search` item, with exactly one gap warning item. **Log:** for the standalone tool, the `alpha/search` request shows "sent unrecorded" at `admit attempt` (`data/ac7-log-excerpts.txt`). **Ledger:** 6 attempt rows before and after. |
| **AC7** web search, failure after the paid response | **PASS** (standalone search tool) | M | **Injection:** triggers refused inserts and updates on `draft_accounting_observations` (`r7-sws-post`). **Log:** "request sent unrecorded step=\"observe usage\"" after each paid response. **Result:** the turn completed with an answer after a completed `web_search` item, and one gap warning. This is the review's Major 4 case. The built-in search was not run in this mode. |
| **AC7** first-run failing modes, with compaction | **PASS** (was turn-level FAIL in the first run, [#351]) | M | **Setup:** `gpt-5.4-mini` with `model_auto_compact_token_limit = 14600`, then `exec resume --last`. **Read-only DB (`r8-t2-readonly`):** server-side compaction ran (`response.compaction.compacting`), the turn finished with an answer, one gap warning, and a notice that the shared state database is read-only. **Lock held about 2 minutes (`r8-t3-busy`, locked 19:27:58Z, released 19:30:08Z):** requests continued during the lock, compaction ran, the turn finished, one gap warning, and a "database is busy" notice (`data/ac7-log-excerpts.txt`). |
| **AC7** image generation | **NOT VERIFIABLE** | — | No image tool on the API-key route (see AC4). |

## Reconciliation

| Run | `/cost` | Independent (raw provider usage, published price) | Match |
| --- | --- | --- | --- |
| M `r1-t1` OpenAI `gpt-5.6-luna` | at least $0.009627; rates $1 / $0.10 / $6, cache write unpriced | 9,594 in (prewarm) + 3 in, 17,337 cache write, 5 out → **$0.00625965** | **no** ([#361]) |
| L `lx-t1` same model | at least $0.009627 | **$0.00480615** | **no** |
| M `r3-terra` `gpt-5.6-terra` | at least $0.024068; rates $2.50 / $0.25 / $15 | **$0.062594** (incl. 17,336 cache write at $2.50) | **no** |
| M `r5-prio` `gpt-5.6-luna` Fast | no price available | $0.0125224 at Fast rates | no figure shown |
| M `gpt-5.4-mini` runs | no price available | `data/recompute-mac.txt` | no figure shown |
| M `r1-t2` Kimi Code | subscription, 10,428 tokens, $0 spent | 10,428 | tokens exact |
| L `lx-t2` Kimi Code | subscription, 6,527 tokens | 6,527 | tokens exact |
| M `r2-kimicustom` | subscription, 7,429 tokens | 7,429 | tokens exact |
| L `r4-claude` | subscription, 8,355 tokens; API equivalent $0.167180 (not spent) | 8,355 (raw SSE); $0.167180 = 4×$10 + 8,347×$20 + 4×$50 per 1M at the catalogue rates the ledger recorded | tokens exact; equivalent internally consistent (not checked against Anthropic's page; not spent) |
| L Claude pane | subscription, 26,041 tokens | Claude Code's own usage report: 2 + 26,035 + 4 | exact |

For each `/cost` figure the product's arithmetic is internally exact: 9,594 × $1 + 3 × $1 + 5 × $6 = $0.009627. The
disagreement is the rates.

## Observations (not verdicts)

1. **`OPENAI_API_KEY` alone fails in `exec`.** `r3-apikey-env` stops with "401 Unauthorized: Missing bearer or basic
   authentication" at the WebSocket handshake. `CODEX_API_KEY` works. The defaults table lists `OPENAI_API_KEY` as the
   built-in credential.
2. **WebSocket prewarm.** Each built-in `openai` session first sends a prewarm request: about 6.6k–13.7k input tokens,
   0 output, recorded as turn `prewarm:…`. OpenAI reports usage for it, so my recompute includes it. Whether OpenAI
   bills it isn't visible to the client.
3. **Claude plan consumption.** The `claude-plan` exec turn shows "Plan consumption: 16,710 tokens", twice the 8,355
   tokens. The recorded plan rate is 2000 millis, i.e. 2x. Nothing client-side confirms it.
4. **Claude pane label.** The picker's "Opus 5 Claude Plan" pane was served `claude-opus-5-5`. `/cost` follows the
   served model.
5. **Claude token in the TUI.** With the token only in `CLAUDE_CODE_OAUTH_TOKEN`, `/providers` showed the Claude
   Account as "Credential needs attention" and said "More than one Claude credential was found". The pane refused to
   run ("The current provider is unavailable or inactive") until the token was saved through the masked
   long-lived-token flow into the disposable home's vault. `exec` used the environment token directly. This is
   Claude-auth behaviour, outside PF-60.
6. **GPT-5.6/6 tools need the code-mode host.** These models run their tools through `codex-code-mode-host`. Without
   it next to the binary, they have no working tools, so I built it at the same commit.

## Safety

- **Disposable homes.** Every built-binary run had `CORBANU_TEST_NO_NATIVE_KEYRING=1` and its own disposable
  `CODEX_HOME`, `CORBANU_HOME` and `PFTERMINAL_HOME`.
- **How keys were resolved.** Only the installed wrapper resolved keys, without the test variables:
  `V(){ env -u … ~/.local/bin/corbanu vault auth-helper "$1" < /dev/null; }`. The value was substituted only on the
  consuming command, e.g. `KIMI_API_KEY="$(V provider/kimi_api_key)" run …` ([tools/execrun.sh](tools/execrun.sh)).
- **Linux.** The value went over ssh stdin into the remote process environment ([tools/rtxrun.sh](tools/rtxrun.sh),
  [tools/rtx-execrun.sh](tools/rtx-execrun.sh)).
- **`claude-plan` ran only on Linux.** That host has no macOS keychain, so neither the built binary nor the Claude
  Code child it starts could reach one.
  - **`exec`:** the `claude-plan` command auth ran the built binary itself as `corbanu` on `PATH`.
  - **Pane:** Claude Code 2.1.295 ran from a disposable `HOME`, with `CLAUDE_CONFIG_DIR` in scratch.
  - **Pane token entry:** the token was entered in the masked view through `tmux load-buffer -` from stdin plus
    `paste-buffer -d`. It never appeared in argv or on screen, and was stored only in the disposable home's vault,
    which has been deleted.
- **Incident: OpenAI bearer written to a local scratch log.** A probe run with `RUST_LOG=trace` wrote the WebSocket
  handshake, including the bearer, to `probe-ws-trace.err` under the local scratch directory.
  - **Cleanup:** I deleted the file unread. It was never committed, copied or sent anywhere.
  - **Logging after that:** restricted to `tungstenite::protocol=trace` (frames only).
  - **Scans:** every later log was scanned against the real values: 0 hits.
  - **Travis:** rotate `openai-api-key` as a precaution, because deletion doesn't prove the bytes are gone from disk.
- **Isolation deviation.** As in the first run, I worked on the hosts under self-discipline, not inside the
  OS-enforced isolated gate that AC11 names. One concrete breach: on the two image prompts, the model read a global
  skill under the real user's `~/.local/share` and ran its script. Skills outside the disposable home are visible.
  The script found no API key, because provider keys are stripped from shell commands, and it made no request.
- **Key scan.** Every scratch log, home, capture and this directory were scanned against the three real values via
  `grep -F -f <(V label)` ([tools/keyscan.sh](tools/keyscan.sh)): 0 files on both hosts. This directory also has 0
  key-shaped strings ([data/key-scan.txt](data/key-scan.txt)).

## Method

- **Design.** Frozen at 18:52:00Z, before any real request ([FROZEN-DESIGN.md](FROZEN-DESIGN.md);
  `data/frozen-at.txt`).
  - **Read before freezing:** the sprint record, the defaults table, the options memo, `real-provider-checks-20261009.md`,
    the first run's record and tools, `sec-common.md`, and the user docs on Claude plan auth, Claude panes,
    configuration and authentication.
  - **From the built binary:** `corbanu features list` and `corbanu debug models`.
- **Deviations from the frozen design.**
  - **Priority value:** R5a said `service_tier="fast"`. I used `"priority"`, the tier id `corbanu debug models` lists
    for these models (name "Fast").
  - **Post-freeze amendments:** extra OpenAI models to find one with a catalogue price (`gpt-5.4-mini`, `gpt-5.2`,
    `gpt-6-luna`, `gpt-5.6-terra`); web-search baselines; the standalone search tool; the after-response injection
    (`r7-sws-post`); and the first-run AC7 modes (`r8-*`), added after the reviewer noted #351 was in this binary.
- **Code-blind boundary.**
  - **Deviation:** I located the crate manifest `codex-rs/code-mode-host/Cargo.toml` by name to build the host binary;
    I did not read its source.
  - **Git log:** I read `git log --oneline` titles for #351.
  - **Not read:** implementation source, PR diffs, lane briefs, worker logs and review files.
- **Builds.** origin/main `5d283fde18` with developer accounting on macOS and Linux
  ([data/build-receipts.txt](data/build-receipts.txt)).
- **Independent channel.** Raw provider events, traced before parsing:
  - SSE via `codex_api=trace` (Kimi, Claude);
  - WebSocket frames via `tungstenite::protocol=trace` (OpenAI built-in, which uses WebSocket).
  - Extracted, with the served `service_tier`, to `data/provider-usage-*.jsonl`; recomputed with Decimal by
    [tools/recompute.py](tools/recompute.py), extended for OpenAI cache writes.
- **Prices.** OpenAI's pricing page, fetched today, covering every model used
  ([data/openai-pricing-20261009.txt](data/openai-pricing-20261009.txt)). The page notes that Priority was renamed Fast
  on 2026-07-30.
- **Views.** `/cost` captured in tmux by the first run's tools.
- **Ledger dumps.** [tools/ledger_dump.py](tools/ledger_dump.py) is a storage cross-check: it names turns (`prewarm:`,
  `search:`, `pane:`, `assess:`) and confirms the basis behind the `/cost` lines (`data/ledger-dumps-*.txt`). No
  verdict rests on it alone.
- **`service_tier` override.** It was passed on the command line (`-c service_tier='"priority"'`); the home's
  `config.toml` keeps `"default"`.

## Spend

- **OpenAI API key:** about **$0.41** at list prices.
  - **Traced:** $0.347 on macOS (`data/recompute-mac.txt`) and $0.005 on Linux (`data/recompute-rtx.txt`).
  - **Untraced:** about $0.013 (`r1-t1-openai`, the deleted probe).
  - **Web search:** 5 calls at $10 per 1k, $0.05.
- **Subscription:** Kimi Code about 24k tokens of membership use; Claude about 34k tokens of subscription use.
- **No image** was generated.

## For Travis

- **Recommendation: accept PF-60-S05 only with two explicit waivers.**
  - **(a) AC10's estimate clause:** waive "matches the published price to the micro-dollar" for OpenAI API-key routes
    until [#361] is fixed. S05 excludes changing prices and the basis is right. But today OpenAI API-key estimates
    are wrong in both directions (luna's lower bound is above the true cost, terra's total 2.6x below it), and most
    OpenAI models show "no price".
  - **(b) AC11 isolation:** accept the run without the OS-enforced isolation gate (see Safety).
- **What holds on real requests:**
  - Kimi Code membership work is never spent (Blocker 1).
  - Subscription work keeps its basis with or without a price.
  - Claude plan and Claude pane work are subscription.
  - The OpenAI API key is pay per use on both routes.
  - Web search and compaction survive accounting and state-DB failures.
- **Rotate `openai-api-key`** as a precaution (Safety).
- **Still NOT VERIFIABLE:**
  - **ChatGPT login:** AC4's ChatGPT paths, AC10, and AC12's ChatGPT route.
  - **Image generation and realtime:** not offered on the API-key route in this build.
  - **No key:** Kimi Open Platform and BigModel.
  - **Code-blind:** mutation tests.

## Review corrections applied

Applied from [REVIEW.md](REVIEW.md); the reviewer's recomputation agreed with every headline number.

- **Recommendation:** it now requires explicit waivers for AC10's estimate clause and for AC11 isolation. The
  `~/.local/share` skill read is now under Safety as an isolation deviation.
- **Price errors:** described by their effect on totals (luna's lower bound above the true cost; terra 2.6x low).
- **Price extract:** it now covers every model recomputed.
- **Priority evidence:** the served `service_tier` is archived in `provider-usage-mac.jsonl`, the override is
  recorded, and fast → priority is listed as a deviation.
- **AC7:**
  - the first-run read-only and busy modes were re-run on this binary (PASS) and labelled as amendments;
  - claims are narrowed to "completed with an answer after a completed `web_search` call";
  - the after-response mode is limited to the standalone tool;
  - log excerpts are archived.
- **Key:** rotation of the OpenAI key is recommended.
- **Citations:** fixed for AC12 Kimi and the pane tokens; the ledger's role is reconciled.
- **Paths:** the pane capture's path is redacted.
- **Spend:** corrected (Linux `lx-t1` is traced).
- **Injections:** the counts in `r7-injections.txt` are labelled.
- **Binaries:** the record says the first run's verdicts are on another binary.

[#351]: https://github.com/CorbanuCore/CorbanuTerminal/issues/351
[#361]: https://github.com/CorbanuCore/CorbanuTerminal/issues/361
