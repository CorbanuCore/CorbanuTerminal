---
sprint_id: "PF-27-S09"
title: "Windows model-client auth through the broker"
status: completed
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 45
owner: "broker lane worker (2026-10-09)"
parallel_lane: "broker"
write_scope: "codex-rs/network-proxy/src/credential_broker/, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/native_certs.rs, codex-rs/network-proxy/src/lib.rs, codex-rs/network-proxy/Cargo.toml, codex-rs/vault/src/lib.rs, codex-rs/arg0/src/lib.rs, codex-rs/arg0/Cargo.toml, codex-rs/http-client/src/model_broker_route.rs, codex-rs/http-client/src/transport.rs, codex-rs/http-client/src/transport_tests.rs, codex-rs/http-client/src/lib.rs, codex-rs/core/src/model_broker_auth.rs, codex-rs/core/src/model_broker_auth_tests.rs, codex-rs/Cargo.lock, .github/workflows/windows-security-probes.yml, qa/security-levels/sprints/PF-27-S09/, qa/demos/index/PF-27-S09.md, qa/demos/specs/pf27s09-win-env-key-brokered.toml, qa/demos/specs/pf27s09-win-vault-key-brokered.toml, qa/demos/specs/pf27s09-win-ssh-brokered-sandbox.toml, docs/sprints/archive/p1-security-hardening/pf-27-s09-windows-model-client-auth.md"
integration_gate: "Per-sprint gate (sec-common decision 5), one PR per slice (1: broker side, network-proxy/vault/arg0; 2: Core and http-client routing): Windows clippy and Linux clippy (RTX box) -D warnings, focused tests on the real Windows machine elevated and in a normal session, windows-security-probes on windows-2022, one Opus 5.5 High review per slice; merged behind the default-off broker_model_auth flag. GLM 5.2 tmux SOP videos on the real Windows machine from a normal session and over SSH."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-27-s09-20261009"
branch: "sec/pf-27-s09-model-auth"
base_commit: "5d283fde18b9924127d061f7c4f5b58211a3a50a"
depends_on: "PF-27-S05, PF-27-S07, PF-27-S08"
merged_behind_flag: "broker_model_auth (default off)"
gate_evidence: "qa/security-levels/sprints/PF-27-S09/README.md"
created: 2026-10-08
updated: 2026-10-10
---

# PF-27-S09 — Windows model-client auth through the broker

Fourth of the four PF-27-S06 limits Travis approved fixing (2026-10-08): the port of
[PF-27-S05](../../archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md) to Windows, with Travis's key
path (c) (2026-10-09): the broker reads the vault key from Credential Manager under its own PF-27-S08 token.
Allocated 2026-10-09 to the broker lane; slice 2 is on `sec/pf-27-s09-core`.

## Closure — 2026-10-10

Completed. Travis **accepted** PF-27-S09 **with known limits** on 2026-10-10 (in chat with the coordinator: "Pass
it"), after the independent code-blind acceptance on the real Windows 11 machine returned **ACCEPT WITH LIMITS**
([acceptance README](../../../../qa/security/pf-27-s09/independent-acceptance-2026-10-09/README.md), PR #384).
Code in PRs #363 (broker side) and #379 (Core); gate evidence in the
[evidence README](../../../../qa/security-levels/sprints/PF-27-S09/README.md#known-limits).
`broker_model_auth` is **still default off**.

Accepted limits, from the sprint's [Known limits](../../../../qa/security-levels/sprints/PF-27-S09/README.md#known-limits):

1. The key overwrite is best effort: copies outside the heaps and the launch environment block (for example on a
   thread stack) are not found; other code's copies are overwritten on purpose; about 140 ms per key (debug).
2. The memory scan covers the key hand-over, not a full Core start-up.
3. No Credential Manager over SSH: the vault key stays in the profile's file fallback there (as before this
   sprint); the broker reads it the same way.
4. Decision (c): the broker's token can read and write the user's other generic credentials.
5. Replacing the broker's copy on a ChatGPT sign-in refresh is covered by unit tests only.
6. Brokered Windows requests lack reqwest's tracing headers and the request debug log.

And from the independent acceptance run:

1. SSH-only runs; no console-session (Credential Manager) run.
2. Memory scan after hand-over only, not Core's start-up.
3. ChatGPT sign-in refresh not tested.
4. Exclusive pipe-squat scenario (the broker cannot create its pipe) not tested.

Received and archived by the P1 integration owner.

Follow-ups (linked, not blockers):

- #389: console-session Credential Manager acceptance run on the real Windows machine.
- #390: exclusive pipe-squat probe.
- #391: decision on turning `broker_model_auth` on by default.

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

- Broker side: `network-proxy` (`credential_broker/` `model_auth.rs`, `env_scrub.rs`, `isolated/`; `native_certs.rs`),
  `vault/src/lib.rs` (lock), `arg0` (stored-key reader).
- Core: `core/src/model_broker_auth.rs`; routing in `http-client/src/model_broker_route.rs`, `transport.rs`.

## Preconditions

- [x] PF-27-S05, S07 and S08 archived (S08 accepted 2026-10-09).
- [x] A real Windows 11 machine (used 2026-10-09), for the GLM 5.2 runs, videos and Credential Manager.

## Acceptance criteria

1. With the flag on, every model request (model client, web search, image generation, model catalog) carries a
   signed frame and goes only to the broker's pipe; with no broker installed it is refused, never sent direct.
2. Env provider keys are handed to the broker and overwritten in Core's environment block; a same-user scan of
   Core's memory finds no raw key after hand-over (positive control before it).
3. Stored provider keys are decrypted inside the broker, with the vault key read from Credential Manager by the
   broker's own token (decision (c)); a ChatGPT sign-in refresh replaces the broker's copy.
4. Fails closed before the broker runs and when it dies; flag off is unchanged.

## Test plan

- `pf_27_s05`/`pf_27_s09` tests on Windows (memory scan with positive controls) in `windows-security-probes`;
  GLM 5.2 runs and SOP videos on the Windows machine; Linux clippy on the RTX box; Opus 5.5 High reviews.

## Done

- [x] Planned (2026-10-08); allocated 2026-10-09 (broker lane).
- [x] Slice 1 (#363, merged), broker side: Windows env scrub (environment block, C runtime tables, every heap copy); no withheld
  value in the broker's launch environment; stored keys read in the broker (lock by Core, read-only lock in the
  broker); checked pipe send; platform roots loaded read-only (the broker trusted none before).
- [x] Slice 2, Core: the Windows start spawns the broker, hands over env keys and routes every frame-bearing
  request through `PipeSender` to the checked pipe; unsupported platforms still refuse.
- [x] Criteria 1–4 measured ([evidence](../../../../qa/security-levels/sprints/PF-27-S09/README.md)); the vault
  key came from Credential Manager in the console session; the ChatGPT refresh is unit-level only.
- [x] GLM 5.2 `corbanu exec` and SOP videos over SSH and in the console (normal) session.
- [x] Independent code-blind acceptance on the real Windows machine: ACCEPT WITH LIMITS (PR #384).
- [x] Travis accepted the sprint with the known limits (2026-10-10); follow-ups under Closure.

## Remaining

None. Follow-ups are listed under Closure.

## Verification

- [x] Windows clippy and Linux clippy (RTX box) `-D warnings`. Focused tests elevated and in a normal session on the
  real Windows 11 machine, plus Linux and macOS. `windows-security-probes` on `windows-2022` (#363).
- [x] Opus 5.5 High reviews: slice 1 APPROVE (and two scoped re-reviews), slice 2 APPROVE; findings fixed or recorded.
- [x] Travis accepted the gate evidence (2026-10-10); received by the P1 integration owner.

## Exit evidence

- [x] Outputs under `qa/security-levels/sprints/PF-27-S09/`; videos in `qa/demos/index/PF-27-S09.md`.
- [x] Record archived (2026-10-10, after Travis's acceptance, by the P1 integration owner).
