# Security source reconciliation — 2026-08-28

Planning evidence for the active [P0 `/security` plan](active/p0-security-levels.md), not shipped-feature documentation or execution proof.

## Decision and scope

Travis Good's 2026-08-28 instruction adopts **Permissive compatibility and mandatory brokered protection above Permissive**, and asks for a complete reconciled sprint sequence with OpenClaw as the primary implementation reference. This supersedes the comparison's “all modes” no-raw-secret wording and the historical child/container raw-secret injection proposal. Permissive retains existing policies; it does not mean disabling existing sandbox/approval controls.

The active plan owns the merged work. The earlier firewall proposal remains historical design input, not a second implementation authority. The inventory contains **72 cancelled firewall sprints**, not 75. All 72 remain untouched; the 23 current security records are retained/reconciled and **40 new records** produced the 63-sprint review snapshot. The subsequent
[accepted architecture refinement](security-architecture-refinements-2026-08-28.md)
added five preparation/foundation units to the historical **68-record snapshot**.
[Upstream reconciliation](security-upstream-reconciliation-2026-08-28.md) now preserves
nine completed archives and adds five narrow follow-ups: **64 current plus nine
completed sprints**. The 72 cancelled-record dispositions and seven unrelated
Autoreview drafts remain unchanged. Completion applies only to each archive's
original scope, not to stronger review requirements.

## Source packet

“Agentic overview” is interpreted as A1 together with the A2 repository overview from the requested working session; both were located and read. Transcript speaker labels are machine-generated, not identity-confirmed.

| ID | Material | Immutable local-content SHA-256 |
| --- | --- | --- |
| A1 | [SecurityComparativeAnalysis.html](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/research/2026-08-23-product-security-session/SecurityComparativeAnalysis.html) — security harness comparison, intended Corbanu architecture, browser planes and roadmap | `656ef026203b0172bb86964c57b02a92cb43fc1ae77fd9a4ea1f72f80fec926e` |
| A2 | [repository-guide.html](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/research/2026-08-23-product-security-session/repository-guide.html) — repository/agentic workflow overview, vault and handoff map | `92a5b840012578333eb7c3474b2dfa978fc134dce897793cd9a6636b62c1a69a` |
| T | [Diarized working-session transcript](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/research/2026-08-23-product-security-session/Working%20session%20-%202026_08_23%2011_31%20MST%20-%20Recording.diarized.md) — use timestamps below, not inferred speaker identity | `9af55c73d55292b78454bb730b2b7a6a7357d0423500e745146712d66bdf54de` |

Local originals are in `research/2026-08-23-product-security-session/`. Links pin the inspected repository revision; historical guide links are not accepted as current code paths.

## OpenClaw implementation reference

Current adoption reference: [OpenClaw `13adff02ca3897768d80d2bca18f5acf08c55d91`](https://github.com/openclaw/openclaw/tree/13adff02ca3897768d80d2bca18f5acf08c55d91), the default-branch tip downloaded on 2026-08-28. The older `6ce272c2a662f81b7779507335d91de4d61c589b` remains A1's historical baseline, not the current reference. [MIT license](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/LICENSE) applies; preserve notices for future adaptation. No upstream implementation is copied by this planning change.

The [direct source review](openclaw-source-review-2026-08-28.md) records named functions, callers, defaults, deliberate differences, test execution and unreviewed areas. It corrects the earlier incomplete memory/taint account. **This is a targeted security-path review, not an entire-codebase audit or platform certification.** Its 87 upstream helper tests and 10 synthetic observation probes do not satisfy any Corbanu sprint's final-tree gate.

| ID | Pinned source | Reuse and explicit limit |
| --- | --- | --- |
| [OC-1](openclaw-source-review-2026-08-28.md#oc-1) | [sentinel](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/sentinel.ts); [provider egress](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/provider-secret-egress.ts) | Authenticated references and final handoff; same-process keys/resolver and raw opt-out are not a secretless OS boundary. |
| [OC-2](openclaw-source-review-2026-08-28.md#oc-2) | [proxy](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/egress-proxy/proxy-server.ts); [stream substitution](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/egress-proxy/stream-substitution.ts) | Exact run/host checks and optional traffic allowlist; add method/path/port, peer/DNS enforcement and reflection scrubbing. Established-channel revocation remains an unexecuted source concern, not a proven pass. |
| [OC-3](openclaw-source-review-2026-08-28.md#oc-3) | [redaction registry](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/logging/secret-redaction-registry.ts); [diagnostic masks](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/logging/redact.ts) | Encoded-value matching; Corbanu must cover short secrets, capacity exhaustion and split chunks, and remove entire secrets rather than diagnostic prefix/suffix masking. |
| [OC-4](openclaw-source-review-2026-08-28.md#oc-4) | [external-content wrappers](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/security/external-content.ts); [hook caller](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/cron/isolated-agent/run-prepare.ts) | Random boundaries, metadata/special-token sanitization and expansion-safe truncation. Heuristics and optional unsafe-hook bypasses do not implement protected-mode enforcement. |
| [OC-5](openclaw-source-review-2026-08-28.md#oc-5) | [turn state](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/embedded-agent-runner/run/turn-taint-state.ts); [maintenance inheritance](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/auto-reply/reply/agent-runner-memory.ts) | Sticky run state plus transcript-derived memory-flush taint exists. Corbanu adds cross-turn ancestry; see OC-11 for persistent memory provenance. |
| [OC-6](openclaw-source-review-2026-08-28.md#oc-6) | [audit](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/audit.ts); [apply](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/apply.ts); [runtime ownership](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/runtime-state.ts) | Consent/completeness flags, preflight and ownership-aware activation/rollback. Add encrypted crash recovery; individual atomic writes and best-effort restoration are not a durable whole-migration transaction. |
| [OC-7](openclaw-source-review-2026-08-28.md#oc-7) | [sandbox explain](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/commands/sandbox-explain.ts); [runtime classification](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/runtime-status.ts) | Resolve session-specific effective policy; distinguish classification from observed broker/engine/network/audit health. |
| [OC-8](openclaw-source-review-2026-08-28.md#oc-8) | [sandbox defaults](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/config.ts); [required sandbox](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/context.ts); [browser config](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/extensions/browser/src/browser/config.ts) | Separate execution, tool policy and privilege; creator-required sandboxes already exist. Corbanu requires verified containment and no raw/host fallback above Permissive. |
| [OC-9](openclaw-source-review-2026-08-28.md#oc-9) | [SSRF/pinned dispatch](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/infra/net/ssrf.ts); [guarded fetch](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/infra/net/fetch-guard.ts) | Per-hop URL/DNS checks, redirect credential/body policy, pinned direct transport and request-local release. Explicit proxy/private-network modes need constrained Corbanu ownership. |
| [OC-10](openclaw-source-review-2026-08-28.md#oc-10) | [web-fetch outputs](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/tools/web-fetch.ts); [web transport wrappers](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/tools/web-guarded-fetch.ts) | Screen metadata, cached/fallback/error results and overflow spills; preserve provenance and quarantine outside the normal workspace. |
| [OC-11](openclaw-source-review-2026-08-28.md#oc-11) | [memory writer](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/memory-write-provenance.ts); [persistent store](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/memory/memory-artifact-provenance.ts); [index classification](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/extensions/memory-core/src/memory/memory-path-provenance.ts) | Reuse sticky origin, canonical identity, reservation rollback and reject-new capacity; missing records or special filenames must not promote protected-mode trust. |
## Requirement ownership

A1's missing-work recommendations are explicit owners, not implied by a generic “security tests” task: secretless process/environment PF-27; global/reflected-output gates PF-28; migration PF-29; durable provenance PF-30; public-browser isolation/downloads PF-31; screened provider routing PF-32; SSRF/actual connection PF-33; sanitizer/quarantine PF-34; local/hosted classifiers PF-35/36; login boundary PF-37; financial executor PF-38; derived/disclosure controls PF-39; behavior monitoring PF-40; actual-state inspector/audit PF-41. PF-13 and PF-15–25 retain existing authority/UI ownership. PF-26 requalifies the whole final candidate.

<a id="pf-27"></a>

### PF-27 — Isolated credential broker and secretless launch

Sources: A1 secrets/process boundary; T 00:35:26, 00:45:38; OC-1/2/8.

Use a separately constrained broker plus authenticated run-scoped IPC. Replace PF-01-S05's raw child-environment injection with references-only agent launch. A proxy environment variable is not enforcement.

Current owners: [PF-27-S04](../sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md), [PF-27-S02](../sprints/current/p0-security-levels/pf-27-s02-secretless-agent-launch.md).

<a id="pf-28"></a>

### PF-28 — Central output and reflected-secret protection

Sources: A1 centralized output protection; T 01:13:46; OC-1/2/3.

Scrub reflected provider/browser responses before they enter agent memory, all persistence/output sinks, short values and split encodings. Do not copy a bounded plaintext LRU into the agent or silently evict live secrets.

Current owners: [PF-28-S01](../sprints/current/p0-security-levels/pf-28-s01-central-secret-output-gate.md), [PF-28-S02](../sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md).

<a id="pf-29"></a>

### PF-29 — Protected-mode inventory and human migration

Sources: A1 migration and operational readiness; A2 auth routes; OC-6.

Dry-run inventory → human preview → encrypted migration/recovery → re-audit. Old contaminated transcripts/memories require clean contexts or quarantine. No plaintext rollback backup or automatic credential rotation.

Current owners: [PF-29-S01](../sprints/current/p0-security-levels/pf-29-s01-protected-mode-inventory.md), [PF-29-S02](../sprints/current/p0-security-levels/pf-29-s02-human-secret-migration.md).

<a id="pf-30"></a>

### PF-30 — Durable provenance and post-taint authority

Sources: A1 durable provenance; T 00:42:33, 00:43:03, 00:48:08; OC-4/5/11.

Immutable ingress-assigned envelopes and sticky lineage across summaries, compaction, memory, imports, child messages and resume. Human exact approval authorizes one action; it does not erase taint.

Current owners: [PF-30-S01](../sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md), [PF-30-S02](../sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md), [PF-30-S03](../sprints/current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md).

<a id="pf-31"></a>

### PF-31 — Isolated retrieval and download promotion

Sources: A1 two-plane browser isolation; T 00:35:26; OC-8/9/10.

Pinned secretless public worker, constrained network/resources, no host fallback. Add explicit sealed download quarantine and exact human promotion; public workers never receive login cookies.

Current owners: [PF-31-S01](../sprints/current/p0-security-levels/pf-31-s01-pinned-retriever-isolation.md), [PF-31-S02](../sprints/current/p0-security-levels/pf-31-s02-bounded-fetch-no-fallback.md), [PF-31-S03](../sprints/current/p0-security-levels/pf-31-s03-download-quarantine-promotion.md).

<a id="pf-32"></a>

### PF-32 — Screened web facade and search providers

Sources: T 00:35:26, 01:06:42, 01:07:54; A2 web tooling; OC-2/4/9/10.

Preserve web.run shape while screening existing SearchClient, Exa, Brave and SearXNG. Remove recent conversation history from protected search, disable unscreened native search, bound same-role failover and spending.

Current owners: [PF-32-S01](../sprints/current/p0-security-levels/pf-32-s01-web-facade-and-registry.md), [PF-32-S02](../sprints/current/p0-security-levels/pf-32-s02-existing-search-and-native-bypass.md), [PF-32-S03](../sprints/current/p0-security-levels/pf-32-s03-exa-search-adapter.md), [PF-32-S04](../sprints/current/p0-security-levels/pf-32-s04-brave-search-adapter.md), [PF-32-S05](../sprints/current/p0-security-levels/pf-32-s05-searxng-search-adapter.md), [PF-32-S06](../sprints/current/p0-security-levels/pf-32-s06-privacy-routing-and-failover.md).

<a id="pf-33"></a>

### PF-33 — Destination validation and connection enforcement

Sources: A1 SSRF/rebinding and broker destinations; OC-2/8/9.

Validate URL/DNS/redirect and actual connected peer, then enforce no direct sockets/proxy escape. Explicit self-hosted search adapter exceptions do not widen the public-fetch lane.

Current owners: [PF-33-S01](../sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md), [PF-33-S02](../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md).

<a id="pf-34"></a>

### PF-34 — Sanitization quarantine and safe review

Sources: A1 sanitization/quarantine; T 00:47:20; OC-4/10/11.

Render-aware visible content, escaped model/terminal markers, encrypted raw artifact retention, bounded rescan, safe human review and failure/restart recovery. Sanitized does not mean trusted.

Current owners: [PF-34-S01](../sprints/current/p0-security-levels/pf-34-s01-render-aware-sanitization.md), [PF-34-S02](../sprints/current/p0-security-levels/pf-34-s02-quarantine-state-and-store.md), [PF-34-S03](../sprints/current/p0-security-levels/pf-34-s03-safe-quarantine-review.md).

<a id="pf-35"></a>

### PF-35 — Local classifier and blind qualification

Sources: T 00:35:26, 00:46:55; A1 untrusted-input pipeline.

Licensed, leakage-free corpus; reproducible CPU artifact; calibrated blind evaluation and fail-closed pre-model screening. Detector misses are contained by independent policy. OpenClaw wrapper heuristics are not a substitute classifier.

Current owners: [PF-35-S01](../sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md), [PF-35-S02](../sprints/current/p0-security-levels/pf-35-s02-local-cpu-detector-artifact.md), [PF-35-S03](../sprints/current/p0-security-levels/pf-35-s03-calibration-and-ingress-gate.md).

<a id="pf-36"></a>

### PF-36 — Optional hosted detector and safe fallback

Sources: T 00:35:26 paid-service hypothesis; original PF-10.

Keep a provider-neutral, disabled-by-default optional lane; human data/cost consent and measured qualification precede activation. Qualified local fallback or pause, never unscreened output. No claim that a commercial vendor is already selected.

Current owners: [PF-36-S01](../sprints/current/p0-security-levels/pf-36-s01-hosted-detector-consent-contract.md), [PF-36-S02](../sprints/current/p0-security-levels/pf-36-s02-hosted-bakeoff-and-local-fallback.md).

<a id="pf-37"></a>

### PF-37 — Origin-bound browser login and human handoff

Sources: A1 first provider plus browser-login boundary; A2 credential handoffs; OC-1/2/8.

New exact-origin login adapter and separate credentialed session boundary, qualified against a fixture and one human-selected permitted origin with a non-production account. No model-controlled selectors, arbitrary eval, cookies/profile export or automated MFA/CAPTCHA bypass; human challenges stay outside recorded model input.

Current owners: [PF-37-S01](../sprints/current/p0-security-levels/pf-37-s01-origin-bound-browser-login.md), [PF-37-S02](../sprints/current/p0-security-levels/pf-37-s02-human-auth-handoff-lifecycle.md).

<a id="pf-38"></a>

### PF-38 — Typed financial execution and exact effects

Sources: T 00:35:26, 00:48:27; product Non-negotiable controls; OC-1/2 boundary pattern only.

Reuse PF-16–19 authority primitives; implement narrow existing-wallet/fake-venue adapters, complete-effect simulation/preview, separate sign/broadcast and idempotent receipts. OpenClaw is not evidence of trading/custody safety.

Current owners: [PF-38-S01](../sprints/current/p0-security-levels/pf-38-s01-typed-financial-executor.md), [PF-38-S02](../sprints/current/p0-security-levels/pf-38-s02-full-effect-preview-and-mandate.md), [PF-38-S03](../sprints/current/p0-security-levels/pf-38-s03-sign-broadcast-and-receipts.md).

<a id="pf-39"></a>

### PF-39 — Derived financial views and disclosure control

Sources: A1 protected data; T 01:06:04; product Non-negotiable controls.

Add purpose-limited derived financial views and reconstruction limits; cover provider payloads, tool arguments, search, clipboard, exports, social/email and child handoffs. Raw operational secrets remain non-disclosable.

Current owners: [PF-39-S01](../sprints/current/p0-security-levels/pf-39-s01-protected-financial-derived-views.md), [PF-39-S02](../sprints/current/p0-security-levels/pf-39-s02-outbound-disclosure-controls.md).

<a id="pf-40"></a>

### PF-40 — Agent Sweep and safe recovery

Sources: T 00:46:06; A1 deterministic containment.

Sanitized behavior events, deterministic anomaly rules, isolated optional advisory model, durable pause/revoke/kill and human recovery. Neither reviewer nor agent can lower a level or authorize actions.

Current owners: [PF-40-S01](../sprints/current/p0-security-levels/pf-40-s01-sweep-events-and-rules.md), [PF-40-S02](../sprints/current/p0-security-levels/pf-40-s02-isolated-sweep-reviewer.md), [PF-40-S03](../sprints/current/p0-security-levels/pf-40-s03-sweep-alerts-and-recovery.md).

<a id="pf-41"></a>

### PF-41 — Effective security inspector and audit

Sources: A1 inspector/audit; T 01:04:09, 01:13:46; OC-7/8.

Inspect effective runtime facts, degradation, broker/classifier/backend health, active grants/expiry, taint, recent denial and audit integrity. Support exports pass the same disclosure gate.

Current owners: [PF-41-S01](../sprints/current/p0-security-levels/pf-41-s01-effective-security-inspector.md), [PF-41-S02](../sprints/current/p0-security-levels/pf-41-s02-tamper-evident-security-audit.md).

## Complete archived-sprint disposition

Each cancelled record maps to explicit owning work, including foundations now archived as completed. “Reconciled” means design reuse, not completed code or restored authorization. Original files remain cancelled and excluded from current navigation.

| Archived record | Disposition | Current owners |
| --- | --- | --- |
| [PF-01-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-01-s01-protected-data-taxonomy-and-types.md) — Protected-data taxonomy and types | Reconciled into bounded current tasks | [PF-28-S01](../sprints/current/p0-security-levels/pf-28-s01-central-secret-output-gate.md), [PF-39-S01](../sprints/current/p0-security-levels/pf-39-s01-protected-financial-derived-views.md) |
| [PF-01-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-01-s02-model-visible-secret-denial-gate.md) — Model-visible secret denial gate | Reconciled into bounded current tasks | [PF-28-S01](../sprints/current/p0-security-levels/pf-28-s01-central-secret-output-gate.md), [PF-30-S03](../sprints/current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md) |
| [PF-01-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-01-s03-vault-credential-reference-api.md) — Vault credential-reference API | Reconciled into bounded current tasks | [PF-13-S01](../sprints/archive/p0-security-levels/pf-13-s01-vault-backed-exact-host-credential-substitution.md) |
| [PF-01-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-01-s04-broker-only-secret-resolution.md) — Broker-only secret resolution | Reconciled into bounded current tasks | [PF-13-S02](../sprints/archive/p0-security-levels/pf-13-s02-scoped-vault-resolver.md), [PF-27-S04](../sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md) |
| [PF-01-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-01-s05-child-and-container-secret-injection.md) — Child and container secret injection | Replaced: no raw secrets in agent children | [PF-27-S02](../sprints/current/p0-security-levels/pf-27-s02-secretless-agent-launch.md) |
| [PF-01-S06](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-01-s06-secret-redaction-and-canary-regression-suite.md) — Secret redaction and canary regression suite | Reconciled into bounded current tasks | [PF-28-S01](../sprints/current/p0-security-levels/pf-28-s01-central-secret-output-gate.md), [PF-28-S02](../sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md), [PF-13-S05](../sprints/archive/p0-security-levels/pf-13-s05-credential-boundary-adversarial-qualification.md) |
| [PF-01-S07](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-01-s07-protected-financial-data-derived-views.md) — Protected financial-data derived views | Reconciled into bounded current tasks | [PF-39-S01](../sprints/current/p0-security-levels/pf-39-s01-protected-financial-derived-views.md) |
| [PF-02-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-02-s01-protected-action-schema.md) — Protected-action schema | Reconciled with PF-16–19; execution added | [PF-16-S01](../sprints/archive/p0-security-levels/pf-16-s01-authorization-decision-contract.md), [PF-38-S01](../sprints/current/p0-security-levels/pf-38-s01-typed-financial-executor.md) |
| [PF-02-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-02-s02-canonical-action-encoding.md) — Canonical action encoding | Reconciled with PF-16–19; execution added | [PF-18-S01](../sprints/archive/p0-security-levels/pf-18-s01-human-mandates-and-receipts.md), [PF-38-S01](../sprints/current/p0-security-levels/pf-38-s01-typed-financial-executor.md) |
| [PF-02-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-02-s03-deterministic-policy-decision-point.md) — Deterministic policy decision point | Reconciled with PF-16–19; execution added | [PF-16-S01](../sprints/archive/p0-security-levels/pf-16-s01-authorization-decision-contract.md), [PF-38-S01](../sprints/current/p0-security-levels/pf-38-s01-typed-financial-executor.md) |
| [PF-02-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-02-s04-approval-binding-and-trusted-preview.md) — Approval binding and trusted preview | Reconciled with PF-16–19; execution added | [PF-18-S01](../sprints/archive/p0-security-levels/pf-18-s01-human-mandates-and-receipts.md), [PF-38-S02](../sprints/current/p0-security-levels/pf-38-s02-full-effect-preview-and-mandate.md) |
| [PF-02-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-02-s05-signing-and-broadcast-separation.md) — Signing and broadcast separation | Reconciled with PF-16–19; execution added | [PF-38-S03](../sprints/current/p0-security-levels/pf-38-s03-sign-broadcast-and-receipts.md) |
| [PF-02-S06](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-02-s06-idempotency-and-redacted-receipts.md) — Idempotency and redacted receipts | Reconciled with PF-16–19; execution added | [PF-18-S01](../sprints/archive/p0-security-levels/pf-18-s01-human-mandates-and-receipts.md), [PF-38-S03](../sprints/current/p0-security-levels/pf-38-s03-sign-broadcast-and-receipts.md) |
| [PF-03-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-03-s01-pinned-retriever-image-and-manifest.md) — Pinned retriever image and manifest | Reconciled into bounded current tasks | [PF-31-S01](../sprints/current/p0-security-levels/pf-31-s01-pinned-retriever-isolation.md) |
| [PF-03-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-03-s02-retriever-sandbox-profile.md) — Retriever sandbox profile | Reconciled into bounded current tasks | [PF-31-S01](../sprints/current/p0-security-levels/pf-31-s01-pinned-retriever-isolation.md), [PF-33-S02](../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md) |
| [PF-03-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-03-s03-isolated-fetch-adapter.md) — Isolated fetch adapter | Reconciled into bounded current tasks | [PF-31-S02](../sprints/current/p0-security-levels/pf-31-s02-bounded-fetch-no-fallback.md) |
| [PF-03-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-03-s04-no-host-browser-fallback.md) — No host-browser fallback | Reconciled into bounded current tasks | [PF-31-S02](../sprints/current/p0-security-levels/pf-31-s02-bounded-fetch-no-fallback.md) |
| [PF-03-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-03-s05-interactive-web-human-handoff.md) — Interactive web human handoff | Reconciled into bounded current tasks | [PF-31-S02](../sprints/current/p0-security-levels/pf-31-s02-bounded-fetch-no-fallback.md), [PF-37-S02](../sprints/current/p0-security-levels/pf-37-s02-human-auth-handoff-lifecycle.md) |
| [PF-04-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s01-web-run-compatibility-facade.md) — web.run compatibility facade | Reconciled into bounded current tasks | [PF-32-S01](../sprints/current/p0-security-levels/pf-32-s01-web-facade-and-registry.md) |
| [PF-04-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s02-provider-capability-registry.md) — Provider capability registry | Reconciled into bounded current tasks | [PF-32-S01](../sprints/current/p0-security-levels/pf-32-s01-web-facade-and-registry.md) |
| [PF-04-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s03-existing-search-api-adapter.md) — Existing Search API adapter | Reconciled into bounded current tasks | [PF-32-S02](../sprints/current/p0-security-levels/pf-32-s02-existing-search-and-native-bypass.md) |
| [PF-04-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s04-exa-search-adapter.md) — Exa search adapter | Reconciled into bounded current tasks | [PF-32-S03](../sprints/current/p0-security-levels/pf-32-s03-exa-search-adapter.md) |
| [PF-04-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s05-brave-search-adapter.md) — Brave Search adapter | Reconciled into bounded current tasks | [PF-32-S04](../sprints/current/p0-security-levels/pf-32-s04-brave-search-adapter.md) |
| [PF-04-S06](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s06-searxng-metasearch-adapter.md) — SearXNG metasearch adapter | Reconciled into bounded current tasks | [PF-32-S05](../sprints/current/p0-security-levels/pf-32-s05-searxng-search-adapter.md) |
| [PF-04-S07](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s07-deterministic-provider-router.md) — Deterministic provider router | Reconciled into bounded current tasks | [PF-32-S06](../sprints/current/p0-security-levels/pf-32-s06-privacy-routing-and-failover.md) |
| [PF-04-S08](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s08-normalized-results-and-stable-ids.md) — Normalized results and stable ids | Reconciled into bounded current tasks | [PF-32-S01](../sprints/current/p0-security-levels/pf-32-s01-web-facade-and-registry.md) |
| [PF-04-S09](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s09-query-context-minimization.md) — Query-context minimization | Reconciled into bounded current tasks | [PF-32-S02](../sprints/current/p0-security-levels/pf-32-s02-existing-search-and-native-bypass.md), [PF-32-S06](../sprints/current/p0-security-levels/pf-32-s06-privacy-routing-and-failover.md) |
| [PF-04-S10](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s10-provider-health-and-same-role-failover.md) — Provider health and same-role failover | Reconciled into bounded current tasks | [PF-32-S06](../sprints/current/p0-security-levels/pf-32-s06-privacy-routing-and-failover.md) |
| [PF-04-S11](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s11-provider-cost-privacy-and-route-audit.md) — Provider cost, privacy, and route audit | Reconciled into bounded current tasks | [PF-32-S06](../sprints/current/p0-security-levels/pf-32-s06-privacy-routing-and-failover.md) |
| [PF-04-S12](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-04-s12-provider-native-web-search-bypass-control.md) — Provider-native web_search bypass control | Reconciled into bounded current tasks | [PF-32-S02](../sprints/current/p0-security-levels/pf-32-s02-existing-search-and-native-bypass.md) |
| [PF-05-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-05-s01-url-canonicalization-gate.md) — URL canonicalization gate | Reconciled into bounded current tasks | [PF-33-S01](../sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md) |
| [PF-05-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-05-s02-dns-and-address-class-policy.md) — DNS and address-class policy | Reconciled into bounded current tasks | [PF-33-S01](../sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md) |
| [PF-05-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-05-s03-redirect-chain-enforcement.md) — Redirect-chain enforcement | Reconciled into bounded current tasks | [PF-33-S01](../sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md) |
| [PF-05-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-05-s04-dns-rebinding-and-connection-pinning.md) — DNS-rebinding and connection pinning | Reconciled into bounded current tasks | [PF-33-S02](../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md) |
| [PF-05-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-05-s05-proxy-socket-and-local-endpoint-denial.md) — Proxy, socket, and local-endpoint denial | Reconciled into bounded current tasks | [PF-33-S02](../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md) |
| [PF-06-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-06-s01-source-envelope-protocol-types.md) — Source-envelope protocol types | Reconciled into bounded current tasks | [PF-30-S01](../sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md) |
| [PF-06-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-06-s02-trusted-ingress-authority-assignment.md) — Trusted ingress authority assignment | Reconciled into bounded current tasks | [PF-30-S01](../sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md) |
| [PF-06-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-06-s03-model-context-serialization.md) — Model-context serialization | Reconciled into bounded current tasks | [PF-30-S01](../sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md), [PF-30-S02](../sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md) |
| [PF-06-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-06-s04-child-agent-provenance-propagation.md) — Child-agent provenance propagation | Reconciled into bounded current tasks | [PF-30-S02](../sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md) |
| [PF-06-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-06-s05-provenance-persistence-and-audit.md) — Provenance persistence and audit | Reconciled into bounded current tasks | [PF-30-S02](../sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md), [PF-41-S02](../sprints/current/p0-security-levels/pf-41-s02-tamper-evident-security-audit.md) |
| [PF-07-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-07-s01-visible-main-content-extraction.md) — Visible main-content extraction | Reconciled into bounded current tasks | [PF-34-S01](../sprints/current/p0-security-levels/pf-34-s01-render-aware-sanitization.md) |
| [PF-07-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-07-s02-hidden-and-non-body-content-removal.md) — Hidden and non-body content removal | Reconciled into bounded current tasks | [PF-34-S01](../sprints/current/p0-security-levels/pf-34-s01-render-aware-sanitization.md) |
| [PF-07-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-07-s03-unicode-control-link-and-size-normalization.md) — Unicode, control, link, and size normalization | Reconciled into bounded current tasks | [PF-34-S01](../sprints/current/p0-security-levels/pf-34-s01-render-aware-sanitization.md) |
| [PF-07-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-07-s04-sealed-raw-artifact-and-digest-chain.md) — Sealed raw artifact and digest chain | Reconciled into bounded current tasks | [PF-34-S01](../sprints/current/p0-security-levels/pf-34-s01-render-aware-sanitization.md), [PF-34-S02](../sprints/current/p0-security-levels/pf-34-s02-quarantine-state-and-store.md) |
| [PF-07-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-07-s05-sanitize-and-rescan-pipeline.md) — Sanitize-and-rescan pipeline | Reconciled into bounded current tasks | [PF-35-S03](../sprints/current/p0-security-levels/pf-35-s03-calibration-and-ingress-gate.md), [PF-34-S02](../sprints/current/p0-security-levels/pf-34-s02-quarantine-state-and-store.md) |
| [PF-08-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-08-s01-classifier-adapter-and-result-schema.md) — Classifier adapter and result schema | Reconciled into bounded current tasks | [PF-35-S01](../sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md) |
| [PF-08-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-08-s02-corpus-manifest-and-license-gate.md) — Corpus manifest and license gate | Reconciled into bounded current tasks | [PF-35-S01](../sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md) |
| [PF-08-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-08-s03-leak-free-train-validation-test-splits.md) — Leak-free train/validation/test splits | Reconciled into bounded current tasks | [PF-35-S01](../sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md) |
| [PF-08-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-08-s04-small-model-training-pipeline.md) — Small-model training pipeline | Reconciled into bounded current tasks | [PF-35-S02](../sprints/current/p0-security-levels/pf-35-s02-local-cpu-detector-artifact.md) |
| [PF-08-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-08-s05-cpu-packaging-and-artifact-verification.md) — CPU packaging and artifact verification | Reconciled into bounded current tasks | [PF-35-S02](../sprints/current/p0-security-levels/pf-35-s02-local-cpu-detector-artifact.md) |
| [PF-08-S06](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-08-s06-calibration-blind-evaluation-and-runtime-gate.md) — Calibration, blind evaluation, and runtime gate | Reconciled into bounded current tasks | [PF-35-S03](../sprints/current/p0-security-levels/pf-35-s03-calibration-and-ingress-gate.md) |
| [PF-09-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-09-s01-ingress-outcome-state-machine.md) — Ingress outcome state machine | Reconciled into bounded current tasks | [PF-34-S02](../sprints/current/p0-security-levels/pf-34-s02-quarantine-state-and-store.md) |
| [PF-09-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-09-s02-quarantine-store-and-retention.md) — Quarantine store and retention | Reconciled into bounded current tasks | [PF-34-S02](../sprints/current/p0-security-levels/pf-34-s02-quarantine-state-and-store.md) |
| [PF-09-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-09-s03-unprivileged-quarantine-review-tui.md) — Unprivileged quarantine review TUI | Reconciled into bounded current tasks | [PF-34-S03](../sprints/current/p0-security-levels/pf-34-s03-safe-quarantine-review.md) |
| [PF-09-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-09-s04-quarantine-failure-retry-and-resume.md) — Quarantine failure, retry, and resume | Reconciled into bounded current tasks | [PF-34-S02](../sprints/current/p0-security-levels/pf-34-s02-quarantine-state-and-store.md), [PF-34-S03](../sprints/current/p0-security-levels/pf-34-s03-safe-quarantine-review.md) |
| [PF-10-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-10-s01-hosted-classifier-adapter-contract.md) — Hosted-classifier adapter contract | Retained as conditional, disabled-by-default lane | [PF-36-S01](../sprints/current/p0-security-levels/pf-36-s01-hosted-detector-consent-contract.md) |
| [PF-10-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-10-s02-hosted-service-opt-in-and-disclosure.md) — Hosted-service opt-in and disclosure | Retained as conditional, disabled-by-default lane | [PF-36-S01](../sprints/current/p0-security-levels/pf-36-s01-hosted-detector-consent-contract.md) |
| [PF-10-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-10-s03-local-fallback-and-vendor-outage-policy.md) — Local fallback and vendor-outage policy | Retained as conditional, disabled-by-default lane | [PF-36-S02](../sprints/current/p0-security-levels/pf-36-s02-hosted-bakeoff-and-local-fallback.md) |
| [PF-10-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-10-s04-commercial-detector-bakeoff-and-cost-gate.md) — Commercial detector bakeoff and cost gate | Retained as conditional, disabled-by-default lane | [PF-36-S02](../sprints/current/p0-security-levels/pf-36-s02-hosted-bakeoff-and-local-fallback.md) |
| [PF-11-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-11-s01-behavior-event-schema-and-redaction.md) — Behavior-event schema and redaction | Reconciled into bounded current tasks | [PF-40-S01](../sprints/current/p0-security-levels/pf-40-s01-sweep-events-and-rules.md) |
| [PF-11-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-11-s02-deterministic-behavioral-anomaly-rules.md) — Deterministic behavioral anomaly rules | Reconciled into bounded current tasks | [PF-40-S01](../sprints/current/p0-security-levels/pf-40-s01-sweep-events-and-rules.md) |
| [PF-11-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-11-s03-isolated-behavior-review-model.md) — Isolated behavior-review model | Reconciled into bounded current tasks | [PF-40-S02](../sprints/current/p0-security-levels/pf-40-s02-isolated-sweep-reviewer.md) |
| [PF-11-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-11-s04-pause-revoke-and-kill-escalation.md) — Pause revoke and kill escalation | Reconciled into bounded current tasks | [PF-40-S03](../sprints/current/p0-security-levels/pf-40-s03-sweep-alerts-and-recovery.md), [PF-19-S01](../sprints/archive/p0-security-levels/pf-19-s01-revocation-contract.md) |
| [PF-11-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-11-s05-agent-sweep-tui-and-recovery-flow.md) — Agent Sweep TUI and recovery flow | Reconciled into bounded current tasks | [PF-40-S03](../sprints/current/p0-security-levels/pf-40-s03-sweep-alerts-and-recovery.md) |
| [PF-12-S01](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s01-synthetic-hostile-source-fixtures.md) — Synthetic hostile-source fixtures | Folded into whole-plan qualification | [PF-26-S01](../sprints/archive/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md) |
| [PF-12-S02](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s02-canary-secrets-and-fake-financial-systems.md) — Canary secrets and fake financial systems | Folded into whole-plan qualification | [PF-26-S01](../sprints/archive/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md) |
| [PF-12-S03](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s03-classifier-benchmark-harness.md) — Classifier benchmark harness | Folded into whole-plan qualification | [PF-35-S01](../sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md), [PF-35-S03](../sprints/current/p0-security-levels/pf-35-s03-calibration-and-ingress-gate.md), [PF-26-S01](../sprints/archive/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md) |
| [PF-12-S04](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s04-forced-classifier-miss-harness.md) — Forced classifier-miss harness | Folded into whole-plan qualification | [PF-30-S03](../sprints/current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md), [PF-26-S01](../sprints/archive/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md) |
| [PF-12-S05](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s05-web-isolation-and-egress-adversarial-suite.md) — Web isolation and egress adversarial suite | Folded into whole-plan qualification | [PF-33-S02](../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md), [PF-26-S01](../sprints/archive/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md) |
| [PF-12-S06](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s06-true-tui-security-qualification.md) — True-TUI security qualification | Folded into whole-plan qualification | [PF-26-S02](../sprints/current/p0-security-levels/pf-26-s02-true-tui-and-live-repository-qualification.md) |
| [PF-12-S07](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s07-tensorcash-and-isometric-live-repo-qualification.md) — TensorCash and Isometric live-repo qualification | Folded into whole-plan qualification | [PF-26-S02](../sprints/current/p0-security-levels/pf-26-s02-true-tui-and-live-repository-qualification.md) |
| [PF-12-S08](https://github.com/CorbanuCore/CorbanuTerminal/blob/f173a0bc97c7495d134a67079aadfbe3657d11a7/docs/sprints/archive/prompt-injection-firewall/pf-12-s08-release-security-ledger-and-human-acceptance.md) — Release security ledger and human acceptance | Folded into whole-plan qualification | [PF-26-S03](../sprints/current/p0-security-levels/pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) |

## Additional work absent or insufficiently explicit in the archive

PF-27 process/OS enforcement replaces generic secret injection; PF-28-S02 addresses reflected credentials; PF-29 adds migration and contaminated resume handling; PF-30-S02/03 make memory taint and post-taint authority durable; PF-31-S03 adds sealed download promotion; PF-37 adds exact-origin credentialed login; PF-38 makes the existing wallet path and uncertain transaction recovery explicit; PF-39-S02 covers non-chat disclosure sinks; PF-41 adds actual-state inspection and tamper-evident safe export.

Credential and policy follow-ups extend accepted foundations without duplicating them. No cancelled firewall record is used as a completed dependency; the nine accepted security-foundation archives are valid dependencies for their original scope. Numeric order is a valid topological implementation order, not a time estimate. The October 8 deadline is retained; expanded effort and hardware/vendor/platform prerequisites must be assessed before readiness, not hidden by marking drafts complete.
