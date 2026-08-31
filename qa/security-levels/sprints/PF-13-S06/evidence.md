# PF-13-S06 credential usage reservation evidence

- Date: 2026-08-30
- Owner: Pauli — credential reservations lane
- Branch: `feat/p0-security-credential-reservations`
- Dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`
- Allocation commit: `e0c23fe95165636d621dae8c16a5366c4f7250ac`
- Initial implementation: `3250bf2345cb19edd8f867bbbd1ac99fc8104baf`
- Lifecycle correction candidate: `e274c266a`
- Contract versions: credential capability v1; additive credential usage v1

## Delivered contract

The existing `BoundedGrant`, authorization request, actor/session/task context,
opaque capability token, and digest-only public ID remain the sole authority.
Usage v1 adds exact model binding and fixed request, token, byte, and spend
dimensions with per-request and aggregate ceilings. Legacy capabilities retain
their accepted one-shot behavior; metered capabilities must reserve worst-case
usage atomically before dispatch.

Only trusted Core metering can settle a reservation. Completed and partial
outcomes charge authenticated measured usage. Pre-dispatch cancellation charges
the request attempt and releases unused token/byte/spend holds; once dispatch is
authorized, an unmeasured cancellation charges the entire reservation, as does
an unknown outcome. Pending reservations may settle after revocation only until
the capability deadline; at expiry every abandoned hold is force-charged Unknown
before the capability is reclaimed. Duplicate settlement is idempotent, settled
history does not consume the active-reservation cap, and aggregate request count
is hard-bounded at 1,024 so retained idempotency records remain bounded.

The reservation bearer is private, non-serializable, redacted in Debug output,
and zeroized on drop. Core authenticates it and authorizes at most one active
broker dispatch; repeat or post-settlement authorization fails closed. This lane
does not resolve a credential, return secret bytes, activate transport, or
implement broker IPC.

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
- `just test -p codex-core credential_capability`: 17 passed; 3,440 tests across
  the selected Core binaries were outside the filter.
- `python3 docs/plans/check.py`: pass.
- `python3 docs/sprints/check.py`: pass.
- `git diff --check`: pass.

The focused Core selection contains six new reservation tests and the eleven
accepted PF-13-S01 tests. New cases cover per-request and aggregate exhaustion,
concurrent over-reservation, partial/cancelled/unknown settlement, retry request
charging, duplicate settlement, forged bearer/excess metering, changed
operation/model/resource, settlement after revocation, expiry-bounded Unknown
reclamation, active-only reservation capacity, the 1,024 request ceiling, and
single-use/state-checked broker authorization.

Final post-correction test transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/credential-reservations/fixes-tests-clean.txt`
(`sha256:3b1b4db5794a03f25bd6912b261300b6e14e3305ed3bff70b57c573c092ab23b`).

Final governance transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/credential-reservations/governance-final.txt`
(`sha256:2a7ed587a4bf662ff92176bd3b1ff0bbc4e531d473f8422636a63b5690a30c86`).

## TMUX smoke and independent review

Exact correction-candidate smoke session `pf13-credential-fix-smoke` ran the
lane-built binary under `RUST_LOG=trace`, read-only sandbox, and lane log directory
`logs/fix-smoke`. Prompt text and Enter were sent separately; Corbanu returned
`PF13-S06-FIX-SMOKE-OK`. Binary SHA-256:
`f6044b8d274d409e590aacf31f5789a29299545225b0ab451536781334714dfa`.
Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/credential-reservations/fix-smoke-transcript.txt`
(`sha256:b2da38771a78acf2b466dc3576e1dc6fcd8376e26b3ede25486b3040b1c51d4f`).

Initial independent review session `pf13-credential-opus5-review` used Corbanu
Terminal with `claude-opus-5-plan`, provider `claude-plan`, reasoning effort
`max`, read-only sandbox, and no approvals. It found three lifecycle issues:
stale reservation reclamation, settled-history capacity accounting, and a
settlement-agnostic vault-reference accessor. All were corrected in `e274c266a`.
Initial transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/credential-reservations/review/initial-review-transcript.txt`
(`sha256:f17c87a1dcf66af0576282a11d9f172de8b8d57fa8313a6b2f50ccc49a888b73`).

Clean follow-up review session `pf13-credential-opus5-final-review` reviewed
exact candidate `fbfe35c53363422166fdd290e04a67d2fb7de4fc` using Corbanu Terminal
with `claude-opus-5-plan`, provider `claude-plan`, reasoning effort `max`,
read-only sandbox, and no approvals. It independently rechecked the corrected
lifecycle, authorization, integrity, race/atomicity, arithmetic, resource-bound,
serde-compatibility, bearer-exposure, and test surfaces and concluded exactly:
`No actionable findings.` Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/credential-reservations/review/final-review-transcript.txt`
(`sha256:494a3a15c75cc3499295aca6c560f38809219c3b97d0e99701d7f7018ad373fb`).

## Integration-owner handoff

The shared `codex-rs/security-policy/src/lib.rs` export surface is outside this
lane. Integration should re-export `CREDENTIAL_USAGE_SCHEMA_VERSION` and
`CREDENTIAL_USAGE_MAX_REQUESTS`, merge the
other disjoint candidates, and rerun the complete policy plus focused Core
credential suites and governance checks. PF-13-S03/PF-27 may later consume the
opaque reservation at an authenticated Core-to-broker boundary; they must not
bypass `reserve`/`settle`, serialize bearer bytes onto model/public surfaces, or
introduce a raw-secret return path. Shared manifests, Bazel/lock files,
navigation, plan mutation, archival, and transport activation remain untouched.

## Integration completion

The integration owner merged the clean lane, re-exported
`CREDENTIAL_USAGE_SCHEMA_VERSION` and `CREDENTIAL_USAGE_MAX_REQUESTS`, and
validated the combined tree based on merge commit
`c30ee50dceb4e4a34df87b9114fcec1ccb866f92` plus the integration-only
registration/archive diff contained by this evidence record's commit.

- `cargo test -p codex-security-policy`: 47/47 passed.
- `cargo test -p codex-core security::`: 51/51 passed, including all 18 focused
  credential-capability cases.
- `cargo test -p codex-config`: 229/229 passed; `cargo test -p codex-core
  config::`: 487/487 passed.
- No transport, vault-resolution path, broker consumer, protected profile or
  TUI control was activated. PF-13-S03/PF-27 retain the authenticated-consumer
  obligations above, and PF-13-S07 retains final composed qualification.

Cargo homes, targets and temporary output for these reruns remained under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/round3-integration/`.
