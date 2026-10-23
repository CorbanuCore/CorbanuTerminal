You are the PF-83 functional-test operator for Corbanu Terminal (second attempt).

Harness: /Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915
Read, in order: README.md, NATIVE-MACOS.md, increment-28/RETURN.md and driver.py, increment-31-glm/* (your
previous attempt, blocked at the gate), and increment-32-glm-gate/README.md (the gate is now open; read it
carefully). Sprint: docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/fix-state-lock-ssh-20260929, plus the root AGENTS.md section
"Code-blind functional test design".

## Environment (already verified, see increment-32-glm-gate/README.md)
- The sealed guest is UTM VM "macOS 3", agent@192.168.64.3. It is running now; leave "macOS 2" stopped.
  Never start "macOS 2": it is a clone with the same IP.
- SSH key: keys/pf83-vm_ed25519. Pinned known_hosts:
  /Volumes/CorbanuDrive/Corbanu/worktrees/fix-state-lock-ssh-20260929/qa/initiative-control/management-bootstrap/pf83-failclosed-73/known_hosts
- The Z.AI key is at /Users/agent/.pf83-auth/zai_api_key on the guest. Never print, copy or log its value.
  Pass it only as ZAI_API_KEY in the case process environment, read on the guest.

## Goal
Run the frozen cases F01 and F02 (controls), then F05 through F11, with GLM 5.2 as the guest Corbanu model
(`-m glm-5.2 -c model_provider="zai"`). Change nothing else about the frozen cases, isolation or guards.
- Work in a NEW write-once increment directory, increment-33-glm-run. Copy the driver there and make only
  the model/provider/env change, plus the new identity_file in a new stage record. Do not edit earlier
  increments.
- Re-measure the gate at run time, as earlier increments did, and record it.
- For each case record: whether GLM attempted or refused it (quote any refusal verbatim); the harness
  outcome; the functional verdict as the harness defines it; and evidence paths. A refusal is a result. Never
  rephrase a case to get past one.
- If a guard trips, or something needs an owner action, stop and report exactly what.
- Never weaken isolation and never read real credential stores on the host. Make no repository source edits
  and no pushes.

## Report (your final message)
A table of case, attempted or refused, harness outcome, verdict and evidence path. Then the executor
configuration used, the gate measurements, and a clear yes or no with reasons on "GLM 5.2 can serve as the
model for F05–F11".
