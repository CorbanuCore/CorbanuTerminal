# Wallet plan overnight defects

| ID | Run/story | Severity | Failed boundary | Evidence | Fix / regression | Replay |
| --- | --- | --- | --- | --- | --- | --- |
| WUX-001 | Pre-run / US-3 | P1 | Purchase result discarded x402 settlement and described queued plans as activated | Live 94×57 receipt inspection before `1f1391d01`; targeted tests | `1f1391d01` preserves settlement, reconciles account, persists latest receipt, distinguishes queued state, wraps receipt fields | Pending Run 1 and Run 6 |
| WUX-002 | Run 1 / US-3 | P1 | Receipt `Done` opened `/wallet` behind the active receipt and consumed its one-shot action | Live `pft_wallet_receipt_qual` at 69×45; repeated Done left receipt visible | Dedicated close-receipt event dismisses the modal before refreshing wallet; regression added | Pending fresh Run 1 replay |

Append findings only. A concrete phrase or input is evidence, never the repair.
