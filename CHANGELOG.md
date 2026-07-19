# PFTerminal 0.1.17

## Fixed

- Restored `/wallet` in standalone installations by shipping the required
  `pfterminal-walletd` companion executable on macOS, Linux, and Windows.
- Made the wallet daemon a mandatory part of the canonical PFTerminal package
  contract, so package validation fails when the runtime dependency is absent.
- Extended every platform release smoke test to verify that the packaged wallet
  daemon exists and starts successfully.

## Qualification status

- The release package is unpacked into a fresh standalone installation and the
  packaged `pfterminal` process starts its packaged wallet daemon before the
  release is published.
- The complete multi-platform package matrix remains a release gate.

Previous release: 0.1.16.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
