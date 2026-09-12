# PF27 descriptor identity compatibility — final test evidence

Tests pass; independent reviews pending. Allocation imported before code at launch
`8eeabe2cc178d41f0fec9ce65c233b062bb4a08f`; relay amendment imported as
`218124bc1`. Source baseline remains `6b393f134f83210be5a55829b63dc513f975fe57`.
Eleven allocated paths, hard800 including this receipt, fixture and runner.

New sealed child identity and additive PF20 entry remain Linux/GNU,
synthetic-feature-only, default OFF. Sole termination/reaping ownership is
unchanged. Both entries use the existing framed protocol and ambiguity handling.
No root.rs composition, system enrollment, credentials or protected activation.
Internal-only policy1.7 N/A is manager accepted; later protected-user/PF26
independent isolated-functional/native qualification remains mandatory.

Preliminary RTX evidence is preserved at
`/home/travis/security-round5/evidence/pf27-root-compat-20260912`.
Initial compilation failed on test-only WaitIdStatus equality; repaired check
and transport compilation pass. Initial full formatting lacked uv in PATH;
Rust formatting succeeded, PATH corrected, subsequent full fmt and fix pass.
Relay static hash/profile inspection and two actual-kernel identity tests pass.
First four transport cases failed before launch because /tmp is tmpfs, correctly
unsupported by PF20; they are rerun separately on a recorded ext-family TMPDIR.
Original attempts are not overwritten. Corrected on-disk run passes all four.

## Frozen source and actual final run

Source `d776e938d8a0c07cc9b50b1d6a818d9f6feb0c11`, Rust
`b48e9c4f9cb89e814b8bdabd6069be86cc47badd`. Later evidence/bookkeeping does not
change runtime. Private RTX TMUX `pf27rootcompat20260912:proof`, text then Enter
separately, began 22:15:35 UTC. All **25 command exits plus suite.exit are 0**.
Both `PF27_ROOT_COMPAT_TMUX_COMPLETE` and `PF27_ROOT_COMPAT_D776E938D_COMPLETE`
are in [actual capture](rtx/final-tmux-capture.txt). No test retries.

| Check | Actual result |
| --- | --- |
| Adapter default / OS | 4 / 10 passed; OS includes both retained-identity cases |
| Service default | 3 passed |
| Pair / actual pair | 5 / 1 passed |
| Owner / actual owner | 8 / 3 passed |
| Admission | 8 passed, including ten actual-peer lifecycle scenarios |
| Service full synthetic | 59 passed; 11 exclusions separately covered |
| PF20 default / synthetic | 18 / 18 passed |
| New compatibility | 4 passed; both entries and five rejection scenarios |
| ELF profiles | hold, inspect, connector and relay: one passed each |
| Strict Clippy | service+adapter 14.69s; protected-state+adapter 6.27s, both exit0 |
| Bazel parity / locks / source | all exit0, unchanged before/after |

Counts overlap, not a distinct-test total. Service's 11 exclusions are six
admission OS cases, three actual-owner, one actual-pair and one profile case,
all explicitly run. PF20's two default exclusions are subprocess helpers called
by passing parent tests; feature adds four ignored compatibility tests, all run
explicitly. GNU2.39 rejection is retained predecessor history, not rerun here.

New cases: retained identity survives handle drops without reaping and rejects
dead/reaped/replacement identity; exhausted-fd clone failure retains its owner.
Both transport entries pass load/CAS/reload, post-handshake child death,
oversized frame and authenticated old-sequence rejection. The relay acknowledges
reply withholding, the parent observes actual committed state, then kills relay:
client reports Ambiguous, consumes capability and never retries. Descriptor
rejections cover wrong live child, inherited parent socket, dead/stale owner and
actual unsupported TCP peer query, with zero handshake bytes received.

Environment/provenance is in [final-provenance.txt](rtx/final-provenance.txt):
nonroot UID1001, Linux7.0.0-31 x86_64, glibc2.43, Rust1.95.0; ext-family TMPDIR.
Shared build.lock serializes jobs4 on the exclusive PF27 receiving target.
No Mac build, unrelated workload termination, source push or native installation.

Cargo.lock only adds protected-state's optional direct adapter edge, SHA256
`5101ffe0dea88d84e213fcd31025f1d2ca0f9a8cbd069b8ac5ea787d392b1459`.
`just bazel-lock-update` exits0 with no MODULE.bazel.lock delta; its SHA256 stays
`c8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979`.
Existing platforms/rules_cc resolution warnings remain; no dependency upgrade.
Module/source invariance and clean checkout were checked after the full suite.

Relay source SHA256 `49d3888ebe9780e3321e996d73a7dd6ec01082f576ebc07d613cdcd8c321c28b`,
static PIE artifact `ac3f6dd19d64c504e5be9bd0cb0cd190ef62325c34a252ab49f597b23eaa8e5c`.
The committed runner hashes and validates its ELF profile before any invocation.
Raw logs/exits/provenance/compiler/ELF evidence, including failures, are preserved
under `rtx/`; generated binaries and transfer bundles are not committed.

## Review and acceptance boundary

Manager authorized scoped Astra High33 and Fable5.1High34 via Corbanu/private
TMUX, against baseline6b393f134. Prior1–32 remain unchanged. Final review outcomes
are pending, so no manager-ready or human-test-ready claim yet. This is transport
compatibility only: root.rs dispatch, separate-principal native containment,
credential data plane, PF26/live-repository and independent functional proof,
human acceptance, benchmarks and whole-sprint/release completion remain open.
