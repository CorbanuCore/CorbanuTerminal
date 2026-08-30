# P0 security-level execution sprints

The active [P0 security plan](../../../plans/active/p0-security-levels.md) owns
**54 current sprints** and **22 completed archives**.
The integrated Sprint 13 branch completes PF-13-S02–S04 and the early PF-26-S01
harness; PF-13-S05 is also completed and archived after its integrated Core and
platform qualification. PF-13-S07 is the final composed credential-boundary gate. PF-27-S01 is the
accepted shared-contract foundation, while PF-27-S04 owns the refactored isolated
broker. Superseded pre-reconciliation PF-28–30 planning records do not compete
with this canonical PF-27–41 graph.

All review follow-ups remain explicit. Up to three independent allocations follow
the checked scope/integration rules. Order is topological, not a duration estimate;
archived evidence proves only its recorded candidate and scope.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 20 | [PF-35-S01](pf-35-s01-classifier-corpus-and-evaluation.md) | Classifier corpus and leakage-free evaluation | draft | PF-34-S04 |
| 21 | [PF-35-S02](pf-35-s02-local-cpu-detector-artifact.md) | Reproducible local CPU detector artifact | draft | PF-35-S01 |
| 22 | [PF-13-S06](pf-13-s06-credential-usage-reservations.md) | Credential usage reservations | draft | PF-13-S01, PF-17-S01 |
| 25 | [PF-41-S03](pf-41-s03-durable-security-event-foundation.md) | Durable security event and recovery foundation | draft | PF-19-S02, PF-20-S02 |
| 27 | [PF-22-S02](pf-22-s02-protected-runtime-and-upstream-seams.md) | Protected runtime integration and upstream seams | draft | PF-22-S01, PF-19-S02, PF-20-S02, PF-21-S02, PF-41-S03 |
| 28 | [PF-27-S04](pf-27-s04-isolated-credential-broker.md) | Isolated credential broker process | draft | PF-27-S01, PF-13-S04, PF-27-S03, PF-41-S03 |
| 29 | [PF-27-S02](pf-27-s02-secretless-agent-launch.md) | Secretless agent launch and bypass containment | draft | PF-27-S04 |
| 30 | [PF-28-S01](pf-28-s01-central-secret-output-gate.md) | Central secret and protected-output gate | draft | PF-27-S02 |
| 31 | [PF-28-S02](pf-28-s02-reflected-secret-response-scrubbing.md) | Reflected-secret response scrubbing | draft | PF-28-S01 |
| 32 | [PF-33-S01](pf-33-s01-url-dns-and-redirect-policy.md) | URL DNS and redirect policy | draft | PF-27-S02, PF-33-S03 |
| 33 | [PF-33-S02](pf-33-s02-connection-pinning-and-bypass.md) | Connection pinning and alternate-egress denial | draft | PF-33-S01 |
| 34 | [PF-24-S01](pf-24-s01-security-command-and-profile-view.md) | Security command and profile view | draft | PF-20-S02, PF-22-S02 |
| 35 | [PF-29-S01](pf-29-s01-protected-mode-inventory.md) | Protected-mode inventory and activation preflight | draft | PF-28-S02, PF-20-S02 |
| 36 | [PF-29-S02](pf-29-s02-human-secret-migration.md) | Human-reviewed credential migration and recovery | draft | PF-29-S01, PF-24-S01 |
| 37 | [PF-30-S01](pf-30-s01-typed-source-envelope.md) | Typed source envelope and trusted ingress | draft | PF-22-S02 |
| 38 | [PF-30-S02](pf-30-s02-persistent-taint-and-memory.md) | Persistent taint across summaries and memory | draft | PF-30-S01 |
| 39 | [PF-30-S03](pf-30-s03-post-taint-authority-checks.md) | Post-taint authority checks | draft | PF-30-S02, PF-13-S05 |
| 40 | [PF-23-S01](pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) | Moderate ingress and disclosure enforcement | draft | PF-13-S05, PF-22-S02, PF-30-S03 |
| 41 | [PF-23-S02](pf-23-s02-aggressive-deny-and-grant-enforcement.md) | Aggressive deny and grant enforcement | draft | PF-17-S01, PF-23-S01 |
| 42 | [PF-23-S03](pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) | Downgrade, restart, and inheritance enforcement | draft | PF-19-S02, PF-20-S02, PF-23-S02 |
| 43 | [PF-24-S02](pf-24-s02-security-confirm-cancel-and-downgrade.md) | Security confirm, cancel, and downgrade | draft | PF-23-S03, PF-24-S01, PF-29-S02 |
| 44 | [PF-25-S01](pf-25-s01-temporary-grant-tui.md) | Temporary grant TUI | draft | PF-17-S01, PF-23-S02, PF-24-S02 |
| 45 | [PF-25-S02](pf-25-s02-revocation-and-kill-switch-tui.md) | Revocation and kill-switch TUI | draft | PF-19-S02, PF-23-S03, PF-25-S01 |
| 46 | [PF-31-S01](pf-31-s01-pinned-retriever-isolation.md) | Pinned retriever artifact and sandbox | draft | PF-33-S02, PF-27-S02, PF-31-S04 |
| 47 | [PF-31-S02](pf-31-s02-bounded-fetch-no-fallback.md) | Bounded fetch adapter with no host fallback | draft | PF-31-S01, PF-30-S01 |
| 48 | [PF-31-S03](pf-31-s03-download-quarantine-promotion.md) | Download quarantine and human file promotion | draft | PF-31-S02, PF-24-S01 |
| 49 | [PF-34-S01](pf-34-s01-render-aware-sanitization.md) | Render-aware content sanitization | draft | PF-31-S02, PF-30-S01, PF-34-S04 |
| 50 | [PF-35-S03](pf-35-s03-calibration-and-ingress-gate.md) | Calibrated detector and ingress enforcement | draft | PF-35-S02, PF-34-S01, PF-30-S03, PF-23-S01 |
| 51 | [PF-34-S02](pf-34-s02-quarantine-state-and-store.md) | Quarantine state and encrypted retention | draft | PF-35-S03, PF-41-S03 |
| 52 | [PF-34-S03](pf-34-s03-safe-quarantine-review.md) | Safe quarantine review and recovery | draft | PF-34-S02, PF-24-S01 |
| 53 | [PF-32-S01](pf-32-s01-web-facade-and-registry.md) | Stable web facade and provider registry | draft | PF-34-S03, PF-31-S03, PF-13-S05 |
| 54 | [PF-32-S02](pf-32-s02-existing-search-and-native-bypass.md) | Existing search adapter and native bypass closure | draft | PF-32-S01 |
| 55 | [PF-32-S03](pf-32-s03-exa-search-adapter.md) | Exa brokered search adapter | draft | PF-32-S02 |
| 56 | [PF-32-S04](pf-32-s04-brave-search-adapter.md) | Brave brokered search adapter | draft | PF-32-S02 |
| 57 | [PF-32-S05](pf-32-s05-searxng-search-adapter.md) | SearXNG brokered search adapter | draft | PF-32-S02 |
| 58 | [PF-32-S06](pf-32-s06-privacy-routing-and-failover.md) | Private query routing and bounded failover | draft | PF-32-S03, PF-32-S04, PF-32-S05 |
| 59 | [PF-36-S01](pf-36-s01-hosted-detector-consent-contract.md) | Optional hosted detector consent contract | draft | PF-35-S03, PF-33-S02 |
| 60 | [PF-36-S02](pf-36-s02-hosted-bakeoff-and-local-fallback.md) | Hosted detector bakeoff and safe local fallback | draft | PF-36-S01 |
| 61 | [PF-37-S01](pf-37-s01-origin-bound-browser-login.md) | Origin-bound brokered browser login | draft | PF-31-S03, PF-28-S02, PF-30-S03, PF-34-S03 |
| 62 | [PF-37-S02](pf-37-s02-human-auth-handoff-lifecycle.md) | Human authentication handoff and session revocation | draft | PF-37-S01, PF-25-S02 |
| 63 | [PF-38-S01](pf-38-s01-typed-financial-executor.md) | Typed financial executor and deterministic limits | draft | PF-30-S03, PF-27-S02, PF-18-S01 |
| 64 | [PF-38-S02](pf-38-s02-full-effect-preview-and-mandate.md) | Full-effect financial preview and exact mandate | draft | PF-38-S01, PF-25-S01 |
| 65 | [PF-38-S03](pf-38-s03-sign-broadcast-and-receipts.md) | Separate signing broadcasting and idempotent receipts | draft | PF-38-S02, PF-41-S03 |
| 66 | [PF-39-S01](pf-39-s01-protected-financial-derived-views.md) | Protected financial derived views | draft | PF-28-S01, PF-38-S01 |
| 67 | [PF-39-S02](pf-39-s02-outbound-disclosure-controls.md) | Outbound disclosure clipboard and export controls | draft | PF-39-S01, PF-30-S03, PF-32-S06 |
| 68 | [PF-40-S01](pf-40-s01-sweep-events-and-rules.md) | Agent Sweep sanitized events and deterministic rules | draft | PF-30-S03, PF-38-S03, PF-39-S02, PF-41-S03 |
| 69 | [PF-40-S02](pf-40-s02-isolated-sweep-reviewer.md) | Isolated advisory Agent Sweep reviewer | draft | PF-40-S01, PF-36-S01 |
| 70 | [PF-40-S03](pf-40-s03-sweep-alerts-and-recovery.md) | Agent Sweep alerts revocation and recovery | draft | PF-40-S02, PF-25-S02 |
| 71 | [PF-41-S01](pf-41-s01-effective-security-inspector.md) | Effective security inspector and degradation state | draft | PF-23-S03, PF-29-S02, PF-32-S06, PF-37-S02, PF-40-S03, PF-24-S02 |
| 72 | [PF-41-S02](pf-41-s02-tamper-evident-security-audit.md) | Tamper-evident audit and safe support export | draft | PF-41-S01, PF-41-S03 |
| 73 | [PF-13-S07](pf-13-s07-integrated-credential-boundary-qualification.md) | Integrated credential boundary qualification | draft | PF-13-S05, PF-13-S06, PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02 |
| 74 | [PF-26-S04](pf-26-s04-final-automated-qualification.md) | Final integrated automated security qualification | draft | PF-26-S01, PF-13-S07, PF-21-S02, PF-23-S03, PF-25-S02, PF-36-S02, PF-41-S02 |
| 75 | [PF-26-S02](pf-26-s02-true-tui-and-live-repository-qualification.md) | True-TUI and live-repository qualification | draft | PF-26-S04 |
| 76 | [PF-26-S03](pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | Human acceptance, finished docs, and release evidence | draft | PF-26-S02 |

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
