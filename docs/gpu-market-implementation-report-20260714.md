# GPU market implementation report — 2026-07-14

## Outcome

The non-billable control plane is implemented on `feat/gpu-market-v2`. It is
fail-closed: neither built-in recipe can create a rental until its immutable
deployment manifest is verified. No provider resource was created and no money
was spent during this implementation run.

The feature does **not** meet the specification's Definition of Done yet. The
known-good DeepSeek V4 Flash manifest was not recoverable from non-secret local
artifacts, no explicit live-test budget was supplied, Vast does not yet prove an
authenticated TLS endpoint transport, and RunPod inventory does not prove the
high-bandwidth topology required by the two-H200 recipe.

## Invariant evidence

| Invariant | Implemented evidence |
| --- | --- |
| One authorization creates at most one resource | Durable client operation IDs, create-operation ledger, ownership-tag inventory adoption, controller leases, and ambiguous-create regression tests. |
| Every possibly billable resource remains visible | Known provider IDs remain billable until provider-confirmed absence; failure routes to cleanup/`TERMINATION_UNCONFIRMED`; global spend status remains visible. |
| Termination is confirmed only by provider state | Delete success enters reconciliation; only provider inventory absence records `TERMINATED_CONFIRMED`; provider-side deletion and ambiguous delete have regressions. |
| No unauthenticated public inference | Per-rental vault token, provider bootstrap environment, HTTPS/loopback URL validation, missing/wrong-token probes, and fail-closed provider transport derivation. |
| Secrets do not enter ordinary state or logs | Typed redacted secret values and create requests, scoped vault labels, command-backed runtime authentication, sanitized provider errors, and adapter assertions. |
| Price is authorization | Offer snapshots expire, exact offer and price are revalidated before create, and hourly/total/TTL authorization is durable. |
| Provisioning is deterministic and recoverable | Immutable image/model validation, capacity/deadline checks, provider-native launch configuration, digest-bound persisted steps, and recovery of a step left `running` by process death. |
| READY means product compatibility | Model identity, real chat, streaming, cancellation/drop recovery, structured tool call, and authentication probes all gate READY. |
| Runtime overlay is cross-process and health-aware | SQLite-backed runtime providers refresh existing TUI catalogs; degradation disables new selection and recovery restores it. |
| Notifications do not flood | READY, DEGRADED, FAILED, cleanup uncertainty, and confirmed termination are keyed by rental state sequence and globally deduplicated in SQLite. |

## Qualification executed

- `cargo test -p codex-gpu-market`: 34 passed.
- `cargo test -p codex-state gpu_`: 10 passed.
- `cargo test -p codex-tui gpu --lib`: 5 passed.
- `cargo clippy -p codex-gpu-market --tests -- -D warnings`: passed.
- `cargo check -p codex-cli -p codex-tui`: passed.
- Built `codex-rs/target/debug/pfterminal` from the final code.
- Ran that debug binary concurrently with existing PFTerminal processes, opened
  and dismissed `/gpu` repeatedly, confirmed both unverified recipes were
  disabled with the fail-closed reason, and observed no state-lock failure or
  input freeze.

The repository's full shared-core suite was not run because `AGENTS.md` requires
explicit approval before running it. No live-provider qualification was run
because it requires a separate explicit hard budget and TTL.

## Remaining release gates

1. Recover or provide the exact prior successful DeepSeek model commit,
   container digest/runtime, launch arguments, encoding path, capacity reserves,
   deadlines, and topology evidence; then mark that manifest verified.
2. Freeze and verify the one-H200 Qwen manifest.
3. Implement and prove a Vast authenticated TLS transport, or keep Vast creation
   unavailable.
4. Prove RunPod allocated topology before qualifying a topology-sensitive recipe.
5. With explicit budgets, pass one cheap full lifecycle independently on each
   provider and record provider-confirmed billing stop.
6. Pass the 30-minute DeepSeek free-form TUI qualification and human UAT.

