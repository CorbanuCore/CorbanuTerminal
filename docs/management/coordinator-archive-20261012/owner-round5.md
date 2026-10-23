You are the operations worker for the Corbanu management loop (round 5).

Read first: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/owner-round4.result.md, the round-4 evidence
under /Volumes/CorbanuDrive/Corbanu/.codex-work/owner-manager-enable-20261002/round4/, and the record
qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911.

## State
- The owner is armed at generation 3, scope tmux-workers, `manager_enabled: true`, and ticking.
- Its first real action, `owner-first-check-01-r2`, is HELD with `wrong_ack`. The model's ACK was correct but came
  as a non-final message with a trailing newline, and the owner's exact-match check refused it. This happened on
  2 of 7 real first replies today.
- The held worker `w-ylg59fqs` is idle on the host. There is no command to clear a hold.

Travis wants the loop on and working. Make the loop actually process work.

## Do
1. **Diagnose the hold path.** Read owner_daemon.py (`hold`, `--recover`, `scheduled_tick(root, recover)`, the
   holds table's resolution columns) and owner_tmux.py (`provenance`, the ACK check), plus the owner-*.md records
   on recovering holds. Find the supported, evidence-backed way to resolve a hold. If none exists, implement one
   as a narrow, reviewed `--recover`/resolve path that records `resolution_evidence_digest` and `resolved_at`.
   Never just delete rows.
2. **Fix the ACK check narrowly.** Accept the exact ACK token line with surrounding whitespace and newlines, and
   accept it from a completed assistant message that is not the final one, only if that is the turn's correlated
   ACK. Keep everything else strict: a wrong action id, allocation digest, model or effort must still be refused.
   Add unit tests for both the newline case and the refusal cases. This changes the package fingerprint.
3. Commit and push fast-forward to integrate/management-workstreams-20260911. Never stage the four uncommitted
   files in scripts/initiative_control/ that belong to another session.
4. **Restage, requalify, promote.** Follow the documented recipe:
   - Stop the held worker cleanly. No unscoped `tmux kill-server`; act only on its own recorded socket or session.
   - Resolve the hold with evidence.
   - Restage the runtime with the new package.
   - Rerun the isolated qualification on the PF-83 VM exactly as in round 4 run 3: the same firewall, hosts,
     credential and shared-folder handling, with an exact restore and digests.
   - Get a fresh independent review with `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c
     model_provider="claude-plan"`, giving it only the evidence and the procedure.
   - Run the preflight, then promote to the next generation.
5. **Prove it works:** have the manager prepare one small, real, safe action (read-only or docs-only in a
   disposable worktree). Show the owner picking it up and taking it through ACK, START and RETURN on the live
   loop, with no holds, and the result returned to the coordinator.
6. Republish the dashboard only if `sync-source.sh` can publish without those four uncommitted files. If it can't,
   skip it and say so.
7. Extend the record, then commit and push fast-forward.

## Never
Never print credentials. Never edit product Rust source. Never force-push. Leave the VM restored exactly after
any qualification run. If a guard refuses and you can't satisfy it honestly, leave the owner in a safe state and
report.

## Report (your final message)
The hold resolution evidence, the ACK fix with its tests, the qualification and review verdict, the new
generation, the live action receipts (ACK/START/RETURN), commits, and anything still open.
