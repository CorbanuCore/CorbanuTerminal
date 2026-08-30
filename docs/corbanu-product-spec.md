---
title: Corbanu Terminal Product Specification
document_type: product_spec
status: living
updated: 2026-08-27
status_labels:
  - live
  - built_not_live
  - to_build
  - to_integrate
  - to_acquire
  - pending
  - principle
priority_labels:
  - P0
  - P1
  - P2
  - continuous
hard_product_gate: 2026-10-08
---

# Corbanu Terminal

## Product definition

**Purpose:** Build the best possible tool for AI agents to trade on the blockchain.

**Product promise:** Give an agent stablecoins, market data, compute, and permissioned financial tools; let the user direct it conversationally; improve the user’s trading process; and permit action without exposing strategy, credentials, or financial information.

**Product boundary:** Corbanu Terminal is a trader-first, wallet-native AI terminal. It is not a generic coding harness, website builder, or corporate assistant. It tracks the strongest upstream behavior from Codex—the general-purpose agent-harness lineage underlying the Terminal—while preserving Corbanu’s trading, provider, wallet, security, identity, and social layers.

## Positioning

Corbanu Terminal is a cyberpunk capital-compounding machine: wallet-native, pseudonymous, powerful, social, and secure enough for real financial work.

Marketing language belongs in this positioning section. Delivery requirements below are operational.

## Product principles

1. **Trader-first:** Every product decision must improve research, backtesting, execution, risk control, or learning.
2. **Security is the product:** Prompt injection, vault extraction, financial-data leakage, and unauthorized financial action are P0 risks.
3. **Wallet-native:** Minimize email, credit-card, real-name, and custodial dependencies without promising avoidance of venue-specific identity or compliance requirements.
4. **Integrate, do not rebuild:** Use strong broker-agent services, data providers, stablecoin rails, and Post Fiat infrastructure.
5. **Private means explicit:** Distinguish Corbanu-controlled inference from third-party inference at selection and use.
6. **Visible control:** Show what an agent can read, disclose, propose, approve, sign, and broadcast.
7. **Upstream velocity:** Maintain continuous Codex parity without removing Corbanu-specific behavior.
8. **One product:** Identity, data, inference, skills, execution, NAV, and social must feel native to one Terminal.
9. **Deterministic authorization:** A model deciding that an action looks safe is never authorization.

## Upstream compatibility — CONTINUOUS

Status: **PRINCIPLE; qualification evidence required, not a current certification**.
Product decision: Travis Good, 2026-08-27. Corbanu-specific capabilities must
remain separable from native Codex lifecycle and provider interfaces so upstream
improvements can be adopted without losing product behavior or weakening security.
Upgrade acceptance includes regression evidence for affected native delegation,
transport recovery, context persistence, and authorization boundaries, not merely
a conflict-free merge. The [upstream integration contract](plans/upstream-integration.md)
owns the engineering process and evidence fields.

# Shipping MVP — LIVE

Corbanu Terminal already exists and is shipping. The roadmap expands this live, multi-provider MVP into the complete trader product; it is not a prerequisite for Corbanu Terminal to exist.

| Area                         | Shipping capability                                                                                                                                              |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Runtime**                  | Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu` command, and legacy `pfterminal` command and state compatibility.                                           |
| **Multi-provider inference** | OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers. |
| **Vault and credentials**    | Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.                                   |
| **Agent orchestration**      | Sauron → Nazgul → Troll → Orc orchestration, model-aware delegation, durable mailboxes, supervision, resume, and recovery.                                       |
| **Workspaces**               | `/panes`, `/agent`, approvals, existing general sandboxing, review, MCP, skills, plugins, apps, connectors, and background terminals.                            |
| **Wallet and payments**      | Local Solana wallet, SOL and canonical USDC support, scoped signing, backup/restore, and Corbanu Plan purchase/recovery.                                         |
| **Compute**                  | Vast.ai and RunPod rental workflows with price, spend, duration, readiness, stop, and termination controls.                                                      |
| **Task Node and identity**   | Tasks, evidence, verification, rewards, balances, chat, context, linked identity, and **live Task Node-linked Nostr identity**.                                  |
| **Remote and context**       | Allowlisted Telegram; durable `/goal` and `/memories`; ephemeral `/side` and `/btw`; `/skills` and `/docs`.                                                      |

## Live MVP versus the P0 security controls

The shipping MVP already has a wallet, vault, scoped signing, approvals, and
general workspace sandboxing. These are live product capabilities.

The new `/security` surface makes additional protection understandable and
user-controlled. **Permissive preserves the shipping behavior and does not
silently change existing policies.** Moderate and Aggressive add deterministic
protections around untrusted content, sensitive data, credentials, tools, and
financial actions.

# Corbanu Plan — LIVE; DEPRECATION TARGET

Corbanu Plan is wallet-native, one-calendar-month prepaid inference purchased through **x402**, normally using canonical USDC on Solana. The wallet proves ownership and receives a revocable Plan credential. Every tier uses the same model catalog and differs by allowance.

| Tier    |    Price | Weekly allowance | Monthly allowance |
| ------- | -------: | ---------------: | ----------------: |
| Starter |   1 USDC |             250K |                1M |
| Basic   |  20 USDC |               5M |               20M |
| Power   |  50 USDC |            12.5M |               50M |
| Pro     | 200 USDC |              50M |              200M |

| Models                          | Backend | Privacy boundary                       |
| ------------------------------- | ------- | -------------------------------------- |
| GLM 5.2, Kimi K2.7 Code         | Ambient | Private, Corbanu-controlled inference. |
| DeepSeek V4 Pro, Claude Fable 5 | xAPI    | Non-private, third-party inference.    |

**x402 is payment. xAPI is inference.** Customers authenticate to Corbanu Plan, not directly to xAPI.

Mainline xAPI uses a protected server-side operator credential and balance gate. A separate branch implements per-wallet xAPI accounts, encrypted tenant keys, capped refills, and wallet-level cost attribution. That work is **BUILT NOT LIVE** until it is merged, deployed, and verified in production.

Model-specific cost normalization, Plan margin targets, and other commercial performance targets are **TBD**. The listed allowances must not be interpreted as equal upstream cost across models.

# Corbanu API — TO BUILD

Product decision: Alex Good, Head of Product, 2026-08-30. Replace new Corbanu
Plan sales with a wallet-funded, dollar-denominated Corbanu API balance. Alex
Good amended the migration decision on 2026-08-30: Corbanu has one production
user, so all legacy paid periods, plan credentials, token allowances, receipts,
and dependent entitlement records are retired and deleted instead of being
grandfathered. Wallet assets and Corbanu API balances, keys, and ledgers are not
legacy plan data and remain intact.

The user tops up with canonical USDC and receives the same number of dollars of
Corbanu API credit. There are no Starter, Basic, Power, or Pro purchase tiers,
calendar-month renewals, or weekly/monthly token allowances for new funding.
Usage debits the balance using an explicit versioned price for input, cached
input, cache creation where applicable, and output tokens. Reservations,
settlement, idempotency, and insufficient-balance enforcement remain atomic and
server-authoritative.

The paying wallet owns one balance shared by its API keys. The first successful
top-up can create a default API key. Its plaintext is returned and displayed
only once through a secure non-transcript view; the service stores only a keyed
hash and display prefix. An unlocked wallet can create and revoke additional
keys without another payment. Each key retains separate creation, last-use,
revocation, request, and spend attribution.

The customer-facing model catalog uses Corbanu identities and displays Corbanu
prices without identifying the upstream compute vendor. The privacy boundary
remains explicit: every route is labeled either **Corbanu-controlled** or
**third-party inference**. Provider credentials and internal routing metadata
never enter model context or customer responses.

Initial intended routes are GLM 5.3 Flash (**Recommended**), GLM 5.3 (labeled as
using balance faster), GPT-5.6 Luna at xhigh, GPT-5.6 Sol, Claude Fable, and
DeepSeek V4 Pro. Vercel is the internal route for both GLM models and the two
OpenAI models; xAPI remains the preferred internal route for Fable and DeepSeek
where it is cheaper and passes reliability checks. Internal routing may change
without changing the public model identity or price during an in-flight request.
Customer pricing is exact upstream cost with zero markup, adopted by Alex Good
on 2026-08-30. Each request pins a versioned upstream route and price schedule;
catalog and ledger records expose the customer rate without exposing the vendor.

Legacy Corbanu Plan names, purchase/recovery/details surfaces, entitlements, and
inference authorization are not part of the product. New and existing UI,
payment, account, and inference flows call the product **Corbanu API**.

# Target product

The expansion roadmap must make the following one coherent trader product:

- native equities and crypto market data;
- first-party reproducible backtesting;
- first-party Hyperliquid support;
- seamless use of existing brokerage-agent services;
- a clear `/security` surface with Permissive, Moderate, and Aggressive levels;
- native Post Fiat NAV infrastructure;
- a first-class Corbanu trollbox using live Task Node/Nostr identity;
- wallet-funded inference, data, and financial tools;
- USDAI as the preferred stablecoin partner rather than centering a Corbanu-native token;
- continuous upstream Codex parity.

# Users and decisions

| Persona             | Primary job                                                                              | Product decision implication                                                                             |
| ------------------- | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| **Buyside Bro**     | Improve personal Hyperliquid or DEX trading without linking activity to a work identity. | Default to private research, pseudonymous identity, explicit disclosure controls, and bounded execution. |
| **Hobbyist**        | Explore and observe AI-assisted trading safely.                                          | Lead with guided research, reproducible backtests, paper trading, and approval-required actions.         |
| **Accelerationist** | Deploy technically capable agents on-chain.                                              | Provide machine-readable tools and constrained autonomy without weakening deterministic controls.        |
| **Crypto AI Guy**   | Use an AI harness that remains explicitly crypto-native.                                 | Keep wallet, stablecoin, venue, identity, and agent workflows native rather than bolted on.              |

All personas prefer reduced centralized identity, custody, and data collection. Corbanu must not imply that pseudonymity overrides a provider’s legal, jurisdictional, KYC, AML, sanctions, or account requirements.

# Discovery and activation

Corbanu Terminal is distributed through **Corbanu.com**, the newsletter and media property for on-chain stock ideas. It appears near the subscription surface and in videos and live demonstrations.

The first interaction must be immediately useful. For example, “Corbanu, show me a chart of AAPL” should lead naturally to approved native data and then to research, a replayable backtest, a monitored idea, or a permissioned execution workflow.

# Ownership and decision rights

| Person or role                                     | Accountability                                                                                                                                                                                                                                                       |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Travis Good — final product authority**          | Owns all product decisions, priorities, gates, scope, and go/no-go decisions.                                                                                                                                                                                        |
| **Alex Good — Head of Product**                    | Owns trading and backtesting skill creation; dataset business development, including Tiingo, Sharadar, and the desired EarningsCall.biz partnership; Stripe accounts; product financialization; commercial integrations; and related business-development execution. |
| **Jim Ricketts — lead Corbanu Terminal developer** | Owns releases, technical implementation coordination, integration delivery, and implementation quality.                                                                                                                                                              |

# Accountable sequencing

Security is the first implementation initiative and begins immediately. Other
roadmap items have their own owners and plans; they are not dependencies or
scope for the `/security` implementation plan.

| Sequence | Capability or gate                                   | Status         | Priority   | Delivery owner  | Dependency                                                                         | Deadline                                | Measurable definition of done                                                                                                                                                                                                                                                                                                                                                       |
| -------: | ---------------------------------------------------- | -------------- | ---------- | --------------- | ---------------------------------------------------------------------------------- | --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|    **1** | **`/security` levels and protected-action controls** | TO BUILD       | P0         | Lead developer  | Existing approval, vault, wallet, tool, and sandbox boundaries                     | **2026-10-08; work begins immediately** | The `/security` tab ships with Permissive, Moderate, and Aggressive. Permissive demonstrably preserves current behavior. Moderate and Aggressive enforce their documented controls outside the model. Critical attack-class regressions pass, no critical finding remains open, audit records contain no secrets, and downgrade, revocation, and kill-switch flows pass end to end. |
|    **2** | **Native market-data gate: Tiingo and Sharadar**     | TO ACQUIRE     | P0         | Head of Product | Commercial authorization and native engineering access                             | **2026-10-08**                          | Tiingo and Sharadar equities datasets are commercially authorized and available natively to Terminal research and backtesting; source, timestamps, and dataset identity are recorded for replay; credentials remain outside model context.                                                                                                                                          |
|    **3** | First-party backtesting skill                        | TO BUILD       | P1         | Head of Product | Native data                                                                        | TBD                                     | A strategy can be replayed from recorded data version, code, parameters, model, and environment; required bias and cost controls are applied; machine-readable results promote to paper trading.                                                                                                                                                                                    |
|    **4** | First-party Hyperliquid skill                        | TO BUILD       | P1         | Head of Product | Protected-action controls and venue review                                         | TBD                                     | Market/account reads and order lifecycle work in paper and approval-required modes; limits are enforced outside the model; credentials are never model-visible; every proposal and action is audited.                                                                                                                                                                               |
|    **5** | Existing brokerage-agent services                    | TO INTEGRATE   | P1         | Head of Product | Protected-action controls, provider authorization, and compliance review           | TBD                                     | At least one approved existing brokerage-agent service is usable through native normalized account, position, order, fill, permission, limit, health, and audit surfaces; read-only is the default.                                                                                                                                                                                 |
|    **6** | Post Fiat NAV infrastructure                         | TO INTEGRATE   | P1         | Head of Product | Protected-action controls, supported Post Fiat capabilities, and compliance review | TBD                                     | Terminal can discover and inspect supported NAV products and complete supported transaction flows with visible proofs, settlement state, permissions, and audit records.                                                                                                                                                                                                            |
|    **7** | Corbanu trollbox                                     | TO BUILD       | P1         | Lead developer  | Live Task Node/Nostr identity and untrusted-input handling                         | TBD                                     | Real-time pseudonymous rooms, distinct human/agent identity, structured cards, moderation controls, consent-gated financial sharing, and untrusted-input treatment work end to end.                                                                                                                                                                                                 |
|    **8** | EarningsCall.biz transcript partnership              | TO ACQUIRE     | P2         | Head of Product | Commercial agreement                                                               | TBD                                     | If acquired, authorized real-time transcripts are available with source and timestamp provenance.                                                                                                                                                                                                                                                                                   |
|    **9** | USDAI preferred stablecoin partnership               | TO ACQUIRE     | P2         | Head of Product | Commercial and compliance review                                                   | TBD                                     | If acquired, approved USDAI flows are native where appropriate. Corbanu does not center a new native token.                                                                                                                                                                                                                                                                         |
|   **10** | Per-wallet xAPI isolation                            | BUILT NOT LIVE | P2         | Lead developer  | Merge, deployment, tenant-isolation review, and accounting verification            | TBD                                     | The existing branch is merged and deployed; wallet isolation, encrypted tenant keys, capped refills, and cost attribution pass production verification.                                                                                                                                                                                                                             |
|   **11** | Upstream Codex parity                                | PRINCIPLE      | CONTINUOUS | Lead developer  | Continuous upstream review and benchmark evidence                                  | Continuous                              | Relevant upstream harness improvements are assessed and adopted without regressions to Corbanu-specific behavior. At least every three releases, the full checked-in coding benchmark catalog is run across the relevant production model set; correctness, end-to-end runtime, and spend are recorded, and a threshold regression blocks release.                                  |
|   **12** | Corbanu API balance and keys                         | TO BUILD       | P1         | Head of Product | Explicit sell prices, payment/compliance review, backend and Terminal qualification | TBD                                     | A wallet can top up a dollar balance with canonical USDC, receive a one-time API key, create/revoke additional keys, inspect balance and per-model prices, and use the approved provider-neutral model catalog without plan tiers, legacy plan surfaces, or expiring token allowances. |

# P0 `/security` levels

## Product surface

`/security` opens one focused tab showing the current security level, a short
plain-English explanation, the protections that change at each level, and a
confirmation step before applying a change.

| Level          | Product promise                                                              | Policy behavior                                                                                                                                                                                                                                                                                                                                           |
| -------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Permissive** | Preserve the Corbanu Terminal users have now.                                | Default for existing users. Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged. The tab explains that no additional security controls are active.                                                                                                                                                                 |
| **Moderate**   | Add strong protection without making normal agent work painful.              | Treat external content and tool output as untrusted; prevent secrets and protected financial data from entering model-visible context; require trusted credential resolution; separate proposal, approval, signing, and broadcast; show the expected effect before protected actions; record redacted audit events; support revocation and a kill switch. |
| **Aggressive** | Minimize sensitive access and require the user to open every important door. | Includes Moderate, then defaults sensitive tools, account access, credential use, protected-data disclosure, financial actions, arbitrary egress, and clipboard/export paths to denied. The user grants narrow, expiring access; every sign or broadcast requires exact human approval; child agents inherit the level and cannot weaken it.              |

Permissive must be verified against the pre-feature candidate so selecting it
does not silently tighten or loosen an existing policy. Moderate is the
recommended level once qualified. Aggressive is an explicit lockdown posture,
not a marketing label.

Only the human can change the level through the TUI. Agents, prompts, project
files, tools, hooks, plugins, connectors, MCP servers, and retrieved content
cannot change or downgrade it. A confirmed change applies immediately to active
and future agents, invalidates incompatible pending approvals, persists across
restart, and produces a secret-free audit event.

## Release rule

Moderate and Aggressive controls must be deterministic policy around the model,
not prompt text. A model may identify risk or propose an action, but cannot
authorize itself, resolve a secret, change the security level, or bypass a
protected-action decision.

## Required trust boundaries

The stronger guarantees in this section, **Non-negotiable controls**, and
**Required adversarial tests** apply to Moderate and Aggressive. Permissive
preserves the existing product, including its existing vault/helper policies;
it does not acquire an all-mode confidentiality guarantee from this initiative.

1. External content enters as untrusted data.
2. The model may research and propose but cannot resolve secrets or authorize itself.
3. Credentials are referenced by label and resolved only inside a trusted execution boundary.
4. Account reads, order construction, approval, signing, and broadcast are separate permissions.
5. Deterministic policy evaluates every proposed financial or disclosure action.
6. Signing and broadcast occur only after policy and approval requirements pass.
7. Audit records capture decisions and identifiers without exposing secrets.

## Non-negotiable controls

- Treat webpages, transcripts, social posts, trollbox messages, files, email, tool output, and retrieved text as **untrusted data**, never executable instructions.
- Classify instruction intent and provenance before external content can influence tools or financial actions.
- Keep vault values, seeds, private keys, broker credentials, balances, positions, PNL, and identifying financial data out of model-visible context except for narrowly scoped derived values.
- Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.
- Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.
- Separate research, account reads, order construction, approval, signing, and broadcast.
- Require explicit venue, destination, asset, size, leverage, price, slippage, time, notional, and loss limits.
- Simulate and display the complete expected effect before signing.
- Support allowlists, denylists, rate limits, daily loss/notional/leverage caps, cooldowns, revocation, and a kill switch.
- Block vault enumeration, credential extraction, portfolio disclosure, policy changes, approval bypass, and unapproved value transfer.
- Detect duplicate, stale, conflicting, and ambiguous financial actions.
- Record tamper-evident policy decisions, tool calls, approvals, signatures, and transaction or order IDs without secrets.

## Required adversarial tests

The release suite must cover:

- direct and indirect prompt injection;
- tool-output and retrieved-content injection;
- task hijacking through repository instructions, misleading review output, or
  requests to weaken tests, including attacks containing no secret canary;
- vault, seed, credential, and financial-data extraction;
- confused-deputy attacks;
- malicious plugins, connectors, and MCP servers;
- trollbox and social-engineering attacks;
- unauthorized policy modification or approval bypass;
- unauthorized venue, asset, destination, leverage, or notional;
- duplicate, replayed, stale, and ambiguous actions;
- sensitive log, clipboard, or egress leakage;
- kill-switch, cooldown, revocation, and limit failures.

**Pass condition:** Every critical attack-class regression passes and no critical finding remains open.

## Moderate/Aggressive isolation and content provenance

Status: **TO BUILD**. Product decision: Travis Good, 2026-08-27, approved the
security-plan refactor while explicitly preserving Permissive as-is.

Browser isolation is a separately scoped feature within the security initiative.
Eligible public-web acquisition runs in an ephemeral isolated process boundary
without host browser profiles, inherited credentials, vault access, host IPC, or
unrestricted workspace access. Network destinations and redirects are enforced;
downloads remain quarantined until an explicitly approved promotion. Missing
isolation denies the affected acquisition path rather than falling back to the
host browser. Authenticated browser login is not part of this initial feature.

Runtime setup decision: Travis Good, 2026-08-27. Support Windows, Linux, and
macOS with containerized Scrapling. Reuse an installed Podman or Docker runtime
without replacing it or changing its global configuration; prefer Podman when
installing a runtime for the first time. Selecting Moderate (also called
“medium” in discussion) or Aggressive checks isolation readiness. Pull the
pinned image if absent, start the Corbanu service if stopped, and recover a
stalled Corbanu-owned service with bounded restart attempts and a fresh health
test. Never restart unrelated workloads or silently downgrade protection.

If no runtime exists, offer a Corbanu-guided installation, then image setup and
end-to-end verification. Explain downloads, disk/VM requirements, and host
changes before consent. Prefer rootless operation; installation or VM/WSL
prerequisites may still require elevation. Only the operating system's trusted
authentication surface accepts the user's password. Corbanu, agents, chat,
transcripts, logs, and configuration must not collect or retain that password.
Cancellation, failed setup, or failed health checks leave acquisition denied
and the missing protection visible. Permissive triggers none of this setup.
Mac/Linux qualification precedes the Windows run; all three remain required.

External content remains untrusted after extraction or sanitization. Source
provenance and taint survive summaries, compaction, children, memory, and resume;
deterministic policy rechecks protected actions after untrusted reads. A
classifier may inform risk but cannot confer authority. Protected values must
remain absent from model-visible outputs, diagnostics, artifacts, and inherited
environments, including reflected provider failures.

The security inspector distinguishes the requested level from effective
enforcement and displays Browser Isolation and External Content Firewall health
separately. Unavailable controls cannot be represented as active protection.
These contracts do not change Permissive or weaken any release evidence gate.

The acceptance model distinguishes semantic model mistakes from deterministic
authority violations. Wrappers, sanitization, and detectors reduce exposure;
they do not guarantee the model never follows malicious text. A task-hijacking
test must report both task integrity and policy outcomes, rather than treating
absence of a secret leak as proof that the task remained intact.

# Trader capabilities

## First-party backtesting skill — TO BUILD

### Inputs

- hypothesis;
- universe;
- horizon;
- approved data source;
- benchmark;
- strategy constraints.

### Required behavior

- Prevent or explicitly detect look-ahead, survivorship, timestamp, corporate-action, and delisting errors.
- Include fees, spread, slippage, financing, borrow, and venue-specific assumptions.
- Support equities, crypto spot, perpetuals, and NAV strategies through adapters.
- Produce trades, equity curve, drawdowns, exposure, PNL attribution, and machine-readable results.
- Record data version, code, parameters, model, and environment for replay.
- Use Corbanu-controlled private inference by default for proprietary strategy work.
- Promote eligible results to paper trading and then to approval-gated execution.

A **reproducible backtest** is one that can be rerun from the recorded data version, code, parameters, model, environment, and cost assumptions, with discrepancies surfaced rather than silently ignored.

## First-party Hyperliquid skill — TO BUILD

- Read approved market and account data, positions, margin, funding, orders, and fills.
- Draft, simulate, place, replace, and cancel orders.
- Enforce leverage, notional, slippage, loss, venue, and instrument limits outside the model.
- Support paper, approval-required, and constrained-autonomy execution modes as separately gated.
- Support builder or referral codes where applicable.
- Never expose exchange credentials to the model.
- Audit every proposed, approved, rejected, and executed action.

## Brokerage-agent layer — TO INTEGRATE

Corbanu must connect existing first-party broker tools, MCP servers, agent accounts, and aggregation services through their own secure authorization flows rather than rebuild brokerage infrastructure.

The Terminal must normalize:

- accounts and balances;
- positions;
- orders and fills;
- permissions and limits;
- connection health;
- recent proposed and completed actions.

Default access is read-only. Corbanu must preserve venue-specific capabilities and requirements, keep credentials outside prompts, and give users a consistent experience across crypto venues, retail brokers, and institutional services.

| External layer                     | Examples from current product context                    | Corbanu approach                                                               |
| ---------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Broker-native agents               | Robinhood, IBKR, Public, Webull, Coinbase, eToro, Moomoo | Integrate approved existing agent surfaces.                                    |
| Developer brokerage infrastructure | Alpaca                                                   | Use the infrastructure and make permissions native and visible.                |
| Multi-broker aggregation           | SnapTrade and similar services                           | Normalize supported brokers through one Corbanu permission layer.              |
| Consumer agent apps                | End-to-end AI trading products                           | Differentiate through security, pseudonymity, extensibility, and one Terminal. |

## Post Fiat NAV — TO INTEGRATE

Make Post Fiat NAV infrastructure native to Corbanu Terminal rather than a separate expert workflow.

The supported Terminal experience must:

- discover NAV products;
- inspect strategy, assets, venues, NAV, reserves, proofs, liquidity, privacy, and risk;
- build, research, and monitor portfolio or NAV strategies;
- subscribe using supported stablecoins and track the resulting position;
- construct supported private OTC or NAV swaps;
- expose proof, settlement, privacy, redemption, and exit state in human- and machine-readable form;
- support exit and redemption as Post Fiat infrastructure makes those flows available;
- use Corbanu data, backtesting, Hyperliquid, wallet, security, and agents as the operating front end.

Reference: [Post Fiat](https://postfiatorg.github.io/).

# Social: Corbanu trollbox

**Status:** TO BUILD
**Priority:** P1

The trollbox is a first-class trading and agent surface, not a community sidebar.

Required capabilities:

- real-time pseudonymous chat using Task Node identity and its already-live Nostr integration;
- rooms for tickers, markets, agents, strategies, and NAV products;
- visibly distinct human and agent identities;
- reputation grounded in Task Node work, evidence, and history;
- structured cards for charts, backtests, positions, tasks, and NAV objects;
- moderation, mute, block, report, spam control, and identity-level reputation;
- explicit consent before sharing balances, positions, orders, PNL, wallets, or private research;
- security classification of every message and card as untrusted external input.

# Data and economic partnerships

| Partner                               | Requirement                                                                                                 | Owner     | Negotiation state      | Fallback |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------- | --------- | ---------------------- | -------- |
| **Tiingo**                            | Required bare-minimum equities data for native research and backtesting by the 2026-10-08 market-data gate. | Alex Good | TBD                    | TBD      |
| **Sharadar**                          | Required bare-minimum equities data for native research and backtesting by the 2026-10-08 market-data gate. | Alex Good | TBD                    | TBD      |
| **EarningsCall.biz**                  | Desired real-time earnings transcript feed for research, alerts, trollbox rooms, and event backtests.       | Alex Good | TBD                    | TBD      |
| **USDAI**                             | Preferred stablecoin and economic partner instead of creating or centering a Corbanu-native token.          | Alex Good | TBD                    | TBD      |
| **Existing brokerage-agent services** | Required integration path for seamless brokerage connectivity without rebuilding brokerage infrastructure.  | Alex Good | Provider selection TBD | TBD      |
| **Post Fiat**                         | Required infrastructure dependency for native NAV discovery and supported transaction flows.                | Alex Good | Integration state TBD  | TBD      |

All agreements, licensing rights, provider terms, commercial targets, and fallbacks remain TBD unless explicitly stated above. Final decision rights are defined in the ownership table.

The intended experience is one coherent Corbanu relationship through which the user receives the appropriate inference, market data, and tools rather than manually assembling unrelated subscriptions.

# Target product loops

These are target expansion workflows, not claims that every step is live today.

## Idea → trade

1. Ask for an idea or chart.
2. Pull approved native data.
3. Produce a replayable backtest.
4. Promote the strategy to paper trading.
5. Stage a Hyperliquid or broker order under explicit limits.
6. Approve manually or execute inside an approved constrained-autonomy policy.
7. Return risk, PNL, and audit evidence to the strategy record.

## Trollbox → research

1. A Task Node/Nostr identity posts an idea.
2. Corbanu treats the post as untrusted social data.
3. The user explicitly elects to research it.
4. Approved data and transcripts feed a backtest or risk report.
5. Results appear as structured social cards without exposing account data.

## NAV

1. Discover a NAV strategy and inspect proofs, liquidity, privacy, and risk.
2. Research or backtest it.
3. Apply the active `/security` level and protected-action controls to every wallet or account action.
4. Subscribe, swap, monitor, exit, or redeem through supported Post Fiat flows.
5. Discuss the strategy through the trollbox without leaking holdings.

# Product measurement

No commercial performance numbers have been supplied. The following metrics must be instrumented, with targets set through the decision rights defined above.

| Area                 | Metric                                                                                | Target                                                  |
| -------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Security             | Critical attack-class regressions passing; critical findings open                     | 100% pass; zero critical findings open                  |
| Data gate            | Tiingo and Sharadar available natively for research and backtesting                   | Binary pass by 2026-10-08                               |
| Backtesting          | Replay success and surfaced discrepancies                                             | TBD                                                     |
| Activation           | Users progressing from first chart or idea to saved research or backtest              | TBD                                                     |
| Paper-to-live funnel | Backtest-to-paper and paper-to-approved-live conversion                               | TBD                                                     |
| Retention            | Returning active traders and active Plan users                                        | TBD                                                     |
| Corbanu Plan         | Purchases, renewal, usage, revenue, and model-level unit economics                    | TBD                                                     |
| Security adoption    | Sessions and protected actions by Permissive, Moderate, and Aggressive                | TBD                                                     |
| Partnerships         | Dataset, transcript, stablecoin, broker, and NAV integration readiness                | Binary milestone status; commercial targets TBD         |
| Trollbox             | Active identities, useful structured cards, moderation events, and consent violations | TBD; zero unauthorized financial disclosure is required |

# Risks, dependencies, and decision gates

| Risk or dependency                                                  | Consequence                                                                                                  | Required gate or mitigation                                                                                                                                                                                                            | Owner                   |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| **`/security` controls miss 2026-10-08**                            | Moderate and Aggressive cannot be presented as available security levels.                                    | Preserve Permissive behavior, do not ship incomplete controls, and issue a revised plan.                                                                                                                                               | Final product authority |
| **Tiingo or Sharadar unavailable**                                  | The required native equities-data gate fails.                                                                | Negotiation state and fallback are TBD; resolve through the defined product decision process.                                                                                                                                          | Head of Product         |
| **Other external partnership failure**                              | Transcript, stablecoin, brokerage, or NAV scope may be delayed or reduced.                                   | Negotiation state and fallback must be recorded before commitment; no partner capability may be presented as live before deployment.                                                                                                   | Head of Product         |
| **Data licensing or use restrictions**                              | Research, display, storage, or backtesting may be constrained.                                               | Complete provider-rights review before production use; encode applicable storage, attribution, and redistribution rules.                                                                                                               | Head of Product         |
| **Brokerage and Hyperliquid jurisdictional exposure**               | Features may not be available to every user or region.                                                       | Before live enablement, review venue terms, jurisdiction, account eligibility, KYC/AML, sanctions, derivatives, disclosures, and required controls. Record an explicit launch decision without inventing a universal legal conclusion. | Final product authority |
| **Stablecoin payments and USDAI integration**                       | Payment or financial workflows may trigger provider, sanctions, tax, or jurisdictional obligations.          | Complete legal/compliance and provider-terms review before launch or material expansion.                                                                                                                                               | Head of Product         |
| **Post Fiat NAV workflows**                                         | Subscription, swap, redemption, marketing, or product structure may require jurisdiction-specific treatment. | Legal/compliance review and explicit product go/no-go are required before each live transaction type.                                                                                                                                  | Final product authority |
| **Referral, builder-code, advertising, and financialization model** | Commercial incentives may require disclosures or restrictions.                                               | Define the commercial implementation and complete required legal/compliance review before launch.                                                                                                                                      | Head of Product         |
| **Third-party inference leakage**                                   | Strategy or financial context may leave Corbanu-controlled infrastructure.                                   | Keep provider privacy labels explicit; private workflows default to Corbanu-controlled inference; minimize model-visible financial data in all modes.                                                                                  | Lead developer          |
| **Upstream Codex changes**                                          | Corbanu may fall behind or regress specialized behavior.                                                     | Continuous parity review with regression tests for Corbanu-specific layers.                                                                                                                                                            | Lead developer          |

## Open decisions

- Post-gate release dates: **TBD**
- Commercial KPI targets: **TBD**
- Tiingo, Sharadar, EarningsCall.biz, USDAI, brokerage, and Post Fiat negotiation states: **TBD**
- Partner fallback plans where not supplied: **TBD**
- Initial brokerage-agent provider selection: **TBD**
- Jurisdiction-by-jurisdiction feature availability: **TBD following required review**
- Per-wallet xAPI merge and deployment date: **TBD**

# Business model

Revenue comes from:

- wallet-funded Corbanu API usage and bundled data;
- referral or builder codes in execution workflows;
- commercial integrations;
- advertising through Corbanu.com;
- future Post Fiat NAV infrastructure where appropriate.

The economic center is useful stablecoin-funded activity, not a new token for its own sake.

Commercial ownership is defined in the roles table. Pricing rationale, revenue targets, conversion targets, and unit-economics thresholds are TBD.

# Glossary

| Term                              | Meaning in this specification                                                                                                                                  |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Codex parity**                  | Continuous adoption of relevant upstream agent-harness behavior without losing Corbanu-specific capabilities.                                                  |
| **Task Node**                     | Corbanu’s task, evidence, verification, reward, balance, chat, context, and identity system. Its Nostr identity linkage is live.                               |
| **Nostr**                         | The live identity and social protocol integration linked through Task Node.                                                                                    |
| **Sauron → Nazgul → Troll → Orc** | Names for the live multi-agent orchestration hierarchy used for delegation, supervision, and execution work.                                                   |
| **x402**                          | The payment mechanism used to purchase Corbanu Plan.                                                                                                           |
| **xAPI**                          | A third-party inference backend; it is not the Corbanu Plan payment or customer-authentication layer.                                                          |
| **Post Fiat NAV**                 | External NAV infrastructure that Corbanu intends to make native for discovery, proofs, subscriptions, swaps, monitoring, exit, and redemption where supported. |
| **Private inference**             | Inference controlled by Corbanu, as identified in the Corbanu Plan model table; it does not describe third-party xAPI inference.                               |
| **Constrained autonomy**          | Live execution restricted to named accounts, venues, instruments, and deterministic signed limits enforced outside the model.                                  |
| **Native market data**            | Commercially authorized provider data accessible directly through Corbanu Terminal and its skills with source and replay provenance.                           |

# North-star acceptance test

The complete trader product passes when a pseudonymous trader can:

1. fund Corbanu with stablecoins;
2. choose clearly labeled Corbanu-controlled private inference;
3. obtain native equities and crypto data;
4. ask for an idea and run a reproducible backtest;
5. discuss it in the trollbox under a Task Node-linked Nostr identity;
6. promote it through paper trading;
7. deploy it through the first-party Hyperliquid skill or an integrated existing brokerage-agent service under the active `/security` level and explicit execution permissions; and
8. use supported Post Fiat NAV infrastructure as a native Terminal capability.

Throughout the loop, external content must be unable to prompt-inject the agent into revealing vault contents, exposing protected financial information, changing policy, bypassing approval, or moving money outside explicit deterministic limits.
