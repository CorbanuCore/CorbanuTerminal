# PFTerminal 0.1.16

## Added

- Added a locally encrypted Solana wallet with create, restore, recovery backup, SOL and canonical
  USDC balances, one-action or timed unlocks, explicit locking, and device removal.
- Added crypto-native PfTerminal Plan purchase, recovery, upgrade, disconnect, receipt, allowance,
  reset, and usage flows directly in `/wallet`, with plan status exposed through `/providers`.
- Added a durable PfTerminal Plan gateway with verified USDC settlement, wallet ownership proofs,
  scoped API keys, weekly and monthly token limits, measured inference accounting, PostgreSQL
  persistence, and operator revenue classification.
- Added the bundled `pfterminal-help` skill for concise product guidance across `/wallet`, `/vault`,
  `/providers`, `/gpu`, `/spawn`, `/orchestrate`, panes, and troubleshooting.

## Fixed

- Preserved wallet and plan state across creation, restoration, unlock retries, cancellation,
  credential replacement, process restarts, and ambiguous payment recovery.
- Prevented duplicate payment confirmations, stale wallet summaries, overlapping wallet surfaces,
  lost unlock continuations, and misleading recovery or upgrade actions.
- Settled measured inference usage after long streams and transport closure while recovering orphaned
  reservations conservatively after gateway restarts.
- Added explicit native Windows installation guidance so PowerShell users no longer receive the
  Unix-only `curl -fsSL ... | sh` bootstrap.

## Qualification status

- Wallet, wallet-daemon, TUI wallet/provider, bundled-skill, and gateway suites are release gates for
  this version.
- Seven fresh wallet/plan development sessions exercised create, restore, purchase, recovery,
  upgrade, disconnect, long-turn inference, and failure recovery. The final deployed-gateway
  inference completed successfully and its measured usage reconciled in the accounting ledger.
- The complete multi-platform package matrix is built and smoke-tested by the release workflow; no
  platform asset is considered published unless that workflow completes successfully.

Previous release: 0.1.15.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
