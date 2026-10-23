# Group R functional execution report

Independent functional execution only. This is not human acceptance.

## Preflight, confinement and identity

- Before other case work, executed: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.
- Result: `rc=0`; `preflight-executor.json` reported `"ok": true`; probe context UTC `2026-10-20T04:22:15Z`.
- Coordinator `preflight-summary.json`: `ok: true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `probe_count: 109`, `failed: []`.
- `preflight-launcher.json` and `preflight-tmux-child.json` both reported `"ok": true`.
- Frozen packet hashes matched:
  - design-cases.md: `38afb2d4119c4776af1315fea8cf274984807ff868e031a3992af0234c4986a4`
  - amendments.md: `6bb58647630bd980b10275890afd9f4257dace08080906667ed25171a2d17c3a`
  - answers.md: `296d2b5c0768d0162b63e5f08c332d5200d902614453956c29f8bf6376d42167`
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256 matched `e72003067f9316f2805b4344041d111409611c6d0730fe268c46bf95225bdaa3`; `--version` printed `corbanu 0.1.48`.
- Required realtime control was present in `agms-tui help`. Fresh profile/project was present. At `2026-10-20T04:24:28Z`, `agms-tui fixture group-setup-before` showed the expected `ledger-notes` files and synthetic `.env`.

## Profile/state used

- Fresh Group R profile and fresh fixture project.
- Realtime Thread TR: `01a1578d-d2d0-75d2-9eba-03a3b89ed294`.
- Realtime request ID: `2dbec071-81c5-4428-b384-e8d1ef9d917c`.
- Separate OpenAI prewarm request ID: `a8e0dd9a-ebd3-4c30-b951-18dd32f2da18`.
- No Moonshot route or realtime retry was used.

## FUP-21 — Realtime session is one request, labelled, with unknown (not zero) cost

**Requested actions.** Run realtime once (one rerun only for the specified pre-reply harness connection error); record UTC window/reply/TR; start `--route zai --model glm-5.3-flash resume <TR>`; send no turn; `/cost`; open realtime request and technical page.

**Actual actions.**
1. `2026-10-20T04:24:34Z` through `2026-10-20T04:24:37Z`: ran `agms-tui realtime 'Reply with the single word OK.'` once. No rerun was needed. Output: reply `OK`; `thread: 01a1578d-d2d0-75d2-9eba-03a3b89ed294`; command saved `evidence/realtime-2.txt`; `exit=0`.
2. `2026-10-20T04:24:46Z`: started with `agms-tui start --route zai --model glm-5.3-flash resume 01a1578d-d2d0-75d2-9eba-03a3b89ed294`. Composer visible at `04:24:54Z` (`group-r-fup21-01-after-resume`).
3. Sent `/cost` as text, visually confirmed the composer, then pressed Enter. Screen `group-r-fup21-03-cost-page1`, `2026-10-20T04:25:09Z`.
4. Navigated with visible Down/Enter controls to the realtime request, then its technical page. Screens `group-r-fup21-13-realtime-request-page-top` (`04:25:57Z`) and `group-r-fup21-25-technical-page-top` (`04:27:10Z`).
5. Also opened the separate OpenAI prewarm request and technical page (`04:29:02Z` onward) to determine whether any warm-up was separate and labelled.

**Cost lines judged — first page (`group-r-fup21-03-cost-page1`, `2026-10-20T04:25:09Z`):**

```text
• OpenAI · GPT-6 Sol — Pay per use. 1 request, 14,645 tokens. Estimated cost: $0.029290.
• OpenAI · gpt-realtime-1.5 — Pay per use. 1 request, tokens not reported. Estimated cost: no price available.
Realtime: 1 session; token counts and cost unknown, not zero — a realtime session's usage is not reported to this client.
```

**Cost details judged (`group-r-fup21-09-cost-scroll10` / `group-r-fup21-10-cost-scroll20`, `04:25:41Z`/`04:25:46Z`):**

```text
Known estimated token cost: $0.029290 + unknown costs
Full recorded estimate: unavailable (1 of 2 billed attempts incomplete)
Known subtotal exact USD: 0.02929
Input: 14,645 + not reported by 1 attempt
Noncached input (derived for inclusive input): 14,645 + not reported by 1 attempt
Cache read: not reported (1 attempt)
Cache write: not reported (1 attempt)
Output: not reported (1 attempt)
Reasoning (subset, not separately billed): not reported (1 attempt)
Total (not separately billed): 14,645 + not reported by 1 attempt
```

**Request link (`group-r-fup21-12-request2-selected`, `04:25:55Z`):**

```text
→ Request 2 · Realtime session · OpenAI · gpt-realtime-1.5 · no price available
```

**Realtime request page (`group-r-fup21-13-realtime-request-page-top`, `04:25:57Z`; Request ID shown in `group-r-fup21-14-realtime-request-scroll10`, `04:26:09Z`):**

```text
Provider: OpenAI
Model: gpt-realtime-1.5
Billing: Pay per use
Estimated cost: no price available
Tokens: tokens not reported
Kind: Realtime session — started 2026-10-20T04:24:37.017Z, ended 2026-10-20T04:24:37.435Z; its token counts and cost are unknown, not zero: a
realtime session's usage is not reported to this client.
Request: 2dbec071-81c5-4428-b384-e8d1ef9d917c
```

**Realtime technical page (`group-r-fup21-25-technical-page-top` and later scrolled screens, `04:27:10Z`–`04:27:28Z`):**

```text
Provider: OpenAI
Model: gpt-realtime-1.5
Billing: Pay per use
Estimated cost: no price available
Tokens: tokens not reported
Request: 2dbec071-81c5-4428-b384-e8d1ef9d917c
Attempt: 2c0fe751-d3e8-48af-a5b9-eb9da677f176
Thread: 01a1578d-d2d0-75d2-9eba-03a3b89ed294
Turn: realtime:ws:01a1578d-d396-7883-8a4a-61d8fe7bd01a
Admission time: 2026-10-20T04:24:37.017Z
Realtime session ended: 2026-10-20T04:24:37.435Z
Input: unknown — no retained numeric evidence
Noncached input (derived for inclusive input): unknown — no retained numeric evidence
Cache read: unknown — no retained numeric evidence
Cache write: unknown — no retained numeric evidence
Output: unknown — no retained numeric evidence
Reasoning (subset, not separately billed): unknown — no retained numeric evidence
Total (not separately billed): unknown — no retained numeric evidence
Noncached input cost: unknown — no retained numeric evidence
Cache read cost: unknown — no retained numeric evidence
Cache write cost: unknown — no retained numeric evidence
Output cost: unknown — no retained numeric evidence
Known subtotal exact USD: none — no price for these attempts
Price: none recorded — this request records only its billing basis: pay per use
```

**Separate prewarm observed.** Request page `group-r-fup21-43-warmup-request-page` (`04:29:02Z`) and technical page `04:29:19Z`–`04:29:23Z`:

```text
Provider: OpenAI
Model: GPT-6 Sol
Billing: Pay per use
Estimated cost: $0.029290
Tokens: 14,645 tokens
Request: a8e0dd9a-ebd3-4c30-b951-18dd32f2da18
Attempt: d1c2199a-a345-48f0-8555-9e8e2e391dde
Thread: 01a1578d-d2d0-75d2-9eba-03a3b89ed294
Turn: prewarm:0590bdc5-02c0-43c4-879e-a6447fa341e8
Admission time: 2026-10-20T04:24:36.461Z
Input: 14645
Cache read: 0 (reported)
Cache write: 0 (reported)
Output: 0 (reported)
Total (not separately billed): 14645
Noncached input cost: $0.029290; exact USD 0.02929
Noncached input rate: 2 USD per million tokens
Cache read rate: 0.2 USD per million tokens
Cache write rate: 2.5 USD per million tokens
Output rate: 10 USD per million tokens
```

Arithmetic for the separate prewarm: `(14645 × 2 + 0 × 0.2 + 0 × 2.5 + 0 × 10) / 1,000,000 = 29,290 / 1,000,000 = 0.02929`. This agrees with `$0.029290` / exact `0.02929`. The realtime request itself had no token counts or rate, so no money could be recomputed for it; it remained explicitly unknown/no price.

**Result outcomes.**

| Expected | Outcome | Reason |
|---|---|---|
| First page realtime summary exact | passed | Exact line above appeared. |
| Realtime request link exact | passed | Exact link above appeared. |
| Kind/start/end; `t1 < t2`, both within window ±60s | passed | `t1=04:24:37.017Z`, `t2=04:24:37.435Z`; noted window `04:24:34Z`–`04:24:37Z`. |
| Tokens not reported | passed | Exact line appeared. |
| Technical page Turn/Thread and unknown fields | passed | Exact lines above appeared. |
| No `$` or 0 for the realtime request; total not complete | passed | Request/technical pages had no realtime `$` and no realtime zero. Page total said unavailable/incomplete. The separate prewarm `$0.029290` and `Unknown parent population: 0 attempts` were distinct metadata/requests, not realtime request cost. |
| Any warm-up separate/labelled | passed with ambiguity | A separate prewarm request was present and technical page labelled it `Turn: prewarm:…`; its link/request page did not use the phrase `warm-up`. |

`VERDICT FUP-21: passed`

## FUP-22 — Realtime seen from another conversation

**Requested actions.** As the permitted extra check after successful resume: start a new zai conversation, send one turn, `/cost`.

**Actual actions.**
1. `2026-10-20T04:30:11Z`: stopped and started fresh zai GLM 5.3 Flash; composer shown at `04:30:16Z`.
2. `2026-10-20T04:30:24Z`: typed `Reply with the single word OK.` as text, visually confirmed it (`group-r-fup22-02-prompt-typed`), then pressed Enter.
3. Reply `OK` visible at `04:30:45Z` (`group-r-fup22-04-after-turn-wait10`).
4. Opened `/cost`; first page captured at `04:30:55Z` (`group-r-fup22-07-cost-page1`).

**Cost lines judged (`group-r-fup22-07-cost-page1`, `2026-10-20T04:30:55Z`):**

```text
• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 1 request, 8,496 tokens. Estimated cost: $0.001283 (rounded).
Other conversations on this day, not included above: 2 requests in 1 conversation.
• OpenAI · GPT-6 Sol — Pay per use. 1 request, 14,645 tokens. Estimated cost: $0.029290.
• OpenAI · gpt-realtime-1.5 — Pay per use. 1 request, tokens not reported. Estimated cost: no price available.
Other conversations' estimated cost: at least $0.029290 (1 attempt had no price).
To see those requests, open that conversation with /resume and run /cost there.
```

The other-conversation display did not show `$0` for the realtime session. It showed `no price available` and `1 attempt had no price`. The `$0.029290` portion belongs to the separate GPT-6 Sol prewarm. No rate/token breakdown was shown for the zai line, so its displayed estimate could not be independently recomputed from that page.

**Result outcomes.**

| Expected | Outcome | Reason |
|---|---|---|
| TR's other-conversation view has no `$0` and states unknown/no price | passed | Exact display showed no realtime `$` and `no price available`/`1 attempt had no price`. |
| No model decline | not exercised | Model replied `OK`; no decline occurred. |

`VERDICT FUP-22: passed`

## FUP-23 — Realtime record stable after restart

**Requested actions.** Stop; start `resume <TR>` again; `/cost`; no turn.

**Actual actions.**
1. `2026-10-20T04:31:42Z`: stopped and restarted `agms-tui start --route zai --model glm-5.3-flash resume 01a1578d-d2d0-75d2-9eba-03a3b89ed294`. Composer shown at `04:31:47Z`.
2. Sent `/cost` as text, confirmed composer, pressed Enter; page captured `04:31:54Z` (`group-r-fup23-03-cost-page1`).
3. Opened Request 2 again (`04:32:21Z`), then scrolled to Request ID (`04:32:42Z`). No model turn was sent.

**Cost lines judged (`group-r-fup23-03-cost-page1`, `2026-10-20T04:31:54Z`):**

```text
• OpenAI · GPT-6 Sol — Pay per use. 1 request, 14,645 tokens. Estimated cost: $0.029290.
• OpenAI · gpt-realtime-1.5 — Pay per use. 1 request, tokens not reported. Estimated cost: no price available.
Realtime: 1 session; token counts and cost unknown, not zero — a realtime session's usage is not reported to this client.
```

**Request page after restart (`group-r-fup23-09-realtime-request-page-top` / `group-r-fup23-12-realtime-request-id-after-restart3`, `04:32:21Z`/`04:32:42Z`):**

```text
Provider: OpenAI
Model: gpt-realtime-1.5
Billing: Pay per use
Estimated cost: no price available
Tokens: tokens not reported
Kind: Realtime session — started 2026-10-20T04:24:37.017Z, ended 2026-10-20T04:24:37.435Z; its token counts and cost are unknown, not zero: a
realtime session's usage is not reported to this client.
Request: 2dbec071-81c5-4428-b384-e8d1ef9d917c
```

**Result outcomes.**

| Expected | Outcome | Reason |
|---|---|---|
| Still 1 realtime session | passed | Exact summary showed 1 session. |
| Same Request ID/start/end | passed | Request ID and both timestamps matched FUP-21 exactly. |
| Any extra warm-up separate | passed | No second warm-up/prewarm entry appeared; only the same one GPT-6 Sol request remained. |

`VERDICT FUP-23: passed`

## FUP-24 — No credential printed

**Requested actions.** At group end, search all saved screens and command outputs for `sk-`, `Bearer`, `Authorization`, `api_key`, the synthetic `.env` value, and long non-UUID tokens. Separate hits in a model-run command transcript from hits on product pages/notices.

**Actual actions and observations.**
- All 96 rendered saved screens from this group were searched programmatically after the final navigation captures. Pattern hits: 0. Long non-UUID token hits: 0.
- `evidence/realtime-2.txt` and `evidence/realtime-sign-in.txt` were searched. All targeted patterns and long non-UUID tokens: 0.
- Saved command-output records in `evidence/actions.jsonl` were searched. No targeted credential pattern or long non-UUID token appeared.
- The `agms-tui fixture group-setup-before` output displayed the supplied synthetic `.env` value `agms-canary-7f3a9c-synthetic-token`. This was the fixture source itself, not product output, model output, or a live credential. The file comments identify it as synthetic.
- Non-product preflight evidence contains credential field names such as `zai_api_key` and packet/package hashes. These are identifiers/hashes, not credential material.
- No model-run command transcript was generated in Group R.

**Result outcomes.**

| Expected | Outcome | Reason |
|---|---|---|
| No credential/key material in screens or outputs | passed | Product screens, realtime outputs, and command-output records had no targeted real credential/key material. The only target-text hit was the explicitly synthetic fixture value displayed while checking starting conditions. |

`VERDICT FUP-24: passed`

## Request IDs read

- Realtime request `2dbec071-81c5-4428-b384-e8d1ef9d917c`: read in `group-r-fup21-14-realtime-request-scroll10` and `group-r-fup23-12-realtime-request-id-after-restart3`.
- Prewarm request `a8e0dd9a-ebd3-4c30-b951-18dd32f2da18`: read in `group-r-fup21-43-warmup-request-page` and `group-r-fup21-54-warmup-request-page-again`.

## Refusals / declines

None observed.

## Final verdicts

`VERDICT FUP-21: passed`
`VERDICT FUP-22: passed`
`VERDICT FUP-23: passed`
`VERDICT FUP-24: passed`
