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

Latest received increments, September 12 01:47 UTC:
`56295f66825e87dc924705d86fcbcebf258ba7f9` combines accounting storage
`d898fbac0` and Task Node native adapter `d7bf73a52`. Each exact candidate passed
its first independent Astra High review with no findings. Final combined run
`8456cb98-c9fc-48d1-8e7b-b177c0e31206`:283 passed,0 skipped,7.439s; no process-leak
markers in retained full log. Earlier two warnings remain unattributed, not
retroactively resolved. Both normal libraries passed offline check in26.59s;
129 Python/governance tests passed. No formatter changed accepted bytes afterward.
Task Node's worker correctly stopped at433 tests against its300-line sub-budget;
manager explicitly reallocated435 test lines using smaller implementation,
preserving all required cases and unchanged hard500non-test/800total:799 actual.
This resolves allocation only; the clean review and tests supply separate evidence.
Original worker stop receipt is retained, not rewritten as compliant-at-return.
No production collection, sender, UI, human or release acceptance is claimed.

Earlier receiving checkpoint, September 12 01:07 UTC:
`486d2fb9481c01f7d73a6f8d9992c6e1b96359dd` includes exact quotation
`e8ffdad4e` and bound one-event engine `7e25982b5`. Both received a clean first
independent Astra High review, then parent verified hashes and integrated them
locally. Combined run `dbcabbc8-c24f-40d6-ac80-02617102842c`: 269 passed,
0 skipped, 6.628s, with two process-leak warnings (not test failures). The JUnit
report confirms zero failures/errors but does not identify those leak markers;
their provenance is unresolved, not assumed to be an accounting defect or an
existing issue. No repeated full run solely to suppress them. Both normal
libraries passed offline cargo check in22.51s. Also all
129 Python/governance regressions pass (54 Task Node, 48 accounting, 5 plan,
22 sprint); checkers remain 3 active / 115 current / 122 archived. Worker
formatting limitations remain in receipts; integrated Rust bytes are unchanged
from those formatted/reviewed candidates. The manager is preparing next
same-sprint storage and native-client allocations, not waiting on Travis.
Both new increments remain test-only; no live authority, collection or send.

Native receiving checkpoint (September 12 UTC / September 11 local):
`c33d47f6ccd00a64fcb05a057472d8ca9f0139d4` combines accounting
`3f39d7a65` and Task Node `2b281975a`. Both exact worker patches passed one
independent Astra High structured review with no findings before local
integration. Final combined native run `62b85020-a5cc-451c-bc1a-c07b0603c9bb`
passed all 244 tests (202 state, 42 Task Node), no skips, 6.415s. The earlier
worker-only LEAK marker on an unchanged state extraction test did not recur.
Combined Python/governance regressions passed 129 tests, plus two pinned
cross-language goldens; checkers report 3 active plans, 115 current / 122 archived.
Worker formatting limitations remain disclosed in their receipts; the integrated
Rust bytes match those formatted/reviewed candidates, with no formatter edits
after the final tests. This proves bounded native increments, not full sprint,
production collection, UI, live repository, benchmark or human acceptance.

| Workstream | Completed | Next action | Who owns the gate? |
| --- | --- | --- | --- |
| PF13 | No new completion inferred | Preserve owner task and paused security allowance | Existing PF13 owner / Travis; do not reactivate here |
| Accounting PF-60-S02 | S01 archived; journal/quotation/storage reviewed and integrated | Retained contributions/recorded-attempt deletion allocated | Manager dispatches; approved defaults unchanged |
| Task Node PF-80-S01 | Native adapter reviewed and integrated | Exact-goal observation reconciliation allocated | Offline work continues while operator design answers are pending |

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

The accepted answer resolved the default-policy decision. The
[S02 handoff](../research/agent-cost-accounting/s02-allocation.md) is now
independently reviewed, S01 accepted/archived and S02's first test-only increment
allocated. Later production wiring still needs its own evidence. Do not ask
Travis this same question again or claim that he enabled billing/live collection.

## Prior Task Node assignment — received, no longer writable

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

## Current next assignments

Both active plans and sprints now record receiving base
`56295f66825e87dc924705d86fcbcebf258ba7f9`. After this allocation's independent
review, parent fast-forwards idle clean workers to its committed HEAD and records
actual agent IDs/launch HEAD in the private continuation receipt. One Astra High
worker per lane; no additional reserved sprint. Earlier literal allocations above
and in old handoffs are historical, not permission to edit those files now.

Accounting: [five-file contributions/deletion](../research/agent-cost-accounting/contribution-deletion-allocation.md),
retained-detail one-version contribution reduction and atomic recorded-attempt
deletion/tombstones. Private storage child, exact connection-reader/journal guard
seams only. Compaction/cutoff/native ownership and production promotion follow.
No repeated policy question, live price fetch or billing. S03 remains gated.

Task Node: [five-file exact-goal reconciliation](../research/tasknode-integration/native-reconciliation-allocation.md),
build real exact-ID activity GET in memory and consume pinned injected response.
No Client/recovery changes, sender or retry.404 remains uncertain, never proof of
absence or permission to resend. Unknown native null expiry stays held.
No queue, automatic retry, production sender or live qualification is implied.

Parent delivered two grouped questions to Travis on September12: approve local
manual publisher/setup direction with existing account/Proposed PF-80 target,
and a separately reviewed native validity design for server-issued null expiry.
Record answers when received, without treating design approval as credential,
enrollment or exact-post authority. No answer is needed for current offline work.
These are delivered questions, not just a private blocker note; do not re-ask
historical account/task creation or approved accounting defaults.
Human packet must preserve an observed native trap: paused tracker recording
does not stop pending TUI sync. Do not launch linked production TUI as read-only.
Exact CLI cancel/status controls and unsupported event-retry commands are
documented in the new allocation; no shell credential repair or invented control.

Live entitlement/enrollment/ownership checks and a send still require a
separate exact account/origin/workspace/target/payload decision. Do not request
credentials before the offline candidate and human test plan are ready.
PF-79 beta and PF-81 remain draft until their recorded dependencies and own
decisions are met. This change does not waive human/live acceptance or quietly
split their prerequisite. Main landing, dashboard source cutover and deployment
are distinct later operations.
