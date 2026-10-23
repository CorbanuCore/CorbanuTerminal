# Group O executor report

## Preflight and identity
- `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned rc=0 at UTC 2026-10-20T03:07:29Z and `"ok": true`.
- Coordinator preflight summary and both `preflight-*.json` files also reported `"ok": true`; their `failed` arrays were empty.
- Candidate identity: `/opt/agms/pkg/bin/corbanu` SHA-256 `e72003067f9316f2805b4344041d111409611c6d0730fe268c46bf95225bdaa3`; the binary identified itself as `corbanu 0.1.48`.
- Frozen packet hashes matched: design `38afb2d4119c4776af1315fea8cf274984807ff868e031a3992af0234c4986a4`; amendments `6bb58647630bd980b10275890afd9f4257dace08080906667ed25171a2d17c3a`; answers `296d2b5c0768d0162b63e5f08c332d5200d902614453956c29f8bf6376d42167`.
- All prompt text and Enter were separate `agms-tui` actions. Start/reopen/turn UTC times are in each case. Screens are in `$AGMS_RUN/evidence/screens/`.

## Cases

### FUP-09 — passed
**Requested versus actual.** Started fresh openai/gpt-6-luna profile. Typed `Reply with the single word OK.` at 03:09:31Z; Enter 03:09:32Z; visible `OK` 03:09:40Z. Opened `/cost`, Request 1 and technical details. Stopped 03:13:45Z; resumed `--last` at 03:13:48Z. Typed the same sentence and Enter at 03:14:00Z; visible `OK` 03:14:08Z. Opened `/cost`, warm-up request and technical details.

**Judged lines.**
- `group-O-FUP-09-05-cost-top.txt` (03:09:47Z): `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,475 tokens. Estimated cost: $0.003695 (rounded).`
- `group-O-FUP-09-07-cost-links2.txt`: `Unknown parent population: 0 attempts, excluded from root total`; `→ Request 1 · OpenAI · GPT-6 Luna · estimated $0.001465 (rounded)`; `→ Request 2 · OpenAI · GPT-6 Luna · estimated $0.002230 (rounded)`.
- `group-O-FUP-09-13-request1-page.txt`: `Estimated cost: $0.001465 (rounded)`, `Tokens: 14,646 tokens`, `Request: 06248f05-eca7-43b2-a182-1868df689c90`.
- `group-O-FUP-09-22-request1-thread3.txt`: `Thread: 01a15748-ce65-7990-a76b-5994b8115171`, `Turn: prewarm:ef03ab24-f794-4f14-8026-c275b4f0a081`.
- `group-O-FUP-09-23-resume.txt`: `⚠ Reopening or forking this conversation sent a warm-up request to prime the model's cache. It is billed like any request; /cost lists it as a reopen` / `warm-up. To skip it, set skip_reopen_warmup = true in config.toml.`
- `group-O-FUP-09-27-cost-after-restart-top.txt`: `• OpenAI · GPT-6 Luna — Pay per use. 4 requests, 64,972 tokens. Estimated cost: $0.005343 (rounded).`; `Reopen warm-up: 1 request, estimated $0.001465. Sent when this conversation was reopened or forked, to prime the model's cache.`
- `group-O-FUP-09-28-cost-after-restart-links.txt`: `→ Request 3 · Reopen warm-up · OpenAI · GPT-6 Luna · estimated $0.001465`.
- `group-O-FUP-09-29-warmup-request.txt`: `Estimated cost: $0.001465`; `Tokens: 14,650 tokens`; `Kind: Reopen warm-up — sent when this conversation was reopened or forked, to prime the model's cache. Billed like any request; set` / `skip_reopen_warmup = true in config.toml to skip it.`
- `group-O-FUP-09-30-warmup-request-id.txt`: `Request: 96d1d2c7-0104-4f0b-8c26-a5af5dd42a53`.
- `group-O-FUP-09-33-warmup-technical-thread.txt`: `Input: 14650`; cache read/write/output/reasoning `0 (reported)`; `Known subtotal exact USD: 0.001465`; `Noncached input rate: 0.1 USD per million tokens`.
- `group-O-FUP-09-37-warmup-technical-thread5.txt`: `Thread: 01a15748-ce65-7990-a76b-5994b8115171`; `Turn: prewarm:reopen:46b53baf-9a3c-4e2d-99ee-333ac2f5ed3b`.

**Recomputation.** 14,650 input × $0.1 / 1,000,000 = $0.001465. This matched the request and exact subtotal.

**Outcome.** No warm-up line/notice before restart: passed. `skip_reopen_warmup` notice after restart: passed. `Reopen warm-up: 1 request`: passed. Warm-up link and `Kind`: passed. Positive tokens/cost and Thread T1: passed. Included in count/total: passed.

### FUP-11 — passed
**Requested versus actual.** Stopped 03:17:32Z and started `resume 01a15748-ce65-7990-a76b-5994b8115171` at 03:17:34Z. Typed `Reply with the single word OK.` and Enter at 03:17:48Z. The reply became visible at 03:17:56Z. Opened `/cost`.

**Judged lines.**
- `group-O-FUP-11-01-resume-id.txt`: `⚠ Reopening or forking this conversation sent a warm-up request to prime the model's cache. It is billed like any request; /cost lists it as a reopen` / `warm-up. To skip it, set skip_reopen_warmup = true in config.toml.`
- Initial `group-O-FUP-11-04-cost-top.txt` briefly showed `1 attempt had incomplete usage`; after reopening `/cost`, `group-O-FUP-11-10-cost-reopen-links.txt` showed `→ Request 6 · OpenAI · GPT-6 Luna · estimated $0.000189 (rounded)` and `→ Request 5 · Reopen warm-up · OpenAI · GPT-6 Luna · estimated $0.001465`.
- `group-O-FUP-11-14-cost-reopen-knownsubtotal2.txt`: `Known subtotal exact USD: 0.01372482` was captured after the final audit; FUP-11's visible top page showed 6 requests and warm-up count 2.

**Recomputation.** FUP-11 added one warm-up and one turn: warm-up $0.001465 + turn $0.000189 ≈ $0.001654. Warm-up count rose from 1 to 2.

**Outcome.** Warm-up count +1: passed. Notice: passed. FUP-10 audit: blocked by the final exact-total mismatch (below); all FUP-11 IDs/costs were stable.

### FUP-12 — passed
**Requested versus actual.** In T1, `/new`; typed `Reply with the single word SECOND.` and Enter at 03:30:38Z; visible `SECOND` 03:30:39Z. Opened `/resume`; selected the second row (`Reply with the single word OK.`, T1) and Enter. This produced a warm-up notice. Typed `Reply with the single word OK.` and Enter at 03:31:09Z; visible `OK` 03:33:05Z. Opened `/cost`.

**Judged lines.**
- `group-O-FUP-12-04-resume-picker.txt` and `group-O-FUP-12-05-pick-T1.txt` show T1 was selected from the picker.
- `group-O-FUP-12-06-resumed-T1.txt`: `⚠ Reopening or forking this conversation sent a warm-up request to prime the model's cache. It is billed like any request; /cost lists it as a reopen` / `warm-up. To skip it, set skip_reopen_warmup = true in config.toml.`
- `group-O-FUP-12-09-cost-top.txt`: `• OpenAI · GPT-6 Luna — Pay per use. 8 requests, 130,042 tokens. Estimated cost: $0.008647 (rounded).`; `Reopen warm-up: 3 requests, estimated $0.004395. Sent when this conversation was reopened or forked, to prime the model's cache.`; `Other conversations on this day, not included above: 2 requests in 1 conversation.`; `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,479 tokens. Estimated cost: $0.003695 (rounded).`

**Recomputation.** Warm-up count rose 2→3. T2 held only its own two product-generated requests: $0.003695 rounded. This is the same prewarm+turn pattern as the first conversation.

**Outcome.** T1 warm-up count +1 and notice: passed. T2 separation: passed. FUP-10 audit: blocked by final exact-total mismatch; no duplicate IDs were observed.

### FUP-13 — passed
**Requested versus actual.** In T1, `/side`; typed `Reply with the single word OK.` and Enter at 03:33:39Z; visible `OK` 03:33:40Z. Left with exactly one Ctrl+C at 03:33:45Z and returned to the saved conversation. Opened `/cost`.

**Judged lines.**
- `group-O-FUP-13-05-cost-top.txt`: `• OpenAI · GPT-6 Luna — Pay per use. 10 requests, 161,742 tokens. Estimated cost: $0.012259 (rounded).`; `Reopen warm-up: 4 requests, estimated $0.005803 (rounded). Sent when this conversation was reopened or forked, to prime the model's cache.`
- `group-O-FUP-13-08-cost-links3.txt`: `→ Request 9 · Reopen warm-up · OpenAI · GPT-6 Luna · estimated $0.001408 (rounded)`; `→ Request 10 · OpenAI · GPT-6 Luna · estimated $0.002204 (rounded)`.
- `group-O-FUP-13-09-request10.txt`: `Estimated cost: $0.002204 (rounded)`; `Request: 19ec34e3-34a0-4644-96e5-f1cd52821a6b`.
- `group-O-FUP-13-14-side-technical-id.txt`: `Thread: 01a15748-ce65-7990-a76b-5994b8115171`; `Turn: side:01a1575f-329a-7e83-97e2-e5fa6031c648`.

**Recomputation.** Request 9 exact $0.0014084 + Request 10 exact $0.0022038 = $0.0036122. Request 9 was labelled `Reopen warm-up`; Request 10 was labelled `Turn: side:`.

**Outcome.** Warm-up and side turn inside T1: passed. `Turn: side:` label: passed. FUP-10 audit: blocked by final exact-total mismatch; no duplicate IDs were observed.

### FUP-14 — failed
**Requested versus actual.** With the composer empty, pressed Esc twice; the product opened a transcript overlay with edit hints. Because the overlay first selected the end of the transcript, a first Enter created a `continuing from this point in a new conversation` view, which was cancelled without sending. The case was restarted with `resume <T1>` to recover, which itself added a warm-up. Repeated Escape navigation then reached the transcript beginning; Enter there opened a continuation from the earliest point, and C-u cleared the prefilled text. Typed `Reply with the single word YES.` and Enter at 03:40:14Z; visible `YES` 03:40:32Z. Opened `/cost`, then resumed T1 with `-c skip_reopen_warmup=true` only to inspect its cost without adding another warm-up.

**Judged lines.**
- `group-O-FUP-14-02-edit-picker.txt`: `↑/↓ to scroll`, `esc/← to edit prev`, `→ to edit next`, `enter to edit message`.
- `group-O-FUP-14-17-earliest-edit.txt`: `• You’re continuing from this point in a new conversation`; composer showed the earliest text.
- `group-O-FUP-14-20-reply.txt`: `• You’re continuing from this point in a new conversation` and `• YES`.
- `group-O-FUP-14-21-cost.txt`: new conversation had `2 requests, 32,479 tokens. Estimated cost: $0.003695 (rounded).`; no `Reopen warm-up` line.
- `group-O-FUP-14-23-T1-cost.txt`: `• OpenAI · GPT-6 Luna — Pay per use. 11 requests, 176,392 tokens. Estimated cost: $0.013724 (rounded).`; `Reopen warm-up: 5 requests, estimated $0.007268 (rounded). Sent when this conversation was reopened or forked, to prime the model's cache.`

**Recomputation.** Before FUP-14, T1 had 10 requests/4 warm-ups. The recovery restart added T1 warm-up R11 (not a case-mandated edit warm-up). The actual YES continuation did not add a labelled warm-up or notice in its new conversation.

**Outcome.** YES reply: passed. Warm-up count +1 attributable to the edit: failed. New warm-up notice attributable to the edit: failed. Old T1 requests remained listed once: passed/inconclusive pending final audit; the final audit found them stable.

### FUP-15 — failed
**Requested versus actual.** An initial picker Enter selected the newer YES continuation, not T1. The product was restarted into T1 with `-c skip_reopen_warmup=true` to avoid another warm-up, then `/resume` was opened and T1 selected.

**Judged lines.**
- `group-O-FUP-15-05-selected-T1.txt` shows `❯` on the second row, `4m ago`, `Reply with the single word OK.`
- Enter at 03:43:01Z produced `group-O-FUP-15-06-already-viewing.txt`: `• Thread forked from 01a15748-ce65-7990-a76b-5994b8115171`; no “Already viewing”; no ⚠ notice.
- `group-O-FUP-15-07-cost.txt`: `• OpenAI · GPT-6 Luna — Pay per use. 1 request, 14,650 tokens. Estimated cost: $0.001465.`; `Reopen warm-up: 1 request, estimated $0.001465. Sent when this conversation was reopened or forked, to prime the model's cache.`

**Recomputation.** The fork warm-up was 14,650 × $0.1 / 1,000,000 = $0.001465 and counted once.

**Outcome.** The action was performed, but it forked instead of showing “Already viewing”: failed. A warm-up was sent, labelled and counted once: passed. No notice appeared: failed.

### FUP-16 — passed
**Requested versus actual.** Stopped 03:43:34Z. Started `-c skip_reopen_warmup=true resume <T1>`. Typed `Reply with the single word OK.` and Enter at 03:43:42Z; visible `OK` 03:45:39Z. Opened `/side`, typed the same sentence and Enter at 03:46:03Z; visible `OK` 03:46:05Z. Left with one Ctrl+C at 03:46:16Z. Opened `/cost`.

**Judged lines.**
- `group-O-FUP-16-01-start-skip.txt` and later screens show no new warm-up notice.
- `group-O-FUP-16-08-cost.txt`: `• OpenAI · GPT-6 Luna — Pay per use. 13 requests, 211,948 tokens. Estimated cost: $0.016119 (rounded).`; `Reopen warm-up: 5 requests, estimated $0.007268 (rounded). Sent when this conversation was reopened or forked, to prime the model's cache.`

**Recomputation.** T1 had 11 requests before FUP-16 and 13 after; only the main turn and side turn were added. Warm-up count stayed 5. Exact sum of audited T1 requests through R13 was $0.01611978, matching the displayed rounded total.

**Outcome.** No warm-up notice; count unchanged: passed. Only the two turn requests added: passed. Turns worked normally: passed. FUP-10 audit: blocked by final exact-total mismatch; no duplicate IDs were observed.

### FUP-25 — passed
**Requested versus actual.** At 03:46:41Z, `agms-tui config 'skip_reopen_warmup = true'` appended the setting. The product was stopped and started with `resume <T1>` and no `-c`. Typed `Reply with the single word OK.` and Enter at 03:46:49Z; visible `OK` 03:46:54Z. Opened `/cost`.

**Judged lines.**
- `group-O-FUP-25-01-start-config.txt`: no warm-up notice.
- `group-O-FUP-25-04-cost.txt`: `• OpenAI · GPT-6 Luna — Pay per use. 14 requests, 229,889 tokens. Estimated cost: $0.016309 (rounded).`; `Reopen warm-up: 5 requests, estimated $0.007268 (rounded). Sent when this conversation was reopened or forked, to prime the model's cache.`

**Recomputation.** T1 had 13 requests before FUP-25 and 14 after; warm-up count stayed 5. Request 14 exact cost is shown under FUP-10.

**Outcome.** No warm-up notice; warm-up count unchanged; only turn requests added: passed.

### FUP-10 — failed
**Requested versus actual.** Final audit from T1 `/cost` opened every request link 1–14 and its technical details page. Request IDs, request costs and technical exact costs were captured. A refresh attempt did not change the subtotal.

**Group line.**
- `group-O-FUP-10-refresh-exact2.txt`: `Known subtotal exact USD: 0.01630897`; `Input: 229,817`; `Noncached input (derived for inclusive input): 87,354`; `Cache read: 89,307`; `Cache write: 53,156`.
- Links show 14 requests, including 5 labelled `Reopen warm-up` links.

**Audit table (request-page cost; technical exact cost).**
1. `06248f05-eca7-43b2-a182-1868df689c90` — $0.001465 rounded; exact $0.001465 (`group-O-FUP-09-33-warmup-technical-thread.txt`).
2. `22b288df-e830-453b-b80d-01ec427b4f61` — $0.002230 rounded; exact $0.002230425 (`group-O-FUP-10-R02-tech-exact2.txt`).
3. `96d1d2c7-0104-4f0b-8c26-a5af5dd42a53` — $0.001465; exact $0.001465 (`group-O-FUP-10-R03-tech-exact.txt`).
4. `4a9567ec-3f13-49c6-bbde-99a8013d29ec` — $0.000183 rounded; exact $0.00018326.
5. `30e40a17-7238-4458-9d2a-297e1302e505` — $0.001465; exact $0.001465.
6. `3cbc4858-1d6b-47b4-a09d-0666a0b73a32` — $0.000189 rounded; exact $0.00018894.
7. `b226eec7-6a5c-4216-b566-5aa5a8cefe6e` — $0.001465; exact $0.001465.
8. `43c35453-8967-4d3a-910a-2ea453bc3041` — $0.000185 rounded; exact $0.000184995.
9. `7f290403-8273-46f8-9f7e-48014ee27fc6` — $0.001408 rounded; exact $0.0014084.
10. `19ec34e3-34a0-4644-96e5-f1cd52821a6b` — $0.002204 rounded; exact $0.0022038.
11. `575ebbea-b14b-4cf4-a5b0-6b413abecc8b` — $0.001465; exact $0.001465.
12. `be8a0f3b-0452-42e8-a1dc-0f1d6938149a` — $0.000184 rounded; exact $0.00018391.
13. `21214471-42ce-4be0-b1f5-df9a3c7c841e` — $0.002211 rounded; exact $0.00221105.
14. `49d7972e-7371-4db5-8448-4687bd766423` — $0.000190 rounded; exact $0.00018959.

**Arithmetic.**
`0.001465000 + 0.002230425 + 0.001465000 + 0.000183260 + 0.001465000 + 0.000188940 + 0.001465000 + 0.000184995 + 0.001408400 + 0.002203800 + 0.001465000 + 0.000183910 + 0.002211050 + 0.000189590 = 0.01630937.`

The group page says `0.01630897`. Difference: `$0.01630937 − $0.01630897 = $0.0000004.` The request-page exact costs therefore do not sum to the group exact subtotal.

**Outcome.**
- Every earlier ID was present once and with the same displayed cost: passed.
- All 14 technical pages were opened: passed.
- No duplicate Request ID: passed.
- New entries only expected: blocked/inconclusive because the recovery restart in FUP-14 added R11 outside the case action.
- Exact sum equals group subtotal: failed by $0.0000004.
- Warm-up labels: passed.

### FUP-24 — passed
Searched all saved screens/evidence for `sk-`, `Bearer`, `Authorization`, `api_key=`, the synthetic `.env` value `agms-canary-7f3a9c-synthetic-token`, and a 40+-character non-UUID token-like expression. The explicit patterns produced no hits. The broad expression matched only screen names/UUID-adjacent transcript paths, package SHA-256 values, and text-token examples; no credential or key material was observed. No model-run command transcript in this group printed a secret.

## Verdicts
VERDICT FUP-09: passed
VERDICT FUP-10: failed
VERDICT FUP-11: passed
VERDICT FUP-12: passed
VERDICT FUP-13: passed
VERDICT FUP-14: failed
VERDICT FUP-15: failed
VERDICT FUP-16: passed
VERDICT FUP-25: passed
VERDICT FUP-24: passed
