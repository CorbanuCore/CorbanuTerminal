---
sprint_id: "PF-27-S05"
title: "Core model-client auth and vault labels through the broker"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 46
owner: "broker lane worker round 5 (2026-10-06)"
parallel_lane: "broker"
write_scope: "codex-rs/network-proxy/src/credential_broker/model_auth.rs, codex-rs/network-proxy/src/credential_broker/isolated/, codex-rs/network-proxy/src/credential_broker/isolated_tests.rs, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/providers.rs, codex-rs/network-proxy/src/lib.rs, codex-rs/model-provider/src/auth.rs, codex-rs/model-provider/src/lib.rs, codex-rs/http-client/src/client.rs, codex-rs/features/src/lib.rs, codex-rs/core/src/model_broker_auth.rs, codex-rs/core/src/model_broker_auth_tests.rs, codex-rs/core/src/client.rs, codex-rs/core/src/lib.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/memory_stage_one.rs, codex-rs/core/config.schema.json, qa/security-levels/sprints/PF-27-S05/, qa/demos/specs/, qa/demos/index/PF-27-S05.md, docs/sprints/current/p0-security-levels/pf-27-s05-model-client-auth-broker.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5); merged behind the new default-off broker_model_auth flag. Shared files kept to small hunks: one builder call per ModelClient construction site, transport selection in core/src/client.rs, one feature entry."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/sec-broker5-20261006"
branch: "feat/pf27-s05-model-auth-broker-20261006"
base_commit: "00376a1fb00cd3fd3bc085373f3466038b861ec2"
depends_on: "PF-27-S02"
created: 2026-10-06
updated: 2026-10-06
---

# PF-27-S05 — Core model-client auth and vault labels through the broker

Created by the coordinator's 2026-10-06 decision (Travis may revisit): moving Core's own model-client auth into
the broker is its own sprint, after PF-27-S02 in the broker lane. It touches the model clients rather than the
proxy, and it needs PF-27-S02's containment before the broker is a real boundary.

## Execution mandate

- Deliver: Core's own model-provider requests are authorized by the isolated broker; Core holds opaque references,
  not raw provider keys, and vault-label credentials are resolved only inside the broker.
- Excludes: agent launch containment (PF-27-S02), output gates (PF-28), Windows (PF-27-S06), Permissive changes.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: raw credentials exist only in the trusted broker, including the ones Core uses itself.

## Code boundaries

- Existing: `codex-rs/model-provider/src/auth.rs`; `codex-rs/login/` (API-key auth); `codex-rs/core/src/client.rs`;
  `codex-rs/network-proxy/src/credential_broker/isolated/`; `codex-rs/vault/src/`.
- Planned: a broker route for model-provider requests; a vault-label resolver inside the broker process.
- Tests: colocated `pf_27_s05` modules; synthetic keys and fake providers only.

## Preconditions

- [x] PF-27-S02 completed and archived (#214); plan active. The record stays `draft` until the plan worker records this
  worktree in the plan (the checker requires it for `in_progress`); the slice merges behind its flag meanwhile.

## Done

- [x] Record created from the PF-27-S04 closure with scope and dependencies.
- [x] Inventory (first slice): Core reads model keys from the provider `env_key` (environment, then the vault label
  `provider/<ENV_KEY>` or legacy `provider_auth.json` through `AuthManager::provider_api_key`), the OpenAI API-key login
  in `auth.json`, `experimental_bearer_token`, sign-in tokens (ChatGPT, agent identity), command-backed bearer auth and
  AWS SigV4. All model requests are signed in `codex-api` `apply_auth` and sent by `core/src/client.rs` transports.
- [x] Slice 1, behind `broker_model_auth` (default off): when a provider's auth is one plain API key (env or vault
  provider key, OpenAI API-key login, `experimental_bearer_token`; `Authorization: Bearer` or `x-api-key`), Core registers
  it once per process with a contained broker (Seatbelt / seccomp; refused if it cannot confine itself) bound to the
  base URL's HTTPS origin and path prefix, and keeps an opaque reference. Every model request (Responses, Chat,
  Anthropic Messages, compact, memories, realtime call) goes as plain HTTP over the broker's private Unix socket with
  a single-use signed frame for its exact origin, method and path; the broker attaches the key and makes the HTTPS
  request. No raw-key fallback: a broker that fails to start or dies, a provider URL the broker cannot bind (plain
  HTTP, IPv6 literal, query) and non-Unix platforms all fail the request with a non-retried error. The broker is never
  respawned in-process (as PF-27-S04). Responses websockets are off under the flag; redirects come back unfollowed.
  Sign-in tokens, agent identity, command, header and AWS auth are not brokered (sent as before).
- [x] `pf_27_s05` tests: broker-only key use for both header styles, single-use frames, origin and path-prefix binding
  enforced by Core and again by the broker, malformed bindings refused, broker death fails closed (network-proxy);
  key extraction matches direct auth and sign-in auth is not extracted (model-provider); base-URL binding and request
  rewrite, unbrokerable key fails closed with a flag-off control (core). Live check: GLM 5.2 via Z.AI answered through
  the broker (`containment=seatbelt`). [Evidence](../../../../qa/security-levels/sprints/PF-27-S05/README.md).
- [x] Opus 5.5 High: review 1 CHANGES REQUESTED (H3 fixed; H1/H2/M1/M2 moved to Remaining), review 2 APPROVE.

## Remaining

- [ ] Core still reads the key to register it (and on each setup to detect a changed key): resolve vault labels inside
  the broker so Core never decrypts them, and read env keys once.
- [ ] Other Core paths that still attach the key directly with the flag on (review 1 H1, H2, M1, M2): web search
  (`ext/web-search/src/tool.rs`) and image generation (`ext/image-generation/src/backend.rs`) via `provider.api_auth()`;
  the model catalog refresh (`model-provider/src/models_endpoint.rs`); the realtime conversation websocket
  (`core/src/realtime_conversation.rs`); OpenAI API-key users of `auth_provider_from_auth` (`core/src/mcp_openai_file.rs`,
  `codex-mcp`, `core-plugins`, `core-skills`, `analytics`). Broker them or refuse them under the flag.
- [ ] Remove env-sourced provider keys from Core's process environment once the broker holds them, so unsandboxed same-user processes (MCP servers, hooks) cannot read them from Core's launch environment (the macOS limit PF-27-S02 records).
- [ ] Resolve vault-label credentials inside the broker only; the agent and Core see labels and dummies.
- [ ] ChatGPT sign-in (refreshing tokens) and Responses websockets: decision recorded for slice 1 (not brokered;
  websockets off under the flag); a later slice needs a broker-side token holder and a websocket upgrade route.
- [ ] Memory-dump check (no raw key in Core's address space) once Core no longer reads the key.
- [ ] Windows: the non-Unix branch fails closed but has no direct test (PF-27-S06, P1).

## Verification

- [x] `just fix -p` and `just fmt`; focused `pf_27_s05` tests and the affected crates (577 + 910 passed); Linux
  clippy clean on the RTX box.
- [x] GLM 5.2 tmux runs as two SOP videos ([index](../../../../qa/demos/index/PF-27-S05.md)); Opus 5.5 High review.

## Exit evidence

- [ ] Outputs under `qa/security-levels/sprints/PF-27-S05/`; Done/Remaining reflect reality; record archived.
