# PFTerminal 0.1.18

## Fixed

- Prevented Kimi Code from silently pausing after a complete response or
  remaining in a false `Working` state after it had already answered.
- Made wallet removal update the visible wallet state before credential cleanup,
  eliminating the stale-wallet screen and restore-loop behavior.
- Built the Linux release package on Ubuntu 22.04 so the downloadable binary
  runs on the supported glibc baseline instead of requiring a newer build host.

## Qualification status

- The exact release candidate completed a native Apple Silicon `/wallet` flow:
  create a Solana wallet, receive SOL and USDC, purchase the 1-USDC Starter
  plan, run paid inference, restart PFTerminal, and recover the active plan,
  receipt, and authoritative usage.
- Funding and purchase transactions finalized on Solana mainnet with no chain
  errors, and the post-payment wallet reconciled to 0.005 SOL and 0 USDC.
- The release workflow continues to require the packaged `pfterminal-walletd`
  companion process and package launch smoke tests on every platform.

Previous release: 0.1.17.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
