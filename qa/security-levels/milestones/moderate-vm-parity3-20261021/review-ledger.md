# Review ledger (utm-par3)

Budget: 5 rounds (brief; coordinator rule, 2026-10-15). Independent read-only Opus 5.5 High via the installed binary;
prompts, outputs and start/end times in [review/](review/). The record of amendment v9 (../moderate-amendment-v9-20261021/)
is reviewed with it.

| Round | Scope | Head | Verdict | Changes applied |
| --- | --- | --- | --- | --- |
| 1 | evidence check of both records | `f6d4472622` | CHANGES | 3 required, all applied: tester deviations completed (T17-tart1, T17-tart2, T17-tart3); the child's reasoning display reported as finding #1266; `expected.json` added to the harness differences. Optional applied: #1175 origin wording, the assistant-message flags, the Orc in the Tart parity T17, latency rows per group, the UTM power-off, the tools that read the unexported `collected.tar`, the v9 README on S17 |
| 2 | the round 1 corrections and both records | `d579a780df` | APPROVE | no required correction; 4 optional applied (T17-tart3: two "Remove managed crew" confirmations; T18-utm1/utm3 deviation rows; the latency row is the upper median, Tart windows include gate requests; decision 2: how the key file was touched, and `auth.json` in the same folder), checked in a short round 3 |
| 3 (short) | the round 2 optional fixes | `002055287a` | APPROVE | no required correction; 1 optional applied (the latency note: UTM's first window, T17-utm1, also includes gate requests), checked in a short round 4 |
| 4 (short) | the round 3 optional fix | `9e11fb0cbe` | APPROVE | none required or optional. 4 of 5 rounds used. The commit after it adds only this ledger row and the round 4 output and logs. |
