---
sprint_id: "PF-27-S08"
title: "Windows broker confined by its own restricted token or AppContainer"
status: in_progress
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 44
owner: "broker lane worker (2026-10-08)"
parallel_lane: "broker"
write_scope: "codex-rs/process-hardening/, codex-rs/network-proxy/src/credential_broker/isolated/, codex-rs/network-proxy/src/credential_broker/isolated_tests.rs, .github/workflows/windows-security-probes.yml, qa/security-levels/sprints/PF-27-S08/, qa/demos/index/PF-27-S08.md, docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md"
integration_gate: "Per-sprint gate (sec-common decision 5), one PR per slice: pf_27_s08 probes on windows-2022 (windows-security-probes, elevated and medium integrity), Windows clippy and Linux clippy (RTX box) -D warnings, one Opus 5.5 High review per slice; merged behind the existing default-off flags. GLM 5.2 tmux run, Credential Manager probe and SOP videos on the real Windows machine."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-27-s08-20261008"
branch: "sec/pf-27-s08-broker-token"
base_commit: "df44211c88d285367f62cff59a0df8203865e3d6"
depends_on: "PF-27-S07"
merged_behind_flag: "isolated_credential_broker, secretless_agent_launch (default off)"
gate_evidence: "qa/security-levels/sprints/PF-27-S08/README.md"
created: 2026-10-08
updated: 2026-10-08
---

# PF-27-S08 — Windows broker confined by its own restricted token or AppContainer

Third of the four PF-27-S06 limits Travis approved fixing (2026-10-08). On macOS and Linux the broker confines
its own file writes (Seatbelt, Landlock); on Windows it only has the process DACL and a no-child-process job, so it
can still write the user's files, open the user's other processes, and ask another process to run something
(WMI, Task Scheduler, out-of-process COM). Allocated 2026-10-08 to the broker lane.

## Execution mandate

- Deliver: the Windows broker runs under a token that cannot write outside its own runtime state, open other
  processes of the user, or start work through WMI, Task Scheduler or COM, while it still serves its pipes,
  reaches upstreams and gets stored keys.
- Excludes: Windows model auth (PF-27-S09), macOS/Linux, Permissive.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/active/p1-security-hardening.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: the process that holds raw credentials is itself confined on every platform.

## Code boundaries

- `process-hardening/src/windows_protected_spawn.rs` (the token the broker starts with) and
  `broker_containment.rs` (containment report `token+dacl+job`).
- `network-proxy/src/credential_broker/isolated/pipe.rs` and `server.rs`: pipe DACLs must admit the broker's token
  (AppContainer or restricting SID) and still only Core.
- Stored keys: `arg0` stored-key reader, `secrets`/`keyring-store` on Windows (the vault key is in Credential
  Manager, which an AppContainer or restricted token cannot read).
- Reuse: `windows-sandbox-rs/src/token.rs` (restricted tokens, capability SIDs).

## Preconditions

- [ ] PF-27-S07 merged.
- [ ] **Needs a real Windows machine** for the Credential Manager path and for the tmux run and videos (the
  `windows-2022` runner has no interactive logon, so its Credential Manager behaviour is not representative).
  Token, pipe, file and process probes run on the runner.
- [ ] Product decision: how the vault key reaches a broker that cannot read Credential Manager. Options:
  (a) a one-shot helper under the user's token reads it and writes it into the broker's control pipe, then exits
  (recommended: Core never holds it); (b) Core reads it and sends it (Core already opens the vault for other
  features, PF-27-S05 limits); (c) grant the broker's token Credential Manager access (not possible for an
  AppContainer).

## Acceptance criteria

1. The broker starts under the chosen token (spike: AppContainer with `internetClient` and
   `privateNetworkClientServer`, against a write-restricted token with a broker capability SID; pick the stronger
   one that passes 2–5) and reports it in its containment string; Core refuses a broker without it.
2. The broker cannot create or modify a file the user can write outside its runtime state (positive control: the
   PF-27-S07 broker can).
3. The broker cannot open another process of the user for any access beyond query-limited (control: it can today).
4. The broker cannot start a process through `Win32_Process.Create`, `Schedule.Service` or an out-of-process COM
   server (control: an unconfined process can).
5. The PF-27-S04/PF-28-S02/PF-33-S02 broker suite passes over the pipes; upstream TLS and DNS work; stored keys
   are available through the decided key path, and the broker's token reading Credential Manager is denied.

## Test plan

- `pf_27_s08` probes in process-hardening and network-proxy on `windows-2022`, each with a positive control.
- Credential Manager path, GLM 5.2 tmux run and SOP videos on the Windows machine.
- Linux clippy on the RTX box; Opus 5.5 High review per slice.

## Decisions

- Token: a write-restricted, low-integrity restricted token (capability SID, logon SID, Everyone; no privileges;
  Administrators and INTERACTIVE deny-only), not an AppContainer: it passes 2–5 as measured, and an AppContainer
  refuses loopback upstreams, needs a registered profile and new ACL grants on the user's profile and vault.
- Launch: the PF-27-S07 holder start, now under the broker token. A direct start (Core as parent, which would remove
  the parent-process-spoofing signal) was measured not to work as built; see the evidence.
- Key path (precondition): **open, for Travis.** The broker token can read Credential Manager (measured), so option
  (c) is what ships; recommendation and alternatives in the evidence.

## Done

- [x] Planned (2026-10-08).
- [x] 1: broker token; containment `token+dacl+job`; Core refuses a broker without `token` (PR #333).
- [x] 2: writes outside its runtime state denied in `%TEMP%` and `LocalLow` with positive controls (limit: delete in
  low-integrity folders).
- [x] 3: an ordinary process of the user and its threads refuse every tested right beyond query-limited.
- [x] 4: WMI denied, Task Scheduler refused, out-of-process COM (`MMC20.Application`) denied, with controls.
- [x] 5 (part): broker suite over pipes, DNS and TCP under the token, elevated and normal session.

## Remaining

- [ ] 5 (part): key-path decision (Credential Manager is readable by the broker token).
- [ ] Real-Windows GLM 5.2 tmux run and SOP videos.

## Verification

- [ ] Probes and suite on `windows-2022` and the real Windows machine (elevated and normal session), Credential
  Manager measured on both: done 2026-10-09; checked when the gate run passes.

## Exit evidence

- [ ] Outputs under `qa/security-levels/sprints/PF-27-S08/` (probes recorded; gate run and videos pending).
