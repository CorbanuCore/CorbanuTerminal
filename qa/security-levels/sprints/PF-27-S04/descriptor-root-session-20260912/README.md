# PF27 private descriptor root-session pump

Private increment: exact RTX proof passed; independent reviews pending.
Manager allocation06bf29c29 and fixed-pair clarification44b39817a imported as
d46565dcf and4e1b57b69 respectively. Source/size base2c63e4cd5; seven exact paths,
target700/hard800 including this receipt, tests, runner and frozen allocation.
Reviews37 Astra High/38 Fable5.1 High allocated; reservation recorded in ledger.

## Contract and fixture boundary

The private pump consumes exactly two nonblocking listeners, accepts at most four
connections per poll round-robin, polls at most two pending tickets and retains
at most two tickets and the existing two root jobs globally. Listener order gives
no authority: only the original live matched child receipt determines namespace
and generation. No new worker, arbitrary listener collection or public bootstrap.
Fatal listener/admission failure, completion, cancellation, deadline, death and
drop fence the pair. Completion waits for actual handler joins and child cleanup;
no wall-time promise is made for stuck filesystem calls.

Tests use the unchanged hashed static relay with distinct journal/policy abstract
listeners. Each child connects server, fixture-client, then fixture-control.
Setup observes readiness without accepting. The pump's first poll itself accepts
both actual server sockets, with policy listener first to check role routing.
Only between polls do fixture helpers accept client/control sockets. The duplicate
case deliberately leaves the control sockets for the pump to reject. No helper
pre-admits or injects a server socket or substitutes mock OS identity.

| New explicit actual-OS case (all passed) | Observable evidence |
| --- | --- |
| routing_and_storage | Reversed listener order; both load/CAS/reload, exact checkpoints |
| flood_fairness | Asymmetric wrong-peer flood; second listener serviced, global cap, no handshake |
| accept_faults_are_bounded | Four Interrupted/WouldBlock attempts; fatal error closes both and reaps |
| lifecycle | Actual death, cancel, deadline and drop close both clients and reap |
| pending_cancellation | Cancel/drop before receipt dispatch; no handshake or stored state |
| duplicate_receipts | Same-child duplicate connections cannot add jobs or retain authority |
| protocol_errors | Server Invalid remains typed; committed lost policy reply remains Ambiguous |

The committed runner retains all28 predecessor commands and explicitly executes
the seven new ignored OS cases. Full service exclusions are not passing evidence
unless exercised by their separate filtered commands.
Remote originals: `/home/travis/security-round5/evidence/pf27-root-session-20260912`.
Preliminary actual-key RTX TMUX run: check0 (14.84s), fix0 (16.06s), fmt0,
all seven session cases passed (0.794s); original logs/exits under `preliminary/`.
Only the four allocated Rust files changed after formatting; copied back exactly.

## Frozen final proof

Source74263bbbc5bf2163f571947c6828fbf108866c3f, Rust
b7275b0c0433f081d6423463993abc658141a6a7. Clean RTX checkout
`/home/travis/worktrees/security-broker-root-session-20260912`, private TMUX
`pf27rootsession20260912:proof:0.0`, September13 00:46:17–00:48:02UTC (105s).
Command text and Enter sent separately. All29 command exits plus suite0 from
this same run; both session and wrapper completion markers in captured pane.
Original wrapper, logs/exits, module/source/fixture provenance and capture in `rtx/`.

| Final check | Actual result |
| --- | --- |
| Adapter default / actual supported OS | 4 / 10 pass |
| Service default / pair / actual pair | 3 / 5 / 1 pass |
| Owner / actual owner / admission | 8 / 3 / 8 pass |
| Full service feature suite | 59 pass; 24 ignored cases separately exercised |
| PF20 default / feature / actual compatibility | 18 / 18 / 4 pass |
| Actual root dispatch / new session | 6 / 7 pass |
| Hold / inspect / connector / both relay profiles | Five sealed artifact profiles pass |
| Strict service+adapter / PF20+adapter Clippy | Exit0, 6.48s / 14.28s |
| Read-only Bazel parity / source and module invariance | Exit0; unchanged |

Filtered suites overlap, not distinct totals. Service exclusions are previous17
plus new7; PF20's helper-only exclusions unchanged. Actual error/interruption
seams are explicit; all happy paths use real OS admission and PF20 transport.
Non-root uid1001, GNU2.43, Rust1.95.0, ext-family TMPDIR, build lock and jobs4.
Cargo.lock SHA5101ffe0dea88d84e213fcd31025f1d2ca0f9a8cbd069b8ac5ea787d392b1459;
MODULE.bazel.lock SHAc8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979.
Both before/after source trees, module files and five input artifacts match.
No failed test/correction in this increment; older failures remain in predecessor
receipts. Existing Bazel version warnings retained. Reviews not yet completed.

Manager accepted policy1.7 internal-only N/A for this private increment. Existing
main exit78, fixed-system factories, public root.rs/Child, Core, Vault, credentials
and native installation are unchanged. Later protected-user/PF26 isolated
functional execution, native/live-repository proof and human acceptance remain
mandatory; this receipt does not complete PF27 or qualify a release.
