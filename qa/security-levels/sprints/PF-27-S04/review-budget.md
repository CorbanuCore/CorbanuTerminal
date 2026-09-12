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
**3 used / 2 available**: review 9 reserved at 06:02 UTC before dispatch.

Record subsequent dispatches below, retaining older windows rather than
overwriting them. These are reviews of the new stage, not repeated service-stage reviews.

| Review | Dispatch UTC | Model / scope | Outcome / evidence |
| --- | --- | --- | --- |
| 7 | 2026-09-12 05:44Z | Astra High; child-admission local diff over f2e6d77e4 | Exit 1: P1 stale socket/PID reuse, P2 reaper-spawn failure; both accepted in scope. See child-admission-20260912/astra-seven.json. |
| 8 | 2026-09-12 05:54Z | Fable 5.1 High through Corbanu/TMUX; repaired child stage and final evidence | Exit 1; runtime correct, one P2 kernel-specific test assertion accepted. See child-admission-20260912/fable-eight.json. |
| 9 | 2026-09-12 06:03Z (reserved 06:02Z) | Fable 5.1 High through Corbanu/TMUX; test-portability repair and final evidence | Exit 0, no findings; child-admission-20260912/fable-nine.json. Stage review closed; no further pass needed. |
