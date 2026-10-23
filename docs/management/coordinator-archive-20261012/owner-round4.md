You are the operations worker who finishes turning the Corbanu management loop on (round 4).

Read first: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/owner-round3.result.md, the evidence under
/Volumes/CorbanuDrive/Corbanu/.codex-work/owner-manager-enable-20261002/round3/ (including the reviewer's file and
its corrections), and the record qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911.

## Travis's decisions (2026-10-03); add them to the record verbatim
1. "Yes": block the VM's direct OpenAI access during the isolated worker test. Temporarily remove the OpenAI
   addresses from the PF-83 guest's pinned egress (/etc/pf.anchors/corbanu.pf83 `inference` table and the matching
   /etc/hosts entries) using the `neo-vm` admin login from the vault at use time. Run the test, then restore both
   files byte-for-byte and reload pf. Record digests before, during and after. Z.AI entries must also be absent
   during the test if they would let the worker bypass the broker. Prove zero direct connections.
2. "Yes": the manager may keep using `corbanu internal-claude-oauth-token` for its Claude login.

## Do
1. Fix the evidence write-up inaccuracies the round-3 reviewer listed.
2. Back up the guest pf anchor and /etc/hosts, then block direct provider egress, keeping SSH from the host and the
   broker path. Verify with probes that chatgpt.com, api.openai.com and api.z.ai are unreachable directly.
3. Credentials isolation as in round 3: move ~/.pf83-auth and the qual probe auth.json to a root-only holding
   folder, and unmount the host shared folder. Restore everything afterwards.
4. Re-run the broker-down check and one full owner worker run (ACK, START, RETURN) on the current package
   fingerprint, with zero direct connections (capture connection evidence). Use the prepared action
   `owner-first-check-01-r2`, or prepare a fresh one per the procedure if it has gone stale.
5. Get a fresh independent review: `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan"`,
   given only the evidence and the procedure.
6. Restore the guest exactly: the pf anchor, /etc/hosts, credentials, the shared folder mount; reload pf. Prove
   it with digests, and run a probe showing api.z.ai and api.openai.com are reachable again and github.com is still
   denied, so PF-83 reruns still work.
7. If the review qualifies it and the preflight passes: run the promotion recipe (generation 3, scope
   tmux-workers, `manager_enabled: true`) and verify from state (interval ticks, no holds, routing tmux-workers).
   Republish the dashboard only if `sync-source.sh` can publish without the other session's four uncommitted files
   in scripts/initiative_control/. If it can't, skip it and say so.
8. Extend the record, then commit and push fast-forward. Never stage those four uncommitted files.

## Never
Never use unscoped `tmux kill-server`. Never print credentials. Never edit product Rust source. Never force-push.
If anything fails, restore the guest firewall, hosts, credentials and mount first, leave the owner safe, and report.

## Report (your final message)
The firewall change and restore digests, connection evidence, the run receipts, the review verdict, the
promotion result and generation, ticks, commits, and anything still open.
