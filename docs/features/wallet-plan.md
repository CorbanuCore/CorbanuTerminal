# Wallet and Corbanu API

## The pain

A trader should not need a separate wallet utility, payment site, and inference
credential workflow just to fund and use metered inference. Corbanu Terminal keeps
local custody, balances, signing, API funding, and API-key management in one
reviewable flow.

## Product contract

| Field | Value |
| --- | --- |
| Status | **0.1.38 integration candidate** |
| Exact product-spec heading | **Corbanu API — TO BUILD** |
| Product decision | Replace new plan sales and legacy entitlements with a wallet-funded, dollar-denominated Corbanu API balance. |
| Funding contract | One canonical USDC adds one dollar of API balance; there are no tiers, renewals, or expiring allowances. |

## Create or restore a wallet

1. Run `/wallet`.
2. Choose **Create wallet** or **Restore wallet**.
3. Save the recovery material from the secure view.
4. Set the local wallet passcode.
5. Use **Receive** to fund the displayed Solana address with the correct assets.

Keep enough SOL for transaction fees unless checkout explicitly reports that
fees are sponsored. Never paste recovery material into chat.

## Fund and use Corbanu API

1. Open `/wallet` and choose **Corbanu API**.
2. Unlock the wallet for the prompted action.
3. Choose **Top up balance** and enter any positive canonical-USDC amount.
4. Review the exact USDC payment and dollar credit.
5. Confirm the transaction.
6. Save the first API key from the one-time secure view.
7. Run `/model` and choose a model offered through Corbanu API.

The account view shows the current dollar balance, per-model input, cached-input,
cache-write, and output prices, plus the privacy boundary. API keys share the
wallet's balance but retain separate creation, last-use, revocation, request,
and spend attribution.

## Wallet and API controls

- **Unlock** grants signing capability to the current TUI for one action or the
  selected duration. **Lock** revokes it.
- **Top up balance** signs and submits only the exact confirmed USDC amount.
- **Manage API keys** lists active key prefixes without revealing plaintext.
- **Create API key** returns plaintext once through a secure non-transcript view.
- **Disconnect Corbanu API** removes only the stored credential from this device.
- **Remove wallet from this device** removes local custody and the stored API
  credential. It does not move on-chain funds or alter the dollar balance.
- **Back up recovery material** requires the wallet passcode and uses the secure
  view.

If settlement is ambiguous, refresh `/wallet` and inspect the server-authoritative
balance before attempting another payment.

## After upgrading Terminal

Terminal checks the daemon protocol before sending a passcode or wallet operation.
If `/wallet` reports `daemon_upgrade_required`, an older detached daemon may still
be running; closing and reopening only the TUI does not replace it.

Let existing payments finish and verify their outcomes. Close Terminal sessions
using the affected home, then use the operating system's process manager to stop
only the `pfterminal-walletd` or `corbanu-walletd` process with the matching
`--codex-home` argument shown in the error. Reopen Terminal to start its matching
daemon. Do not delete wallet files, the socket, or the ownership lock. Do not pay
again while a previous payment's outcome is unknown.

`/wallet lock` remains available against legacy daemons, but locking is revocation,
not proof that a payment has settled. Automatic process termination is deliberately
not used: older daemons cannot safely acknowledge that all work has drained.

Environment-provided Corbanu keys are recognized by provider setup, model
eligibility and read-only API balance/usage views. They are not copied into the
credential store. Disconnecting the stored credential does not unset a shell key;
remove it from the launching environment and restart Terminal if needed.

## Custody boundary

The wallet and vault are separate. The wallet manages signing keys, balances,
and API account ownership. The vault manages service credentials. A model response is
never authorization to reveal recovery material, sign an unrelated message, or
send funds.
