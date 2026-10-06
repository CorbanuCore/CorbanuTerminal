---
title: "Private inference user flow for Corbanu (PF) Terminal"
document_type: proposed_product_spec
status: draft
change_class: product-initiative
created: 2026-10-05
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Product principles"
  requirement_excerpt: "Private means explicit: Distinguish Corbanu-controlled inference from third-party inference at selection and use."
related_research_plan: docs/plans/proposed/portfolio-private-inference-research.md
related_feature: PF-65
tasknode_task: task_96e88f7099fd906d9b8080775ab6c402
---

# Private inference user flow

**Status: draft proposal. Nothing here is shipped, and this document does not
authorize implementation.** It turns the Ambient–Post Fiat private-inference
thesis into a buildable product contract: what a retail trader sees and does in
Corbanu Terminal (formerly PF Terminal; the `pfterminal` command and state
remain compatible) when requesting private inference. It has three sections
that can be implemented separately:

- [A. User flow](#a-user-flow): triggers, tier choice, what comes back.
- [B. Payment flow](#b-payment-flow): pricing, reservation and settlement in PFT.
- [C. Verification surface](#c-verification-surface): what accompanies each
  response and how a user verifies it on a CPU.

**Lifecycle.** The related plan, [PF-65](../plans/proposed/portfolio-private-inference-research.md),
is a draft research plan; its scope excludes production deployment and none of
its sprints is ready. This spec is an input to that research and to a later
decision, not an executable sprint. Building it is a **product initiative**
under `AGENTS.md`. It needs a new or amended active plan, a ready sprint and
answers to the [open decisions](#open-decisions-and-dependencies). OD1 (PFT
settlement chain) is decided: `xrpl_pft`. OD2 (product-spec amendment for PFT)
blocks the payment milestone. Terminal-side work that is independent of the chain (receipts,
verifier, tier picker against a fixture gateway) can start once a plan is
active.

**Privacy wording.** The privacy claims below are **provisional** until the
PF-65 threat model ([S01](../sprints/current/portfolio-private-inference-research/pf-65-s01-threat-model-and-evidence-review.md))
names an adversary and evidence for each. Every privacy property in this spec
names the adversary it is meant to hold against.

## 0. Grounding: what ships today and what this spec adds

| Surface (shipping or candidate) | What it does today | How this spec uses it |
| --- | --- | --- |
| `/providers`, `/model` ([doc](../features/model-providers.md)) | Authenticates routes and selects provider, model and effort. The doc states Corbanu "never silently switches providers". | Tiers appear inside `/model`. The no-silent-switch rule becomes "no silent tier change" (§1 rule 1). |
| Ambient provider ([doc](../integrations/ambient.md)) | Built-in `ambient` provider, `https://api.ambient.xyz/v1`, default model `z-ai/glm-5.2` (`AMBIENT_DEFAULT_MODEL` in `codex-rs/model-provider-info/src/lib.rs`; the integration doc still names the older `zai-org/GLM-5.2-FP8` slug). Uses the user's own `AMBIENT_API_KEY` (vault label `provider/ambient_api_key`). | **Unchanged and untiered.** The tiers use Corbanu's gateway, which holds Ambient credentials server-side (§1.1). |
| Corbanu API ([doc](../features/wallet-plan.md), integration candidate) | Wallet-funded USD balance in micro-USD (`balance`/`reserved`/`available`); per-model versioned prices for input, cache read, cache write and output; each model has `privacy: "corbanu-controlled" \| "third-party"` (`codex-rs/wallet/src/corbanu_api.rs`), shown in `/wallet` (`codex-rs/tui/src/chatwidget/wallet_api.rs`). Served through the wallet-linked `corbanu-plan` provider route; key from the vault or `CORBANU_API_KEY`. | The account, API keys, provider route, price fields, ledger semantics and privacy label are reused. PFT becomes a second balance on the same account. |
| Corbanu Plan | LIVE, **deprecation target**, being replaced by Corbanu API. | Not used. Tiers are not sold as plans. |
| `/gpu` rentals on Vast.ai / RunPod ([doc](../features/gpu-rentals.md)) | User-paid USD rentals with price/spend caps. Recipes pin image `sha256` digests, model commit revisions and weight `sha256` checks (`codex-rs/gpu-market/src/recipe.rs`, `gguf_recipes.rs`). Readiness requires HTTPS or loopback and rejects requests whose authorization header is missing or wrong (`readiness.rs`). Ready endpoints register as `gpu-*` providers. | A separate **self-hosted mode** outside the three tiers (§1.2), and the optional L3 deep audit (C.7). |
| `/vault` ([doc](../features/vault.md)) | Encrypted service credentials, label-only references, masked entry. Refuses seed phrases and private keys. | Holds the Corbanu API key. Never holds PFT wallet keys. |
| `/tasknode balance` ([doc](../features/tasknode.md)) | **Read-only** PFT balance of the Task Node–linked wallet. Terminal cannot sign PFT transfers. | Shown on the deposit screen. Phase 1 funding needs no Terminal signing. |

**Not shipped anywhere today:** tiers, PFT payment, inference receipts, proof
bundles, a verifier and any zero-data-retention (ZDR) attestation. These are
the build scope of this spec.

## 1. The three tiers

UI labels use "Tier" plus a short name so they do not collide with "L1" as in
"layer-1 blockchain". The internal enum keeps the task's L1/L2/L3 names.

| | **L1 · Aggregated** | **L2 · Zero retention** | **L3 · Verified** |
| --- | --- | --- | --- |
| Enum (ordered) | `l1_aggregated` = 1 | `l2_zdr` = 2 | `l3_verified` = 3 |
| Backend | Corbanu API routes labeled `third-party` (the aggregator routes named in **Corbanu API — TO BUILD**, e.g. Vercel/xAPI) | Corbanu API routes labeled `corbanu-controlled` (Ambient GLM) | Ambient route that returns Proof of Logits (PoL) artifacts per request |
| Who can read the prompt (adversary model) | Corbanu gateway, aggregator and upstream model vendor, each under its own retention policy | Corbanu gateway and the serving operator, in memory during the request; nothing persisted by policy | Same as L2. Verification adds integrity, **not** confidentiality |
| Retention | Vendor policy; Terminal shows "may be logged by third party" | Corbanu policy `zdr-v1`: no prompt or response bodies persisted; metering keeps token counts and salted commitments | Same as L2; only salted hashes leave the request path (C.3) |
| What returns with the response | Signed receipt (C.2) | Signed receipt with `retention_policy` and `operator_class` | Signed receipt, then a signed proof attestation (C.3) |
| Integrity claim the user can check | Charge and content commitments match what Terminal sent and received | Same, plus Corbanu's signed retention statement (a promise, not a proof) | Same, plus "Ambient validators accepted this output as produced by weights digest *D*" |
| Default for | General chat, public-data research | Strategy, positions, PnL, any workspace marked sensitive | Results the trader wants to cite, audit or backtest from; agent output that feeds a protected action |
| Relative price | Lowest | Middle | Highest (adds a per-request verification fee) |

### 1.1 Route and credential per tier

All three tiers use the **existing `corbanu-plan` provider route** and the
user's Corbanu API key. Ambient and aggregator credentials stay on the Corbanu
gateway, matching the product-spec rule that provider credentials "never enter
model context or customer responses".

| Tier | Provider ID (Terminal) | Proposed model IDs | Credential in Terminal | Upstream credential |
| --- | --- | --- | --- | --- |
| L1 | `corbanu-plan` | Existing Corbanu API catalog IDs whose `privacy` is `third-party` | Corbanu API key (vault, or `CORBANU_API_KEY`) | Gateway-held |
| L2 | `corbanu-plan` | `corbanu/glm-5.2` (proposed) | Same | Gateway-held Ambient key |
| L3 | `corbanu-plan` | `corbanu/glm-5.2-verified` (proposed) | Same | Gateway-held Ambient key |

A model's tier is a fixed catalog field (`tier`). The same weights at two tiers
are two catalog entries, so a selection always names exactly one tier.

### 1.2 Self-hosted mode (outside the tiers)

A ready `/gpu` rental (`gpu-*` provider) is shown in the picker as
**S · Your GPU**. It is not L1, L2 or L3:

- It is billed in USD by Vast.ai/RunPod, never through the PFT ledger.
- It gets a **local receipt** (C.2 fields without a Corbanu signature). The
  verifier result is `local_receipt`.
- The marketplace host can observe GPU memory. The picker shows that sentence.
- It satisfies a workspace floor of L2 only if
  `private_inference.self_hosted_satisfies = "l2"` is set in `config.toml`.
  By default it satisfies L1 only. It never satisfies L3.

The direct `ambient` provider (the user's own Ambient key) remains available
and is treated the same way for floors: it has no tier and satisfies L1 unless
the same setting names it.

### 1.3 Rules

1. **No silent tier change.** A request runs on the tier of the selected model
   or it does not run. Terminal never moves a request to a lower tier (privacy)
   or to a higher tier (cost) without an explicit user choice.
2. **"Private" wording** appears only on L2/L3 and always with the adversary
   sentence from the table. Until PF-65 S01 is accepted, the UI says
   "Zero retention (policy)", not "Private".

## A. User flow

### A.1 Triggers and tier resolution

A tiered request is any model turn sent to a model whose catalog entry has a
`tier`. The tier comes from the **model the user selected**. Other inputs only
set a **floor** (a minimum) that the selected model must meet.

| # | Input | Kind | Set by | Scope |
| --- | --- | --- | --- | --- |
| T1 | `/model` → **Private inference** → choose a model row | Selection (determines the tier) | User | Persists like any model selection (profile-scoped) |
| T2 | `/tier min <l1\|l2\|l3>` (`/tier` alone shows the current floor) | Floor | User | Current thread; shown in the status line |
| T3 | `private_inference.min_tier` in `config.toml`, or a `[[private_inference.sensitive_paths]]` glob (e.g. `strategies/**`, `*.positions.csv`) that matches a file entering context | Floor | Config, evaluated by Terminal code | Workspace |
| T4 | Message prefix `!l3` (or `!l2`) | Floor | User | That turn only |

Resolution, evaluated by Terminal before any bytes leave the host:

```text
floor   = max(T2, T3, T4)                   # unset inputs count as 0
tier    = selected_model.tier               # catalog models: 1, 2 or 3
        # self-hosted and direct Ambient (tier null): 2 if
        # self_hosted_satisfies = "l2", else 1; never 3
allowed = tier >= floor
```

- `allowed` → send. The turn runs at the selected model's tier, even if it is
  above the floor.
- not `allowed` → **blocked** (A.4, "below floor"). Terminal offers models that
  meet the floor, with their price delta. The user can switch for **this turn**
  or **from now on**. Nothing is upgraded without that choice.
- Child agents (`/spawn`, subagents) inherit the parent's floor and cannot
  lower it. Their own model selection is checked against it the same way.
- T3 is code, not model judgment. A model asserting that content is safe never
  lowers a floor.

**Tools.** The tier covers the model request and response only. Tools run in
Terminal under the existing permission and approval policy. Tiered models do
not enable provider-hosted tools. A tool that sends data to an external service
(web fetch, MCP server, connector) is outside the tier boundary. When the floor
is L2 or higher, that tool's approval prompt adds "sends data outside the L2
boundary" and such calls are never auto-approved by the tier itself.

### A.2 Screens

**Picker** (`/model` → Private inference):

```text
 Private inference · 412.5000 PFT available (12.0000 reserved)
 ──────────────────────────────────────────────────────────────────────────
 › L3 · Verified        GLM 5.2       in 0.80 · out 3.20 PFT/M · +0.02/req
                        Ambient · zero retention (policy) · proof of logits
   L2 · Zero retention  GLM 5.2       in 0.50 · out 2.00 PFT/M
                        Corbanu-controlled · no prompt storage (policy)
   L1 · Aggregated      DeepSeek V4   in 0.10 · out 0.40 PFT/M
                        Third-party inference · may be logged by vendor
   S  · Your GPU        gpu-7f3a      billed in USD by RunPod
                        Your rental · the host can see GPU memory
 ──────────────────────────────────────────────────────────────────────────
 Floor: L2 (strategies/**) · enter select · p full prices · d details
```

Prices are placeholders; real values come from the signed schedule (B.2).
Rows below the floor are dimmed and disabled with "below floor L2". `p` shows
all four rates (input, cache read, cache write, output) and the per-request fee.

**Status line:** `GLM 5.2 · L3 verified · 412.5000 PFT`.

**In flight:** the existing turn spinner plus `reserved 0.0349 PFT`.

**Response footer** (one line under each assistant message):

```text
 L3 · verified ✓ · 0.0231 PFT · rcpt_01J9Z… · /receipt to inspect
```

Footer states: `verifying…` (proof not final), `verified ✓` (L3),
`receipt ok` (L1/L2), `local receipt` (self-hosted), `verification pending`,
`verification FAILED ✗` (red; A.4).

**Receipt view** (`/receipt` for the last turn, `/receipt <id>`, or `r` on a
footer): tier, model, weights digest, operator class, retention policy, token
counts, price schedule, charge, and each check from C.6 with its result and
the "does not prove" text from C.5. Actions: **Verify again**, **Export
shareable bundle**, **Export full package** (warns that it contains the
transcript and salt), **Copy receipt ID**.

### A.3 Happy path (L3)

1. The trader selects GLM 5.2 · L3 in `/model` (T1). Terminal checks that a
   Corbanu API key is present, the PFT account is reachable and the catalog
   lists the model.
2. The trader asks about a strategy file. T3 sets floor L2; L3 meets it.
3. Terminal generates a request ID and a 32-byte salt, stores both with the
   canonical request in local state, and sends the request (B.5 headers).
4. The gateway counts input tokens on the request it received and reserves
   the worst-case charge (B.3). On success it streams the response. Tool calls
   work as on the existing `corbanu-plan` route.
5. At completion the gateway settles the charge and returns the signed
   **receipt** (C.2) in the final stream event. Terminal stores the canonical
   response and checks C5-1 to C5-3 immediately. The footer shows
   `verifying…`.
6. When Ambient finalizes the job (seconds to minutes), Terminal fetches the
   signed **proof attestation** (C.3), runs C5-4 to C5-7 and shows
   `verified ✓`.
7. The settled charge appears in the footer and in `/wallet` → Corbanu API →
   **Private inference usage**.

L2 and L1 stop after step 5 with `receipt ok`. L1 rows and footers also show
"Third-party inference".

### A.4 Failure and edge states

| State | What the user sees | What Terminal does |
| --- | --- | --- |
| No Corbanu API key | "Private inference needs a Corbanu API key." → **Open /wallet** | Turn not sent |
| Below floor (T2/T3/T4) | "This turn includes `strategies/alpha.py` (floor L2). DeepSeek V4 · L1 is below it." → **Use L2 model for this turn (+0.0016 PFT est.)** / **Switch model** / **Remove file from context** / **Cancel** | Blocked before any network I/O |
| Insufficient PFT (`402`) | "Needs 0.0349 PFT, 0.0120 available." → **Deposit PFT** / **Lower output cap** / **Cancel** | No reservation remains; draft kept |
| Tier route unavailable | "L3 is unavailable right now." → **Retry** / **Choose another L3 model** / **Cancel** | Never offers a model below the floor |
| Stream interrupted or cancelled | Partial response kept; footer "cancelled · 0.0221 PFT for 210 output tokens" | Gateway settles actual usage; receipt issued |
| Proof not final after 10 minutes | Footer `verification pending`; **Verify again** in `/receipt` | Background retry with backoff for 24 h, then `unavailable` |
| Verification fails | Red footer `verification FAILED ✗ (C5-5)`; the message is marked untrusted for protected actions | Dispute submitted (B.4); receipt kept as evidence |
| Settlement outcome unknown (disconnect) | "Charge pending confirmation." | Re-reads the receipt by request ID; never re-sends the prompt to resolve billing |

## B. Payment flow

### B.1 Account and balance

- PFT is a **second balance on the existing Corbanu API account** (same
  wallet-owned account and keys). The ledger stores integer **micro-PFT**
  (`1 PFT = 1_000_000 µPFT`) with `balance`, `reserved` and `available`,
  mirroring the micro-USD fields of `CorbanuApiBalance`.
- L1, L2 and L3 requests are priced and settled only in PFT. The USD balance
  keeps paying for untiered Corbanu API models. The two balances never convert
  automatically.

### B.2 Pricing

A price schedule is a signed, versioned document fetched from the catalog:

```json
{
  "schedule_id": "pi-2026-11-v1",
  "effective_from": "2026-11-01T00:00:00Z",
  "currency": "PFT",
  "unit": "micro_pft_per_million_tokens",
  "routes": [
    {
      "tier": "l3_verified",
      "model": "corbanu/glm-5.2-verified",
      "input": 800000,
      "cache_read": 80000,
      "cache_write": 1000000,
      "output": 3200000,
      "per_request": 20000
    }
  ],
  "usd_reference": null,
  "signing_key_id": "corbanu-schedules-2026-11",
  "signature": "ed25519:…"
}
```

- Token fields are disjoint, as in today's Corbanu API pricing: each input
  token is billed once as `input`, `cache_read` or `cache_write`.
- **Charge** (µPFT) =
  `ceil((input·r_in + cache_read·r_cr + cache_write·r_cw + output·r_out) / 10^6) + per_request`.
  All arithmetic is integer, rounded up once per request.
- L3's `per_request` fee covers verification. L1 and L2 use `per_request = 0`
  unless the schedule says otherwise.
- `usd_reference`, if an approved source exists, is display-only ("≈ $…") and
  never changes a charge.
- A request pins the schedule in force when it is reserved. A new schedule
  never reprices in-flight or settled requests.
- Real rates are open decision OD3.

### B.3 Funding, reservation and settlement

**Funding, phase 1 (deposit from any PFT wallet; no Terminal signing).**

1. `/wallet` → Corbanu API → **Deposit PFT** shows the Corbanu deposit address,
   the account's **deposit tag**, the linked Task Node wallet's balance (read
   from `/tasknode balance`), and a warning that deposits without the tag are
   not credited automatically.
2. The user sends PFT from their own wallet with the tag.
3. The gateway credits the account once the deposit is final. Terminal shows
   `pending → credited` with the transaction hash.
4. Untagged or malformed deposits are held and can be refunded to the sender
   through support.

OD1 is decided (Travis, 2026-10-06): settle on the ledger where Task Node pays
PFT today (`xrpl_pft`). The chain-specific parts sit behind one gateway
interface, so a later move to PostFiat L1 changes only an adapter:

```text
trait PftSettlementAdapter {
  deposit_instructions(account) -> { address, tag, asset_id, min_amount }
  observe_deposits(since_cursor) -> [{ tx_hash, tag, amount_micro_pft, final: bool }]
  send_withdrawal(destination, amount_micro_pft, idempotency_key) -> tx_hash
}
```

| Adapter | Use if OD1 picks | Tag | Finality and validity rule |
| --- | --- | --- | --- |
| `xrpl_pft` (**selected**, OD1) | The ledger where Task Node pays PFT to the linked `r…` address today | 32-bit `DestinationTag`, unique per account | Transaction in a validated ledger, result `tesSUCCESS`, currency and issuer equal to the canonical PFT asset. Credit `meta.delivered_amount`, never `Amount` (prevents partial-payment over-crediting) |
| `postfiat_l1` (possible later migration) | PostFiat L1 | Per-account deposit memo or a derived deposit address | Accepted receipt in a certified, finalized block. Same delivered-amount rule |

Phase 2 (separate initiative) adds in-Terminal signing once the wallet daemon
supports Post Fiat keys. Terminal's wallet signs Solana transactions only.

**Per-request lifecycle (server-authoritative, atomic, idempotent).**

```text
receive ─► count input ─► reserve ─► run ─► settle ─► receipt ─► (L3) proof
              │              │         └─► fail before first token ─► release all
              │              └─► insufficient ─► 402 {shortfall_micro_pft}
              └─► (gateway tokenizer, on the request actually received)
```

1. **Reserve** the worst case: every input token at
   `max(r_in, r_cr, r_cw)`, plus `output_cap · r_out`, plus `per_request`,
   rounded up. `output_cap` is the request's output-token limit (the model's
   catalog maximum when none is set). The gateway moves the reserve from
   `available` to `reserved` atomically, keyed by `Idempotency-Key` (the
   Terminal request ID). A repeated key returns the original outcome and never
   runs the model twice.
2. **Run** on the model's pinned tier. Any route change across tiers fails the
   request.
3. **Settle** at completion or cancellation: debit the actual charge (B.2) and
   release the rest. A failure before the first output token releases
   everything.
4. **Receipt**: token counts, `schedule_id` and the charge go into the signed
   receipt (C.2), so each charge can be checked.

**Worked example** (placeholder L3 rates above; 1,840 input tokens, no cache
hits or writes, output cap 4,096, 512 output tokens generated):

| Step | Calculation (µPFT) | µPFT | PFT |
| --- | --- | ---: | ---: |
| Reserve | ceil((1840·1000000 + 4096·3200000)/10^6) + 20000 = 14948 + 20000 | 34,948 | 0.0349 |
| Settle | ceil((1840·800000 + 512·3200000)/10^6) + 20000 = 3111 + 20000 | 23,111 | 0.0231 |
| Released | 34,948 − 23,111 | 11,837 | 0.0118 |
| Cancel after 210 output tokens | ceil((1840·800000 + 210·3200000)/10^6) + 20000 = 2144 + 20000 | 22,144 | 0.0221 |

The screens in section A use these numbers.

### B.4 Refunds, statements and withdrawal

- **Disputes.** When the verifier reports a failed check, Terminal submits the
  receipt ID and the failing check to `POST /v1/private-inference/disputes`.
  Disputes for checks the gateway can re-run without plaintext (C5-2 to C5-8)
  are automatic. If the gateway confirms the failure, the full charge returns
  to `available`. A `commitment_mismatch` (C5-1) dispute requires the user to
  choose to reveal the salt and transcript; Terminal never sends them by
  default. A proof still `unavailable` after 24 hours is refunded down to the
  L2 price for the same tokens. (Policy is open decision OD4.)
- **Daily statement.** The gateway publishes one signed statement per account
  per UTC day: opening and closing µPFT balance, and a Merkle root (SHA-256,
  sorted leaves) over that day's `receipt_hash` values. Terminal checks that
  each stored receipt is included (C5-8).
- **Withdrawal.** `/wallet` → Corbanu API → **Withdraw PFT** returns
  `available` PFT to an address that previously deposited to the account,
  after a fresh wallet-ownership challenge (as key creation uses today), through
  `send_withdrawal`.

### B.5 Proposed interfaces

Gateway (`https://api.corbanu.com`):

| Method and path | Purpose |
| --- | --- |
| `GET /v1/private-inference/catalog` | Models with `tier`, the current signed schedule, receipt and schedule signing keys |
| `GET /v1/pft/account` | µPFT `balance`/`reserved`/`available`, deposit address, tag and asset ID, pending deposits |
| `POST /v1/chat/completions` | Existing endpoint. Tiered models add headers `X-Corbanu-Tier` (must equal the model's tier, else `409`), `X-Corbanu-Client-Salt` (base64, 32 bytes) and `Idempotency-Key`. Insufficient funds → `402 {shortfall_micro_pft}`. The final stream event carries the receipt |
| `GET /v1/private-inference/receipts/{id}` | Signed receipt (immutable) |
| `GET /v1/private-inference/receipts/{id}/proof` | `200` signed proof attestation, `202` pending, `404` unavailable |
| `GET /v1/private-inference/statements/{date}` | Signed daily statement plus this account's inclusion paths |
| `POST /v1/private-inference/disputes` | Receipt ID and failing check |
| `POST /v1/pft/withdrawals` | Wallet-challenge-authorized withdrawal |

Terminal (proposed boundaries; the implementing sprint fixes the exact files):

| Area | Path | Change |
| --- | --- | --- |
| API types | `codex-rs/wallet/src/corbanu_api.rs` | `PftBalance`, `Tier`, `PriceSchedule`, `Receipt`, `ProofAttestation`; `tier` on `CorbanuApiModel` |
| Request shaping | `codex-rs/core/src/client.rs` | Tier, salt and idempotency headers on tiered models; floor check before send |
| Receipts and verifier | new crate `codex-rs/private-inference/` | JCS canonicalization, commitments, Ed25519 and Merkle checks. The offline checks do no network I/O |
| Local storage | `codex-rs/state/src/runtime/` (pattern of `gpu_rentals.rs`) | `inference_receipts` table: receipt, proof, salt, canonical request/response, keyed by thread, turn and receipt ID |
| TUI | `codex-rs/tui/src/chatwidget/model_popups.rs`, `wallet_api.rs`; new `/tier` and `/receipt` commands | Picker, floors, footer, receipt view, deposit and withdrawal screens |
| CLI | `codex-rs/cli/` | `corbanu receipt verify <id\|file> [--online] [--json]` |

## C. Verification surface

### C.1 Artifacts

| Artifact | Signed by | Produced | Contains | Leaves the machine? |
| --- | --- | --- | --- | --- |
| Receipt (C.2) | Corbanu | At completion, all tiers; immutable | Commitments, model, tier, tokens, charge | Shareable |
| Proof attestation (C.3) | Corbanu, wrapping Ambient validator signatures | When Ambient finalizes, L3 only | `receipt_hash`, PoL bundle | Shareable |
| Local package | — | At send and completion | Salt, canonical request and response | **Never by default** |
| Daily statement (B.4) | Corbanu | Daily | Balance, receipt Merkle root | Fetched |

The **shareable bundle** `<receipt_id>.cpr.json` contains the receipt and,
for L3, the proof attestation. The **full package** adds the local package and is exported
only through the warning in A.2.

### C.2 Receipt (all tiers)

```json
{
  "receipt_id": "rcpt_01J9Z…",
  "version": 1,
  "request_id": "req_…",
  "account_id_hash": "sha256:…",
  "tier": "l3_verified",
  "model": "corbanu/glm-5.2-verified",
  "weights_digest": "sha256:…",
  "operator_class": "corbanu-controlled",
  "retention_policy": "zdr-v1",
  "request_commitment": "sha256:…",
  "response_commitment": "sha256:…",
  "tokens": { "input": 1840, "cache_read": 0, "cache_write": 0, "output": 512 },
  "schedule_id": "pi-2026-11-v1",
  "charge_micro_pft": 23111,
  "started_at": "2026-11-02T14:03:11Z",
  "completed_at": "2026-11-02T14:03:19Z",
  "signing_key_id": "corbanu-receipts-2026-11",
  "signature": "ed25519:…"
}
```

- `request_commitment = SHA-256(salt ‖ JCS(request))` and
  `response_commitment = SHA-256(salt ‖ JCS(response))`. JCS is RFC 8785 JSON
  canonicalization. The response object holds the text and tool calls in order.
  Terminal sends the salt in `X-Corbanu-Client-Salt`. The gateway keeps it only
  in memory for that request, so published hashes cannot be brute-forced back
  to short prompts.
- `signature` is Ed25519 over `JCS(receipt without signature)`.
  `receipt_hash = SHA-256(JCS(receipt including signature))`.
- Signing keys come from the catalog. Terminal pins them on first use and
  warns when a key changes.
- `weights_digest` is required for L2 and L3, where Corbanu controls the
  weights. For L1 it is `null` and the receipt carries
  `upstream_model_version` as reported by the aggregator; Terminal labels it
  "reported by third party".
- Self-hosted local receipts use the same fields with `operator_class:
  "user-rented"`, `tier: null`, no charge and no signature.

### C.3 L3 proof attestation

A separate document, so the receipt never changes after it is signed:

```json
{
  "receipt_hash": "sha256:…",
  "system": "ambient-proof-of-logits",
  "chain": "ambient-mainnet",
  "job_id": "…",
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
  ],
  "signing_key_id": "corbanu-receipts-2026-11",
  "signature": "ed25519:…"
}
```

- Only hashes and salted commitments appear on-chain or in the attestation; no
  prompt or response plaintext.
- The field set depends on what Ambient exposes per request. If Ambient cannot
  bind its job record to `response_commitment`, L3 does not launch (D1).

### C.4 Public background on Proof of Logits

Ambient ([ambient.xyz](https://ambient.xyz/)) describes PoL as validators checking a miner's work by recomputing
logits at a randomly sampled token position and comparing logit hashes, rather
than rerunning the whole generation. That is why C5-5 verifies validator
signatures and Merkle paths instead of recomputing logits. Whether Ambient's
public API returns these artifacts per request is unconfirmed (D1).

### C.5 What a passing check proves, and what it does not

| Check passes | Proves | Does not prove |
| --- | --- | --- |
| Commitments (all tiers) | The receipt covers exactly what Terminal sent and received | Who saw the content |
| Corbanu signatures (all tiers) | Corbanu attests to tier, operator class, retention policy and charge | That the retention promise was kept; L2/L3 privacy is a trust-in-policy claim |
| PoL attestation (L3) | Ambient validators accepted that weights `weights_digest` produced this output, within the protocol's floating-point tolerance and spot-check probability | Confidentiality, or that the answer is correct |

The receipt view shows the right-hand column next to each result so that
"verified" is never read as "private".

### C.6 CPU verification procedure

`corbanu receipt verify` and the TUI run the same checks. None needs a GPU or
model weights. Target: under one second on a laptop CPU for C5-1 to C5-5 and
C5-7. This is a target to measure in PF-65 S02, not a measured result.

| Step | Check | Inputs | Offline? | Fails as |
| --- | --- | --- | --- | --- |
| C5-1 | Recompute both commitments; compare to the receipt | Local package + receipt | Yes (full package only) | `commitment_mismatch` |
| C5-2 | Receipt Ed25519 signature (all tiers) and attestation signature (L3) against pinned Corbanu keys | Receipt, (L3) attestation, pinned keys | Yes | `bad_signature` |
| C5-3 | Recompute the charge from token counts and the signed schedule | Receipt, schedule | Yes | `charge_mismatch` |
| C5-4 | (L3) Attestation's `receipt_hash`, `response_commitment` and `weights_digest` equal the receipt's | Receipt, attestation | Yes | `proof_binding_mismatch` |
| C5-5 | (L3) Each `logits_hash` + `merkle_path` hashes to `logits_root`; each validator signature verifies against the validator set at `block_height` | Attestation, validator set | Yes, given a cached validator set | `bad_spot_check` |
| C5-6 | (L3) `job_inclusion_proof` places the job in `block_hash`, and that block is final according to at least two independent Ambient RPC endpoints chosen by the user | Attestation, RPC | No (`--online`) | `inclusion_failed` / `not_finalized` |
| C5-7 | (L2/L3) `weights_digest` equals the digest published for that model in the signed catalog | Receipt, cached catalog | Yes | `unknown_weights` |
| C5-8 | `receipt_hash` is included in that day's signed statement | Receipt, statement | No (`--online`) | `missing_from_statement` |

Results (`--json`): `{receipt_id, tier, result, checks: [{id, result,
detail}]}` where `result` is `verified` (L3, all applicable checks pass),
`receipt_ok` (L1/L2), `local_receipt` (self-hosted), `pending`, `failed` or
`unavailable`. Checks that were not run (no `--online`, or a shareable bundle
without the local package) are reported as `skipped` and the overall result
says `partial`. The exit code is 0 only for `verified`, `receipt_ok` or
`local_receipt` with no failed check.

The TUI runs C5-1 to C5-3 at completion (plus C5-7 for L2), C5-4 to C5-7 when
an L3 proof arrives, and C5-8 once a day.

### C.7 Optional deep audit (later phase; not CPU)

For a user who wants re-execution rather than validator trust, **Deep audit**
in the receipt view rents a GPU through the existing `/gpu` flow, with its
price, spend, duration and terminate controls. It uses a pinned recipe whose
model revision matches `weights_digest`, recomputes logits at the sampled
positions and compares them with `spot_checks`. It needs the local package, so
the plaintext stays on the user's own rental. The marketplace bills in USD.

## Acceptance tests for the implementing sprints

| Flow | Start / action | Pass criterion |
| --- | --- | --- |
| Success L3 | Funded account, L3 model, prompt with a strategy file | Response streams; footer reaches `verified ✓`; `corbanu receipt verify --online --json` returns `verified`; ledger debit equals `charge_micro_pft` |
| Success L2 / L1 | Same at L2 and L1 | Footer `receipt ok`; L1 row and footer show "Third-party inference" |
| Floor enforcement | L1 model selected, `sensitive_paths` file in context | Blocked; fixture gateway receives zero requests |
| No silent tier change | L3 route returns 503 | No request reaches an L2 or L1 model; options limited to L3 models |
| Explicit one-turn switch | Below-floor prompt; choose "Use L2 model for this turn" | That turn runs on L2; the next turn uses the original selection and re-checks the floor |
| Insufficient funds | `available` < reserve | `402`; shortfall shown; draft kept; no reservation remains |
| Cancel mid-stream | Cancel after 210 tokens | Charge matches the B.3 example formula; rest released; receipt issued |
| Idempotency | Replay the same `Idempotency-Key` after a forced disconnect | One model run, one debit, same receipt ID |
| Deposit validity | Fixture partial payment whose `delivered_amount` < `Amount` | Credit equals `delivered_amount` |
| Tamper | Change one byte of the stored response | C5-1 fails; footer red; no automatic disclosure of plaintext |
| Bad proof | Attestation with a wrong Merkle path | C5-5 fails; dispute submitted; refund on gateway confirmation |
| Receipt immutability | Proof arrives after the receipt | Receipt bytes and `receipt_hash` unchanged; attestation binds to it |
| Recovery | Restart Terminal while a proof is pending | Verification resumes from local state |
| Child agents | Parent with floor L2 spawns a subagent selecting L1 | Child turn blocked |
| Secrets | Search receipts, logs, statements, exports and Task Node evidence | No API key; no salt or plaintext outside the local store and an explicitly exported full package |

The interactive flows also need the repository's true-TUI evidence before
release (`AGENTS.md`, "Interactive product proof").

## Milestones

| Milestone | Scope | Depends on |
| --- | --- | --- |
| M0 | Plan activated; PF-65 S01 accepted; OD2 decided | Travis / Alex |
| M1 | Receipts, local store, `/receipt`, `corbanu receipt verify` C5-1 to C5-3, tier picker and floors, all against a fixture gateway with no money movement | M0 |
| M2 | PFT ledger: adapter, deposits, reserve/settle/release, statements, withdrawals | M1, OD1, OD3 |
| M3 | L3 attestation, C5-4 to C5-8, disputes and refunds | M2, D1, OD4 |
| M4 | Deep audit through `/gpu` | M3 and a recipe matching the L3 weights digest |

## Open decisions and dependencies

OD1 is decided; the rest are open.

| ID | Question | Owner | Blocks |
| --- | --- | --- | --- |
| OD1 | **Decided 2026-10-06 (Travis):** settle PFT on the ledger where Task Node pays PFT today (`xrpl_pft`). Revisit a move to PostFiat L1 (`postfiat_l1`) once it is live and PFT is used there | Travis | — |
| OD2 | Product-spec amendment. The spec funds Corbanu API in USDC and names "USDAI as the preferred stablecoin partner rather than centering a Corbanu-native token". PFT is Post Fiat's token, not Corbanu's, but PFT settlement still changes a financial flow | Alex Good (Head of Product) / Travis | M0 |
| OD3 | Rates and whether a USD reference is shown | Alex Good | M2 |
| OD4 | Refund policy for failed or unavailable proofs (B.4 proposes full refund, or refund down to L2) | Alex Good | M3 |
| D1 | Ambient returns per-request job ID, logits root, spot checks and an inclusion proof bound to a client-supplied commitment | Ambient | M3 / L3 launch |
| D2 | Ambient and Corbanu gateway retention configuration that substantiates `zdr-v1` | Ambient / Corbanu infra | L2/L3 labels |
| D3 | PF-65 S01 adversary model and S02 measurements, including verifier latency | Privacy research lead | Any "private" wording; C.6 target |

## Traceability to the Task Node task

| Task requirement | Where it is met |
| --- | --- |
| Concrete request flow: what triggers a request, what tier options appear, what the user receives back | A.1 triggers and resolution; A.2 screens; A.3 happy path; A.4 failures |
| Three tiers (L1 cheap aggregator, L2 zero-data-retention, L3 fully verified) presented, priced and settled in PFT | §1 and §1.1; A.2 picker; B.2 pricing; B.3 funding, reservation and settlement; B.4 refunds, statements and withdrawal |
| Proof or hash with each response; how a user verifies on CPU | C.1 artifacts; C.2 receipt; C.3 attestation; C.5 claim boundaries; C.6 procedure |
| One document with distinct user, payment and verification sections | Sections A, B and C |
| Specific enough for an engineer to begin implementation | §1.1 routes; B.5 interfaces and code boundaries; JSON schemas; acceptance tests; milestones with gating decisions |
