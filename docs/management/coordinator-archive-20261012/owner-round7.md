You are the operations worker for the Corbanu management loop (round 7).

Read first: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/owner-round6.result.md and owner-round6.md
(same directory; its Do/Never rules still apply), the round-6 evidence under
/Volumes/CorbanuDrive/Corbanu/.codex-work/owner-manager-enable-20261002/round6/, and the record
qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911.

## State
- Owner armed at generation 9, tmux-workers, `manager_enabled: true`, no holds. Owner side is hands-off: it adopts
  manager-prepared actions, runs ACK/START/RETURN, closes workers, and the manager can give verdicts.
- Travis wants a fully automatic loop. Remaining gaps from round 6 are below.

## Do
1. **Automatic manager cycles.** Today someone must start each manager cycle; `manager_enabled` only labels the
   tick. Make the armed owner (or the same launchd schedule) start a manager cycle by itself when there is work
   for it: returned results awaiting a verdict, or pending events, or no prepared work queued. Rate-limit it (one
   in flight at a time, a minimum interval, a daily cap recorded in config), record every cycle in the audit log,
   and fail closed (hold, no retries storm) on manager errors. Respect pause/disarm. Test it, including a crash
   mid-cycle and a restart.
2. **Briefing budget.** The manager briefing sits at its 64 KiB limit with ~85 events waiting. Make it structural:
   a bounded, prioritized briefing (returned results and holds first, then newest events), with older events
   summarized or windowed, and routine automatic compaction of finished allocations using the existing method.
   Nothing may be silently dropped: anything left out must be counted and recoverable on the next cycle. Test it.
3. **Main worktree scope.** Remove the main integration worktree (`worktrees/management-workstreams-20260911`)
   from the owner's auto-adopt worktrees. Auto-adopted actions may only run in disposable worktrees created for
   them; anything aimed at the main worktree stays on the hand lane. Test it.
4. Full suite in a clean worktree (include `manager_cycle_test.py`). Independent code review with
   `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c
   model_reasoning_effort="high"`; fix what it finds. Keep reviews within budget if possible; record any extension.
5. Commit and push fast-forward to integrate/management-workstreams-20260911. Never stage the four uncommitted
   files in scripts/initiative_control/. Pull first; others may push to that branch.
6. Requalify on the PF-83 VM exactly as round 6, fresh independent review, preflight, promote.
7. **Prove it with no human step at all:** after promotion, do not start any manager cycle yourself. Seed one
   small, safe goal through the normal intake (read-only or docs-only, disposable worktree) and show the loop on
   its own: manager cycle starts → prepares action → owner adopts → ACK/START/RETURN → worker closed → manager
   cycle starts → verdict. Then show it idles (no runaway cycles) for at least 15 minutes.
8. Extend the record, commit and push fast-forward. Dashboard: same rule as before.

## Never
Never print credentials. Never edit product Rust source. Never force-push. `rm -f` is blocked. Leave the VM
restored exactly. If a guard refuses and you can't satisfy it honestly, leave the owner in a safe state and report.

## Report (your final message)
Each fix with tests, review verdicts, qualification, new generation, the no-human-step receipts with timestamps,
the idle evidence, commits, and anything still open.
