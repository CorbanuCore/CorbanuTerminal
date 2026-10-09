# PF-60-S05 independent functional acceptance (2026-10-09)

**Result.**

- On real runs, the accounting changes behave as the sprint specifies on macOS and Linux. Every pay-per-use total I
  report equals my recomputation from provider-reported usage to the micro-dollar.
- Each finding of the body review is fixed where it could be observed, but several are only partly verifiable with the
  credentials in the vault. See [Body-review findings](#body-review-findings).
- One AC7 mode is a real gap: with a read-only state DB, or another writer holding it for about 2 minutes, compaction
  finishes but the turn then fails. The failure is in the provider-request throttle check, not in accounting
  ([#351]).
- Credentials missing from the vault make some parts **NOT VERIFIABLE**: Kimi Code, ChatGPT login, a Claude plan login
  a built binary may use, OpenAI API key, Kimi Open Platform and BigModel.
- [#352] collects low-severity `/cost` display nits.

Executor: an independent Codex worker, code-blind. One independent reviewer audited the first draft:
[REVIEW.md](REVIEW.md) (conclusion "Supported, with corrections"). The corrections are listed at the end.

## Verdicts

Platform: **M** = macOS, **L** = Linux.

| AC | Verdict | Platforms | Evidence |
| --- | --- | --- | --- |
| AC1 declarations | **PASS** | M, L | **Observed with placeholder credentials:** 20 of 21 built-ins recorded an attempt on L and 19 on M (`claude-plan` was not run on M). Every basis matches the defaults table: Kimi Code, Z.AI Anthropic and Claude Plan are subscription; Ollama and LM Studio are local; the rest are pay per use (`captures/{mac,rtx}/*-c1-day.txt`, `data/ledger-dumps-*.txt`). Ollama and LM Studio are told apart only in the ledger dump, because `/cost` labels both "gpt-oss" ([#352]). **Built-in `openai`:** it never sends a request with a bad key (WebSocket 401 at the handshake), so its row rests on the unit test `every_built_in_provider_declares_its_default_basis` (74/74 pass, `data/unit-tests-rtx.log`). |
| AC2 not from auth type | **PASS** (observable parts); **NOT VERIFIABLE**: ChatGPT login, mutation tests | M (Bedrock, routes); M+L (Kimi) | **Bedrock:** `amazon-bedrock` with a command `auth` is pay per use (`mac-c2-bedrock-cmdauth-day`; config in `data/configs/mac-h-c2.toml`). **Kimi:** `kimi-code` with `KIMI_API_KEY` is subscription. **OpenAI:** the built-in recorded nothing (see AC1). A custom provider at `api.openai.com/v1` with an API key is pay per use (`mac-c12-routes-day`), and `basis_does_not_follow_the_auth_type` passes. **Mutation tests:** they need source edits, which a code-blind executor can't make. |
| AC3 override, invalid, undeclared | **PASS** | M, L | **Override:** with `model_providers.zai.billing = "subscription"`, T2 shows "(set in your config)" and is not spent. **History:** T1 keeps its estimate, $0.017597 on M (exact 0.01759704) and $0.003078 on L (exact 0.00307788). **Invalid value:** `billing = "free"` fails both M builds and L with `unknown variant … in model_providers.zai.billing` (`data/c3-invalid-billing-*.txt`). **Undeclared custom provider:** "Billing basis not declared … Next step: set model_providers.zaiproxy.billing = …"; tokens are recorded, and the work is counted as neither spent nor subscription. |
| AC4 ChatGPT, priority, image, realtime, pane bridge | **NOT VERIFIABLE** | — | These paths need a ChatGPT login or Claude panes. **Substitute evidence (M, L):** subscription routes with no catalogue price (Z.AI coding plan, `zai-anthropic`, Kimi Code, a `billing = "subscription"` override) show "Covered by your subscription". They never show "Pay per use" or "no price", and are not in the "N attempts had no price" count: 11 = exactly the unpriced pay-per-use attempts in `mac-c1-day`. |
| AC5 Kimi Code real | **NOT VERIFIABLE** | — | No membership key in the vault. A placeholder-key attempt records as "Covered by your subscription" (M, L). |
| AC6 ledger format | **PASS**, with caveats below | M, L | **Tests:** `ledger_written_before_plan_basis_validates_and_reads` and `newer_ledger_format_is_refused_by_name` pass. **Pre-`5bae` ledger (L):** my own ledger, written by a binary built at `3b6c338f8f`, reads in the new build: 4 requests and 25,781 tokens, equal to the provider-reported usage. A new turn is recorded: "$0.003028 (rounded)", recomputed 0.00302848. The ledger stays in format 1. **Real ledgers:** 09-22..10-09 copies read on 09-23, 10-02 and 10-03 (M, L). **Future format:** one warning per session, turns succeed and nothing new is recorded (M, L). |
| AC7 best effort | **PARTIAL**: PASS as written for the modes run; turn-level **FAIL** in two modes, caused outside accounting ([#351]); **NOT VERIFIABLE**: validation failure, post-response failure, web search, image generation | M, L (refusal); M (others) | **Passing modes** (auto-compaction finished, the turn finished, one gap warning): new attempts refused by the store, M `c7b-t3` and L `l-c7-t2`; the store locked for 40 s, past accounting's ~16 s budget, M `c7e-t2`. **Failing modes** (compaction finished and one gap warning appeared, then the turn failed in the provider-request throttle check, `M c7c-t2`, `c7d-t2`): a read-only DB, and a lock held for about 2 minutes. My frozen expectation was "the turn finishes", so I record these as turn-level failures. **Not injected:** every failure I injected hit before the request was sent (`step="admit attempt"` or `"open sampling"`). A failure after the paid response (the review's Major 4 case) was not injected. My corrupted-estimate run (`c7f`) caused no write failure: validation is not on the write path within the hour, and `/cost` flagged that conversation as unreadable. Web search and image generation need OpenAI credentials. |
| AC8 guardian | **PASS** | M, L | **Single review:** with `approvals_reviewer = "auto_review"` and an escalated `curl`, the reviewer's request is under the parent conversation and Technical details shows turn `review:…` (M; L from the ledger dump). The conversation equals the provider-reported total: 0.0448102 on M and 0.012911 on L. **Parallel reviews (M `c8b`):** 3 reviews, 2 of them concurrent, left only 1 reviewer rollout file, so at least one ran in an unpersisted fork. All 3 are under the parent thread as `review:` turns, and the conversation is 6 requests, exact 0.03890804 = provider-reported. Nothing is counted twice: the reviewer's own thread records nothing. |
| AC9 ephemeral | **PASS** | M, L | `exec --ephemeral` (2 requests) logs exactly 1 `accounting.excluded="ephemeral_session"` and records nothing. The control logs 0 and records 2. The 2026-10-09 coverage audit lists `exec --ephemeral`. |
| AC10 real providers | **PASS** (pay per use); **NOT VERIFIABLE**: `claude-plan`, ChatGPT, `kimi-code` | M, L | **Exact to the micro-dollar:** `zai` GLM 5.2 (M: 0.01170668, 0.00697936; L: 0.00307788, 0.00498788, 0.00501388) and `deepseek` off-peak (M: 0.001753362, 0.000170376; L: 0.000315918, 0.000233664). **OpenRouter:** "Billed by OpenRouter" equals the provider's `usage.cost` (0.0017016, 0.0083586; 0.00180282). **Others:** `ambient` and `vercel` are pay per use (M). |
| AC11 independent acceptance | **Done, with an isolation deviation** | — | The design was frozen at 12:15:40Z, before any S05 result was read (`FROZEN-DESIGN.md`, `data/frozen-design-sha256.txt`). One code-blind Opus 5.5 High review: [REVIEW.md](REVIEW.md). AC11 asks for the [isolated execution gate](../../../../code-blind-functional/isolated-execution.md); I ran on the host under self-discipline, as S03's executor did, not inside an OS-enforced sandbox. |
| AC12 option B routes | **PASS** (Z.AI routes, unknown route, overflow note); **NOT VERIFIABLE**: real requests on Kimi Code, Kimi Open Platform, BigModel coding and general, OpenAI API key, ChatGPT | M, L (Z.AI); M (placeholder routes) | **Real requests:** the Z.AI general URL is pay per use; the coding URL (custom provider) and `zai-anthropic` are subscription. A local proxy route is "not declared". **Overflow note:** across all 65 captured S05-build views, it appears exactly on the views that contain subscription work; no pay-per-use-only view shows it. The 3 S03-build comparison views predate the note. **Placeholder keys:** the declared basis is recorded for `api.moonshot.ai` and BigModel general (pay per use), BigModel coding and a Kimi Code custom route (subscription), and `api.openai.com/v1` with an API key (pay per use). **Z.AI plan state:** I can't see whether the vault account holds a GLM Coding Plan or whether its balance was debited (no console access). The coding URL and the Anthropic route served every request with this key. |

### AC6 caveats

- **Partial reads on real ledgers.** Some real-ledger day views say "N conversations could not be read in full; their
  cost is unknown and not included". N is 1, 3 and 34 on 09-23, 10-02 and 10-03 for the `acct63` copy (and 1, 22 and
  44 for the runtime-home copy).
  - The S03-accepted build `63ea3d0cbd` gives **identical** output on the same copy: the same counts, totals and lines
    (`captures/rtx/rtx-c6-a63-{s03,acct}-build-*.txt`). So this is not an S05 regression.
  - On 09-23, the conversation not read in full is the one with 1,010 attempts that day. I infer a read-size limit;
    the cause is not visible code-blind.
- **What my pre-`5bae` ledger covers.** It holds 4 unpriced attempts and 8 pre-`5bae` estimate payloads, but no price
  snapshot (the old build wrote none for these providers). The pre-`5bae` *price-record* path of Major 3 therefore rests
  on the lane's committed fixture test.
- **No real ledger from before 2026-09-21 exists** on either host (`data/real-ledger-search.txt`); the absence is
  recorded here.
- **Not tested:** an older build opening a newer ledger.

## Body-review findings

| Finding | Status | Basis |
| --- | --- | --- |
| Blocker 1 (Kimi Code counted as spend) | **Partly verified** | Placeholder attempts record as subscription on both platforms. No real Kimi tokens (AC5 NOT VERIFIABLE). |
| Major 2 (priceless plan work shown as "Pay per use") | **Partly verified** | Priceless subscription routes (Z.AI coding, `zai-anthropic`, a `billing` override, Kimi Code) show subscription and are not in the no-price count. The named ChatGPT, priority-tier, image, realtime and pane-bridge paths: NOT VERIFIABLE. |
| Major 3 (older ledgers stop reading) | **Verified for estimates and real 09-22+ ledgers; price-record path by the fixture test** | See the AC6 caveats. |
| Major 4 (best effort) | **Partly verified** | Admission and open failures: compaction returns. Post-response and validation failures: not injected. Web search and image: NOT VERIFIABLE. Turn-level failure in two modes outside accounting ([#351]). |
| Major 5 (uncollected sessions) | **Verified** | `exec --ephemeral` gives one warning. Guardian reviews, including parallel ones without a persisted thread, are recorded under the parent conversation. |
| Minor 10 (command `auth` treated as subscription) | **Verified** (macOS) | Bedrock with a command `auth` is pay per use. |

## What I read (code-blind boundary)

- **Before freezing the design:**
  - the sprint record and plan header;
  - `billing-basis-defaults.md`, `both-behaviour-options.md`, `ledger-format-check.md`, `acct-coverage-audit-20261009.md`;
  - the S03 acceptance README and tools, and the S03 body review;
  - `sec-common.md`;
  - `docs/` (Claude plan authentication, model providers, the Z.AI integration);
  - `/cost help`.
- **After freezing:** `real-provider-checks-20261009.md`, the updated sprint record and `qa/demos/index/PF-60-S05.md`.
  They were used only to confirm that no vault label had been missed.
- **Deviations:**
  - I grepped `codex-rs/core/config.schema.json` once for the `approvals_reviewer` key name.
  - `git diff --stat` between `38362eaf57` and `d7846e29d5` showed the names of changed `codex-rs` files (not their
    contents).
- **Not read:** implementation source, PR diffs, lane briefs, worker logs, review files.

### What each verdict rests on

| Evidence | Used for |
| --- | --- |
| `/cost` screens | AC1, AC3, AC4, AC6, AC8, AC10, AC12, and the overflow note |
| Exec JSON and log lines | AC7 (turn outcome, gap warning) and AC9 |
| Ledger dumps ([tools/ledger_dump.py](tools/ledger_dump.py)) | Telling Ollama from LM Studio, Linux `review:` turns, and the AC9 control |
| Unit tests | The built-in `openai` row, and the named tests in AC2 and AC6 |

## Setup

- **Commit:** `origin/main` `d7846e29d5` (merge of PR #346). I polled PR #346 every 5 minutes until it merged at
  12:45:39Z.
- **Builds:** see [data/build-receipts.txt](data/build-receipts.txt).
  - Developer-accounting debug builds on macOS 26.6.2 arm64 and Linux 6.8 x86_64 (RTX box).
  - A default build on macOS, for the config check.
  - On Linux, a `3b6c338f8f` (`5bae03414e^`) build to write a pre-`5bae` ledger, and the S03-accepted `63ea3d0cbd` build
    for the C6 comparison.
- **Safety:**
  - Every built-binary run had `CORBANU_TEST_NO_NATIVE_KEYRING=1` and its own disposable `CODEX_HOME`, `CORBANU_HOME`
    and `PFTERMINAL_HOME`.
  - Keys came only from the installed `corbanu vault auth-helper provider/<label>`, substituted on the consuming
    command. The form is `env "KEY=$(…)" corbanu-acct exec …` ([tools/execrun.sh](tools/execrun.sh)), so the value
    passes briefly through `env`'s argv. That is a small variant of the prescribed `KEY="$(…)" cmd` form.
  - On Linux, the substitution feeds the ssh stdin of the consuming command ([tools/rtxrun.sh](tools/rtxrun.sh)).
  - `claude-plan` was not run on macOS, because its login source can reach the keychain. On Linux, a fake `corbanu` on
    `PATH` printed a placeholder token, so a 401 attempt was recorded. I did not work around the vault's block on
    `provider/claude-code-oauth-token`.
  - View-only TUIs used a placeholder key. Real ledgers were copied with `sqlite3 -readonly … .backup`
    ([tools/copy-real-ledgers.sh](tools/copy-real-ledgers.sh)).
- **Key scan:**
  - This directory: 0 hits for every label used, and 0 key-shaped strings ([data/key-scan.txt](data/key-scan.txt)).
  - All scratch logs, homes and captures on both hosts were scanned the same way before deletion: 0 files with a hit.
- **Credentials present:** `provider/zai_api_key`, `deepseek_api_key`, `openrouter_api_key`, `ambient_api_key` and
  `ai_gateway_api_key`.
- **Absent:** Kimi Code, Kimi Open Platform, OpenAI API key, ChatGPT login, Anthropic API key, BigModel, Meta,
  Baseten, Corbanu API and Bedrock. `claude-code-oauth-token` exists but is provider-only.

## Method

- **Real work:** an empty git repo holding two text files. Prompts asked for a one-line reply or shell commands (`ls`,
  `cat`, `wc -c`, escalated `curl`). Literal prompts are in `captures/exec-json/`; every home's `config.toml` is in
  `data/configs/`.
- **Provider-reported usage, independent of the ledger:**
  - Raw SSE events, traced before parsing (`RUST_LOG=codex_api=trace`). The usage-bearing events are archived in
    `data/provider-usage-{mac,rtx}.jsonl` ([tools/usage_extract.py](tools/usage_extract.py)).
  - An external pass-through proxy for custom routes (`data/proxy-zai-*.jsonl`).
  - A stdlib mock for the local providers.
- **Recomputation:** [tools/recompute.py](tools/recompute.py) uses exact Decimal arithmetic and dedups by provider
  response id. Prices are from the providers' own pages, read on 2026-10-09:
  - Z.AI GLM-5.2: $1.40 / $0.26 / $4.40 per 1M tokens ([data/zai-pricing-20261009.txt](data/zai-pricing-20261009.txt)).
  - DeepSeek `deepseek-flash` off-peak: $0.15 / $0.003 / $0.60. All DeepSeek runs were 13:06–13:26Z on a Friday,
    outside the peak windows ([data/deepseek-pricing-20261009.txt](data/deepseek-pricing-20261009.txt)).
  - OpenRouter's own `usage.cost` per request.
  - Outputs: [data/recompute-mac.txt](data/recompute-mac.txt) and [data/recompute-rtx.txt](data/recompute-rtx.txt).
- **Store failures:**
  - a SQLite trigger refusing inserts into `draft_accounting_attempts`;
  - an exclusive transaction held by another process ([tools/hold_lock.py](tools/hold_lock.py)), for 40 s and for about
    2 minutes;
  - `chmod a-w` on the state DB;
  - one estimate row made noncanonical;
  - for the future format, an extra `_accounting_migrations` row (version 99).
- **Forcing compaction:** `model_auto_compact_token_limit` set just above the first request's size, then
  `exec resume --last`.
- **Screens:** [tools/cap.sh](tools/cap.sh) saves both the line union (identical lines collapse, as in S03) and, for the
  C1 day views, every distinct frame verbatim (`captures/raw/`).

## Reconciliation

| View | `/cost` | Independent | Match |
| --- | --- | --- | --- |
| M `zai` T1, then T2 under `billing = "subscription"` | T1 Pay per use, exact 0.01759704; T2 subscription, "not billed per token"; day spent $0.017597 | T1 12,612 / 64 / 3 → 0.01759704 | exact; T2 not spent |
| L, same | T1 exact 0.00307788; T2 subscription | 6,431 / 5,248 / 13 → 0.00307788 | exact |
| `zaiproxy` (no declaration), M / L | not declared; 9,898 and 3,715 tokens | proxy: 9,883 + 15, 3,702 + 13 | tokens exact |
| M `zai` C10 | 0.01170668; 0.00697936 | same | exact |
| M `deepseek` | 0.001753362; 0.000170376 | same | exact |
| M `openrouter` | billed $0.001702, $0.008359 | stated 0.0017016; 0.00691306 + 0.00144554 | equal at display precision |
| L mixed day (zai, deepseek, coding plan, zai-anthropic) | spent $0.005304; subscription "not available" | 0.00498788 + 0.000315918 = 0.005303798 | exact |
| L pay-only day | zai 0.00501388, deepseek 0.000233664, OpenRouter billed $0.001803 | same; stated 0.00180282 | exact |
| Guardian M / L / M parallel | 0.0448102 / 0.012911 / 0.03890804 | same (parent + reviewer requests) | exact |
| L pre-`5bae` ledger + new turn | 4 old requests (25,781 tokens, unpriced as recorded) + 1 new; "$0.003028 (rounded)" | 25,781; 0.00302848 | exact at display precision |
| M Vercel `kimi-k3` | billed $0.037207; "at least $0.000540" (cache-write count missing) | Vercel metadata 0.036567 + 0.00054 + 0.0001 fee = 0.037207 | exact |

**List-price spend of this QA:**

- About $1.17 pay per use: Z.AI $1.12, Vercel $0.037, OpenRouter $0.012, Ambient $0.005, DeepSeek $0.0025. Most of it
  came from the forced-compaction runs.
- About 71k tokens on subscription routes.

## Incidents (harness, not product)

1. **Proxy died.** The first `zaiproxy` proxy exited with its subshell. T4 and T5 failed to connect and left 30 failed
   `zaiproxy` attempts in `h-c3`. They were rerun as T4b and T5b.
2. **Compaction loop.** `c7f-t1` set the compaction limit below the base prompt, so a fresh session compacted in a loop
   until the repeated-tool-call guard stopped it. No accounting was involved.
3. **Lock released early.** The ~2-minute lock holder in `c7c` was killed when its parent shell ended. The lock was
   still held when the turn failed at 13:19:23Z.
4. **Future-checkpoint ledger.** The `acct64` real-ledger copies read "Unavailable": their retention checkpoint is
   2026-12-21, from an earlier faked-clock experiment. That is clock-behind behaviour (#287), not a format failure.
5. **`c7a` did not compact.** It ran on a new thread. For macOS, the insert-refusal evidence is `c7b-t3`.

## For Travis

- **Recommendation:** accept PF-60-S05 for what can be checked today.
  - Subscription work is never counted as spending.
  - Undeclared routes say so instead of guessing.
  - Old and real ledgers still read.
  - Compaction, guardian reviews and ephemeral sessions behave as specified.
  - No total disagrees with an independent recomputation.
- **Open items:**
  - [#351] is a small follow-up outside accounting, so that a busy or read-only state DB doesn't end the turn.
  - [#352] is cosmetic.
- **To close the NOT VERIFIABLE parts, provide:**
  - a Kimi Code membership key (AC5, and Kimi in AC10 and AC12);
  - a ChatGPT login (AC4, AC10, AC12);
  - an OpenAI API key (web search and image in AC7, and the API-key route in AC12);
  - a Claude plan token that a disposable-home build may use (real `claude-plan` in AC10).

## Review corrections applied

The reviewer recomputed every headline number and found no disagreement. Following its report:

- **AC7** is now PARTIAL. The record states that nothing was injected after a paid response, that `c7f` caused no write
  failure, and that the frozen "turn finishes" expectation was not met in two modes. It cites `c7b-t3` for macOS.
- **AC6** now discloses the partial reads and adds the S03-build comparison (identical output). It states what the
  pre-`5bae` ledger covers and records that no real pre-09-21 ledger exists.
- **The top line** no longer says the findings are "fixed on real runs". The per-finding table is added.
- **AC8** adds a parallel-review run (`c8b`) showing that unpersisted reviewer forks are attributed to the parent.
- **AC3** archives the invalid-value errors for both macOS builds and for Linux.
- **AC12 and AC2** now list the Kimi Code and OpenAI API-key real requests as NOT VERIFIABLE, and say the built-in
  `openai` recorded nothing.
- **Archived:** usage extracts, the Z.AI pricing extract, every `config.toml`, and the real-ledger copy step.
- **Safety:** the scratch logs were scanned on both hosts, and the `env` argv variant is disclosed.
- **Corrected** "every verdict rests on `/cost`" (see the evidence table above), the spend figure, the frozen time,
  the per-AC platforms, the DeepSeek column header and "exact" versus "rounded".
- **Not changed:** the reviewer's L2 nit about OpenRouter rows saying "No money is stated" next to "Billed cost" is
  added to [#352] as a comment, not as a separate issue.

[#351]: https://github.com/CorbanuCore/CorbanuTerminal/issues/351
[#352]: https://github.com/CorbanuCore/CorbanuTerminal/issues/352
