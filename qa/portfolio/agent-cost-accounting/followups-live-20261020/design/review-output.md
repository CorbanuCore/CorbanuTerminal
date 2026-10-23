**VERDICT: CHANGES**

The design covers all five behaviours and the four standing rules, and the group plan is sound. It can't be run reliably as written, for these reasons:

- **Moonshot:** the group isn't gated on `d3.txt`. It allows a retry, which the integrator forbids. It changes models with `/model`, which may not work on that route; the integrator's answer is to stop and start with `--model`.
- **`/side`:** cases leave it with Esc, which the integrator says doesn't work. Only one Ctrl+C does.
- **Picking conversations:** `/resume` picker choices are ambiguous, because every conversation starts with the same "OK" message.
- **Expected results not backed by `intent.md` or the owner decisions:**
  - FUP-15 expects no warm-up when resuming the open conversation. The intent doesn't say so, and the old screen shows one.
  - Tests demand "10k–15k tokens", but the intent only says "typically".
  - FUP-14 requires the reply to be "YES".
  - Several cases demand exactly "1 request" or "2 requests", but one turn can make more than one request.
  - "No 'unavailable' anywhere" would fail on every technical page, because the line "wall time: unavailable" is always there.
  - "No 0 anywhere" would fail on IDs and on "0 attempts".
- **Too weak or too loose:**
  - D25 needs a captured screen showing `side:`, and no case asks for one.
  - The `config.toml` way of setting `skip_reopen_warmup` is untested. The designer assumed the profile's config can't be edited, but `agms-tui config` can.
  - The realtime case reopens on OpenAI, which spends an extra warm-up.
  - "Cache read 0" has no blocked outcome that avoids a retry.
- **Unreliable verdicts:** there's no Refresh rule for the ledger lag, and the Group O audits are too heavy for the 80-minute budget.
- **Screen oddity:** the designer read s06 as having no cost line. That page is just cropped, so it's not evidence of a missing price.

**Coverage:**

| What | Cases |
| --- | --- |
| Code review attribution | FUP-03, 04, 06 |
| `side:` labels | FUP-05, 13 |
| Warm-up | FUP-09, 11–16, plus new FUP-25 |
| Request-ID stability | FUP-07, 10 |
| Moonshot | FUP-17–20, gated |
| Realtime | FUP-21–23 |

The standing rules are covered by FUP-21 (unknown, not zero), FUP-10 (nothing counted twice), FUP-07/10 (history never re-priced) and FUP-24 (no credentials).

**Named limits, which are acceptable:**
- Pricing "whatever the provider is named" can't be tested, because only `kimi` exists.
- The realtime $0.05 limit is checked from the relay log, not in the product.
- If the realtime conversation can't be reopened, FUP-21 is blocked and FUP-22 gives partial evidence.

## Amendments

1. **ALL — starting conditions (add).** "Ledger lag: if a request just sent is missing from `/cost`, choose `Refresh` once (or close and reopen `/cost`). Wait up to 30 s before judging. Record UTC times for every turn, reopen and realtime run." Why: the integrator says `/cost` can read the ledger before the latest request is written.
2. **ALL — starting conditions (add).** "Save every screen a verdict rests on (`agms-tui screen > <group>-<case>-<n>.txt`) in the group's evidence folder." Why: D25 needs captured screens, and FUP-24 needs material to search.
3. **FUP-01 — expected (replace the 2nd bullet).** "One Z.AI · GLM 5.3 Flash line reading 'Pay per use' with N request(s), N ≥ 1. Record N and every Request ID; later cases compare against N, not 1. Its cost is a $ estimate, or 'no price' with a stated reason, never $0 for a request with tokens." Why: one turn can make more than one request, and the packet gives no Z.AI rate.
4. **FUP-03 — actions (replace steps 1, 2 and 4).**
   - Step 1: "`/new`; send `Reply with the single word YANKEE.`; record this conversation's Thread as Y and its request count as N_Y."
   - Step 2: "`printf 'Anoter line for review.\n' >> \"$AGMS_RUN/fixture/workspace/README.md\"`."
   - Step 4: "Wait for the review to finish (`<< Code review finished >>` or the product's own finish marker), allowing up to 15 minutes."

   Why: a distinct first message makes the conversation easy to pick in `/resume` later; the integrator gave the real workspace path; Z.AI is slow (1.8 tok/s on screen).
5. **FUP-03 — expected (replace the 2nd bullet, add one).**
   - Replace: "Y's request count is N_Y + K, with K ≥ 1 review requests."
   - Add: "Save one screen of a review request's technical page with its `Turn: code-review:` line visible."

   Why: avoids assuming one request per turn; D25.
6. **FUP-04 — actions and expected.**
   - Actions, step 2 (replace): "`/resume`, pick X (the conversation whose first message is the `OK` one, not `YANKEE`), then `/cost`."
   - Expected (replace): "X shows the same N requests as in FUP-01, with the same Request IDs and exact costs; X has no `code-review:` turns; X's 'other conversations' total equals Y's own total from FUP-03; Y's 'other conversations' total for X equals X's FUP-01 total."

   Why: the picker choice was ambiguous, and comparing the two totals shows that every review request is counted under Y.
7. **FUP-05 — actions (replace step 3).** "Leave the side conversation with exactly one Ctrl+C (`agms-tui key C-c`), and wait for the product's message that it has returned to the saved conversation. Never press Ctrl+C twice. Then `/cost`, and open the new requests' technical pages." Why: answer 6 says Esc doesn't leave `/side`; a second Ctrl+C may quit.
8. **FUP-05 — expected (replace the 2nd bullet, add one).**
   - Replace: "Each side request has a `Turn:` line starting `side:`, Thread X (or listed under X's 'Descendant attempts'), a cost shown as a $ estimate or 'no price' with a reason (never $0 for a request with tokens), and appears once."
   - Add: "Save one screen with the `Turn: side:` line visible (D25)."

   Why: the 'missing cost' oddity comes from a cropped page; D25.
9. **FUP-06 — actions (replace).** "`/side`; `/review` with the uncommitted-changes option; wait up to 15 minutes; leave with exactly one Ctrl+C and wait for the return message; then `/cost`." Why: the same `/side` control as amendment 7.
10. **FUP-07 — actions (replace step 1).** "Stop; start with `resume <X>` (X's Thread ID); then `/cost`, and open every request." Why: `--last` is ambiguous after `/side` and `/resume`.
11. **FUP-09 — expected (replace bullets 2, 4 and 5).**
    - "After the restart, the session shows a notice containing `skip_reopen_warmup` (before or after the first turn)."
    - "One request link contains `Reopen warm-up`, and its request page shows `Kind: Reopen warm-up`."
    - "Its token count is positive and its cost is a $ estimate (record the input tokens; 10k–15k is typical, not required); Thread T1."

    Why: the old expectations relied on the ⚠ glyph, model display names and a "typically" figure.
12. **FUP-10 — starting conditions and expected.**
    - Starting conditions (replace): "Group O; run after FUP-09, FUP-12, FUP-14 and FUP-25 (the final audit). Each audit records every Request ID and the cost from the request page. The final audit also opens each technical page for the exact USD."
    - Expected (replace the 2nd bullet): "New entries appear only after a new turn or a reopen; every warm-up is labelled."

    Why: fits the 80-minute budget, and doesn't assume one request per turn.
13. **FUP-12 — actions (replace).** "`/new`; send `Reply with the single word SECOND.` (T2). `/resume`; pick T1 (first message `OK`, not `SECOND`); Enter. Send `Reply with the single word OK.`; `/cost`." Why: the picker choice was ambiguous.
14. **FUP-13 — actions (replace).** "In T1: `/side`; send `Reply with the single word OK.`; leave with exactly one Ctrl+C and wait for the return message; `/cost`." Why: the `/side` control.
15. **FUP-14 — actions and expected.**
    - Actions, step 1 (replace): "Edit the earliest message using the product's own on-screen hint (try Esc twice in an empty composer first). If no edit control is found within 5 minutes, mark the case blocked."
    - Expected, 1st bullet (replace): "A reply arrives (YES is not required), the warm-up count rises by 1, and a notice containing `skip_reopen_warmup` is shown."

    Why: the edit keys are an assumption (answer 7), and the reply is outside the intent.
16. **FUP-15 — expected (replace).** "Record whether a warm-up and notice occurred. If a warm-up was sent, it is labelled `Reopen warm-up`, counted once, and the notice appears. The FUP-10 audit is unaffected." Why: the intent doesn't say whether resuming the open conversation sends a warm-up.
17. **FUP-16 — actions (replace).** "Stop; start with `-c skip_reopen_warmup=true resume <T1>` (if the argument order is rejected, use `resume <T1> -c skip_reopen_warmup=true`; an argument error is not a product failure). Send a turn; `/side`; send a turn; leave with one Ctrl+C; `/cost`." Why: a deterministic conversation and the `/side` control.
18. **New case FUP-25 (advisory, Group O, last).**
    - Start: after FUP-16.
    - Actions: `agms-tui config 'skip_reopen_warmup = true'`; stop; start with `resume <T1>` and no `-c`; send `Reply with the single word OK.`; `/cost`.
    - Expected: no notice; warm-up count unchanged; only the turn's requests added.

    Why: the intent names the config file as a way to set this, and it can't be undone, so it must run last.
19. **FUP-17 — starting conditions, actions, expected.**
    - Starting conditions (replace): "Read `d3.txt` first. If it grants no Moonshot turns, do not start Group M; report FUP-17–20 as 'not run (D54 not granted)'. Otherwise start fresh profile M with `agms-tui start --route moonshot --model kimi-k3`. Never retry a Moonshot turn."
    - Actions (replace): "Send `Reply with the single word OK.`. As soon as it replies, send FUP-18's turn, then open `/cost` and both technical pages."
    - Expected, last bullet (replace): "No cost or token field of these requests is unknown, incomplete or a lower bound. A cache count shown as 0 is acceptable. Metadata lines such as 'wall time: unavailable' are not cost fields."

    Why: D3/D54 gating; the 5-minute cache lifetime; the wording was robust against the screens' metadata lines.
20. **FUP-18 — expected (replace the last bullet).** "If Cache read is 0, record the evidence line and mark the case blocked ('no cache read observed'), not failed, and do not retry. If the relay answers `test harness: Moonshot request limit reached`, mark it blocked. Fail only if a cache read is shown but priced wrongly, or the parts don't add up." Why: no retries allowed; the tester can't tell a provider-side miss from a product fault.
21. **FUP-19 — actions and expected.**
    - Actions (replace): "Stop; start with `agms-tui start --route moonshot --model kimi-k2.7-code` (a new conversation). Send `Reply with the single word OK.`, then immediately `Reply with the single word OK again.`. Then `/cost` and both technical pages."
    - Expected (add): "Cache read 0 or the limit message: handle as in FUP-18."

    Why: answer 2 (model change by restart).
22. **FUP-20 — actions and expected.**
    - Actions, step 2 (replace): "Stop; start with `--route moonshot --model kimi-k3 resume <k3 Thread>`; send no turn; `/cost`."
    - Expected (replace bullets 1 and 3): "Each conversation lists only the requests from its own two turns", and "No cost field or total is unknown, incomplete, unavailable or a lower bound."

    Why: no extra spend; one turn can be several requests; the "unavailable" wording.
23. **FUP-21 — actions and expected.**
    - Actions (replace): "Run the realtime command once. Rerun it only if the first connection fails with a harness connection error before any reply, and never a third time. Record the UTC window, the reply and TR. Start with `--route zai --model glm-5.3-flash resume <TR>`, send no turn, `/cost`, and open the realtime request and its technical page."
    - Expected (replace the 3rd and 6th bullets): "t1 < t2, both within the noted window ±60 s", and "No $ amount, and no token count or cost of 0, is shown for this request; any total that includes it is marked unknown or incomplete."

    Why: D16/answer 13; resuming on Z.AI avoids an OpenAI warm-up; clock skew; IDs contain zeros.
24. **FUP-22 — expected (replace).** "The other-conversations total covering TR shows no $0 for the session (it is unknown or 'no price')." Why: the intent doesn't specify that line's wording.
25. **FUP-24 — actions (replace).** "At the end of each group, search all saved screens and command outputs for `sk-`, `Bearer`, `Authorization`, `api_key`, the synthetic `.env` value, and long non-UUID tokens. Report a hit inside the transcript of a command the model ran separately from a hit on a product page or notice." Why: makes the check concrete and separates model-run commands from product output.

```json
{"verdict":"CHANGES","amendments":[
{"n":1,"case":"ALL","field":"starting conditions","change":"add","text":"Ledger lag: if a request just sent is missing from /cost, choose Refresh once (or close and reopen /cost) and wait up to 30 s before judging. Record UTC times for every turn, reopen and realtime run.","why":"The integrator says /cost can read before the latest request is written."},
{"n":2,"case":"ALL","field":"starting conditions","change":"add","text":"Save every screen a verdict rests on (agms-tui screen > <group>-<case>-<n>.txt) in the group's evidence folder.","why":"D25 needs captured screens; FUP-24 needs material to search."},
{"n":3,"case":"FUP-01","field":"expected","change":"replace","text":"One Z.AI · GLM 5.3 Flash line reading 'Pay per use' with N request(s), N >= 1 (record N and every Request ID; later cases compare against N, not 1); its cost is a $ estimate or 'no price' with a stated reason, never $0 for a request with tokens.","why":"One turn can make several requests; the packet gives no Z.AI rate."},
{"n":4,"case":"FUP-03","field":"actions","change":"replace","text":["'/new'; send 'Reply with the single word YANKEE.'; record this conversation's Thread as Y and its request count as N_Y.","printf 'Anoter line for review.\\n' >> \"$AGMS_RUN/fixture/workspace/README.md\"","Type '/review', press Enter, choose the uncommitted-changes option.","Wait for the review to finish ('<< Code review finished >>' or the product's own finish marker), allowing up to 15 minutes.","Open '/cost', then every request and its technical page."],"why":"A distinct first message for the picker later; the real workspace path; Z.AI is slow."},
{"n":5,"case":"FUP-03","field":"expected","change":"replace","text":["Session shows a code-review start marker and a review result.","Y's request count is N_Y + K, with K >= 1 review requests.","Each review request: 'Turn: code-review:', distinct Request ID, listed once; Thread is Y or under Y's 'Descendant attempts'.","Exact USD sum of requests equals the known subtotal.","'Unknown parent population: 0 attempts'; 'Unknown provider/model attribution (none)'.","Save one screen of a review request's technical page with its 'Turn: code-review:' line visible."],"why":"Avoids assuming one request per turn; D25."},
{"n":6,"case":"FUP-04","field":"actions and expected","change":"replace","text":{"actions":["Read the 'other conversations' line on Y's first /cost page.","'/resume', pick X (first message 'OK', not 'YANKEE'), then '/cost'."],"expected":["X shows the same N requests as in FUP-01, with the same Request IDs and exact costs.","X has no code-review: turns.","X's 'other conversations' total equals Y's own total from FUP-03.","Y's 'other conversations' total for X equals X's FUP-01 total."]},"why":"The picker choice was ambiguous; comparing totals shows every review request is counted under Y."},
{"n":7,"case":"FUP-05","field":"actions","change":"replace","text":["'/side', press Enter, send 'Reply with the single word SIDE.' and wait.","Leave with exactly one Ctrl+C (agms-tui key C-c) and wait for the product's message that it returned to the saved conversation; never press Ctrl+C twice.","'/cost'; open the new requests' technical pages."],"why":"Answer 6: Esc does not leave /side; a second Ctrl+C may quit."},
{"n":8,"case":"FUP-05","field":"expected","change":"replace","text":["X's count rises only by the side requests.","Each side request: Turn line starting 'side:', Thread X (or under X's 'Descendant attempts'), a cost shown as a $ estimate or 'no price' with a reason (never $0 for a request with tokens), listed once.","The side conversation is not a separate 'other conversation'.","Unknown parent population stays 0.","Save one screen with the 'Turn: side:' line visible (D25)."],"why":"The missing-cost oddity comes from a cropped page; D25."},
{"n":9,"case":"FUP-06","field":"actions","change":"replace","text":["'/side'; '/review' with the uncommitted-changes option; wait up to 15 minutes.","Leave with exactly one Ctrl+C and wait for the return message; '/cost'."],"why":"The /side control."},
{"n":10,"case":"FUP-07","field":"actions","change":"replace","text":["Stop; start with 'resume <X>' (X's Thread ID); '/cost'; open every request.","Send 'Reply with the single word OK.'; '/cost' again."],"why":"'--last' is ambiguous after /side and /resume."},
{"n":11,"case":"FUP-09","field":"expected","change":"replace","text":["Before the restart: no warm-up line or notice.","After the restart the session shows a notice containing 'skip_reopen_warmup' (before or after the first turn).","First page: 'Reopen warm-up: 1 request' with a $ cost.","One request link contains 'Reopen warm-up'; its request page shows 'Kind: Reopen warm-up'.","Positive token count and a $ estimate (record input tokens; 10k-15k is typical, not required); Thread T1.","Included in the count and total."],"why":"Relied on the glyph, display names and a 'typically' figure."},
{"n":12,"case":"FUP-10","field":"starting conditions and expected","change":"replace","text":{"starting_conditions":"Group O; run after FUP-09, FUP-12, FUP-14 and FUP-25 (final). Each audit records every Request ID and the request-page cost; the final audit also opens each technical page for exact USD.","expected":["Every earlier ID is present exactly once with the same cost.","New entries appear only after a new turn or a reopen; every warm-up is labelled.","No Request ID appears twice.","Final audit: exact USD sum of requests equals the known subtotal."]},"why":"Fits the 80-minute budget; doesn't assume one request per turn."},
{"n":13,"case":"FUP-12","field":"actions","change":"replace","text":["'/new'; send 'Reply with the single word SECOND.' (T2).","'/resume', pick T1 (first message 'OK', not 'SECOND'), press Enter.","Send 'Reply with the single word OK.'; '/cost'."],"why":"The picker choice was ambiguous."},
{"n":14,"case":"FUP-13","field":"actions","change":"replace","text":["In T1: '/side'; send 'Reply with the single word OK.'.","Leave with exactly one Ctrl+C and wait for the return message; '/cost'."],"why":"The /side control."},
{"n":15,"case":"FUP-14","field":"actions and expected","change":"replace","text":{"actions":["Edit the earliest message using the product's own on-screen hint (try Esc twice in an empty composer first); if no edit control is found within 5 minutes, mark blocked.","Replace the text with 'Reply with the single word YES.' and press Enter.","'/cost'."],"expected":["A reply arrives (YES not required); the warm-up count rises by 1; a notice containing 'skip_reopen_warmup' is shown.","Requests from rolled-back turns are still listed once, with the same IDs and costs."]},"why":"The edit keys are an assumption; the reply is outside the intent."},
{"n":16,"case":"FUP-15","field":"expected","change":"replace","text":["Record whether a warm-up and notice occurred; if a warm-up was sent, it is labelled 'Reopen warm-up', counted once and the notice appears; the FUP-10 audit is unaffected."],"why":"The intent doesn't say whether this sends a warm-up."},
{"n":17,"case":"FUP-16","field":"actions","change":"replace","text":["Stop; start with '-c skip_reopen_warmup=true resume <T1>' (if the argument order is rejected, use 'resume <T1> -c skip_reopen_warmup=true'; an argument error is not a product failure).","Send 'Reply with the single word OK.'.","'/side'; send 'Reply with the single word OK.'; leave with one Ctrl+C; '/cost'."],"why":"A deterministic conversation; the /side control."},
{"n":18,"case":"NEW FUP-25","field":"all","change":"add","text":{"priority":"advisory","group":"O","starting_conditions":"After FUP-16; last in Group O (config.toml can't be undone).","actions":["agms-tui config 'skip_reopen_warmup = true'","Stop; start with 'resume <T1>' (no -c).","Send 'Reply with the single word OK.'; '/cost'."],"expected":["No warm-up notice; warm-up count unchanged; only the turn's requests added."]},"why":"The intent names the config file as a way to set this; it was untested."},
{"n":19,"case":"FUP-17","field":"starting conditions, actions, expected","change":"replace","text":{"starting_conditions":"Read d3.txt first. If it grants no Moonshot turns, do not start Group M; report FUP-17..20 as 'not run (D54 not granted)'. Otherwise fresh profile M: agms-tui start --route moonshot --model kimi-k3. Never retry a Moonshot turn.","actions":["Send 'Reply with the single word OK.'; as soon as it replies, send FUP-18's turn; then '/cost' and both technical pages."],"expected_last":"No cost or token field of these requests is unknown, incomplete or a lower bound (a cache count of 0 is fine; metadata lines such as 'wall time: unavailable' are not cost fields)."},"why":"D3/D54 gating; the 5-minute cache lifetime; the wording."},
{"n":20,"case":"FUP-18","field":"expected","change":"replace","text":"If Cache read is 0: record the evidence line and mark blocked ('no cache read observed'), not failed; do not retry. If the relay answers 'test harness: Moonshot request limit reached': blocked. Fail only if a cache read is priced wrongly or the parts don't add up.","why":"No retries allowed; the tester can't tell a provider miss from a product fault."},
{"n":21,"case":"FUP-19","field":"actions and expected","change":"replace","text":{"actions":["Stop; start with agms-tui start --route moonshot --model kimi-k2.7-code (new conversation).","Send 'Reply with the single word OK.', then immediately 'Reply with the single word OK again.'.","'/cost'; open both technical pages."],"expected_add":"Cache read 0 or the limit message: handle as in FUP-18."},"why":"Answer 2: change model by restarting with --model."},
{"n":22,"case":"FUP-20","field":"actions and expected","change":"replace","text":{"actions":["Read the k2.7 conversation's first /cost page.","Stop; start with --route moonshot --model kimi-k3 resume <k3 Thread>; send no turn; '/cost'; repeat."],"expected":["Each conversation lists only the requests from its own two turns.","The other conversation's total equals its own total.","No cost field or total is unknown, incomplete, unavailable or a lower bound.","Earlier IDs and costs unchanged."]},"why":"No extra spend; one turn can be several requests; wording."},
{"n":23,"case":"FUP-21","field":"actions and expected","change":"replace","text":{"actions":["Run the realtime command once (rerun only if the first connection fails with a harness connection error before any reply; never a third time); record the UTC window, reply and TR.","Start with --route zai --model glm-5.3-flash resume <TR>; send no turn; '/cost'; open the realtime request and its technical page."],"expected_replace":["t1 < t2, both within the noted window +/-60 s.","No $ amount, and no token count or cost of 0, is shown for this request; any total that includes it is marked unknown or incomplete."]},"why":"D16/answer 13; avoids an OpenAI warm-up; clock skew; IDs contain zeros."},
{"n":24,"case":"FUP-22","field":"expected","change":"replace","text":"The other-conversations total covering TR shows no $0 for the session (it is unknown or 'no price').","why":"The intent doesn't specify that line's wording."},
{"n":25,"case":"FUP-24","field":"actions","change":"replace","text":"At the end of each group, search all saved screens and command outputs for 'sk-', 'Bearer', 'Authorization', 'api_key', the synthetic .env value and long non-UUID tokens; report a hit inside the transcript of a command the model ran separately from a hit on a product page or notice.","why":"Makes the check concrete and separates model-run commands from product output."}
]}
```