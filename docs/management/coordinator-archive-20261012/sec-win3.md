You are the Windows defect worker for PF-27-S06. Read sec-common.md (binding; never print secrets; vault label
"drone windows password" only via stdin if Windows asks for a password) and PR #293's evidence
(qa/security-levels/sprints/PF-27-S06/README.md "Real Windows gate run") plus win-gate1's notes in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/. Real Windows 11 machine (toolchain already provisioned
under C:\CorbanuQA by win-gate1):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
The real-Windows gate run found two defects that CI missed (CI runners are elevated and use a restricted-read profile):
 D1: in a normal medium-integrity session with secretless_agent_launch on, every agent command fails with
     `CreateProcessWithLogonW failed: 5`.
 D2 (SECURITY, highest priority): under the product's workspace-write profile, agent commands can READ the vault store
     (CODEX_HOME\secrets\*) because the launch contract's deny entries are never applied as ACLs.
First file two GitHub issues (D2 labelled security), then fix each in its own PR with regression tests that FAIL before and
PASS after on the real Windows machine from a normal (non-elevated) session, plus a CI change so the windows-security-probes
job also covers a non-elevated/medium-integrity path and the workspace-write profile where GitHub runners allow it (document
any gap honestly). Opus 5.5 High review per PR; Linux clippy on the RTX box; merge each when green
(`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts; never force-push).
Worker sec-win2 is concurrently landing PF-27-S07 (PRs #281/#284) in windows-sandbox-rs/process-hardening/launch_contract;
merge origin/main often and avoid editing its open-PR files except via rebase-level conflict resolution.
Finally re-run the failed gate items on the real machine, record a short video of the vault read now being denied under
workspace-write and of agent commands working in a normal session, add to the PF-27-S06 gate record in a docs PR. Report.
