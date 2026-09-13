# PF-27-S04 — accepted private root-session allocation

September13 manager authorization within active PF-27, **Non-negotiable controls**:
“Permit agents to reference credentials only by label; resolve them solely inside
the trusted execution boundary.” The original proposal below is accepted with
these bounds; its proposal-only wording is retained preparation history.

Manager inspected exact combined eb5855959 proof: all28 command exits+suite0,
137s00:13:35–00:15:52UTC, actual RTX/TMUX markers, source/locks/input invariance,
all result summaries and literal receipt. Accept internal root-dispatch increment
only. Mac combined100/575/Clippy/check also passed; no review repetition needed.
Sole existing /root owner/worktree/branch and original plan base remain unchanged;
next unit source base2c63e4cd535b89c5e5bee02f435c1a35760e855c. Import this
allocation-only checkpoint first and record clean actual launch HEAD.

Exactly seven paths below; allocation file is manager-owned/frozen. Target700,
hard800 added+deleted, tests/runner/receipt/allocation counted. Fixed accept budget
at most4 per poll, ticket budget at most2, at most2 pending tickets and2 existing
root jobs; excess peers must not starve lifecycle polling or mint authority.
No blocking receive, detached cleanup or wall-time claim for stuck storage.
Manager allocates new Astra High37 and Fable5.1 High38 via existing Corbanu TMUX,
preserving1–36; necessary substantive correction returns to integrator for scope.
No Core/locks/manifest or native/public bootstrap changes. Existing owner retains
exclusive RTX target/lock; accounting owns separate Mac target. Full28 predecessor
proof plus new explicit actual-OS cases, strict scopes/parity/format-before-test.
Internal-only policy1.7 N/A accepted for this private gated pump, not whole PF27,
protected-user/PF26/native readiness, main or release. Return frozen candidate and
literal evidence for exact receiving proof before another successor.

## Preserved inspected design

### Manager clarification — fixed pair of owned listeners

Owner's actual frozen relay has distinct journal/policy rendezvous listeners.
Accept consuming a fixed [UnixListener; 2], not a configurable collection. The
accept<=4 and ticket<=2 budgets are aggregate per poll, round-robin over the
pair; pending<=2/jobs<=2 remain global. Listener index conveys no role/generation
authority. RootSession itself must accept every real server socket; fixture setup
may consume only separate fixture client/control connections between polls, not
pre-admit or inject server sockets to bypass the pump. Record this fixture phase
explicitly. Prove both-listener fairness/flood bounds and receipt-based namespace
routing. Same seven paths/hard800, no relay/protocol/public constructor widening.
This supersedes singular-listener wording in the original proposal below.

# PF27 successor proposal — bounded private root-session pump

Proposal only, September 12. No implementation or additional review dispatched.
Prerequisite: manager acceptance of owner `2c63e4cd535b89c5e5bee02f435c1a35760e855c`
and exact combined receiving proof. Source baseline would be that accepted owner,
not an earlier compatibility stage. Existing owner/worktree/branch stays unchanged.

## Smallest missing connection

The accepted dispatch has real PF20 protocol and owned-child lifetime proof, but
its caller still manually collects sockets, submits/polls admission tickets and
dispatches the receipts. `main.rs` correctly remains exit78. Add one private
bounded event pump to perform that connection from a trusted owned UnixListener
to the existing admission queue and RootDispatch. This is not a public service,
listener installation, system-path constructor, credential API or readiness flag.
The listener locates peers; the existing kernel identity checks supply authority.

## Proposed exact scope (seven paths, target700 / hard800)

1. `codex-rs/secret-broker-service/src/launch/manifest/root_dispatch.rs` — nested private session module only.
2. `codex-rs/secret-broker-service/src/launch/manifest/root_session.rs` — new bounded pump.
3. `codex-rs/secret-broker-service/src/launch/manifest/root_session_tests.rs` — new actual-OS and fault cases.
4. `codex-rs/secret-broker-service/src/launch/manifest/root_dispatch_tests.rs` — narrowly reuse existing cfg(test) pair/relay helpers, no production constructor widening.
5. `qa/security-levels/sprints/PF-27-S04/descriptor-root-session-20260912/qualify-rtx.sh` — preserve all28 predecessor commands and add explicit new cases.
6. `qa/security-levels/sprints/PF-27-S04/descriptor-root-session-20260912/README.md` — final receipt and case mapping.
7. `qa/security-levels/sprints/PF-27-S04/descriptor-root-session-allocation-20260912.md` — manager's frozen contract/allocation, no runtime logic.

Count authored tests, receipt and runner in the ceiling; original raw proof and
coordinated plan/sprint/review-ledger bookkeeping remain separately recorded.
No manifests, PF20/adapter changes, new relay, root.rs/children.rs, Core, Vault,
proxy, executable entry point, CLI/config/schema, global listener path or install.
If helper reuse cannot fit without broader source changes, stop and reallocate.

## Proposed contract and proof

- Consume the trusted listener; require nonblocking operation, retain ownership
  through the session, and never let the caller supply a peer role or generation.
- A poll performs bounded work: a fixed small accept/ticket budget, at most two
  retained admission tickets and the existing maximum two root jobs. No new thread
  per connection, unbounded Vec, busy wait or blocking receive on the owner loop.
- Never dispatch without a successful original one-shot admission receipt.
  Preserve same-pair/role/generation checks and original typed terminal errors.
- Listener failure, closed admission, job completion, cancellation, deadline,
  dropped caller or child death fences the pair and closes pending sockets.
  Completion still requires handler joins and actual child cleanup. No forced
  reservation release or claimed bound on an indefinitely stalled storage call.
- Actual existing static relay, fresh synthetic PF20 roots and real listeners:
  reversed connection order, both load/CAS/reload, wrong/duplicate-peer rejection,
  flood/pending limits without supervisor starvation, connection loss/cancel/
  deadline/drop, late receipt and actual reaping. Deterministic seams only for
  hard-to-trigger accept/poll errors; preserve failed attempts and exact final run.
- Non-root RTX/TMUX, formatting before affected tests, strict scopes and unchanged
  source/locks/artifacts. Reuse exclusive target/build lock rather than rebuild
  unrelated packages. Request only the manager-selected new-stage review allowance.

## Gate disposition requested with allocation

Reasoned policy1.7 N/A for this private, feature-gated internal pump only; later
native bootstrap and protected-user/PF26 qualification still require independently
isolated functional execution. It must not be presented as a live service or as
completion of PF27. The next subsequent boundary would be a separately allocated
fixed-system bootstrap/native deployment proof, not automatically authorized here.
