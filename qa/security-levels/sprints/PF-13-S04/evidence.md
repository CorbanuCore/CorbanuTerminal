# PF-13-S04 final-tree evidence

- Date: 2026-08-26 UTC
- Baseline commit: `4c62364fe47d4d49bf11023ae6d120533e5f7524`
- Implementation commit: `f1a8c5c75dbf2926ab7937d64711b2471553a884`
- Tested code tree: `f1a8c5c75dbf2926ab7937d64711b2471553a884`
- Product requirement: `docs/corbanu-product-spec.md`, **Required trust boundaries** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- TUI applicability: none; this sprint changes no interactive surface.

## Result

Credential capabilities are now consumed atomically under the lifecycle store's
write lock before Core returns authority for resolution. A successful use,
concurrent duplicate, sequential replay, or restarted runtime therefore cannot
reuse the capability. Forged or mismatched requests do not consume the valid
entry. Current revocation state remains read-locked across Vault resolution and
the trusted transport callback, so a revocation linearizes either before a new
resolution and denies it, or after an already-started one-shot resolution.

Every resolver outcome constructs and emits a structured, secret-free
`ActionReceipt` with the capability id, policy reason, operation, destination,
and outcome. The receipt schema and log emission contain no credential label or
credential value. Existing non-credential receipts retain their serialized
shape and receipt-id binding when the optional metadata is absent.

The supported raw `vault auth-helper` path remains unchanged under Permissive.
Moderate and Aggressive reject it before label normalization, storage access, or
stdout output. The CLI evaluates the maximum of persisted and requested
security posture, so `-c security.level="permissive"` cannot downgrade a
persisted protected posture. Protected execution is directed to the brokered
label path instead.

## Authority lifecycle matrix

| Case | Result | Automated evidence |
| --- | --- | --- |
| First exact use | Authority removed atomically and returned once | `issued_capability_is_consumed_only_for_the_complete_bound_request` |
| Sequential replay | Unknown capability; no second resolution | same test and scoped proxy one-shot tests |
| Eight concurrent duplicates | Exactly one success; store empty afterward | `concurrent_duplicate_consumption_allows_exactly_one_use` |
| Forged bearer or adjacent bound request | Denied without consuming the valid entry | `forged_bearer_and_public_id_alone_cannot_authorize`; adjacent-dimension test |
| Runtime restart | Old in-memory authority is absent | `capability_authority_does_not_survive_runtime_restart` |
| Revoked before resolve | Denied without Vault access | `credential_authority_revocation_before_resolve_denies_without_vault_access` |
| Revoked during use | Active resolution completes; revocation blocks the next use | `credential_authority_revoke_during_use_linearizes_after_the_active_resolution` |
| Callback failure, cancellation, or panic | Stable secret-free failure/cancellation; source allocation is released | Vault capability regression suite |

## Raw-secret bypass matrix

| Surface | Result and evidence |
| --- | --- |
| CLI stdout / command substitution | Moderate and Aggressive fail with empty stdout; CLI tests also prove the error omits the requested label. |
| CLI security override | Persisted Moderate plus requested Permissive still fails at the protected-level gate. |
| Permissive helper | Reaches the unchanged legacy lookup path; the protected-level message is absent. |
| Vault storage | The protected-level gate runs before label normalization and storage access; a stored canary remains unread. |
| Child environment | Existing scoped broker tests prove supported raw environment values are replaced with fresh opaque dummy capabilities and cannot be reused by another child. |
| Tool/proxy output | Resolution exposes the secret only to the synchronous trusted injection callback; errors return fixed enums and the resolver does not return the value. |
| Logs and audit | Structured receipt tracing includes only receipt id, capability id, reason, operation, destination, and outcome. Integration tests scan captured logs for absence of label and secret canaries. |
| Serialization | Exact receipt JSON tests reject label/value fields and bind metadata into the receipt digest. |

Representative receipt shape (synthetic identifiers only):

```json
{
  "schema_version": 1,
  "receipt_id": "<receipt-sha256>",
  "mandate_id": "<capability-sha256>",
  "preview_digest": "<authority-sha256>",
  "outcome": "executed",
  "completed_at_unix_seconds": 120,
  "credential_use": {
    "capability_id": "<capability-sha256>",
    "policy_reason": "matching_grant",
    "operation": "responses.create",
    "destination": "https://api.openai.com:443"
  }
}
```

## Final-tree commands

```text
cd codex-rs && just fix -p codex-security-policy && just fix -p codex-vault && just fix -p codex-network-proxy && just fix -p codex-core && just fix -p codex-cli && just fmt
PASS — affected crates fixed and formatted; only seven pre-existing Core test dead-code warnings were emitted.

git diff --check
PASS — no whitespace errors; final implementation diff inspected.

cd codex-rs && just test -p codex-security-policy
PASS — 21 tests passed.

cd codex-rs && just test -p codex-vault capability
PASS — 5 focused tests passed; 24 unrelated tests skipped.

cd codex-rs && just test -p codex-vault
PASS — 29 tests passed. `corrupt_encrypted_state_fails_closed` passed on nextest retry and was reported flaky.

cd codex-rs && just test -p codex-network-proxy credential_broker
PASS — 14 focused tests passed; 194 unrelated tests skipped.

cd codex-rs && just test -p codex-network-proxy
PASS — 208 tests passed.

cd codex-rs && just test -p codex-cli vault
PASS — 13 tests passed; 1366 unrelated tests skipped.

cd codex-rs && just test -p codex-core credential_capability
PASS — 11 targeted tests passed; 3382 unrelated tests skipped.

cd codex-rs && just test -p codex-core credential_authority
PASS — 3 targeted integration tests passed; 3390 unrelated tests skipped.

python3 docs/plans/check.py && python3 docs/sprints/check.py
PASS — plans active 1/2; sprints current 19 and archived 83 before archival.
```

No Cargo manifests or dependencies changed, so Cargo/Bazel lock regeneration was
not applicable. The repository policy requires approval for the full Core
suite; this sprint used the targeted Core commands above.

## Changed paths

- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/tests/vault.rs`
- `codex-rs/core/src/config/network_proxy_credential.rs`
- `codex-rs/core/src/config/network_proxy_credential_tests.rs`
- `codex-rs/core/src/security/credential_capability.rs`
- `codex-rs/core/src/security/credential_capability_tests.rs`
- `codex-rs/security-policy/src/lib.rs`
- `codex-rs/security-policy/src/mandate.rs`
- `codex-rs/security-policy/src/security_policy_tests.rs`
- `codex-rs/vault/src/capability.rs`
- `codex-rs/vault/src/lib.rs`
- `codex-rs/vault/src/tests.rs`
- `docs/plans/active/p0-security-levels.md`
- `docs/sprints/current/p0-security-levels/index.md`
- `docs/sprints/current/p0-security-levels/pf-13-s04-authority-lifecycle-and-raw-secret-bypass.md`
- `docs/sprints/index.md`
