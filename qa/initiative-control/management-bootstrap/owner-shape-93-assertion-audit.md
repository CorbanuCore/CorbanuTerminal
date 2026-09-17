# owner-shape-93 — promotion assertion audit

No live owner root, coordinator, installation contents or authentication files
were inspected. The frozen brief supplies only the cycle input shape and a
report that a transport and first action were prepared. It supplies no status
JSON, ownership map, installation receipt or selected action record. Therefore
“unknown” below is a material evidence gap, not a pass. “Reported” means the
manager's description, not a fresh state measurement.

## Failures established from the supplied shape, in execution order

1. Original recipe: `action["inputs"]["worker"]["worktree"]` raises
   `KeyError: worker` on every supplied flat cycle input. It never reaches the
   membership assertion. The corrected recipe deliberately reports
   `recorded_worker_runtime_required` at this point until preparation is fixed.
2. If execution were continued independently, the next original expression,
   `action["inputs"]["worker"]["policy"] == "--yolo"`, also cannot evaluate.
   Neither provider nor policy is recorded anywhere in the supplied eight fields.
   Adding only a worktree or renaming reasoning_effort would not resolve it.
3. No further live failure can be established honestly from the supplied inputs.
   In particular, worktree membership, manager ownership, holds, pins, resource
   conflicts and enabled/armed state cannot be inferred from a model or action ID.

These are two evaluations of the same missing binding, not two observed live
command failures. The assignment currently running in this worktree is itself
flat, but its provided claim means it is already dispatched; it is not a
prepared candidate for promotion. No claim is made that the separate action the
manager mentioned has that same status.

## Complete assertion walk in execution order

The following covers every explicit assertion in the corrected executable
recipe, plus stateful validators and important implicit lookups. Repeated
coverage-helper assertions are listed at their first call and reapplied at the
two later calls. Every future-state result remains unobserved for live state.

| Order | Assertion or enforced prerequisite | What supplied live evidence establishes |
| --- | --- | --- |
| 1 | Private schedule, readable installation receipt, phase installed; pins identify Python/runtime/config | Unknown; paths/receipt not supplied |
| 2 | Config has no transport (one-time fixture-only starting installation) | Unknown; preparing a transport file does not show it was installed |
| 3 | Transport exact schema, absolute paths, private runs directory, binary hash, no linked binary/tmux paths, auth basename | Transport preparation reported; contents/pins unknown. Validator does not authenticate or prove inference access |
| 4 | Selected action set nonempty; each ID exists | First action preparation reported; exact selection/record unknown |
| 5 | Manager is None | Unknown; must finish/reconcile any outstanding manager cycle |
| 6 | Every selected action is prepared and kind is a supported worker kind | Prepared reported for a different prospective action; exact kind/status unknown |
| 7 | Recorded five-field runtime, explicit --yolo, nonempty strings/identifiers, absolute worktree and agreement with flat duplicates | **Fails for every supplied unbridged cycle shape** |
| 8 | Runtime worktree belongs to configured worktrees | Unknown; missing worker blocked the old lookup first |
| 9 | Allocation digest matches current allocation; inputs exactly copy its inputs plus allocation ID | Unknown live; coordinator acceptance enforces this initially, but snapshot is needed for present state |
| 10 | Sprint/workstream executable; resources available | Unknown; active same-resource work may block even a valid prepared action |
| 11 | Coordinator dispatch enabled | Unknown |
| 12 | Installed-runtime activation-status command exits 0; complete and armed | Status read reported, result not supplied |
| 13 | Candidate-reader status complete, fixture-only scope, no interrupted activation recovery, no unresolved holds, no active owner claims | Unknown; even an excluded hand hold blocks reconfigure. New checks expose these before disarm |
| 14 | BEFORE coverage complete; manager covered; future defaults covered; hand guidance says covered by this watchdog | Expected candidate semantics for fixture scope; no live snapshot |
| 15 | BEFORE covered IDs exactly equal watchdog_will_report rows; excluded IDs match overdue/unreported dispatching/dispatched/running rows outside coverage; excluded set empty | Unknown counts/rows; prepared actions are not overdue coverage |
| 16 | Disarm exits 0 using current generation; uninstall exits 0 and recorded job becomes absent | Future effects, not current facts; generation/service races may refuse |
| 17 | Received source/assets copied; OFF reconfigure exits 0 | Requires same coordinator/config receipt, uninstalled absent job, all operations applied, no unresolved holds, no active owner claims, valid replacement config. Current satisfaction unknown |
| 18 | Fresh snapshot still has manager None; all selected actions still prepared | Must recheck while cooperating dispatchers are quiescent; unknown |
| 19 | Handoff exits 0 and returns HANDED_OFF | Exact revision, exhaustive initial pending map, from/to/claim/status/digest fences must match; unknown until commit |
| 20 | Status exits 0, complete and off; owner-owned set equals selected set | Future postcondition; unknown |
| 21 | BEFORE ARM coverage helper: complete, manager covered, future defaults excluded, manager-responsible hand guidance, covered/excluded IDs consistent | Candidate semantics expected after hand-default partition; actual rows unknown |
| 22 | Activation decision has actual authority/reference, integer revision, next generation, exact config/package hashes, tmux-workers scope | Operator inputs unknown; a nonempty label is not proof of real authorization |
| 23 | Repin/install command exits 0 preserving Python/hash, label/domain/cadence/publication | Requires OFF uninstalled receipt, valid private pins/source/template, permitted service domain and absent conflicting service; unknown |
| 24 | Explicit OFF tick exits exactly 2 | Expected refusal; preserve reason and confirm owner_off rather than assuming every exit 2 means owner_off |
| 25 | Arm command exits 0 | Requires unchanged pins/config/package, next generation, OFF state, validated authority and dispatch handoff; unknown |
| 26 | Schedule recover command exits 0 | Requires valid installation/state and recorded recovery reason; does not resolve operation holds |
| 27 | Final status exits 0, complete, armed, tmux-workers | Future postcondition; unknown |
| 28 | AFTER coverage helper repeats partitioned predicates and ID-set consistency | Future postcondition; unknown |
| 29 | First interval/ACK/START/working/RETURN observations, single claims/effects, unchanged hand claims, steady successful ticks | Not code assertions but mandatory observation instructions. **No real-worker evidence supplied or obtained this round** |

Every `run` call additionally checks its expected exit code (normally 0).
Coverage predicates are semantics, not service-admission proof. New actions
default to hand after partition; manual --hand-run does not provide recurring
hand watchdog supervision.

The synthetic exact-recipe replay exercises these predicates on disposable
state only. It does not change any “unknown live” entry to “pass live.”

## Additional disposable failure observed

The second synthetic replay refused at order 16: `activate --owner uninstall`
exited 1 because publication encountered an existing
`publish/owner-recurrence.json.pending` file. Its cleanup hit the same exception.
The receipt had reached `uninstalled` before that publication exception.
The raw replay and stderr are retained; no pending file was deleted to make a
replay pass. This is a demonstrated disposable cutover failure, **not** proof that
the live installation has a pending file. The fixed-name exclusive publication
write and bootout timing need separate reconciliation before calling promotion
robust. A fresh successful run cannot erase this attempt.

The first replay completed the recipe and reached returned, then failed the
harness's immediate generation assertion. A later retained inspection found
armed tmux-workers generation 3. The original status value was not logged, so
lock contention is an inference; the harness now preserves each advisory
status attempt and waits at most five seconds for a complete read.

## Required order before attempting live promotion

Round 94 supersedes the sequence below with the six prerequisites and the
[runnable five-item preflight](owner-promotion-94-preflight.md). It inserts the
explicit stale-publication refusal before any disarm/uninstall, checks all known
cutover inputs up front, and retains future-state command fences. The round 93
failures and unknown-live findings above remain historical evidence.

1. Integrate the named bridge into actual manager allocation preparation, with
   explicit authorized provider and policy. Preserve the seven task/runtime
   fields plus allocation ID; add the worker block before digest/acceptance.
2. Finish active reservations before replacing allocations. Use coordinator
   put_allocation replacement to cancel obsolete prepared actions, then a fresh
   accepted manager decision with new IDs. Never edit claimed/frozen inputs.
3. Supply an approved isolated real-worker inference lane and non-live transport;
   qualify real ACK/START/RETURN or have product authority explicitly accept the
   named limited first-action qualification. This round makes no such acceptance.
4. The live manager must supply/read its actual receipt, config and status and
   evaluate the full table while dispatchers are quiescent. Preserve failures.
5. Execute the corrected recipe only after those prerequisites. Installing this
   round alone cannot promote the currently described flat actions.
