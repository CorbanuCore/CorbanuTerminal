# Review ledger (utm-par2)

Budget: 5 rounds (coordinator rule, 2026-10-15). Independent read-only Opus 5.5 High via the installed binary; prompts,
outputs and start/end times in [review/](review/).

| Round | Scope | Head | Verdict | Changes applied |
| --- | --- | --- | --- | --- |
| 1 | evidence check of the whole record | `f470787768` | CHANGES | 8 required and 9 optional corrections, all applied (recommendation stated per the frozen rule; MOD-22 and #1139 claims; MOD-21 route checked in the product session files, Tart ones added under `tart-reference/`; tester-only note in UTM T17; MOD-26; transport header; seal assets disclosed) |
| 2 | the round 1 corrections and the whole record | `3269d3cd3a` | CHANGES | 2 required (MOD-21: the session files show the route, not the wrapper, claims weakened and the Tart report left open; isolation records cite the as-assembled `effective-policy.txt`, kept) and 5 optional (D95 check method, 429 sequence, seal assets incl. tmux/libevent, results.json wording, window overlap), all applied |
| 3 | the round 2 corrections and the whole record | `8d6c780779` | APPROVE | no required correction; 5 optional wording fixes applied (origin only on the first typed task; the wrapper unrecorded for every input; decision 2's check shows the route only; seal assets in the policy file; policy erratum), so a short round 4 checks them |
| 4 (short) | the round 3 wording fixes | `2c7c79e6d3` | APPROVE | no required correction; 3 optional wording fixes applied (what the excerpt scan covers; 'unchanged repeats'; the current policy file's SHA-256), checked in a short round 5 |
| 5 (short) | the round 4 wording fixes | `5bea9d2c9c` | APPROVE | none required or optional. Budget spent (5 of 5). The commit after it adds only this ledger row and the round 5 output and logs. |
