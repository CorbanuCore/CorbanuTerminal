---
sprint_id: "PF-27-S09"
title: "Windows model-client auth through the broker"
status: in_progress
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-27"
execution_order: 45
owner: "broker lane worker (2026-10-09)"
parallel_lane: "broker"
write_scope: "codex-rs/network-proxy/src/credential_broker/, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/native_certs.rs, codex-rs/network-proxy/src/lib.rs, codex-rs/network-proxy/Cargo.toml, codex-rs/vault/src/lib.rs, codex-rs/arg0/src/lib.rs, codex-rs/arg0/Cargo.toml, codex-rs/http-client/src/model_broker_route.rs, codex-rs/http-client/src/transport.rs, codex-rs/http-client/src/transport_tests.rs, codex-rs/http-client/src/lib.rs, codex-rs/core/src/model_broker_auth.rs, codex-rs/core/src/model_broker_auth_tests.rs, codex-rs/Cargo.lock, .github/workflows/windows-security-probes.yml, qa/security-levels/sprints/PF-27-S09/, qa/demos/index/PF-27-S09.md, qa/demos/specs/pf27s09-win-env-key-brokered.toml, qa/demos/specs/pf27s09-win-vault-key-brokered.toml, qa/demos/specs/pf27s09-win-ssh-brokered-sandbox.toml, docs/sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md"
integration_gate: "Per-sprint gate (sec-common decision 5), one PR per slice (1: broker side, network-proxy/vault/arg0; 2: Core and http-client routing): Windows clippy and Linux clippy (RTX box) -D warnings, focused tests on the real Windows machine elevated and in a normal session, windows-security-probes on windows-2022, one Opus 5.5 High review per slice; merged behind the default-off broker_model_auth flag. GLM 5.2 tmux SOP videos on the real Windows machine from a normal session and over SSH."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-27-s09-20261009"
branch: "sec/pf-27-s09-model-auth"
base_commit: "5d283fde18b9924127d061f7c4f5b58211a3a50a"
depends_on: "PF-27-S05, PF-27-S07, PF-27-S08"
merged_behind_flag: "broker_model_auth (default off)"
gate_evidence: "qa/security-levels/sprints/PF-27-S09/README.md"
created: 2026-10-08
updated: 2026-10-09
---

# PF-27-S09 — Windows model-client auth through the broker

Fourth of the four PF-27-S06 limits Travis approved fixing (2026-10-08): the port of
[PF-27-S05](../../archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md) to Windows. Today
`broker_model_auth` on Windows refuses every brokered request (`model_broker_auth.rs`, non-Unix branch). Plan
allocated 2026-10-09 to the broker lane (slice 1 on `sec/pf-27-s09-model-auth`, slice 2 on `sec/pf-27-s09-core`). Runs after PF-27-S08 (merged in #333). Key path: Travis
decided (c) on 2026-10-09: the broker reads the vault key from Credential Manager itself, under its PF-27-S08
token, the same as the macOS and Linux brokers read the OS keyring, so S05's stored-key design ports as is. One
Windows addition: every vault read opens `secrets/.vault.lock` for writing, which the S08 token cannot do on a
medium-labeled file; grant it (capability-SID entry and low label, like S05's `prepare_vault_lock` Landlock
exception) or hand the broker an opened lock. The Windows broker reads no stored keys yet (`server.rs`).

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
- Vault lock for the broker token: `isolated/server.rs` (`prepare_vault_lock`), `vault/src/lib.rs`.
- Routing: `http-client/src/model_broker_route.rs`, `transport.rs` (named pipe instead of a Unix socket, with the
  PF-27-S06 server process-id check).
- `model-provider/src/model_key_broker.rs`; stored-key reader in `arg0`.

## Preconditions

- [x] PF-27-S05 and S07 archived; PF-27-S08 merged behind its flags (#333; `ready`, awaiting Travis).
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
