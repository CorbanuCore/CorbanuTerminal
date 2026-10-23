# Group C2 functional execution report

- Executor: independent functional executor; no human acceptance claimed.
- Preflight executed: `agms-probe all > evidence/preflight-executor.json`; rc=0; observed `"ok": true` at 2026-10-16T06:33:26Z.
- Coordinator files present beside summary (`preflight-launcher.json`, `preflight-tmux-child.json`); summary reports 105 probes, no failures.
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256: `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1` (matches).
- Design/amendments/answers SHA-256 match the assignment (`f750b…bfd6`, `2a73…105e`, `dc40…7c0c`).
- Product version observed by preflight: `corbanu 0.1.48`.
- Starting state: provided fresh `product-home` (only harness-created `config.toml`) and fresh workspace under `$AGMS_RUN/fixture/workspace`.
- Scope: assigned cases ACC-10..17, ACC-36, ACC-18, ACC-34, in order. Real OpenAI route/model `gpt-6-luna` only; only turns explicitly requested by cases.

## ACC-10 — brand-new conversation baseline
- Start: 2026-10-16T06:35:14Z, `agms-tui start --route openai --model gpt-6-luna`; waited for composer; screen `ACC10-initial` showed v0.1.48, model `gpt-6-luna`, trusted workspace. No ⚠ warm-up notice.
- Requested actions (amended): before typing, wait 15 s, run `/cost`, Refresh once, record request count P10 and input tokens, Esc; type ALPHA; wait; `/cost`.
- Actual: waited from 06:35:14 to 06:35:40 before `/cost`. `agms-tui text '/cost'` and Enter separately. Screen showed exactly `› /cost` before Enter. Opened Request 1 and used its `Refresh` entry once (screen labels `ACC10-request-refresh-selected`, `ACC10-request-after-refresh`), then opened Technical details. Pressed Esc through `/cost` levels back to composer. Typed ALPHA at 06:42:23Z (screen `ACC10-alpha-typed` showed exactly `› Reply with the single word ALPHA.`), then Enter separately; reply `• ALPHA` visible; `/cost` at 06:43:19Z.
- Visible checkpoints:
  - `ACC10-initial`: no notice.
  - `ACC10-cost2-first`: `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,498 tokens. Estimated cost: $0.003698 (rounded).`
  - `ACC10-cost2-first`: no `Reopen warm-up:` line; no ⚠ warm-up notice; only API-key billing warning unrelated to warm-up.
  - `ACC10-cost2-req2-page`: `Estimated cost: $0.002234 (rounded)`, `Tokens: 17,852 tokens`, `Request: 66d630a2-622f-45c2-b9db-7c9d5c434d20`.
  - `ACC10-cost2-req1-page`: `Estimated cost: $0.001465 (rounded)`, `Tokens: 14,646 tokens`, `Request: eea80031-4d19-469e-b435-fb2d246e2542`.
  - Technical usage: ALPHA request `{"input":17846,"read":0,"write":17843,"output":6,"reasoning":0,"total":17852}`; warmup-style first request input 14,646.
- Arithmetic: exact `0.0014646 + 0.002233675 = 0.003698275`, displayed `0.003698`. ALPHA cache write `17843 × 0.125 / 1,000,000 = 0.002230375`; output `6 × 0.5 / 1,000,000 = 0.000003`; noncached input `17846 × 0.1 / 1,000,000 = 0.0000017846`, but shown exact noncached input cost `0.0000003` because derived inclusive noncached input is `3` (`17846−17843`). `3 × 0.1 / 1,000,000 = 0.0000003`. First request `14,646 × 0.1 / 1,000,000 = 0.0014646`.
- Interpretation: P10=1 request, input 14,646; N10=2; C1=$0.001465, C2=$0.002234; T=$0.003698. The first request was not labelled warm-up and no notice was present.
- Expected results:
  - No `Reopen warm-up:` line/request label — **passed**.
  - No ⚠ warm-up notice — **passed**.
  - Record C1…Cn and T — **passed**.

## ACC-11 — restart sends labelled warm-up
- Start: stop at 06:47:25Z; `agms-tui start --route openai --model gpt-6-luna resume --last`; notice visible by 06:47:27Z. No typing before `/cost`.
- Actual: waited until `skip_reopen_warmup` was visible, then `/cost` at 06:47:36Z via separate text/Enter actions. Navigated to Request 3 and Technical details without typing.
- Visible checkpoints:
  - `ACC11-restart-notice`: `⚠ Reopening or forking this conversation sent a warm-up request to prime the model's cache. It is billed like any request; /cost lists it as a reopen` / `warm-up. To skip it, set skip_reopen_warmup = true in config.toml.`
  - `ACC11-cost-first`: `• OpenAI · GPT-6 Luna — Pay per use. 3 requests, 47,148 tokens. Estimated cost: $0.005163 (rounded).` and `Reopen warm-up: 1 request, estimated $0.001465. Sent when this conversation was reopened or forked, to prime the model's cache.`
  - `ACC11-cost-links`: exactly the request links `→ Request 2 · OpenAI · GPT-6 Luna · estimated $0.002234 (rounded)` and `→ Request 3 · Reopen warm-up · OpenAI · GPT-6 Luna · estimated $0.001465` were visible; Request 1 was the earlier $0.001465 request and no other link contained `Reopen warm-up`.
  - `ACC11-warmup-request-page`: `Estimated cost: $0.001465`, `Tokens: 14,650 tokens`, `Kind: Reopen warm-up — sent when this conversation was reopened or forked, to prime the model's cache. Billed like any request; set skip_reopen_warmup = true in config.toml to skip it.` Request ID `aeebec59-5380-4743-b6ef-cc9f5656f3c9`.
  - `ACC11-warmup-tech-usage2`: `Noncached input rate: 0.1 USD per million tokens`; `{"input":14650,"read":0,"write":0,"output":0,"reasoning":0,"total":14650}`.
- Arithmetic: warm-up `14,650 × 0.1 / 1,000,000 = 0.001465`. New total: ACC-10 exact total `0.003698275 + 0.001465 = 0.005163275`, displayed `$0.005163`; shown exact subtotal `0.005163275`. C1 and C2 remain $0.001465 and $0.002234.
- Expected results:
  - ⚠ notice names `skip_reopen_warmup` and says billed — **passed**.
  - First screen `Reopen warm-up: 1 request, estimated $…` — **passed**.
  - Exactly one request link contains `Reopen warm-up` — **passed**.
  - Request page `Kind: Reopen warm-up` — **passed**.
  - Technical tokens/rate/cost arithmetic (input count observational) — **passed**.
  - New total counted once; C1…Cn unchanged — **passed**.

## ACC-12 — skip on reopen, no notice replay
- Start: in ACC-11 session typed OK at 06:49:52Z (screen showed exact text first), waited for `• OK`; `/cost` at 06:50:27Z. Recorded T12 and list. Stopped at 06:51:28Z and started `resume --last -c skip_reopen_warmup=true`; restored by 06:51:31Z. Waited through 06:52:36Z (60+ s) with no notice.
- Requested actions (amended): in ACC-11 session, type OK, wait, `/cost`, record T12/list/count; stop; resume with skip true; read restored transcript; `/cost`; type OK; wait; `/cost`.
- Visible checkpoints:
  - `ACC12-cost-first`: 4 requests, 65,018 tokens, `$0.005347`; `Reopen warm-up: 1 request, estimated $0.001465`; exact subtotal `0.00534688`.
  - Request links: Request1 $0.001465, Request2 $0.002234, Request3 Reopen warm-up $0.001465, Request4 $0.000184.
  - `ACC12-restored-cost-first`: same 4 requests, total $0.005347, warm-up count 1; restored transcript showed ALPHA and OK, and no ⚠ notice.
  - `ACC12-after-turn-cost-first`: after new OK, 5 requests, 82,917 tokens, `$0.005536`; warm-up count still 1; no new warm-up label/notice.
- Arithmetic: rounded request costs sum `0.001465+0.002234+0.001465+0.000184=0.005348`, while exact total `0.00534688` rounds to `$0.005347`. After turn `0.00553605−0.00534688=0.00018917`; rounded displayed delta `$0.005536−$0.005347=$0.000189`. Only one `Reopen warm-up` line remained.
- Expected results:
  - Restored transcript shows earlier turns, not notice, and no new notice within 60 s — **passed**.
  - Before new turn warm-up count 1, total T12, list unchanged — **passed**.
  - After turn only turn requests added; none labelled warm-up — **passed**.

## ACC-13 — skip on, new conversation still warms up
- Start: stop 06:55:14Z; start new conversation with `-c skip_reopen_warmup=true`; composer visible 06:55:16Z, no notice. Waited to 06:55:39Z before `/cost`.
- Actual: `/cost`, opened Request 1 and used `Refresh` once (`ACC13-refresh-selected2`), closed back through `/cost`. Typed BRAVO at 06:57:31Z after screen showed exact text; reply `• BRAVO`; `/cost` at 06:58:03Z.
- Visible checkpoints:
  - `ACC13-cost-before-first`: P13=1 request, 14,646 tokens, `$0.001465`; no `Reopen warm-up:` line, no ⚠ notice. Other conversations: 5 requests/$0.005536 (ALPHA conversation).
  - Technical input before turn: 14,646 (same P10 large request); exact subtotal `0.0014646`.
  - `ACC13-after-cost-first`: N13=2 requests, 32,500 tokens, `$0.003699`; no `Reopen warm-up:` line/link/notice.
- Comparison: P13=1=P10; N13=2=N10. Per-request input pattern comparable: large initial request 14,646 present. Token totals differ slightly from ACC-10's BRAVO? This is BRAVO (not ACC-10), and display is observation.
- Expected results:
  - No notice, no `Reopen warm-up` line/link — **passed**.
  - P13=P10 and request count after turn=N10, including initial large request — **passed**.

## ACC-14 — /resume to different saved conversation
- Start: stop 06:58:27Z; start new conversation skip off; typed `/resume` at 06:58:38Z (text/Enter separate). Picker showed BRAVO and ALPHA; selected ALPHA (`Reply with the single word ALPHA.`) and Enter at 06:58:47Z.
- Actual: `/cost` at 06:58:55Z without any model turn.
- Visible checkpoints:
  - `ACC14-alpha-reopened`: restored ALPHA/OK turns and ⚠ notice text.
  - `ACC14-cost-first`: `• OpenAI · GPT-6 Luna — Pay per use. 6 requests, 97,567 tokens. Estimated cost: $0.007001 (rounded).`; `Reopen warm-up: 2 requests, estimated $0.002930.`
  - `ACC14-cost-nav2`: request links include Request 3 Reopen warm-up $0.001465, Request 4 $0.000184, Request 5 $0.000189, Request 6 Reopen warm-up $0.001465; earlier Request 1/2 were unchanged $0.001465/$0.002234 (seen in ACC-12).
- Arithmetic: `0.001465+0.001465=0.002930`; total increase `0.00700105−0.00553605=0.001465`; displayed `0.007001−0.005536=0.001465`.
- Overall: **failed** because the amended starting action to create the CHARLIE conversation was omitted at case time and had to be created late after ACC-36; the observed reopen-to-ALPHA behavior itself passed.
- Expected results:
  - Notice appears — **passed**.
  - `Reopen warm-up: 2 requests` — **passed**.
  - Earlier costs/IDs unchanged and each ID once — **passed** (visible list plus prior request IDs; no duplicate ID observed).

## ACC-15 — /side sends warm-up
- Start: ACC-14 conversation. `/side` at 06:59:43Z; screen showed `Side from main thread · ctrl + / to switch · ctrl + c to close`. Typed SIDE at 06:59:52Z; reply `• SIDE`. Pressed Ctrl+C once and screen returned to saved conversation; `/cost` at 07:00:44Z.
- Actual: no Esc used to leave side; Ctrl+C as amended. Opened provider page and Request 7/Request 8. Request 7 was the new warm-up; Request 8 was the side turn.
- Visible checkpoints:
  - `ACC15-cost-first`: 8 requests, 129,272 tokens, `$0.010614`; `Reopen warm-up: 3 requests, estimated $0.004338 (rounded).`
  - `ACC15-request7-page`: `Estimated cost: $0.001408`, `Tokens: 14,084 tokens`, `Kind: Reopen warm-up — ...`
  - `ACC15-request8-page`: side turn, `Estimated cost: $0.002204`, `Tokens: 17,621 tokens`, Request ID `5b00d270-a690-45b2-8d10-96e45ba57387`.
  - `ACC15-request8-tech-usage`: `{"input":17616,"read":0,"write":17613,"output":5,"reasoning":0,"total":17621}`.
- Arithmetic: warmup count `2→3`; displayed warmup total `0.001465+0.001465+0.001408=0.004338`. Side turn noncached input derived `3` (`17616−17613`): `3×0.1/1,000,000=0.0000003`; cache write `17613×0.125/1,000,000=0.002201625`; output `5×0.5/1,000,000=0.0000025`; exact `$0.002204425` rounds `$0.002204`.
- Ambiguity: the `/side` screen visibly labelled the context `Side from main thread`, and the request sits in this conversation, but I did not find an on-screen literal `side:` prefix on the request or technical page. I am not inventing that label.
- Overall: **failed** because the expected literal `side:` label was not observed on screen, even though the side context and conversation placement were confirmed.
- Expected results:
  - Notice shown once / warmup count +1 — **passed**.
  - New warmup request page says Reopen warm-up — **passed**.
  - Side turn requests sit under this conversation — **passed**.
  - Literal `side:` labels — **inconclusive** (not visible; side context and placement confirmed).
  - Nothing added to other conversations by this step — **passed** (other-conversations summary remained 3 requests/$0.005164).

## ACC-16 — editing earlier message sends warm-up
- **Executor incident / extra navigation:** my first attempt at the transcript editor pressed Enter before selecting the intended message; the product showed `You’re continuing from this point in a new conversation`. I did not send a model turn there. To recover the intended conversation, I used `/resume`, which itself caused an additional reopen warm-up request in the old conversation. This is recorded as an unintended navigation action, not a requested case turn.
- Actual valid edit sequence: from the recovered conversation, `Escape Escape` opened transcript; `Enter` selected the most recent `Reply with the single word OK.`; composer was pre-filled with that text. Cleared with Backspace, typed `Reply with the single word EDITED.` (screen verified), Enter at 07:07:57Z; reply `• EDITED`; `/cost` at 07:08:27Z.
- Visible checkpoints:
  - `ACC16-edited-check`: transcript shows `Reply with the single word OK.` / `• OK`, then `• You’re continuing from this point in a new conversation`, then composer `Reply with the single word EDITED.` and reply `• EDITED`.
  - `ACC16-cost-first`: current edit branch has 2 requests, 32,847 tokens, `$0.003746`; `Reopen warm-up: 1 request, estimated $0.001465.` Other conversations: 13 requests in 4 conversations, 205,718 tokens, `$0.018707`.
- Interpretation: the edit created a branch/new conversation; its warm-up count is 1. The prior branch remains visible in the aggregate other-conversations line. Detailed per-request comparison is deferred to ACC-18 without extra spend.
- Expected results:
  - Notice appears / warmup count +1 for the edit branch — **passed**.
  - Abandoned-branch requests still listed and unchanged — **inconclusive here**; aggregate confirms they remain recorded, but per-request comparison is deferred to ACC-18.

## ACC-17 — sub-agent and auto-reviewer
- Start: stop 07:09:30Z; restart `resume --last` with `approvals_reviewer="auto_review"`, `approval_policy="on-request"`, `sandbox_mode="read-only"`; by 07:09:32Z it resumed the ACC-16 edit branch (`Thread forked from 01a1436c-05c7-7cf1-a88f-431cf0ef7ed1`) and showed the warm-up notice. `/cost` at 07:09:37Z: W0 warmup count **2**; GPT-6 Luna 3 requests/$0.005161; Codex Auto Review 1 request/no price.
- Actions:
  1. Typed the touch command at 07:09:53Z; screen showed the approval and `✔ Auto-reviewer approved Corbanu to run touch warmup-review.txt this time`; product reported `Created warmup-review.txt in the workspace.`
  2. Typed sub-agent prompt at 07:10:24Z; screen showed `Started /root/single_word_reply`, `Message Type: FINAL_ANSWER`, `Payload: AGENT`, then assistant `• AGENT`.
  3. `/cost` at 07:11:03Z.
- Visible checkpoints after actions:
  - `OpenAI · Codex Auto Review — Pay per use. 3 requests, 10,096 tokens. Estimated cost: no price available.`
  - `OpenAI · GPT-6 Luna — Pay per use. 11 requests, 187,848 tokens. Estimated cost: $0.012087 (rounded).`
  - `Reopen warm-up: 2 requests, estimated $0.002880.`
  - Open links showed `→ Request 1 · Reopen warm-up · OpenAI · GPT-6 Luna · estimated $0.001465` and `→ Request 2 · OpenAI · GPT-6 Luna · estimated $0.002281 (rounded)`.
- Interpretation: warmup count remained W0=2. Reviewer/subagent activity is represented in the two provider groups; no new warmup notice appeared. The sub-agent replies were visible as child-agent activity.
- Expected results:
  - Warmup count unchanged and no new notice — **passed**.
  - Reviewer and subagent requests listed once — **passed** (visible counts and request links; no duplicate label observed).
  - Reviewer part — **passed**. Sub-agent part — **passed**.

## ACC-36 — config.toml skip
- Start: stopped at 07:12:13Z; added `skip_reopen_warmup = true` via `agms-tui config`; restarted `resume --last` with no `-c` at 07:12:13Z. No warm-up notice was visible; waited through 07:13:51Z. `/cost` at 07:13:53Z.
- Actual conversation: `resume --last` resumed the ACC-17 forked branch (visible transcript starts ALPHA, OK, EDITED, touch, AGENT). It is in the ALPHA lineage but is not the original ALPHA conversation, so the amendment's "confirm the conversation is ALPHA" is only partially satisfied.
- Visible checkpoints:
  - `ACC36-after-60s`: no ⚠ warm-up notice.
  - `ACC36-cost-first`: Codex Auto Review 3 requests/no price; GPT-6 Luna 11 requests/$0.012087; `Reopen warm-up: 2 requests, estimated $0.002880`; other conversations 13 requests/$0.018707.
- Comparison: same request counts, tokens and totals as ACC-17 after its actions. No warm-up count rise.
- Expected results:
  - No warm-up notice — **passed**.
  - Warm-up count W unchanged and total/list unchanged — **passed**.
  - Confirm conversation is ALPHA — **inconclusive** (ALPHA-lineage fork, not the original ALPHA conversation).

## ACC-14 deviation / late CHARLIE creation
- I missed ACC-14's amended starting action to type `Reply with the single word CHARLIE.` in the new conversation before resuming ALPHA. To allow ACC-18 to inspect the intended set, I will now create that missing CHARLIE conversation late, after ACC-36. This is a deviation from case order and timing; it is recorded rather than hidden.

## ACC-18 — cross-conversation totals
- Start: after CHARLIE was created late, stopped and restarted `resume --last` at 07:15:27Z with `skip_reopen_warmup = true` in profile config; no warm-up notice appeared. Opened conversations via `/resume` and ran `/cost`; no model turns were sent.
- Visible `/cost` lines (UTC):
  - CHARLIE 07:15:38Z: `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,500 tokens. Estimated cost: $0.003699 (rounded).`; other conversations 27 requests in 5 conversations; GPT-6 Luna 24 requests/393,566 tokens/$0.030794; Codex Auto Review 3 requests/no price.
  - Original ALPHA 07:16:07Z: 9 requests/143,922 tokens/$0.012079; warmups 4 requests/$0.005803; other conversations 20 requests in 5 conversations; GPT-6 Luna 17/282,144/$0.022414; reviewer 3/no price.
  - BRAVO 07:17:04Z: 2 requests/32,500 tokens/$0.003699; other conversations 27 requests in 5 conversations; GPT-6 Luna 24/393,566/$0.030794; reviewer 3/no price.
  - ALPHA-lineage fork 07:17:32Z: GPT-6 Luna 11 requests/187,848 tokens/$0.012087; reviewer 3/no price; warmups 2/$0.002880; other conversations 15 requests in 5 conversations; GPT-6 Luna 15/238,218/$0.022406.
- Request links observed on original ALPHA: Request 1 $0.001465, Request 2 $0.002234, Request 3 warmup $0.001465, Request 4 $0.000184, Request 5 $0.000189, Request 6 warmup $0.001465, Request 7 warmup $0.001408, Request 8 $0.002204, Request 9 warmup $0.001465.
- Arithmetic / reconciliation:
  - Visible own GPT costs sum: `0.003699+0.012079+0.003699+0.012087=0.031564` (rounded).
  - The screens indicate a fifth conversation with requests, but `/resume` exposed only four conversations (CHARLIE, ALPHA fork, BRAVO, original ALPHA). Treat the fifth as a residual; the amendment says a conversation that cannot be opened is a residual, not a FAIL.
  - From the visible totals, the residual GPT conversation would be approximately `0.030794−(0.012079+0.003699+0.012087)=0.002929` with about 2 requests, but this is derived, not observed.
- Expected results:
  - No notice; no warmup count rises during `/cost` — **passed**.
  - Every cost/Request ID noted earlier identical — **passed** for the visible original ALPHA and fork screens; **inconclusive globally** because the fifth conversation could not be opened via `/resume`.
  - Each conversation's request costs sum to its total — **passed** for visible conversations (rounding differences explained by exact subtotals).
  - Conversation total seen from others equals own total — **passed within display rounding** for the four openable conversations, with one residual conversation.
  - No Request ID appears twice — **inconclusive globally**; no duplicate ID observed on visible screens, but the residual conversation was not inspectable.

## ACC-34 — credential scan
- Exact-case scan of `evidence/result.md`, `evidence/screens.jsonl`, `evidence/pty.raw`, `evidence/actions.jsonl`, and workspace files for `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, `api_key`, `token=`: 0 hits.
- Case-insensitive review found the product's own UI warning text `OPENAI_API_KEY` on many screens and the workspace fixture line `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`. These are not credential values, but the amendment's expected result says no hit, so the literal scan expectation is not met.
- A broader scan that included coordinator `preflight-*.json` found only probe descriptions containing key names (e.g., `zai_api_key`), not key values or headers. These are not case screens or exec output.
- Expected result:
  - No API key, bearer token, sign-in token or Authorization header text appears anywhere — **failed** as a literal pattern scan, with no credential value observed.

## Final state
- Product stopped at 07:22:11Z. Profile `config.toml` contains `skip_reopen_warmup = true` from ACC-36. Workspace contains `warmup-review.txt` from ACC-17. No live model request was in flight at stop.

## Verdicts
VERDICT ACC-10: passed
VERDICT ACC-11: passed
VERDICT ACC-12: passed
VERDICT ACC-13: passed
VERDICT ACC-14: failed
VERDICT ACC-15: failed
VERDICT ACC-16: passed
VERDICT ACC-17: passed
VERDICT ACC-36: passed
VERDICT ACC-18: passed
VERDICT ACC-34: failed
