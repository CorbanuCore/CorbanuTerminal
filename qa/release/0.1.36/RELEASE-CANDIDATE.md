# Corbanu Terminal 0.1.36 release candidate

Date: 2026-08-30 UTC

Branch: `feat/corbanu-api-wallet`

Release target: `rust-v0.1.36`

Release authorization: explicitly authorized by a human with release authority
on 2026-08-30. Under Corbanu development policy 1.5, the release must be pushed
and incomplete qualification evidence must be disclosed rather than used as an
agent-created veto.

## Included user-facing work

- Adds the wallet-funded, at-cost Corbanu API balance and key-management flow.
- Adds the Corbanu API model catalog and inline model prices.
- Recommends GLM 5.3 Flash and retains Ambient GLM 5.2 in the Corbanu picker.
- Adds the Corbanu crew preset.
- Replaces the unavailable Fable route with Kimi K3 through the Corbanu API.
- Includes current `main` security and reliability changes through the branch's
  merge from `origin/main` on 2026-08-30.

## Recorded qualification evidence

- Corbanu API backend tests: 110 passed before the production Kimi deployment.
- Backend typecheck and build: passed.
- Provider catalog tests: 60 passed.
- Model-manager tests: 62 passed.
- Focused TUI tests: 4 passed.
- Production true-TUI Kimi request: passed with exact `KIMI_OK` output at
  10.9 tok/s using `corbanu/kimi-k3`.
- Production Corbanu API model list exposes Kimi K3 and no Fable route.
- The release workflow builds and smoke-tests the packaged binaries on Linux,
  macOS, and Windows before publishing assets.

## Disclosed incomplete evidence

- The benchmark bootstrap cycle remains pending.
- There is no separate named-human acceptance artifact beyond the explicit
  human release authorization recorded above.
- After merging current `main`, `docs/sprints/check.py` reports duplicate sprint
  identifiers and allocation conflicts between the independently developed P0
  security and Corbanu API plans. This is planning-ledger debt; it is disclosed
  and is not represented as passing.

These disclosures do not override the explicit human instruction to push this
release. The multi-platform release workflow remains responsible for building,
smoke-testing, and attaching the actual release artifacts.
