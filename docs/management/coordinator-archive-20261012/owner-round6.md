You are the operations worker for the Corbanu management loop (round 6).

Read first: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/owner-round5.result.md, owner-round5.md
(same directory; its Do/Never rules still apply), the round-5 evidence under
/Volumes/CorbanuDrive/Corbanu/.codex-work/owner-manager-enable-20261002/round5/, and the record
qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911.

## State
- Owner armed at generation 5 (tmux-workers, `manager_enabled: true`), no holds. `owner-first-check-01-r3` is
  `returned` but not yet accepted/rejected by the manager.
- Travis wants this to be an automatic loop: work should flow manager → owner → worker → back without hand
  steps or manual hold clearing.

## Do
1. **Boot-time drift.** A shift in the recorded boot time (62 ms seen in round 5) makes running workers hold
   `host_changed`, and the normal close refuses them. Make host identity tolerant of sub-second boot-time jitter
   (compare at a coarse granularity, or use a stable identifier such as the kernel boot session UUID), while a
   real reboot still holds. Unit-test: jitter accepted, real reboot refused, close works after jitter.
2. **Manager JSON fences.** 1 of 2 manager cycles failed `invalid_json` because the JSON was wrapped in a
   Markdown code fence. Accept exactly one fenced block (```json or ```) whose content is the whole JSON object,
   nothing else around it; everything else stays strict. Unit-test both.
3. **Close after RETURN.** The owner leaves idle workers running after RETURN. After the result is recorded in
   the coordinator, close the worker cleanly through its own socket only, with an audit record. Test it.
4. **Routing.** New actions default to `hand`, so each needs an explicit handoff. Find where that default is set
   and the documented reason. If no safety reason is recorded, make actions the manager prepares inside the
   armed scope route to the owner by default; keep `hand` for anything outside scope. If a recorded safety
   reason exists, don't change it; report it.
5. **r3 verification.** Let the manager accept or reject `owner-first-check-01-r3` through the normal path.
6. Run the full suite in a clean worktree. Get an independent code review with `~/.local/bin/corbanu exec -m
   claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high"` and fix what it finds.
7. Commit and push fast-forward to integrate/management-workstreams-20260911. Never stage the four uncommitted
   files in scripts/initiative_control/.
8. Requalify on the PF-83 VM exactly as round 5 (same fence, hosts, credentials, share handling, exact restore,
   digests), fresh independent review, preflight, promote to the next generation. Reuse round 5's adapted
   restage/preflight variants where the recipe assumes fixture-only.
9. **Prove it end to end, hands-off:** the manager prepares two small, safe, real actions (read-only or
   docs-only in disposable worktrees). Both must go manager → owner → ACK/START/RETURN → manager verdict → worker
   closed, with no manual handoff and no holds.
10. Extend the record, commit and push fast-forward. Dashboard: same rule as round 5.

## Never
Never print credentials. Never edit product Rust source. Never force-push. `rm -f` is blocked. Leave the VM
restored exactly. If a guard refuses and you can't satisfy it honestly, leave the owner in a safe state and report.

## Report (your final message)
Each fix with tests, review verdicts, qualification result, new generation, the two hands-off action receipts,
commits, and anything still open.
