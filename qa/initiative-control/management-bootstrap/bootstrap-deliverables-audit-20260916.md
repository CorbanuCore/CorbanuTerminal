# Auditing the five bootstrap deliverables against code and live state

**Fable, 2026-09-16.** The sprint item says "complete all five bootstrap
deliverables and gates" and carries my own warning that stale-closed and
genuinely-open look identical from the ledger. DEC021 turned out to be stale by
weeks. So I checked all five against the code and against the live coordinator
store rather than against the record that describes them.

The five are the numbered items under **Full completion evidence** in
`docs/research/tasknode-integration/coordinator-bootstrap-20260913.md`. The
allocations earlier in that file are the units that build toward them, not the
deliverables themselves; I checked those too, below.

## Verdict

| # | Deliverable | Verdict |
| --- | --- | --- |
| 1 | Fresh Fable: independent runs, model/effort/session IDs, structured decisions, bounded cancel | **satisfied, live** |
| 2 | Coordination: event → manager → action → native dispatch → ACK → event, dedup, stale denial, watchdog | **satisfied, live** |
| 3 | Integration: exclusive writer, real merge/receiving receipt, successor promotion | **satisfied, live** |
| 4 | Slack: authenticated question, human reply, canonical decision, ACK, restart/recovery | **partly open** |
| 5 | Initialization/rehearsal, then qualified recurring enablement and a final completion ping | **open** |

Three of five are genuinely met and were met by work that has already happened.
Two are genuinely open, and neither is open for a reason I can close alone.

## What the live store proves, rather than what the ledger claims

The audit table in the production coordinator is the part I trust, because
nothing writes to it except the operations themselves:

```
begin_manager 187   manager_decision 120   manager_batch_selected 104
dispatched 134      acknowledged 135       native_receipt 28
verified 130        manager_failed 67      watchdog 2
owner_register_sprint 2  owner_complete 1  owner_archive 1  owner_successor 1
wait_recorded 4
```

That is deliverables 1, 2 and 3 exercised against real state, not fixtures.
`owner_complete`, `owner_archive` and `owner_successor` each appear exactly once,
which is right: they were used once, today, to take PF-60-S02 to completed and
archived and to stand PF-60-S03 up behind it.

The four bootstrap allocations that read as if they might be stale-open are all
present in code and covered by tests that name the behaviour, not the function:
`fit_briefing`'s FIFO prefix walk with `manager_batch_selected` in the audit,
lossless action-input indexing in `manager_cycle.py` with three tests that pin
reconstruction and digest mismatch, the owner-only lifecycle transitions in
`coordinator.py` and `coordinator_cli.py`, and `record_wait` with six tests plus
four real `wait_recorded` rows. The disposable lifecycle rehearsal has its own
receipt. None of these is the blocker.

## Deliverable 4: what is and is not broken

The question-and-answer path works, and there is plenty of evidence for it —
fifteen decisions raised, answered by Travis, and recorded canonically. What does
not work is the **receipted** alert path. `Transport.gate` refuses to send unless
`last_verified` is within 900 seconds; ours expired at `2026-09-15T12:05:48Z` and
is still expired, and the dashboard says so honestly:
`decision_feed.slack.state = stale`.

I cannot re-qualify it myself and should not try. `Transport.qualify` takes
owner UI evidence, and its docstring says in terms that API results are not
acceptable evidence. Calling the API and presenting the result would defeat the
control rather than satisfy it. There is a second condition: the fence records 45
marks against a journal ingress of 42, from the listener crash on 2026-09-15, so
`qualify` additionally requires a gap review. Three inbound callbacks were fenced
and never recorded. They are not recoverable and must not be invented.

This has been true since 2026-09-15 and I recorded it in a repository document
and never raised it as a dashboard decision. That is the same mistake I made with
the S02 archival divergence. It is now a decision.

## Deliverable 5: approved is not the same as enabled

Travis approved unattended recurrence on 2026-09-16T00:03:07Z. Approval is
recorded; **enablement is not**. There is no `com.corbanu.initiative-owner` job
in `launchctl`, so the recurring owner is not running as a service. What is
actually providing recurrence is me, polling on a cycle. That is a real loop and
it is doing the work, but it is not the qualified recurring enablement this
deliverable names, and I would rather say so than tick it.

The final Slack completion ping is downstream of the sprint actually completing,
so it is correctly still open.

## What I did not do

I did not tick the sprint item. Three of five being satisfied is not the item.
I also did not re-run any of the three satisfied deliverables' gates as part of
this audit — this was a state-and-code audit, and the gate audit is a separate
record from the same day.
