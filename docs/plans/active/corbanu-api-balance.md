---
title: "Corbanu API balance and keys"
status: active
change_class: product-initiative
priority: P1
owner: "Alex Good"
max_active_sprints: 2
integration_owner: "Jim Ricketts"
activation_authority: "Alex Good — Head of Product"
activation_basis: "2026-08-30 directives to replace Corbanu Plan tiers with wallet-funded Corbanu API keys and dollar balance, then delete all legacy plan state for the sole production user"
target_release: "TBD"
deadline: "TBD"
created: 2026-08-30
updated: 2026-08-30
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Corbanu API — TO BUILD"
  requirement_excerpt: "Replace new Corbanu Plan sales with a wallet-funded, dollar-denominated Corbanu API balance."
implementation_worktrees:
  - path: "/home/pfrpc/repos/CorbanuAPI"
    branch: "feat/corbanu-api-balance"
    base_commit: "66097f417815bb094f070bd9733007d27be98725"
  - path: "/home/pfrpc/repos/CorbanuTerminal"
    branch: "feat/corbanu-api-wallet"
    base_commit: "4ff38e974b4e63cebffc5d608c5584e2d453cf1b"
---

# Corbanu API balance and keys

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | **Active** |
| Active-plan slot | **2 of 2** |
| Product authority | Alex Good, Head of Product, for product financialization and commercial integration |
| Authoritative decision | 2026-08-30 user directive recorded in the product specification |
| Final launch authority | Travis Good; compliance and provider-terms review remain required |
| Target release | TBD |
| Deadline | TBD |

## User pain

The current product sells expiring token bundles through four tiers even though
models have materially different costs. Users cannot see a simple dollar
balance, compare per-model prices, or manage multiple API keys in one place.
The word “Plan” also conflates payment, entitlement, and inference.

## Product intent and ideal flow

The user opens `/wallet`, enters **Corbanu API**, and sees a dollar balance,
priced models, and API-key summaries. They choose any positive canonical-USDC
top-up amount, review the exact transfer, unlock the wallet, and confirm it.
Settlement credits the same number of dollars to the wallet account. If the
account has no active key, Corbanu creates one and shows it exactly once in a
secure secret view while storing it in the encrypted client credential store.
Returning users can create and revoke additional keys with an unlocked wallet;
list views expose only prefixes and metadata.

Inference reserves estimated dollars atomically, routes through an internal
provider adapter, and settles actual input/cache/output cost. Rejected calls
release the reservation; ambiguous calls retain a conservative debit with an
auditable disposition. Insufficient balance fails before upstream inference.
Restart and resume preserve balances, keys, idempotency, and unsettled work.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | **Corbanu API — TO BUILD** |
| Requirement excerpt | “Replace new Corbanu Plan sales with a wallet-funded, dollar-denominated Corbanu API balance.” |
| Product outcome advanced | Wallet-funded inference with transparent API pricing |
| North-star criterion advanced | Fund Corbanu with stablecoins while keeping provider credentials and protected financial data out of model context |

## Scope

### In

- Replace new tier purchases with arbitrary positive canonical-USDC top-ups at one USDC to one dollar of credit.
- Store wallet-account balances in integer microdollars with atomic reservations and settlement.
- Share one wallet balance across independently revocable API keys and attribute spend per key.
- Show a newly created plaintext key exactly once in a secure non-transcript view; persist only its hash server-side.
- Add provider-neutral public model IDs, recommendation/speed guidance, privacy class, and versioned input/cache/output sell prices.
- Add a separate `/spawn` quick-start crew that uses the wallet-funded Corbanu API routes: Fable Nazgul, GPT-5.6 Luna Troll, and three GLM 5.3 Flash Orcs, without replacing the existing Standard Crew.
- Price every Corbanu API route at the pinned upstream cost with zero markup and microdollar settlement granularity.
- Route GLM 5.3 Flash, GLM 5.3, GPT-5.6 Luna, and GPT-5.6 Sol through the protected server-side Vercel credential.
- Route Claude Fable and DeepSeek V4 Pro through xAPI when enabled, cheaper, and healthy.
- Keep internal vendor, account, and credential metadata out of public responses.
- Delete legacy paid periods, plan credentials, token allowances, receipts, and dependent entitlement records while preserving wallet assets and Corbanu API state.
- Remove legacy Plan status, details, receipt, recovery, and inference authorization surfaces; call the product “Corbanu API”.
- Add backend, payment, concurrency, metering, migration, TUI snapshot, true-PTY, and production-readiness evidence.

### Out

- Converting legacy token allowances into dollars.
- Silently changing a published customer price during an in-flight request.
- Exposing upstream vendor names or credentials to customers or models.
- Shipping a customer-visible model before its sell price is explicitly approved.
- Launching before required stablecoin-payment, provider-terms, and compliance review.

## Invariants

- Money is integer microdollars; token counts and floating point never represent account value.
- Settlement identity is idempotent and cannot credit two accounts.
- Reservation plus available balance cannot go negative under concurrency.
- Plaintext customer keys are never persisted server-side or written to transcript, logs, or evidence.
- Every key is wallet-owned, revocable, and separately attributable; keys share only the wallet balance.
- Provider credentials resolve only inside the backend trust boundary.
- Public model identity and price are independent from internal route selection.
- Privacy remains explicit as Corbanu-controlled or third-party without naming the vendor.
- Legacy plan state cannot authorize inference or appear in the wallet UI.
- An unavailable backend fails visibly and never falls through to an unapproved route.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | `/home/pfrpc/repos/CorbanuAPI` | `feat/corbanu-api-balance` | `66097f417815bb094f070bd9733007d27be98725` | Private gateway domain, provider adapters, persistence, API, and tests |
| Jim Ricketts | `/home/pfrpc/repos/CorbanuTerminal` | `feat/corbanu-api-wallet` | `4ff38e974b4e63cebffc5d608c5584e2d453cf1b` | Public Terminal provider/catalog and `/wallet` UI |

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `CorbanuPlan/src/models.ts` | Existing internal route registry and privacy classification |
| `CorbanuPlan/src/store.ts::GatewayStore` | Existing key, entitlement, reservation, and settlement contract |
| `CorbanuPlan/src/postgres-store.ts` | Existing atomic PostgreSQL implementation and compatibility tables |
| `CorbanuPlan/src/x402.ts` | Current static tier checkout; replaced by amount-bound top-up intents |
| `CorbanuPlan/src/app.ts::createGatewayApp` | Customer API, authentication, catalog, and inference boundary |
| `codex-rs/wallet-daemon` | Existing local signing and payment capability boundary |
| `codex-rs/tui/src/chatwidget/wallet_menu.rs` | Existing purchase/recovery UI and encrypted credential storage |
| `codex-rs/tui/src/chatwidget/model_popups.rs` | Existing provider/model picker mapping and privacy label |

## Upstream-touch record

| Baseline field | Value / evidence |
| --- | --- |
| Canonical upstream URL and verified full SHA | `https://github.com/openai/codex.git` at `ba6cf9c69277caec51a4c12c5b7401a9920930e0`; fork merge-base `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa` |
| Fork base and candidate SHA | Public client base `0c3129f266d4859bfb21c291c7d3a05fc3c284e9`; private backend base `66097f417815bb094f070bd9733007d27be98725` |
| Proposed upstream update SHA | Not applicable; this initiative does not update upstream Codex |

| Feature / sprint | Upstream files and native interface | Product-owned boundary / reason | Integration owner | Contract tests / artifact | Upgrade disposition |
| --- | --- | --- | --- | --- | --- |
| PF-31 / S01 | None; standalone private backend | Product-owned provider adapter and catalog contract | Jim Ricketts | Backend adapter tests at `ef31361e5becfabc971db7a3670ed340433f18ea`; passed | No upstream patch |
| PF-32 / future | Native model provider/catalog interfaces | Thin Corbanu API provider aliases and model metadata | Jim Ricketts | Provider resolution and request-wire tests; pending | Keep adapter separable |
| PF-33 / future | Native wallet/TUI selection interfaces | Product-owned payment, key, and balance views | Jim Ricketts | Wallet daemon, snapshots, true-PTY; pending | Keep wallet UI modular |

## Sprint execution map

| Feature ID | Plan feature | Current sprint records | State |
| --- | --- | --- | --- |
| `PF-31` | Provider-neutral backend registry and Vercel adapter | [PF-31-S01](../../sprints/archive/corbanu-api-balance/pf-31-s01-vercel-adapter.md) | completed at `ef31361e5becfabc971db7a3670ed340433f18ea` |
| `PF-32` | Dollar balance, top-up intents, key lifecycle, and legacy migration | [PF-32-S01](../../sprints/archive/corbanu-api-balance/pf-32-s01-balance-topups-and-keys.md), [PF-32-S02](../../sprints/archive/corbanu-api-balance/pf-32-s02-terminal-wallet-auth.md) | completed; Terminal bridge at `cd79361d8b4f286291556a641288757d0451f52c` |
| `PF-33` | Versioned at-cost metering and xAPI/Vercel selection | [PF-33-S01](../../sprints/archive/corbanu-api-balance/pf-33-s01-at-cost-metering.md), [PF-33-S02](../../sprints/archive/corbanu-api-balance/pf-33-s02-customer-response-boundary.md) | completed; response boundary corrected at `778b4b33445aa452dce09ab416e520e6b4aaeab1` |
| `PF-34` | Terminal provider, balance/key/top-up UI, one-time secret view, and Corbanu API spawn preset | [PF-34-S01](../../sprints/archive/corbanu-api-balance/pf-34-s01-wallet-client.md), [PF-34-S02](../../sprints/current/corbanu-api-balance/pf-34-s02-wallet-ui.md) | wallet client complete; TUI in progress |
| `PF-35` | Qualification, deployment, migration docs, and human acceptance | [PF-35-S01](../../sprints/current/corbanu-api-balance/pf-35-s01-production-candidate.md) | production candidate in progress; final qualification follows PF-34 |

### Dependency graph and lane allocation

| Lane | Sprint(s) | Owner | Write scope | Shared-interface prerequisite | Integration checkpoint |
| --- | --- | --- | --- | --- | --- |
| backend | PF-31-S01 (completed) | Jim Ricketts | `src/config.ts`, `src/models.ts`, `src/vercel.ts`, `tests/config.test.ts`, `tests/vercel-routing.test.ts` | Existing `ModelRoute` and configuration contracts | 86 package tests pass; staged routes remain outside legacy catalog |
| backend | PF-32-S01 (completed) | Jim Ricketts | Store, payment, API, exact-money, and tests recorded in sprint | PF-31-S01 | 92 package tests pass; PostgreSQL fixture added but runtime unavailable |
| backend | PF-33-S01 (completed) | Jim Ricketts | Versioned price registry, dollar reservation/settlement, active provider-neutral routes, and tests | PF-32-S01 | 101 package tests and 13 disposable-PostgreSQL tests pass; typecheck and build pass |
| backend | PF-33-S02 (completed) | Jim Ricketts | Structured customer-response sanitization and live route regression | PF-33-S01 | 104 package tests pass; live GLM 5.3 Flash response is provider-neutral |
| backend | PF-32-S02 (completed) | Jim Ricketts | Signed-wallet account operations required by the Rust Terminal client | PF-33-S02 | 106 package tests pass; operation-bound auth at `cd79361d8b4f286291556a641288757d0451f52c` |
| production-candidate | PF-35-S01 | Jim Ricketts | `Dockerfile`, `fly.toml`, `src`, `tests` | PF-32-S02 and PF-33-S02 | Deploy the private backend candidate, verify database migration and provider-neutral production contracts, then hand the endpoint to PF-34 human UI testing |

### Requirement traceability

| Product requirement / adopted design | Feature and sprint | State | Acceptance evidence |
| --- | --- | --- | --- |
| Protected Vercel routing for four routes | PF-31 / PF-31-S01 | completed | `ef31361e5becfabc971db7a3670ed340433f18ea`; 86 package tests pass |
| Dollar balance and no new tiers | PF-32 | completed | `00a410be45d6f463e04d6342255df864af56a92b`; exact top-up and compatibility tests |
| Versioned per-model pricing | PF-33 / PF-33-S01 | completed | `6aa81161ece53b26915f05c3346a9ebe11b094fd`; zero-markup schedules, exact reservation/settlement, and provider-neutral catalog tests pass |
| One-time key reveal and multiple keys | PF-32, PF-34 | backend complete | API response-only key tests pass; secure-view TUI proof remains PF-34 |
| Provider-neutral customer surface with privacy class | PF-33, PF-34 | backend complete | `778b4b33445aa452dce09ab416e520e6b4aaeab1`; JSON/SSE/error sanitization and live provider-neutral response pass; Terminal snapshots remain PF-34 |
| Corbanu API crew quick start | PF-34 / PF-34-S02 | in progress | Separate Fable → Luna → 3× Flash preset, spawn-picker snapshot, runtime mapping tests, and true-TUI proof |
| Legacy plans deleted and deauthorized | PF-34, PF-35 | in progress | Terminal removal, production deletion audit, and balance/key preservation checks |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Primary success | Funded wallet, no API balance | Enter amount, unlock, confirm | Exact USDC settles, equal dollar balance appears, default key appears once securely | Balance and stored client credential survive restart |
| Additional key | Positive balance, unlocked wallet | Choose Create API key | New plaintext shown once; list later shows prefix only | Both keys work with separate attribution |
| Failure/cancel | Locked wallet or cancelled confirmation | Cancel or fail signing/payment | No debit, credit, or key creation | Idempotent retry is safe |
| Insufficient balance | Valid key, low balance | Submit priced inference | Rejected before upstream with required/available dollars | No negative balance or provider spend |
| Recovery/resume | Existing wallet on fresh install | Restore and authenticate | Balance/key summaries recover; old plaintext does not reappear | New key can be created without top-up |
| Corbanu API crew | Funded Corbanu API credential | Choose the Corbanu API crew from `/spawn` | Nazgul, Troll, and three Orcs are created with the exact requested Corbanu routes; no task starts | Existing Standard Crew remains available and the new crew has a distinct durable preset identity |
| Legacy retirement | Any legacy Plan period or credential | Open `/wallet` or attempt legacy inference | No Plan surface or authorization; Corbanu API remains available | Legacy rows are absent and wallet/API state is preserved |

## Implementation sequence

1. Land the private provider-neutral Vercel adapter without customer-visible activation.
2. Add dollar accounts, amount-bound x402 top-ups, atomic reservations, and remove legacy plan state.
3. Add approved price schedules and activate the provider-neutral model catalog.
4. Add the Terminal Corbanu API UI and secure one-time key display.
5. Qualify, migrate, deploy, and obtain human acceptance.

## Automated evidence

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Backend focused | `corepack pnpm exec tsx --test --test-concurrency=1 tests/pricing.test.ts tests/api-balance.test.ts tests/vercel-routing.test.ts tests/xapi-routing.test.ts` | 35 passed, 0 failed | `6aa81161ece53b26915f05c3346a9ebe11b094fd` |
| Backend full suite | `corepack pnpm test` | 101 passed, 0 failed | `6aa81161ece53b26915f05c3346a9ebe11b094fd` |
| PostgreSQL integration | Isolated PostgreSQL 16 plus `tests/postgres-store.test.ts` | 13 passed, 0 failed | disposable container removed after run |
| Backend build/typecheck | `corepack pnpm typecheck && corepack pnpm build` | passed | `6aa81161ece53b26915f05c3346a9ebe11b094fd` |
| Customer response boundary | `corepack pnpm test && corepack pnpm typecheck && corepack pnpm build` plus live GLM 5.3 Flash probe | 104 passed; live 200 with Corbanu model identity and no vendor/cost metadata | `778b4b33445aa452dce09ab416e520e6b4aaeab1` |
| Public Rust crates | `just test -p codex-wallet`; `just test -p codex-wallet-daemon`; `just test -p codex-model-provider-info` after `just fmt` | 14, 9, and 58 passed | `594d618306d922963cf6676d3600cd381922759c` |
| Snapshot | focused `codex-tui` wallet API and wallet-menu suites with reviewed `insta` changes | 3 wallet-API and 23 wallet-menu tests passed | `594d618306d922963cf6676d3600cd381922759c` |
| Payment/adversarial | Duplicate settlement, concurrent reserve, key leakage, fail-closed route matrix | pending | pending |

## True-TUI evidence

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Primary | pending | TensorCash | `/wallet`, Corbanu API, amount, unlock, confirm | Balance and one-time secure key | pending | pending |
| Failure/cancel | `target/debug/codex` at `594d618306d922963cf6676d3600cd381922759c` | Corbanu Terminal checkout; live-repository repetition pending | `/wallet`, Corbanu API, `1.25`, cancel; disposable wallet-bound backend | Exact $1.25 confirmation, one-time/no-tier copy, 1 USDC available, pay disabled, cancel returned without signing | passed for local candidate | `/tmp/corbanu-api-ui-qa-20260830/codex-tui.log`; no panic signature |
| Recovery/resume | pending | both | Restart, restore, create/revoke key | Balance persists; old plaintext absent | pending | pending |

## Live-repository applicability

| Repository | Applicable? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | yes | pending | pending | Trading-oriented wallet-funded inference workflow |
| Isometric Game | yes | pending | pending | TUI layout, secure view, and model picker workflow |

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Travis Good | pending | pending | Payment, key, pricing, and legacy retirement | pending | pending |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/features/corbanu-api.md` | pending | pending |

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| Customer sell prices and markup for six models | Commercial decision | Alex Good | PF-33 | **resolved 2026-08-30: exact pinned upstream cost, zero markup** |
| Shared wallet balance across keys | Product interpretation | Alex Good | PF-32 | adopted; correct before PF-32 if per-key balances were intended |
| Vercel model IDs | Provider contract | Jim Ricketts | PF-31 | verified from live catalog on 2026-08-30 |
| Legacy period treatment | Migration decision | Alex Good | PF-34, PF-35 | **amended 2026-08-30: delete all legacy plan state; preserve wallet assets and Corbanu API state** |
| Stablecoin/provider terms and compliance | Launch gate | Head of Product | PF-35 | pending |
| Existing dirty deep-research work | Integration dependency | Jim Ricketts | PF-35 | isolated in another worktree; merge explicitly later |

## Release linkage

- Release record: pending
- Benchmark tracker row: pending
- Remaining blocker: PF-34 Terminal implementation, PF-35 qualification, compliance, deployment, and human acceptance

## Completion

- [ ] Product linkage, scope, invariants, and worktrees are current.
- [ ] Every implementation unit is represented by a valid single-feature sprint.
- [ ] Required final-tree automated evidence passes.
- [ ] Required true-TUI and live-repository evidence passes.
- [ ] Human acceptance passes.
- [ ] Finished documentation matches the candidate.
- [ ] Release and benchmark records are linked.
- [ ] No hard release gate remains pending.
