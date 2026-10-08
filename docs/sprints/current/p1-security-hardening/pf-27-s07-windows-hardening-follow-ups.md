---
sprint_id: "PF-27-S07"
title: "Windows hardening follow-ups"
status: draft
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 43
owner: "broker lane worker (2026-10-08)"
parallel_lane: "broker"
write_scope: "codex-rs/process-hardening/, codex-rs/network-proxy/src/credential_broker/isolated/client.rs, codex-rs/windows-sandbox-rs/src/acl.rs, codex-rs/windows-sandbox-rs/src/acl_tests.rs, codex-rs/windows-sandbox-rs/src/lib.rs, codex-rs/core/src/security/launch_contract.rs, codex-rs/core/src/security/launch_contract_windows_tests.rs, codex-rs/core/src/config/mod.rs, .github/workflows/windows-security-probes.yml, qa/security-levels/sprints/PF-27-S07/, docs/sprints/current/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md"
integration_gate: "Per-sprint gate (sec-common decision 5), one PR per slice: pf_27_s07 tests on the windows-2022 runner (windows-security-probes workflow), Linux clippy on the RTX box, one Opus 5.5 High review per slice; merged behind the existing default-off flags. The GLM 5.2 tmux run and SOP videos need a real Windows machine (as for PF-27-S06)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-27-s07-20261008"
branch: "sec/pf-27-s07-win-hardening"
base_commit: "ad96c55cb519e79d08da40c6513ead4c847b0866"
depends_on: "PF-27-S06"
created: 2026-10-08
updated: 2026-10-08
---

# PF-27-S07 — Windows hardening follow-ups

Travis approved fixing the four limits PF-27-S06 documented (2026-10-08). This sprint fixes the two that need no
real Windows machine; the other two are [PF-27-S08](pf-27-s08-windows-broker-restricted-token.md) (the broker's
own token) and [PF-27-S09](pf-27-s09-windows-model-client-auth.md) (Windows model auth). Status stays `draft`
until the P1 plan is activated, as for PF-27-S06. One PR per slice, from branches named after this one.

## Execution mandate

- Deliver: Core and broker threads that no same-user process can open, even right after they are created; the
  `CODEX_HOME` deny removed exactly when `secretless_agent_launch` is off.
- Excludes: the broker's own token (PF-27-S08), Windows model auth (PF-27-S09), macOS/Linux changes, Permissive.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/active/p1-security-hardening.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: no raw managed secret enters agent environment, command line or process memory on Windows.

## Code boundaries

- New threads: `process-hardening/src/windows_thread_creation.rs` (`CreateThread` import redirect, token default
  DACL), `windows_protected_spawn.rs` (`spawn_protected`), `windows_process_access.rs`, `broker_containment.rs`;
  the broker's Windows start in `network-proxy/src/credential_broker/isolated/client.rs`.
- `CODEX_HOME` deny: `windows-sandbox-rs/src/acl.rs` (`remove_deny_read_ace_for_new_files`),
  `core/src/security/launch_contract.rs` (armed lock, `release_codex_home_when_unarmed`), `core/src/config/mod.rs`.
- Tests: `pf_27_s07` in process-hardening, windows-sandbox and core, run by `windows-security-probes`.

## Preconditions

- [x] PF-27-S06 slices merged (#267, #269, #270, #272, #277); `windows-2022` runner for real probes.

## Acceptance criteria

1. Every thread Core or the broker creates through `CreateThread` (every Rust thread) has the protected DACL when
   it is created. Measured: a same-user process without privileges, and one under the unelevated sandbox's token,
   cannot open a brand-new thread held in its creation window (created suspended, so its TLS callback has not
   run). Positive control: a thread created without that import can be opened in the same window.
2. The broker is never openable: it starts with protected process and first-thread DACLs, suspended, and its
   token's default DACL is the protected thread DACL before it runs, so every thread it creates, however it is
   started, is protected at creation. Measured with the same probe on both kinds of thread.
3. Threads keep the rights they need on themselves (priority, impersonating themselves).
4. With `secretless_agent_launch` off, the first config load of a process without an armed contract removes exactly
   the `CODEX_HOME` new-file deny (and the copies files in `CODEX_HOME` inherited); every other entry and the
   DACL's protection are unchanged (compared as SDDL). Never while another process has the contract armed.

## Done

- [x] Record created with the follow-up plan records PF-27-S08 and PF-27-S09.

## Remaining

- [ ] Slice 1: `CODEX_HOME` deny removed on flag-off; armed processes hold a shared lock file in `CODEX_HOME`.
- [ ] Slice 2: new threads protected from creation (import redirect in Core and broker; default DACL and
  `spawn_protected` for the broker).
- [ ] Evidence and record.

- [ ] Decision 5 tmux run and SOP videos on a real Windows machine (same requirements as PF-27-S06).
- [ ] Threads Windows starts inside Core itself (thread-pool workers) still get the protected DACL only in their
  TLS callback, and loader workers not at all; Core cannot use the default DACL (its child pipes and child
  processes inherit it). Recorded as a limit.

## Verification

- [ ] `pf_27_s07` on `windows-2022`; PF-27-S06 suites still pass; Linux clippy clean on the RTX box.
- [ ] Opus 5.5 High review per slice.

## Exit evidence

- [ ] Outputs under `qa/security-levels/sprints/PF-27-S07/`.
- [ ] Record archived (after the Windows-host gate items).
