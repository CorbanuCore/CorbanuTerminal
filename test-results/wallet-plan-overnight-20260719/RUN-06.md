# Run 06 — narrow rendering, upgrade comprehension, and provider state

- Stories: US-3, US-5, US-6
- Qualified commit: `a374ab638b3ddce2230e3cb1bcc3ad01d87804a7`
- Binary SHA-256: `ea59e11260f6e3652507a3d6b19a8ab612f364df5ec38353d6acce42cb095270`
- Terminal: 69×45, then 94×57
- Verdict: PASS

## Required evidence

Inspect receipt, `/usage`, exact timestamps, reset descriptions, upgrade copy,
and provider/model state at both sizes. Do not purchase another period. Every
material field must wrap without clipping, and opening wallet surfaces must not
change the selected provider.

## Evidence

The final binary started at 69×45 with 47,506,692 settled and zero in-flight
tokens. The locked wallet showed network, canonical assets, fee guidance,
0.010000 SOL, 0 USDC, the 200-USDC Pro period, exact weekly/monthly totals, and
complete ISO boundaries without clipping. The receipt preserved and wrapped
the complete settlement transaction, amount, active interval, and post-payment
balance. `/usage` showed prepaid non-recurring spend, used/in-flight/limit/
remaining totals, percentage, relative reset descriptions, and exact reset/end
timestamps.

The first exact-binary attempt found WUX-016: the unwrapped upgrade sentence
clipped after a dangling `after`. On the rebuilt final binary the same Pro
no-higher-tier screen rendered at 69×45 as:

`Choose a tier above Pro. It starts 2026-08-19T07:30:37.912Z`

`after the paid period you already own.`

The preservation sentence and disabled highest-tier explanation were complete.
After resizing live to 94×57, the same content reflowed cleanly with no stale
or duplicated plan state. No purchase was possible or attempted.

The model footer remained `Ambient GLM 5.2 standard` throughout. `/providers`
resolved `PfTerminal Plan` to `Active · Pro plan · 4RBjjHA…hNxSGn`; opening
wallet, receipt, usage, upgrade, and provider surfaces did not alter the active
model/provider. No inference request or chain transaction occurred.

Run 6 passes US-3, US-5, and US-6 on the final binary.

## Renewed responsive replay — WUX-020

- Replayed: 2026-07-19T14:13Z–14:16Z
- Commit: `c0bbab5727fa8268d40be727959702165f449677`
- Binary SHA-256: `1b70e4ed3a51a52bdab833d2f4d72fc91d96b96f6ce93eea6c70e3f66dc63b94`
- Wallet-daemon SHA-256: `337bf8ff83b444183b5631247544b2411b7f10ea2283ee555010716d8a03f39c`
- Terminal: isolated tmux sockets at true 69×45 and 94×57 pane sizes
- Verdict: PASS for the responsive replay; same-binary seven-run count restarts

The predecessor candidate clipped the closing `)` from the weekly percentage
because wallet headers had been pre-wrapped to 64 columns before the popup
reported a 63-column content area. The repaired candidate rendered `/usage`
and `/wallet` at both required sizes. At 69×45, the complete weekly line ended
in `(9.6%)`, the monthly percentage wrapped intact to `(2.4%)`, and exact reset
and period timestamps remained visible. At 94×57, both usage windows and the
wallet's spend, balance, allowance, reset, receipt, unlock, and upgrade copy
rendered completely. The model/provider footer remained unchanged.

Automated coverage rendered the same semantic wallet lines at 36, 63, 64, and
90 content columns, compared the normalized output to the complete source
text, and reviewed narrow/wide snapshots. The wallet-focused TUI suite passed
35/35. No inference request, wallet unlock, payment, chain transaction, or
token-accounting change occurred during the live replay.
