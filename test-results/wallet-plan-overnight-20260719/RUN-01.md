# Run 01 — funded locked account, receipt, usage, and spend comprehension

- Stories: US-2, US-3, US-5
- Started: 2026-07-19T03:32:49Z
- Commit: `1f1391d01ac7533dbdf453589e7edb61a77a2c52`
- Binary SHA-256: `df994701f1a4af853ef81b128ea3cff206345ae0f55b9364c98ad5f91e57d4aa`
- Terminal: tmux `pft_wallet_receipt_qual`, 94×57; narrow replay 69×45
- State: funded internal wallet, locked; 0.010000 SOL; 4.00 USDC;
  Starter active through 2026-08-19; Basic queued through 2026-09-19.
- Spend authorization used: read-only, zero expected SOL/USDC delta.

## Steps and evidence

1. Restart the TUI on the exact binary and open `/wallet` while locked.
2. Refresh and verify authoritative SOL, USDC, active plan, queued plan, weekly
   remaining, monthly remaining, and reset/period timestamps.
3. Open `View latest plan receipt` and verify 20 USDC, Basic queued status,
   complete wrapped transaction signature, both plan boundaries, current
   Starter state, and 4.00-USDC remaining balance.
4. Repeat receipt and usage inspection at 69×45.
5. Open `/usage` and compare used/remaining/reset data with `/wallet`.
6. Confirm that no wallet unlock or payment occurred and balances did not move.

## Verdict

IN PROGRESS.
