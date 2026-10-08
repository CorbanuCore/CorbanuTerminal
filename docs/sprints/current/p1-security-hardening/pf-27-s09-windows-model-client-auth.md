---
sprint_id: "PF-27-S09"
title: "Windows model-client auth through the broker"
status: draft
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 45
owner: "broker lane (unassigned)"
parallel_lane: "broker"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-27-S05, PF-27-S07"
created: 2026-10-08
updated: 2026-10-08
---

# PF-27-S09 — Windows model-client auth through the broker

Fourth of the four PF-27-S06 limits Travis approved fixing (2026-10-08): the port of
[PF-27-S05](../../archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md) to Windows. Today
`broker_model_auth` on Windows refuses every brokered request (`model_broker_auth.rs`, non-Unix branch). Plan
only: nothing is implemented until this record is allocated. Runs after PF-27-S08; if S08 has landed, stored keys
use its key path.

## Execution mandate

- Deliver: with `broker_model_auth` on Windows, Core's model requests are authorized by the broker over named
  pipes; Core holds references, not raw keys.
- Excludes: the broker's own token (PF-27-S08), agent launch (PF-27-S02/S06), macOS/Linux, Permissive.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/active/p1-security-hardening.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: raw credentials exist only in the trusted broker on Windows too, including Core's own.

## Code boundaries

- Core: `core/src/model_broker_auth.rs` (the `cfg(not(unix))` refusal becomes the Windows start).
- Broker: `network-proxy/src/credential_broker/model_auth.rs` and `env_scrub.rs` (Unix-only today), `isolated/`.
- Routing: `http-client/src/model_broker_route.rs`, `transport.rs` (named pipe instead of a Unix socket, with the
  PF-27-S06 server process-id check).
- `model-provider/src/model_key_broker.rs`; stored-key reader in `arg0`.

## Preconditions

- [ ] PF-27-S05 archived (done) and PF-27-S07 merged.
- [ ] **Needs a real Windows machine** for the GLM 5.2 tmux run and SOP videos with a real provider key, and for
  stored keys through Credential Manager. Everything else runs on `windows-2022`.

## Acceptance criteria

1. With the flag on, every model request (model client, web search, image generation, model catalog) carries a
   signed frame and goes only to the broker's pipe; with no broker installed it is refused, never sent direct.
2. Env provider keys are handed to the broker and overwritten in Core's environment block; a same-user scan of
   Core's memory finds no raw key after hand-over (positive control before it).
3. Stored provider keys are decrypted inside the broker; a ChatGPT sign-in refresh replaces the broker's copy.
4. Fails closed before the broker runs and when it dies; flag off is unchanged.

## Test plan

- The `pf_27_s05` suites in network-proxy, model-provider, http-client and core, enabled on Windows and run by
  `windows-security-probes`; a Windows memory scan of Core (own process) with a positive control.
- GLM 5.2 tmux run and SOP videos on the Windows machine; Linux clippy on the RTX box; Opus 5.5 High review.

## Done

- [x] Planned (2026-10-08).

## Remaining

- [ ] Everything under Acceptance criteria.

## Verification

- [ ] Suites on `windows-2022`; tmux run and videos on a real Windows machine.

## Exit evidence

- [ ] Outputs under `qa/security-levels/sprints/PF-27-S09/`.
