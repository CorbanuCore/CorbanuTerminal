# PF27 private fixed-system preflight

Final exact-source RTX proof passes; allocated reviews pending. Not native-ready.
Manager allocationab985f5f8 imported clean asb65bf1e932f8fb6d9b24d945fe67b18a57bb0ac2.
Source/size/review baseae2a4c9407983d1877fce18f66d054d7cf90aa7e; seven exact
paths target550/hard700 including allocation, tests, runner and this receipt.
Reviews39 Astra High/40 Fable5.1 High reserved in the preserved review ledger.

## Ownership and compatibility

An uncalled private factory acquires the existing reservation before preparing
anything. It obtains two inspections through the existing root-only fixed-system
factory and compares executable/source path, complete source stamp, digest,
all three role identity pairs and anchor group before consuming either inspection.
Existing seals/profile checks then prepare two images; existing PF20 journal and
policy factories open only pre-existing roots. No enrollment fallback exists.
The private result retains images, roots and reservation; declaration order drops
images and roots before releasing the reservation. Every failure drops its owned
intermediates. No Clone/FD export or transfer/launch API is introduced.

The comparison establishes consistency of retained validated observations, not
filesystem-wide atomicity. Existing image mutation checks remain. No child,
listener, handler, real credential, identity change, root write, readiness flag
or native activation is introduced. Same-UID/non-root launcher guards, public
root.rs/Child, fixed factories and main exit78 are unchanged.

## Cases and evidence boundary

Five new cases exercise actual fixture filesystem inspection, kernel sealing,
ELF profile checks, descriptor cleanup and reservation lifecycle. Fresh synthetic
roots are prepared by the existing PF20 fixture before the preflight. The test
operations are confined to cfg(test); production uses only fixed factories.

| Case | Required observation |
| --- | --- |
| Complete owner and busy reservation | Two retained images/root references; no preparation under contention; drop closes/releases |
| Eight preparation-stage failures | Each return releases descriptors, root references and permit; fresh roots remain empty |
| Same bytes/different recipe or source | UID, GID, anchor, source path and inode changes deny before either image is sealed |
| Invalid manifest/hash/profile | Real parser, digest and sealed-profile rejection without lingering descriptors/permit |
| Non-root fixed-system entry | Actual identity guard denies; no root access or child left behind |

The eight-stage injection includes both existing-root open failures. These are
explicit faults, not root-positive access or a new PF20 enrollment test. Actual
missing/corrupt-state denial and no-enrollment behavior remain covered by the
retained PF20 suite; this unit does not widen its root/path API to recreate it.
All29 predecessor commands plus the explicit new filter are required in the final
exact-tree RTX/TMUX proof. Preliminary RTX check0 (14.78s), fix0 (16.35s), fmt0;
all five new cases pass (0.019s), with original logs/exits retained remotely.
Only the four allocated Rust files changed after formatting and were copied back.
Final proof ran once September13 01:17:15–01:19:00UTC (105s), non-root uid1001,
GNU2.43/Rust1.95.0, exclusive existing RTX target/build lock, ext-family TMPDIR.
Frozen source19f6624938c53e0c9709e1b25ada4a44fd13267e,
Rust09f72396304d80a82cf466e220378f763ea8313b; no subsequent Rust changes.

| Final proof | Actual result |
| --- | --- |
| All30 command exits plus suite | All0; original final-tmux/*.exit and suite.exit |
| New preflight filter | 5 passed in0.018s; no retry |
| Full service / default | 64 passed / 3 passed; service24 exclusions covered by explicit retained filters |
| Retained ownership/admission | Pair5/real1; owner8/real3; admission8; adapter4/OS10 |
| Protected storage and session | Default18/feature18; compatibility4; dispatch6/session7 |
| Five explicit profile checks | Hold, inspect, connector, root relay and dispatch relay each pass |
| Strict scoped Clippy / compatibility Clippy | Exit0, 6.32s /14.38s |
| Cargo/Bazel/source provenance | Parity0; source, four module inputs and five fixture hashes unchanged |

Actual keys launched the frozen wrapper in RTX private TMUX
pf27preflight20260913:proof:0.0, text and Enter separately. Both
PF27_SYSTEM_PREFLIGHT_TMUX_COMPLETE and PF27_SYSTEM_PREFLIGHT_19F662493_COMPLETE
appear in [the actual pane capture](rtx/final-tmux-capture.txt).
Original commands, full logs, preliminary attempts and before/after provenance
are under [rtx/](rtx/). Remote originals remain at
/home/travis/security-round5/evidence/pf27-system-preflight-20260913;
checkout /home/travis/worktrees/security-broker-system-preflight-20260913.
No new-case failure or repair was omitted; predecessor failures stay preserved.
Focused filter counts overlap the full suite, not additional unique tests.

Policy1.7 internal-only N/A accepted for this uncalled preflight only. Later
distinct-principal native bootstrap, independently isolated protected-user/PF26
functional testing, native denial/platform evidence and human acceptance remain
required. No installation, real-root test, whole-PF27 or release claim.
