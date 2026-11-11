---
title: "Accounting follow-ups after PF-60"
status: completed
change_class: product-initiative
priority: P1
owner: "Codex accounting lane; Travis Good accepts"
parallel_sprint_limit: 1
integration_owner: "Codex coordinator"
activation_authority: "Travis Good"
activation_basis: "Travis, 2026-10-10: accept PF-60-S04's remaining gaps on condition that each is carried into a future sprint. Travis, 2026-10-14 (decision 29): start PF-60-S06 to S10 after the Moderate picker sprint. Activated in slot 2, freed when the coordinator's worker closed the provider-onboarding plan (PF-84 sprints all archived); Travis confirmed that closing on 2026-10-14 (#574)."
target_release: "None: closed without a release; /cost stays developer-only"
deadline: "TBD (S07's Sol promotion end landed in #762 before 2026-11-21)"
created: 2026-10-10
updated: 2026-10-20
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Product measurement"
  requirement_excerpt: "No commercial performance numbers have been supplied."
implementation_worktrees:
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s06"
    branch: "pf60-s06-side-attribution"
    base_commit: "ecb9069c4b3e0bdb20cb88d520fc8b54745cea34"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s07"
    branch: "pf60-s07/moonshot-route-pricing"
    base_commit: "df3d8c5d1c521228a8dbb1035831af572afe1284"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s08"
    branch: "pf60-s08/cost-clarity-reopen-warmup"
    base_commit: "45916ab64ef7faf4a5525a8dbdbfd4f706dd9f79"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s09"
    branch: "pf60-s09/verification-paths"
    base_commit: "643838f2f4ed87dfe71be24f983a6dcc91709faa"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s10"
    branch: "qa/pf60-s10-acceptance"
    base_commit: "395ac0adcd96f4970220e569a1608141a3e748f3"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/acc-tart1"
    branch: "qa/acc-tart1-followup-acceptance"
    base_commit: "ab854bb7a6"
---

# Accounting follow-ups after PF-60

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md). Predecessor:
[PF-60 accounting](../completed/main-2026-10-10-portfolio-agent-cost-accounting.md), completed 2026-10-10.
Active from 2026-10-14 in slot 2 (Travis's decision); one sprint at a time. **Completed 2026-10-20** without a
release: the follow-up live run is recorded, and slot 2 is free.

**Status 2026-10-16:** all five sprints accepted by Travis and archived (S06 2026-10-14; S07–S09 2026-10-15; S10
2026-10-16, 29 passed / 1 failed / 7 blocked of 37, decision D24). The last item, the follow-up isolated live run Travis set on 2026-10-16
(D25, D3, D16), is recorded below.

**Status 2026-10-20:** the follow-up live run ran on Tart (D34) and is recorded in
[followups-live-20261020](../../../qa/portfolio/agent-cost-accounting/followups-live-20261020/README.md): 25 cases,
18 passed, 2 failed, 5 blocked. Travis's acceptance of the run is pending.
- All three D25 gaps are closed: ACC-06 `/review` cost, Request IDs re-read after reopens, and the `side:` screen.
- D16: the realtime session passed. It cost about $0.005, but app-server text warm-ups bring the check to about
  $0.063 (one warm-up is probable, from the shakedown). That is over the $0.05 cap; see #1045.
- D3: real Moonshot cache reads are shown at the conversation level. The per-request technical screens were not
  reached, so three cases are blocked.
- Three new issues, not carried here (Travis to place): #1038, editing the first message sends an unlabelled warm-up;
  #1039, the `/review`-in-`/side` refusal text is erased; #1045, the realtime path's paid text warm-up.

## Activation record

- Status: **completed** 2026-10-20 (active from 2026-10-14, slot 2). Created 2026-10-10 as a draft when PF-60 closed to free slot 2 for PF-84.
- Activation: Travis, 2026-10-14 (decision 29 of that day's list): start S06–S10 after the Moderate picker sprint
  (done: #764, #781). All three slots were full; the coordinator's worker closed the
  [provider-onboarding plan](../completed/main-2026-10-14-unified-provider-auth.md), whose PF-84 sprints were all
  accepted and archived, to free slot 2, and moved its flag-removal work to a
  [draft plan](../proposed/named-accounts-flag-removal.md). Travis confirmed that closing on 2026-10-14 (comment on
  #574).
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

Each sprint gets its own worktree under `/Volumes/CorbanuDrive/Corbanu/worktrees/`, branch and base commit, recorded
here and in the sprint before it becomes `ready`.

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Codex worker `pf60-s06` | `/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s06` | `pf60-s06-side-attribution` | `ecb9069c4b3e0bdb20cb88d520fc8b54745cea34` | PF-60-S06 (lane `accounting`) |
| Codex worker `pf60-next` | `/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s07` | `pf60-s07/moonshot-route-pricing` | `df3d8c5d1c521228a8dbb1035831af572afe1284` | PF-60-S07 (lane `accounting`) |
| Codex worker `pf60-next` | `/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s08` | `pf60-s08/cost-clarity-reopen-warmup` | `45916ab64ef7faf4a5525a8dbdbfd4f706dd9f79` | PF-60-S08 (lane `accounting`) |
| Codex worker `pf60-next` | `/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s09` | `pf60-s09/verification-paths` | `643838f2f4ed87dfe71be24f983a6dcc91709faa` | PF-60-S09 (lane `accounting`) |
| Codex worker `pf60-s10` | `/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s10` | `qa/pf60-s10-acceptance` | `395ac0adcd96f4970220e569a1608141a3e748f3` | PF-60-S10 (record only, under `qa/`) |
| Codex worker `acc-tart1` | `/Volumes/CorbanuDrive/Corbanu/worktrees/acc-tart1` | `qa/acc-tart1-followup-acceptance` | `ab854bb7a6` | Follow-up live run (record only, under `qa/`) |

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

All records belong to feature **PF-60**. S06–S09 are independent of each other; S10 accepts all four. All five are
completed and archived (2026-10-16).

| Sprint | Record | Depends on | Items | Issues |
| --- | --- | --- | --- | --- |
| PF-60-S06 | [Side-conversation and temporary-fork attribution](../../sprints/archive/portfolio-agent-cost-accounting-followups/pf-60-s06-side-and-fork-attribution.md) | PF-60-S04 | (i), (ii) | #400, #401 |
| PF-60-S07 | [Price catalogue refresh](../../sprints/archive/portfolio-agent-cost-accounting-followups/pf-60-s07-price-catalogue-refresh.md) | PF-60-S04 | (iii) | #368 |
| PF-60-S08 | [`/cost` page clarity and reopen warm-up cost](../../sprints/archive/portfolio-agent-cost-accounting-followups/pf-60-s08-cost-page-clarity-and-reopen-warmup.md) | PF-60-S04 | (v), (vi), (vii) | #396, #403 |
| PF-60-S09 | [Verification paths for unverified flows](../../sprints/archive/portfolio-agent-cost-accounting-followups/pf-60-s09-verification-paths-for-unverified-flows.md) | PF-60-S04 | (iv) | #402 |
| PF-60-S10 | [Isolated code-blind acceptance of the follow-ups](../../sprints/archive/portfolio-agent-cost-accounting-followups/pf-60-s10-isolated-code-blind-acceptance.md) | PF-60-S06, PF-60-S07, PF-60-S08, PF-60-S09 | (viii) | #404 |

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
| TensorCash | Not applicable until access is granted (Travis, 2026-10-14); S09 recorded it | n/a | not applicable |
| Isometric Game | Not applicable: accounting doesn't change repository work | n/a | n/a |

## Human acceptance

Travis accepts each sprint and the S10 acceptance. Agent review never substitutes for it.

## Documentation

Finished-feature docs change only if Travis authorises shipping `/cost`; until then evidence stays under
`qa/portfolio/agent-cost-accounting/`.

## Dependencies, decisions, and blockers

- **Decided:** activation (Travis, 2026-10-14; slot 2); S07's OpenRouter rule (Travis, 2026-10-14: the reported
  charge first, the catalogue rate as a labelled estimate, else "no price"; PR #784); the Sol promotion end landed as
  a bounded fix (#762).
- **Decided 2026-10-14 (Travis, all as recommended):** S06 accepted as limited testing (S10 runs it code-blind); the
  PF-84 plan closing confirmed; S07 prices Moonshot by route first (BigModel deferred) and prices Vercel and Corbanu
  API requests from their reported charges, as OpenRouter's; #786 option (a), a reasoning count above the output count
  is recorded as not reported; S08 takes #403 option (C), the reopen warm-up shown in `/cost` and on reopen with a
  setting to skip it (off by default); S09 verifies ChatGPT login with a mock sign-in server only, records TensorCash
  as not applicable until access is granted, and runs image generation and realtime for real on an OpenAI API key.
- **Decided 2026-10-15 (Travis, "1-6 as recommended"):** accept S07, S08 and S09 (S09 with the realtime failure in
  #824, sent for a fix as option (a), done in #882); #814 option (b); S10 runs on S06–S09 before they are archived.
- **Decided 2026-10-16 (Travis, "all as recommended"):** D24 accept S10 as recorded (29/1/7) and archive S06–S10;
  D25 re-check the S10 gaps in #898's acceptance run, not a full S10 re-run; D26 accept S10's wording-only r3 fail;
  D3 one real Moonshot cache read (under $0.01) and D16 one real OpenAI realtime session (under $0.05) in that run;
  D14 ledger format 6; D15 fix #889 item 1 (wrong account grouping).
- **Done 2026-10-20 (acc-tart1):** the follow-up live run ([record](../../../qa/portfolio/agent-cost-accounting/followups-live-20261020/README.md)). **Was remaining before completion:** one isolated live run with ACC-06 (#898, fixed by #917), the Request-ID checks of
  ACC-11/14/18, ACC-15's `side:` screen, the Moonshot cache read and the realtime session. Then complete the plan
  (no release) and free slot 2. Not carried here: #368's BigModel and DeepSeek 2027 items, #824 (WebRTC), #889, #912,
  #913, #814's rare cases.
- **Isolation infrastructure** (S10) is shared with P1 security's milestone runs; reuse it rather than build twice.
- **Open accounting issues not assigned here** (Travis to place or leave): #352 `/cost` display nits, #359 busy state
  DB waits, #325 `/cost` leftovers, #324 delete-time retention sweep, #314 invisible unrecorded gaps, #290
  `exec --json` usage zeros, #127 Grok tiered pricing, #129 OpenRouter cap reconciliation.

## Release linkage

None yet. Carried from PF-60: collection is developer-only; a release needs Travis's authorisation to ship `/cost`.

## Completion

- [x] Every item (i)–(viii) is closed by an accepted sprint or by Travis's recorded decision (2026-10-14 to 2026-10-16).
- [x] S10's isolated acceptance: Travis accepted it as recorded on 2026-10-16 (D24). The checker stops at the first
      unresolved case (ACC-04), as expected; the gaps go to one follow-up live run (D25).
- [x] Linked issues are closed or re-filed with a reason: #396, #400–#404 and #786 closed with the archive (#786's two
      remaining rare cases are tracked in #814); #368 kept open for BigModel and DeepSeek's 2027 dates; #824 for WebRTC.
- [x] PF-60-S05's waiver (b) is discharged by S10's isolated run (Travis's acceptance, D24; dated addendum in the
      archived S05 record).
- [x] The follow-up isolated live run (D25, D3, D16) is recorded: acc-tart1 on Tart, 2026-10-20,
      [record](../../../qa/portfolio/agent-cost-accounting/followups-live-20261020/README.md) (18/2/5 of 25; Travis's
      acceptance pending; new issues #1038, #1039 and #1045 left for Travis to place). The plan moved to `completed/`.
