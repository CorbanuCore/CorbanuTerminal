---
sprint_id: "PF-27-S08"
title: "Windows broker confined by its own restricted token or AppContainer"
status: draft
plan_file: "docs/plans/proposed/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 44
owner: "broker lane (unassigned)"
parallel_lane: "broker"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-27-S07"
created: 2026-10-08
updated: 2026-10-08
---

# PF-27-S08 — Windows broker confined by its own restricted token or AppContainer

Third of the four PF-27-S06 limits Travis approved fixing (2026-10-08). On macOS and Linux the broker confines
its own file writes (Seatbelt, Landlock); on Windows it only has the process DACL and a no-child-process job, so it
can still write the user's files, open the user's other processes, and ask another process to run something
(WMI, Task Scheduler, out-of-process COM). Plan only: nothing is implemented until this record is allocated.

## Execution mandate

- Deliver: the Windows broker runs under a token that cannot write outside its own runtime state, open other
  processes of the user, or start work through WMI, Task Scheduler or COM, while it still serves its pipes,
  reaches upstreams and gets stored keys.
- Excludes: Windows model auth (PF-27-S09), macOS/Linux, Permissive.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/proposed/p1-security-hardening.md#pf-27).
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

## Done

- [x] Planned (2026-10-08).

## Remaining

- [ ] Everything under Acceptance criteria.

## Verification

- [ ] Probes and suite on `windows-2022`; Credential Manager path on a real Windows machine.

## Exit evidence

- [ ] Outputs under `qa/security-levels/sprints/PF-27-S08/`.
