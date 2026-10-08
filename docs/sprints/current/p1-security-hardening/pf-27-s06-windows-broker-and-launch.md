---
sprint_id: "PF-27-S06"
title: "Windows broker and secretless launch"
status: draft
plan_file: "docs/plans/proposed/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 45
owner: "broker lane worker (2026-10-08)"
parallel_lane: "broker"
write_scope: "codex-rs/process-hardening/, codex-rs/network-proxy/src/credential_broker/isolated/, codex-rs/network-proxy/src/credential_broker/isolated.rs, codex-rs/network-proxy/src/credential_broker/isolated_tests.rs, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/providers.rs, codex-rs/network-proxy/src/upstream.rs, codex-rs/network-proxy/src/lib.rs, codex-rs/network-proxy/Cargo.toml, codex-rs/arg0/src/lib.rs, codex-rs/core/src/security/launch_contract.rs, codex-rs/core/src/security/launch_contract_tests.rs, codex-rs/core/src/security/launch_contract_windows_tests.rs, codex-rs/core/src/security/inspection.rs, codex-rs/core/src/tools/sandboxing.rs, codex-rs/core/src/tools/sandboxing_tests.rs, codex-rs/core/src/tools/runtimes/apply_patch.rs, codex-rs/windows-sandbox-rs/src/acl.rs, codex-rs/windows-sandbox-rs/src/lib.rs, codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs, codex-rs/core/Cargo.toml, codex-rs/network-proxy/src/mitm.rs, codex-rs/network-proxy/src/certs.rs, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/arg0/Cargo.toml, codex-rs/core/tests/suite/windows_sandbox.rs, codex-rs/Cargo.lock, MODULE.bazel.lock, .github/workflows/windows-security-probes.yml, qa/security-levels/sprints/PF-27-S06/, docs/sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md"
integration_gate: "Per-sprint gate (sec-common decision 5), one PR per slice: pf_27_s06 tests on the windows-2022 runner (windows-security-probes workflow), Linux clippy on the RTX box, one Opus 5.5 High review per slice; merged behind the existing default-off flags. The GLM 5.2 tmux run and SOP videos need a real Windows machine."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-27-s06-20261008"
branch: "sec/pf-27-s06-windows-broker"
base_commit: "7e6ef740ef1574d48c3caf2e84af82779c5b5969"
depends_on: "PF-27-S02"
created: 2026-10-06
updated: 2026-10-08
---

# PF-27-S06 — Windows broker and secretless launch

Explicit follow-up from PF-27-S04 (no Windows broker) and PF-27-S02 (macOS and Linux only), per the
coordinator's 2026-10-06 instruction not to block on Windows. Until this lands, Windows with
`isolated_credential_broker` virtualizes but never injects, and with `secretless_agent_launch` it refuses
agent commands with a stated reason. Moved to the P1 hardening plan on 2026-10-06 (Travis); its dependency
on [PF-27-S02](../../archive/p0-security-levels/pf-27-s02-secretless-agent-launch.md) is unchanged.

Started 2026-10-08 (Travis approved starting before the plan is activated). Status stays `draft` until the
plan worker activates the P1 plan and records these coordinates (the lifecycle checker requires both; plan slots
are 3/3). No Windows host yet: real probes run on the `windows-2022` CI runners (`windows-security-probes`
workflow); the tmux run and videos wait for a Windows machine. One PR per slice, from branches named after this one.

## Execution mandate

- Deliver: the isolated broker process and the secretless launch contract on Windows, with measured containment.
- Excludes: macOS/Linux changes, model-client auth (PF-27-S05), Permissive changes.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/proposed/p1-security-hardening.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: no raw managed secret enters agent environment, command line or process memory on Windows.

## Code boundaries

- Existing: `codex-rs/network-proxy/src/credential_broker/isolated/` (Unix only); `codex-rs/core/src/security/launch_contract.rs`;
  `codex-rs/windows-sandbox-rs/`.
- Shipped: `process-hardening/src/windows_process_access.rs`, `network-proxy/src/credential_broker/isolated/pipe.rs`
  (and the Windows `transport` in `server.rs`), `core/src/security/launch_contract_windows_tests.rs`,
  `windows-sandbox-rs/src/acl.rs` (`add_deny_read_ace_for_new_files`).
- Tests: `pf_27_s06` modules, run on Windows CI.

## Preconditions

- [x] PF-27-S02 completed and archived; Windows CI runner for real probes (a Windows host is still needed for the tmux run and videos).

## Done

- [x] Record created as the explicit Windows follow-up.
- [x] Process containment (slice 1, PR #267): Core and the broker replace their process and thread DACLs (user:
  query-limited and synchronize only; OWNER RIGHTS: read-control), new threads included through a TLS callback that
  hardening verifies is linked. Measured on `windows-2022`: a command under the unelevated sandbox's restricted token
  and a same-user process without privileges cannot open them for `PROCESS_VM_READ`, read their environment,
  duplicate their handles, inject, re-ACL, or open any thread for its context; unhardened targets are the positive
  controls.
- [x] Broker transport on Windows (slices 2a/2b, PRs #269/#270): named pipes with random first-instance names, a
  DACL for the user, remote clients refused, no inheritable handles (measured by a handle scan with a positive
  control), the client process id checked before any byte, the server process id checked by Core, an overlapped
  control pipe. The broker runs contained (`dacl+job`: no child processes, no desktop/clipboard/atoms), holds its
  controller's handle and exits with it. The PF-27-S04/S28/S33 broker suite passes over the pipes.
- [x] Launch contract (slice 3, PR #272): the Windows refusal is now a measured pass under the elevated sandbox
  (separate sandbox user) and a stated refusal for the unelevated one. Measured: vault, sign-in, policy store and
  state databases unreadable (readable in the base-profile control), `CODEX_HOME` unwritable, files created or
  replaced during a run denied (an inherit-only, files-only deny on `CODEX_HOME`), Core stand-ins unopenable.
- [x] `pf_27_s06` tests run on every PR touching this code (`windows-security-probes` workflow); Linux clippy clean
  on the RTX box; Opus 5.5 High reviews per slice, all approved (2-3 rounds each).

## Remaining

- [ ] Decision 5 tmux run (GLM 5.2 driving the TUI) and SOP videos on a real Windows machine ([requirements](../../../../qa/security-levels/sprints/PF-27-S06/README.md#windows-machine-needed-for-the-remaining-gate)).
- [ ] Travis's acceptance of the documented limits (evidence README), including the new-thread DACL window.
- [ ] Follow-ups for the plan worker: a restricted or AppContainer token for the broker; Windows model auth
  (PF-27-S05); file tools other than patches under the contract on Windows; the elevated sandbox's read of
  `~/.git-credentials`, `.ssh`, `.npmrc`, `.config/gh` if profile reads are ever granted (setup excludes most).

## Verification

- [x] On `windows-2022` (job 113190329753, the merged slice 3 head): process-hardening 9, network-proxy broker and
  pipe suite 24, core 5 `pf_27_s06` tests pass. Linux (RTX box): clippy `-D warnings` clean; broker 55,
  process-hardening 7, core 12 + 23 tests pass. macOS: the same suites pass.
- [ ] GLM 5.2 tmux run and SOP videos on a Windows host.

## Exit evidence

- [x] Outputs under `qa/security-levels/sprints/PF-27-S06/` ([evidence](../../../../qa/security-levels/sprints/PF-27-S06/README.md)).
- [ ] Record archived (after the Windows-host gate items).
