# P0: packaged wallet daemon cannot start

Classification: bounded fix restoring **Shipping MVP — LIVE**, **Wallet and payments**: “Local Solana wallet, SOL and canonical USDC support, scoped signing, backup/restore, and Corbanu Plan purchase/recovery.” No authorization, signing, vault, or financial policy changes.

Base: `rust-v0.1.41`, `739897fd64527898fdddc2dec1ab15be287f620a`.
Branch: `fix/0.1.41-wallet-daemon-launch`.
Worktree: `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix`.

The release archive ships `corbanu-walletd`; the client only searches for `pfterminal-walletd`. Release checks launched the daemon directly and did not exercise automatic startup through WalletDaemonClient. A clean installation passes packaging checks but cannot open Wallet.

Repair: resolve the canonical daemon beside the running executable, retaining the same-directory legacy fallback for older package layouts. Do not search PATH or another installation. Release validation must exercise the production client from the extracted package with a fresh disposable home and no daemon already running.

## Verified result

- Original release binary reproduces the reported missing `pfterminal-walletd` error in a real PTY.
- New discovery regression fails on the original code; all 12 wallet daemon tests pass with the repair, including protocol mismatch, scoped capabilities and locking.
- Final-tree `just fmt`, `git diff --check`, five existing installer contract tests, and release workflow YAML validation pass.
- The CLI builds successfully in an isolated data-volume target (3m 09s). The locally rebuilt terminal, staged with the released package resources and canonical daemon, passes `/wallet` cold startup, cancel/reopen, missing-file error, Retry after file restoration, and cold restart in a real PTY. No compatibility link is present in this candidate.
- The original 0.1.41 binary passes the same PTY matrix after adding the compatibility link. The documented immediate recovery is verified on Linux.
- The release client probe successfully cold-starts the packaged daemon and checks complete fresh-home status twice. It rejects an executable daemon that exits zero without serving wallet requests; temporary probe and process tree are cleaned up.
- Linux/macOS/Windows release jobs now build and run the client probe against the extracted canonical-only package before direct daemon checks. Only Linux was executed locally; macOS/Windows are not claimed as tested.

[Checks, hashes and PTY evidence](wallet-daemon-launch-2026-09-10/checks.json).
Private build logs and packages: `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-package-qa` and sibling `corbanu-wallet-*.log` files.

No model calls, wallet creation, signing or payments were performed. Daemons for the disposable homes and all test TUI sessions were stopped. The first interactive fixture used a home longer than Unix socket path limits; the harness was corrected to use a short disposable path before the passing runs.

## Publication status

This is an unpublished bounded-fix candidate based on 0.1.41, not a published 0.1.42 release. No new compatibility or authorization contract, dependency, plan or sprint is required. New full release benchmark qualification, default live-repository qualification and cross-platform execution have not been completed. A separate named-human acceptance record is not available. Publish through the normal versioned release pipeline under explicit emergency release authority; do not replace immutable 0.1.41 binaries silently.
