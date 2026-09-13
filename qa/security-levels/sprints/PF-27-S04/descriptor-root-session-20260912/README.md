# PF27 private descriptor root-session pump

Work in progress, not qualified or ready for native/product use.
Manager allocation06bf29c29 and fixed-pair clarification44b39817a imported as
d46565dcf and4e1b57b69 respectively. Source/size base2c63e4cd5; seven exact paths,
target700/hard800 including this receipt, tests, runner and frozen allocation.
Reviews37 Astra High/38 Fable5.1 High allocated, not yet dispatched.

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

| New explicit actual-OS case | Intended observable evidence |
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
unless exercised by their separate filtered commands. Exact final source, raw
attempts, formatting, strict scopes, lock parity and review receipts remain pending.
Remote originals: `/home/travis/security-round5/evidence/pf27-root-session-20260912`.
Preliminary actual-key RTX TMUX run: check0 (14.84s), fix0 (16.06s), fmt0,
all seven session cases passed (0.794s); original logs/exits under `preliminary/`.
Only the four allocated Rust files changed after formatting; copied back exactly.

Manager accepted policy1.7 internal-only N/A for this private increment. Existing
main exit78, fixed-system factories, public root.rs/Child, Core, Vault, credentials
and native installation are unchanged. Later protected-user/PF26 isolated
functional execution, native/live-repository proof and human acceptance remain
mandatory; this receipt does not complete PF27 or qualify a release.
