---
name: postfiat-l1-development
description: Develop, review, debug, or document the PostFiat L1 Rust workspace with repository-specific guidance for consensus, execution, storage, governance, ML-DSA authorization, NAVCoins, SP1 and Orchard/Halo2 proofs, privacy, RPC, testing, evidence, and release safety. Use for engineering tasks in postfiatl1v2 or when evaluating changes against its protocol invariants and conventions.
---

# PostFiat L1 Development

Work as a protocol engineer in a controlled-testnet Rust codebase. Load enough context to identify the owning boundary, applicable invariants, and affected consumers.

## Ramp up efficiently

Start from the repository root.

1. Inspect `git status`, the branch, and recent commits. Preserve unrelated work.
2. Read `README.md`, `STATUS.md`, `CONTRIBUTING.md`, and `docs/architecture/overview.md`.
3. Load only the task-specific documents below. Treat `roadmap.md`, `docs/plans/`, and handoffs as intent until implemented.
4. Trace the relevant type, validation, execution, persistence, RPC, and tests before editing. Search by protocol type, serialized field, receipt code, state field, or domain constant.

## Route by risk

- For documentation, tooling, RPC presentation, or client work that does not change signed bytes or state semantics, stay in the owning surface and its tests.
- For consensus, authorization, execution, storage, or replay work, load the relevant map row and apply the consensus-critical checklist.
- For NAVCoin, settlement-lane, governance, or shielded-proof work, also apply that domain's hard rules below.
- If active source, tests, and versioned architecture documents disagree on the touched behavior, stop and surface the discrepancy. Do not resolve it through an incidental code change.

Safety rules outrank completeness rules, which outrank engineering conventions. Change a safety rule only behind an explicit protocol version or activation boundary.

## Repository map

Consult only the rows relevant to the task.

| Area | Start here | Primary crates |
| --- | --- | --- |
| Types and canonical identities | domain module under `crates/types/src/` | `types` |
| Signatures, hashes, authorization | `crates/crypto_provider/src/lib.rs` | `crypto_provider` |
| State transitions and receipts | `docs/architecture/transaction-lifecycle.md` | `execution`, `types` |
| Consensus-v2 finality | `docs/architecture/finality.md` | `ordering_fast`, `node` |
| State roots, snapshots, replay | `docs/architecture/state-and-storage.md` | `storage`, `node` |
| FastPay and FastSwap | `docs/architecture/settlement-lanes.md` | `fastpay-prototype`, `fastswap_model`, `execution`, `storage`, `node` |
| Validator governance | `docs/governance/deterministic-governance-overview.md` | `consensus_cobalt`, `node` |
| NAVCoins and reserves | `docs/navcoins/index.md`, `docs/navcoins/reserve-primitives.md` | `types`, `execution`, `node`, `rpc_sdk` |
| Shielded execution | `docs/privacy/overview.md`, `docs/privacy/orchard-halo2.md` | `privacy`, `privacy_orchard` |
| Proof boundaries | `docs/privacy/orchard-halo2.md`, `docs/security/halo2-dependency.md` | `proofs`, `privacy_orchard`, `execution` |
| Mempool and transport | nearest module and tests | `mempool_dag`, `network`, `node` |
| Client behavior | protocol type and server handler first | `rpc_sdk`, `node`, `wallet_wasm` |
| Capability claims | `docs/architecture/evidence-model.md`, `docs/evidence/index.md` | scripts, tests, evidence |

Treat `crates/node` as orchestration. Put reusable protocol rules in their owning crate; keep node code focused on composition, persistence, transport, and RPC.

## Apply the relevant hard invariants

### Consensus and finality

- **Success:** Require both a valid block certificate and a matching accepted receipt. A committed block can contain a rejected transaction.
- **Finality:** Preserve the active prepare/precommit flow, durable locks, persisted signer-safety state, deterministic proposer rotation, and signed timeout certificates. Do not reinterpret it as chained HotStuff.
- **Quorum:** Use the shared quorum functions; do not reimplement thresholds inline. Count distinct verified validator identities, not raw signatures.

Never infer success from convergence, a certificate alone, an RPC 200, or a generated report. Verify the lane's complete success condition.

### Settlement and governance

- **Settlement lanes:** Keep FastPay and FastSwap object certificates separate from block consensus. Preserve each lane's authorization, terminal proof, recovery, and durability conditions.
- **Atomic swaps:** Execute both legs in one authorized transition or reject without partial mutation.
- **Governance:** Require distinct ML-DSA-65 authorizations from the active validator registry plus consensus ordering for live mutation. Treat Cobalt or model-generated artifacts as decision support unless explicitly activated.

### NAVCoin accounting

Treat NAVCoins as floating-net-asset-value issued assets, not fixed-peg stablecoins. Their safety depends on proof profiles, reserve packets, finalized epochs, supply discipline, and bounded mint, redeem, bridge, and market operations.

- Preserve the backing condition: verified net assets must cover valid global supply at the profile's NAV floor. Make integer units, rounding, haircuts, and valuation policy explicit.
- Bind each reserve packet to its asset, epoch, profile, supply, net assets, source root, attestor root, policy, and evidence.
- Enforce freshness, challenge, liveness, and settlement-deadline rules before finalization or value movement.
- Keep external fetches outside consensus. Normalize observations into deterministic roots, then require the configured registered-attestor quorum with no failing verdicts.
- Treat reserve proofs as evidence about the disclosed perimeter, not proof against undisclosed liabilities, custody failure, or source credit risk.
- Keep source-labeled bridged assets on the generic vault-bridge receipt path. Never encode named assets, venues, tickers, or providers as consensus branches.
- Recheck conservation across native supply, bridge representations, inventory, redemptions, and shielded boundaries.

Trace NAV work through `crates/types/src/core_chain.rs`, `market_nav_asset_types.rs`, execution, node/RPC workflows, SDK validation, Python builders, and the nearest NAVCoin smoke or replay test.

### Proof boundaries

- For shielded value, use upstream Orchard/Halo2 through the PostFiat adapter. Preserve proof, anchor, nullifier, authorization, conservation, public/private value turnstile, and resource bounds.
- Treat `crates/proofs` artifacts as gated controlled-testnet debug proofs, never production validity evidence.
- Route NAV reserve proofs through the profile-selected verifier, including bounded SP1 Groth16 verification.
- Route Asset-Orchard through the real upstream Orchard/Halo2 verifier; preserve the pinned vendored source and compatibility-patch boundary.
- Preserve proof-system and circuit IDs, ordered public inputs, canonical statement hashes, domains, policies, size limits, and verifying-key identity.
- Validate public inputs independently. Bind verified output to the exact consuming state transition and fail closed on any mismatch.
- Never substitute a hash, quorum, attestation, fixture, debug adapter, or report for a cryptographic proof.

Run `scripts/test-proof-public-input-inventory` for proof-surface changes. For Halo2-boundary changes, also run `scripts/verify-vendored-halo2` and obtain cryptography review.

### Any consensus-critical surface

For signed bytes, IDs, votes, certificates, execution, receipts, roots, snapshots, validator sets, or replay:

- **Determinism:** Exclude unordered iteration, floating point, local time, randomness, environment state, and locale-dependent formatting from consensus paths.
- **Encoding:** Use canonical encoders and domain separators. Bind every required chain, genesis, protocol, committee, height/view, parent, payload, state-root, phase, policy, and replay field.
- **Bounds:** Validate schemas, lengths, counts, identities, arithmetic, and resource limits before allocation, cryptography, mutation, or persistence.
- **Signer safety:** Persist vote high-water marks, locks, timeouts, and related safety state before returning signatures.
- **Atomicity:** Reject without partial changes to balances, versions, nullifiers, roots, or replay protection except where the protocol explicitly specifies an effect.
- **Failure policy:** Fail closed on unknown versions, missing ancestry, unresolved certificates, stale committees, malformed history, storage errors, or ambiguous compatibility. Do not panic on untrusted input.
- **Evolution:** Preserve deterministic replay. Introduce explicit versions or activation boundaries before changing signed bytes or state interpretation.

Inspect every consumer of the changed type or field before editing.

## Complete the owning boundary

Trace applicable changes through canonical types and validation; authorization and verification; deterministic execution and receipts; commitments, storage, snapshots, migrations, and replay; then node, transport, RPC, SDK, wallet, and operator consumers. Add owner tests, integration coverage at the first real boundary, and the nearest architecture or status update.

Repair the owner. Do not hide a state-transition defect in RPC output or compensate for invalid types in orchestration.

## Follow engineering conventions

These rules improve maintainability but never justify weakening a protocol invariant.

- Use Rust 2021 and the toolchain pinned in `rust-toolchain.toml`; keep `rustfmt` and Clippy `-D warnings` clean.
- Prefer explicit domain types, established validators, canonicalizers, receipts, storage patterns, and contextual errors over string branching or panics.
- Add scenario-named tests for the behavior class and adjacent invalid cases, not only the reported input.
- Never create or grow a source file beyond 5,000 lines. Extract a cohesive module before adding substantial behavior near the ceiling.
- Prefer narrow domain modules over unrelated additions to `lib.rs`, `main.rs`, RPC dispatchers, or omnibus tests. Do not mass-refactor unrelated large files.
- Keep generated artifacts, node data, keys, wallet backups, proofs, private notes, captures, and one-off reports out of git.

## Test in proportion to risk

Run focused tests while iterating, then expand:

```bash
cargo test -p <owning-package> <focused_test_name> --locked
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

`scripts/check` does not replace workspace tests. Add applicable coverage for wrong domains or roots; duplicate, stale, replayed, conflicting, truncated, or oversized inputs; rejection without partial mutation; persistence/restart/replay; conservation; rejected receipts in accepted blocks; and lane-specific cancellation, late-certificate, nullifier, or version-fence behavior.

Use devnet, WAN, fork, or secret-backed gates only when required. Never present an offline, skipped, or controlled-environment pass as production evidence.

## Maintain evidence discipline

Tie protocol, security, performance, and operational claims to stable code, tests, scripts, runbooks, or redaction-safe artifacts. In the handoff, state the changed invariant, affected surfaces, commands run, tests omitted, and remaining deployment or audit boundaries. Do not claim mainnet readiness; follow `README.md`, `STATUS.md`, and `SECURITY.md`.
