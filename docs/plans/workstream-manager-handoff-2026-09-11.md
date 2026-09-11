# Workstream continuation — September 11

Status: local receiving integration, not remote main or release. Travis requested
that management fix stalled progression. The
[sprint continuation rule](../sprints/index.md#manager-owned-continuation) owns
the mandate. Product citation: **Internal delivery control — TO BUILD**,
“Use sequential sprints per initiative”. No fourth initiative is created.

## Receiving checkpoint

- Owner/worktree: Codex management,
  `/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911`.
- Branch: `integrate/management-workstreams-20260911`.
- Combined baseline: `0415a00dc3d3ee55a96662920c3eedbc0f0d4838`.
- Received planning `db24b600aa6529f9d14b9f141ab8862efd6d3b26`,
  accounting `1c5978690ee4cec0f3d2cab60ad13bf447ebdf84`, and Task Node
  `8640ca452b3b3955c917782dd78f25b392cd2142` without conflicts.
- Remote main remains `7bb0697cf8a5e88fc50bc232b65fea8d77f6da4a`.
  No main merge, push, release or server source cutover is authorized here.
- The previous planning checkout is historical, not the receiving owner now.
  The dashboard still publishes the explicitly labeled recovery source.

Both worker candidates had clean final Astra High structured reviews before
integration. Three accounting and two Task Node review invocations were recorded
in the private follow-up receipt; no exhausted PF13 allowance was reused.
On the combined baseline, 48 accounting tests passed in 0.069s and 54 Task Node
tests passed in 2.518s. Both governance checkers passed: 3 active plans, 116 current
and 121 archived sprints. Task Node service/timer messages were mocked fixture
output, not an actual deployment. This is now combined-tree evidence, not a
promise to integrate later. New allocation/policy changes receive their own review.

## Current queue and owners

| Workstream | Completed | Next action | Who owns the gate? |
| --- | --- | --- | --- |
| PF13 | No new completion inferred | Preserve owner task and paused security allowance | Existing PF13 owner / Travis; do not reactivate here |
| Accounting PF-60-S01 | Reviewed contract, fixtures, corrections, local tests and approved defaults | Finish concrete S02 adapter handoff and reconcile/archive S01 | Manager; no repeated product question |
| Task Node PF-80-S01 | Reviewed offline port, corrections, local combined-tree tests | Native pure preparation slice below, then independent review | Manager allocates and dispatches; no live authority needed |

Missing preparation, allocation or receiving evidence is manager work. It is
not a request for Travis to coordinate workers. The scheduled parent must keep
one current assignment/next action or an actually delivered decision per lane.

## Accounting decision

Owner: Travis. Decision: **approved** in this task on September 11, in response
to the exact accounting-defaults question (“Approve these defaults”). Accept the existing
[contract v1](../research/agent-cost-accounting/contract.md) with these explicit
implementation defaults below. No billing or live collection was enabled:

- Keep measured usage, estimated cost, billed cost, provider allowance and
  Corbanu API balance separate. Unknown never means zero or free.
- USD-only token estimates; exact arithmetic, sum before six-decimal half-even
  display rounding, explicitly mark rounded/sub-micro amounts.
- Accept only operator-approved snapshots from provider-published rates or the
  native Corbanu API catalog, prospectively from observation/approval time.
  Pin immutable price identity with each estimate. No backdating, invoice
  ingestion, live price fetching or invented prices is implied.
- Retain local numeric detail for 90 days and aggregates for 365 days. Proposed
  replay horizon: 365 days with minimal opaque deduplication tombstones; reject
  older imports rather than double count. No prompt/tool-body collection.
- Explicit user deletion removes the associated detail and attributable
  aggregates; retain only the minimal opaque replay tombstones for that horizon.
  Explain this retention in the deletion UI and allow export before deletion.
- Historical usage without trustworthy presence/attempt/pricing evidence stays
  unknown. No rebilling or balance-derived spend.

The accepted answer resolves the default-policy decision, not missing technical
evidence. Manager now finishes the exact S02 handoff, reconciles final evidence,
and archives S01 once its remaining exit conditions are satisfied. Do not ask
Travis this same question again or claim that he enabled billing/live collection.

## Task Node next assignment

Same feature/sprint: PF-80 / PF-80-S01, in progress. Same worker checkout/branch:
`/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`,
`workstream/tasknode-pf80-s01-20260911`. Recorded base is the combined checkpoint
above; fast-forward the clean idle worker to the reviewed allocation commit before
dispatch and record the actual launch HEAD in the run receipt.

Literal worker writes:
`codex-rs/tasknode-session/src/delivery_goal.rs`,
`codex-rs/tasknode-session/src/delivery_goal_tests.rs`,
`codex-rs/tasknode-session/src/lib.rs` (one module declaration only),
`qa/initiative-control/pf-80-s01/native-preparation/`.
Manager delegates that single registration to this worker, with no concurrent
writer in the crate; the manager reviews and integrates it before consumers.
No CLI/client/tracker/auth/config/dependency/lockfile/BUILD changes are allocated.

Deliver one pure preparation module for an externally selected immutable
delivery-control goal. Preserve the existing payload/event identity and bind
the exact approved full-payload digest; reject changed identity/payload,
non-goal or unknown fields, unsafe/overlong content and branch bytes, wrong
workspace/tasks, invalid sequences/timestamps and historical PF-76 ambiguity.
Use synthetic inputs and explicit time/expected mapping arguments; no ambient
profile, filesystem, credential, network, outbox scan, capture, sync or retry.
Cross-language goldens must match the receiving Python adapter and pinned
first-party shape; do not silently replace its canonical hash representation.
Return advisory preparation only, never a send authorization.

Feature boundary: internal library only, no production call sites, menus,
commands, jobs or resume hooks. Existing posting stays OFF. This is the first
native preparation slice, not a sender or completed integration.

Tests: focused `just test -p codex-tasknode-session delivery_goal` and the full
affected crate through `just test`, after `just fmt` and scoped fix if needed.
No direct cargo test, shared target cleanup or toolchain/manifest repair.
Record actual nonzero tests, failures and environmental blockers. Keep
non-mechanical changes within the Rust size guidance; return a smaller coherent
slice if the exact contract cannot fit. One implementation assignment followed
by one independent Astra High review; scoped corrections follow autoreview's
two-cycle reclassification rule, never resetting an exhausted allowance.

Upstream record: canonical remote is `https://github.com/openai/codex.git`.
Locally verified common ancestor of this fork and cached upstream/main is
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`; cached upstream/main is
`1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6`. Remote HEAD observed during
allocation is `c210f4c222f8f7a447d0b3a013f3cb9cc3cd43b0`, not integrated.
This slice touches only the product-owned tasknode-session crate, with one
manager-owned registration line. No upstream protocol/history/permission or
provider-wire change; retain existing crate behavior with full crate tests.
No upstream upgrade qualification is claimed.

## Following assignments, not activated

After native preparation review, manager allocates profile-bound one-event
transport using the existing Client, with explicit account/origin/event/digest
checks and no bulk queue path. Preparing that allocation is manager work;
implement only after recording its exact boundaries and tests. Existing profile
link/expiry/recovery mechanisms must be reused, not redesigned.

Live entitlement/enrollment/ownership checks and a send still require a
separate exact account/origin/workspace/target/payload decision. Do not request
credentials before the offline candidate and human test plan are ready.
PF-79 beta and PF-81 remain draft until their recorded dependencies and own
decisions are met. This change does not waive human/live acceptance or quietly
split their prerequisite. Main landing, dashboard source cutover and deployment
are distinct later operations.
