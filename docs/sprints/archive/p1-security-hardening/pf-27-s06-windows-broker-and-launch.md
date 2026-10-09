---
sprint_id: "PF-27-S06"
title: "Windows broker and secretless launch"
status: completed
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 42
owner: "Windows-host gate owner (Jim Ricketts; code by the broker lane worker, 2026-10-08)"
parallel_lane: "windows-host"
write_scope: "qa/security-levels/sprints/PF-27-S06/, qa/demos/index/PF-27-S06.md, docs/sprints/archive/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md"
integration_gate: "Per-sprint gate (sec-common decision 5), one PR per slice: pf_27_s06 tests on the windows-2022 runner (windows-security-probes workflow), Linux clippy on the RTX box, one Opus 5.5 High review per slice; merged behind the existing default-off flags. Remaining: GLM 5.2 tmux run and SOP videos on a real Windows machine, received by the P1 integration owner."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-27-s06-20261008"
branch: "sec/pf-27-s06-windows-broker"
base_commit: "7e6ef740ef1574d48c3caf2e84af82779c5b5969"
depends_on: "PF-27-S02"
merged_behind_flag: "isolated_credential_broker, secretless_agent_launch"
gate_evidence: "qa/security-levels/sprints/PF-27-S06/README.md"
created: 2026-10-06
updated: 2026-10-08
---

# PF-27-S06 — Windows broker and secretless launch

Explicit follow-up from PF-27-S04 (no Windows broker) and PF-27-S02 (macOS and Linux only), per the
coordinator's 2026-10-06 instruction not to block on Windows. Until this lands, Windows with
`isolated_credential_broker` virtualizes but never injects, and with `secretless_agent_launch` it refuses
agent commands with a stated reason. Moved to the P1 hardening plan on 2026-10-06 (Travis); its dependency
on [PF-27-S02](../../archive/p0-security-levels/pf-27-s02-secretless-agent-launch.md) is unchanged.

Started 2026-10-08 (Travis approved starting early). All code merged (PRs #267, #269, #270, #272) behind the
default-off flags above; real probes run on `windows-2022` CI. The first real-Windows gate run (2026-10-08) failed
with two defects. They were fixed in #298 (#294) and #302 (#295), and the gate rerun passed in a normal session
(Verification).

## Closure — 2026-10-08

Completed. Travis **accepted** PF-27-S06 **with known limits** on 2026-10-08 (in chat with the coordinator): the
limits are the [Known limits](../../../../qa/security-levels/sprints/PF-27-S06/README.md#known-limits) list in the
evidence README. Received and archived by the P1 integration owner; gate rerun evidence merged in PR #312.

Follow-ups (linked, not blockers):

- #300 (deny-read not enforced on the unelevated tool path): decided, fail closed; being implemented.
- #301 and #304 (deny entries revoked by a flag-off session / never revoked): decided, rule-driven removal; draft PR #326.
- #307 (launcher pipe inheritance) fixed by PR #321; #320 (`spawn_protected` stdout pipe) fixed by PR #327.
- #323 (an armed contract's deny entries vs. another `CODEX_HOME`'s sync): open.
- [PF-27-S08](../../current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) (broker token) and
  [PF-27-S09](../../current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md) (model auth): planned.
- Unplaced: file tools other than patches under the Windows contract; elevated-sandbox profile reads
  (`~/.git-credentials`, `.ssh`, `.npmrc`, `.config/gh`); both in the plan's carried-forward table.

## Execution mandate

- Deliver: the isolated broker process and the secretless launch contract on Windows, with measured containment.
- Excludes: macOS/Linux changes, model-client auth (PF-27-S05), Permissive changes.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/active/p1-security-hardening.md#pf-27).
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
  query-limited and synchronize only), new threads included. Measured: a restricted-token command and a same-user
  process cannot read their memory or environment, duplicate handles, inject, re-ACL or open threads (unhardened
  positive controls).
- [x] Broker transport on Windows (slices 2a/2b, PRs #269/#270): named pipes with random first-instance names, a
  user DACL, remote clients refused, no inheritable handles, both peers' process ids checked. The broker runs
  contained (`dacl+job`) and exits with its controller. The PF-27-S04/S28/S33 broker suite passes over the pipes.
- [x] Launch contract (slice 3, PR #272): the Windows refusal is now a measured pass under the elevated sandbox
  (separate sandbox user) and a stated refusal for the unelevated one. Measured: vault, sign-in, policy store and
  state databases unreadable (readable in the base-profile control), `CODEX_HOME` unwritable, files created or
  replaced during a run denied (an inherit-only, files-only deny on `CODEX_HOME`), Core stand-ins unopenable.
- [x] `pf_27_s06` tests run on every PR touching this code (`windows-security-probes` workflow); Linux clippy clean
  on the RTX box; Opus 5.5 High reviews per slice, all approved (2-3 rounds each).
- [x] Travis approved fixing the four documented limits (2026-10-08): [PF-27-S07](pf-27-s07-windows-hardening-follow-ups.md)
  (new threads, `CODEX_HOME` deny), [S08](../../current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) (broker token), [S09](../../current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md) (model auth).
- [x] Both gate defects fixed (#298 for #294, #302 for #295), each with a regression test that fails before and passes
  after on the real machine; `windows-security-probes` also runs them at medium integrity.
- [x] Travis accepted the sprint with its known limits (2026-10-08); follow-ups placed under Closure.

## Remaining

None. Follow-ups are listed under Closure.

## Verification

- [x] `windows-2022` (job 113190329753): process-hardening 9, broker and pipe suite 24, core 5 `pf_27_s06` tests
  pass. Linux (RTX box): clippy `-D warnings` clean; broker 55, process-hardening 7, core 12 + 23 pass. macOS: same.
- [x] GLM 5.2 tmux run and videos on real Windows: the first run (2026-10-08) **failed** with two defects; the
  [rerun](../../../../qa/security-levels/sprints/PF-27-S06/README.md#gate-rerun-after-the-fixes-2026-10-08-pass) at
  `661b5c6a48cd` **passed** in a normal session ([videos](../../../../qa/demos/index/PF-27-S06.md)).
- [x] Gate evidence received by the P1 integration owner (2026-10-08; PR #312).

## Exit evidence

- [x] Outputs under `qa/security-levels/sprints/PF-27-S06/` ([evidence](../../../../qa/security-levels/sprints/PF-27-S06/README.md)).
- [x] Record archived (2026-10-08, after Travis's acceptance).
