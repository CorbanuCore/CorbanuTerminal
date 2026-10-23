# RETURN — owner-assurance-109

Partial implementation return to the Fable manager. The qualification regression
is fixed; fresh supervisor projections are downgraded. Distinct held notices,
explicit supervisor-cause wording and aging of an already-published snapshot
remain blocked by the dispatch write allowlist. This is not a completed
functional handoff or human-test readiness claim.

Brief SHA-256 verified before other reads:
`a36bb357791f3510d1d2dff90c48480ec4fdb3c184730ae9d86799a7e62c9265`.
Assigned base and observed HEAD:
`bc4493331f390801ef14016caeceae5c2ca94bab`.
Allocation digest:
`fc3fe80a0c39339dc561a7430622f0aa03346c3624a1c17c06b2b1b85d8b271b`.
Claim: `943101e3-f795-4ad3-9095-7f49027885e0`.

Bounded reliability fix under product-spec heading **Internal delivery control
— TO BUILD**: “durable event dispatch, acknowledgments and watchdog”; “Slack
connection, exact destination and delivery require verification”. Existing
initiative-delivery-control / PF-80-S01 context; this worker does not advance
plan/sprint status or claim product acceptance.

## Supervisor downgrade and remaining renderer defect

`project_disclosure` now downgrades an otherwise `last-verified` projection to
`stale` for `observation-stale`, or `unknown` for unavailable/missing/malformed
observations. The structured supervisor reason is retained. Both manager status
and a newly generated dashboard projection exercise this shared function.
Existing hold/fence/unhealthy evidence still prevents verified status.

The unchanged renderer only displays the generic state, so it does **not**
explicitly say “supervisor observation stale” or “supervisor observation
unreadable”. That part of requirement 1 is unfinished. The exact rendered
fresh-projection notices are:

**Supervisor stale:**

> Slack observation: stale; assessed 2026-09-12T12:00:06Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

**Supervisor unreadable:**

> Slack observation: unknown; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

There is also a remaining read-time defect in `decision_feed.slack_health`:
it reassesses supervisor freshness but selects the top state from saved status.
A healthy snapshot generated at 12:00:00 and rendered at 12:00:06 still says
`last-verified` although its returned supervisor reason is `observation-stale`.
The seventh [captured observation](owner-assurance-109-observations.jsonl)
proves that residual false assurance. Its notice is byte-for-byte identical to
the healthy notice. The fix is therefore explicitly limited to fresh projections.

The dispatch allows neither `decision_feed.py` / `test_decision_feed.py` nor
`attention.py` / `test_attention.py`. These are the remaining implementation
and regression-test boundaries; they have not been edited or bypassed through
runtime patching in production.

## Qualification regression and recovery

Previously an invalid explicit `gap_review` on a healthy quiet journal wrote
`hold="qualifying"` before validation failed. An ordinary retry then required a
review because the durable hold was no longer empty. The assigned base was
executed against a disposable store: invalid input created exactly that hold,
and retrying plain qualification left it there. The current exact-review path
recovered it. See [base replay](owner-assurance-109-base-regression.json) and
[reproducer](owner_assurance_109_reproduce.py).

The change validates required review input before persisting the hold or
performing credential/HTTP access, then validates again after authentication.
Invalid quiet-journal input leaves the entire transport file byte-for-byte
unchanged. Intervening ingress or lifecycle/fence changes still prevent a
successful qualification; no rollback erases intervening faults. The existing
explicit quiet-legacy-history review remains supported.

The reported live `qualifying` hold and frozen `last_verified` are consistent
with this reproduced bug, but do not prove its cause. Authentication failure,
intervening state changes or final persistence failure can also leave that hold.
**No live journal was inspected or repaired.**

Operator recovery for a journal already pinned in `qualifying`:

1. Use the existing owner manager with the correct binding and a genuinely
   connected, renewing listener session. If it has died, restore that listener
   through the supported supervisor path before collecting review facts.
2. Inspect the outstanding gap/quarantine and resolve or drain pending events
   through the existing owner operations. Qualification requires no undrained
   transport events. Do not manually edit the hold or create a fake session.
3. Collect a new exact review under the existing store/transport locks:
   current `watermark`, actual ingress-fence count as `ingress`, canonical
   `decisions.digest(binding)`, live `session` identifier and `epoch`, and a
   nonempty `evidence` identifier for the actual owner review. Collect fresh
   supported-UI evidence (`binding`, `observed_at`, `receipt`, and the exact
   existing `slack_transport.UI_CHECKS` checklist).
4. Invoke the existing owner-controlled `decision_manager.py qualify --store
   <operator-store> --live`, sending one newline-terminated owner JSON object
   on stdin containing `binding`, `ui_evidence`, and `gap_review`. Use the
   established credential mechanism; credentials are not part of that object.
   Plain qualification without `gap_review` will not clear an existing
   `qualifying` hold.
5. After success, verify the appended review, refreshed `last_verified` and
   `hold=null`, then regenerate the projection and confirm fresh supervisor
   observation. If ingress/session/epoch changes during the operation, inspect
   and collect a new review; do not reuse stale numbers or force-clear the hold.

A successful review acknowledges the reviewed uncertainty; it does not claim
that an expired or quarantined reply was delivered. This recovery uses the
existing operation and was verified only with disposable stores.

## Three held notices, verbatim — still unsplit

Requirement 3 is not implemented: the actual renderer is outside the writable
scope. These are the final-tree outputs, not proposed replacement text.

**Waiting for route:**

> Slack observation: held; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

**Quarantined:**

> Slack observation: held; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

**Expired undelivered:**

> Slack observation: held; assessed 2026-09-12T12:15:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

## Six-situation surface table

Every row uses the unchanged suffix: “This is a saved observation, not a live
connection or work authorization. If held or stale, answer in the manager task.”
All literal last-verified timestamps are `2026-09-12T12:00:00Z`.

| Situation | Rendered prefix | Assessed at | Remaining limitation |
| --- | --- | --- | --- |
| Healthy | `Slack observation: last-verified` | `2026-09-12T12:00:00Z` | No explicit no-action wording |
| Held-waiting | `Slack observation: held` | `2026-09-12T12:00:00Z` | Does not tell operator to wait |
| Quarantined | `Slack observation: held` | `2026-09-12T12:00:00Z` | Does not identify quarantine/review action |
| Expired | `Slack observation: held` | `2026-09-12T12:15:00Z` | Does not disclose expiry/undelivered disposition |
| Supervisor stale, freshly projected | `Slack observation: stale` | `2026-09-12T12:00:06Z` | Does not identify supervisor cause; old published snapshot still incorrectly renders verified |
| Supervisor unreadable, freshly projected | `Slack observation: unknown` | `2026-09-12T12:00:00Z` | Does not identify unreadable supervisor cause |

**Exact duplicate pair:** held-waiting / quarantined. Expired has identical
state/action wording but a different timestamp, which is not a useful state
distinction. Separately, healthy / saved-snapshot-aged are exact duplicates.
The three-way operator distinction is not claimed as solved.

## Verification

Full discovery: **866 passed** (481.753s). Focused owner/feed/attention/control/
promotion/preflight: **296 passed** (98.800s). Targeted regressions: **5 passed**
(7.236s). JavaScript renderer: **1 program passed**. Captured **7 notices** and
**1 assigned-base regression/recovery replay**. All test/evidence gates exited
0; failures/errors/skips: **0**; failure names: **none**. One initial read-only
inspection command exited 1 because no nested AGENTS files matched; no other
top-level command returned nonzero. Source/tests changed **128 added / 8 deleted**
lines across four allowed files. Commands, counts, line locations and hashes are
recorded in [the verification record](owner-assurance-109-verification.md).

No live transport, profile, credential store, native prompt, Rust test,
workspace formatter, commit or push was used. Only disposable profiles/stores
and synthetic SDK endpoints were used. Independent code-blind design,
permission-isolated execution and evidence review, packaged true-TUI proof,
live-repository qualification and named-human acceptance are not claimed.
The manager must retain these functional gates; this is a bounded revise-worker
return with open scope blockers, not a release or human-testing handoff.

## Brief corrections

The regression and fresh-projection false-assurance findings are correct.
The reported live journal cannot be diagnosed conclusively from the supplied
hold/timestamp alone. “Dead supervisor reads as verified and healthy” describes
the misleading top-level assurance: the nested supervisor health already said
`unknown`, rather than `healthy`. The unreadable and stale reasons were already
available structurally. Their absence from the notice remains unresolved.
The earlier withdrawn identical-string finding has not been reinstated:
expiry differs literally in time, but waiting/quarantine are actual identical
strings and all three lack the requested state/action distinction.
