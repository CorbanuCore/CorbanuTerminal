# Worker brief: PF-83 independent code-blind execution and evidence review

You are an Opus 5.5 High integrator-side worker. Work autonomously; do not ask questions.

## Goal
Close the two remaining PF-83-S01 gates in
`/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md`:
- "Provision independent code-blind executor with enforced filesystem/process/IPC/network boundaries and negative
  controls; execute original cases with explicit approved-scope amendment."
- "Independent evidence check and schema-2 isolation receipts; original F01-F11 mapped, no unaccepted waiver."

## Read first (and follow exactly)
- `qa/code-blind-functional/isolated-execution.md`, `README.md`, `RECORD_TEMPLATE.md`, `executor-prompt.md`,
  `designer-prompt.md` (same worktree) and the root AGENTS.md sections on code-blind functional testing and on the
  model for adversarial functional cases (GLM 5.2 via `-m glm-5.2 -c model_provider="zai"`).
- The frozen F01-F11 cases and prior evidence: `.codex-work/functional-pf83.20260915/` (increments 32–34),
  `qa/initiative-control/management-bootstrap/pf83-*` records.
- PF-83 VM/harness facts: sealed guest VM "macOS 3" at `agent@192.168.64.3` (keep "macOS 2" stopped; same IP), key
  `.codex-work/functional-pf83.20260915/keys/pf83-vm_ed25519`, pinned known_hosts under
  `worktrees/fix-state-lock-ssh-20260929/qa/initiative-control/management-bootstrap/pf83-failclosed-73/known_hosts`,
  admin `neo2` via vault label `neo-vm` (use only through `corbanu vault auth-helper neo-vm` inside a consuming
  command), Z.AI key at `/Users/agent/.pf83-auth/zai_api_key` in the guest, guest firewall allowing api.z.ai only.
  Travis's standing instruction for this VM: programmatically erase data as needed to support fresh tests; don't
  rebuild the VM.

## Do
1. As integrator, provision the isolated executor exactly per `isolated-execution.md`: read-only exact package,
   frozen case packet, private writable state/evidence/PTY/TMUX socket, no source/history/prior findings, network
   denied except mediated inference (Z.AI) and declared test services. Capture the required negative probes
   (repo read/list, history, symlink escape, credential access, cross-run IPC, unapproved network, package
   modification, child escape) before each run/group, with synthetic canaries only.
2. Launch a fresh code-blind executor (GLM 5.2 per policy) per case group with only `executor-prompt.md`, the frozen
   cases and neutral navigation. A new executor context for every corrected replay. Record real session, model,
   machine, profile, socket and launcher identity.
3. Run the original F01-F11 against the exact pinned package the sprint names (verify digests). Record each case's
   actual actions/observations per `RECORD_TEMPLATE.md` (schema 2).
4. Independent review: a separate fresh reviewer (`~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c
   model_provider="claude-plan" -c model_reasoning_effort="high"`), independent of implementer and executor, checks
   actions, observations, provenance, isolation and dispositions. Fix evidence issues; never coach the executor.
5. Record results in the sprint file (update the two checkboxes only if truly met; otherwise record precisely what
   is met and what isn't, with the blocking capability) and receipts under
   `qa/initiative-control/management-bootstrap/`. Commit on a fresh worktree off `origin/integrate/management-workstreams-20260911`
   and push fast-forward (pull first). Never stage the four uncommitted `scripts/initiative_control/` files in the
   main management worktree — don't work in that worktree.
6. Restore the VM to a clean state afterwards and record the restore evidence.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Never modify the product or the frozen cases. Never
bypass a tool denial.

## Final report (your last message)
Per case F01-F11: PASS/FAIL/BLOCKED with evidence path; isolation receipts and negative-probe results; reviewer
verdict; commits; what remains open.
