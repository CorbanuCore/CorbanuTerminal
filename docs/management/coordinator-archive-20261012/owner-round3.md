You are the operations worker who finishes turning the Corbanu management loop on (round 3).

Read first: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/owner-worker-test.result.md (round 2) and
owner-enable.result.md (round 1), the record qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md
in /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911, and the owner-*.md procedure records it
cites (especially owner-limited-113b-return.md for the isolated worker test).

## Travis's decisions (2026-10-03); add them to the record
- B1: the manager reads its Claude login from the vault, not from a file. Implement it that way.
- B2: reuse the PF-83 test VM ("macOS 3", agent@192.168.64.3, key
  /Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/keys/pf83-vm_ed25519, pinned known_hosts
  /Volumes/CorbanuDrive/Corbanu/worktrees/fix-state-lock-ssh-20260929/qa/initiative-control/management-bootstrap/pf83-failclosed-73/known_hosts).
- B3: "These VMs are expensive and annoying to create. Just programmatically erase data as needed to support fresh
  tests."

## What that allows on the VM
- Delete old PF-83 run folders under /Users/agent (pf83-cases-*) to free space. Their evidence is already sealed on
  the host under .codex-work/functional-pf83.20260915/increment-*. Keep the most recent pinned package
  directory (~/pf83-cases-29.Y0k1xE/package) unless the test needs it gone. Record what you deleted and the space
  freed.
- For the worker test to count as isolated, the test account must not be able to read real credentials.
  Before the test, move /Users/agent/.pf83-auth (OpenAI auth.json and the Z.AI key) out of the agent account's
  reach. After the test, restore it exactly: same files, owner, modes and digests, so PF-83 reruns still work.
  The guest admin login is in the vault as `neo-vm` (account neo2). Use it only at use time:
  `PASS="$(corbanu vault auth-helper neo-vm)"` piped to `sudo -S` on the guest over SSH as neo2. If neo2 has no key
  access, use the agent account plus the existing documented admin path. Never print it or store it in a file.
- Do not change the guest firewall. If the test needs an egress change, stop and report.
- Keep "macOS 2" stopped (it is a clone with the same IP).

## Do
1. **B1:** change the manager's Claude-login path so it reads from the vault at use time (auth-helper pattern), in
   memory only. Add tests, then commit and push fast-forward to integrate/management-workstreams-20260911. Never
   stage the four uncommitted files in scripts/initiative_control/ that belong to another session.
2. **Preflight item 2:** let the manager prepare a fresh action for `owner-first-check-01` (re-register with
   `replace: true` as your round-2 record describes).
3. **Preflight item 4:** free space on the VM, isolate the credentials, run the real isolated worker test
   (ACK/START/RETURN) on the staged binary and package, then restore the credentials. Get an independent review
   with `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan"`, giving it only the
   evidence and the procedure.
   - Mind the OpenAI refresh risk you recorded: start right after the host client refreshes `~/.codex/auth.json`.
4. **Preflight item 5:** the quiet check. The coordinator's hand workers use
   /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/tmux.sock; ignore the `tasknode2` session and your
   own. Then rerun the preflight.
5. If it passes, run the promotion recipe (generation 3, tmux-workers, `manager_enabled: true`) and verify from
   state: ticks, no holds, the owner picking up and returning the prepared action. Republish the dashboard.
6. Extend the record, then commit and push fast-forward.

## Never
Never use unscoped `tmux kill-server`. Never print credentials. Never edit product Rust source. Never force-push.
If a guard refuses and you can't satisfy it honestly, leave the owner safe (armed fixture-only, or disarmed),
restore the VM credentials, and report.

## Report (your final message)
Per preflight item: the evidence; then the VM cleanup and restore proof (digests), the review verdict,
generation and state proof, commits, and anything still open.
