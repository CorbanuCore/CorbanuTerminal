# PF27 review allowance

Travis authorized: “You get 5 more review slots. Reset review slots every 6 hours.”
Recorded at **2026-09-12 05:25:45 UTC** (September 11, 22:25:45 Phoenix).

- Scope: this PF27 workstream; other lanes' historical budgets are unchanged.
- Five additional passes are available immediately. Replenish to five at each
  six-hour boundary anchored to the timestamp above; unused slots do not accrue.
- First reset: **2026-09-12 11:25:45 UTC** (04:25:45 Phoenix), then every six hours.
- Count a review at dispatch, including failed/interrupted requests. Preserve its
  timestamp, model, scope, evidence and outcome. Never erase historical reviews
  or restart review numbering when the allowance replenishes.
- At each dispatch, compute the active window and subtract reviews dispatched
  in that window. Reserve the slot before starting; root serializes dispatches.
- This is a ceiling, not a target. No unnecessary reviews, duplicate clean
  passes, or waiting for a reset merely to evade unresolved findings.
- No privileged installation or scope expansion is authorized by this allowance.

## History and current window

Reviews 1–6 are spent; preserve their existing records. Review 6 is documented
in [resumed service evidence](resume-20260911/README.md).

Window **2026-09-12 05:25:45Z ≤ dispatch < 2026-09-12 11:25:45Z**:
**0 used / 5 available** at this checkpoint; no new review has started.

Planned, not dispatched: review 7 Astra High for the new trusted-child stage;
review 8 Fable 5.1 High final review through Corbanu/TMUX. Record subsequent
dispatches below, retaining older windows rather than overwriting them.

| Review | Dispatch UTC | Model / scope | Outcome / evidence |
| --- | --- | --- | --- |
