# P0 security-level execution sprints

The active [P0 `/security` plan](../../../plans/active/p0-security-levels.md)
owns these **63 current sprints**: 23 retained/reconciled records and 40 new
records from the 2026-08-28 full-scope planning decision.
[Sources and all 72 archive dispositions](../../../plans/security-source-reconciliation.md)
explain the reuse and additions. Cancelled records are history, not dependencies.

[OpenClaw implementation review](../../../plans/openclaw-source-review-2026-08-28.md)
pins the current adoption reference and names callers, defaults, tests and gaps.
Each affected sprint adds explicit adoption regressions; passing reference probes
is not completion evidence. The review changes no sprint order or readiness.

Work only from one sprint's Remaining checklist. PF-15-S01 is the only ready
record; all other records remain draft until their dependencies are completed
and archived. Seven foundation commits are recorded as present, not accepted.
The implementation worktree/branch/base remain those allocated in the plan;
this documentation checkout is not an implementation worktree.

Order below is topological, not a duration estimate. The first credential boundary
is qualified before broader research, browser and financial integrations; final
PF-26 qualification covers every feature. Permissive stays compatible, while
Moderate and Aggressive require the broker and their full readiness conditions.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [PF-15-S01](pf-15-s01-security-level-domain-foundation.md) | Security-level domain foundation reconciliation | ready | — |
| 2 | [PF-16-S01](pf-16-s01-authorization-decision-contract.md) | Authorization decision contract reconciliation | draft | PF-15-S01 |
| 3 | [PF-17-S01](pf-17-s01-bounded-delegation-grants.md) | Bounded delegation grants reconciliation | draft | PF-16-S01 |
| 4 | [PF-18-S01](pf-18-s01-human-mandates-and-receipts.md) | Human mandates and receipts reconciliation | draft | PF-16-S01 |
| 5 | [PF-19-S01](pf-19-s01-revocation-contract.md) | Revocation contract reconciliation | draft | PF-17-S01, PF-18-S01 |
| 6 | [PF-20-S01](pf-20-s01-versioned-security-persistence.md) | Versioned security persistence reconciliation | draft | PF-15-S01 |
| 7 | [PF-21-S01](pf-21-s01-permissive-compatibility-baseline.md) | Permissive compatibility baseline reconciliation | draft | PF-16-S01, PF-20-S01 |
| 8 | [PF-22-S01](pf-22-s01-runtime-policy-and-agent-inheritance.md) | Runtime policy and agent inheritance | draft | PF-19-S01, PF-20-S01, PF-21-S01 |
| 9 | [PF-13-S01](pf-13-s01-vault-backed-exact-host-credential-substitution.md) | Typed credential capability and bounded store | draft | PF-16-S01, PF-17-S01, PF-19-S01, PF-22-S01 |
| 10 | [PF-13-S02](pf-13-s02-scoped-vault-resolver.md) | Scoped vault resolver | draft | PF-13-S01 |
| 11 | [PF-13-S03](pf-13-s03-openai-exact-host-proxy-substitution.md) | OpenAI exact-host proxy substitution | draft | PF-13-S02 |
| 12 | [PF-13-S04](pf-13-s04-authority-lifecycle-and-raw-secret-bypass.md) | Credential authority lifecycle and raw-secret bypass closure | draft | PF-13-S03 |
| 13 | [PF-27-S01](pf-27-s01-isolated-credential-broker.md) | Isolated credential broker process | draft | PF-13-S04 |
| 14 | [PF-27-S02](pf-27-s02-secretless-agent-launch.md) | Secretless agent launch and bypass containment | draft | PF-27-S01 |
| 15 | [PF-28-S01](pf-28-s01-central-secret-output-gate.md) | Central secret and protected-output gate | draft | PF-27-S02 |
| 16 | [PF-28-S02](pf-28-s02-reflected-secret-response-scrubbing.md) | Reflected-secret response scrubbing | draft | PF-28-S01 |
| 17 | [PF-33-S01](pf-33-s01-url-dns-and-redirect-policy.md) | URL DNS and redirect policy | draft | PF-27-S02 |
| 18 | [PF-33-S02](pf-33-s02-connection-pinning-and-bypass.md) | Connection pinning and alternate-egress denial | draft | PF-33-S01 |
| 19 | [PF-24-S01](pf-24-s01-security-command-and-profile-view.md) | Security command and profile view | draft | PF-20-S01, PF-22-S01 |
| 20 | [PF-29-S01](pf-29-s01-protected-mode-inventory.md) | Protected-mode inventory and activation preflight | draft | PF-28-S02, PF-20-S01 |
| 21 | [PF-29-S02](pf-29-s02-human-secret-migration.md) | Human-reviewed credential migration and recovery | draft | PF-29-S01, PF-24-S01 |
| 22 | [PF-13-S05](pf-13-s05-credential-boundary-adversarial-qualification.md) | Credential boundary adversarial qualification | draft | PF-13-S04, PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02 |
| 23 | [PF-30-S01](pf-30-s01-typed-source-envelope.md) | Typed source envelope and trusted ingress | draft | PF-22-S01 |
| 24 | [PF-30-S02](pf-30-s02-persistent-taint-and-memory.md) | Persistent taint across summaries and memory | draft | PF-30-S01 |
| 25 | [PF-30-S03](pf-30-s03-post-taint-authority-checks.md) | Post-taint authority checks | draft | PF-30-S02, PF-13-S05 |
| 26 | [PF-23-S01](pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) | Moderate ingress and disclosure enforcement | draft | PF-13-S05, PF-22-S01, PF-30-S03 |
| 27 | [PF-23-S02](pf-23-s02-aggressive-deny-and-grant-enforcement.md) | Aggressive deny and grant enforcement | draft | PF-17-S01, PF-23-S01 |
| 28 | [PF-23-S03](pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) | Downgrade, restart, and inheritance enforcement | draft | PF-19-S01, PF-20-S01, PF-23-S02 |
| 29 | [PF-24-S02](pf-24-s02-security-confirm-cancel-and-downgrade.md) | Security confirm, cancel, and downgrade | draft | PF-23-S03, PF-24-S01, PF-29-S02 |
| 30 | [PF-25-S01](pf-25-s01-temporary-grant-tui.md) | Temporary grant TUI | draft | PF-17-S01, PF-23-S02, PF-24-S02 |
| 31 | [PF-25-S02](pf-25-s02-revocation-and-kill-switch-tui.md) | Revocation and kill-switch TUI | draft | PF-19-S01, PF-23-S03, PF-25-S01 |
| 32 | [PF-31-S01](pf-31-s01-pinned-retriever-isolation.md) | Pinned retriever artifact and sandbox | draft | PF-33-S02, PF-27-S02 |
| 33 | [PF-31-S02](pf-31-s02-bounded-fetch-no-fallback.md) | Bounded fetch adapter with no host fallback | draft | PF-31-S01, PF-30-S01 |
| 34 | [PF-31-S03](pf-31-s03-download-quarantine-promotion.md) | Download quarantine and human file promotion | draft | PF-31-S02, PF-24-S01 |
| 35 | [PF-34-S01](pf-34-s01-render-aware-sanitization.md) | Render-aware content sanitization | draft | PF-31-S02, PF-30-S01 |
| 36 | [PF-35-S01](pf-35-s01-classifier-corpus-and-evaluation.md) | Classifier corpus and leakage-free evaluation | draft | PF-34-S01 |
| 37 | [PF-35-S02](pf-35-s02-local-cpu-detector-artifact.md) | Reproducible local CPU detector artifact | draft | PF-35-S01 |
| 38 | [PF-35-S03](pf-35-s03-calibration-and-ingress-gate.md) | Calibrated detector and ingress enforcement | draft | PF-35-S02 |
| 39 | [PF-34-S02](pf-34-s02-quarantine-state-and-store.md) | Quarantine state and encrypted retention | draft | PF-35-S03 |
| 40 | [PF-34-S03](pf-34-s03-safe-quarantine-review.md) | Safe quarantine review and recovery | draft | PF-34-S02, PF-24-S01 |
| 41 | [PF-32-S01](pf-32-s01-web-facade-and-registry.md) | Stable web facade and provider registry | draft | PF-34-S03, PF-31-S03, PF-13-S05 |
| 42 | [PF-32-S02](pf-32-s02-existing-search-and-native-bypass.md) | Existing search adapter and native bypass closure | draft | PF-32-S01 |
| 43 | [PF-32-S03](pf-32-s03-exa-search-adapter.md) | Exa brokered search adapter | draft | PF-32-S02 |
| 44 | [PF-32-S04](pf-32-s04-brave-search-adapter.md) | Brave brokered search adapter | draft | PF-32-S02 |
| 45 | [PF-32-S05](pf-32-s05-searxng-search-adapter.md) | SearXNG brokered search adapter | draft | PF-32-S02 |
| 46 | [PF-32-S06](pf-32-s06-privacy-routing-and-failover.md) | Private query routing and bounded failover | draft | PF-32-S03, PF-32-S04, PF-32-S05 |
| 47 | [PF-36-S01](pf-36-s01-hosted-detector-consent-contract.md) | Optional hosted detector consent contract | draft | PF-35-S03, PF-33-S02 |
| 48 | [PF-36-S02](pf-36-s02-hosted-bakeoff-and-local-fallback.md) | Hosted detector bakeoff and safe local fallback | draft | PF-36-S01 |
| 49 | [PF-37-S01](pf-37-s01-origin-bound-browser-login.md) | Origin-bound brokered browser login | draft | PF-31-S03, PF-28-S02, PF-30-S03 |
| 50 | [PF-37-S02](pf-37-s02-human-auth-handoff-lifecycle.md) | Human authentication handoff and session revocation | draft | PF-37-S01, PF-25-S02 |
| 51 | [PF-38-S01](pf-38-s01-typed-financial-executor.md) | Typed financial executor and deterministic limits | draft | PF-30-S03, PF-27-S02, PF-18-S01 |
| 52 | [PF-38-S02](pf-38-s02-full-effect-preview-and-mandate.md) | Full-effect financial preview and exact mandate | draft | PF-38-S01, PF-25-S01 |
| 53 | [PF-38-S03](pf-38-s03-sign-broadcast-and-receipts.md) | Separate signing broadcasting and idempotent receipts | draft | PF-38-S02 |
| 54 | [PF-39-S01](pf-39-s01-protected-financial-derived-views.md) | Protected financial derived views | draft | PF-28-S01, PF-38-S01 |
| 55 | [PF-39-S02](pf-39-s02-outbound-disclosure-controls.md) | Outbound disclosure clipboard and export controls | draft | PF-39-S01, PF-30-S03, PF-32-S06 |
| 56 | [PF-40-S01](pf-40-s01-sweep-events-and-rules.md) | Agent Sweep sanitized events and deterministic rules | draft | PF-30-S03, PF-38-S03, PF-39-S02 |
| 57 | [PF-40-S02](pf-40-s02-isolated-sweep-reviewer.md) | Isolated advisory Agent Sweep reviewer | draft | PF-40-S01, PF-36-S01 |
| 58 | [PF-40-S03](pf-40-s03-sweep-alerts-and-recovery.md) | Agent Sweep alerts revocation and recovery | draft | PF-40-S02, PF-25-S02 |
| 59 | [PF-41-S01](pf-41-s01-effective-security-inspector.md) | Effective security inspector and degradation state | draft | PF-23-S03, PF-29-S02, PF-32-S06, PF-37-S02, PF-40-S03, PF-24-S02 |
| 60 | [PF-41-S02](pf-41-s02-tamper-evident-security-audit.md) | Tamper-evident audit and safe support export | draft | PF-41-S01 |
| 61 | [PF-26-S01](pf-26-s01-security-harnesses-and-standards-crosswalk.md) | Security harnesses and standards crosswalk | draft | PF-13-S05, PF-21-S01, PF-23-S03, PF-25-S02, PF-36-S02, PF-41-S02 |
| 62 | [PF-26-S02](pf-26-s02-true-tui-and-live-repository-qualification.md) | True-TUI and live-repository qualification | draft | PF-26-S01 |
| 63 | [PF-26-S03](pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | Human acceptance, finished docs, and release evidence | draft | PF-26-S02 |

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
