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
**5 used / 0 available**: review 11 reserved at 06:46 UTC before dispatch.

Record subsequent dispatches below, retaining older windows rather than
overwriting them. These are reviews of the new stage, not repeated service-stage reviews.

| Review | Dispatch UTC | Model / scope | Outcome / evidence |
| --- | --- | --- | --- |
| 7 | 2026-09-12 05:44Z | Astra High; child-admission local diff over f2e6d77e4 | Exit 1: P1 stale socket/PID reuse, P2 reaper-spawn failure; both accepted in scope. See child-admission-20260912/astra-seven.json. |
| 8 | 2026-09-12 05:54Z | Fable 5.1 High through Corbanu/TMUX; repaired child stage and final evidence | Exit 1; runtime correct, one P2 kernel-specific test assertion accepted. See child-admission-20260912/fable-eight.json. |
| 9 | 2026-09-12 06:03Z (reserved 06:02Z) | Fable 5.1 High through Corbanu/TMUX; test-portability repair and final evidence | Exit 0, no findings; child-admission-20260912/fable-nine.json. Stage review closed; no further pass needed. |
| 10 | 2026-09-12 06:44Z | Astra High; new existing-root adapter over 34bba8814, formatted source; tests running separately | Exit0, findings[]; root-composition-20260912/astra-ten.json. |
| 11 | 2026-09-12 06:46Z | Fable 5.1 High Corbanu/TMUX; same root adapter and completed exact-tree evidence | Exit1, runtime correct, P3 missing ignored logs in review bundle; explicit tracking verified before commit, no source change. Original root-composition-20260912/fable-eleven.json retained. No duplicate review needed. |

## Integrator extension — September 12, 08:43 UTC

Manager task `01a08522-76a1-7ad1-afe6-ad690d55c0d7` relayed Travis's explicit
policy amendment permitting integrator-authorized additional reviews and granted
**two scoped passes now** for the frozen probe: Astra12 and required Fable13.
This is **+2**, not a reset: the five scheduled-window passes above remain spent.
The six-hour replenishment anchor and all historical outcomes are unchanged.
No new scope, native installation, activation or release authority is granted.
Avoid further opinions once no substantive findings remain; proceed to integration.

| Review | Reservation UTC | Model / scope | Outcome / evidence |
| --- | --- | --- | --- |
| 12 | 2026-09-12 08:43Z; dispatched 08:44Z | Astra High; frozen probe branch diff over a666c6cd9 | Exit0, findings[]; probe-20260912/astra-twelve.json. |
| 13 | 2026-09-12 08:43Z; dispatched 08:44Z | Fable 5.1 High via Corbanu/private TMUX; same probe | Exit1, patch correct, sole P3 fixture Python-path portability; probe-20260912/fable-thirteen.json. No runtime/security finding. |

Extension accounting: **2 used / 0 available**. Both processes finished.
Do not silently retry or erase the nonzero Fable result. The portability
disposition and integration checkpoint are in probe-20260912/README.md.

## Next launcher stage — additional integrator allowance

After verified main `3e8bf6c95`, manager task
`01a08522-76a1-7ad1-afe6-ad690d55c0d7` explicitly authorized **up to3 additional
necessary scoped design/code/evidence passes** for the next launcher allocation
without waiting for11:25Z. This is a separate +3 extension, not a reset of the
five scheduled passes or the consumed +2 extension. No installation authority.

Scope: [launcher recipe/identity preparation](launcher-next-20260912.md).
**2 used / 1 available**; review14 dispatched09:15Z, review15 dispatched09:16Z. Prefer
the required Astra High and Fable5.1High closeout, keeping the third for a
concrete repair/evidence need. Do not spend a pass just because it is available.

| Review | Reservation UTC | Model / scope | Outcome / evidence |
| --- | --- | --- | --- |
| 14 | 2026-09-12 09:14:49Z | Astra High; launcher recipe/preparation over08a32a80e, Rust tree49d901362418ee11232d70e468cebe84ec37abfb | Exit0, findings[]; launcher-20260912/astra-fourteen.json. |
| 15 | 2026-09-12 09:16:18Z | Fable5.1High Corbanu/privateTMUX; same source plus completed RTX/TMUX evidence | Exit0, findings[]; launcher-20260912/fable-fifteen.json. |

## Manifest inspection — scoped integrator extension

September12 manager task `01a08522-76a1-7ad1-afe6-ad690d55c0d7` accepted
manifest-next-20260912.md and authorized TWO necessary new-stage passes:
Astra High and Fable5.1High via Corbanu/TMUX. **2 used / 0 available**.
The prior unused contingency is retained separately; reviews1–15 and the
scheduled six-hour anchor are unchanged. Reserve each dispatch before starting.
No root-positive execution, installation or main-write window is granted.

| Review | Reservation UTC | Model / scope | Outcome / evidence |
| --- | --- | --- | --- |
| 16 | 2026-09-12 09:51:08Z | Astra High; manifest inspection over919b1652d, Rust5df6837f11abe2b176c18e1e69a53ef27b595194 | Exit0, findings[]; manifest-20260912/astra-sixteen.json. |
| 17 | 2026-09-12 09:52Z | Fable5.1High Corbanu/privateTMUX; same source and completed RTX/TMUX evidence | Engine started09:52:45Z; exit0/findings[]; manifest-20260912/fable-seventeen.json. |

## Sealed image — scoped integrator extension

September12 manager task `01a08522-76a1-7ad1-afe6-ad690d55c0d7` accepted
sealed-image-next-20260912.md and granted TWO necessary new-stage passes,
Astra High and Fable5.1High Corbanu/TMUX. **2 used / 0 available**.
Historical1–17 and the old unused contingency remain unchanged; no scheduled
wait or reset. Reserve each dispatch first. No executable invocation, privileged
installation, new main window or dynamic-loader trust is authorized.

| Review | Reservation UTC | Model / scope | Outcome / evidence |
| --- | --- | --- | --- |
| 18 | 2026-09-12 10:12:08Z | Astra High; sealed image over e66438e33, Rust0aa65bd04f5e30f3a21e1309aac7aa6b83336859 | Exit0, findings[]; sealed-20260912/astra-eighteen.json. |
| 19 | 2026-09-12 10:13:19Z | Fable5.1High Corbanu/privateTMUX; same source and completed RTX/TMUX evidence | Exit0, findings[]; sealed-20260912/fable-nineteen.json. |
