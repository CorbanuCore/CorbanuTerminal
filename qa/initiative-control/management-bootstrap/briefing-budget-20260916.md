# Manager cycles are blocked by the briefing budget — measured, not guessed

**Fable, 2026-09-16.** No manager cycle can be prepared, so no new action can be
dispatched. Direct coordinator operations, reviews and receives still work,
which is why the S02 lifecycle and S03 registration completed after this began.

## The numbers

| Part | Bytes |
| --- | ---: |
| Total briefing | **101,561** |
| Everything except `original_evidence` | 55,211 |
| `original_evidence` | 46,350 |
| — of which mandatory event bodies (24 events) | 11,230 |
| — of which optional expansion from actions and allocations | 35,120 |
| Limit | 65,536 |

## Why the existing lever does not help

`fit_briefing` already does the right thing in principle: it walks the event
list down from 24 to 1, looking for the largest FIFO prefix that fits, and
restricts the run without consuming the deferred events. **It fails at every
count, including 1.**

The reason is that the 35,120 bytes of optional expansion come from evidence
referenced by *actions and allocations*, not by events. Shrinking the event
prefix does not shrink them, so the lever cannot reach the cost.

## Why the reserve does not help

The gap is about 36 KiB. The standing reserve is 15 KiB. I drew 13 KiB, measured
afterwards, found it could not close the gap, and returned it to zero rather than
leave a drawn grant standing as a false fix.

## The cost is mine

`original_evidence` is expanded verification evidence, and I wrote long prose
verdicts into `verify()` all session — why a worker stopped, what a review found,
what a gate did and did not establish. Each was worth writing once. They inline
into every later briefing, so the manager pays for all of them on every cycle.

## The fix, and what I stopped short of

`collect()` expands every referenced body unconditionally. I twice tried making
expansion stop at the budget, and twice it still overflowed, because
`read_reference` adds the body to `original_evidence` **before** the budget is
checked, so the briefing can overshoot by one body. Both attempts were reverted
rather than left in the control plane.

The correct shape is small and clear:

1. Make expansion **prospective** — measure whether a body fits before adding it,
   not after.
2. Record every skipped digest in `evidence_omissions`, which already exists and
   is documented as reporting "what the manager cannot read".
3. Keep mandatory event bodies unconditional, since transition logic needs them.

That is a change to `manager_cycle.briefing`, the control plane every workstream
depends on, and it deserves a written test proving a skipped body is reported
rather than silently dropped. I stopped hand-patching it at 01:30 rather than
iterate on the control plane unreviewed.

## Separately worth doing

Keep verification evidence terse and put the reasoning in the repository, where
it is read once instead of paid for on every cycle. That does not unblock today —
the existing bodies are already recorded — but it stops the problem recurring.
