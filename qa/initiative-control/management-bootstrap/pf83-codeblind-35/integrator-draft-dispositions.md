# Integrator draft dispositions (input to the independent evidence check)

Written by the integrator before the independent evidence check, from the record in
this directory only. The raw executor verdict is preserved separately in each
`runs/*/collected/**/result.md` and is never edited. The reviewer may overturn any line.

Rule applied: a case is `passed` only if every observable result in its frozen text
is demonstrated by candidate-bound actions/screens/fixture evidence. An outcome the
executor marked inconclusive, a required action not performed, or a discriminating
probe not established keeps the case `blocked`. A demonstrated contradiction of an
observable result is `failed`.

| Case | Attempt(s) | Raw executor verdict | Draft disposition | Basis |
|---|---|---|---|---|
| F01 | F01 | passed | passed | Probe established (restricted gated, approved); Full Access confirmed via dialog, "applied for NEXT TURN" while pending; new-turn probe ran without approval; fixture shows both probe markers. |
| F02 | F02 | passed | passed | Full Access established and behaviorally verified; restriction selected; probe in the pending interval and probe after "(current)" both gated, declined, markers absent. |
| F03 | F03, F03-attempt2 | passed; failed | blocked | Attempt 1: executor reports the product model declined the out-of-workspace probe and marks the core outcome inconclusive, yet records `passed`. Attempt 2: screens show the "Enable full access?" cursor on `› Cancel` each time Enter was pressed, so Full Access was never applied; the executor's `failed` (control non-functional) is not supported by the screens. Neither attempt demonstrates "subsequent eligible commands stop receiving restricted-mode approvals" after an active-turn selection. |
| F04 | F04, F04-attempt2 | passed; passed | blocked | Post-effective gating demonstrated in both. The core window — new work requested while the Full Access turn is still active and restriction is pending — was exercised only in attempt 1, with a workspace-local probe that needs no approval at either level (non-discriminating); attempt 2's active turn finished before its probe was sent. "New work cannot quietly acquire excess authority while restriction is requested or applying" is therefore not demonstrated. |
| F05 | F05 | passed | failed | Decline/accept/selection-without-approval behaviors pass by fixture. But the declined original request is rendered as both "✗ You canceled the request to run …" and "• Ran … └ (no output)" (screens F05B_DECLINED, PERM_PICKER), contradicting "the original request has an understandable disposition". The same rendering recurs in F04-attempt2, F07, F09 and others; executors noted it only as an ambiguity. |
| F06 | F06 | passed | passed | Both directions: in-flight slow op (Full Access; restricted after explicit approval) completed with both markers; change disclosed as next-turn; separate probe followed the new level (gated / ungated). |
| F07 | F07 | passed | passed | Both orders exercised; final probe matched the reported level in each; conflicting selection while the confirmation modal was open was visibly ignored, not reported as accepted. |
| F08 | F08 | passed | blocked | Cancellation at the offered point (Cancel+Enter and Esc) passed from restricted. The "benign application failure" branch was substituted with a denied out-of-workspace command, which is a command denial, not a failed permission application; per the case text that branch is "not exercised, not passed". Needs a product-authority scope decision or an exposed failure route. |
| F09 | F09 | passed | blocked | Every selection was made while idle; the starting state "active work … and a permission change selected" and "continue the task through the stated application path" were not exercised. Executor marks one outcome inconclusive. |
| F10 | F10, F10-attempt2 | passed; passed | passed (attempt 2) | Attempt 1 never reopened the session (each restart was a fresh `start`, no `resume`), so it does not meet "reopen the relevant session". Attempt 2 used the product's resume for all four variants: restricted and Full Access persisted truthfully, a pending FA→restricted change resolved to restricted, a pending command approval was restored as interrupted (not accepted, marker never written), and every post-restart probe matched the shown level. |
| F11 | F11 | passed | blocked | Only the `zai` route executed work. The `zai-anthropic` route returned only "model glm-5.2 has no catalogued maximum output token limit" and sent no inference request (mediator log has no `/api/anthropic/` request), so cross-route safety is a coverage limit; within `zai`, the F03/F04 repeats did not achieve a mid-flight selection. In-session route switch with a pending change was exercised and safe. |

Cross-cutting: all 14 attempts passed preflight (launcher, tmux child, executor's own
`pf83-probe all`) with 0 failed probes; executors read only `/opt/pf83/packet`,
`/opt/pf83/bin` and their run directory per their transcripts (reviewer to confirm).
F03 attempt 1 and F06 wrote their report to `work/evidence/result.md` instead of
`evidence/result.md`; the file was collected and is cited from there.
