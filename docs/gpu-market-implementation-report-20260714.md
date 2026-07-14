# GPU market implementation report — 2026-07-14

## Outcome

The non-billable control plane is implemented on `feat/gpu-market-v2`. The
DeepSeek V4 Flash recipe is frozen and enabled; the Qwen recipe remains
fail-closed pending its own reproducibility spike. No provider resource has
been created and no money has been spent during this implementation run.

The feature does **not** meet the specification's Definition of Done or the
new-pane live acceptance criterion yet. Live spending is authorized, but this
machine has no RunPod or Vast key in the PFTerminal vault or environment. A
candidate credential file at `/home/pfrpc/vast.txt` has not been read because
the spec requires explicit import confirmation. Vast also does not yet prove
an authenticated TLS endpoint transport.

## Frozen DeepSeek manifest

- Model: `deepseek-ai/DeepSeek-V4-Flash`
- Model commit: `fea5c29efd213e8f5e6a8e7d897a68b40a390bdf`
- Model bytes: `159634522129`; weight bytes: `159617149040`
- Runtime: SGLang `v0.5.12`, source commit
  `127b9e3283f7c2a43234b852ff5c9f1796d53624`
- Linux/amd64 image manifest:
  `lmsysorg/sglang@sha256:015f39a45844be5a7b35270c56dc4d9ebcfe9b0c21a3b4f877a4ee22e795bd7a`
- Runtime CUDA: 13.0; architecture: Hopper `sm_90`
- Allocation: 2× NVIDIA H200, 400 GiB disk, 256 GiB host RAM, 384,000-token
  advertised context, two concurrent requests
- Pre-server gates: exact GPU count/model, minimum driver, NVLink topology,
  and SGLang P2P check
- Compatibility gates: dedicated `deepseekv4` tool parser, `deepseek-v4`
  reasoning parser, scoped bearer token, exact model identity, chat, streaming,
  actual client cancellation, and forced structured tool call

## Invariant evidence

| Invariant | Implemented evidence |
| --- | --- |
| One authorization creates at most one resource | Durable client operation IDs, create-operation ledger, ownership-tag inventory adoption, controller leases, and ambiguous-create regressions. |
| Every possibly billable resource remains visible | Known provider IDs remain billable until provider-confirmed absence; failure routes to cleanup/`TERMINATION_UNCONFIRMED`; global spend status remains visible. |
| Termination is confirmed only by provider state | Delete success enters reconciliation; only provider inventory absence records `TERMINATED_CONFIRMED`; provider-side deletion and ambiguous delete have regressions. |
| No unauthenticated public inference | Per-rental vault token, provider bootstrap environment, HTTPS/loopback URL validation, missing/wrong-token probes, and provider capability gating that excludes Vast before search/create until its secure transport is qualified. |
| Secrets do not enter ordinary state or logs | Typed redacted secret values and create requests, scoped vault labels, masked `/gpu` credential entry, sanitized provider errors, and adapter assertions. |
| Price is authorization | Offer snapshots expire, exact offer and price are revalidated before create, and hourly/total/TTL authorization is durable. |
| Provisioning is deterministic and recoverable | Immutable image/model validation, recipe-owned runtime command, aggregate weight/KV/workspace capacity checks, actual allocated RAM/disk/model/count checks, pre-server NVLink/P2P/driver gates, digest-bound persisted steps, and recovery after process death. |
| READY means product compatibility | Model identity, real chat, streaming, actual client cancellation/drop recovery, structured tool call, and authentication probes all gate READY. |
| Runtime overlay is cross-process and health-aware | SQLite-backed runtime providers refresh existing TUI catalogs; degradation disables new selection and recovery restores it; endpoint auth invokes the exact running PFTerminal binary. |
| Notifications do not flood | READY, DEGRADED, FAILED, cleanup uncertainty, and confirmed termination are keyed by rental state sequence and globally deduplicated in SQLite. |

## Qualification executed

- `cargo test -p codex-gpu-market`: 40 passed.
- `cargo test -p codex-state gpu_`: 10 passed (prior candidate).
- `cargo test -p codex-tui gpu --lib`: 5 passed (prior candidate).
- `cargo clippy -p codex-gpu-market --tests -- -D warnings`: passed.
- `cargo check -p codex-cli -p codex-tui`: passed after the manifest,
  credential, topology, and exact-binary auth-helper changes.
- Focused `codex-core` runtime-provider auth test: passed (1 test, 2,059
  unrelated unit tests and 890 integration tests filtered out).
- Built and ran the real debug binary concurrently with existing PFTerminal
  processes. `/gpu` showed masked RunPod/Vast credential actions, kept Qwen
  disabled, enabled the pinned DeepSeek recipe, accepted a bounded authorization,
  and returned a missing-credential error without creating a rental.

The repository's full shared-core suite was not run because `AGENTS.md` requires
explicit approval before running it. No live-provider qualification has run
because no provider key is configured.

## Remaining release gates

1. Configure a RunPod key through the masked `/gpu` action and run the exact
   DeepSeek rental acceptance from a newly opened PFTerminal pane/process.
2. Freeze and verify the one-H200 Qwen manifest.
3. Implement and prove a Vast authenticated TLS/tunnel transport, or keep Vast
   creation unavailable.
4. With bounded budgets, pass one cheap full lifecycle independently on each
   provider and record provider-confirmed billing stop.
5. Pass the 30-minute DeepSeek free-form TUI qualification, including model
   selection and ordinary tool use in a fresh pane, then human UAT.
