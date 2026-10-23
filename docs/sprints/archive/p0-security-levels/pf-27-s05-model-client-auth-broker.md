---
sprint_id: "PF-27-S05"
title: "Core model-client auth and vault labels through the broker"
status: completed
plan_file: "docs/plans/completed/main-2026-10-08-p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 46
owner: "broker lane worker round 6 (2026-10-06)"
parallel_lane: "broker"
write_scope: "codex-rs/network-proxy/src/credential_broker/model_auth.rs, codex-rs/network-proxy/src/credential_broker/isolated/, codex-rs/network-proxy/src/credential_broker/isolated_tests.rs, codex-rs/network-proxy/src/credential_broker/env_scrub.rs, codex-rs/network-proxy/src/credential_broker/env_scrub_tests.rs, codex-rs/network-proxy/src/credential_broker/memory_scan_tests.rs, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/providers.rs, codex-rs/network-proxy/src/lib.rs, codex-rs/model-provider/src/auth.rs, codex-rs/model-provider/src/lib.rs, codex-rs/model-provider/src/provider.rs, codex-rs/model-provider/src/model_key_broker.rs, codex-rs/http-client/, codex-rs/login/src/auth/manager.rs, codex-rs/login/src/lib.rs, codex-rs/secrets/src/local.rs, codex-rs/process-hardening/, codex-rs/arg0/, codex-rs/features/src/lib.rs, codex-rs/core/src/model_broker_auth.rs, codex-rs/core/src/model_broker_auth_tests.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/lib.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/memory_stage_one.rs, codex-rs/core/src/realtime_conversation.rs, codex-rs/core/config.schema.json, codex-rs/Cargo.lock, qa/security-levels/sprints/PF-27-S05/, qa/demos/index/PF-27-S05.md, docs/sprints/archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5); merged behind the default-off broker_model_auth flag. Shared files serialized by the integration owner, not reserved here: small hunks in codex-rs/core/src/client.rs (transport and websocket selection; PF-23-S01 reserves it) and codex-rs/core/src/config/mod.rs (marks the process brokered, as PF-27-S02 arms its contract; PF-60-S03 reserves the file) and two new demo specs qa/demos/specs/pf27s05-*.toml (new files only, in the directory PF-23-S01 reserves)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/sec-broker6-20261006"
branch: "feat/pf27-s05-broker-finish-20261006"
base_commit: "c5bdadd322d266d4961479ca5fa34c70c9486e11"
depends_on: "PF-27-S02"
created: 2026-10-06
updated: 2026-10-07
---

# PF-27-S05 — Core model-client auth and vault labels through the broker

Coordinator decision 2026-10-06: Core's model-client auth moves into the broker, behind `broker_model_auth` (off).

## Closure — 2026-10-07

Completed under the per-sprint gate (PR #237, slice 1 #229), merged behind `broker_model_auth` (off by default).
Known limits below are follow-ups for the plan worker; the main one is the in-process vault opens by other features.

## Execution mandate

- Deliver: Core's model-provider requests authorized by the broker; Core holds references, not raw keys.
- Excludes: agent launch containment (PF-27-S02), output gates (PF-28), Windows (PF-27-S06), Permissive changes.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/completed/main-2026-10-08-p0-security-levels.md#pf-27); feature `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: raw credentials exist only in the trusted broker, including the ones Core uses itself.

## Code boundaries

- Process switch and broker use: `model-provider/src/model_key_broker.rs`, `auth.rs`, `provider.rs`; Core side
  `core/src/model_broker_auth.rs` (set up by `core/src/config/mod.rs` and the session).
- Frame routing: `http-client/src/model_broker_route.rs`, `transport.rs`. Broker: `network-proxy/src/credential_broker/`
  (`model_auth.rs`, `env_scrub.rs`, `isolated/`), stored-key reader in `arg0`, containment in `process-hardening`.
- Tests: `pf_27_s05` in network-proxy, model-provider, http-client, process-hardening, secrets, core.

## Preconditions

- [x] PF-27-S02 archived (#214); worktree in the plan front matter.

## Done

- [x] Slice 1 (#229): `ModelClient` API keys held by a contained broker; every model request is signed for it.
- [x] A loaded config that enables the flag makes the whole process brokered (one-way). From then on Core reads no
  provider key (it holds a placeholder). It attaches no plain key or sign-in token itself. Before the broker runs,
  such uses fail instead of going direct. Header-only users of a first-party login get no credential. Websockets
  and realtime conversations are off.
- [x] Brokered through `resolve_provider_auth`: the model client, web search, image generation and the model
  catalog. This covers env and vault provider keys, the OpenAI API-key login, `experimental_bearer_token` and
  ChatGPT sign-in access tokens. A refreshed token replaces the broker's copy, and a sign-in used for an env-key
  provider goes with that provider's header. Not brokered (sent as before): command, AWS, header and
  agent-identity auth.
- [x] A frame-bearing request goes only to the broker's socket (refused with no broker installed).
- [x] Stored provider keys are decrypted inside the broker:
  - the broker creates the vault lock before containment, and never through a symlink;
  - the secrets layer no longer chmods files that are already private, so a read needs no write.
- [x] Env provider keys are handed to the broker and overwritten in Core's environment (`ps -E` shows zeros). If
  the broker refuses one or fails to start, the keys are still scrubbed and the broker is marked failed.
- [x] Tests:
  - memory scan: Core's writable memory has no raw key after hand-over (positive control first; Linux also
    checks `/proc/self/environ`);
  - the non-Unix branch refuses;
  - fail closed before the broker runs; flag-off direct auth unchanged; command auth sent as before;
  - the vault lock is never a symlink; registrations race safely.

## Remaining

- None in this sprint; follow-ups are under Known limits.

## Known limits (follow-ups for the plan worker)

- Other Core/TUI features (provider status, `/vault`, Task Node, Telegram, wallet, campaign tracker) still open the
  encrypted vault in-process, which decrypts the file in Core memory; only the provider key *for requests* is read
  by the broker alone. Follow-up: split the vault index from values or serve metadata through the broker. These
  features no longer see an env-only provider key.
- Env scrubbing runs at session start (races C-level `getenv` and `setenv` from outside `codex-network-proxy`,
  whose own writers share its lock since #222; a launch value `.env` replaced keeps its bytes);
  moving it before `main` needs the flag decided at process start. Only the first enabling config's key variables
  are handed over; pre-session uses (first catalog refresh, `corbanu doctor`) fail closed.
- Core still holds and refreshes ChatGPT sign-in tokens; agent-identity registration sends the access token
  directly; a request signed with a replaced reference fails once. Windows: PF-27-S06 (P1).

## Verification

- [x] `just fix -p` and `just fmt`. Affected crates: 1312 passed; core subsets: 1099 passed. The 2 `suite::client::skills_*`
  failures are unrelated: real `~/.agents/skills` leak into the test. Linux clippy `-D warnings` is clean on the RTX box,
  and the Linux `pf_27_s05` tests pass.
- [x] GLM 5.2 tmux runs as SOP videos, keyring-isolated ([index](../../../../qa/demos/index/PF-27-S05.md)).
- [x] Opus 5.5 High reviews 3 and 4 (changes requested, fixed); see [evidence](../../../../qa/security-levels/sprints/PF-27-S05/README.md).
- [x] Final re-review (review 5): APPROVE.

## Exit evidence

- [x] Outputs under `qa/security-levels/sprints/PF-27-S05/`; Done and Known limits reflect reality.
- [x] Record archived with the PR (#237).
