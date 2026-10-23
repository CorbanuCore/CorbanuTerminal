# #654 under Aggressive (stored level, credential broker on), GLM 5.3 Flash on Z.AI, disposable home

## BEFORE: installed dev build (main before the fix), same spec (the final wait for a recorded request timed out)
```
› Reply with exactly one word: pong
• pong
⚠ Developer accounting could not record one or more model requests in this turn. They were sent anyway; /cost does not
  include them. The cause is in the log (target codex_core::accounting).
  Cost — this conversation
  Requested UTC day: 2026-10-13
› Today (UTC) in this conversation: no recorded requests.
  No other conversation recorded requests on this day.
  Costs are estimates from published prices; your provider's bill is the final amount.
  —— Details ——
  No recorded attempts in this conversation or its subagents; collection coverage unknown.
  Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.
  Logical requests may have attempts on other days; this UTC day is not their complete lifetime.
  Snapshot is not current; newer activity is unverified
  Other dates: /cost [YYYY-MM-DD]  or  START END hour|day|week|month  (UTC)
  ↑↓ scroll · Enter open · Esc back/close
```
Log (codex_core::accounting, WARN):
```
2026-10-13T11:53:24.559272Z accounting sampling closed; later requests in this turn are sent unrecorded at=core/src/accounting_transport.rs:72:23
2026-10-13T11:53:24.559449Z accounting: request sent unrecorded step="admit attempt" cause=accounting route mismatch
```

## AFTER: 9686fb97d0 (this PR), same spec
```
│  Limits:               data not available yet                                                                       │
╰─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
› Reply with exactly one word: pong
• pong
  Cost — this conversation
  Requested UTC day: 2026-10-13
› Today (UTC) in this conversation:
  • Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 1 request, 8,837 tokens. Estimated cost: $0.001332 (rounded).
  No other conversation recorded requests on this day.
  Costs are estimates from published prices; Z.AI doesn't report its charges — check Z.AI's bill.
  Select a provider below to see its requests.
  —— Details ——
  Estimated token cost for recorded attempts: $0.001332 (rounded)
  Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.
  Other dates: /cost [YYYY-MM-DD]  or  START END hour|day|week|month  (UTC)
  ↑↓ scroll · Enter open · Esc back/close
```
Log: no codex_core::accounting WARN lines; broker lines:
```
2026-10-13T11:52:22.905260Z provider keys handed to the credential broker and removed from the environment: ZAI_API_KEY
2026-10-13T11:52:22.958229Z model provider credential held by the credential broker (host=api.z.ai, port=443, path=/api/paas/v4, header=Bearer)
```
