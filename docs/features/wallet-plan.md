# Wallet and Corbanu Plan

## The pain

A trader should not need a separate wallet utility, payment site, and inference
credential workflow just to fund and use an AI plan. Corbanu Terminal keeps
local custody, balances, signing, Plan purchase, and Plan recovery in one
reviewable flow.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Wallet and payments: Local Solana wallet, SOL and canonical USDC support, scoped signing, backup/restore, and Corbanu Plan purchase/recovery.” |
| Exact Plan heading | **Corbanu Plan — LIVE** |
| Plan excerpt | “Corbanu Plan is wallet-native, one-calendar-month prepaid inference purchased through x402, normally using canonical USDC on Solana.” |

## Create or restore a wallet

1. Run `/wallet`.
2. Choose **Create wallet** or **Restore wallet**.
3. Save the recovery material from the secure view.
4. Set the local wallet passcode.
5. Use **Receive** to fund the displayed Solana address with the correct assets.

Keep enough SOL for transaction fees unless checkout explicitly reports that
fees are sponsored. Never paste recovery material into chat.

## Buy and use a Plan

1. Open `/wallet` and choose **Buy Corbanu Plan**.
2. Unlock the wallet for the prompted action.
3. Select a tier.
4. Review the exact USDC payment and period.
5. Confirm the transaction.
6. Wait for a durable receipt or refreshed Plan state.
7. Run `/model` and choose a model offered through Corbanu Plan.

| Tier | Price | Weekly allowance | Monthly allowance |
| --- | ---: | ---: | ---: |
| Starter | 1 USDC | 250K | 1M |
| Basic | 20 USDC | 5M | 20M |
| Power | 50 USDC | 12.5M | 50M |
| Pro | 200 USDC | 50M | 200M |

Every tier uses the same model catalog and differs by allowance.

| Models | Backend | Privacy boundary |
| --- | --- | --- |
| GLM 5.2, Kimi K2.7 Code | Ambient | Private, Corbanu-controlled inference |
| DeepSeek V4 Pro, Claude Fable 5 | xAPI | Non-private, third-party inference |

**x402 is payment. xAPI is inference.** The customer authenticates to Corbanu
Plan rather than directly to xAPI.

## Wallet and Plan controls

- **Unlock** grants signing capability to the current TUI for one action or the
  selected duration. **Lock** revokes it.
- **Plan details** shows tier, token use, limits, reset dates, and queued periods.
- **Upgrade** buys the period shown in the confirmation. It does not imply an
  immediate tier change when the UI shows a future period.
- **Recover existing plan** signs an ownership proof and sends no USDC.
- **Disconnect Corbanu Plan** removes the local Plan credential while keeping
  the wallet and paid period.
- **Remove wallet from this device** removes local custody and the local Plan
  credential. It does not move on-chain funds or cancel a paid period.
- **Back up recovery material** requires the wallet passcode and uses the secure
  view.

If settlement is ambiguous, refresh `/wallet` and inspect the latest receipt
before attempting another payment.

## Custody boundary

The wallet and vault are separate. The wallet manages signing keys, balances,
and Plan ownership. The vault manages service credentials. A model response is
never authorization to reveal recovery material, sign an unrelated message, or
send funds.
