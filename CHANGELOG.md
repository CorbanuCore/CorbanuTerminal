# PFTerminal 0.1.19

## Fixed

- Prevented updates and headless launches from making an existing encrypted
  credential vault unreadable when the OS keyring is temporarily unavailable.
- Existing vaults now validate every available key source against their
  ciphertext and never generate or overwrite a key during recovery.
- Stale file-fallback keys can no longer shadow a valid OS-keyring key, while a
  valid fallback still recovers a vault when the OS-keyring entry is stale.

## Qualification status

- Regression coverage reproduces the update-shaped macOS failure: an existing
  vault plus an unavailable OS keyring cannot create a replacement fallback.
- Candidate-key tests cover stale fallback, stale primary, missing key, and two
  conflicting wrong keys; every failure path preserves ciphertext and keys.
- The secrets, vault, and provider-key integration suites pass, along with
  warnings-as-errors linting for the changed crate.

Previous release: 0.1.18.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
