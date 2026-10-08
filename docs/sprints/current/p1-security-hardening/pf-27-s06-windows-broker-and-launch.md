---
sprint_id: "PF-27-S06"
title: "Windows broker and secretless launch"
status: active
plan_file: "docs/plans/proposed/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 45
owner: "broker lane worker (2026-10-08)"
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

Started 2026-10-08 (Travis approved starting before the plan is activated). No Windows host yet: real probes run
on the `windows-2022` CI runners (`windows-security-probes` workflow); the tmux run and videos wait for a Windows
machine. Several reviewable PRs, each from a branch named after this one.

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
- Planned: named-pipe broker transport with peer-process checks; restricted-token/AppContainer containment probes.
- Tests: `pf_27_s06` modules, run on Windows CI.

## Preconditions

- [x] PF-27-S02 completed and archived; Windows CI runner for real probes (a Windows host is still needed for the tmux run and videos).

## Done

- [x] Record created as the explicit Windows follow-up.

## Remaining

- [ ] Broker transport on Windows (named pipe, DACL to the controller, client process id check, no inheritable handles).
- [ ] Process containment probes: agent cannot open Core or broker with `PROCESS_VM_READ`, read their environment, or inherit their handles.
- [ ] Turn the launch contract's Windows refusal into a measured pass, or keep it with the reason.
- [ ] `pf_27_s06` tests and the decision 5 gate (tests, tmux run, review, videos).

## Verification

- [ ] `just test -p codex-network-proxy pf_27_s06` and `just test -p codex-core pf_27_s06` on Windows.

## Exit evidence

- [ ] Outputs under `qa/security-levels/sprints/PF-27-S06/`; record archived.
