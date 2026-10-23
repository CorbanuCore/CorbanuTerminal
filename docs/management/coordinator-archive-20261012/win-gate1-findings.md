# PF-27-S06 real-Windows gate: defects found (win-gate1, 2026-10-08)

Host: Windows 11 Pro 26200 (desktop-ujs3hc4), build of origin/main 63ea3d0cbd0c (debug), GLM 5.2 via zai.
Both defects were missed by CI because the `windows-2022` runners run elevated and the CI probe uses a
restricted-read base profile instead of the product's `workspace-write` profile.

## 1. Normal (unelevated) session: every agent command fails with the contract on
- Config: `[windows] sandbox = "elevated"`, `secretless_agent_launch = true`, launched from the user's
  interactive shell (medium integrity). Every command fails: `windows sandbox: CreateProcessWithLogonW failed: 5`.
- Same session and profile with `secretless_agent_launch = false` (broker on or off): commands run as
  `CodexSandboxOffline`. Same profile from an elevated session: works.
- Security log shows no logon attempt for the sandbox user, so seclogon refuses before logging on. Likely cause:
  `restrict_current_process_access()` (armed by the contract, launch_contract.rs:664) leaves the user only
  query-limited + synchronize on Core's process; seclogon impersonates the caller and opens its process to set
  up the child, which the hardened DACL now denies. PF-27-S07 slices 2a/2b (#282, #284) extend that hardening,
  so please re-test them in a medium-integrity session.

## 2. The vault store is readable under the contract (product `workspace-write` profile)
- Same elevated sandbox, contract on, elevated session: an agent command reads `CODEX_HOME\secrets\local.age`
  (pre-existing or created during the run). `auth.json`, `config.toml` and `*state*.sqlite` are denied, but only
  by the files-only deny on `CODEX_HOME` itself (it does not reach files in subdirectories).
- The contract's deny entries are never applied as ACLs: `.sandbox\deny_read_acl_state.json` stays
  `{"principals": {}}`, and the sandbox log shows only the `.git` deny. The contract's `verify_permissions`
  accepts the profile because it contains the deny entries, but the Windows backend does not enforce them
  under a full-read profile.

Evidence and videos: qa/security-levels/sprints/PF-27-S06/README.md (gate section) and qa/demos/index/PF-27-S06.md.
