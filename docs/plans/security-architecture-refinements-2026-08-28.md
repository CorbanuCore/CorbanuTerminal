# Security architecture refinements — 2026-08-28

This is an acceptance/sequencing appendix to the [active P0 plan](active/p0-security-levels.md),
not finished-feature guidance or a second active plan. Travis Good authorized
adopting the relevant findings after assessment. Change class: routine planning
and process; no runtime feature, platform or release acceptance is claimed.

Product authority: **Reconciled security scope — TO BUILD** — “Unknown or
unsupported protected paths fail visibly rather than falling back to raw secrets
or unscreened execution.” **P0 `/security` levels** — “Existing approval, sandbox,
vault, wallet, tool, network, and agent policies are unchanged.”

The raw Opus review and corrected assessment remain historical evidence under
`qa/security-levels/planning/opus-2026-08-28/`; the decisions below summarize them.
Source claims refer to OpenClaw pin
`13adff02ca3897768d80d2bca18f5acf08c55d91`, not proven Corbanu defects.

## Accepted changes and boundaries

| Review / validation | Decision | Execution owner |
| --- | --- | --- |
| OR-01 | Bounded three-worker allocation; measure effort/capacity, do not infer calendar duration from node counts | Plan delivery/integration owner; sprint process/checker |
| OR-02, OR-03 | Per-OS trusted-process design and tamper-resistant authoritative policy state | PF-27-S03 → PF-20-S02, PF-27-S04/S02 → PF-13-S05 |
| OR-04, OR-05 | Independent baseline plus upstream control/drift evidence; explicit hook/owner/test register | PF-21-S02, PF-22-S02, every hook-owning sprint, PF-26 |
| OR-06, OR-07, OR-08 | Completed preparation/interface sprints unblock construction, not protected activation | PF-31-S04, PF-33-S03, PF-34-S04; early PF-35-S01/S02 |
| OR-09 | Quantitative reservations/metering at credential transport reuse BoundedGrant | PF-13-S06/S03/S04 |
| OR-10 | Action/profile usability matrix; preserve data and control-flow ancestry | PF-30-S03, PF-23-S01, PF-26-S02 |
| OR-11 | Immediate restriction with separate uncertain-effect reconciliation | PF-19, PF-25-S02, PF-38-S03, PF-40-S03 |
| OR-12 | Typed, bounded descendant request UI; requests do not grant authority | PF-25-S01 |
| OR-13, OR-15 | New-route closed-world tests and named final capture-sink tests | PF-30-S01, PF-22, PF-28, PF-13-S05 |
| OR-16 | Preserve distinction between code present and behavior accepted | Existing Done/Remaining ledgers; no historical-evidence rewriting |
| OR-17 | Same-run re-registration with cached TLS handlers and fresh connections | PF-13-S04, PF-27-S04 |
| OR-18, OR-19 | Explicit absent/empty/public/private policy semantics and actual-peer checks | PF-33-S03/S01/S02 |
| OR-20 | Reject claimed capacity exploit; retain capacity-before-commit and filename/hash tests | PF-30-S02; corrected assessment |
| OR-21 | Protocol-specific buffer/latency targets; no unexamined ingress prefixes | PF-28-S02, PF-34-S04, PF-35-S03, PF-26-S02 |
| OR-22 | Retain qualified-enabled or evidenced disabled-no-qualified-vendor disposition | PF-36-S02; no commercial dependency invented |
| Additional validation | Add missing browser-login screening edge and actual detector/policy integration edges | PF-37-S01 depends on PF-34-S03; PF-35-S03 depends on PF-34-S01 and PF-30-S03/PF-23-S01 |
| Additional validation | Move durable event/commit/recovery foundation before its consumers | PF-41-S03 → PF-22, broker, quarantine, financial, Sweep; PF-41-S02 keeps inspection/export |

Do not adopt a weaker “Moderate v1,” automatic clean-value declassification,
same-candidate-only compatibility oracle, or waiting for financial settlement
before emergency kill. Do not treat notarization, a separate account or universal
administrator installation as proven necessary/sufficient OS mechanisms. The
report's arithmetic deadline claim is not measured evidence. OR-14 is absent.

## Reconciliation with completed upstream work

Upstream `1bdc515bff48a4d9048dae7d06c6214e884265bc` completed PF-15–22 and
PF-13-S01 under their original contracts. Preserve those archives and evidence.
The new guarantees above belong to PF-13-S06 and PF-19/20/21/22-S02, not reopened
S01 records. The reconciled graph has **64 remaining plus nine completed nodes**;
longest total/remaining-only chains are **34/31**. Historical review snapshots
remain unchanged. PF-13-S02 is ready; no additional sprint is activated.

The [merge record](security-upstream-reconciliation-2026-08-28.md) distinguishes
preserved historical results, new planning validation and still-pending runtime
qualification. Upstream's typed tmux contract is retained in PF-26-S01/S02.

## Allocation and scheduling

Follow [bounded parallel implementation](../sprints/index.md#bounded-parallel-implementation).
The active plan opts into a limit of three; its existing delivery owner Jim
Ricketts is integration owner. Only the recorded foundation worktree is currently
allocated. Additional named workers, distinct worktrees/branches/base commits and
complete write scopes must be recorded before parallel implementation starts.
No sprint is made ready or in progress by this amendment.

| Suggested lane | Early work after allocation | Integration gate / file ownership |
| --- | --- | --- |
| Foundation/platform | PF-13-S02 resolver; PF-27-S03 design/probes; new PF-13/19/20/21/22 follow-ups and early PF-41-S03 | Core/protocol/policy/state changes serialized by integration owner; completed platform contract precedes protected-state implementation |
| Browser preparation | PF-31-S04 artifact/engine fixtures; PF-33-S03 pure destination contract | Retriever packaging and destination-contract files only; live PF-31-S01 retains completed broker/launch/network dependencies |
| Ingress/classifier preparation | PF-34-S04 segment/verdict fixtures → PF-35-S01 corpus/evaluator → PF-35-S02 CPU artifact | Pure content contract/evaluator paths; live sanitizer and PF-35-S03 deterministic policy integration remain required |

Later rotate slots to login, financial work or Exa/Brave/SearXNG adapters. The
three provider adapters share completed PF-32-S02, but are concurrent only if
registry, manifests, lockfiles and shared tests are disjoint or integrated in a
separate serialized step. Three lanes are a capacity ceiling, not three staffed
teams or permission for every lane to edit Core simultaneously.

The historical pre-merge amendment had **68 nodes and a longest unweighted chain of 35 nodes**,
versus 63 and 34 before refinement. Five explicit preparation/foundation units
add auditable gates; this does not claim a shorter calendar schedule. The graph
exposes earlier independent work, and node count is not a duration estimate.

Before allocating the second worker, Jim records a dated estimate for each
remaining unit: effort range, named capacity, platform access, integration/rework
allowance, reviewer availability and evidence lead time. Calculate a resource-
constrained schedule and flag the October 8 risk to product authority. Keep the
current deadline pending that decision; do not waive scope or release evidence.
The existing single-worker PF-13-S02 allocation need not stop for this estimate.

## Platform and authoritative-state acceptance

PF-27-S03 produces the concrete matrix, with one row per OS/backend combination.
Each row must name trusted controller/broker identities, untrusted execution
identities, process-memory/debug and inherited-handle restrictions, filesystem
and IPC permissions, network enforcement, signing/entitlements, required user
approval/elevation, capability probe and unsupported behavior. Mechanisms must
be demonstrated, not selected by platform folklore.

The human-controlled authoritative level, grants, revocation/kill generations,
integrity roots and recovery state cannot be overwritten, deleted, swapped,
symlinked or rolled back by an agent. Ordinary editable preferences are not that
authority. Missing state after a protected installation cannot masquerade as a
new legacy Permissive install. PF-20 owns persistence; PF-27 owns process access
controls; PF-13-S05 tests the composed actual agent context on every target OS.

Early contract probes and production containment evidence are separate. Protected
activation remains unavailable if required identity, migration, broker, audit,
network, retriever or local screening readiness is absent or stale. Runtime
health reports must bind their measured generation and expire on relevant change.

## Codex integration seam register

PF-22-S02 establishes `qa/security-levels/upstream-seams.json` and its contract tests;
each hook-owning sprint extends it before completion. The following are candidate
integration paths from the plan, not a claim that this checkout qualifies them:

| Candidate upstream seam | Corbanu-owned responsibility | Owning sprint / required regression |
| --- | --- | --- |
| `core/src/tools/{router,registry}.rs`, `mcp_tool_call.rs`, `exec.rs` | Protected dispatch, closed-world route inventory | PF-23/PF-30: unknown ingress/egress, force-allow cannot grant |
| `core/src/agent/{control,registry}.rs` | Child identity, inherited level/taint/generation | PF-22/PF-30: nested child, mailbox, resume, stale actor |
| `core/src/config/network_proxy_spec.rs`, `network-proxy/src/credential_broker.rs` | Typed broker/transport adapter | PF-13/PF-27/PF-33: bounds, actual peer, stale pooled/TLS state |
| `config/src/config_toml.rs`, `tui/src/app/config_persistence.rs` | Human update adapter to controller-owned state | PF-20/PF-24: forged update, tamper/restart, schema migration |
| `protocol/src/models.rs`, provider context adapters, memory readers/writers | Durable source identity and conservative ancestry | PF-30: unknown variants, round trips, compaction/import/hash/capacity |
| `tui/src/bottom_pane/approval_overlay.rs`, slash dispatch | Human-only exact grant/request/transition UI | PF-24/PF-25: Esc, spoofed labels, exact actor/effect, expiry |

For each row record the **exact existing symbol**, upstream commit, Corbanu-owned
module, named owner, semantic contract, actual test command and last upstream
revision tested. Unverified entries remain pending. The lead developer already
owns upstream parity; this register supplies execution detail, not new authority.
Use small hooks and fork-owned modules; protocol fields may be necessary. A side
table is acceptable only with proven identity, persistence and missing-record
semantics, not as an automatic compatibility improvement.

Every relevant upstream merge requires reviewed hook diffs and reruns of seam,
ingress/egress, inheritance, persistence and Permissive comparisons. Keep the
independent pre-feature baseline, add an upstream-aligned control with a reviewed
drift ledger, and use feature-on/off comparisons only as supplemental checks.
Never resolve a security-hook conflict simply by accepting either side.

## Taint, authority and usability

PF-30/PF-23 implement this matrix without treating classifier confidence as
authority. PF-26 measures realistic research flows in both live repositories.

| Action after untrusted content | Moderate | Aggressive |
| --- | --- | --- |
| Analysis of admitted content | Continue with preserved source/taint; required screening still applies | Same within permitted context |
| Public research | Isolated, screened and destination-validated under resolved profile | Same plus explicit narrow provider/destination grant |
| Unchanged preauthorized narrow action | Existing grant may satisfy only its unchanged actor/resource/effect/budget | Same; sign and broadcast still each require exact human approval |
| New/changed sensitive effect or disclosure | Trusted reconstruction or exact bounded human mandate; model-selected clean-looking values are insufficient | Explicit narrow permission plus required exact approvals |
| Policy mutation, raw secret export, unknown route | Deny agent authority; unknown content/path fails conservatively | Deny |

Keep both data and control-flow ancestry across turns, compaction, memory,
children and imports. Exact approval authorizes an effect, not declassification.
Descendant requests are typed untrusted notices with rate/deduplication bounds,
not authority-bearing messages. A human can inspect, deny or approve an exact
eligible grant without authorizing an adjacent action.

Before PF-26-S02 readiness, product authority approves numeric usability targets
for the fixed benign/hostile research tasks: completion rate, approval count,
false-positive interruptions, time-to-first-safe-output and end-to-end latency.
PF-35's existing quality/CPU targets remain unchanged. Missing targets/evidence
stay pending; throughput cannot justify releasing unscreened prefixes or relaxing
deterministic protection.

## Durable events and irreversible effects

PF-41-S03 owns common IDs, producer ownership, reservation/intent, acknowledgment,
integrity checkpoints and crash recovery. Consumers can use separate stores but
must specify their cross-store failure protocol and test it. A missing required
record blocks new protected dispatch; a hash chain does not defeat a compromised
trusted host. Later PF-41-S02 composes actual producers and safe support export.

Emergency restriction fences new work even if audit storage is unavailable or a
financial effect is submitted/unknown. Keep unresolved receipts visible and
reconcile separately; never label uncertainty cancelled or blindly rebroadcast.
Crash/restart must remain fail-closed without claiming unsupported durability.

## Required integration regressions

Every row needs a final candidate case, expected/actual result and safe artifact
in PF-26's crosswalk, in addition to its owning sprint's tests:

- All-OS process-memory, handles, filesystem/IPC/network and policy-tamper/restart canaries.
- Open-channel/upload/queued-work revocation; same-run fresh-connection stale TLS cache; unaffected sibling survival.
- Managed secrets in short/encoded/split/rotating responses, headers/trailers/SSE/errors/debug capture; resource exhaustion denies safely.
- Actual DNS/peer/redirect binding, NO_PROXY/alternate proxy/direct socket/UDP/QUIC, separate private-service policy.
- New unregistered ingress/egress, corrupt/missing envelopes and detector allow/error/timeout; no unapproved effect.
- Memory/summary/import/export/children and control-flow taint; provenance capacity before commit and read-time content identity.
- Migration ownership/crash without plaintext backups; uncertain submission plus immediate kill/restart without duplicate effect.
- Upstream seam/dual-baseline regression and final true-TUI workflows, both live repositories and named human acceptance.

These are required future evidence, not results produced by a planning amendment.
