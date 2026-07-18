# Ambient Crypto Gateway

This service accepts one-month PfTerminal subscriptions through x402 on Solana, binds the
settlement to the paying wallet, and issues revocable PfTerminal API keys after a signed-wallet
challenge. Customer keys proxy only the supported Ambient inference routes; the operator's
Ambient credential stays server-side.

End customers do not need an Ambient browser account. The operator must supply an Ambient
account and upstream credential that are approved for the intended proxy arrangement.

## Current boundary

The implementation is suitable for internal and funded end-to-end qualification. Public launch
also requires one of these economic controls:

- a dedicated Ambient team/credential and subscription for each customer; or
- an agreed plan-to-usage schedule plus durable request/token metering in this gateway.

Do not expose a shared upstream balance without one of those controls. Payment and entitlement
expiry alone do not cap inference consumption.

## Plans

| Plan | Payment | Entitlement |
| --- | ---: | --- |
| `starter` | 1 USDC | one calendar month |
| `basic` | 20 USDC | one calendar month |
| `power` | 50 USDC | one calendar month |
| `pro` | 200 USDC | one calendar month |

Additional purchases queue consecutive calendar-month periods, with a maximum of 12 months from
the settlement time. Plan changes therefore never discard an already-paid period.

## Service configuration

The service fails at startup when any required value is absent or malformed.

| Variable | Purpose |
| --- | --- |
| `DATABASE_URL` | PostgreSQL connection URL |
| `AMBIENT_API_KEY` | operator's upstream credential |
| `AMBIENT_BASE_URL` | optional upstream URL; defaults to `https://api.ambient.xyz` |
| `PFT_AMBIENT_TOKEN_PEPPER` | at least 32 characters, used to HMAC customer keys |
| `PFT_AMBIENT_PUBLIC_BASE_URL` | externally visible gateway origin used in signed challenges |
| `PFT_X402_NETWORK` | exact Solana mainnet or devnet CAIP-2 identifier |
| `PFT_X402_PAY_TO` | Solana receiving address |
| `PFT_X402_FACILITATOR_URL` | optional override; defaults by network |
| `PFT_AMBIENT_HOST` | bind address; defaults to loopback |
| `PFT_AMBIENT_PORT` | port; defaults to `4021` |

Mainnet defaults to the PayAI facilitator. Devnet defaults to the x402.org test facilitator. A
remote Ambient, facilitator, or public gateway URL must use HTTPS.

```sh
corepack pnpm --filter @agticorp/ambient-crypto-gateway build
corepack pnpm --filter @agticorp/ambient-crypto-gateway start
```

`GET /healthz` is process liveness. `GET /readyz` also checks PostgreSQL.

## Customer flow

Generate a dedicated Solana payer wallet without printing its secret:

```sh
PFT_SOLANA_WALLET_FILE="$HOME/.pfterminal/ambient-payer.json" \
  corepack pnpm --filter @agticorp/ambient-crypto-gateway wallet:generate
```

The file is created with mode `0600` and refuses overwrite. For mainnet, fund its displayed address
with the plan price in Solana USDC. The current facilitator advertises itself as fee payer; a small
SOL balance is optional contingency rather than a hard requirement.

Pay and create a customer key:

```sh
PFT_AMBIENT_GATEWAY_URL=https://gateway.example \
PFT_SOLANA_WALLET_FILE="$HOME/.pfterminal/ambient-payer.json" \
PFT_AMBIENT_PLAN=starter \
PFT_AMBIENT_KEY_OUTPUT="$HOME/.pfterminal/ambient-customer-key.json" \
PFT_AMBIENT_RECEIPT_OUTPUT="$HOME/.pfterminal/ambient-receipt.json" \
  corepack pnpm --filter @agticorp/ambient-crypto-gateway qualify:live
```

The purchase command first reads the x402 challenge and checks the payer's token balance. Set
`PFT_SOLANA_RPC_URL` to use a private Solana RPC endpoint instead of the public network default.

An already-subscribed wallet can recover access without paying again:

```sh
PFT_AMBIENT_GATEWAY_URL=https://gateway.example \
PFT_SOLANA_WALLET_FILE="$HOME/.pfterminal/ambient-payer.json" \
PFT_AMBIENT_KEY_OUTPUT="$HOME/.pfterminal/ambient-customer-key.json" \
  corepack pnpm --filter @agticorp/ambient-crypto-gateway key:issue
```

The raw wallet, upstream credential, and customer key must never be pasted into a prompt or log.

## Qualification

The default suite uses no funds:

```sh
corepack pnpm --filter @agticorp/ambient-crypto-gateway typecheck
corepack pnpm --filter @agticorp/ambient-crypto-gateway test
corepack pnpm --filter @agticorp/ambient-crypto-gateway build
```

Set `TEST_DATABASE_URL` to include the PostgreSQL concurrency and persistence tests. Set
`RUN_LIVE_X402_DISCOVERY=1` to contact the live facilitator and verify the mainnet challenge without
settling a transaction. A full mainnet qualification is complete only after the on-chain receipt,
wallet-signed key creation, non-streaming inference, and streaming inference all pass.

## Security properties

- Settlement transaction IDs are globally idempotent and cannot be rebound to another wallet or
  plan.
- Per-wallet database locks serialize concurrent purchases.
- Signed-wallet nonces are atomically consumed, preventing parallel replay.
- Customer keys are random opaque values stored only as HMAC hashes and can be revoked.
- Customer authorization is replaced with the upstream credential; response cookies and arbitrary
  upstream headers are not forwarded.
- Inference bodies are JSON-only and capped at 2 MiB.
- Secret files refuse overwrite and require owner-only permissions.
