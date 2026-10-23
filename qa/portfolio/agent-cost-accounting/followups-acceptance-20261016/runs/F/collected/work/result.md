# Group F functional execution report — independent executor

Run UTC started: 2026-10-16T06:10:43Z (probe context)

## Isolation preflight and candidate identity
- Executed before case work: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; rc=0 and top-level `"ok": true`.
- Coordinator preflight files observed beside it:
  - `preflight-summary.json`: `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `probe_count: 105`, `failed: []`, `ok: true`.
  - `preflight-launcher.json`: top-level `"ok": true`.
  - `preflight-tmux-child.json`: top-level `"ok": true`.
- Packet hashes:
  - design-cases.md `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6` (matches)
  - amendments.md `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e` (matches)
  - answers.md `dc4044d36ac3868205ff2dc71cee36805969c6896c683852a6f9d705dc2d7c0c` (matches)
- Candidate `/opt/agms/pkg/bin/corbanu` digest `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1` (matches; reported by probe and confirmed by expected value).
- Probe also launched package and observed `corbanu 0.1.48`.
- Starting profile: fresh `$AGMS_RUN/product-home` (only an initial `config.toml` was present before launch).
- Starting project: supplied fixture `$AGMS_RUN/fixture/workspace` unchanged before product launch.

Notes:
- The packet screen copies are layout references from earlier builds and are not treated as expected outcomes.
- Product reply navigation and exact `/cost` observations are recorded live.

## Cases

### ACC-30 — catalogued `gpt-5.4` is covered by the subscription

**Actual actions (requested = executed literally, except navigation corrections):**
1. 2026-10-16T06:12:46Z: `agms-tui start --route chatgpt --model gpt-5.4` (stand-in sign-in succeeded). Waited for composer; screen label ACC-30-01 showed `model: gpt-5.4`.
2. 2026-10-16T06:12:58Z: `agms-tui text 'Reply with the single word OK.'`; ACC-30-02 showed the composer exactly as typed.
3. 2026-10-16T06:13:03Z: `agms-tui key Enter`; waited and captured ACC-30-03/04. The product showed `• openai-mock: authorized as work (token gen 2)` and returned to the composer. The literal word `OK` was not visible in the retained screen; this did not affect the accounting checks.
4. 2026-10-16T06:13:32Z: typed `/cost`, checked the composer text, then pressed Enter. Captured ACC-30-06.
5. Opened the request page and technical page through their visible links (some neutral navigation overshoots/corrections occurred; no other action was taken).

**Visible accounting checkpoints:**
- ACC-30-06 `/cost` first screen (2026-10-16T06:13:32Z):
  `• OpenAI · gpt-5.4 — Covered by your subscription (not billed per request). 1 request, 107 tokens. Same work at API prices: at least $0.000105.`
  `Known subtotal exact USD: none — subscription work is not billed per token`
- ACC-30-35 request page (2026-10-16T06:18:49Z):
  `Request`
  `Billing: Covered by your subscription (not billed per request)`
  `Same work at API prices: at least $0.000105`
  `Tokens: 107 tokens`
- ACC-30-46 technical page (2026-10-16T06:19:58Z):
  `Billing: Covered by your subscription (not billed per request)`
  `Same work at API prices: at least $0.000105`
  `Tokens: 107 tokens`
  Later details included `Billing basis: subscription`.
- The only dollar amount observed was explicitly `Same work at API prices` / `Same tokens at API rates`; no `Estimated cost: $`, `Cost: $`, `charged by`, or `no price available` was observed. No known subtotal dollar amount was observed.

**Recomputation:** There is no billed cost to recompute. The API-price display was already a lower-bound line; no cost or subtotal was presented. At shown API rates, 100 input × $2.50/M = $0.00025 and 7 output × $15/M = $0.000105; cache rates do not increase the lower bound because reported cache counts were 0/not reported. The display's `at least $0.000105` is the known output component and is labelled as API prices, not cost.

**Other observation:** At start, the product displayed an MCP startup warning for `codex_apps`: `unexpected server response: HTTP 404`. No case action required that MCP client; navigation and `/cost` remained available.

**Outcome per expected result:**
- First-screen line contains Covered by your subscription: **passed**.
- Request page contains Covered by your subscription: **passed**.
- Forbidden cost/no-price phrases absent; no known subtotal dollar amount: **passed**.
- Known subtotal shows `none — subscription work …`: **passed** (`none — subscription work is not billed per token`).
- Dollar figure is labelled as API prices: **passed**.

### ACC-31 — uncatalogued `gpt-mock-unlisted` is covered by the subscription

**Actual actions (requested = executed literally, except navigation corrections):**
1. 2026-10-16T06:21:31Z: stopped the ACC-30 terminal, then started a new conversation with `agms-tui start --route chatgpt --model gpt-mock-unlisted`. Stand-in sign-in succeeded and ACC-31-01 showed `model: gpt-mock-unlisted`.
2. 2026-10-16T06:21:43Z: `agms-tui text 'Reply with the single word OK.'`, verified exact composer text, then Enter. Reply completed and showed the same `authorized as work` checkpoint. As in ACC-30, the literal word `OK` was not visible in the retained screen.
3. 2026-10-16T06:21:53Z: typed `/cost`, verified composer text, pressed Enter; captured ACC-31-04.
4. Opened Request 1 and Technical details (attempt 1) through visible links; navigation corrections were neutral only.

**Visible accounting checkpoints:**
- ACC-31-04 `/cost` first screen (2026-10-16T06:21:53Z):
  `• OpenAI · gpt-mock-unlisted — Covered by your subscription (not billed per request). 1 request, 107 tokens. Same work at API prices: not available.`
  `Other conversations on this day, not included above: 1 request in 1 conversation.`
  `• OpenAI · gpt-5.4 — Covered by your subscription (not billed per request). 1 request, 107 tokens. Same work at API prices: at least $0.000105.`
- ACC-31-07 request page (2026-10-16T06:22:27Z):
  `Billing: Covered by your subscription (not billed per request)`
  `Same work at API prices: not available`
- ACC-31-13 technical page (2026-10-16T06:25:03Z):
  `Billing: Covered by your subscription (not billed per request)`
  `Same work at API prices: not available`
- ACC-31-15 technical middle:
  `No recorded attempt here was billed per token.`
  `Same tokens at API rates: unavailable — the catalogue states no API price for 1 of 1 plan attempts`
  `Billing basis: subscription`

**Recomputation:** No billed cost. There is no charge and no known subtotal. The uncatalogued API-price line says `not available`, which the amendment explicitly allows; it is not the forbidden `no price available`.

**Other observations:** The product displayed `⚠ Model metadata for 'gpt-mock-unlisted' not found. Defaulting to fallback metadata; this can degrade performance and cause issues.` Accounting pages remained available. The same MCP warning appeared at start.

**Outcome per expected result:**
- First-screen line and request page covered: **passed**.
- Technical page covered and billing basis subscription: **passed**.
- Forbidden `Estimated cost: $`, `Cost: $`, `charged by`, `no price available` absent: **passed**.
- Known subtotal has no dollar amount; API-price `not available` allowed: **passed**.
- ACC-30 conversation appears under other conversations as covered, never as spend: **passed** (shown above).

### ACC-32 — priority tier is covered by the subscription

**Actual actions:**
1. 2026-10-16T06:26:16Z: stopped prior terminal and started a new conversation with `agms-tui start --route chatgpt --model gpt-5.4 -c 'service_tier="priority"'`. The command matched the case's stated flag.
2. ACC-32-01 showed `model: gpt-5.4 fast` and status line `gpt-5.4 default fast`. This is an ambiguity: the visible model-tier label said `fast`, although the requested `service_tier="priority"` was passed unchanged. I did not inspect internals or alter the case.
3. 2026-10-16T06:26:37Z: typed the exact prompt, checked the composer, then Enter.
4. 2026-10-16T06:26:46Z: typed `/cost`, verified composer text, pressed Enter; captured ACC-32-04.
5. Opened Request 1 and Technical details (attempt 1) through visible links.

**Visible accounting checkpoints:**
- ACC-32-04 `/cost` first screen (2026-10-16T06:26:46Z):
  `• OpenAI · gpt-5.4 — Covered by your subscription (not billed per request). 1 request, 107 tokens. Same work at API prices: not available.`
  `Known subtotal exact USD: none — subscription work is not billed per token` (visible in ACC-32-08)
- ACC-32-11 request page (2026-10-16T06:28:23Z):
  `Billing: Covered by your subscription (not billed per request)`
  `Same work at API prices: not available`
- ACC-32-16 technical page bottom (after 2026-10-16T06:28:44Z open):
  `Output cost: unknown — rate unavailable`
  `Known subtotal exact USD: none — subscription work is not billed per token`
  `Price: none recorded — this request records only its billing basis: subscription`

**Recomputation:** No billed cost and no known subtotal dollar amount. `Output cost: unknown — rate unavailable` is an unpriced API-rate component, not a billed cost; the known subtotal is explicitly none. There is no charge arithmetic.

**Outcome per expected result:**
- First-screen and request pages covered: **passed**.
- No `Estimated cost: $`, `Cost: $`, `charged by`, or `no price available`: **passed**.
- Known subtotal shows no dollar amount and says none: **passed**.
- `Same work at API prices: not available` is allowed: **passed**.
- The requested priority-tier flag was used, but the screen displayed `fast`; accounting outcome still passed. This ambiguity is recorded.

### ACC-33 — a ChatGPT image is covered by the subscription

**Actual actions:**
1. 2026-10-16T06:29:53Z: stopped the prior terminal; ran `agms-tui standin chatgpt-image` (confirmed `next answer uses the image tool`); started a new `chatgpt`/`gpt-5.4` conversation and waited for composer.
2. 2026-10-16T06:30:05Z: typed `Make an image of a red square.`, checked exact composer text, then Enter. The product showed `Generated Image: a plain red square on white` and a product-home image path.
3. 2026-10-16T06:30:14Z: typed `/cost`, checked composer text, pressed Enter; captured ACC-33-04.

**Visible accounting checkpoints:**
- ACC-33-04 `/cost` first screen (2026-10-16T06:30:14Z):
  `• OpenAI · gpt-5.4 — Covered by your subscription (not billed per request). 2 requests, 214 tokens. Same work at API prices: at least $0.000210.`
  `• OpenAI · gpt-image-2 — Covered by your subscription (not billed per request). 1 request, 277 tokens. Same work at API prices: not available.`
  `Other conversations on this day, not included above: 3 requests in 3 conversations.`
  `• OpenAI · gpt-5.4 — Covered by your subscription (not billed per request). 2 requests, 214 tokens. Same work at API prices: at least $0.000105.`
  `• OpenAI · gpt-mock-unlisted — Covered by your subscription (not billed per request). 1 request, 107 tokens. Same work at API prices: not available.`

**Recomputation:** No billed cost. The API-price lower bound shown for text attempts is $0.000210 for two 107-token gpt-5.4 requests (100 input × $2.50/M = $0.00025 each; 7 output × $15/M = $0.000105 each; the displayed at-least amount reflects the one priced output component). This is labelled API prices and is not spend. The image request has no catalogue API price and no billed cost.

**Outcome per expected result:**
- `gpt-image-2` line is covered and has 1 request: **passed**.
- No spend and no forbidden cost/no-price wording: **passed**.
- Image request counted once: **passed** (own group shows one image request; other conversations show three prior requests in three conversations, none image).

### ACC-34 — no credential is printed (Group F scope only)

**Actual actions:**
1. Identified Group F evidence sources: `evidence/screens.jsonl`, `evidence/actions.jsonl`, `evidence/pty.raw`, and `evidence/chatgpt-sign-in.txt`; plus files under `fixture/workspace`. There was no Group F `exec` output because no Group F case requested `exec`.
2. 2026-10-16T06:31:38Z: ran a case-sensitive extended-regex pattern scan for:
   `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, `api_key`, `token=`
   over the Group F evidence sources and the workspace files. No hits were returned. Excluded only this report and the coordinator's isolation preflight JSON, which are not product captures, exec output, or workspace files.

**Recomputation / interpretation:** No API key, bearer token, sign-in token, Authorization header text, synthetic value, or key=value credential pattern appeared in Group F captures, sign-in output, or workspace files. No human acceptance is claimed.

**Outcome per expected result:**
- Group F captures and workspace files have no specified credential-pattern hit: **passed**.
- All-groups/exec-output cross-profile scan: **not exercised** by this executor because only Group F was assigned and this group produced no exec output; per the assignment, ACC-34 applies to this group's own screens and output only.

Group UTC end: 2026-10-16T06:31:38Z (after final ACC-34 pattern scan).

## Overall verdicts

VERDICT ACC-30: passed
VERDICT ACC-31: passed
VERDICT ACC-32: passed
VERDICT ACC-33: passed
VERDICT ACC-34: passed
