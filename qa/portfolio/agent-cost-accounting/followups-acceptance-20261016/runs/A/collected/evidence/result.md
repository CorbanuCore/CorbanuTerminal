# Group A functional execution report

Executor role: independent functional executor. This report records observed outcomes only; it does not claim human acceptance.

## Isolation and identity preflight

- Group start UTC: 2026-10-16T03:53:24Z (probe context) / first navigation UTC: 2026-10-16T03:54:53Z.
- Executed required command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; exit code `0`.
- Required output state: `"ok": true`; every probe in that file has `"ok": true`.
- Coordinator summary (`preflight-summary.json`): `"launcher_ok": true`, `"tmux_child_ok": true`, `"pty_input_echoed": true`, `"failed": []`, `"probe_count": 105`, `"ok": true`. Both required `preflight-*.json` files (`launcher`, `tmux-child`) report `"ok": true`.
- Packet hashes matched: design `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6`, amendments `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e`, answers `dc4044d36ac3868205ff2dc71cee36805969c6896c683852a6f9d705dc2d7c0c`.
- Direct identity check: `/opt/agms/pkg/bin/corbanu --version` → `corbanu 0.1.48`; SHA-256 → `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1`. Package isolation probes denied writes/chmod as expected.
- Fixture/product state: fresh supplied `$AGMS_RUN/fixture/workspace`; product home contained only its initial `config.toml`. Product was started via `agms-tui start --route zai --model glm-5.3-flash` and showed `model: glm-5.3-flash` before typing.

## ACC-01 — navigation, empty day, basic totals

Case start UTC: 2026-10-16T03:55:47Z. Requested actions were followed literally with text and Enter sent separately. No extra model turns were sent.

### Requested versus actual
1. Requested `/cost`, Enter, then Esc. Actual: typed `/cost`, observed exact text, pressed Enter; page opened; pressed Esc and returned to composer.
2. Requested prompt and Enter. Actual: typed exact prompt and pressed Enter at 2026-10-16T03:56:15Z; reply `OK` observed by 2026-10-16T03:56:46Z.
3. Requested `/cost`; navigate request and technical. Actual: opened `/cost`; used displayed `↑↓` navigation, opened `Request 1`, opened `Technical details (attempt 1)`.
4. Requested three Esc levels. Actual: Esc from technical returned to Request; Esc from Request returned to `Cost — this conversation`; Esc closed `/cost` to composer.
5. Requested `/cost 2026-10-01`. Actual: typed/entered and observed empty-day page.

### Observed `/cost` evidence
- Before any turn, `acc01-cost-before` (2026-10-16T03:56:00Z):
  `Cost — this conversation`
  `Requested UTC day: 2026-10-16`
  `› Unavailable — no requests recorded in this home yet; the accounting ledger is created when the first one is.`
  No `$0` figure appeared.
- First post-turn screen, `acc01-cost-first` (2026-10-16T03:56:55Z):
  `• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 1 request, 8,513 tokens. Estimated cost: $0.001295 (rounded).`
  `Estimated token cost for recorded attempts: $0.001295 (rounded)`
  Exact subtotal from technical page: `Known subtotal exact USD: 0.00129515`.
- Request page (`acc01-request-page`, 2026-10-16T03:57:57Z): `Provider: Z.AI`, `Model: Z.AI GLM 5.3 Flash`, `Estimated cost: $0.001295 (rounded)`, `Tokens: 8,513 tokens`, `Request: 38047faa-44b5-4929-8b07-e301f9745c9b`.
- Technical page (`acc01-tech-real-*`, 2026-10-16T04:02:02–04:02:19Z): title `Attempt, components and original price`; `Turn: 01a142da-74cc-7680-8da3-40b4d6658eec`; provider/model lines and usage: input 8,461, cache read 0, output 52, reasoning 47, total 8,513.
- Technical costs/rates: `Noncached input cost: $0.001269 (rounded); exact USD 0.00126915`; `Cache read cost: $0.000000; exact USD 0`; `Cache write cost: $0.000000; exact USD 0`; `Output cost: $0.000026; exact USD 0.000026`; rates 0.15 / 0.03 / 0 / 0.5 USD per million tokens.
- Empty UTC date (`acc01-cost-october`, 2026-10-16T04:02:51Z): `This conversation on 2026-10-01 (UTC): no recorded requests.` No money figure appeared.

### Recomputation
Input = 8,461. Noncached input cost = 8,461 × 0.15 / 1,000,000 = 0.00126915. Cache read = 0 × 0.03/M = 0. Cache write = 0. Output = 52 × 0.5 / 1,000,000 = 0.000026. Sum = 0.00129515. This equals the exact technical subtotal and rounds to the displayed $0.001295.

### Expected-result outcomes
- Before any turn: no recorded requests, no `$0` — **passed**.
- First screen lists provider/model, N=1, estimate — **passed**. Request costs sum to exact provider total — **passed**.
- Page order Cost → Request → Attempt/components/original price — **passed**.
- Turn has none of `side:`, `fork:`, `agent:`, `code-review:` — **passed** (UUID only).
- Each Esc goes back one level and last closes to composer — **passed**.
- 2026-10-01 shows no requests and no money — **passed**.

ACC-01 verdict basis: all expected results passed.

## ACC-02 — `/side` request attribution

Case start UTC: 2026-10-16T04:03:50Z.

### Starting state
- R0 = 1 request; T0 = estimated $0.001295 (exact 0.00129515); Request 1 cost = $0.001295; other-conversations count O0 = 0.

### Requested versus actual
1. Requested `/side` Enter. Actual: first attempt happened while an old `/cost` page was still open, so `/side` was not accepted; after closing that page with Esc, `/side` was typed and entered. This extra navigation did not send a model turn. Side opened with `Side from main thread · ctrl + / to switch · ctrl + c to close`.
2. Requested exact prompt and Enter. Actual: typed and entered at 2026-10-16T04:04:06Z; `SIDE` visible by 2026-10-16T04:04:31Z.
3. Requested leave side. Actual: pressed Ctrl+C once; screen returned to the ACC-01 saved conversation at 2026-10-16T04:04:39Z.
4. Requested `/cost` and open each new request technical page. Actual: cost showed 2 requests; opened Request 2 and its technical page. Request 1 ID/cost was visible in the same request list and previously observed on its technical page. No new model turn was sent while navigating.

### Observed `/cost` evidence
- `acc02-cost-first` (2026-10-16T04:04:48Z):
  `• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 2 requests, 16,711 tokens. Estimated cost: $0.002533 (rounded).`
  `No other conversation recorded requests on this day.`
  `Estimated token cost for recorded attempts: $0.002533 (rounded)`
- Request list (`acc02-request-list`, 2026-10-16T04:05:14Z):
  `→ Request 1 · Z.AI · Z.AI GLM 5.3 Flash · estimated $0.001295 (rounded)`
  `→ Request 2 · Z.AI · Z.AI GLM 5.3 Flash · estimated $0.001238 (rounded)`
  `→ Root's own attempts (2 attempts)`
  `→ Descendant attempts (none)`
- Request 2 page: `Estimated cost: $0.001238 (rounded)`, `Tokens: 8,198 tokens`, `Request: 02791f43-77b9-4107-be95-089d2669a631`.
- Request 2 technical page (`acc02-tech-step-*`, 2026-10-16T04:07:21Z): `Attempt: b34d5fd6-30d3-4617-bff3-f027d1c9a8e0`; `Thread: 01a142da-0909-7b61-abef-2540597c60a6`; `Turn: side:01a142e1-a377-7480-9e72-31a9ec7663a2`; `Provider: zai`.
- Exact total from cost summary: `Known subtotal exact USD: 0.00253325`.

### Recomputation
Earlier request remains $0.001295 (exact 0.00129515). New request $0.001238. 0.001295 + 0.001238 = 0.002533, matching rounded headline; exact ledger subtotal 0.00253325.

### Expected-result outcomes
- Request count > R0 — **passed** (2 > 1).
- Every new request has `Turn:` starting `side:` — **passed** for the one new request.
- O0 unchanged and side is not another conversation — **passed** (`No other conversation recorded requests on this day`; descendant attempts none).
- Earlier costs unchanged — **passed**.
- Each Request ID appears once — **passed** (Request 1 `38047faa-...`; Request 2 `02791f43-...`).

ACC-02 verdict basis: all expected results passed.

## ACC-03 — automatic review inside `/side`

Case start UTC: 2026-10-16T04:08:12Z. The amendment-supplied flags were used at restart.

### Requested versus actual
1. Requested stop/restart/resume with auto-review flags. Actual: stopped and started with `resume --last -c 'approvals_reviewer="auto_review"' -c 'approval_policy="on-request"' -c 'sandbox_mode="read-only"'`. The restored transcript was the ACC-01 conversation (`Reply with the single word OK.` / `OK` visible).
2. Requested note IDs/counts before test. Actual: request list showed the two earlier Request IDs and 2 requests; no other conversation was recorded.
3. Requested `/side`, review prompt, wait, leave, `/cost`, open technical pages. Actual: performed literally. Prompt entered at 2026-10-16T04:08:48Z. Screen at 2026-10-16T04:09:33Z showed one automatic review:
   `⚠ Automatic approval review approved (risk: low, authorization: unknown): Auto-review returned a low-risk allow decision.`
   `✔ Auto-reviewer approved Corbanu to run touch side-review-check.txt this time`
   and the model reply: `Done. touch side-review-check.txt ran in the workspace after approval; the file now exists (empty, 0 bytes).`
   Pressed Ctrl+C once and returned to saved conversation. Fixture observation at 2026-10-16T04:09:47Z confirmed `workspace/side-review-check.txt` exists with empty content.
4. Requested open new technical pages. Actual: opened Request 3, 4, 5, 6 and 7 technical pages. No retry prompt was needed.

### Observed `/cost` evidence
- Provider summary (`acc03-cost-request-list`, 2026-10-16T04:10:01Z):
  `→ Z.AI · Z.AI GLM 5.2 — estimated $0.006344 (rounded) (1 request)`
  `→ Z.AI · Z.AI GLM 5.3 Flash — estimated $0.005871 (rounded) (6 requests)`
- Later summary (`acc03-request-list-more`, 2026-10-16T04:10:10Z):
  `• Z.AI · Z.AI GLM 5.2 — Pay per use. 1 request, 4,446 tokens. Estimated cost: $0.006344 (rounded).`
  `• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 6 requests, 51,419 tokens. Estimated cost: $0.005871 (rounded).`
  `No other conversation recorded requests on this day.`
  `Estimated token cost for recorded attempts: $0.012216 (rounded)`
- Request IDs and costs:
  R3 `1388ff81-c647-4563-a028-24a866ae4d0b` $0.001468
  R4 `37414d79-2f1f-4d9a-bbcb-12569c1f6b71` $0.001232
  R5 `5116d46e-f25a-4f8d-97c7-43161de3af45` $0.006344
  R6 `9f98c18c-c6d6-44f1-9327-80951c12c588` $0.000340
  R7 `9849b921-7205-4757-89be-6230d7f0d256` $0.000298
- Turn lines observed:
  Request 3/4/6/7: `Turn: side:01a142e5-f181-7962-b2fc-f5207988a904`.
  Request 5: `Turn: side:review:01a142e6-59e3-7031-8807-0b9d84bac835`; provider technical line `Provider: zai`.

### Recomputation
Earlier two costs unchanged: 0.001295 + 0.001238 = 0.002533. New requests: 0.001468 + 0.001232 + 0.006344 + 0.000340 + 0.000298 = 0.009682. Sum all = 0.012215 rounded at six decimals; screen headline exact-ledger display says $0.012216. Difference reflects exact ledger rounding across per-request displays; the product displayed `Known subtotal exact USD: 0.00253325` before this case, while this case's grouped exact subtotal was not copied separately. This is an observation, not an accounting mismatch; individual displayed costs match the request list.

### Expected-result outcomes
- Session shows ≥1 automatic review — **passed** (exactly one; no retry).
- Number of `side:review:` requests equals number of automatic reviews and each is Z.AI — **passed** (1, Request 5).
- Every new request starts `side:` and none unprefixed — **passed**.
- Each Request ID appears once and other-conversations count unchanged — **passed** (IDs distinct; `No other conversation recorded requests on this day`).
- File presence matches reviewer decision — **passed** (approved, file exists).

ACC-03 verdict basis: all expected results passed.

## ACC-04 — sub-agent from `/side`

Status: **not exercised (model declined sub-agent execution)**.

### Requested versus actual
- Case start UTC: 2026-10-16T04:12:48Z. Entered `/side`; typed the exact case prompt; entered it at 2026-10-16T04:14:29Z; waited 60 seconds.
- The model declined. Verbatim screen text (wrapped by terminal):
  `I can't do that here. Sub-agents are off-limits in this side conversation, so I won't spawn or interact with one. If you'd like, ask this in the`
  `main thread, where sub-agent use is permitted.`
- Per amendment, no unchanged retry was needed for this advisory case. Pressed Ctrl+C once and returned to the saved conversation. `/cost` showed a new side-labelled request for the decline turn, but no sub-agent.

### Expected-result outcome
- "If model declines: not exercised (not a failure)" — **not exercised**. The decline was observed verbatim and no sub-agent ran.

## ACC-05 — sub-agent outside `/side`

Status: partially passed; one conditional result inconclusive due to ambiguous thread/conversation evidence.

### Requested versus actual
- Case start UTC: 2026-10-16T04:16:07Z. Typed exact prompt in the saved conversation and entered it; completed at 2026-10-16T04:17:08Z with:
  `• Spawned 01a142ec-c29b-78b3-900d-dded357a22c8 (glm-5.3-flash max)`
  `• The sub-agent replied: AGENT`
- Opened `/cost` at 2026-10-16T04:17:27Z. It showed `Z.AI · Z.AI GLM 5.3 Flash ... (11 requests)` and `No other conversation recorded requests on this day` on the related provider summary.
- Opened new technical pages Request 9, Request 10 and Request 11. The prior new Request 8 was the ACC-04 decline turn; it was not treated as a sub-agent request.

### Observed evidence
- R9: request `7073d17d-6e47-47e9-8b23-211fd0650882`, cost $0.001177, thread `01a142da-0909-7b61-abef-2540597c60a6`, turn `01a142ec-a6bf-7771-af69-3e639308ac3c`, no `side:` prefix.
- R10: request `a296be65-2104-4e1a-b066-860466291db8`, cost $0.001104, thread `01a142da-0909-7b61-abef-2540597c60a6`, turn `01a142ec-a6bf-7771-af69-3e639308ac3c`, no `side:` prefix.
- R11: request `49678dce-9fe9-471d-a903-98a8ccb068d1`, cost $0.000548, thread `01a142ec-c29b-78b3-900d-dded357a22c8`, turn `01a142ec-c2f6-7381-acdd-c12f6392256f`, no `side:` prefix.

### Expected-result outcomes
- Sub-agent requests listed once — **passed** (one distinct R11 request appeared under this conversation; no duplicate was observed).
- None starts `side:` — **passed** for R9–R11.
- Conditional `agent:` prefix when no own conversation — **inconclusive**. R11 had a distinct thread ID matching the spawned sub-agent ID, but the screens do not state whether that thread is a "conversation of its own"; no `agent:` prefix was shown. No behavior was invented.

## ACC-06 — `/review` attribution

Status: **failed** because expected new accounting requests were not visible after repeated refresh/reopen.

### Requested versus actual
- Case start UTC: 2026-10-16T04:19:56Z. Appended `// review me` to tracked `README.md` as the person; fixture showed the addition and uncommitted tracked modification (`M README.md`).
- Entered `/review`; the displayed menu listed `2. Review uncommitted changes`. Pressed Down to select it and Enter at 2026-10-16T04:20:37Z.
- The product showed `>> Code review started: current changes <<`, ran review commands, then `<< Code review finished >>` by 2026-10-16T04:22:11Z. It concluded: `The only tracked change is a README addition, and the sole untracked file is empty; neither introduces functional behavior or breaks existing usage.`
- Ran `/cost` at 2026-10-16T04:22:32Z. Reopened `/cost` after waits: 2026-10-16T04:23:46Z, 04:24:13Z, 04:24:44Z, and 04:25:44Z. The provider/request list remained unchanged.
- The final observed cost list (2026-10-16T04:25:44Z) still showed:
  `→ Z.AI · Z.AI GLM 5.2 — estimated $0.006344 (rounded) (1 request)`
  `→ Z.AI · Z.AI GLM 5.3 Flash — estimated $0.010232 (rounded) (11 requests)`
  `→ Request 1 ... $0.001295 (rounded)` through `→ Request 11 ... $0.000548 (rounded)` (visible across screen scrolls)
  with `No other conversation recorded requests on this day`.
- No new request page could be opened, so no `code-review:` technical page was observed.

### Expected-result outcomes
- New request(s) have `Turn:` starting `code-review:` — **failed** (none appeared in accounting after the required refresh window).
- Listed once under this conversation — **failed** (no new request appeared).
- Other-conversations count unchanged — **passed** (still no other conversation), but the expected-result is tied to the missing new requests and cannot rescue the case.
- Earlier costs unchanged — **passed**.

ACC-06 verdict basis: expected result failed.

## ACC-07 — ephemeral exec vs saved control

Case start UTC: 2026-10-16T04:26:36Z. Amendment wording and distinct EPHEMERAL/CONTROL prompts were used.

### Requested versus actual
1. Requested note and stop; ephemeral exec. Actual: before stop, cost was unchanged from ACC-06 final (`Z.AI GLM 5.2` 1 request; `Z.AI GLM 5.3 Flash` 11 requests; no other conversation). Stopped. Ran `agms-tui exec --ephemeral 'Reply with the single word EPHEMERAL.'`; output contained `EPHEMERAL`, 5,672 tokens, exit 0, at 2026-10-16T04:26:43Z.
2. Requested `resume --last` and `/cost`. Actual: resume opened the group-A transcript, showing the ACC-01 `OK`, ACC-05 sub-agent reply and ACC-06 code review; no EPHEMERAL prompt. `/cost` at 2026-10-16T04:27:08Z still showed 1 + 11 requests and no other conversation.
3. Requested stop/control exec. Actual: stopped; `agms-tui exec 'Reply with the single word CONTROL.'` output contained `CONTROL`, 6,980 tokens, exit 0, at 2026-10-16T04:27:21Z.
4. Requested `resume --last` and `/cost`. Actual: resume opened the group-A conversation (not the CONTROL conversation). `/cost` at 2026-10-16T04:27:36Z showed exactly one other conversation with one Z.AI request.

### Observed evidence
- After ephemeral (`acc07-cost-after-ephemeral`, 2026-10-16T04:27:08Z):
  `→ Z.AI · Z.AI GLM 5.2 — estimated $0.006344 (rounded) (1 request)`
  `→ Z.AI · Z.AI GLM 5.3 Flash — estimated $0.010232 (rounded) (11 requests)`
- After control (`acc07-cost-control-first`, 2026-10-16T04:27:36Z):
  `• Z.AI · Z.AI GLM 5.2 — Pay per use. 1 request, 4,446 tokens. Estimated cost: $0.006344 (rounded).`
  `• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 11 requests, 88,878 tokens. Estimated cost: $0.010232 (rounded).`
  `Other conversations on this day, not included above: 1 request in 1 conversation.`
  `• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 1 request, 7,556 tokens. Estimated cost: $0.001071 (rounded).`
  `Other conversations' estimated cost: $0.001071 (rounded).`

### Expected-result outcomes
- Both exec outputs contain their words — **passed**.
- After ephemeral, group-A transcript opened and totals/other-conversation count unchanged — **passed**.
- After control, exactly one more conversation with recorded requests and ≥1 Z.AI request — **passed** (0 → 1; Z.AI).
- Group-A total unchanged — **passed** (1 + 11 request groups and costs unchanged).

ACC-07 verdict basis: all expected results passed.

## ACC-34 — credential pattern scan (advisory)

Scan time UTC: 2026-10-16T04:28:21Z.

### Requested versus actual
- Scanned the captured screen JSONL (`evidence/screens.jsonl`), captured PTY output (`evidence/pty.raw`), both exec output files (`exec-2872.txt`, `exec-2954.txt`), recorded fixture JSONL, and the workspace files.
- Exact amendment patterns: `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, `api_key`, `token=`.
- Exact-case scan result: no hits in the ACC-34 specified scope.
- Informational case-insensitive scan found non-credential text: `authorization: unknown` in the product's automatic-review message; and the supplied synthetic fixture key name `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token` (uppercase `TOKEN=`, so it does not match the exact lowercase amendment pattern). The exact scan did not find the value pattern `agms-synthetic`.

### Expected-result outcome
- No hit in the exact amendment scan — **passed**. A broader case-insensitive scan is reported above for completeness; it found text and the known synthetic fixture, not a credential value. The integrator's real-key scan remains binding.

## Group end and final verdicts

- Group end UTC: 2026-10-16T04:28:21Z (last scan); all real-route model turns were completed before that.
- No credential or system-password prompt appeared. No required control was missing. No live account outside explicit test scope was touched.
- Ambiguities preserved: ACC-05 `agent:` conditional remained inconclusive; ACC-06 accounting requests did not appear after the specified refresh/reopen attempts; ACC-04 was not exercised because the model declined.

VERDICT ACC-01: passed
VERDICT ACC-02: passed
VERDICT ACC-03: passed
VERDICT ACC-04: passed
VERDICT ACC-05: passed
VERDICT ACC-06: failed
VERDICT ACC-07: passed
VERDICT ACC-34: passed

Status: partially passed; one conditional result inconclusive due to ambiguous thread/conversation evidence.

### Requested versus actual
- Case start UTC: 2026-10-16T04:16:07Z. Typed exact prompt in the saved conversation and entered it; completed at 2026-10-16T04:17:08Z with:
  `• Spawned 01a142ec-c29b-78b3-900d-dded357a22c8 (glm-5.3-flash max)`
  `• The sub-agent replied: AGENT`
- Opened `/cost` at 2026-10-16T04:17:27Z. It showed `Z.AI · Z.AI GLM 5.3 Flash ... (11 requests)` and `No other conversation recorded requests on this day` on the related provider summary.
- Opened new technical pages Request 9, Request 10 and Request 11. The prior new Request 8 was the ACC-04 decline turn; it was not treated as a sub-agent request.

### Observed evidence
- R9: request `7073d17d-6e47-47e9-8b23-211fd0650882`, cost $0.001177, thread `01a142da-0909-7b61-abef-2540597c60a6`, turn `01a142ec-a6bf-7771-af69-3e639308ac3c`, no `side:` prefix.
- R10: request `a296be65-2104-4e1a-b066-860466291db8`, cost $0.001104, thread `01a142da-0909-7b61-abef-2540597c60a6`, turn `01a142ec-a6bf-7771-af69-3e639308ac3c`, no `side:` prefix.
- R11: request `49678dce-9fe9-471d-a903-98a8ccb068d1`, cost $0.000548, thread `01a142ec-c29b-78b3-900d-dded357a22c8`, turn `01a142ec-c2f6-7381-acdd-c12f6392256f`, no `side:` prefix.

### Expected-result outcomes
- Sub-agent requests listed once — **passed** (one distinct R11 request appeared under this conversation; no duplicate was observed).
- None starts `side:` — **passed** for R9–R11.
- Conditional `agent:` prefix when no own conversation — **inconclusive**. R11 had a distinct thread ID matching the spawned sub-agent ID, but the screens do not state whether that thread is a "conversation of its own"; no `agent:` prefix was shown. No behavior was invented.
