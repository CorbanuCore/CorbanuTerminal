# PF-60-S04 independent code-blind acceptance (2026-10-10)

**Result: every PF-60-S04 criterion I could exercise PASSES on current main, with real provider calls through the TUI.**

- **Pay-per-use totals recompute exactly.** Each pay-per-use total matches an independent recompute from the
  provider's own usage events at published prices, to the last digit the product shows. That covers 17 OpenAI
  requests (`gpt-5.4`, `gpt-5.6-luna`, `gpt-5.6-terra`) and 5 Z.AI GLM 5.3 Flash requests.
- **Waiver (a) re-check: PASS.** OpenAI API-key estimates are exact for `gpt-5.4` and for the two models S05 failed
  on, `gpt-5.6-luna` and `gpt-5.6-terra`, cache writes included. On the Fast tier the product shows
  "no price available" instead of guessing.
- **Both S05 gaps are closed on this binary.**
  - `/side` requests are recorded under the parent conversation as `side:`, and memory consolidation under the
    conversation that started it as `consolidation:`. `exec --ephemeral` is excluded, with exactly one warning.
  - A request with no price shows one "Price: none recorded" line, with no Price ID, source or rate lines.
- **Unknown values never show as zero.** A cancelled request is "usage incomplete" and the total becomes
  "at least …". A missing price is "no price available". A failed retry attempt has its tokens "not reported".
- **Totals hold after restarts.** `/resume`, kill -9 and restart leave every earlier request unchanged.
- **History and dates work.** Past days and ranges match per-day sums computed straight from the ledger rows, and
  invalid ranges are refused with a reason.
- **NOT VERIFIABLE:** ChatGPT login, image generation, realtime and TensorCash (see the end of the verdict table).
- **One display defect filed, [#396]:** an unpriced page still shows "Cache write cost: $0.000000" for 0 tokens. No
  total is affected.
- **Deviation:** like the lane run and S05, this run was not inside an OS-enforced isolation gate.

Executor: an independent Codex worker, code-blind by self-discipline; it was not inside an enforced isolation gate
(see Method, "Isolation deviation"). One separate code-blind reviewer audited the first draft: [REVIEW.md](REVIEW.md)
(conclusion "Supported with corrections"). The corrections are listed at the end.

## Candidate

| Item | Value |
| --- | --- |
| Commit | `b1e20a8ec6192ea16a4b3c7e730d28cb712d3707`, origin/main on 2026-10-10. `git diff --stat 1e9cd464b9 b1e20a8ec6 -- codex-rs` is empty ([data/out-of-band-steps.txt](data/out-of-band-steps.txt)), so this is the same code as the lane's commit. The binary is this run's own build, not the lane's (`cb378c27…`); these verdicts qualify this binary. |
| Binary | `corbanu 0.1.48`, Linux x86_64 debug, `--features codex-core/developer-accounting,codex-tui/developer-accounting`; sha256 `953802b1…a6d89cf1` ([build receipt](data/build-receipt.txt)) |
| Host | RTX box, Ubuntu 26.04.1, kernel 7.0.0-38. Linux, so no macOS keychain is reachable. |
| Homes | Disposable `h1` to `h6`, each with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and `CODEX_HOME`, `CORBANU_HOME` and `PFTERMINAL_HOME` set to the home; a trusted scratch git workspace |
| Keys | `openai-api-key` (as `OPENAI_API_KEY`), `provider/zai_api_key`, `provider/kimi_api_key` and `claude-plan-test-token` (as `CLAUDE_CODE_OAUTH_TOKEN`). The installed wrapper resolved each one on the consuming command and piped it over ssh stdin into one private tmux server's environment ([tools/q](tools/q), [tools/start.sh](tools/start.sh)). No key was printed or written. |
| Driver | The repo's tmux-harness rules (docs/tmuxHarness.md): a private `tmux -L ia-<home>` server per home, literal text and Enter sent separately, stable-capture waits. Every TUI command sent is in [data/keys-sent.log](data/keys-sent.log). Steps outside the driver are listed with their times in [data/out-of-band-steps.txt](data/out-of-band-steps.txt): build, seeding, the retry proxy, config edits, the extraction tools and the suite. That file also holds each home's config. |

## Verdicts

Captures are in `captures/`. A scroll capture lists each line twice, once with the `›` cursor, because the capture
tool (the lane's `drv.sh scroll`) keeps the union of the screens.

| # | Criterion (sprint record) | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | **Parent/child journey, OpenAI parent.** One aggregate; each request counted once; subscription work kept apart from spend. | **PASS** | **Setup:** a `gpt-5.4` parent (`h1`) spawned `glm-5.3-flash`/`zai`, `k3`/`kimi-code` and `claude-opus-5-5-plan`/`claude-plan` children, which answered ZED, KIMI and CLAUDE (`h1-02`). **`/cost`:** 11 attempts, root 7 and descendants 4. OpenAI 7 requests $0.102580; Z.AI 1 request $0.001132; pay-per-use total $0.103711, exact USD 0.10371118. Kimi (2 requests, 8,331 tokens) and Claude (1 request, 15,126 tokens) are "Covered by your subscription". Claude's "$0.121088 at API prices" is labelled as such, not as spent (`h1-03`). **Recompute:** 0.10371118 from the raw events (`data/recompute-all.txt`, h1 rows up to 05:38:21Z). The children's thread ids match the spawned ids (`h1-04` to `h1-06`). |
| 1b | **Parent/child journey, GLM 5.3 Flash parent.** | **PASS** | **Setup:** a `glm-5.3-flash` parent on `zai` (`h2`) with Kimi and Claude children. **`/cost`:** 7 attempts, root 4 and descendants 3. Z.AI 4 requests, 34,660 tokens, $0.003003, exact 0.00300274. Kimi 5,720 tokens and Claude 11,051 tokens are subscription (`h2-02`). **Recompute:** 0.00300274 exact. The tokens match per provider. |
| 2 | **Gap (i): `/side`** is recorded under the conversation it belongs to, labelled `side:`. | **PASS** | **Run:** `/side Reply with the single word SIDE.` answered SIDE, then Ctrl-C returned to the main thread (`h1-07`, `h1-08`). **`/cost`:** requests 1–11 unchanged; the parent gained requests 12 and 13 (`h1-09`). **Their pages:** `Turn: side:prewarm:…` ($0.0241225, 9,649 input) and `Turn: side:01a12457-…` ($0.0358105), both on `Thread: 01a12451-0c3e-…`. That is the parent's own id, as printed by `corbanu resume 01a12451-0c3e-…` on exit (`h1-10`, `h1-15`). **Recompute:** both exact. |
| 3 | **Gap (i): memory consolidation** is recorded under the conversation that started it, labelled `consolidation:`. | **PASS** | **Setup:** `h4` on Kimi Code `k3` with `features.memories = true`. Session A's first turn started the consolidation agent. **`/cost` in session A:** 4 requests, 22,854+ tokens (`h4-05`). Requests 2 and 4 read `Thread: 01a12468-99a5-…` (session A) and `Turn: consolidation:01a12468-a6f9-…`, "Billing basis: subscription" (`h4-06`). **Tokens:** request 2's 14,740 in + 206 out equals the provider's own usage event. Request 4 was cut off when I exited A; it shows "Input: unknown", not zero. **Request 3** is an `assess:` request; its usage was also cut off by the exit (`obs=none`). **Kept apart:** session B's 2 requests (8,304 tokens) appear only under "Other conversations". **Limit:** two consolidation requests were started and both are recorded (`data/ledger-dumps.txt` h4); only one completed. This shows attribution and labelling. It does not show completeness over a long consolidation; the lane's run had 6 requests. |
| 4 | **Gap (i): `exec --ephemeral`** stays excluded, with one log warning. | **PASS** | **Run:** `corbanu exec --ephemeral` on Z.AI (`h5`, e1) answered EPH. The provider reported 6,016 tokens. **Log:** exactly one `WARN codex_core::accounting: … not recorded accounting.excluded="ephemeral_session"`. **Ledger:** the home had no accounting table at all (`data/ephemeral.txt`). **Control:** the same command without `--ephemeral` (e2) created exactly one ledger row, with no exclusion warning (`data/ledger-dumps.txt` h5). |
| 5 | **Gap (ii): a request with a billing basis but no price** shows plainly that it has no price and records only its basis. | **PASS** | **Kimi subscription request** (`h1-04`): "Price: none recorded — this request records only its billing basis (subscription)". **Same line on:** an unpriced pay-per-use request (`h1-20`, Fast tier); a custom route with a declared basis (`h3-03` and `h3-04`); a consolidation request (`h4-06`). **Absent there:** 0 lines matching "Price ID" or "Price source" in those six captures. **Follow-up [#396]:** the unpriced Fast-tier page still states "Cache write cost: $0.000000; exact USD 0" for a reported 0 tokens. It is the only money figure on that page; display only, no total affected. **Priced requests keep the full block:** the Claude plan-rate record (`h1-05`) and Z.AI (`h1-06`) each have a Price ID, source and the four rates. The line appears once per page; the second hit in each file is the duplicated cursor line. |
| 6 | **Cancellation:** no invented zero, no double count. | **PASS** | **Run:** Esc during a streamed reply gave "Conversation interrupted" (`h1-11`, `h1-12`). **`/cost`:** request 14 is "usage incomplete". The OpenAI line reads "131,867+ tokens … at least $0.162512 (1 attempt had incomplete usage)", and the next step is to check OpenAI's bill. The known subtotal is unchanged at 0.16364418 (`h1-13`). **Its page:** "Estimated cost: not available — usage incomplete", "Input: unknown", "Tokens: tokens not reported" (`h1-14`). **Raw events:** at that point 13 response ids had been created and 12 completed. The cancelled one never reported usage, so "unknown" is correct. Over the whole `h1` log it is 15 and 14, with the same single id missing ([data/provider-usage-raw.jsonl](data/provider-usage-raw.jsonl), last `h1` line). |
| 7 | **Missing price:** "no price", never a Standard-rate guess or a zero. | **PASS** | **Run:** relaunched with `-c service_tier="priority"`; the served tier was `priority` on both responses. **`/cost`:** requests 18 and 19 show "no price available". The total reads "at least $0.221053 … (2 attempts had no price, 1 had incomplete usage)" and names OpenAI's bill. The known subtotal stays 0.22105268 (`h1-19`). **Recompute at the published Fast rates:** $0.123971. Nothing is shown for these requests, rather than a wrong figure. |
| 8 | **Duplicate retry:** one logical request, its attempts linked, tokens counted once. | **PASS** | **Setup:** a local pass-through proxy returned 503 to the first POST ([data/retry-proxy.jsonl](data/retry-proxy.jsonl)); the custom route `zai-retry` has `billing = "pay_per_use"`. **`/cost`:** 1 request, root's own attempts 2, "5,572+ tokens" (`h3-02`). **Attempt 2:** "Retry predecessor: cc527f8d-…", which is attempt 1's id, with 5,538 in and 34 out. The proxy logged the same 5,538 / 34 (`h3-03`). **Attempt 1:** "Tokens: tokens not reported", "Input: unknown" (`h3-04`). **Limit:** the custom route has no price, so this checks token dedup and the attempt link, not money dedup. |
| 9 | **Reopen (`/resume`):** the same totals; nothing recounted. | **PASS** (observation 1) | **OpenAI:** after `/exit` and `resume 01a12451-…`, requests 1–14 are identical. The reopen added one paid prewarm, request 15 at $0.025365 (10,146 input), recorded as a new request (`h1-16`); exact 0.18900918. **Z.AI:** `h2` before and after resume is byte-identical over the provider lines, the 7 request rows and the exact subtotal (`h2-02` vs `h2-04`, diffed). |
| 10 | **Kill -9 and restart:** totals and attribution agree, with no manual edit. | **PASS** | **Kill:** SIGKILL (exit 137) right after the answer "KILLED" (`h1-17`). **Restart:** after `resume`, requests 1–15 are identical (diffed). The killed turn's own request 16 ($0.006678) was recorded, plus the restart's prewarm, request 17 ($0.025365). **Recompute:** exact USD 0.22105268, the same as the raw events (`h1-18`). |
| 11 | **Historical inspection:** past days are read from the ledger. | **PASS** | **Seed:** synthetic history over 14 days (`accounting_demo_seed <home> history <thread>`, 32 rows). **Days:** `/cost 2026-10-07`, `2026-09-28` and `2026-10-09` show 0.15569982, 0.161997 and 0.03180032 (`h2-05-*`). That is exactly the per-day sums computed from the ledger rows' tokens and bound rates ([tools/day_totals.py](tools/day_totals.py), `data/day-totals.txt`); the sums also equal the lane's golden file. **Live day:** today's live rows were unchanged by the seeding (criterion 9). |
| 12 | **`/cost` date bounds and refusal reasons.** | **PASS** | **Valid ranges:** `2026-10-05 2026-10-11 day` $0.291501; `2026-09-28 2026-10-12 week` $0.920787; `2026-09-01 2026-11-01 month` $1.001880. Range views show only rounded figures; each matches the ledger sums (0.2915007, 0.92078666, 1.00188048) at the displayed rounding. The bucket containing today reads "In progress — totals so far" (`h2-06` to `h2-08`). **Refused:** "2026-10-12 is after today"; "Range refused: reversed range: end precedes start"; "Range refused: future start is unavailable"; a bad grouping gives "grouping must be hour, day, week or month". `2026-13-40` and `yesterday` show the usage line. **Before retention:** a 2025 range gives "Range total unavailable — partial or unavailable buckets excluded; no partial total" (`h2-09` to `h2-15`). `/cost help` prints the usage (`h1-01`). |
| 13 | **Every pay-per-use total recomputes** from provider-reported usage at published prices. | **PASS** | [tools/recompute_ia.py](tools/recompute_ia.py) reads the raw provider events from `codex-tui.log` (WebSocket frames for OpenAI, SSE for Z.AI, Kimi and Claude), keyed by response id. Prices: [data/published-prices-20261010.txt](data/published-prices-20261010.txt). Results are in the Reconciliation table below. |
| 14 | **(iii) Waiver (a) re-check:** a real OpenAI API-key request on the final binary, recomputed at published prices. | **PASS** | **`gpt-5.4`, Standard tier:** 12 real requests (8 turns and 4 WebSocket prewarms) recompute at $2.50 / $0.25 / $15 to 0.2198695. That equals `/cost`'s OpenAI share (0.22105268 − 0.00113168 Z.AI), and each request matches on its own. **The two models S05 failed on (#361), `h6`:** 3 `gpt-5.6-luna` and 2 `gpt-5.6-terra` requests show exact USD 0.06130286 (`h6-03`). The recompute at OpenAI's published Standard rates, including cache writes (luna $0.20 / $0.02 / $0.25 write / $1.20; terra $2 / $0.20 / $2.50 write / $12), is 0.06130286, so each request is exact. **Terra's page** lists exactly those four rates, plus the long-context band (`h6-04`). **Fast tier:** shows "no price available", not a mis-estimate. **Prices:** fetched from developers.openai.com today ([data/openai-pricing-20261010.txt](data/openai-pricing-20261010.txt)). |
| 15 | **(iv) #368 price follow-ups** documented as limitations only. | **PASS** (document check) | The qualification's "Known limitations" lists OpenRouter per-endpoint pricing, the Moonshot/BigModel missing rows, `gpt-6.1-sol`, the GPT-5.6 Sol promotion date and DeepSeek holidays. A code-blind run can't prove that no price changed. The catalogue rates the request pages show (`gpt-5.4`, `gpt-5.6-luna`/`-terra`, `glm-5.3-flash`) each equal the provider's published page today. |
| 16 | **Subscription work is never counted as spent.** | **PASS** | In every view, Kimi Code and Claude Plan are "Covered by your subscription" and appear under "Subscription work — same work at API prices", never in the pay-per-use total (`h1-03`, `h2-02`, `h4-05`). |
| 17 | **Focused tmux suite** on the candidate. | **PASS** | `CORBANU_TMUX_REQUIRED=1 python3 ../scripts/isolated_rust_tests.py -p codex-tui --test all tmux --retries 0`. That is the `just test` recipe; `just` is not installed on the RTX box. Result: 71 run, 71 passed, 24 skipped, exit 0 ([data/tmux-suite.txt](data/tmux-suite.txt)). |
| 18 | **Basis per route** (qualification flow 12). | **PASS** | **Subscription:** `kimi-code` and `claude-plan`. **Pay per use:** `openai` with an API key and `zai` (`h1-03` to `h1-06`, `h2-02`, `h6-03`). **Declared:** a custom route with `billing = "pay_per_use"` reads "pay per use (set in your config)" (`h3-03`). **Not re-run:** DeepSeek V4 Flash and GLM 5.2. Per the 2026-10-10 owner amendment, GLM 5.3 Flash was used, so the qualification's DeepSeek and GLM 5.2 recomputes were not independently re-confirmed. |
| — | ChatGPT-login routes (AC10/AC12 ChatGPT paths) | **NOT VERIFIABLE** | Travis won't swap his ChatGPT account login (unchanged from S05). |
| — | Image generation, realtime | **NOT VERIFIABLE** | **Image generation:** no image tool is offered on the API-key route. That is S05's finding, not re-checked here. **Realtime:** voice-only and under development; it can't be driven in tmux. |
| — | TensorCash live repository | **NOT VERIFIABLE** | `gh repo view agtico/tensorcash` returns "Could not resolve to a Repository", as in the lane run. Isometric Game is not applicable, because accounting doesn't change repository work. |
| — | Guardian review or sub-agents started from a `/side` conversation | Not tested | The sprint carries it as a known limitation (still excluded), not as a criterion. |

## Reconciliation

| Run | `/cost` exact | Independent | Match |
| --- | --- | --- | --- |
| `h1` after the journey: OpenAI 7 + Z.AI 1 | 0.10371118 | 0.1025795 + 0.00113168 = 0.10371118 | exact |
| `h1` after `/side` (+2 requests) | 0.16364418 | +0.0241225 +0.0358105 | exact |
| `h1` after reopen, kill and restart: OpenAI 12 Standard (4 prewarms at tier `auto`, costed as Standard) + Z.AI 1 | 0.22105268 | 0.1002175 + 0.1197035 + 0.00113168 | exact |
| `h1` cancelled request | usage incomplete | no `response.completed` (13 created, 12 completed) | consistent |
| `h1` Fast tier, 2 requests | no price available | $0.123971 at $5 / $0.50 / $30 | no figure shown (correct) |
| `h6` `gpt-5.6-luna` 3 + `gpt-5.6-terra` 2 (27,381 cache-write tokens) | 0.06130286 | 0.0021104 + 0.00341855 + 0.00028441 + 0.021104 + 0.0343855 | exact |
| `h2` Z.AI GLM 5.3 Flash, 4 requests | 0.00300274 | 0.00300274 at $0.15 / $0.03 / $0.50 | exact |
| `h3` custom `zai-retry` | no price (custom route) | 5,538 / 34 tokens = the proxy log | tokens exact |
| Kimi Code tokens | 8,331 (`h1`); 5,720 (`h2`); 22,854+ (`h4`) | 7,895 + 436; 5,300 + 420; 7,908 + 14,946 + 2 cut-off | exact |
| Claude Plan tokens and API equivalent | 15,126, $0.121088 (`h1`); 11,051, $0.048146 (`h2`) | 4 + 15,114 write + 8 out at the page's $4 / $8 write / $20 → 0.121088; 4 + 5,172 read + 5,867 write + 8 → 0.0481464 | exact (not spent) |
| Seeded history and ranges | see criteria 11 and 12 | `data/day-totals.txt` | exact |

## Observations (not verdicts)

1. **Every reopen of an OpenAI conversation costs a paid prewarm,** about $0.025 on `gpt-5.4`. That covers `/resume`,
   restart and `/side`; each one is recorded honestly (`prewarm:`, `side:prewarm:`). Same as the lane's observation 1.
2. **Wording nit, still present:** with a declared basis the gap (ii) line reads "(pay per use (set in your config))"
   (`h3-03`), the lane's observation 4.
3. **Filed as [#396]:** on the unpriced Fast-tier page, "Cache write cost: $0.000000; exact USD 0" is shown for a
   reported 0 tokens, while the other components say "unknown — rate unavailable" (`h1-20`). The issue also covers
   the nested parentheses in item 2.
4. **Memory consolidation starts at the first turn of the first session with memories on,** not on a later startup.
   Exiting quickly leaves its later requests with unknown usage, shown as such.
5. **Driver slip.** In `h6` a stray "/model" plus "/exit" was sent as one message, "/model/exit". The model answered
   it, which made `gpt-5.6-luna` request 3 ($0.000284). It is recorded and recomputed like any other request.
6. **GPT-5.6 runs without `codex-code-mode-host`.** The TUI warned "Code Mode is unavailable … host executable was not
   found". Plain text replies were unaffected; I did not build the host.
7. **`/cost` covers only the open conversation.** Other conversations appear only as a one-line summary, as designed.
   To see consolidation under session A, I had to resume A.

## Safety

- **Every built-binary run was isolated.** Each had `CORBANU_TEST_NO_NATIVE_KEYRING=1` and a disposable home; all ran
  on Linux.
- **How keys were resolved.** Only the installed signed wrapper resolved keys, without the test variables:
  `V(){ env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1" < /dev/null; }`.
  Each value was substituted only on the consuming ssh command.
- **Tracing was limited to frames.** `RUST_LOG=codex_api=trace,tungstenite::protocol=trace,…`, never handshake or
  HTTP traces, to avoid S05's bearer-in-log incident.
- **Key scan.** [tools/keyscan.sh](tools/keyscan.sh) checks all four vault values. It aborts if a value is empty, so a
  missing value can't produce a false 0.
  - **Where:** the remote homes, logs, captures and scratch workspace, and this directory.
  - **Values:** 0 files everywhere.
  - **Key-shaped strings:** 0 in this directory. Remotely, 96 files, all bundled plugin documentation under each
    home's `.tmp/plugins/`; 0 in logs, captures and the workspace ([data/key-scan.txt](data/key-scan.txt)).
- **Redaction.** Host, user and home paths are replaced with `<rtx-home>`, `<rtx-user>@<rtx-host>` and `<mac-home>`.
- **Spend.**
  - **OpenAI:** about $0.42 at list prices. That is $0.221 `gpt-5.4` Standard, $0.124 Fast, $0.061 luna and terra,
    and the cancelled partial, estimated under $0.02.
  - **Z.AI:** about $0.006.
  - **Kimi Code and Claude:** ran on their subscriptions.

## Method

- **Isolation deviation.** As in S05 (waiver (b)), I worked under self-discipline on the hosts. I was not inside an
  OS-enforced isolation gate, and no negative access probes are recorded. The lane's qualification likewise didn't use
  the isolated gate. Travis should accept this run only with that deviation stated: waive the enforced-isolation
  requirement again, or order an isolated re-run. The verdicts themselves rest on captures and provider data, not on
  the boundary.
- **Code-blind boundary.**
  - **Read:** the S04 sprint record, `qualification.md` and `pf-60-s04/` evidence, the demo index, and the S03 and S05
    acceptance READMEs and tools. Also user docs (`docs/tmuxHarness.md`, `docs/integrations/zai-glm-52.md`,
    `docs/features/context-tools.md`), `/cost help`, the security worker rules (`.codex-work/workers-20261002/sec-common.md`), the `justfile` test recipe,
    and `git log` titles. The run had the repository checked out, because I built the binary myself from `git archive`.
  - **Deviation:** to find the memories idle-time setting, I read the `memories` property descriptions in the
    generated `codex-rs/core/config.schema.json`. That is the config reference, not implementation. The setting
    (`min_rollout_idle_hours = 0`, added to `h4` at 06:07Z) turned out not to matter: the consolidation had already run
    at 06:03Z.
  - **Not read:** implementation source, PR diffs, lane briefs, worker logs or review files.
- **Independent channel.** Raw provider events traced before parsing were extracted per response id and recomputed
  with Decimal at published prices. The ledger dump ([tools/ledger_dump.py](tools/ledger_dump.py), the lane's
  read-only tool) and [tools/day_totals.py](tools/day_totals.py) are storage cross-checks. Only the historical days,
  being synthetic, rest on ledger rows.
- **Tools reused.** From the lane's `pf-60-s04/tools`: `drv.sh`, `costcmd`, `tech`, `attempt` and `retry_proxy.py`,
  with only paths and socket names changed. `q`, `start.sh`, `closepop`, `ephem.sh`, `recompute_ia.py`,
  `day_totals.py` and `keyscan.sh` are new.
- **Launch commands** are in [data/launch-commands.txt](data/launch-commands.txt).

## For Travis

- **Accept PF-60-S04.** Every flow the sprint names passed again on current main, in an independent run, with
  real keys:
  - parent and child journeys on four providers;
  - `/side` and consolidation recorded where they belong, and the ephemeral exclusion;
  - the single "Price: none recorded" line;
  - cancellation, missing price, duplicate retry, reopen, kill -9 and history;
  - date bounds.

  Every pay-per-use figure recomputed exactly.

- **Accept with one stated deviation:** no enforced isolation gate (see Method), as S05's waiver (b) did.
- **Lift waiver (a) for estimate correctness.**
  - **Verified:** OpenAI API-key estimates are exact to the micro-dollar for `gpt-5.4`, `gpt-5.6-luna` and
    `gpt-5.6-terra`, Standard tier, short context, cache writes included. Luna and terra are the two models S05 failed
    on.
  - **Never wrong:** a tier or model the catalogue doesn't price (Fast here) shows "no price available", never a
    wrong figure.
  - **Not exercised:** the long-context band, Batch/Flex and other OpenAI models.
  - **Remaining limitation:** "no price" coverage (unpriced tiers and models) is a separate coverage limit, not a
    mismatch. Of the unpriced models, only `gpt-6.1-sol` is a listed #368 item.
- **Display follow-up:** [#396] (cosmetic, no total affected).
- **Still NOT VERIFIABLE:** ChatGPT login, image generation, realtime and TensorCash.

## Review corrections applied

Applied from [REVIEW.md](REVIEW.md). The reviewer recomputed every headline number and they all matched.

- **Isolation:** the self-enforced code-blind boundary is now stated as a deviation for Travis to waive.
- **Waiver (a):** the scope is narrowed to estimate correctness for the three models on the Standard tier. "No price"
  coverage is now described as separate from #368.
- **Consolidation:** the `assess:` request and the started-versus-recorded limit are now explained.
- **Ranges:** they match at the displayed rounding; "equals" is gone.
- **Cache-write tokens:** 27,381, not 27,383.
- **Audit trail:**
  - `data/out-of-band-steps.txt` lists the build, seeding, proxy, config edits and extraction steps, the
    `git diff --stat` result and each home's config;
  - `data/provider-usage-raw.jsonl` archives the verbatim provider usage objects and the created/completed counts.
- **Key scan:** it now aborts on an empty value, covers the scratch workspace and scans remotely for key-shaped
  strings.
- **Display defect:** the "$0.000000" cache-write line is filed as [#396].
- **Retry:** the verdict notes that money dedup wasn't exercised.
- **Basis per route:** a verdict is added (row 18), noting that DeepSeek and GLM 5.2 were not re-run.
- **TensorCash:** the access probe is recorded. Image generation is marked as relying on S05.
- **Nits:** homes now say `h1` to `h6`; "six captures"; `sec-common.md` is identified; the binary is named as this
  run's own build.

[#396]: https://github.com/CorbanuCore/CorbanuTerminal/issues/396
