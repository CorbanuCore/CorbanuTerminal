# PF-60 cost accounting — qualification on one recorded binary (PF-60-S04)

**Result: every cost flow passes on one recorded binary.**

- **Unknown and estimated values stay distinct.** A missing price, incomplete usage, a cancelled request and an
  injected failure each show as "no price", "usage incomplete" or "at least …", never as a zero.
- **Subscription work is never counted as spent.**
- **Totals agree after a reopen and after a kill -9.**
- **Every pay-per-use figure matches an independent recompute to the micro-dollar.** That covers OpenAI `gpt-5.4`
  on an API key, Z.AI GLM 5.2 and DeepSeek V4 Flash.
- **Both S05 hand-overs are closed:**
  - `/side` conversations and memory consolidation are now recorded under their conversation;
  - a request with no price no longer shows a "Price source".
- **Waiver (a) can be lifted.** The OpenAI API-key estimate is now exact.

This is the lane's own qualification. The independent code-blind acceptance and Travis's named acceptance come next.

## Candidate and inputs

| Item | Value |
| --- | --- |
| Commit | `1e9cd464b978d38aa1e2c5d99be32a8dd318f92b`, main after #382, #383 (gap i), #381 (env credentials) and #385 (gap ii) |
| Binary | `corbanu 0.1.48`, Linux x86_64 debug build, `cargo build --locked -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting`; sha256 `cb378c27331fa0fec973af3feffda251c061c20227887dec7574492fb37a1d58` |
| Host | RTX box (Ubuntu, kernel 7.0.0-38). Linux was chosen so that nothing could reach the macOS login keychain. |
| Homes | Disposable `q1` to `q4`, each with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and a trusted scratch git workspace |
| Credentials | Vault labels `openai-api-key` (as `OPENAI_API_KEY`, now honoured after #381), `provider/zai_api_key`, `provider/kimi_api_key`, `claude-plan-test-token` (as `CLAUDE_CODE_OAUTH_TOKEN`) and `provider/deepseek_api_key` |
| How keys were passed | The installed `corbanu vault auth-helper` resolved each key locally and piped it over ssh stdin into a private tmux server's environment. No key was printed or written. |
| Spend | About $0.5 on the OpenAI key (the run plus a smoke check), $0.03 on Z.AI and under $0.01 on DeepSeek, at list prices. Kimi Code and Claude ran on their subscriptions. |
| Driver | Private `tmux -L` servers with real keys, text and Enter sent separately ([tools](pf-60-s04/tools/)). Every command is in [keys-sent.log](pf-60-s04/data/keys-sent.log). |

## Flows: expected and actual

Captures are under [`pf-60-s04/captures/`](pf-60-s04/captures/); ledger rows are in
[ledger-dumps.txt](pf-60-s04/data/ledger-dumps.txt).

| # | Flow | Expected | Actual | Verdict |
| --- | --- | --- | --- | --- |
| 1 | **Three providers, parent and children** (`q1`) | One aggregate: the root's own requests plus each child's, each counted once; subscription work kept apart from money spent. | **Setup:** an OpenAI `gpt-5.4` parent spawned children on Kimi Code `k3`, Claude Plan Opus 5.5 and DeepSeek V4 Flash. **`/cost`:** 11 attempts (root 7, descendants 4). OpenAI 7 requests, $0.106906, and DeepSeek 1 request, $0.001390, make the pay-per-use total of $0.108297. Kimi (2 requests) and Claude (1 request) are "covered by your subscription". Claude's API equivalent ($0.032886) is labelled as such, not as spent. Exact USD 0.1082967 (`q1-02-cost`). | PASS |
| 1b | **GLM 5.2 parent** (`q3`) | As 1, on Z.AI. | A GLM 5.2 parent on `zai` with Kimi and Claude children: 8 attempts (root 5, descendants 3). Z.AI 5 requests, $0.031737, exact 0.03173724. Kimi and Claude are subscription (`q3-04-cost`). | PASS |
| 2 | **`/side`** (gap i) | The side conversation's requests are recorded under its conversation. | After `/side`, the parent gained 2 requests: `Turn: side:prewarm:…` ($0.0241225) and `Turn: side:…` ($0.0360855), both on the parent's thread (`q1-12`, `q1-13`). Before S04 they weren't recorded. | PASS |
| 3 | **Memory consolidation** (gap i) | The consolidation agent's requests are recorded under the conversation that started it. | With `features.memories` on (`q4`), the consolidation agent's 6 requests (89k tokens of Kimi Code subscription) appear in that conversation's `/cost` as `Turn: consolidation:…` (`q4-02`, `q4-03`). | PASS |
| 4 | **Request with no price** (gap ii) | One plain line; no Price ID or Price source. | A Kimi request's technical page reads "Billing basis: subscription" and "Price: none recorded — this request records only its billing basis (subscription)", with no Price ID, source or rate lines (`q1-05`). Priced requests keep the full block (`q1-06` Claude plan, `q1-07` DeepSeek). | PASS |
| 5 | **Cancellation** | No invented zero, no double charge. | Esc mid-stream gives "Conversation interrupted". The request shows "usage incomplete" and the total becomes "at least $0.239622 (1 attempt had incomplete usage)", with the next step to check OpenAI's bill. Its page reads "Input: unknown", "Known subtotal: none — usage incomplete" (`q1-16` to `q1-18`). | PASS |
| 6 | **Missing price** | "No price", never a Standard-rate guess or a zero. | Two `gpt-5.4` requests on the priority tier show "no price available". The total says "2 attempts had no price" and names OpenAI's bill. The known subtotal is unchanged at 0.2720492 (`q1-21`). | PASS |
| 7 | **Duplicate retry** | One logical request, its attempts linked, tokens counted once. | A local proxy returned 503 to the first POST ([retry-proxy.jsonl](pf-60-s04/data/retry-proxy.jsonl)). `/cost` shows 1 request with 2 attempts. Attempt 2 has "Retry predecessor" set to attempt 1. Tokens are 6,015+: the 5,889 + 126 of the served retry, with the failed attempt's usage unknown, not zero (`q2-02` to `q2-05`). | PASS |
| 8 | **Reopen** (`/resume`) | The same totals; nothing recounted. | Requests 1–11 are unchanged after the reopen. The reopen adds one paid startup prewarm (`Turn: prewarm:…`, $0.025365), recorded as a new request (`q1-02` vs `q1-04`, `q1-08`). | PASS (see observation 1) |
| 9 | **Kill -9 and restart** | Totals and attribution agree, with no manual database edit. | **Kill:** SIGKILL (exit 137) right after the answer "KILLED". **Restart:** `resume <thread>` shows requests 1–16 identical (diffed). The killed turn's own request was recorded ($0.005672), plus the restart's prewarm. Before plus new: $0.239622 + $0.005672 + $0.025365 = $0.270659, as shown (`q1-17`, `q1-20`). | PASS |
| 10 | **Historical inspection** | Past days read from the ledger. | 14 earlier days were seeded with synthetic GLM 5.2 and GPT-5.4 requests (`accounting_demo_seed history`). `/cost 2026-10-07`, `2026-09-28` and `2026-10-09` each show that day's providers and an exact subtotal (`q3-05`, `q3-14`, `q3-15`). | PASS |
| 11 | **`/cost` date bounds** | Ranges group correctly; invalid input is refused with its reason. | **Valid ranges:** `day`, `week` and `month` ranges open, and a range that includes today says "In progress — totals so far". **Refused or unavailable:** a future day is "after today"; a reversed range is "Range refused: reversed range"; a future start is "future start is unavailable"; a malformed date shows the usage line; a range before the aggregate retention shows "Range total unavailable … no partial total" (`q3-06` to `q3-13`). | PASS |
| 12 | **Basis per route** | Subscription, pay-per-use and user-declared bases as declared. | `kimi-code` and `claude-plan` are subscription. `openai` with an API key, `zai` and `deepseek` are pay per use. A custom route with `billing = "pay_per_use"` reads "pay per use (set in your config)" (`q2-04`). | PASS |

## Reconciliation against golden totals

| Check | `/cost` | Independent | Match |
| --- | --- | --- | --- |
| OpenAI `gpt-5.4`, standard tier, `q1` (13 requests, including two reopen prewarms and the side requests) | $0.270659 | $0.2706590, from provider-reported usage (WebSocket trace) at the published $2.50 / $0.25 / $15 ([recompute](pf-60-s04/data/recompute-q1-gpt54.txt)) | exact |
| OpenAI, flow 1 only (7 requests) | 0.1069065 (0.1082967 − DeepSeek) | 0.1069065 | exact |
| Z.AI GLM 5.2, `q3` (5 requests) | 0.03173724 | 0.03173724 at $1.40 / $0.26 / $4.40 ([recompute](pf-60-s04/data/recompute-q3-glm.txt)) | exact |
| DeepSeek V4 Flash, `q1` (Saturday, off peak) | 0.0013902 | 9,220 × $0.15 + 12 × $0.60 per million = 0.0013902 | exact |
| Kimi Code tokens | 8,353 (`q1`); 5,563 (`q3`) | 7,855 + 59 + 346 + 93; 5,120 + 22 + 346 + 75 | exact |
| Claude Plan tokens and API equivalent | 15,126 tokens, $0.032886 (`q1`); 10,884 tokens, $0.046810 (`q3`) | 4 + 11,308 read + 3,806 write + 8 out at the recorded $4 / $0.20 / $8 / $20 → $0.0328856; 10,884 → $0.0468104 | exact (not spent) |
| Seeded history, per day | 2026-10-07 0.15569982; 2026-09-28 0.161997; 2026-10-09 0.03180032 | [golden-history.txt](pf-60-s04/data/golden-history.txt), recomputed from the seed's inputs and prices | exact |
| Range 2026-09-26 to 2026-10-11 | $1.030615 (rounded) | 0.99887774 seeded + 0.03173724 live = 1.03061498 | matches the rounding |
| Contract goldens | — | `python3 qa/portfolio/agent-cost-accounting/pf-60-s01/test_contract.py`: 48 OK. `just test -p codex-state -E 'test(golden) \| test(accounting_contract)'`: 6/6, including raw replay and two reopens | pass |

**AC10 waiver (a) re-check.** S05's waiver (a) covered the OpenAI API-key estimate until #361 was fixed. #369 fixed
it. On this binary, 13 real `gpt-5.4` requests recompute to the micro-dollar. **Recommendation: lift waiver (a).**

## Verification commands

- [x] From `codex-rs`: `CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all tmux --retries 0`, on the
  candidate commit (RTX). `just` isn't installed there, so its recipe for `test`,
  `python3 ../scripts/isolated_rust_tests.py`, ran with the same arguments. Result: 71 run, 71 passed, 24 skipped, exit 0 ([log summary](pf-60-s04/data/tmux-suite.txt)).
- [x] From the repo root: `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`, all clean
  on the evidence branch.
- [x] Slice gates: see [#383](https://github.com/CorbanuCore/CorbanuTerminal/pull/383) and
  [#385](https://github.com/CorbanuCore/CorbanuTerminal/pull/385). Each has focused tests with and without
  `developer-accounting`, Linux clippy `-D warnings` and Opus 5.5 High review rounds ending in APPROVE.
- [x] Key scan of every qualification home, capture and log for all five vault values. **Results:**
  - **Captures, logs and qualification homes:** 0 hits.
  - **One expected hit:** an `auth.json` in a pre-qualification smoke home where an `--with-api-key` login was tried
    before #381 landed. It was deleted with the scratch.
  - **Key-shaped strings:** the matches are base64 fragments inside encrypted reasoning blobs and plugin
    documentation, not credentials.

## Counterexamples and observations

1. **Every reopen of an OpenAI conversation costs a startup prewarm.** `/resume`, restart and the fork made by Esc Esc
   each sent a paid prewarm (~10k input tokens, $0.025 on `gpt-5.4`). It is recorded honestly, as a `prewarm:`
   request, but users may not expect a charge for reopening.
2. **A cancelled request shows "usage incomplete" and its cost is unknown.** That is correct: OpenAI may still bill
   the partial output, and the next step names the bill.
3. **Esc Esc on an idle composer edits the previous message and forks the conversation.** This happened once during
   the run, from a driver keystroke. The fork was a separate conversation, and its prewarm appears under "other
   conversations", not in the parent's totals. That is correct attribution, but a surprising control.
4. **Wording nit (gap ii line):** with a user-declared basis the line reads "… its billing basis (pay per use (set in
   your config))", with nested parentheses (`q2-04`).
5. **Wording nit (existing):** on subscription requests with no price, the per-bucket lines still read "unknown —
   rate unavailable" or "no retained numeric evidence" (#352).
6. **Denied provider (supplemental, interim build 590e0b9, not the recorded binary):** while Z.AI returned 429 / code
   1113 ("insufficient balance"), `/cost` showed the two denied turns as "Pay per use, 2 requests, tokens not
   reported, usage incomplete", with no zero, and 10 attempts from retries under 2 logical requests
   ([denied-zai-1113-cost.txt](pf-60-s04/captures/denied-zai-1113-cost.txt)). Z.AI was serving again when the recorded
   binary ran.

## Known limitations (carried, not discharged)

- **Still unrecorded:** a guardian review of a `/side` conversation, and sub-agents started from one, are still
  excluded. Their parent, the side conversation, isn't persisted. Nothing is misattributed.
- **`side:` label scope:** it labels every ephemeral fork of a persisted conversation, not only TUI `/side`.
- **Price follow-ups (#368), documented only:** this sprint excludes price changes.
  - OpenRouter is priced per upstream endpoint.
  - Moonshot/Kimi Open Platform and BigModel have no catalogue rows ("no price").
  - `gpt-6.1-sol` is missing from the catalogue.
  - The GPT-5.6 Sol promotion ends 2026-11-21 (re-read the price before then).
  - DeepSeek holidays after 2026-10-07 aren't in the catalogue.
- **Unchanged from S05:** ChatGPT-login paths, image generation and realtime remain NOT VERIFIABLE. The isolated
  execution gate was not used for this lane run.
- **Live repositories:**
  - TensorCash was not reachable: `agtico/tensorcash` resolves to no repository for this account, and the RTX box has
    no credentials to clone it.
  - Isometric Game is not applicable, because accounting doesn't change repository work.
  - The journeys ran in a disposable git workspace.
- **Collection is still developer-only** (`developer-accounting`, debug builds). Shipping `/cost` to users is not
  authorised.

## Handoff

- **Changed scope:** `Session::accounting_owner` now attributes side conversations (`side:`) and memory consolidation
  (`consolidation:`), alongside guardian reviews (`review:`). The `/cost` request detail no longer shows price
  metadata for basis-only records. No schema, ledger format or price changed.
- **Contracts:** the ledger format is unchanged (formats 1–3 as before). Turn labels gain the `side:` and
  `consolidation:` prefixes.
- **Next:**
  1. The independent code-blind acceptance of this record.
  2. Travis's named acceptance, with waiver (a) lifted if he agrees.
  3. Then archive S04 and close PF-60's sprint map.
- **Combined-tree evidence needed at release:** the release record must rerun the slice tests and the tmux suite on
  the release candidate. This record qualifies `1e9cd464b9` only.
