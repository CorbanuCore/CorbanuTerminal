# Astra site/API key-generation review — 2026-09-04

Scope: read-only followup to the Terminal reconciliation review. No live payments,
key creation/revocation, site/backend edits, deployment or publication occurred.
This is source/test evidence, not verification of the production deployment.

## Source identity

- Website: `/home/pfrpc/repos/corbanucore.github.io`, local
  `be37b234735394f52145baab2768dd2479b095f4`; relevant key-generation logic compared
  with origin `48e2151`. The checked-in JS bundle matched an in-memory esbuild build.
- Backend: `/home/pfrpc/repos/CorbanuAPI`, local
  `bdede1235de9837c64782b30e2fc3146a9f42d8b`; relevant code compared with origin
  `f4ae8dc` (CorbanuCore/CorbanuPlan).
- Reviewer: explicitly requested Astra agent; primary agent inspected the site
  checkout/error flow and reviewed the supporting evidence.

## Findings

No P0 or demonstrated key-generation authorization bypass was found in this scope.

1. **P1 — misleading post-payment Solana errors.**
   `assets/src/api-checkout.ts:780–791` submits the payment, then requests more
   wallet signatures for account confirmation and key creation. Its catch handler
   retains pending-payment context only for EVM. `checkoutError`, lines 272–282,
   consequently says “No funds were sent” for a rejected subsequent signature,
   or “No payment was requested” for a failed fetch after Solana settlement.
   Retrying checkout can produce a duplicate payment. Fix the payment-stage and
   recovery boundary for every supported rail; do not infer non-payment from a
   generic wallet/provider exception.
2. **P2 — funded-wallet key recovery requires another payment.**
   `assets/src/api-checkout.ts:754` passes the count of every historical key into
   `shouldCreateApiKeyWithoutPayment` (`assets/src/evm-settlement.ts:11`). The
   backend includes revoked keys in that list (`src/store.ts:440`), so a funded
   wallet with revoked or lost keys falls through to payment instead of creating
   a replacement. Backend wallet-authorized `create_key` supports this recovery
   without another top-up. Expose key recovery independently of payment; do not
   equate historical key count with the user possessing a usable credential.

Both findings reproduced with memory-only source-extracted diagnostics. They
remain unfixed: the user requested site review, while implementation authority in
this turn covers the three previously reported Terminal findings.

## Generation and custody

- `src/token.ts:26`: 32 cryptographically random bytes per customer key.
- `src/token.ts:35`: keyed HMAC-SHA256 lookup/storage hash.
- `src/postgres-store.ts:652`: stores the hash, not customer-key plaintext.
- `src/wallet-auth.ts:178`: validates wallet, exact operation, expiry and signature.
- `src/postgres-store.ts:1196`: atomic nonce consumption prevents proof replay.
- The browser reveals plaintext using `textContent`, clears the reveal on close
  or Escape, and stores transaction references rather than API keys in browser
  storage. These protections do not eliminate the checkout-recovery findings.

## Supporting tests

Run by Astra against the local commits above, all successful:

```bash
# CorbanuAPI
node_modules/.bin/tsx --test tests/wallet-auth.test.ts tests/api-balance.test.ts tests/store.test.ts
# 39 passed, 0 failed

# corbanucore.github.io
node --test tests/*test.mjs
# 10 passed, 0 failed
node_modules/.bin/tsc -p tsconfig.api.json --noEmit
# exit 0
```

Original outputs were retained in the review conversation (backend session 86394,
site test chunk 9b6d13, typecheck session 73112), not as repository log files.
No live browser/provider-signature, on-chain settlement, production PostgreSQL,
or deployment-version claim is made.
