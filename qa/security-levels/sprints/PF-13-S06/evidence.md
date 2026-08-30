# PF-13-S06 credential usage reservation evidence

- Date: 2026-08-30
- Owner: Pauli — credential reservations lane
- Branch: `feat/p0-security-credential-reservations`
- Dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`
- Allocation commit: `e0c23fe95165636d621dae8c16a5366c4f7250ac`
- Implementation candidate: recorded in the final handoff after commit
- Contract versions: credential capability v1; additive credential usage v1

## Delivered contract

The existing `BoundedGrant`, authorization request, actor/session/task context,
opaque capability token, and digest-only public ID remain the sole authority.
Usage v1 adds exact model binding and fixed request, token, byte, and spend
dimensions with per-request and aggregate ceilings. Legacy capabilities retain
their accepted one-shot behavior; metered capabilities must reserve worst-case
usage atomically before dispatch.

Only trusted Core metering can settle a reservation. Completed and partial
outcomes charge authenticated measured usage; cancellation charges the request
attempt but releases unused token/byte/spend holds; unknown outcomes charge the
entire reservation. Pending reservations survive expiry or revocation only to
permit fail-closed settlement, while new reservations are denied. Duplicate
settlement is idempotent and retries cannot replenish spent request authority.

The reservation bearer is private, non-serializable, redacted in Debug output,
and zeroized on drop. This lane does not resolve a credential, return secret
bytes, activate transport, or implement broker IPC.

## Changed paths

- `codex-rs/security-policy/src/credential.rs`
- `codex-rs/security-policy/src/credential_tests.rs`
- `codex-rs/security-policy/src/grant.rs`
- `codex-rs/core/src/security/credential_capability.rs`
- `codex-rs/core/src/security/credential_capability_tests.rs`
- `docs/sprints/current/p0-security-levels/pf-13-s06-credential-usage-reservations.md`
- `qa/security-levels/sprints/PF-13-S06/evidence.md`

PF-13-S01 archive and historical evidence were not modified.

## Deterministic verification

All Cargo homes, targets, temporary files, logs, and review artifacts were
placed under `/Volumes/CorbanuDrive/Corbanu/.codex-work/credential-reservations/`.

- `just fix -p codex-security-policy`: pass.
- `just fix -p codex-core`: pass; one unrelated automatic formatting edit was
  removed before the final diff.
- `just fmt` and `just fmt-check`: pass.
- `just test -p codex-security-policy credential`: 8 passed.
- `just test -p codex-security-policy grant`: 5 passed.
- `just test -p codex-security-policy`: 47 passed.
- `just test -p codex-core credential_capability`: 15 passed; 2,348 Core unit
  tests, 1,094 integration tests, and 4 header tests were outside the filter.
- `python3 docs/plans/check.py`: pass.
- `python3 docs/sprints/check.py`: pass.
- `git diff --check`: pass.

The focused Core selection contains four new reservation tests and the eleven
accepted PF-13-S01 tests. New cases cover per-request and aggregate exhaustion,
concurrent over-reservation, partial/cancelled/unknown settlement, retry request
charging, duplicate settlement, forged bearer/excess metering, changed
operation/model/resource, and settlement after expiry/revocation.

## TMUX smoke and independent review

Final session names, artifact paths, SHA-256 hashes, exact model/effort, and the
review disposition are recorded after the exact implementation candidate is
committed and exercised. TUI behavior is not changed by this accounting-only
contract; the smoke proves candidate startup, separate text/Enter input, and a
completed response under trace logging.

## Integration-owner handoff

The shared `codex-rs/security-policy/src/lib.rs` export surface is outside this
lane. Integration should re-export `CREDENTIAL_USAGE_SCHEMA_VERSION`, merge the
other disjoint candidates, and rerun the complete policy plus focused Core
credential suites and governance checks. PF-13-S03/PF-27 may later consume the
opaque reservation at an authenticated Core-to-broker boundary; they must not
bypass `reserve`/`settle`, serialize bearer bytes onto model/public surfaces, or
introduce a raw-secret return path. Shared manifests, Bazel/lock files,
navigation, plan mutation, archival, and transport activation remain untouched.
