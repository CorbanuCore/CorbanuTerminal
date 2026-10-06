---
title: "Private inference user flow for Corbanu (PF) Terminal"
document_type: implementation_spec
status: draft
change_class: product-initiative
created: 2026-10-05
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Product principles"
  requirement_excerpt: "Private means explicit: Distinguish Corbanu-controlled inference from third-party inference at selection and use."
research_plan: docs/plans/proposed/portfolio-private-inference-research.md
feature: PF-65
tasknode_task: task_96e88f7099fd906d9b8080775ab6c402
---

# Private inference user flow

**Status: draft proposal. Nothing here is shipped.** This spec turns the
Ambient–Post Fiat private-inference thesis into a buildable product contract:
what a retail trader sees and does in Corbanu Terminal (formerly PF Terminal;
the `pfterminal` command and state remain compatible) when requesting private
inference. It has three sections that an engineer can implement separately:

- [A. User flow](#a-user-flow): triggers, tier choice, what comes back.
- [B. Payment flow](#b-payment-flow): pricing, reservation and settlement in PFT.
- [C. Verification surface](#c-verification-surface): what accompanies each
  response and how a user verifies it on a CPU.

Implementation is a **product initiative** under `AGENTS.md`: it needs an
active plan and a ready sprint, and the gates in [Open decisions](#open-decisions-and-dependencies)
must be resolved first. The privacy claims below are **provisional** until the
PF-65 threat model ([S01](../sprints/current/portfolio-private-inference-research/pf-65-s01-threat-model-and-evidence-review.md))
names an adversary and evidence for each one. Where this spec states a privacy
property, it also states the adversary it is meant to hold against.

## 0. Grounding: what ships today and what this spec adds

| Surface (shipping or candidate) | What it does today | How this spec uses it |
| --- | --- | --- |
| `/providers`, `/model` ([doc](../features/model-providers.md)) | Authenticates routes; selects provider, model, effort. Never silently switches providers. | Tier picker lives inside `/model`; the "no silent switch" rule becomes "no silent downgrade". |
| Ambient provider ([doc](../integrations/ambient.md)) | Built-in `ambient` route to `https://api.ambient.xyz/v1`, default model `zai-org/GLM-5.2-FP8`, key in vault at `provider/ambient_api_key`. | L2/L3 backend. The product spec labels Ambient routes "Private, Corbanu-controlled inference". |
| Corbanu API ([doc](../features/wallet-plan.md), integration candidate) | Wallet-funded USD balance in micro-USD with `balance`/`reserved`/`available`; per-model versioned prices; each model carries `privacy: "corbanu-controlled" \| "third-party"` (`codex-rs/wallet/src/corbanu_api.rs`), rendered in `/wallet` (`codex-rs/tui/src/chatwidget/wallet_api.rs`). | The account, API key, reservation/settlement ledger and privacy label are reused. PFT becomes a second funding/settlement currency. |
| Corbanu Plan | LIVE, **deprecation target**; replaced by Corbanu API. | Not used. No tiers are sold as plans. |
| `/gpu` rentals on Vast.ai / RunPod ([doc](../features/gpu-rentals.md)) | User-paid (USD, to the marketplace) rentals with price/spend caps; recipes pin image `sha256` digests, model commit revisions and weight `sha256` checks (`codex-rs/gpu-market/src/recipe.rs`, `gguf_recipes.rs`); readiness requires HTTPS or loopback and rejects missing/wrong bearer tokens (`readiness.rs`); ready endpoints register as `gpu-*` providers. | Optional "Your rented GPU" L2 route, and the optional deep-audit path for L3 (C.6). |
| `/vault` ([doc](../features/vault.md)) | Encrypted service credentials, label-only references, masked entry. | Holds the Corbanu API key and any Ambient key. Never holds PFT wallet keys (generic vault refuses keys/seeds). |
| `/tasknode balance` ([doc](../features/tasknode.md)) | **Read-only** linked-wallet PFT balance. Terminal cannot sign PFT transfers. | Funding source shown in the deposit screen. Signing stays outside Terminal in phase 1. |

**Not shipped anywhere today:** tiers, PFT payment, inference receipts, proof
bundles, a verifier, and any zero-data-retention (ZDR) attestation. These are
the build scope of this spec.

## 1. The three tiers

UI labels use "Tier" plus a short name so they do not collide with "L1" as in
"layer-1 blockchain". The internal enum keeps the task's L1/L2/L3 names.

| | **L1 · Aggregated** | **L2 · Zero retention** | **L3 · Verified** |
| --- | --- | --- | --- |
| Enum | `tier_l1_aggregated` | `tier_l2_zdr` | `tier_l3_verified` |
| Backend | Corbanu API routes labeled `third-party` (aggregator gateways such as the Vercel/xAPI routes named in **Corbanu API — TO BUILD**) | Corbanu API routes labeled `corbanu-controlled` (Ambient GLM); optionally the user's own `/gpu` rental | Ambient route with Proof of Logits (PoL) artifacts returned per response |
| Who can read the prompt (adversary model) | Corbanu gateway, aggregator, and upstream model vendor, each under its own retention policy | Corbanu gateway and the serving operator in memory during the request; nothing persisted by contract/config | Same as L2. Verification adds integrity, **not** confidentiality |
| Retention | Vendor policy; Terminal shows "may be logged by third party" | Corbanu policy `zdr-v1`: no prompt/response bodies persisted; metering keeps only token counts and commitments | Same as L2; only salted hashes ever leave the request path (C.3) |
| What returns with the response | Signed receipt (C.2) | Signed receipt with `retention_policy` and `operator_class` | Signed receipt plus PoL proof bundle (C.3) |
| Integrity claim the user can check | Charge and content commitments match what Terminal sent and received | Same, plus Corbanu's signed retention statement (a promise, not a proof) | Same, plus "network validators accepted this output as produced by model weights digest *D*" |
| Default for | General chat, public-data research | Strategy, positions, PnL, any workspace marked sensitive | Results the trader wants to cite, audit or back-test from, and agent decisions that feed a protected action |
| Relative price | Lowest | Middle | Highest (adds a per-request verification fee) |

Rules:

1. **Tier is a floor, not a hint.** A request runs at its tier or higher, or it
   does not run. There is no automatic fallback to a lower tier (matching the
   provider rule "It never silently switches providers").
2. **L2 with a rented GPU** is labeled `operator_class: user-rented`. Terminal
   states that the Vast.ai/RunPod host can observe GPU memory and that rental
   costs are billed in USD by the marketplace, not in PFT. These requests do not
   touch the PFT ledger and get a local-only receipt (no Corbanu signature).
3. **"Private" wording** appears only on L2/L3, and only together with the
   adversary sentence from the row above. Before PF-65 S01 is accepted, the UI
   text is "Zero retention (policy)" rather than "Private".

## A. User flow

### A.1 Triggers

A private-inference request is any model turn whose effective tier is L2 or
L3, or any explicit L1 request through the tiered catalog. Four entry points
set the effective tier; the highest one wins.

| # | Trigger | Who/what sets it | Persistence |
| --- | --- | --- | --- |
| T1 | `/model` → **Corbanu Private Inference** → pick a model row with a tier badge | User | Persists like any model selection (profile-scoped) |
| T2 | `/tier [l1\|l2\|l3]` | User | Current thread; shown in the status line |
| T3 | Workspace floor: `private_inference.min_tier = "l2"` in `config.toml`, or a `[[private_inference.sensitive_paths]]` glob (e.g. `strategies/**`, `*.positions.csv`) whose files enter context | Config, evaluated deterministically by Terminal | Workspace |
| T4 | Per-turn escalation: user prefixes a message with `!l3` | User | That turn only |

T3 is evaluated by Terminal code before the request is built, never by the
model. Child agents (`/spawn`, subagents) inherit the parent's effective floor
and cannot lower it. A turn whose effective tier is above the selected model's
tier is **blocked** (A.4, B4), not rerouted.

### A.2 Screens

**Tier picker** (`/model` → Corbanu Private Inference, or `/tier` without args):

```text
 Private inference · balance 412.50 PFT available (12.00 reserved)
 ─────────────────────────────────────────────────────────────────────
 › L3 · Verified        GLM 5.2        in 0.80 · out 3.20 PFT/M · +0.02/req
                        Ambient · zero retention · proof of logits
   L2 · Zero retention  GLM 5.2        in 0.50 · out 2.00 PFT/M
                        Corbanu-controlled · no prompt storage
   L2 · Your GPU        gpu-7f3a       billed by RunPod · host can see memory
   L1 · Aggregated      DeepSeek V4    in 0.10 · out 0.40 PFT/M
                        Third-party inference · may be logged by vendor
 ─────────────────────────────────────────────────────────────────────
 Workspace floor: L2 (strategies/**)  ·  enter select · p prices · d details
```

Prices are illustrative placeholders; real values come from the versioned
schedule (B.2). Rows below the current floor are shown dimmed and disabled with
the reason "below workspace floor L2".

**Status line** during a session: `GLM 5.2 · L3 verified · 412.50 PFT`.

**In-flight**: the existing turn spinner plus `reserved 0.0346 PFT`.

**Response footer** (one line under each assistant message):

```text
 L3 · verified ✓ · 0.0231 PFT · rcpt_01J9Z… · /receipt to inspect
```

Footer states, in order of precedence: `verifying…` (proof not final yet),
`verified ✓`, `receipt ok` (L1/L2: receipt checks pass), `receipt only
(unsigned)` (user GPU), `verification FAILED ✗` (red; see A.4).

**Receipt view** (`/receipt` for the last turn, `/receipt <id>`, or `r` on a
footer): tier, model and weights digest, route operator class, retention policy,
token counts, price schedule version, charge, each verification check with
pass/fail, and actions **Verify again**, **Export bundle**
(`<id>.cpr.json`), **Copy receipt ID**.

### A.3 Happy path (L3)

1. Trader selects GLM 5.2 · L3 in `/model` (T1). Terminal checks: Corbanu API
   key present in vault, PFT balance > 0, tier catalog reachable.
2. Trader types a question about a strategy file. T3 also requires ≥ L2;
   L3 satisfies it.
3. Terminal builds the request, generates a 32-byte client salt, and asks the
   gateway for a quote and reservation (B.3). The reservation uses the turn's
   output-token cap (the model default when none is set).
4. On `reserved`, Terminal streams the response normally. Tool calls work as
   they do on the `ambient` route today.
5. On completion the gateway returns the signed receipt; the PoL bundle follows
   once Ambient finalizes it (seconds to minutes). The footer shows
   `verifying…`.
6. Terminal's built-in verifier (C.5) runs locally on CPU when the bundle
   arrives and flips the footer to `verified ✓`. The receipt and bundle are
   stored in local state.
7. The settled charge appears in the footer and in `/wallet` → Corbanu API →
   **Private inference usage**.

L2 is the same flow without steps 5–6's PoL bundle; the footer shows
`receipt ok`. L1 is the same as L2 with the third-party label.

### A.4 Failure and edge states

| State | What the user sees | What Terminal does |
| --- | --- | --- |
| No Corbanu API key | "Private inference needs a Corbanu API key." → **Open /wallet** | Turn not sent |
| Insufficient PFT for the reservation | "Needs 0.0346 PFT, 0.0120 available." → **Deposit PFT** / **Lower max output** / **Cancel** | Turn not sent; draft kept |
| Selected tier unavailable (route down) | "L3 is unavailable right now." → **Retry** / **Choose another model at L3 or higher** / **Cancel** | Never offers a lower tier unless the user explicitly changes `/tier` *and* no floor (T3) forbids it |
| Turn needs a higher tier than the selected model (T3/T4) | "This turn includes `strategies/alpha.py` (floor L2). Current model is L1." → **Switch to an L2+ model** / **Remove the file from context** / **Cancel** | Blocked before any bytes leave the host |
| Stream interrupted / user cancels | Partial response kept, footer "cancelled · charged 0.0221 PFT for 210 output tokens" | Settles actual usage (B.3) |
| Proof not final after 10 minutes | Footer `verification pending`; `/receipt` shows **Verify again** | Background retry with backoff for 24 h, then `unavailable` |
| Verification fails | Red footer `verification FAILED ✗ (check C5-4)`; the message is marked untrusted for protected actions | Automatic refund request (B.4); the receipt is kept as evidence |
| Settlement ambiguous (network error after completion) | "Charge pending confirmation." | Re-reads the receipt by idempotency key; never re-sends the prompt to "fix" billing |

## B. Payment flow

### B.1 Account and balance

- PFT is a **second currency on the existing Corbanu API account** (same
  wallet-owned account and API keys). The ledger holds integer **micro-PFT**
  (`1 PFT = 1_000_000 µPFT`) with `balance`, `reserved` and `available`,
  mirroring `CorbanuApiBalance`'s micro-USD fields.
- L1/L2/L3 requests are priced and settled in PFT. The existing USD balance
  keeps paying for non-tiered Corbanu API models; the two balances never
  convert into each other automatically.

### B.2 Pricing

A price schedule is a signed, versioned document:

```json
{
  "schedule_id": "pi-2026-10-v1",
  "effective_from": "2026-11-01T00:00:00Z",
  "currency": "PFT",
  "unit": "micro_pft_per_million_tokens",
  "routes": [
    {
      "tier": "tier_l3_verified",
      "model": "corbanu/glm-5.2",
      "input": 800000, "cached_input": 80000, "output": 3200000,
      "per_request": 20000
    }
  ],
  "usd_reference": { "pft_usd": "0.0000", "source": "TBD", "as_of": "…" },
  "signature": "ed25519:…"
}
```

- Charge (µPFT) = `ceil((input·r_in + cached·r_cached + output·r_out) / 10^6) + per_request`,
  where rates are µPFT per million tokens. All arithmetic is integer; the
  result is rounded up once per request.
- L3's `per_request` fee covers verification overhead. L1 and L2 have
  `per_request = 0` unless the schedule says otherwise.
- `usd_reference` is display-only ("≈ $0.04") and is omitted if no approved
  source exists. It never changes a charge.
- A request pins the `schedule_id` in force when its reservation is created. A
  new schedule never reprices in-flight or settled requests.
- The actual numbers are an open decision (see Open decisions).

**Worked example** (placeholder L3 rates above; 1,840 input tokens, output cap
4,096, 512 output tokens generated):

| Step | Calculation | µPFT | PFT |
| --- | --- | ---: | ---: |
| Reserve | ceil((1840·800000 + 4096·3200000)/10^6) + 20000 = 14580 + 20000 | 34,580 | 0.0346 |
| Settle | ceil((1840·800000 + 512·3200000)/10^6) + 20000 = 3111 + 20000 | 23,111 | 0.0231 |
| Released | 34,580 − 23,111 | 11,469 | 0.0115 |

The same numbers are used in the screen examples in section A.

### B.3 Funding, reservation and settlement

**Funding (phase 1: deposit from any Post Fiat wallet, no Terminal signing).**

1. `/wallet` → Corbanu API → **Deposit PFT** shows the Corbanu deposit
   address, the account's **deposit tag** (a memo/destination tag binding the
   deposit to this account), and the linked Task Node wallet's PFT balance from
   `/tasknode balance` as a convenience.
2. The user sends PFT from their own wallet, including the tag.
3. The gateway watches the chain and credits the account after finality
   (confirmation rule depends on the settlement chain; see Open decisions).
   Terminal shows `pending → credited`, with the transaction hash.
4. A deposit without a valid tag is held, not credited; support can refund it
   to the sending address.

Phase 2 (separate initiative) adds in-Terminal signing once the wallet daemon
supports Post Fiat keys. Terminal's wallet signs Solana transactions only.

**Per-request lifecycle (server-authoritative, atomic, idempotent).**

```text
quote ──► reserve ──► run ──► settle ──► (L3) proof attached
             │          │         └─► release unused reservation
             │          └─► fail before first token ─► release all
             └─► insufficient ─► 402 + shortfall (no run)
```

1. **Reserve**: `reserve = charge(input_tokens_estimate, 0 cached,
   output_cap)` (the charge already includes `per_request`). The gateway atomically moves `reserve`
   from `available` to `reserved` under `Idempotency-Key` = Terminal-generated
   request ID. A repeated key returns the original outcome.
2. **Run** at the pinned tier. Any route change across tiers is a hard error.
3. **Settle** on completion or cancellation: debit the actual charge (actual
   input, cached and generated output tokens), release the rest. A failure
   before the first output token releases the full reservation.
4. **Receipt**: the settled charge, token counts and `schedule_id` are written
   into the signed receipt (C.2), so the user can check their bill per request.

### B.4 Refunds, statements and withdrawal

- **Verification failure (L3)**: when the user's verifier reports a failed
  check, Terminal submits the receipt to `POST /v1/private-inference/disputes`.
  If the gateway's own check confirms the failure, the full charge is refunded
  to `available`. A proof that is still `unavailable` after 24 hours is
  refunded down to the L2 price for the same tokens. Automatic disputes cover
  checks the gateway can re-run without plaintext (C5-3 to C5-6, C5-8). A
  `commitment_mismatch` (C5-1) dispute requires the user to choose to reveal
  the salt and transcript; Terminal never sends them by default.
- **Daily statement**: the gateway publishes one signed statement per account
  per UTC day: opening/closing balance and a Merkle root over that day's
  receipt hashes. Terminal checks that every locally stored receipt is included
  (C.5 step 7). This lets the user audit billing without trusting the
  dashboard.
- **Withdrawal**: `/wallet` → Corbanu API → **Withdraw PFT** returns
  `available` PFT to a previously used deposit-origin address. It requires a
  fresh wallet-ownership challenge, as key creation does today.

### B.5 Proposed API (Corbanu API gateway, `https://api.corbanu.com`)

| Method and path | Purpose |
| --- | --- |
| `GET /v1/private-inference/catalog` | Tiers, models, current signed price schedule, receipt signing public keys |
| `GET /v1/pft/account` | µPFT `balance`/`reserved`/`available`, deposit address and tag, pending deposits |
| `POST /v1/private-inference/quote` | Body: tier, model, token estimates. Returns reservation amount and `schedule_id` (no hold) |
| `POST /v1/chat/completions` | Existing endpoint plus headers `X-Corbanu-Tier`, `X-Corbanu-Client-Salt`, `Idempotency-Key`. Reserve, run and settle happen server-side. Insufficient funds → `402` with `{shortfall_micro_pft}` |
| `GET /v1/private-inference/receipts/{id}` | Signed receipt; `proof` field filled in when final |
| `GET /v1/private-inference/statements/{date}` | Signed daily statement |
| `POST /v1/private-inference/disputes` | Submit a failed-verification receipt |
| `POST /v1/pft/withdrawals` | Wallet-challenge-authorized withdrawal |

Terminal changes (proposed boundaries, to be fixed by the implementing sprint):

| Area | Path | Change |
| --- | --- | --- |
| API types | `codex-rs/wallet/src/corbanu_api.rs` | `PftBalance`, `PrivateInferenceTier`, `PriceSchedule`, `InferenceReceipt` |
| Request shaping | `codex-rs/core/src/client.rs` | Tier, salt and idempotency headers on tiered routes; reject cross-tier routing |
| Receipts and verifier | new crate `codex-rs/private-inference/` | Canonicalization, commitments, signature and Merkle checks; **no network I/O in the offline verifier** |
| Local storage | `codex-rs/state/` (pattern of `gpu_rentals.rs`) | `inference_receipts` table keyed by thread/turn/receipt ID |
| TUI | `codex-rs/tui/src/chatwidget/model_popups.rs`, `wallet_api.rs`, new `/tier` and `/receipt` commands | Picker badges, footer, receipt view, deposit/withdraw screens |
| CLI | `codex-rs/cli/` | `corbanu receipt verify <id\|file> [--json]` |

## C. Verification surface

### C.1 What every tier gets

Each response produces a **receipt**. L3 adds a **proof bundle**. Both are
plain JSON, stored locally, exportable together as `<receipt_id>.cpr.json`,
and verifiable offline by `corbanu receipt verify` on any CPU.

### C.2 Receipt (all tiers)

```json
{
  "receipt_id": "rcpt_01J9Z…",
  "version": 1,
  "account_id_hash": "sha256:…",
  "tier": "tier_l3_verified",
  "model": "corbanu/glm-5.2",
  "weights_digest": "sha256:…",
  "operator_class": "corbanu-controlled",
  "retention_policy": "zdr-v1",
  "request_commitment": "sha256:…",
  "response_commitment": "sha256:…",
  "tokens": { "input": 1840, "cached_input": 0, "output": 512 },
  "schedule_id": "pi-2026-10-v1",
  "charge_micro_pft": 23111,
  "started_at": "…", "completed_at": "…",
  "proof": null,
  "signing_key_id": "corbanu-receipts-2026-10",
  "signature": "ed25519:…"
}
```

- `request_commitment = SHA-256(salt ‖ JCS(request))` and
  `response_commitment = SHA-256(salt ‖ JCS(response))`, where JCS is RFC 8785
  JSON canonicalization and the response includes text and tool calls in order.
  The 32-byte salt is generated by Terminal per request and sent in
  `X-Corbanu-Client-Salt`. It is stored only locally and in the gateway's
  memory for that request, so published hashes cannot be brute-forced back to
  short prompts.
- The signature covers the JCS form of every field except `signature`.
  Signing keys are listed in the catalog; Terminal pins them on first use and
  warns when they change.
- `account_id_hash` lets statements prove inclusion without publishing the
  account ID.

### C.3 L3 proof bundle

Attached to `receipt.proof` once final:

```json
{
  "system": "ambient-proof-of-logits",
  "job_id": "…",
  "chain": "ambient-mainnet",
  "block_height": 0,
  "block_hash": "…",
  "job_inclusion_proof": ["…"],
  "logits_root": "sha256:…",
  "response_commitment": "sha256:…",
  "weights_digest": "sha256:…",
  "spot_checks": [
    {
      "position": 347,
      "logits_hash": "sha256:…",
      "merkle_path": ["…"],
      "validator_id": "…",
      "validator_signature": "…"
    }
  ]
}
```

- Only hashes and salted commitments are published on-chain or in the bundle;
  no prompt or response plaintext.
- The exact field set depends on what Ambient exposes per request. If Ambient
  cannot bind its job record to `response_commitment`, L3 does not launch
  (dependency D1).

### C.4 What a passing check proves (and does not)

| Check passes | Proves | Does not prove |
| --- | --- | --- |
| Commitments (all tiers) | The receipt covers exactly what Terminal sent and received | Anything about who saw it |
| Corbanu signature (all tiers) | Corbanu attests to tier, route class, retention policy and charge | That the retention promise was kept (L2/L3 privacy remains a trust-in-policy claim) |
| PoL bundle (L3) | Ambient validators accepted that weights `weights_digest` produced this output, within the protocol's floating-point tolerance and spot-check probability | Confidentiality, or the correctness of the answer itself |

The receipt view shows the right-hand column next to each check, so no user
mistakes "verified" for "private".

### C.5 CPU verification procedure

`corbanu receipt verify` and the TUI run the same steps. Steps 1–6 work offline
from the exported bundle plus pinned keys and a block header. Step 7 needs the
day's statement. Expected cost is well under one second on a laptop CPU; no GPU
or model weights are needed.

| Step | Check | Fails as |
| --- | --- | --- |
| C5-1 | Recompute both commitments from the locally stored transcript and salt; compare to the receipt | `commitment_mismatch` |
| C5-2 | Verify the Ed25519 receipt signature against a pinned Corbanu key | `bad_receipt_signature` |
| C5-3 | Recompute the charge from tokens and the signed `schedule_id`; compare to `charge_micro_pft` | `charge_mismatch` |
| C5-4 | (L3) `proof.response_commitment` and `proof.weights_digest` equal the receipt's | `proof_binding_mismatch` |
| C5-5 | (L3) Each spot check's `logits_hash` plus `merkle_path` hashes to `logits_root`; each validator signature verifies against the validator set at `block_height` | `bad_spot_check` |
| C5-6 | (L3) `job_inclusion_proof` places the job in `block_hash`; the header is finalized according to at least two independent Ambient RPC endpoints chosen by the user (configurable) | `not_finalized` / `inclusion_failed` |
| C5-7 | (All, daily) The receipt hash is included in the signed statement's Merkle root | `missing_from_statement` |
| C5-8 | `weights_digest` equals the digest published for that model in the signed catalog (and, where one exists, the matching `/gpu` recipe's pinned revision) | `unknown_weights` |

Output (`--json`): `{receipt_id, tier, result: "verified" | "receipt_ok" |
"pending" | "failed" | "unavailable", checks: [{id, result, detail}]}`.
Exit code is 0 only for `verified` (L3) or `receipt_ok` (L1/L2).

### C.6 Optional deep audit (not CPU; later phase)

For a user who wants to re-execute rather than trust validators: **Deep audit**
in the receipt view rents a GPU through the existing `/gpu` flow, with the
usual price/spend/duration caps and terminate controls. It uses a pinned recipe
whose model revision matches `weights_digest`, recomputes logits at the sampled
positions, and compares them to `spot_checks`. This needs the request
plaintext, which never leaves the user's own rental. Cost is billed by the
marketplace in USD.

## Acceptance tests for the implementing sprints

| Flow | Start / action | Pass criterion |
| --- | --- | --- |
| Success L3 | Funded account, L3 model, prompt with a strategy file | Response streams; footer reaches `verified ✓`; `corbanu receipt verify --json` → `verified`; ledger debit equals receipt charge |
| Success L2/L1 | Same at L2 and L1 | Footer `receipt ok`; L1 row and footer show "Third-party inference" |
| Floor enforcement | L1 model selected, file under `sensitive_paths` in context | Request blocked before network I/O (asserted by a fixture server receiving zero requests) |
| No downgrade | L3 route returns 503 | No request reaches any L2/L1 route; user offered retry/other L3+ model |
| Insufficient funds | `available` < reservation | `402` path shows shortfall; draft kept; no reservation remains |
| Cancel mid-stream | Cancel after N tokens | Charge = actual tokens; remaining reservation released; receipt issued |
| Idempotency | Replay the same `Idempotency-Key` after a simulated disconnect | One debit, same receipt ID |
| Tamper | Modify one response byte in local transcript | C5-1 fails; footer red; dispute submitted |
| Bad proof | Fixture bundle with wrong Merkle path | C5-5 fails; refund path exercised |
| Recovery | Restart Terminal while proof pending | Pending receipts resume verification from local state |
| Child agents | Parent at L2 floor spawns a subagent | Child requests carry L2+; child cannot select L1 |
| Secrets | Grep receipts, logs, statements, Task Node evidence | No API key, salt (outside local store), prompt or response plaintext |

The interactive flows also require the repository's true-TUI evidence before
release (`AGENTS.md` → Interactive product proof).

## Milestones

| Milestone | Scope | Depends on |
| --- | --- | --- |
| M0 | PF-65 S01 threat model accepted; open decisions OD1–OD4 resolved; plan activated | Travis / Alex |
| M1 | Receipts and verifier steps C5-1…C5-3 against a fixture gateway with no money movement: commitments, signature, local store, `/receipt`, `corbanu receipt verify` | M0 |
| M2 | PFT ledger: deposit tag, crediting, reserve/settle/release, statements, withdrawal; tier picker and floors | M1, OD1, OD2 |
| M3 | L3 proof bundle and verifier steps C5-4…C5-6; dispute/refund | M2, D1 |
| M4 | Deep audit via `/gpu` | M3 and a recipe matching the L3 model digest |

## Open decisions and dependencies

These need a human owner before implementation; none is decided by this spec.

| ID | Question | Owner | Why it blocks |
| --- | --- | --- | --- |
| OD1 | Settlement chain for PFT: where Task Node already pays PFT to the linked `r…` address, or PostFiat L1 (FastPay lane)? This fixes the deposit tag format and the finality rule. | Travis | B.3 funding and withdrawal |
| OD2 | PFT settlement versus the product spec, which says "USDAI as the preferred stablecoin partner rather than centering a Corbanu-native token" and funds Corbanu API in USDC. PFT is Post Fiat's token, not Corbanu's, but adding it is a product-spec amendment. | Alex Good (Head of Product) / Travis | Product authority (`AGENTS.md`) |
| OD3 | Price schedule numbers and whether a USD reference is shown | Alex Good | B.2 |
| OD4 | Refund policy for failed or unavailable proofs (B.4 proposes full refund / refund down to L2) | Alex Good | B.4 |
| D1 | Ambient exposes per-request job ID, logits root, spot checks and an inclusion proof bound to a client-supplied commitment | Ambient | L3 cannot launch without it |
| D2 | Ambient and Corbanu gateway retention configuration that substantiates `zdr-v1` | Ambient / Corbanu infra | L2/L3 labels |
| D3 | PF-65 S01 adversary model and S02 experiment results | Privacy research lead | Any "private" wording (rule 3) |

## Traceability to the Task Node task

| Task requirement | Where it is met |
| --- | --- |
| Concrete inference request flow: trigger, tier options, what comes back | A.1 triggers, A.2 screens, A.3 happy path, A.4 failures |
| Three tiers named, presented, priced and settled in PFT | §1 tier table; A.2 picker; B.2 pricing; B.3 reservation/settlement; B.4 refunds/withdrawal |
| Verification surface: proof/hash per response; CPU verification | C.2 receipt; C.3 PoL bundle; C.4 claim boundaries; C.5 CPU procedure |
| One document with distinct user, payment and verification sections | Sections A, B and C |
| Specific enough to begin implementation | B.5 endpoints and code boundaries; JSON schemas; acceptance tests; milestones |
