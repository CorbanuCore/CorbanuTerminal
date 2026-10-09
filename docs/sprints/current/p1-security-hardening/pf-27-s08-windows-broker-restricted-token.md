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
can still write the user's files, open the user's other processes, or ask another process to run something
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

- `process-hardening/src/windows_protected_spawn.rs` (broker token), `broker_containment.rs` (`token+dacl+job`).
- `network-proxy/src/credential_broker/isolated/pipe.rs` and `server.rs`: pipe DACLs must admit the broker's token
  (AppContainer or restricting SID) and still only Core.
- Stored keys: `arg0` stored-key reader, `secrets`/`keyring-store` on Windows (vault key in Credential Manager).

## Preconditions

- [x] PF-27-S07 merged; a real Windows machine (used 2026-10-09).
- [ ] Product decision: how the vault key reaches the broker: (a) one-shot helper, (b) Core sends it,
  (c) broker reads Credential Manager itself. Measured: (c) works; (a)/(b) need a stronger token.

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

- Token: write-restricted, low-integrity restricted token, not an AppContainer; launch: the PF-27-S07 holder start.
  Reasons, measurements and limits: the evidence README.
- Key path: **open, for Travis.** The broker token can read Credential Manager (measured), so (c) is what ships.

## Done

- [x] Planned (2026-10-08).
- [x] Criteria 1–4 and 5 except the key path, with positive controls, on `windows-2022` and the real machine (PR #333).
- [x] Real-Windows GLM 5.2 tmux run and SOP videos (2026-10-09).

## Remaining

- [ ] 5 (part): key-path decision (Credential Manager is readable by the broker token).

## Verification

- [x] Probes and suite on `windows-2022` and the real Windows machine (elevated and normal session), Credential
  Manager measured on both: done 2026-10-09.
- [ ] Independent review verdict: REQUEST_CHANGES (2 blocking: B1, B2), then APPROVE after fixes.

## Exit evidence

- [x] Outputs under `qa/security-levels/sprints/PF-27-S08/` (probes, gate run and videos).
- [ ] Key-path decision for Travis: how the vault key reaches the broker (Credential Manager is readable by the broker token).
