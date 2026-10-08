---
title: "P1 security hardening"
status: draft
change_class: product-initiative
priority: P1
owner: "Jim Ricketts"
parallel_sprint_limit: 1
integration_owner: "Codex /root security round-five coordinator"
activation_authority: "Product authority defined in the product specification"
activation_basis: "Travis Good's 2026-10-06 decision 3: deferred security features leave the P0 core and form this plan."
target_release: "TBD"
deadline: "TBD"
created: 2026-10-06
updated: 2026-10-08
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "P0 /security levels"
  requirement_excerpt: "Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged."
implementation_worktrees:
  - path: "UNALLOCATED"
    branch: "UNALLOCATED"
    base_commit: "UNALLOCATED"
---

# P1 security hardening

Policy: repository-root `AGENTS.md`. Plan lifecycle: `docs/plans/index.md`.

This draft holds the security features Travis deferred on 2026-10-06 (decision 3)
from the [P0 `/security` plan](../active/p0-security-levels.md). It authorizes no
implementation: it activates only through the plan lifecycle once an active slot
is free. The sprint records moved unchanged except for their `plan_file`, plan
link and date; every dependency is intact, including the ones on P0 core sprints.
The full contracts and their history are in the
[P0 history file](../history/p0-security-levels-2026-10-06.md#expanded-feature-contracts).

## Features

| Feature | Contract (one line) |
| --- | --- |
| PF-27 | Windows only: broker and secretless launch (S06, moved from P0 on 2026-10-06), its hardening follow-ups (S07), the broker's own token (S08) and model-client auth through the broker (S09), added 2026-10-08 when Travis approved fixing S06's four documented limits. |
| PF-26 | Final whole-program qualification: automated (S04), true-TUI and live repositories (S02), human acceptance and finished docs (S03). The P0 milestone gates replace it for the core. |
| PF-31 | Isolated public retrieval with no host-browser fallback; sealed downloads until exact human promotion. |
| PF-32 | Screened web facade: existing search, Exa, Brave and SearXNG adapters, private routing and bounded failover. |
| PF-34 | Render-aware sanitization, encrypted quarantine and safe human review. |
| PF-35 | Licensed offline CPU injection detector with leakage-free blind qualification (targets below). |
| PF-36 | Optional hosted detector: consent contract, bakeoff, safe local fallback; no vendor assumed. |
| PF-37 | Origin-bound brokered browser login with human MFA/CAPTCHA handoff and revocation. |
| PF-38 | Typed financial executor, full-effect preview and exact mandate, separate sign and broadcast. |
| PF-39 | Purpose-limited derived financial views and outbound disclosure, clipboard and export controls. |
| PF-40 | Agent Sweep: sanitized events, deterministic rules, isolated advisory reviewer, alerts and recovery. |
| PF-41 | S02 only: tamper-evident audit and safe support export. PF-41-S01 (inspector) stays in P0. |

## Sprint execution map

Order is topological, carried over from the P0 plan. Dependencies on P0 core
sprints (PF-23, PF-24, PF-25, PF-27, PF-28, PF-30, PF-33, PF-41-S01, PF-13-S07)
must complete there first.

| Order | Sprint | Outcome | Depends on |
| ---: | --- | --- | --- |
| 20 | [PF-35-S01](../../sprints/current/p1-security-hardening/pf-35-s01-classifier-corpus-and-evaluation.md) | Classifier corpus and leakage-free evaluation | PF-34-S04 |
| 21 | [PF-35-S02](../../sprints/current/p1-security-hardening/pf-35-s02-local-cpu-detector-artifact.md) | Reproducible local CPU detector artifact | PF-35-S01 |
| 42 | [PF-27-S06](../../sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md) | Windows broker and secretless launch | PF-27-S02 |
| 43 | [PF-27-S07](../../sprints/current/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md) | Windows hardening follow-ups: new threads protected at creation, `CODEX_HOME` deny removed on flag-off | PF-27-S06 |
| 44 | [PF-27-S08](../../sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) | Windows broker confined by its own restricted token or AppContainer | PF-27-S07 |
| 45 | [PF-27-S09](../../sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md) | Windows model-client auth through the broker | PF-27-S05, PF-27-S07 |
| 46 | [PF-31-S01](../../sprints/current/p1-security-hardening/pf-31-s01-pinned-retriever-isolation.md) | Pinned retriever artifact and sandbox | PF-33-S02, PF-27-S02, PF-31-S04 |
| 47 | [PF-31-S02](../../sprints/current/p1-security-hardening/pf-31-s02-bounded-fetch-no-fallback.md) | Bounded fetch adapter with no host fallback | PF-31-S01, PF-30-S01 |
| 48 | [PF-31-S03](../../sprints/current/p1-security-hardening/pf-31-s03-download-quarantine-promotion.md) | Download quarantine and human file promotion | PF-31-S02, PF-24-S01 |
| 49 | [PF-34-S01](../../sprints/current/p1-security-hardening/pf-34-s01-render-aware-sanitization.md) | Render-aware content sanitization | PF-31-S02, PF-30-S01, PF-34-S04 |
| 50 | [PF-35-S03](../../sprints/current/p1-security-hardening/pf-35-s03-calibration-and-ingress-gate.md) | Calibrated detector and ingress enforcement | PF-35-S02, PF-34-S01, PF-30-S03, PF-23-S01 |
| 51 | [PF-34-S02](../../sprints/current/p1-security-hardening/pf-34-s02-quarantine-state-and-store.md) | Quarantine state and encrypted retention | PF-35-S03, PF-41-S03 |
| 52 | [PF-34-S03](../../sprints/current/p1-security-hardening/pf-34-s03-safe-quarantine-review.md) | Safe quarantine review and recovery | PF-34-S02, PF-24-S01 |
| 53 | [PF-32-S01](../../sprints/current/p1-security-hardening/pf-32-s01-web-facade-and-registry.md) | Stable web facade and provider registry | PF-34-S03, PF-31-S03, PF-13-S05 |
| 54 | [PF-32-S02](../../sprints/current/p1-security-hardening/pf-32-s02-existing-search-and-native-bypass.md) | Existing search adapter and native bypass closure | PF-32-S01 |
| 55 | [PF-32-S03](../../sprints/current/p1-security-hardening/pf-32-s03-exa-search-adapter.md) | Exa brokered search adapter | PF-32-S02 |
| 56 | [PF-32-S04](../../sprints/current/p1-security-hardening/pf-32-s04-brave-search-adapter.md) | Brave brokered search adapter | PF-32-S02 |
| 57 | [PF-32-S05](../../sprints/current/p1-security-hardening/pf-32-s05-searxng-search-adapter.md) | SearXNG brokered search adapter | PF-32-S02 |
| 58 | [PF-32-S06](../../sprints/current/p1-security-hardening/pf-32-s06-privacy-routing-and-failover.md) | Private query routing and bounded failover | PF-32-S03, PF-32-S04, PF-32-S05 |
| 59 | [PF-36-S01](../../sprints/current/p1-security-hardening/pf-36-s01-hosted-detector-consent-contract.md) | Optional hosted detector consent contract | PF-35-S03, PF-33-S02 |
| 60 | [PF-36-S02](../../sprints/current/p1-security-hardening/pf-36-s02-hosted-bakeoff-and-local-fallback.md) | Hosted detector bakeoff and safe local fallback | PF-36-S01 |
| 61 | [PF-37-S01](../../sprints/current/p1-security-hardening/pf-37-s01-origin-bound-browser-login.md) | Origin-bound brokered browser login | PF-31-S03, PF-28-S02, PF-30-S03, PF-34-S03 |
| 62 | [PF-37-S02](../../sprints/current/p1-security-hardening/pf-37-s02-human-auth-handoff-lifecycle.md) | Human authentication handoff and session revocation | PF-37-S01, PF-25-S02 |
| 63 | [PF-38-S01](../../sprints/current/p1-security-hardening/pf-38-s01-typed-financial-executor.md) | Typed financial executor and deterministic limits | PF-30-S03, PF-27-S02, PF-18-S01 |
| 64 | [PF-38-S02](../../sprints/current/p1-security-hardening/pf-38-s02-full-effect-preview-and-mandate.md) | Full-effect financial preview and exact mandate | PF-38-S01, PF-25-S01 |
| 65 | [PF-38-S03](../../sprints/current/p1-security-hardening/pf-38-s03-sign-broadcast-and-receipts.md) | Separate signing broadcasting and idempotent receipts | PF-38-S02, PF-41-S03 |
| 66 | [PF-39-S01](../../sprints/current/p1-security-hardening/pf-39-s01-protected-financial-derived-views.md) | Protected financial derived views | PF-28-S01, PF-38-S01 |
| 67 | [PF-39-S02](../../sprints/current/p1-security-hardening/pf-39-s02-outbound-disclosure-controls.md) | Outbound disclosure clipboard and export controls | PF-39-S01, PF-30-S03, PF-32-S06 |
| 68 | [PF-40-S01](../../sprints/current/p1-security-hardening/pf-40-s01-sweep-events-and-rules.md) | Agent Sweep sanitized events and deterministic rules | PF-30-S03, PF-38-S03, PF-39-S02, PF-41-S03 |
| 69 | [PF-40-S02](../../sprints/current/p1-security-hardening/pf-40-s02-isolated-sweep-reviewer.md) | Isolated advisory Agent Sweep reviewer | PF-40-S01, PF-36-S01 |
| 70 | [PF-40-S03](../../sprints/current/p1-security-hardening/pf-40-s03-sweep-alerts-and-recovery.md) | Agent Sweep alerts revocation and recovery | PF-40-S02, PF-25-S02 |
| 72 | [PF-41-S02](../../sprints/current/p1-security-hardening/pf-41-s02-tamper-evident-security-audit.md) | Tamper-evident audit and safe support export | PF-41-S01, PF-41-S03 |
| 74 | [PF-26-S04](../../sprints/current/p1-security-hardening/pf-26-s04-final-automated-qualification.md) | Final integrated automated security qualification | PF-26-S01, PF-13-S07, PF-21-S02, PF-23-S03, PF-25-S02, PF-36-S02, PF-41-S02 |
| 75 | [PF-26-S02](../../sprints/current/p1-security-hardening/pf-26-s02-true-tui-and-live-repository-qualification.md) | True-TUI and live-repository qualification | PF-26-S04 |
| 76 | [PF-26-S03](../../sprints/current/p1-security-hardening/pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | Human acceptance, finished docs, and release evidence | PF-26-S02 |

PF-35-S01 keeps its external-campaign status: DeepSeek API generation through
Corbanu Terminal (Travis, 2026-09-11), fine-tuning on the RTX PRO 6000, offline
CPU inference.

### Local classifier qualification targets

PF-35's qualification baseline, carried from the P0 plan. Changing a target needs product review.

| Measure | Target |
| --- | --- |
| Benign false positives | ≤0.1% on ≥100,000 held-out benign segments, with confidence interval |
| Known-family detection | ≥80% at that low-FPR operating point |
| Unseen-source/evasion detection | ≥65% at the same threshold; every miss remains policy-contained |
| Benign position/trigger perturbations | ≤2 percentage-point rejection increase |
| CPU envelope | p95 ≤50 ms per 2,048-token segment; peak RSS ≤512 MiB; model ≤300 MiB |
| Privacy | No real customer secrets/protected financial records in corpus, hosted payloads, metrics or artifacts |
| Deterministic safety | All critical unauthorized-disclosure/action cases deny, including forced detector misses |

## Activation prerequisites

- A free active-plan slot and a product decision to activate it.
- The P0 core's flag-removal milestone, or an explicit decision to overlap.
- Exact worktrees, the delivery gate (per-sprint videos plus milestone code-blind runs, as in P0) and a lane allocation recorded here.
- Open external inputs from the P0 history: login origin and test account (PF-37),
  hosted vendor and data terms (PF-36), corpus, licences and hardware pins (PF-35).
