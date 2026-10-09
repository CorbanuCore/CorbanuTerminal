---
sprint_id: "PF-27-S08"
title: "Windows broker confined by its own restricted token or AppContainer"
status: ready
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
updated: 2026-10-09
---

# PF-27-S08 — Windows broker confined by its own restricted token or AppContainer

Third of the four PF-27-S06 limits Travis approved fixing (2026-10-08). Unlike macOS and Linux (Seatbelt, Landlock),
the Windows broker had only the process DACL and a no-child-process job, so it could write the user's files, open
their processes, or start work through WMI, Task Scheduler or out-of-process COM. Broker lane, 2026-10-08.

## Execution mandate

- Deliver: the Windows broker runs under a token that cannot write outside its own runtime state, open other
  processes of the user (beyond query-limited and terminate, measured), or start work through WMI, Task Scheduler
  or COM, while it still serves its pipes and reaches upstreams; the stored-key path is decided and measured.
- Excludes: Windows model auth and the Windows stored-key reader (PF-27-S09), macOS/Linux, Permissive.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/active/p1-security-hardening.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: the process that holds raw credentials is itself confined on every platform.

## Code boundaries

- `process-hardening/src/windows_protected_spawn.rs` (broker token), `broker_containment.rs` (`token+dacl+job`).
- `network-proxy/src/credential_broker/isolated/pipe.rs`, `server.rs`: pipe DACLs admit the broker token and Core only.

## Preconditions

- [x] PF-27-S07 merged; a real Windows machine (used 2026-10-09); key-path decision (Travis, 2026-10-09: (c)).

## Acceptance criteria

1. The broker starts under the chosen token (spike: AppContainer with `internetClient` and
   `privateNetworkClientServer`, against a write-restricted token with a broker capability SID; pick the stronger
   one that passes 2–5) and reports it in its containment string; Core refuses a broker without it.
2. The broker cannot create or modify a file the user can write outside its runtime state (positive control: the
   PF-27-S07 broker can). Measured exception: delete in low-integrity folders (Known limits).
3. The broker cannot open another process of the user for any access beyond query-limited (control: it can today).
   Measured exception: `PROCESS_TERMINATE` (Known limits).
4. The broker cannot start a process through `Win32_Process.Create`, `Schedule.Service` or an out-of-process COM
   server (control: an unconfined process can).
5. The PF-27-S04/PF-28-S02/PF-33-S02 broker suite passes over the pipes; upstream TLS and DNS work; stored keys
   come from Credential Manager, read by the broker under its own token (decision (c)): the token can read it
   (measured); the Windows reader (`arg0`, vault lock file; none in `server.rs` yet) is PF-27-S09's.

## Test plan

- `pf_27_s08` probes (positive controls) on `windows-2022` and the Windows machine; tmux run, videos, one review.

## Decisions

- Token: write-restricted low-integrity token, not an AppContainer; PF-27-S07 holder start (evidence README).
- Key path: **(c), Travis, 2026-10-09**: the broker reads Credential Manager itself, as the macOS/Linux brokers read
  the OS keyring (PF-27-S05); PF-27-S09 ports S05. Accepted: a compromised broker can read and write the user's
  other generic credentials (and likely overwrite or delete them; see the evidence README).
- #333 merged with five checks pending; they passed later on main (`d0544c1c91`: windows-security-probes, CI).

## Done

- [x] Planned (2026-10-08); criteria 1–5 as amended, with positive controls, on `windows-2022` and the real
  machine (PR #333); key path decided 2026-10-09.
- [x] Real-Windows GLM 5.2 tmux run and SOP videos (2026-10-09).

## Remaining

- [ ] Travis's acceptance with the known limits in the evidence README, then archive.

## Verification

- [x] Probes and suite on `windows-2022` and the real machine (elevated and normal session), 2026-10-09; rechecked
  on main `5d283fde18` (job 113966110399): `pf_27_s08` 7/7 and suite 25/25, both sessions.
- [x] Opus 5.5 High review: REQUEST_CHANGES (B1, B2), fixed in #333; scoped re-review 2026-10-09 confirmed both,
  docs corrections from it applied (`workers-20261002/pf27s08-review-s2`).
- [ ] Travis accepts the gate evidence.

## Exit evidence

- [x] Outputs under `qa/security-levels/sprints/PF-27-S08/` (probes, gate run, videos, key-path decision).
- [ ] Record archived after Travis's acceptance.
