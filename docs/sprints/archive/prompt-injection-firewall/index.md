# Prompt-injection firewall sprints

These are the archived mechanical decomposition records for the
[Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
plan. The register contains **72 single-feature sprints**.

All 72 records were cancelled unstarted on 2026-08-24. They were useful as a
design inventory, but their dependency graph and breadth did not express a safe
first implementation slice. They remain here as historical input only and do
not authorize implementation. The replacement sequence begins with PF-13-S01
under the active `/security` plan.

## PF-01 — Secretless agent boundary

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 1 | [PF-01-S01 — Protected-data taxonomy and types](pf-01-s01-protected-data-taxonomy-and-types.md) | cancelled | — |
| 2 | [PF-01-S02 — Model-visible secret denial gate](pf-01-s02-model-visible-secret-denial-gate.md) | cancelled | `PF-01-S01` |
| 3 | [PF-01-S03 — Vault credential-reference API](pf-01-s03-vault-credential-reference-api.md) | cancelled | `PF-01-S02` |
| 4 | [PF-01-S04 — Broker-only secret resolution](pf-01-s04-broker-only-secret-resolution.md) | cancelled | `PF-01-S03` |
| 5 | [PF-01-S05 — Child and container secret injection](pf-01-s05-child-and-container-secret-injection.md) | cancelled | `PF-01-S04` |
| 6 | [PF-01-S06 — Secret redaction and canary regression suite](pf-01-s06-secret-redaction-and-canary-regression-suite.md) | cancelled | `PF-01-S05` |
| 7 | [PF-01-S07 — Protected financial-data derived views](pf-01-s07-protected-financial-data-derived-views.md) | cancelled | `PF-01-S06` |

## PF-02 — Deterministic sensitive-action broker

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 8 | [PF-02-S01 — Protected-action schema](pf-02-s01-protected-action-schema.md) | cancelled | — |
| 9 | [PF-02-S02 — Canonical action encoding](pf-02-s02-canonical-action-encoding.md) | cancelled | `PF-02-S01` |
| 10 | [PF-02-S03 — Deterministic policy decision point](pf-02-s03-deterministic-policy-decision-point.md) | cancelled | `PF-02-S02` |
| 11 | [PF-02-S04 — Approval binding and trusted preview](pf-02-s04-approval-binding-and-trusted-preview.md) | cancelled | `PF-02-S03` |
| 12 | [PF-02-S05 — Signing and broadcast separation](pf-02-s05-signing-and-broadcast-separation.md) | cancelled | `PF-02-S04` |
| 13 | [PF-02-S06 — Idempotency and redacted receipts](pf-02-s06-idempotency-and-redacted-receipts.md) | cancelled | `PF-02-S05` |

## PF-03 — Isolated browser retrieval

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 14 | [PF-03-S01 — Pinned retriever image and manifest](pf-03-s01-pinned-retriever-image-and-manifest.md) | cancelled | — |
| 15 | [PF-03-S02 — Retriever sandbox profile](pf-03-s02-retriever-sandbox-profile.md) | cancelled | `PF-03-S01` |
| 16 | [PF-03-S03 — Isolated fetch adapter](pf-03-s03-isolated-fetch-adapter.md) | cancelled | `PF-03-S02` |
| 17 | [PF-03-S04 — No host-browser fallback](pf-03-s04-no-host-browser-fallback.md) | cancelled | `PF-03-S03` |
| 18 | [PF-03-S05 — Interactive web human handoff](pf-03-s05-interactive-web-human-handoff.md) | cancelled | `PF-03-S04` |

## PF-04 — Brokered multi-provider web retrieval

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 19 | [PF-04-S01 — web.run compatibility facade](pf-04-s01-web-run-compatibility-facade.md) | cancelled | — |
| 20 | [PF-04-S02 — Provider capability registry](pf-04-s02-provider-capability-registry.md) | cancelled | `PF-04-S01` |
| 21 | [PF-04-S03 — Existing Search API adapter](pf-04-s03-existing-search-api-adapter.md) | cancelled | `PF-04-S02` |
| 22 | [PF-04-S04 — Exa search adapter](pf-04-s04-exa-search-adapter.md) | cancelled | `PF-04-S03` |
| 23 | [PF-04-S05 — Brave Search adapter](pf-04-s05-brave-search-adapter.md) | cancelled | `PF-04-S04` |
| 24 | [PF-04-S06 — SearXNG metasearch adapter](pf-04-s06-searxng-metasearch-adapter.md) | cancelled | `PF-04-S05` |
| 25 | [PF-04-S07 — Deterministic provider router](pf-04-s07-deterministic-provider-router.md) | cancelled | `PF-04-S06` |
| 26 | [PF-04-S08 — Normalized results and stable ids](pf-04-s08-normalized-results-and-stable-ids.md) | cancelled | `PF-04-S07` |
| 27 | [PF-04-S09 — Query-context minimization](pf-04-s09-query-context-minimization.md) | cancelled | `PF-04-S08` |
| 28 | [PF-04-S10 — Provider health and same-role failover](pf-04-s10-provider-health-and-same-role-failover.md) | cancelled | `PF-04-S09` |
| 29 | [PF-04-S11 — Provider cost, privacy, and route audit](pf-04-s11-provider-cost-privacy-and-route-audit.md) | cancelled | `PF-04-S10` |
| 30 | [PF-04-S12 — Provider-native web_search bypass control](pf-04-s12-provider-native-web-search-bypass-control.md) | cancelled | `PF-04-S11` |

## PF-05 — Network egress and SSRF policy

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 31 | [PF-05-S01 — URL canonicalization gate](pf-05-s01-url-canonicalization-gate.md) | cancelled | — |
| 32 | [PF-05-S02 — DNS and address-class policy](pf-05-s02-dns-and-address-class-policy.md) | cancelled | `PF-05-S01` |
| 33 | [PF-05-S03 — Redirect-chain enforcement](pf-05-s03-redirect-chain-enforcement.md) | cancelled | `PF-05-S02` |
| 34 | [PF-05-S04 — DNS-rebinding and connection pinning](pf-05-s04-dns-rebinding-and-connection-pinning.md) | cancelled | `PF-05-S03` |
| 35 | [PF-05-S05 — Proxy, socket, and local-endpoint denial](pf-05-s05-proxy-socket-and-local-endpoint-denial.md) | cancelled | `PF-05-S04` |

## PF-06 — Source-provenance and authority labels

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 36 | [PF-06-S01 — Source-envelope protocol types](pf-06-s01-source-envelope-protocol-types.md) | cancelled | — |
| 37 | [PF-06-S02 — Trusted ingress authority assignment](pf-06-s02-trusted-ingress-authority-assignment.md) | cancelled | `PF-06-S01` |
| 38 | [PF-06-S03 — Model-context serialization](pf-06-s03-model-context-serialization.md) | cancelled | `PF-06-S02` |
| 39 | [PF-06-S04 — Child-agent provenance propagation](pf-06-s04-child-agent-provenance-propagation.md) | cancelled | `PF-06-S03` |
| 40 | [PF-06-S05 — Provenance persistence and audit](pf-06-s05-provenance-persistence-and-audit.md) | cancelled | `PF-06-S04` |

## PF-07 — Untrusted-text sanitization

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 41 | [PF-07-S01 — Visible main-content extraction](pf-07-s01-visible-main-content-extraction.md) | cancelled | — |
| 42 | [PF-07-S02 — Hidden and non-body content removal](pf-07-s02-hidden-and-non-body-content-removal.md) | cancelled | `PF-07-S01` |
| 43 | [PF-07-S03 — Unicode, control, link, and size normalization](pf-07-s03-unicode-control-link-and-size-normalization.md) | cancelled | `PF-07-S02` |
| 44 | [PF-07-S04 — Sealed raw artifact and digest chain](pf-07-s04-sealed-raw-artifact-and-digest-chain.md) | cancelled | `PF-07-S03` |
| 45 | [PF-07-S05 — Sanitize-and-rescan pipeline](pf-07-s05-sanitize-and-rescan-pipeline.md) | cancelled | `PF-07-S04` |

## PF-08 — Local prompt-injection classifier

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 46 | [PF-08-S01 — Classifier adapter and result schema](pf-08-s01-classifier-adapter-and-result-schema.md) | cancelled | — |
| 47 | [PF-08-S02 — Corpus manifest and license gate](pf-08-s02-corpus-manifest-and-license-gate.md) | cancelled | `PF-08-S01` |
| 48 | [PF-08-S03 — Leak-free train/validation/test splits](pf-08-s03-leak-free-train-validation-test-splits.md) | cancelled | `PF-08-S02` |
| 49 | [PF-08-S04 — Small-model training pipeline](pf-08-s04-small-model-training-pipeline.md) | cancelled | `PF-08-S03` |
| 50 | [PF-08-S05 — CPU packaging and artifact verification](pf-08-s05-cpu-packaging-and-artifact-verification.md) | cancelled | `PF-08-S04` |
| 51 | [PF-08-S06 — Calibration, blind evaluation, and runtime gate](pf-08-s06-calibration-blind-evaluation-and-runtime-gate.md) | cancelled | `PF-08-S05` |

## PF-09 — Ingress enforcement and quarantine

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 52 | [PF-09-S01 — Ingress outcome state machine](pf-09-s01-ingress-outcome-state-machine.md) | cancelled | `PF-08-S01` |
| 53 | [PF-09-S02 — Quarantine store and retention](pf-09-s02-quarantine-store-and-retention.md) | cancelled | `PF-09-S01` |
| 54 | [PF-09-S03 — Unprivileged quarantine review TUI](pf-09-s03-unprivileged-quarantine-review-tui.md) | cancelled | `PF-09-S02` |
| 55 | [PF-09-S04 — Quarantine failure, retry, and resume](pf-09-s04-quarantine-failure-retry-and-resume.md) | cancelled | `PF-09-S03` |

## PF-10 — Optional hosted classifier service

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 56 | [PF-10-S01 — Hosted-classifier adapter contract](pf-10-s01-hosted-classifier-adapter-contract.md) | cancelled | `PF-08-S01` |
| 57 | [PF-10-S02 — Hosted-service opt-in and disclosure](pf-10-s02-hosted-service-opt-in-and-disclosure.md) | cancelled | `PF-10-S01` |
| 58 | [PF-10-S03 — Local fallback and vendor-outage policy](pf-10-s03-local-fallback-and-vendor-outage-policy.md) | cancelled | `PF-10-S02` |
| 59 | [PF-10-S04 — Commercial detector bakeoff and cost gate](pf-10-s04-commercial-detector-bakeoff-and-cost-gate.md) | cancelled | `PF-10-S03` |

## PF-11 — Agent Sweep behavioral monitor

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 60 | [PF-11-S01 — Behavior-event schema and redaction](pf-11-s01-behavior-event-schema-and-redaction.md) | cancelled | — |
| 61 | [PF-11-S02 — Deterministic behavioral anomaly rules](pf-11-s02-deterministic-behavioral-anomaly-rules.md) | cancelled | `PF-11-S01` |
| 62 | [PF-11-S03 — Isolated behavior-review model](pf-11-s03-isolated-behavior-review-model.md) | cancelled | `PF-11-S01, PF-11-S02` |
| 63 | [PF-11-S04 — Pause revoke and kill escalation](pf-11-s04-pause-revoke-and-kill-escalation.md) | cancelled | `PF-11-S02` |
| 64 | [PF-11-S05 — Agent Sweep TUI and recovery flow](pf-11-s05-agent-sweep-tui-and-recovery-flow.md) | cancelled | `PF-11-S02, PF-11-S03, PF-11-S04` |

## PF-12 — Security regression and red-team harness

| Order | Sprint | Status | Depends on |
| ---: | --- | --- | --- |
| 65 | [PF-12-S01 — Synthetic hostile-source fixtures](pf-12-s01-synthetic-hostile-source-fixtures.md) | cancelled | — |
| 66 | [PF-12-S02 — Canary secrets and fake financial systems](pf-12-s02-canary-secrets-and-fake-financial-systems.md) | cancelled | `PF-12-S01` |
| 67 | [PF-12-S03 — Classifier benchmark harness](pf-12-s03-classifier-benchmark-harness.md) | cancelled | `PF-08-S05, PF-12-S01` |
| 68 | [PF-12-S04 — Forced classifier-miss harness](pf-12-s04-forced-classifier-miss-harness.md) | cancelled | `PF-12-S01, PF-12-S02` |
| 69 | [PF-12-S05 — Web isolation and egress adversarial suite](pf-12-s05-web-isolation-and-egress-adversarial-suite.md) | cancelled | `PF-03-S04, PF-04-S12, PF-05-S05, PF-12-S01` |
| 70 | [PF-12-S06 — True-TUI security qualification](pf-12-s06-true-tui-security-qualification.md) | cancelled | `PF-12-S02, PF-12-S04, PF-12-S05` |
| 71 | [PF-12-S07 — TensorCash and Isometric live-repo qualification](pf-12-s07-tensorcash-and-isometric-live-repo-qualification.md) | cancelled | `PF-12-S06` |
| 72 | [PF-12-S08 — Release security ledger and human acceptance](pf-12-s08-release-security-ledger-and-human-acceptance.md) | cancelled | `PF-12-S03, PF-12-S06, PF-12-S07` |

## Machine check

From the repository root:

```bash
python3 docs/sprints/check.py --json
```

The check fails on multi-feature sprint records, invalid lifecycle placement,
missing plan backlinks, unallocated executable work, dependency errors, missing
Done/Remaining checklists, or a current sprint that has grown into prose.
