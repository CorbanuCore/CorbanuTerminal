---
title: "Accounting follow-ups after PF-60"
status: draft
change_class: product-initiative
priority: P1
owner: "Codex accounting lane (proposed); Travis Good accepts"
parallel_sprint_limit: 1
integration_owner: "Codex coordinator (proposed)"
activation_authority: "Travis Good"
activation_basis: "Travis, 2026-10-10: accept PF-60-S04's remaining gaps on condition that each is carried into a future sprint; activation pending a free slot and his decision"
target_release: "TBD"
deadline: "TBD; PF-60-S07's Sol promotion item must land before 2026-11-21"
created: 2026-10-10
updated: 2026-10-10
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Product measurement"
  requirement_excerpt: "No commercial performance numbers have been supplied."
implementation_worktrees: []
---

# Accounting follow-ups after PF-60

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md). Predecessor:
[PF-60 accounting](../completed/main-2026-10-10-portfolio-agent-cost-accounting.md), completed 2026-10-10.
This is a **draft**: it authorizes no implementation until Travis activates it into a free slot.

## Activation record

- Status: draft; no active slot. Created 2026-10-10 when PF-60 closed to free slot 2 for PF-84.
- Source: Travis's 2026-10-10 decisions when accepting [PF-60-S04](../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md):
  accept the remaining gaps (i)–(viii) but make sure each is covered by a future sprint.
- Gate: Travis activates the plan (or authorises single sprints as bounded fixes), and decides the open questions below.

## User pain

PF-60's `/cost` is accepted, but some paid work is still unrecorded or mislabelled, some prices are missing or about to
go stale, a few pages are confusing, and four flows have never been verified with real use.

## Product intent and ideal flow

Every paid request made on the user's behalf is recorded under the right conversation with a label saying what it was,
priced from current published rates (or plainly "no price"), shown without misleading figures, and verified end to end
by an independent executor inside an enforced isolation gate.

## Product linkage

- Exact heading: **Product measurement** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “No commercial performance numbers have been supplied.”
- Feature: **PF-60** (continued; same feature as the completed plan, as PF-13-S07 continued into P1 security).
  Sprints PF-60-S06 to PF-60-S10.

## Scope

- In: items (i)–(viii) from Travis's 2026-10-10 decision, mapped below; catalogue rows that state provider prices.
- Out: shipping `/cost` to users (still developer-only); rebilling or re-pricing history; changing what Corbanu
  charges; collecting prompts; the open accounting issues not listed here unless Travis adds them.

| Item (Travis, 2026-10-10) | Issue | Sprint |
| --- | --- | --- |
| (i) Guardian reviews of a `/side` conversation, and sub-agents started from one, are not recorded | #400 | PF-60-S06 |
| (ii) The `side:` label also covers other temporary forks | #401 | PF-60-S06 |
| (iii) #368 prices: OpenRouter per-endpoint, Moonshot/BigModel routes, `gpt-6.1-sol`, Sol promotion end 2026-11-21, DeepSeek holidays | #368 | PF-60-S07 |
| (iv) NOT VERIFIABLE flows: ChatGPT login, image generation, realtime, TensorCash | #402 | PF-60-S09 |
| (v) #396 display follow-up ("Cache write cost: $0.000000" on an unpriced page) | #396 | PF-60-S08 |
| (vi) Paid warm-up request on every OpenAI reopen (~$0.025) should be visible or avoidable | #403 | PF-60-S08 |
| (vii) "(pay per use (set in your config))" nested parentheses | #396 | PF-60-S08 |
| (viii) S05 waiver (b): no enforced isolation gate | #404 | PF-60-S10 |

## Invariants

- PF-60's invariants hold: unknown stays unknown, never zero; subscription work is never spent; nothing is counted
  twice; history is never re-priced; no credential is printed or committed.
- Ledger format changes are versioned; older ledgers still read.

## Ownership and implementation worktrees

Unallocated. On activation each sprint gets its own worktree under `/Volumes/CorbanuDrive/Corbanu/worktrees/`, branch
and base commit, recorded here and in the sprint before it becomes `ready`.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `codex-rs/core/src/session/mod.rs::accounting_owner` | Where a session's requests are recorded and which label they get |
| `codex-rs/core/src/session_startup_prewarm.rs`, `codex-rs/core/src/accounting.rs::prewarm_turn_label` | Startup warm-up request and its label |
| `codex-rs/core/src/accounting_prices.rs`, `codex-rs/models-manager/models.json`, `scripts/check_openai_api_prices.py` | Price tables and the OpenAI price check |
| `codex-rs/tui/src/chatwidget/tokens.rs` | `/cost` request pages |
| `qa/code-blind-functional/isolated-execution.md`, `qa/code-blind-functional/check.py` | Enforced isolation contract and checker |

## Native lifecycle and upstream-touch record

Inherits PF-60's record. Warm-up (S08) and realtime (S09) touch upstream Codex paths; each sprint records its
upstream seams and keeps product code behind thin adapters per [upstream integration](../upstream-integration.md).

## Sprint execution map

All records belong to feature **PF-60**. S06–S09 are independent of each other; S10 accepts all four.

| Sprint | Record | Depends on | Items | Issues |
| --- | --- | --- | --- | --- |
| PF-60-S06 | [Side-conversation and temporary-fork attribution](../../sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s06-side-and-fork-attribution.md) | PF-60-S04 | (i), (ii) | #400, #401 |
| PF-60-S07 | [Price catalogue refresh](../../sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s07-price-catalogue-refresh.md) | PF-60-S04 | (iii) | #368 |
| PF-60-S08 | [`/cost` page clarity and reopen warm-up cost](../../sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s08-cost-page-clarity-and-reopen-warmup.md) | PF-60-S04 | (v), (vi), (vii) | #396, #403 |
| PF-60-S09 | [Verification paths for unverified flows](../../sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s09-verification-paths-for-unverified-flows.md) | PF-60-S04 | (iv) | #402 |
| PF-60-S10 | [Isolated code-blind acceptance of the follow-ups](../../sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s10-isolated-code-blind-acceptance.md) | PF-60-S06, PF-60-S07, PF-60-S08, PF-60-S09 | (viii) | #404 |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | A persisted conversation opens `/side`, which triggers a guardian review and spawns a sub-agent | Every request appears once under the parent conversation, each labelled with what it was |
| Failure/cancel | A request with no price, or a cancelled one | No money figure is invented; the page says why |
| Recovery/resume | Reopen an OpenAI conversation | Any warm-up request is either not sent or plainly shown as a reopen warm-up; earlier totals unchanged |

## Implementation sequence

1. S06, S07, S08 and S09 in any order, one at a time (`parallel_sprint_limit: 1`); S07 first if the Sol promotion
   date is near.
2. S10 last: one isolated code-blind acceptance of all four, then Travis.

## Automated evidence

Per sprint: `just test` for the touched crates with and without `developer-accounting`; Linux clippy `-D warnings`;
`python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`. Exact selectors go in each sprint.

## True-TUI evidence

Each user-facing sprint: a tmux run with real keys on GLM 5.3 Flash or the provider under test, in disposable homes
with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, plus demo videos per `qa/demos/README.md`. Result: pending.

## Live-repository applicability

| Repository | Applicability for this scope | Checkout/base | Result |
| --- | --- | --- | --- |
| TensorCash | Required by S09 (was NOT VERIFIABLE in PF-60-S04) | Access to be resolved in S09 | pending |
| Isometric Game | Not applicable: accounting doesn't change repository work | n/a | n/a |

## Human acceptance

Travis accepts each sprint and the S10 acceptance. Agent review never substitutes for it.

## Documentation

Finished-feature docs change only if Travis authorises shipping `/cost`; until then evidence stays under
`qa/portfolio/agent-cost-accounting/`.

## Dependencies, decisions, and blockers

- **Decisions for Travis:** activation and slot; S07's OpenRouter rule (reported charge first, catalogue as a labelled
  fallback, or "no price"); S08's warm-up choice (avoid or show; recommendation in the sprint); S09's ChatGPT test
  account and TensorCash access; whether S07's Sol promotion item may land as a bounded fix before 2026-11-21 if this
  plan is not active by then.
- **Isolation infrastructure** (S10) is shared with P1 security's milestone runs; reuse it rather than build twice.
- **Open accounting issues not assigned here** (Travis to place or leave): #352 `/cost` display nits, #359 busy state
  DB waits, #325 `/cost` leftovers, #324 delete-time retention sweep, #314 invisible unrecorded gaps, #290
  `exec --json` usage zeros, #127 Grok tiered pricing, #129 OpenRouter cap reconciliation.

## Release linkage

None yet. Carried from PF-60: collection is developer-only; a release needs Travis's authorisation to ship `/cost`.

## Completion

- [ ] Every item (i)–(viii) is closed by an accepted sprint or by Travis's recorded decision.
- [ ] S10's isolated acceptance passes the code-blind checker; Travis accepts.
- [ ] Linked issues are closed or re-filed with a reason.
