I read only `intent.md` and `screens/`. The screens come from earlier builds, and some were made against stand-ins, so they show what the pages look like, not that the feature works.

## 1. Questions about missing intent

1. **Moonshot spend:** how many Moonshot turns are allowed? *Assumption:* 2 turns per Kimi model, 4 in total (roughly $0.06–0.08). If fewer are allowed, FUP-19 and the second turn of FUP-20 are dropped and reported as not verified.
2. **Choosing route and model:** how does the tester pick the route and model at start (an argument, a `-c` setting or `/model`)? *Assumption:* the run brief gives the start arguments, and `/model` switches between models on the same route.
3. **Prices for OpenAI and Z.AI:** the packet has no rates for `gpt-6-luna` or `glm-5.3-flash`. *Assumption:* those costs are checked only for consistency (the parts add up to the total, the total equals the sum of the requests, and nothing is $0 when it should be unknown). Only the Moonshot prices are checked against published rates.
4. **Label for a review inside `/side`:** the intent says `side:code-review:`, but screen `s06` shows `Turn: side:review:` for an "automatic review". Are these different kinds of request? *Assumption:* `/review` inside `/side` must read `side:code-review:`, and `side:review:` belongs to a different automatic reviewer that this run does not test.
5. **`/review` inside `/side`:** is it allowed? *Assumption:* either the product refuses it clearly and sends nothing, or it runs and is labelled correctly.
6. **Using `/side`:** how is it opened and closed, and which conversation does `/cost` show from inside it? *Assumption:* the product's own on-screen hint closes it (otherwise Esc), and `/cost` is read after returning to the parent conversation.
7. **Editing an earlier message:** what keys do this? *Assumption:* with the composer empty, Esc then Esc opens the earlier-message picker, and Enter selects a message for editing.
8. **When the warm-up is sent:** is it sent as soon as the conversation reopens, or with the first turn after it? The `s08` notice appears after a turn. *Assumption:* one short turn is always sent after a reopen before checking.
9. **Warm-up from `/side`:** does it count in the parent's `Reopen warm-up: N` line, and what does its `Turn:` line say? *Assumption:* it counts under the parent and keeps the "Reopen warm-up" label; any `side:` prefix is acceptable.
10. **`/resume` of the conversation already open:** should it send a warm-up? Screen `s08-resume-same-conversation` shows the warm-up notice next to "Already viewing". *Assumption:* no warm-up should be sent (advisory). If one is sent, it must still be labelled and counted once.
11. **Where review requests are recorded:** must they be the conversation's own attempts, or may they be under "Descendant attempts"? *Assumption:* either is fine, as long as they are inside this conversation's total.
12. **Realtime conversation:** can the conversation the realtime driver used be opened with `resume <id>`, and on which route? If it is reopened on OpenAI, one extra warm-up is spent. *Assumption:* try it on the route the brief allows. If OpenAI, expect one separately labelled warm-up. If it can't be reopened, check from another conversation (FUP-23).
13. **Realtime spend:** the session's cost is "unknown" by design, so the $0.05 limit can't be checked in the product. *Assumption:* keeping the session to one short line is enough.
14. **No cache read on the second Kimi turn:** if `Cache read` is 0, is that a product failure or a provider-side miss? *Assumption:* the case fails unless the brief accepts a provider-side miss. The tester records the evidence line and retries only if spend allows.
15. **Provider name:** the rule "priced whatever the person named the provider" can't be tested, because the environment only has a provider named `kimi`. *Assumption:* not tested in this run.
16. **Config file:** can the tester edit `config.toml` in the profile? *Assumption:* no. Only `-c skip_reopen_warmup=true` is tested.
17. **Oddities in the old screens:** in `s08`, Request 1 and the warm-up both cost $0.001465, and there is no cache read after a warm-up that is meant to prime the cache. In `s06`, the technical page for the side request shows no cost line. *Assumption:* the tests check that Request IDs are distinct and that side requests show a price. A cache read after a warm-up is not required.
18. **UTC day boundary:** 17:00 in Phoenix is 00:00 UTC. *Assumption:* each group runs within one UTC day. Otherwise the tester uses `/cost YYYY-MM-DD` for both days.

## 2. Functional cases

**Profile groups (each starts a fresh profile, and the cases in a group share it in the order listed):**
- **Z:** `zai` route, `glm-5.3-flash`: FUP-01 to FUP-08.
- **O:** `openai` route, `gpt-6-luna`: FUP-09 to FUP-16, about 8 turns plus about 6 warm-ups, roughly $0.03.
- **M:** `moonshot` route (provider `kimi`): FUP-17 to FUP-20.
- **R:** realtime driver on the OpenAI key, then the TUI: FUP-21 to FUP-23.
- **All groups:** FUP-24 is checked throughout.

All turns send short text such as `Reply with the single word OK.` followed by Enter.

### Group Z (zai, glm-5.3-flash)

**FUP-01 — blocker — Basic `/cost` pages and navigation**
- **Start:** fresh profile Z.
- **Actions:**
  1. Send `Reply with the single word OK.` and wait for `OK`.
  2. Type `/cost` and press Enter. Press ↑ until the top is shown, then ↓ to the links.
  3. Press Enter on `→ Request 1`, then Enter on `→ Technical details`.
  4. Press Esc three times.
- **Expected:**
  - The first page is titled "Cost — this conversation" and shows today's UTC day.
  - There is one Z.AI · GLM 5.3 Flash line: "Pay per use. 1 request …", with a `$` estimate.
  - `Unknown parent population: 0 attempts`.
  - The Request page shows provider, model, `Billing: Pay per use`, cost, tokens and `Request: <id>`.
  - The technical page shows the same Request ID, plus `Attempt:`, `Thread:` (record it as X) and a `Turn:` with no `code-review:`, `side:` or `realtime:` prefix.
  - The component costs add up to the exact USD figure.
  - Each Esc goes back one page, and the last one closes `/cost` and returns to the composer.

**FUP-02 — advisory — Other dates**
- **Start:** after FUP-01.
- **Actions:**
  1. `/cost 2026-01-01`, then Esc.
  2. `/cost <today UTC>`, then Esc.
  3. `/cost 2026-13-40`.
- **Expected:**
  - The 2026-01-01 page says no requests were recorded, with no invented spend.
  - The page for today matches FUP-01.
  - The invalid date gives a clear error and nothing crashes.

**FUP-03 — blocker — `/review` is recorded under the conversation that ran it**
- **Start:** after FUP-01 (conversation X exists).
- **Actions:**
  1. `/new`, send `Reply with the single word OK.` and record this conversation's Thread as Y (from its technical page).
  2. Append the line `Anoter line for review.` to `README.md` in the workspace.
  3. Type `/review`, press Enter, and choose the uncommitted-changes option.
  4. Wait for `<< Code review finished >>`.
  5. Open `/cost`, then open every request and its technical page.
- **Expected:**
  - The session shows `>> Code review started: current changes <<` and a review result.
  - Y's request count is 1 plus K, where K ≥ 1 review requests.
  - Each review request has `Turn: code-review:…`, a distinct Request ID, is listed once, and its Thread is Y or appears under Y's "Descendant attempts".
  - The exact USD sum of the requests equals the group total and the known subtotal.
  - `Unknown parent population: 0 attempts` and `Unknown provider/model attribution (none)`.

**FUP-04 — blocker — Nothing from the review leaks into another conversation**
- **Start:** after FUP-03.
- **Actions:**
  1. Read the "other conversations" line on Y's first `/cost` page.
  2. `/resume`, pick conversation X, then `/cost`.
- **Expected:**
  - X still shows 1 request, with the same Request ID and exact cost as in FUP-01.
  - X has no `code-review:` turns.
  - Y's "other conversations" total for X is unchanged.

**FUP-05 — blocker — `/side` work is labelled and recorded under the parent**
- **Start:** after FUP-04 (in X).
- **Actions:**
  1. Note X's request count.
  2. Type `/side`, press Enter, send `Reply with the single word SIDE.` and wait for the reply.
  3. Close the side conversation, then open `/cost` and the new requests' technical pages.
- **Expected:**
  - X's count rises only by the side requests.
  - Each side request has `Turn: side:…`, Thread X (or a descendant of X), an estimated `$` cost (not blank or $0), and is listed once.
  - The side conversation does not appear as a separate entry under "other conversations".
  - Unknown parent population stays 0.

**FUP-06 — advisory — `/review` inside `/side`**
- **Start:** after FUP-05, with `README.md` still changed.
- **Actions:** `/side`, then `/review` and the uncommitted-changes option. Wait, close the side conversation, and open `/cost`.
- **Expected:** either:
  - a clear refusal and no new requests, or
  - the review's requests sit under X with `Turn: side:code-review:…`, are counted once and are in X's total. (`side:review:` should be flagged.)

**FUP-07 — blocker — History is unchanged after a restart (zai)**
- **Start:** after FUP-06. Write down every Request ID and exact cost in X.
- **Actions:**
  1. Stop, then start with `resume --last`, then `/cost` and open every request.
  2. Send `Reply with the single word OK.` and open `/cost` again.
- **Expected:**
  - After the restart, every earlier Request ID is present once with the same exact cost, and the count is unchanged.
  - After the turn, the count rises by exactly the new turn's requests, and earlier entries are unchanged.

**FUP-08 — advisory — No warm-up on a non-WebSocket route**
- **Start:** observed during FUP-07.
- **Actions:** check the session and the first `/cost` page after the restart.
- **Expected:** no warm-up notice and no `Reopen warm-up` line (or the line shows 0).

### Group O (openai, gpt-6-luna)

**FUP-09 — blocker — Restart warm-up is labelled, gives a notice and is counted once**
- **Start:** fresh profile O.
- **Actions:**
  1. Send `Reply with the single word OK.`, open `/cost` and record Request 1's ID and cost, and Thread T1.
  2. Stop, then start with `resume --last`.
  3. Send `Reply with the single word OK.`, then `/cost`. Open the warm-up's request and technical pages.
- **Expected:**
  - Before the restart there is no warm-up line or notice.
  - After the restart, a `⚠` notice naming `skip_reopen_warmup` appears once the warm-up has been sent.
  - The first page shows `Reopen warm-up: 1 request, <cost>`.
  - The link reads `→ Request n · Reopen warm-up · OpenAI · GPT-6 Luna …`, and the request page says "Kind: Reopen warm-up".
  - It has about 10k–15k input tokens and a `$` estimate, with Thread T1.
  - It is included in the group's count and total.

**FUP-10 — blocker — Request IDs and costs stay the same after every reopen (audit)**
- **Start:** group O, repeated after FUP-09, FUP-11, FUP-12, FUP-13, FUP-14 and FUP-16.
- **Actions:** in `/cost` for T1, open every request link and record its Request ID and exact USD. Compare with the previous audit.
- **Expected:**
  - Every earlier ID is present exactly once with the same cost.
  - The only new entries are the expected warm-ups and turns.
  - No Request ID appears twice.
  - The exact USD sum of the requests equals the group total.

**FUP-11 — blocker — Warm-up on `resume <id>`**
- **Actions:** stop, start with `resume <T1>`, send a turn, then `/cost`.
- **Expected:** the warm-up count rises by 1, the notice is shown, and the FUP-10 audit passes.

**FUP-12 — blocker — Warm-up on `/resume` from the picker**
- **Actions:**
  1. `/new` and send a turn (this is T2).
  2. `/resume`, pick T1 and press Enter, send a turn, then `/cost`.
- **Expected:**
  - T1's warm-up count rises by 1 and the notice is shown.
  - T2 holds only its own turn, as shown in T1's "other conversations" line.
  - The FUP-10 audit passes.

**FUP-13 — blocker — Warm-up on `/side`, and side labels on OpenAI**
- **Actions:** in T1, `/side`, send `Reply with the single word OK.`, close the side conversation, then `/cost`.
- **Expected:**
  - T1 gains one labelled warm-up and the side turn(s), with `Turn: side:…`.
  - Everything is inside T1's total, and the FUP-10 audit passes.

**FUP-14 — blocker — Editing an earlier message: warm-up, and the old requests still listed**
- **Actions:**
  1. With the composer empty, press Esc, Esc, select the earliest message and press Enter.
  2. Replace the text with `Reply with the single word YES.` and press Enter.
  3. Open `/cost`.
- **Expected:**
  - The reply is `YES`, the warm-up count rises by 1 and the notice is shown.
  - Requests from turns that were rolled back are still listed once, with the same IDs and costs.

**FUP-15 — advisory — `/resume` of the conversation already open**
- **Actions:** in T1, `/resume`, pick T1, then `/cost`.
- **Expected:** "Already viewing", no new warm-up and no notice. If a warm-up is sent anyway, it must be labelled and counted once.

**FUP-16 — blocker — `skip_reopen_warmup = true`**
- **Actions:**
  1. Stop, then start with `-c skip_reopen_warmup=true resume --last`.
  2. Send a turn, then `/side`, send a turn, close the side conversation, then `/cost`.
- **Expected:**
  - No warm-up notice and no change in the warm-up count.
  - Only the requests for the turns are added.
  - The turns work normally, and the FUP-10 audit passes.

### Group M (moonshot, provider `kimi`)

**FUP-17 — blocker — First `kimi-k3` turn priced at published rates**
- **Actions:** send `Reply with the single word OK.`, then open `/cost` and the technical page.
- **Expected:**
  - `Provider: kimi`, `Model: kimi-k3`, `Billing: Pay per use`.
  - Rates shown are 3 / 0.3 / 3 / 15.
  - Uncached + cache read + cache write = Input.
  - Exact USD = (U×3 + R×0.30 + W×3 + O×15) / 1e6.
  - Nothing is marked unknown, incomplete or a lower bound.

**FUP-18 — blocker — Second `kimi-k3` turn reads the cache, priced at 0.30**
- **Actions:** within 5 minutes of FUP-17, send `Reply with the single word OK again.`, then `/cost`.
- **Expected:**
  - Request 2 shows `Cache read` > 0, with a cache-read cost of R × 0.30 / 1e6.
  - The input parts add up, and the formula holds.
  - Request 1's ID and cost are unchanged.
  - If R is 0, record the evidence line (see question 14).

**FUP-19 — blocker — `kimi-k2.7-code` turns, cache read at 0.19 and cache write priced as input**
- **Actions:**
  1. `/new`, then `/model kimi-k2.7-code`.
  2. Send two turns within 5 minutes, then `/cost`.
- **Expected:**
  - Rates shown are 0.95 / 0.19 / 0.95 / 4.
  - Turn 2 has R > 0.
  - Exact USD = (U×0.95 + R×0.19 + W×0.95 + O×4) / 1e6.
  - The parts add up to Input.

**FUP-20 — blocker — Prices are complete and the conversations are kept apart**
- **Actions:** in the `k2.7` conversation's first `/cost` page, read the "other conversations" line and the details. Use `/resume` to return to the k3 conversation and repeat.
- **Expected:**
  - Each conversation lists only its own 2 requests.
  - The other conversation's total equals its own total.
  - No "incomplete", "lower bound" or "unavailable" estimate appears.
  - Earlier IDs and costs are unchanged.

### Group R (realtime)

**FUP-21 — blocker — Realtime session is one request, labelled, with unknown (not zero) cost**
- **Actions:**
  1. Note the UTC time before and after running the realtime driver with `Reply with the single word OK.`. Record the reply and the thread ID (TR).
  2. Start with `resume <TR>`, then `/cost`. Open the realtime request and its technical page.
- **Expected:**
  - The first page shows `Realtime: 1 session; token counts and cost unknown, not zero`.
  - The link reads `Request n · Realtime session · OpenAI · … · no price available`.
  - The request page says "Kind: Realtime session — started t1, ended t2", with t1 < t2 and both inside the noted window.
  - "Tokens: tokens not reported".
  - The technical page shows `Turn: realtime:…` and Thread TR, and every token and cost field is unknown.
  - There is no `$` figure and no 0 for this request anywhere.
  - The conversation total is not shown as complete.
  - Any warm-up is a separate, labelled request.

**FUP-22 — advisory — Realtime seen from another conversation**
- **When:** if `resume <TR>` fails, or as an extra check.
- **Actions:** start a new conversation on zai, send one turn, then `/cost`.
- **Expected:** the "other conversations" line for TR shows no `$` figure and no $0 for the session, and says its cost is unknown.

**FUP-23 — advisory — Realtime record is stable after a restart**
- **Actions:** stop, start with `resume <TR>` again, then `/cost`.
- **Expected:** still 1 session, with the same Request ID and start/end times. Any extra warm-up is separate.

### All groups

**FUP-24 — blocker — No credential is printed**
- **Actions:** check every screen, every `/cost` page, the realtime driver's output and any `corbanu exec` output for `sk-`, `Bearer`, `Authorization`, `api_key=` or long secret-like strings that are not UUIDs.
- **Expected:** none found.

## 3. JSON

```json
{"questions":[
"Moonshot spend answer: how many Kimi turns are allowed? Assumption: 2 per model (4 total, about $0.06-0.08); if fewer, FUP-18 and FUP-19 turn 2 are reported as not verified.",
"How are the route and model chosen at start (argument, -c setting or /model)? Assumption: the run brief gives start arguments; /model switches models on the same route.",
"No rates are given for gpt-6-luna or glm-5.3-flash. Assumption: OpenAI and Z.AI costs are checked only for consistency (parts add up, total equals sum of requests, unknown is never $0); only Moonshot is checked against published rates.",
"Screen s06 labels a side automatic review 'side:review:' but the intent says 'side:code-review:'. Assumption: /review inside /side must read 'side:code-review:'; 'side:review:' is a different automatic reviewer and is not tested.",
"Is /review allowed inside /side? Assumption: either a clear refusal that sends nothing, or correct labelling.",
"How is /side opened and closed, and which conversation does /cost show inside it? Assumption: close it with the product's own hint (otherwise Esc) and read /cost in the parent.",
"What keys edit an earlier message? Assumption: Esc, Esc with an empty composer, then Enter to choose a message.",
"Is the warm-up sent as soon as the conversation reopens, or with the first turn after it? Assumption: one short turn is always sent after a reopen before checking.",
"Does a /side warm-up count in the parent's 'Reopen warm-up: N' line, and what is its Turn prefix? Assumption: yes, under the parent; a side: prefix is acceptable.",
"Should /resume of the conversation already open send a warm-up (s08 shows one)? Assumption: no (advisory); if sent, it must be labelled and counted once.",
"Must review requests be the conversation's own attempts, or may they be descendant attempts? Assumption: either, as long as they are inside the conversation's total.",
"Can the realtime conversation be opened with resume <id>, and on which route? Assumption: try on the route the brief allows; on OpenAI one separate labelled warm-up is expected; otherwise check from another conversation (FUP-22).",
"The realtime $0.05 limit can't be checked in the product because its cost is unknown by design. Assumption: a short session is enough.",
"If the second Kimi turn shows Cache read 0, is that a product failure or a provider-side miss? Assumption: the case fails unless the brief accepts a miss; record the evidence line and retry only within spend.",
"Pricing regardless of provider name can't be tested because only a provider named 'kimi' exists. Assumption: not tested.",
"Can the tester edit config.toml? Assumption: no; only -c skip_reopen_warmup=true is tested.",
"Old screens show oddities (s08: Request 1 and the warm-up both cost $0.001465, and no cache read after the warm-up; s06: no cost line on the side request). Assumption: Request IDs must be distinct and side requests must show a price; a cache read after a warm-up is not required.",
"17:00 in Phoenix is 00:00 UTC. Assumption: each group runs within one UTC day; otherwise use /cost YYYY-MM-DD for both days."],
"cases":[
{"id":"FUP-01","priority":"blocker","group":"Z","starting_conditions":"Fresh profile Z; zai route; glm-5.3-flash; small git workspace.","actions":["Send 'Reply with the single word OK.' and wait for 'OK'.","Type '/cost', press Enter; press Up until the top is shown, then Down to the links.","Press Enter on '→ Request 1', then Enter on '→ Technical details'.","Press Esc three times."],"expected":["First page titled 'Cost — this conversation' for today's UTC day; one Z.AI GLM 5.3 Flash line: 'Pay per use. 1 request', with a $ estimate.","'Unknown parent population: 0 attempts'.","Request page shows provider, model, 'Billing: Pay per use', cost, tokens and 'Request: <id>'.","Technical page has the same Request ID, plus Attempt, Thread (record as X) and a Turn with no code-review:, side: or realtime: prefix; component costs add up to the exact USD.","Each Esc goes back one page; the last closes /cost and returns to the composer."]},
{"id":"FUP-02","priority":"advisory","group":"Z","starting_conditions":"After FUP-01.","actions":["'/cost 2026-01-01', then Esc.","'/cost <today UTC>', then Esc.","'/cost 2026-13-40'."],"expected":["2026-01-01 says no requests recorded, with no invented spend.","The page for today matches FUP-01.","The invalid date gives a clear error; nothing crashes."]},
{"id":"FUP-03","priority":"blocker","group":"Z","starting_conditions":"After FUP-01 (conversation X exists); zai; glm-5.3-flash.","actions":["'/new', send 'Reply with the single word OK.'; record this conversation's Thread as Y from its technical page.","Append the line 'Anoter line for review.' to README.md in the workspace.","Type '/review', press Enter, choose the uncommitted-changes option.","Wait for '<< Code review finished >>'.","Open '/cost', then every request and its technical page."],"expected":["Session shows '>> Code review started: current changes <<' and a review result.","Y's request count is 1 + K, K >= 1 review requests.","Each review request: 'Turn: code-review:', distinct Request ID, listed once; Thread is Y or under Y's 'Descendant attempts'.","Exact USD sum of requests equals the group total and the known subtotal.","'Unknown parent population: 0 attempts'; 'Unknown provider/model attribution (none)'."]},
{"id":"FUP-04","priority":"blocker","group":"Z","starting_conditions":"After FUP-03.","actions":["Read the 'other conversations' line on Y's first /cost page.","'/resume', pick conversation X, then '/cost'."],"expected":["X still shows 1 request, same Request ID and exact cost as FUP-01.","X has no code-review: turns.","Y's 'other conversations' total for X is unchanged."]},
{"id":"FUP-05","priority":"blocker","group":"Z","starting_conditions":"After FUP-04, in X; note X's request count.","actions":["'/side', press Enter, send 'Reply with the single word SIDE.' and wait.","Close the side conversation.","'/cost'; open the new requests' technical pages."],"expected":["X's count rises only by the side requests.","Each side request: 'Turn: side:', Thread X or a descendant of X, an estimated $ cost (not blank or $0), listed once.","The side conversation is not a separate 'other conversation'.","Unknown parent population stays 0."]},
{"id":"FUP-06","priority":"advisory","group":"Z","starting_conditions":"After FUP-05, with README.md still changed.","actions":["'/side', then '/review' and the uncommitted-changes option; wait.","Close the side conversation; '/cost'."],"expected":["Either a clear refusal and no new requests, or the review's requests sit under X with 'Turn: side:code-review:', counted once and in X's total ('side:review:' should be flagged)."]},
{"id":"FUP-07","priority":"blocker","group":"Z","starting_conditions":"After FUP-06; write down every Request ID and exact cost in X.","actions":["Stop; start with 'resume --last'; '/cost'; open every request.","Send 'Reply with the single word OK.'; '/cost' again."],"expected":["After the restart, every earlier Request ID is present once with the same exact cost; count unchanged.","After the turn, the count rises only by the new turn's requests; earlier entries unchanged."]},
{"id":"FUP-08","priority":"advisory","group":"Z","starting_conditions":"Observed during FUP-07.","actions":["Check the session and the first /cost page after the restart."],"expected":["No warm-up notice and no 'Reopen warm-up' line (or it shows 0)."]},
{"id":"FUP-09","priority":"blocker","group":"O","starting_conditions":"Fresh profile O; openai route; gpt-6-luna.","actions":["Send 'Reply with the single word OK.'; '/cost'; record Request 1's ID and cost, and Thread T1.","Stop; start with 'resume --last'.","Send 'Reply with the single word OK.'; '/cost'; open the warm-up's request and technical pages."],"expected":["Before the restart: no warm-up line or notice.","After the restart: a '⚠' notice naming skip_reopen_warmup appears once the warm-up is sent.","First page: 'Reopen warm-up: 1 request, <cost>'.","Link '→ Request n · Reopen warm-up · OpenAI · GPT-6 Luna'; request page 'Kind: Reopen warm-up'.","About 10k-15k input tokens; $ estimate; Thread T1.","Included in the group's count and total."]},
{"id":"FUP-10","priority":"blocker","group":"O","starting_conditions":"Group O; repeated after FUP-09, 11, 12, 13, 14 and 16.","actions":["In T1's /cost, open every request link; record Request ID and exact USD; compare with the previous audit."],"expected":["Every earlier ID is present exactly once with the same cost.","Only the expected warm-ups and turns are new.","No Request ID appears twice.","Exact USD sum of requests equals the group total."]},
{"id":"FUP-11","priority":"blocker","group":"O","starting_conditions":"After FUP-09.","actions":["Stop; start with 'resume <T1>'.","Send 'Reply with the single word OK.'; '/cost'."],"expected":["Warm-up count +1; notice shown.","FUP-10 audit passes."]},
{"id":"FUP-12","priority":"blocker","group":"O","starting_conditions":"After FUP-11.","actions":["'/new'; send 'Reply with the single word OK.' (T2).","'/resume', pick T1, press Enter.","Send 'Reply with the single word OK.'; '/cost'."],"expected":["T1's warm-up count +1; notice shown.","T2 holds only its own turn (T1's 'other conversations' line).","FUP-10 audit passes."]},
{"id":"FUP-13","priority":"blocker","group":"O","starting_conditions":"After FUP-12, in T1.","actions":["'/side'; send 'Reply with the single word OK.'.","Close the side conversation; '/cost'."],"expected":["T1 gains one labelled warm-up and the side turn(s) with 'Turn: side:'.","Everything inside T1's total; FUP-10 audit passes."]},
{"id":"FUP-14","priority":"blocker","group":"O","starting_conditions":"After FUP-13, in T1, composer empty.","actions":["Press Esc, Esc; select the earliest message; press Enter.","Replace with 'Reply with the single word YES.' and press Enter.","'/cost'."],"expected":["Reply is YES; warm-up count +1; notice shown.","Requests from rolled-back turns are still listed once, with the same IDs and costs."]},
{"id":"FUP-15","priority":"advisory","group":"O","starting_conditions":"After FUP-14, in T1.","actions":["'/resume'; pick T1; '/cost'."],"expected":["'Already viewing'; no new warm-up and no notice; if one is sent, it is labelled and counted once."]},
{"id":"FUP-16","priority":"blocker","group":"O","starting_conditions":"After FUP-15.","actions":["Stop; start with '-c skip_reopen_warmup=true resume --last'.","Send 'Reply with the single word OK.'.","'/side'; send 'Reply with the single word OK.'; close the side conversation; '/cost'."],"expected":["No warm-up notice; warm-up count unchanged.","Only the turns' requests are added; turns work normally.","FUP-10 audit passes."]},
{"id":"FUP-17","priority":"blocker","group":"M","starting_conditions":"Fresh profile M; moonshot route (provider 'kimi' at api.moonshot.ai/v1); kimi-k3.","actions":["Send 'Reply with the single word OK.'.","'/cost'; open the technical page."],"expected":["'Provider: kimi', 'Model: kimi-k3', 'Billing: Pay per use'.","Rates shown: 3 / 0.3 / 3 / 15.","Uncached + cache read + cache write = Input.","Exact USD = (U*3 + R*0.30 + W*3 + O*15)/1e6.","Nothing marked unknown, incomplete or a lower bound."]},
{"id":"FUP-18","priority":"blocker","group":"M","starting_conditions":"Within 5 minutes of FUP-17, same conversation.","actions":["Send 'Reply with the single word OK again.'.","'/cost'; open Request 2's technical page."],"expected":["Cache read > 0; cache-read cost = R*0.30/1e6.","Input parts add up; formula holds.","Request 1's ID and cost unchanged.","If R = 0, record the evidence line (see questions)."]},
{"id":"FUP-19","priority":"blocker","group":"M","starting_conditions":"After FUP-18.","actions":["'/new'; '/model kimi-k2.7-code'.","Send two turns within 5 minutes ('Reply with the single word OK.', then 'Reply with the single word OK again.').","'/cost'; open both technical pages."],"expected":["Rates shown: 0.95 / 0.19 / 0.95 / 4.","Turn 2 has R > 0.","Exact USD = (U*0.95 + R*0.19 + W*0.95 + O*4)/1e6; parts add up to Input."]},
{"id":"FUP-20","priority":"blocker","group":"M","starting_conditions":"After FUP-19.","actions":["Read the k2.7 conversation's first /cost page: 'other conversations' line and details.","'/resume' to the k3 conversation; '/cost'; repeat."],"expected":["Each conversation lists only its own 2 requests.","The other conversation's total equals its own total.","No 'incomplete', 'lower bound' or 'unavailable' estimate.","Earlier IDs and costs unchanged."]},
{"id":"FUP-21","priority":"blocker","group":"R","starting_conditions":"Fresh profile R; OpenAI route's API key; realtime driver available.","actions":["Note UTC time; run the realtime driver with 'Reply with the single word OK.'; note UTC time; record the reply and thread ID TR.","Start with 'resume <TR>'; '/cost'; open the realtime request and its technical page."],"expected":["First page: 'Realtime: 1 session; token counts and cost unknown, not zero'.","Link 'Request n · Realtime session · OpenAI · … · no price available'.","Request page 'Kind: Realtime session — started t1, ended t2'; t1 < t2, both in the noted window; 'Tokens: tokens not reported'.","Technical page: 'Turn: realtime:', Thread TR; all token and cost fields unknown.","No $ figure and no 0 for this request anywhere; conversation total not shown as complete.","Any warm-up is a separate, labelled request."]},
{"id":"FUP-22","priority":"advisory","group":"R","starting_conditions":"After the realtime run, if 'resume <TR>' fails (or as an extra check).","actions":["Start a new conversation on zai; send 'Reply with the single word OK.'; '/cost'."],"expected":["The 'other conversations' line for TR shows no $ figure and no $0 for the session; its cost is said to be unknown."]},
{"id":"FUP-23","priority":"advisory","group":"R","starting_conditions":"After FUP-21, if resume works.","actions":["Stop; start with 'resume <TR>'; '/cost'."],"expected":["Still 1 session; same Request ID and start/end times; any extra warm-up separate."]},
{"id":"FUP-24","priority":"blocker","group":"All","starting_conditions":"Throughout groups Z, O, M and R.","actions":["Check every screen, every /cost page, the realtime driver's output and any corbanu exec output for 'sk-', 'Bearer', 'Authorization', 'api_key=' or long secret-like non-UUID strings."],"expected":["No credential or key material appears anywhere."]}
]}
```