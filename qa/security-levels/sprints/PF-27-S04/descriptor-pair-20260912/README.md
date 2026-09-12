# PF27 Stage A: descriptor identity and private pair lifecycle

Status: final-source tests, scoped lint/parity and both allocated independent
reviews pass. Ready for manager receiving integration, not service activation
or product readiness.

## Frozen scope for review

Existing product initiative: `docs/plans/active/p0-security-levels.md`, PF-27;
sprint PF-27-S04 remains `in_progress`. Product **Non-negotiable controls**:
“Permit agents to reference credentials only by label; resolve them solely
inside the trusted execution boundary.” This internal prerequisite does not
itself establish credential isolation for a running product.

Owner `/root`; branch `feat/security-broker-resume-20260911`; worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`;
original allocation base `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb`.
Review ONLY this incremental diff against accepted source
`62956038e77d41dd4ee1a3130b8b8ab77558a83f`, not the entire original branch.
Frozen implementation `e0eb9eac4f48dbf1e2760f8265826825a9cb5519`, Rust tree
`f64820d0ca0312efbb612f208fb20636b6f547cf`. Later evidence-only commits do not
change that Rust tree. No unqualified source push is requested for review.

The manager's [exact Stage A allocation](../descriptor-pair-a-allocation-20260912.md)
supersedes the earlier combined proposal. All nine old source/test files are
preserved in local/RTX WIP archives, with hashes in the allocation. Stage B
admission code was removed from this landing tree, not declared complete.

Ten code/test/fixture paths change **614 lines**, additions plus deletions
against 62956038e, within hard800. Runtime/registration changes are177;
tests400; committed runner37. No dependency, manifest or lockfile changes.

| Paths | Changed lines |
| --- | ---: |
| `linux-pidfd-spawn/src/lib.rs`, `peer.rs`, `spawn.rs` | 36 |
| `linux-pidfd-spawn/src/peer_tests.rs` | 78 |
| `secret-broker-service/src/launch/manifest/mod.rs`, `pair.rs`, `spawn.rs` | 141 |
| `secret-broker-service/src/launch/manifest/pair_tests.rs`, `spawn_tests.rs` | 322 |
| `qualify-rtx.sh` in this directory | 37 |

Rust paths above are under `codex-rs/`. No `children.rs`, `root.rs`, PF20,
Core, Vault, login, HTTP client, native activation or privileged setup changes.
No new admission API, generation/UID parameter, socket or connector fixture.

## Contract and invariants

`PairImages` and `Reservation::launch_pair(images, deadline)` reuse the existing
reservation, `LaunchHandle`, cancellation/status/Drop and precreated worker.
One permit is acquired before both images. Worker-creation failure returns both
images. First-spawn failure does not attempt the second. Second-spawn failure
returns ownership of the first child to the supervisor and requests cancellation;
it does not falsely release capacity. Cancellation observed between spawns
prevents the second. A spawn already started may finish late; ownership remains
with the worker through cancellation, caller Drop and deadline.

Pair polling and stop attempt both children even after one error. Either child
exit requests whole-pair cancellation. Both must be positively reaped before
capacity is reusable. Persistent signal/wait failures retain ownership and retry;
panic quarantines capacity without a reset API. Pair Drop signals both before
individual potentially blocking owned-child Drops. Process-crash durable
supervision remains outside this increment.

The adapter compares retained pidfs device/inode identities only on64-bit
targets and rejects non-pidfs descriptors. Cached reaped ownership returns false.
This is identity, NOT liveness, readiness or socket authorization. No numeric
PID lookup authorizes a peer. The real test reads `/proc/.../children` solely
to confirm two actual test children existed, never for access decisions.

## Exact-source qualification

RTX clean detached source e0eb9eac4, worktree
`/home/travis/worktrees/security-broker-pair-20260912`, exclusive target
`/home/travis/security-round5/targets/pf27-pair-20260912`, non-root UID1001,
Linux7.0.0-31-generic x86_64/GNU libc2.43/Rust1.95.0. Builds use four jobs
under the existing build lease. No Mac build or shared target modification.

`just fix` scoped to both changed crates with synthetic-fixture, then `just fmt`,
then strict `cargo clippy --locked --no-deps -p codex-secret-broker-service
-p codex-linux-pidfd-spawn --features synthetic-fixture --tests -- -D warnings`
passed before the final suite. Scope is these crates, not whole-workspace lint.
Logs are in [rtx/pf27-pair-20260912](rtx/pf27-pair-20260912/).

The committed [runner](qualify-rtx.sh) was invoked in private TMUX socket
`pf27paira20260912`, session `lifecycle`,150x45, by separate literal-text and
Enter sends. [Capture](rtx/tmux-capture.txt) records source/host and the positive
`PF27_PAIR_A_TMUX_COMPLETE` marker. All ten `.exit` files are0.

| Final check | Passed | Skipped by filter/default |
| --- | ---: | ---: |
| Adapter normal | 4 | 9 |
| Adapter supported-OS ignored tests | 8 | 5 |
| Service default features | 3 | 0 |
| Pair deterministic failures/races | 5 | 57 |
| Real two-hold pair cancel/Drop/reaping | 1 | 61 |
| Existing owner deterministic | 8 | 54 |
| Existing owner real processes | 3 | 59 |
| Profile hold / inspect | 1 each | 61 each |
| Full synthetic service | 57 | 5 |

Do not sum overlapping focused/full-suite counts. Five service ignored tests
are real pair1, real owner3, profile1; each was executed separately (profile with
two artifacts). Adapter ignored run excludes the unqualified-libc test, which
requires the separate unsupported GNU2.39 host; its earlier rejection proof is
in descriptor-launch-20260912, not a fresh Stage A pass or positive support claim.
Four other adapter skips there are the normal tests executed separately.

Required pair cases are covered by five named tests: reservation/prelaunch and
worker failure; partial-spawn failures/inter-spawn cancel; blocked second-spawn
cancel/Drop/deadline; either death/asymmetric signal/wait failures; panic
quarantine. The ignored real pair test observes two live hold children, cancels
or drops the caller, verifies bounded caller return and confirms both reaped
before reuse. Earlier `partial-pair` logs are historical combined-WIP diagnostics,
not qualification of removed admission code.

Read-only `bazel --output_base=<exclusive-pair-bazel> mod deps
--lockfile_mode=error` passes. Cargo.lock SHA256
`be1c1136f5c80bbf033290586040d2719240a8e11825f23959f56a44a8c5165d` and
MODULE.bazel.lock SHA256
`c8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979`
match before/after. Source status remains empty. Existing platforms/rules_cc
resolved-version warnings remain in the original log; no Bazel Rust build claim.

Static hold SHA256 `59ba145dc8178ae9680914943e8ba12c23c37d17ac4cc4a05a423539cbda1a6f`;
inspect `cc821a2ee024b3f17a9073de108a6212e5f2c34f77818715b775c59f880477ca`;
probe `f6ca8e3368dcbc8e5ba92bbbc7860ee825628b9470ac53116c6c3f6996bef3f0`.
All are existing qualified synthetic fixtures; no real secrets or accounts.

## Policy applicability and deferred gates

Accepted policy1.7: manager receiving423bdb29674bad54594dfbb19ffe845f0e2d5786,
local main8cfcff990a9bdbcf018d0f93a547cc5115af88b8, adoption receipt
`qa/code-blind-functional/adoption-20260912.md` on those refs. Independent
code-blind functional execution is N/A ONLY for this private internal increment:
no exposed product interaction exists to exercise. This is not a sprint/plan
exemption. Later affected protected-user flows and PF26 must meet schema2
independent isolated execution and a separate evidence reviewer. Manager owns
portable runner/provisioning qualification. No claim of native/user acceptance,
live TensorCash/Isometric proof, benchmark completion or release qualification.

Stage B remains unallocated until A is accepted/integrated: socket admission,
bounded queues, role/UID/generation matching, real connector, duplicate/parent/
unrelated/old-generation denial, reverse arrival, queue-full, pending/accepted
reply loss, peer death, late delivery and blocked-read wakeup. Nothing is waived.
Native service containment, trusted credential migration, all-OS qualification,
protected activation and PF26 final-product checks also remain open.

Reviews28 AstraHigh and29 Fable5.1High via Corbanu/private TMUX use this frozen
scope and the existing ledger. Prior review failures/usage are retained; this
two-pass manager extension is not a reset. No new review of the old accepted
single-child stage or removed Stage B code is requested.

Both helpers completed exit0 with `findings: []`, patch correct; original
[Astra28](astra28.json) and [Fable29](fable29.json), text and exit receipts are
preserved here. Reserved20:18UTC, dispatched20:18:32–35UTC on September12.
They inspected code and supplied evidence; no tests were run by the reviewers.
No findings accepted/rejected, no review-triggered changes or extra pass.
Final Rust tree remainsf64820d0; evidence/ledger closeout is documentation-only.
