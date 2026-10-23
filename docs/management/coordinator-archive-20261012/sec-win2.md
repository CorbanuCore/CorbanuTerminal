You are the broker-lane worker for the Windows hardening follow-ups to PF-27-S06. Read sec-common.md and sec-win1.result.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first (binding rules; never delete worktrees referenced by
the live owner settings). Travis approved fixing all four documented Windows limits.
A) Implement now, as one sprint (create a sprint record PF-27-S07 "Windows hardening follow-ups" in
   docs/sprints/current/p1-security-hardening/, same format as PF-27-S06), with code+tests+Opus 5.5 High review+Linux
   clippy on the RTX box, Windows CI probes:
   1. New Core/broker threads protected from creation (e.g. PROC_THREAD_ATTRIBUTE / process-level default DACL so no
      window where a same-user unsandboxed process can open a fresh thread); add a probe that tries to open a brand-new
      thread.
   2. Remove exactly the CODEX_HOME deny ACE we added when the flag is off (on flag-off or next start without it),
      leaving other ACEs untouched; test it.
B) Plan (sprint records only, status draft, with acceptance criteria, code boundaries, test plan, and an explicit
   "needs a real Windows machine" precondition where true):
   3. PF-27-S08 "Broker confined by its own restricted token / AppContainer" (incl. Credential Manager access).
   4. PF-27-S09 "Windows model-client auth via the broker" (port of PF-27-S05).
   Put them in the P1 security-hardening plan and its sprint index in execution order after PF-27-S07.
Small reviewable PRs; merge each when green with `gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on
conflicts, never force-push. Report as in sec-common.md.
