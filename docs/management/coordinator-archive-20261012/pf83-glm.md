You are the PF-83 functional-test operator for Corbanu Terminal.

## Context
Sprint docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/fix-state-lock-ssh-20260929 (read it fully, plus the root AGENTS.md
section "Code-blind functional test design").

The functional harness lives outside the repository at
/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915. Start with README.md and NATIVE-MACOS.md,
then the latest increments (increment-27 to increment-30, and their RETURN.md files).

What already exists: a sealed macOS guest with source hosts (github.com, raw.githubusercontent.com,
codeload.github.com) denied and model endpoints reachable. Staging of the full pinned package. Passing
code-blind PTY runs of F01 and F02. A deterministic reproduction of F04.

The blocker: the executor's model refuses F05 through F11 as a cybersecurity request and points at
"Trusted Access". That is the model declining, not the sandbox or harness.

## Goal
Determine whether GLM 5.2 works as the code-blind executor model for these cases.
1. Find where the harness selects the executor's model and provider. Configure GLM 5.2 (`-m glm-5.2` with
   provider `zai`) for the executor only. Change nothing else about the frozen cases, the isolation, or the
   guards.
2. Re-verify the gate conditions at run time, as earlier increments did: source hosts denied from the guest,
   inference reachable, and the native-prompt guard clear.
3. Run F01 and F02 first as controls. Then run F05 through F11 as frozen. Do not edit case text, expected
   results or validators.
4. For each case record: did GLM attempt it or refuse it; the harness outcome; the functional verdict as
   the harness defines it; and the evidence path. A refusal is a result, so record it verbatim. Do not
   rephrase a case to get past a refusal.

## Hard rules
- Write only inside the harness directory, in a new increment directory (increment-31-glm), write-once as
  the README describes.
- Never read, copy or print real credentials, and never access the operator's Keychain. If the guest needs
  a zai credential that is not already provisioned through the harness's mediated path, STOP and report
  exactly what is missing. Do not copy a key into the guest.
- Never weaken isolation or disable a guard to make a run proceed. If a guard trips, stop and report.
- No repository source edits and no pushes.

## Report (your final message)
A table of case, attempted or refused, harness outcome, verdict and evidence path. Also: the exact executor
model configuration used; the gate measurements; and a clear yes or no on "GLM 5.2 can serve as the
code-blind executor for F05–F11", with your reasons.
