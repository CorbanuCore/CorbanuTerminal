# R05 functional execution report

Execution scope: independent functional executor. This report records observed outcomes only and does not claim human acceptance.

## Isolation preflight and candidate identity

UTC preflight start/end: 2026-10-21T04:41:53Z; final verification read 2026-10-21T04:43:05Z.

- Executor preflight command requested: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.
- Actual command output: `rc=0`.
- `$AGMS_RUN/evidence/preflight-executor.json` reports `"ok": true`.
- Coordinator preflight summary `$AGMS_RUN/evidence/preflight-summary.json` reports `"ok": true`; the files beside it, `preflight-launcher.json` and `preflight-tmux-child.json`, each report `"ok": true`.
- Candidate requested: Corbanu Terminal 0.1.48 package at `/opt/agms/pkg`, program `/opt/agms/pkg/bin/corbanu`.
- Observed program SHA-256: `2a1ad7626102290c38b2dd573ac5aad179b11d5c834546fdb661e2a29ddc7a1f` (matches).
- Frozen-file SHA-256 observations match the assignment:
  - `acc/design-cases.md`: `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6`
  - `acc/amendments.md`: `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e`
  - `acc/answers.md`: `dc4044d36ac3868205ff2dc71cee36805969c6896c683852a6f9d705dc2d7c0c`
  - `amendments-L2.md`: `929fdb22940243167eb37e410a78fd6db238e05b17e9aa5383ddda31b948449d`

## ACC-04 — sub-agent from inside `/side` is labelled `side:agent:`

- Priority: advisory.
- Outcome: passed (observed functional result; no human-acceptance claim).
- Starting state: fresh product profile (`product-home` contained only `config.toml` at 04:43:39Z), fresh workspace, clean git status, and fixture present. Pre-case fixture label: `pre-case`.
- UTC group start: 2026-10-21T04:43:39Z.
- Requested versus actual: all assigned ACC-04 actions were requested and performed; no extra case actions were taken.
- L-1 model recording, positive:
  - Trust prompt screen label `ACC04-trust` at 2026-10-21T04:45:03Z showed `› 1. Yes, continue`, `2. No, quit`, and `Press enter to continue`. Action: Enter on option 1.
  - Header after trust, screen label `ACC04-model`: `model:     Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixe…`; status line: `Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixed-4-8bit via qwen default · ~/runs/agms-R05-agmx2-20261021T044015Z/fixture/workspace · Corbanu Terminal…`; header version `v0.1.48`. This is the required model/connection.
- Baseline turn:
  - Request text sent verbatim: `Reply with the single word OK.`; text action timestamp 04:45:22Z, Enter timestamp 04:45:30Z. Composer screen `ACC04-baseline-composer` showed the exact request before Enter.
  - Screen `ACC04-baseline-turn-end` at 2026-10-21T04:50:38Z showed product cell `• OK` and model reply `OK`, with idle composer. The expected metadata warning was present. Turn end treated as this observation because no `Worked for` line appeared.
- `/cost` navigation and baseline records:
  - Command `/cost` sent verbatim; Enter timestamp about 04:50:52Z. Screen `ACC04-cost-initial` at 04:50:54Z recorded:
    - Provider line: `• qwen · Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixed-4-8bit — Billing basis not declared. 1 request, 13,086 tokens. Counted neither as money spent nor as subscription work.`
    - Other-conversations line: `No other conversation recorded requests on this day.`
    - `R0 = 1`.
  - Request page screen `ACC04-request1-details1`: `Request: e9d9bd78-dbfa-4f24-9871-8370eb293739`.
  - Technical page screen `ACC04-request1-tech-details`:
    - `Request: e9d9bd78-dbfa-4f24-9871-8370eb293739`
    - `Attempt: 617b5312-85c2-4313-95d1-03dfa2715ce6`
    - `Thread: 01a15cc7-0919-7a11-9a86-cf042379d984`
    - `Turn: 01a15cc7-59df-7cf3-9d78-5267ddde7c3c`
  - Esc sequence observed technical → request (`ACC04-esc1`) → provider (`ACC04-esc2`) → cost (`ACC04-esc3`) → composer (`ACC04-esc4`) at 04:54:33Z.
- ACC-04 action 1 (`/side`):
  - Sent `/side` text at 04:55:01Z and Enter; screen `ACC04-side-open` at 04:55:03Z showed side status: `Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixed-4-8bit via qwen default · ~/runs/agm… Side from main thread · ctrl + / to switch · ctrl + c to close`.
- ACC-04 action 2, first request (no decline/resubmission needed):
  - Request text verbatim: `Start one sub-agent that replies with the single word AGENT, then tell me its answer.`; text action timestamp 04:55:14Z, Enter timestamp 04:55:22Z. Composer screen `ACC04-side-request-composer` showed it word for word before Enter.
  - No product prompt appeared; no one-time option was needed.
  - Screen `ACC04-side-request-t30` at 2026-10-21T04:56:04Z showed the turn complete and these product cells verbatim:
    - `• Spawned 01a15cd0-afa9-7e80-aab0-133f153260d0 (Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixed-4-8bit medium)`
    - `  └ Reply with the single word AGENT.`
    - `• Waiting for 01a15cd0-afa9-7e80-aab0-133f153260d0`
    - `  └ Wakes on a child report/message, follow-up task, human steering, or timeout.`
    - `• Wait ended — child status update`
    - `  └ 01a15cd0-afa9-7e80-aab0-133f153260d0: Completed - AGENT AGENT`
    - `• Closed 01a15cd0-afa9-7e80-aab0-133f153260d0`
  - Model reply verbatim: `• The sub-agent replied "AGENT".`
  - No request was declined. The expected metadata warning was present.
- ACC-04 action 3, leave `/side`:
  - Pressed Ctrl+C once at 2026-10-21T04:56:18Z. Screen `ACC04-side-exit-3s` at 04:56:23Z showed the saved main conversation and status line without `Side from main thread`. The build showed no separate confirmation, as the L2 amendment states.
- Post-side `/cost`:
  - Command `/cost` sent verbatim at 04:56:32Z; screen `ACC04-post-cost-initial` at 04:56:35Z already listed the expected requests, so no Refresh was needed. It recorded:
    - Provider line: `• qwen · Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixed-4-8bit — Billing basis not declared. 6 requests, 74,545 tokens. Counted neither as money spent nor as subscription work.`
    - Other-conversations line: `No other conversation recorded requests on this day.`
    - Request count after side: 6; therefore five new requests (Request 2 through Request 6) relative to R0=1.
    - On the Open list, screen `ACC04-post-provider-open`: `→ Descendant attempts (none)`.
  - Technical pages for every new request:
    - Request 2: ID `48642a21-3634-4488-bf73-00b4691c7ef2`; Thread `01a15cc7-0919-7a11-9a86-cf042379d984`; Turn `side:01a15cd0-6001-7e33-b502-fade0e3444df`.
    - Request 3: ID `b0a0d293-da99-4f53-83c7-d5b740e934ac`; Thread `01a15cc7-0919-7a11-9a86-cf042379d984`; Turn `side:01a15cd0-6001-7e33-b502-fade0e3444df`.
    - Request 4: ID `84588b0a-4994-4ca5-8634-15a06e0fedf0`; Thread `01a15cc7-0919-7a11-9a86-cf042379d984`; Turn `side:agent:01a15cd0-afcf-7992-a9ec-b1f4a7539727`.
    - Request 5: ID `088f6d2b-a517-4006-9fd3-d14440b3c1bd`; Thread `01a15cc7-0919-7a11-9a86-cf042379d984`; Turn `side:01a15cd0-6001-7e33-b502-fade0e3444df`.
    - Request 6: ID `afbe4628-0ded-42cb-aeca-77a8ef89c069`; Thread `01a15cc7-0919-7a11-9a86-cf042379d984`; Turn `side:01a15cd0-6001-7e33-b502-fade0e3444df`.
  - Screen labels for technical details: `ACC04-R2-tech-details`, `ACC04-R3-tech-details2`, `ACC04-R4-tech-details`, `ACC04-R5-tech-details`, and `ACC04-R6-tech-details`.
  - The current cost view's Open list showed no descendant-attempt entries; the visibly spawned sub-agent's sole identified request is Request 4 via the `side:agent:` Turn prefix. Its Thread remained the baseline thread ID.
- Observations and ambiguity:
  - The visible Spawned cell ID is `01a15cd0-afa9-7e80-aab0-133f153260d0`, while Request 4's `side:agent:` Turn suffix is `01a15cd0-afcf-7992-a9ec-b1f4a7539727`. Under the L2 rule, Request 4 is nevertheless a sub-agent request because its Turn starts with `side:agent:`.
  - Four side-conversation requests had the plain `side:` prefix and are recorded but not judged under the L2 sub-agent rule.
  - The request count rose from 1 to 6. The single identified sub-agent request is listed under the current conversation; no other-conversation requests appeared.
  - No credential/system password prompt, unexpected profile access, missing required control, or lost observation occurred.
- Expected-result observations:
  - "If a sub-agent runs": a sub-agent visibly ran (`Spawned`/`Waiting`/`Closed` cells and reply `AGENT AGENT`).
  - Sub-agent request attribution: met for Request 4: `Turn: side:agent:…`.
  - "appear once under this conversation": met on the visible evidence: it appears as Request 4 under this conversation; no duplicate was seen.
  - "do not appear among other conversations": met on the visible evidence: `No other conversation recorded requests on this day.`
  - No human acceptance decision is claimed.
- End of case/group: 2026-10-21T05:04:47Z. `agms-tui fixture post-case` showed the same non-credential file contents and the same `.env` size/SHA-256 as `pre-case` (`98` bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`); `outside-notes.txt` remained unchanged.
- `agms-tui stop` returned `stopped` and rc 0.
- State/profile: neutral accounting profile with no security features; required local model and `qwen` connection shown throughout; no wrong-model observation.
- Model declines/refusals: none.
- Prompts: only the initial workspace trust prompt was observed; Enter selected `1. Yes, continue`. No product command/tool approval prompt appeared.
- Requested versus actual actions summary: requested trust-enter, baseline OK turn, baseline `/cost` inventory, `/side`, sub-agent request, Ctrl+C exit, and post-side `/cost` inventory; all were performed. No additional prompts or case actions were used.

VERDICT ACC-04: passed
