# RETURN — owner-isolated-112 (corrected by owner-limited-113b)

The round-112 operational definition is superseded by
[the round-113b return](owner-limited-113b-return.md). Its former sections 1–4
and corrections section were removed, including every requirement that the
pinned same-UID transport deny its worker access to the controller runtime.
They must not be used as provisioning instructions.

The corrected minimum is a disposable guest-machine boundary, external mediated
inference with no readable long-lived provider credential in the guest, and
explicitly accepted same-UID trust between controller and worker. The guest
worker can read staged Python helpers and the auth-link target and can reach
same-UID process/IPC surfaces; the target must contain only a synthetic marker,
not an upstream secret. None of those accesses is claimed as denied.
Separate actor UID/namespace, a transport-native broker-only credential interface
with no readable auth-file contract, and enforced worker-level child denial
require a transport change, new allocation, review and re-pin. They are not
provisioning work for this pinned lane. The corrected return contains the full
ordered gate predicates, executable qualification recipe and audit-only procedure.

## Preserved historical provenance (not a new run)

Round-112 allocation digest:
`0a16457e983e02c4b76fb36be728abb8c0e3ab0002a3796efdb3fb7dbd535b2b`.
Claim: `fe1b4144-4152-446d-9be3-f494b26a73a3`.
Runtime: `gpt-6-astra`, effort `high`.
Original base: `7d22801b942eb8b487e6c7cbc3ff7c44e0c297f1`.
Original brief SHA-256:
`85875adaf6e42405824a2f7cdbeefc02bc5854cb78a43ed0dd30d06480d2fe99`.

The earlier raw test artifacts are preserved without alteration:

- [Full discovery](owner-isolated-112-suite.txt): 880 passed, 505.623s, exit 0.
- [Focused modules](owner-isolated-112-focused.txt): 483 passed, 386.401s, exit 0.
- [Venv setup](owner-isolated-112-venv.txt) and
  [disposable root](owner-isolated-112-test-root.txt).
- Earlier failures/errors/skips: zero; failure names: none.

These historical tests did not qualify the VM, real inference or live promotion.
Current verification and file-change accounting belong to the round-113b return.
