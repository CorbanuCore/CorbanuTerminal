---
title: "Prompt-injection firewall and brokered authority"
status: draft
change_class: product-initiative
priority: P0
owner: "Jim Ricketts"
activation_authority: "Product authority defined in the product specification"
activation_basis: "User-directed proposal capture; implementation is not authorized"
target_release: "TBD"
deadline: "TBD"
created: 2026-08-23
updated: 2026-08-28
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "P0 /security levels"
  requirement_excerpt: "Treat external content and tool output as untrusted; prevent secrets and protected financial data from entering model-visible context; require trusted credential resolution."
implementation_worktrees: []
---

# Prompt-injection firewall and brokered authority

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

This is **historical proposed design input**, retained as a draft rather than
reactivated. On 2026-08-28, Travis Good requested selective reuse and reconciliation
into the [active P0 plan](../active/p0-security-levels.md). That plan now owns all
accepted implementation scope and its 64 current plus nine completed sprints. This document and its
72 cancelled sprints do not authorize parallel work or describe finished behavior.

[The complete source and archive crosswalk](../security-source-reconciliation.md)
records what was reused, replaced or made conditional. Current product decisions
and the active plan supersede conflicting historical prose below; in particular,
Permissive retains compatibility and protected agent children do not receive raw
secrets through environment injection. OpenClaw is the primary implementation
reference; older Ambient comparisons remain contextual evidence only.

## Activation record

| Field | Value |
| --- | --- |
| Status | **Draft / proposed** |
| Active-plan slot | None |
| Product authority | Defined once in the product specification |
| Delivery owner | Jim Ricketts |
| Authoritative decision | Original proposal captured on August 23; accepted contracts reconciled into the active P0 plan on August 28 |
| Activation gate | Not a separate activation candidate; use the active P0 plan and new sprint map |
| Target release | TBD |
| Deadline | TBD |

## User pain

A trading agent must read hostile material from webpages, search results, files,
transcripts, social feeds, tools, and other agents. Today those inputs can contain
instructions that sound authoritative to a model. If the same model can also see
credentials or invoke a broad financial tool, one poisoned input can become a
secret leak, portfolio disclosure, or unauthorized trade.

Prompt text cannot solve that failure mode. Corbanu needs an architecture in
which untrusted text is screened before model ingestion, provenance survives every
hop, secrets never enter model-visible context, and a deterministic executor—not
the agent—decides whether a sensitive operation may run.

## Product intent and ideal flow

The user selects a security level in `/security` and then works normally.
Corbanu labels every external source, routes public-web retrieval through an
isolated workload, extracts visible task-relevant content, and screens it before
the main agent sees it. Safe content arrives with provenance. Suspicious content
is rejected or quarantined with a plain-English reason and a recovery action.

When the agent needs a credential or financial action, it names the credential
and proposes a typed operation. A separate trusted process resolves the secret,
checks deterministic policy, shows the exact effect when approval is required,
executes the bounded operation, and returns a redacted receipt. The model never
receives the secret. A separate behavior monitor can pause work, revoke pending
authority, or activate the kill switch when activity departs from the user's
mandate; it cannot silently change `/security` or perform financial actions.

The user can always tell:

- which source produced the data;
- why content was allowed, quarantined, or rejected;
- which process—not which model—held authority;
- what exact action was proposed, approved, and executed; and
- how to revoke access, recover, and resume.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | `P0 /security levels` |
| Requirement excerpt | “Treat external content and tool output as untrusted; prevent secrets and protected financial data from entering model-visible context; require trusted credential resolution.” |
| Product outcome advanced | Moderate and Aggressive become enforceable architectures rather than prompt instructions |
| North-star criterion advanced | Agents can research and propose without acquiring authority over secrets, policy, or money |

## Relationship to the active P0 plan

The [active P0 plan](../active/p0-security-levels.md) is the sole implementation
authority. The 2026-08-28 decision selected merging the accepted contracts into
that plan, not activating a second overlapping initiative. All 72 old sprint
records remain cancelled; their individual dispositions and current owners are
listed in the [source reconciliation](../security-source-reconciliation.md).

## Feature register

Every concept from the source discussion is classified below. `Ambient status`
means the verified state of the public Ambient Desktop snapshot at commit
[`3dd37c3d`](https://github.com/ambient-xyz/AmbientDesktop/tree/3dd37c3d186bf5734448f67451259967f74ede67),
not a claim about Corbanu.

| ID | Feature | User problem solved | Ambient status | Corbanu proposal |
| --- | --- | --- | --- | --- |
| PF-01 | Secretless agent context | A poisoned prompt cannot extract a value the model never receives | **Implemented for managed secret paths and brokered flows**; unmanaged workspace secrets remain a separate policy concern | Make all Corbanu vault, wallet, broker, exchange, and provider credentials references-only to the model |
| PF-02 | Deterministic sensitive-tool executor | The agent cannot turn arbitrary language into a privileged or financial action | **Partial reference** through typed privileged actions; current Ambient templates can still express broad host commands | Use narrow per-operation schemas and deterministic policy; no generic shell escape for protected actions |
| PF-03 | Isolated public-web retrieval | A browser exploit or hostile page does not run in the user's ordinary browser context | **Implemented for eligible Scrapling reads**, but direct Ambient browser tools and browser fallback still exist | Moderate and Aggressive route eligible retrieval only through an isolated, pinned workload |
| PF-04 | Search-provider broker and stable web-tool facade | Research does not depend on direct Google browsing, one provider, or provider-specific agent behavior | **Partial**: Exa, Scrapling, Ambient Browser defaults; Brave and SearXNG catalog paths | Preserve Corbanu's `web.run` entry point while routing its logical search and fetch operations through approved providers whose output is always untrusted and screened |
| PF-05 | Network and IP gate | SSRF, redirect, metadata, private-network, and DNS-rebinding paths cannot escape the web sandbox | **Partial**: URL and literal-IP checks plus MCP network policy; the reviewed Scrapling profile still allows broad outbound 80/443 | Resolve and enforce public destinations at every redirect and connection boundary |
| PF-06 | Typed source and authority labels | External text no longer sounds like a user or system authority | **Not found as a general ingestion contract** | Attach a non-forgeable source envelope to every model-visible segment |
| PF-07 | Render-aware content cleaner | Hidden page text and non-content markup do not enter model context by default | **Partial**: Scrapling requests Markdown with `main_content_only`; this is extraction, not injection defense | Strip non-visible and non-body material, preserve provenance, then rescan; visible text remains untrusted |
| PF-08 | Local prompt-injection classifier | Hostile instructions can be stopped before reaching the main agent | **No general runtime classifier found** | Ship a small CPU-capable classifier after benchmark qualification |
| PF-09 | Quarantine and pre-model rejection | Detection has an enforceable product outcome instead of a warning in the prompt | **No general ingestion quarantine found** | Implement typed allow, sanitize-and-rescan, quarantine, and reject outcomes |
| PF-10 | Optional hosted classifier service | Users can buy stronger or faster continuously updated detection | **No Ambient service integration found** | Offer an opt-in premium lane only if privacy, latency, and benchmark gates beat the local baseline |
| PF-11 | Agent Sweep / rogue-behavior monitor | A compromised agent can be stopped when its behavior drifts even if content detection misses | **No general monitor found** | Observe redacted events, apply deterministic anomaly rules plus optional model review, and escalate to a human or kill switch |
| PF-12 | Adversarial security harness | Regressions are caught against realistic poisoned data and fake financial actions | **Partial security fixtures and threat-model tests**; not a complete classifier or financial-agent harness | Build synthetic sources, canary secrets, fake venues, forced detector misses, and true-TUI attack flows |

## Sprint execution map

The [archived sprint register](../../sprints/archive/prompt-injection-firewall/index.md)
retains the original **72-record** decomposition as historical design input.
Those records were cancelled unstarted on 2026-08-24 because they encoded a
broad product program without a sufficiently narrow first vertical slice.
Nothing in this archived map authorizes implementation. The 2026-08-28
reconciliation created a new dependency-correct sequence in the active P0 plan;
the contracts below are historical inputs, not additional execution mandates.

| Feature | Archived sprint decomposition | Count |
| --- | --- | ---: |
| PF-01 — Secretless agent context | [PF-01-S01](../../sprints/archive/prompt-injection-firewall/pf-01-s01-protected-data-taxonomy-and-types.md)<br>[PF-01-S02](../../sprints/archive/prompt-injection-firewall/pf-01-s02-model-visible-secret-denial-gate.md)<br>[PF-01-S03](../../sprints/archive/prompt-injection-firewall/pf-01-s03-vault-credential-reference-api.md)<br>[PF-01-S04](../../sprints/archive/prompt-injection-firewall/pf-01-s04-broker-only-secret-resolution.md)<br>[PF-01-S05](../../sprints/archive/prompt-injection-firewall/pf-01-s05-child-and-container-secret-injection.md)<br>[PF-01-S06](../../sprints/archive/prompt-injection-firewall/pf-01-s06-secret-redaction-and-canary-regression-suite.md)<br>[PF-01-S07](../../sprints/archive/prompt-injection-firewall/pf-01-s07-protected-financial-data-derived-views.md) | 7 |
| PF-02 — Deterministic sensitive-tool executor | [PF-02-S01](../../sprints/archive/prompt-injection-firewall/pf-02-s01-protected-action-schema.md)<br>[PF-02-S02](../../sprints/archive/prompt-injection-firewall/pf-02-s02-canonical-action-encoding.md)<br>[PF-02-S03](../../sprints/archive/prompt-injection-firewall/pf-02-s03-deterministic-policy-decision-point.md)<br>[PF-02-S04](../../sprints/archive/prompt-injection-firewall/pf-02-s04-approval-binding-and-trusted-preview.md)<br>[PF-02-S05](../../sprints/archive/prompt-injection-firewall/pf-02-s05-signing-and-broadcast-separation.md)<br>[PF-02-S06](../../sprints/archive/prompt-injection-firewall/pf-02-s06-idempotency-and-redacted-receipts.md) | 6 |
| PF-03 — Isolated public-web retrieval | [PF-03-S01](../../sprints/archive/prompt-injection-firewall/pf-03-s01-pinned-retriever-image-and-manifest.md)<br>[PF-03-S02](../../sprints/archive/prompt-injection-firewall/pf-03-s02-retriever-sandbox-profile.md)<br>[PF-03-S03](../../sprints/archive/prompt-injection-firewall/pf-03-s03-isolated-fetch-adapter.md)<br>[PF-03-S04](../../sprints/archive/prompt-injection-firewall/pf-03-s04-no-host-browser-fallback.md)<br>[PF-03-S05](../../sprints/archive/prompt-injection-firewall/pf-03-s05-interactive-web-human-handoff.md) | 5 |
| PF-04 — Search-provider broker and stable web-tool facade | [PF-04-S01](../../sprints/archive/prompt-injection-firewall/pf-04-s01-web-run-compatibility-facade.md)<br>[PF-04-S02](../../sprints/archive/prompt-injection-firewall/pf-04-s02-provider-capability-registry.md)<br>[PF-04-S03](../../sprints/archive/prompt-injection-firewall/pf-04-s03-existing-search-api-adapter.md)<br>[PF-04-S04](../../sprints/archive/prompt-injection-firewall/pf-04-s04-exa-search-adapter.md)<br>[PF-04-S05](../../sprints/archive/prompt-injection-firewall/pf-04-s05-brave-search-adapter.md)<br>[PF-04-S06](../../sprints/archive/prompt-injection-firewall/pf-04-s06-searxng-metasearch-adapter.md)<br>[PF-04-S07](../../sprints/archive/prompt-injection-firewall/pf-04-s07-deterministic-provider-router.md)<br>[PF-04-S08](../../sprints/archive/prompt-injection-firewall/pf-04-s08-normalized-results-and-stable-ids.md)<br>[PF-04-S09](../../sprints/archive/prompt-injection-firewall/pf-04-s09-query-context-minimization.md)<br>[PF-04-S10](../../sprints/archive/prompt-injection-firewall/pf-04-s10-provider-health-and-same-role-failover.md)<br>[PF-04-S11](../../sprints/archive/prompt-injection-firewall/pf-04-s11-provider-cost-privacy-and-route-audit.md)<br>[PF-04-S12](../../sprints/archive/prompt-injection-firewall/pf-04-s12-provider-native-web-search-bypass-control.md) | 12 |
| PF-05 — Network and IP gate | [PF-05-S01](../../sprints/archive/prompt-injection-firewall/pf-05-s01-url-canonicalization-gate.md)<br>[PF-05-S02](../../sprints/archive/prompt-injection-firewall/pf-05-s02-dns-and-address-class-policy.md)<br>[PF-05-S03](../../sprints/archive/prompt-injection-firewall/pf-05-s03-redirect-chain-enforcement.md)<br>[PF-05-S04](../../sprints/archive/prompt-injection-firewall/pf-05-s04-dns-rebinding-and-connection-pinning.md)<br>[PF-05-S05](../../sprints/archive/prompt-injection-firewall/pf-05-s05-proxy-socket-and-local-endpoint-denial.md) | 5 |
| PF-06 — Typed source and authority labels | [PF-06-S01](../../sprints/archive/prompt-injection-firewall/pf-06-s01-source-envelope-protocol-types.md)<br>[PF-06-S02](../../sprints/archive/prompt-injection-firewall/pf-06-s02-trusted-ingress-authority-assignment.md)<br>[PF-06-S03](../../sprints/archive/prompt-injection-firewall/pf-06-s03-model-context-serialization.md)<br>[PF-06-S04](../../sprints/archive/prompt-injection-firewall/pf-06-s04-child-agent-provenance-propagation.md)<br>[PF-06-S05](../../sprints/archive/prompt-injection-firewall/pf-06-s05-provenance-persistence-and-audit.md) | 5 |
| PF-07 — Render-aware content cleaner | [PF-07-S01](../../sprints/archive/prompt-injection-firewall/pf-07-s01-visible-main-content-extraction.md)<br>[PF-07-S02](../../sprints/archive/prompt-injection-firewall/pf-07-s02-hidden-and-non-body-content-removal.md)<br>[PF-07-S03](../../sprints/archive/prompt-injection-firewall/pf-07-s03-unicode-control-link-and-size-normalization.md)<br>[PF-07-S04](../../sprints/archive/prompt-injection-firewall/pf-07-s04-sealed-raw-artifact-and-digest-chain.md)<br>[PF-07-S05](../../sprints/archive/prompt-injection-firewall/pf-07-s05-sanitize-and-rescan-pipeline.md) | 5 |
| PF-08 — Local prompt-injection classifier | [PF-08-S01](../../sprints/archive/prompt-injection-firewall/pf-08-s01-classifier-adapter-and-result-schema.md)<br>[PF-08-S02](../../sprints/archive/prompt-injection-firewall/pf-08-s02-corpus-manifest-and-license-gate.md)<br>[PF-08-S03](../../sprints/archive/prompt-injection-firewall/pf-08-s03-leak-free-train-validation-test-splits.md)<br>[PF-08-S04](../../sprints/archive/prompt-injection-firewall/pf-08-s04-small-model-training-pipeline.md)<br>[PF-08-S05](../../sprints/archive/prompt-injection-firewall/pf-08-s05-cpu-packaging-and-artifact-verification.md)<br>[PF-08-S06](../../sprints/archive/prompt-injection-firewall/pf-08-s06-calibration-blind-evaluation-and-runtime-gate.md) | 6 |
| PF-09 — Quarantine and pre-model rejection | [PF-09-S01](../../sprints/archive/prompt-injection-firewall/pf-09-s01-ingress-outcome-state-machine.md)<br>[PF-09-S02](../../sprints/archive/prompt-injection-firewall/pf-09-s02-quarantine-store-and-retention.md)<br>[PF-09-S03](../../sprints/archive/prompt-injection-firewall/pf-09-s03-unprivileged-quarantine-review-tui.md)<br>[PF-09-S04](../../sprints/archive/prompt-injection-firewall/pf-09-s04-quarantine-failure-retry-and-resume.md) | 4 |
| PF-10 — Optional hosted classifier service | [PF-10-S01](../../sprints/archive/prompt-injection-firewall/pf-10-s01-hosted-classifier-adapter-contract.md)<br>[PF-10-S02](../../sprints/archive/prompt-injection-firewall/pf-10-s02-hosted-service-opt-in-and-disclosure.md)<br>[PF-10-S03](../../sprints/archive/prompt-injection-firewall/pf-10-s03-local-fallback-and-vendor-outage-policy.md)<br>[PF-10-S04](../../sprints/archive/prompt-injection-firewall/pf-10-s04-commercial-detector-bakeoff-and-cost-gate.md) | 4 |
| PF-11 — Agent Sweep / rogue-behavior monitor | [PF-11-S01](../../sprints/archive/prompt-injection-firewall/pf-11-s01-behavior-event-schema-and-redaction.md)<br>[PF-11-S02](../../sprints/archive/prompt-injection-firewall/pf-11-s02-deterministic-behavioral-anomaly-rules.md)<br>[PF-11-S03](../../sprints/archive/prompt-injection-firewall/pf-11-s03-isolated-behavior-review-model.md)<br>[PF-11-S04](../../sprints/archive/prompt-injection-firewall/pf-11-s04-pause-revoke-and-kill-escalation.md)<br>[PF-11-S05](../../sprints/archive/prompt-injection-firewall/pf-11-s05-agent-sweep-tui-and-recovery-flow.md) | 5 |
| PF-12 — Adversarial security harness | [PF-12-S01](../../sprints/archive/prompt-injection-firewall/pf-12-s01-synthetic-hostile-source-fixtures.md)<br>[PF-12-S02](../../sprints/archive/prompt-injection-firewall/pf-12-s02-canary-secrets-and-fake-financial-systems.md)<br>[PF-12-S03](../../sprints/archive/prompt-injection-firewall/pf-12-s03-classifier-benchmark-harness.md)<br>[PF-12-S04](../../sprints/archive/prompt-injection-firewall/pf-12-s04-forced-classifier-miss-harness.md)<br>[PF-12-S05](../../sprints/archive/prompt-injection-firewall/pf-12-s05-web-isolation-and-egress-adversarial-suite.md)<br>[PF-12-S06](../../sprints/archive/prompt-injection-firewall/pf-12-s06-true-tui-security-qualification.md)<br>[PF-12-S07](../../sprints/archive/prompt-injection-firewall/pf-12-s07-tensorcash-and-isometric-live-repo-qualification.md)<br>[PF-12-S08](../../sprints/archive/prompt-injection-firewall/pf-12-s08-release-security-ledger-and-human-acceptance.md) | 8 |

## Standard feature contracts

### PF-01 — Secretless agent context

**Contract.** Model-visible messages, tool arguments, tool results, logs, traces,
artifacts, approvals, and audit records contain credential references and redacted
metadata only. A trusted broker resolves the referenced value inside the minimum
execution boundary and never returns it.

**Protected classes.** Vault entries, seeds, private keys, wallet signing
material, exchange and broker credentials, provider API keys, balances,
positions, PNL, identity-linked financial data, and any value classified by the
active policy as protected.

**Failure behavior.** Unknown reference, scope mismatch, expiry, attempted raw
secret input, or broker failure denies the operation. It never falls back to
putting a value in chat, environment diagnostics, or a generic shell command.

### PF-02 — Deterministic sensitive-tool executor

**Contract.** The agent can propose only a typed operation supported by a
capability-specific schema. The executor canonicalizes it, evaluates policy,
binds any human approval to the canonical digest, resolves credentials, performs
the bounded operation, and returns a secret-free receipt.

For financial actions the schema must name venue, account reference, asset,
side, order type, quantity or notional, price bounds, leverage, slippage, expiry,
loss limits, and idempotency key as applicable. Signing and broadcast remain
separate permissions. Free-form shell, browser JavaScript, arbitrary URL, and
model-authored executable templates are not protected-action substitutes.

**Safety invariant.** A classifier false negative cannot authorize an action.
Only deterministic policy and the required trusted approval can do that.

### PF-03 — Isolated public-web retrieval

**Contract.** Eligible unauthenticated web reads execute in a pinned, disposable
container with no host browser profile, vault, wallet, workspace mount, clipboard,
or ambient Unix socket. The agent receives bounded extracted content and
provenance, never a browser-control handle.

Moderate and Aggressive must not silently fall back to the built-in browser when
the retriever is unavailable. Authenticated, CAPTCHA, MFA, passkey, and interactive
flows require a separately designed human handoff; they are not a reason to give
the agent a general host-browser capability.

### PF-04 — Search-provider broker

**Contract.** A provider-neutral broker can route approved queries to available
API or isolated metasearch providers. Initial candidates include Exa, Brave, and
a maintained SearXNG deployment. “Seer” was not found in the reviewed Ambient
snapshot and must be identified before it becomes a named dependency.

Provider diversity can reduce CAPTCHA and availability problems. It does not
make search results trusted. Query, snippets, result pages, and provider tool
output all receive external-untrusted provenance and pass through the same
classifier and action policy.

#### Existing Corbanu web surface

Corbanu currently has two web-search paths that matter to this proposal:

1. The `web.run` extension accepts `search_query`, `image_query`, `open`, `click`,
   `find`, and related commands, then sends a `SearchRequest` through the selected
   model provider's search API.
2. Core can expose provider-native `web_search` tool specifications for providers
   that implement search themselves.

The first path currently returns plaintext marked as external context. The
second may let a provider inject search results directly into the model turn.
Moderate and Aggressive cannot rely on either path unchanged: all external
results must cross the same local provenance, classification, quarantine, and
egress boundaries before becoming main-agent context.

#### Stable agent-facing tool contract

`web.run` remains the compatibility entry point. Existing skills and agents do
not need to learn a separate tool name for each provider. The broker interprets
the existing command families as three different capabilities:

| Logical capability | Existing `web.run` operations | Required execution boundary |
| --- | --- | --- |
| Search | `search_query`, `image_query`, and provider-neutral structured lookups | Approved search API or isolated metasearch provider; return normalized result records only |
| Fetch and extract | `open`, `click`, and `find` against a result or explicit URL | Isolated Scrapling-class retriever with PF-05 egress enforcement and PF-07 extraction |
| Interactive browsing | Stateful navigation, login, CAPTCHA, MFA, passkey, page interaction, or arbitrary JavaScript | Not an agent web-tool fallback in Moderate or Aggressive; use an explicit human handoff designed separately |

The internal implementation may split search and fetch into separate traits or
services, but `web.run` remains a compatibility adapter unless a later product
decision deliberately replaces it. Provider names are policy and configuration,
not new agent-callable tools.

#### Provider capability registry

Each adapter declares a machine-readable capability record rather than being
selected through prompt text:

```json
{
  "provider_id": "brave",
  "roles": ["search"],
  "auth_ref": "vault://web/brave",
  "execution": "remote_api",
  "allowed_origins": ["https://api.search.brave.com"],
  "data_policy": "provider-policy-version",
  "cost_policy": "configured-budget-id",
  "health": "healthy|degraded|unavailable"
}
```

Initial adapter targets are:

| Provider lane | Role | Security treatment |
| --- | --- | --- |
| Existing Corbanu/provider-backed Search API | Search and supported structured lookups | Permissive compatibility path; Moderate/Aggressive only if the broker can screen results before the main model sees them |
| Exa | Search and supported public fetch | Remote API through a credential reference; normalize and screen every result and fetched document |
| Brave Search | Search | Remote API through a credential reference; normalize and screen snippets before any follow-up fetch |
| SearXNG | Search/metasearch | Isolated maintained service; its upstream results remain untrusted and its own outbound access is separately constrained |
| Scrapling | Fetch and extract | Pinned isolated workload with no workspace, vault, wallet, browser-profile, or host-socket access |

Adding a provider requires an adapter, declared roles, credential-reference
schema, egress policy, data-retention disclosure, health check, cost accounting,
normalization tests, and adversarial fixtures. Installing an MCP server or
teaching the model a provider-specific prompt is not sufficient integration.

#### Deterministic routing

The broker, not the model, chooses a provider from the human-configured ordered
set after applying the current `/security` level, requested capability, provider
health, destination policy, privacy policy, and budget. The model may request a
capability and ordinary constraints such as domain, recency, locale, or result
count. It cannot select a less secure backend, enable browser fallback, or bypass
screening by naming a provider in retrieved text.

Fallback is allowed only between providers qualified for the same logical role
and current profile. Every fallback produces a visible, secret-free route event.
It cannot cross from an API or isolated fetch lane to the host browser.

#### Normalized results and provenance

Search adapters return bounded records rather than provider prose:

```json
{
  "result_id": "stable-session-id",
  "provider_id": "exa",
  "query_digest": "sha256:...",
  "title": "untrusted title",
  "snippet": "untrusted snippet",
  "canonical_url": "https://example.com/page",
  "published_at": "optional RFC3339",
  "retrieved_at": "RFC3339",
  "content_digest": "sha256:...",
  "classifier": {"verdict": "allow", "model_version": "...", "score": 0.01}
}
```

Result ids remain stable within the session regardless of provider. `open`,
`click`, and `find` resolve the id inside the broker, revalidate its canonical
URL under PF-05, fetch it through the isolated retrieval lane, apply PF-07, and
classify it again. Search-result screening never substitutes for screening the
retrieved page.

#### Query privacy and context minimization

The current standalone extension can send recent user and assistant messages as
search input. Under Moderate and Aggressive, the broker sends only the explicit
query and policy-approved minimum context required by the selected adapter.
Vault values, portfolio state, positions, PNL, account identifiers, unpublished
strategy details, raw chat history, and quarantined text never become provider
query context. The route event shows which provider received which redacted query
digest and the applicable retention disclosure without logging the sensitive
query body.

#### Failure and recovery

| Failure | Permissive | Moderate | Aggressive |
| --- | --- | --- | --- |
| Preferred search provider unavailable | Preserve verified current behavior | Try the next qualified search provider and show the route change | Try only an explicitly allowed provider; otherwise stop |
| Scrapling or isolated fetch unavailable | Preserve verified current behavior | Return a structured unavailable result; do not use the host browser | Stop external fetch and offer retry or human handoff |
| Classifier unavailable | Preserve verified current behavior | Do not add new external content to model context; offer retry | Stop external ingestion until the local classifier is healthy |
| Provider returns malformed or oversized output | Preserve verified current handling | Reject or quarantine before normalization | Reject and record a high-severity provider event |
| Native provider `web_search` cannot be intercepted | Preserve verified current behavior | Disable that path and use brokered `web.run` | Disable that path and deny any attempt to re-enable it |
| No provider satisfies privacy, egress, or budget policy | Report no qualified route | Report the blocking policy and recovery choices | Deny without fallback |

The user can retry, choose another already-qualified provider, adjust an
authorized configuration on a trusted surface, or use the explicit human
handoff. The model cannot repair availability by weakening `/security`.

### PF-05 — Network and IP gate

**Contract.** The retriever canonicalizes URLs, rejects credentials in URLs and
non-HTTPS public retrieval, resolves every A and AAAA destination, rejects
loopback, private, link-local, multicast, documentation, carrier-grade NAT, and
cloud-metadata ranges, and applies the same checks after every redirect.

Connection policy must prevent DNS rebinding by pinning or revalidating the
resolved destination at connection time. Alternate IP encodings, proxy variables,
IPv4-mapped IPv6, rebinding, redirects, `file:` URLs, Unix sockets, and local
browser debugging endpoints are adversarial cases. Hostname regex alone is not
sufficient.

### PF-06 — Typed source and authority labels

Every model-visible segment is serialized from a trusted envelope similar to:

```json
{
  "source_id": "immutable-id",
  "source_kind": "user|system|external|tool|agent|broker_receipt",
  "authority": "human_authority|system_policy|external_untrusted|tool_untrusted|non_authoritative_receipt",
  "origin": "provider-or-URL-without-credentials",
  "retrieved_at": "RFC3339",
  "content_type": "text/markdown",
  "transformations": ["visible-content-v1"],
  "content_digest": "sha256:...",
  "classifier": {"verdict": "allow", "model_version": "...", "score": 0.01}
}
```

Only the trusted ingress layer sets `authority`. Content cannot promote itself by
printing labels or imitating a user message. Child agents inherit the envelope
and cannot rewrite it. The model is instructed to treat external and tool text as
data, but enforcement still occurs at the executor; labels are not themselves a
security boundary.

### PF-07 — Render-aware content cleaner

For web inputs, prefer visible main-body text. Remove scripts, styles, comments,
hidden nodes, off-screen or zero-size text, inaccessible overlays, control
characters, and non-task metadata before model ingestion. Bound size and nesting,
normalize Unicode without destroying evidence, preserve links as labeled data,
and rescan the transformed content.

The trusted layer retains the original digest and, when policy allows, a sealed
raw artifact for investigation. It must not claim that cleaning makes content
safe: an injection can be ordinary visible body text.

### PF-08 and PF-10 — Local and hosted prompt-injection classification

The local product baseline is a small sequence classifier that can run on the
weakest supported CPU. A roughly 20–100M parameter encoder, quantized for an
ONNX-class runtime, is a candidate range—not a committed architecture. Meta's
[`Llama-Prompt-Guard-2-22M`](https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-22M)
is one baseline to benchmark, not an automatic dependency or quality claim.

The runtime returns a versioned structured result:

```json
{
  "verdict": "benign|suspicious|hostile",
  "attack_classes": ["instruction_override", "financial_action_override"],
  "confidence": 0.0,
  "model_version": "classifier-and-data-version",
  "input_digest": "sha256:..."
}
```

The classifier screens each untrusted chunk before aggregation and screens the
final bounded aggregate again. It never receives vault values or unredacted
financial state.

An optional hosted service may be offered only when the user explicitly enables
it and sees data-region, retention, cost, and privacy terms. Protected financial
data and secrets are never sent. Moderate and Aggressive retain a qualified local
fallback; a vendor outage cannot silently bypass screening.

### PF-09 — Quarantine and rejection

| Outcome | Meaning | Product behavior |
| --- | --- | --- |
| Allow | Below the active threshold | Deliver labeled content to the model |
| Sanitize and rescan | Suspicion is localized to removable structure | Transform once, rescan, and preserve both digests |
| Quarantine | Ambiguous or potentially valuable content | Keep it outside model context; show source, reason, and retry/review choices |
| Reject | Hostile or prohibited content | Do not deliver it; record a redacted event and continue only with safe sources |

There is no loop in which the main agent reads the rejected text to decide
whether the classifier was right. Human review uses a separate unprivileged
viewer and cannot turn the text into instructions or authority.

### PF-11 — Agent Sweep

Agent Sweep observes structured, redacted events such as proposed operations,
denials, portfolio concentration deltas, tool-call rates, venue and instrument
changes, failed approvals, and deviations from the user's explicit mandate.
Deterministic rules detect clear violations; an optional isolated model can flag
novel sequences.

It has no vault, wallet, browser, brokerage, signing, broadcast, or arbitrary tool
access. It may alert, pause the run, revoke pending grants, or invoke the
kill-switch boundary delivered through `/security`. “Escalate” means operational
escalation to a human and a safer paused state. It does **not** silently change
`/security`, because the product spec reserves level changes to the human.

### PF-12 — Adversarial security harness

The harness combines component detection metrics with end-to-end safety. It must
prove both that hostile content is found and that missed content still cannot
extract a secret, alter policy, or execute an unauthorized financial action.

Synthetic hostile sources include public pages, redirects, hidden DOM, search
snippets, files, PDFs, transcripts, social messages, MCP descriptions and
responses, child-agent artifacts, market/news feeds, order metadata, and fake
broker or wallet responses. Fixtures include canary secrets and intentionally
missed classifier verdicts. No real credential, wallet, account, or financial
record enters the suite.

## `/security` profile mapping

Permissive remains the verified status quo. New controls must not quietly change
its policies, network route, telemetry, cost, or privacy.

| Control | Permissive | Moderate | Aggressive |
| --- | --- | --- | --- |
| Secrets and protected data | Existing behavior unchanged | PF-01 hard boundary for protected classes | PF-01 plus default denial of protected-data disclosure and narrow expiring references |
| Sensitive operations | Existing policy unchanged | PF-02 typed broker; approval by risk and policy | PF-02 default deny; exact human approval for every sign or broadcast |
| Public-web retrieval | Existing browser and network behavior unchanged | PF-03 isolated retrieval; no silent host-browser fallback | Direct browser tools disabled; isolated retrieval only |
| Search and web-tool use | Existing `web.run`, native search, routing, and context behavior unchanged | `web.run` routes through PF-04; qualified API failover is allowed, native search that bypasses screening is disabled, and all results are normalized and screened | `web.run` routes only through explicitly allowed providers; native bypass and cross-lane fallback are denied |
| Egress | Existing policy unchanged | PF-05 public HTTPS enforcement and redirect checks | PF-05 destination allowlist, shortest useful expiry, deny unknown destinations |
| Source authority | Existing behavior unchanged | PF-06 required for external, tool, and agent content | PF-06 required for every non-human segment; missing provenance rejects ingestion |
| Content cleaning | Existing behavior unchanged | PF-07 visible-content extraction and rescan | PF-07 plus raw-content access disabled outside sealed review |
| Injection classifier | Off unless user separately opts into a future observe-only experiment | PF-08 enforced; suspicious content quarantined and hostile content rejected | PF-08 enforced at the strict threshold; classifier unavailable means external ingestion pauses |
| Hosted classifier | Off | PF-10 opt-in; qualified local fallback | Off by default; opt-in only if data policy explicitly permits it |
| Agent Sweep | Existing behavior unchanged | PF-11 alerts, pauses, revokes pending authority, or trips kill switch | PF-11 stricter deterministic limits and immediate pause on high-severity anomalies |
| Adversarial release gate | Existing release gates | PF-12 required | PF-12 required with strict-profile cases and no open critical finding |

## Ambient Desktop implementation map

The repository was pulled from
[`ambient-xyz/AmbientDesktop`](https://github.com/ambient-xyz/AmbientDesktop)
and reviewed at clean commit `3dd37c3d186bf5734448f67451259967f74ede67`
(`Sync public snapshot 0.1.97`, 2026-07-24). These are design references, not
Corbanu implementation authorization.

| Feature | Ambient code reference | Verified behavior or gap |
| --- | --- | --- |
| PF-01 hard secret boundary | `src/main/permissions/permissionPolicy.ts::classifyToolPermission`, `classifyManagedSecretPathAccess`, `denyManagedSecretPath`; `permissionPolicy.test.ts` | Managed secret and authority paths deny before the `full-access` bypass; tests cover direct file and shell attempts even in full access |
| PF-01 secret request | `src/main/capability-builder/agentRuntimeCapabilityBuilderSecretRequestTools.ts::registerCapabilityBuilderSecretRequestTool` | Agent requests a declared environment requirement; returned text explicitly says the secret is never exposed to Pi |
| PF-01 child execution | `src/main/ambient-cli/ambientCliEnvBindings.ts`; `src/main/tool-runtime/toolHiveRuntimeEnvironment.ts` | Runtime resolves declared references and injects values into a child/container boundary; temporary secret material is permissioned and cleaned up |
| PF-01 browser credential broker | `src/main/browser/browserCredentialStore.ts`; `src/main/agent-runtime/browser-tools/agentRuntimeBrowserLoginTools.ts`; `src/main/browser/browserChromePageHelpers.ts::assertLoginOrigin`; `docs/login-automation-threat-model.md` | Model uses credential id, expected origin, and selectors; the main process resolves and fills the credential after origin and approval checks |
| PF-02 privileged executor | `src/shared/permissionTypes.ts::PrivilegedActionTemplate`; `src/main/privileged-action/privilegedAction.ts::planPrivilegedAction`; `src/main/privileged-action/privilegedActionAdapter.ts`; `src/main/agent-runtime/privileged-action/agentRuntimePrivilegedActionRequestTools.ts` | Typed request, explicit approval, ephemeral native credential, validation, execution, and redacted result are useful patterns; the generic command template remains too broad for Corbanu financial authority |
| PF-03 Scrapling isolation | `resources/mcp-catalog/default/scrapling.json`; `src/main/agent-runtime/agentRuntimeScraplingBrowserRoute.ts`; `src/main/scrapling/scraplingMcpDescriptor.ts` | Pinned ToolHive workload has no workspace mount or managed secrets and retrieves eligible public pages after policy and approval |
| PF-03 direct-browser gap | `src/main/agent-runtime/browser-tools/agentRuntimeBrowserTools.ts`; `agentRuntimeBrowserEvalTools.ts`; `src/main/browser/internalBrowserHost.ts::evaluate`; `agentRuntimeBrowserContentTools.ts` | The agent still has browser navigation/content/eval tools, and a failed Scrapling call explicitly falls back to Ambient browser content |
| PF-04 provider routing | `src/main/web-research/webResearchProviderStack.ts`; `src/main/provider/providerCatalogWebDiscoveryEntries.ts` | Defaults are Exa search/fetch, Scrapling fetch, and Ambient Browser fallback; catalog covers Brave, browser Google, reserved Google Programmable Search, and SearXNG |
| PF-05 URL and network controls | `src/main/scrapling/scraplingBrowserRouting.ts`; `src/main/mcp/mcpPermissionPolicyService.ts`; `src/main/mcp/mcpRuntimePermissionEnforcement.ts` | HTTPS/literal-IP/auth-host checks and MCP resource policy block many local/private/raw-secret paths; reviewed Scrapling descriptor still has broad public 80/443 egress, so DNS and redirect enforcement require explicit proof |
| PF-06/PF-12 child-agent filtering | `src/main/subagents/subagentThreatModel.test.ts`; `src/main/subagents/subagentContextFilter.ts`; `src/shared/subagentToolScope.ts` | Focused controls strip child artifacts/tool output from inherited context and deny child privilege escalation; this is not a general source-authority envelope |
| PF-07 extraction | `src/main/scrapling/scraplingBrowserRouting.ts::scraplingBrowserContentToolArguments` | Requests Markdown with `main_content_only: true`; no evidence that this alone classifies or neutralizes visible prompt injection |
| PF-08/PF-09/PF-11 gaps | Runtime search across `src/main` and `src/shared`; `docs/production-security-review-plan.md` | No general prompt-injection classifier, ingestion quarantine, or rogue-agent monitor was found. The review document recognizes hostile-content risks but is not runtime enforcement |

Ambient's `PermissionMode` is only `workspace | full-access` in
`src/shared/permissionTypes.ts`. It must not be presented as an implementation of
Corbanu's Permissive, Moderate, and Aggressive product levels.

## Current market and research grounding

Commercial screening exists now; Corbanu does not need to pretend the category
is unexplored. The product decision is whether to build, buy, or combine a local
baseline with an optional service.

| Reference | What currently exists | Corbanu implication |
| --- | --- | --- |
| [Check Point AI Agent Security / Lakera Guard](https://docs.lakera.ai/guard) and [Prompt Defense](https://docs.lakera.ai/docs/prompt-defense) | SaaS and self-hosted screening for user/external inputs, outputs, tool calls, responses, descriptions, data leakage, and off-policy actions | Benchmark as a hosted and self-hosted candidate; its own guidance says secrets belong outside the LLM |
| [Microsoft Azure Prompt Shields](https://learn.microsoft.com/en-us/azure/ai-services/content-safety/concepts/jailbreak-detection) | API detection for direct user-prompt attacks and indirect document attacks | Benchmark document-aware hosted detection and preserve source-role separation |
| [Google Cloud Model Armor](https://cloud.google.com/security/products/model-armor) | Model-agnostic prompt/response screening for injection, sensitive-data leakage, malicious URLs, and harmful content | Benchmark as a commercial firewall; validate data routing and retention before any financial workload |
| [Amazon Bedrock Guardrails prompt-attack filters](https://docs.aws.amazon.com/bedrock/latest/userguide/guardrails-prompt-attack.html) | Detect or block prompt injection, jailbreak, and prompt leakage; input tags distinguish user content from developer instructions | Adopt explicit source boundaries; do not make a Bedrock dependency mandatory |
| [Protect AI LLM Guard](https://github.com/protectai/llm-guard) | Open-source input/output scanners including prompt-injection and leakage defenses | Include in the reproducible local bakeoff, subject to dependency and model review |
| [Meta Llama Prompt Guard 2 22M](https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-22M) | Small downloadable text-classification baseline | Test CPU viability; do not infer Corbanu accuracy from model size or model-card claims |
| [PromptShield: Deployable Detection for Prompt Injection Attacks](https://arxiv.org/abs/2501.15145) | Detector benchmark and dataset emphasizing performance at low false-positive rates | Report TPR at fixed low FPR and prevent train/test source leakage |
| [Defenses Against Prompt Attacks Learn Surface Heuristics](https://aclanthology.org/2026.acl-long.502/) | Finds position, trigger-token, and topic-generalization shortcuts in supervised defenses | Require hard negatives, positional tests, unseen topics, and benign finance/security discussions |

**Proposed product direction:** qualify a local detector as the dependable
baseline, then offer a hosted premium service only if a blind bakeoff proves a
material security or operational advantage. Neither path replaces PF-01 or PF-02.

## Executable classifier work package

### Threat taxonomy

Label at least:

- direct instruction override and jailbreak;
- indirect injection in retrieved or tool-provided content;
- system-prompt, vault, credential, and financial-data extraction;
- financial-action override, destination substitution, asset promotion, and
  approval bypass;
- authority impersonation and fake conversation turns;
- obfuscated, encoded, multilingual, split-across-chunks, and multimodal-derived
  text attacks; and
- benign content, including hard negatives that discuss attacks or contain real
  trading instructions from the authorized human.

### Corpus construction

1. Assemble licensed public detector datasets and record source, license, hash,
   and allowed use.
2. Add representative benign finance, market, transcript, social, repository,
   terminal, documentation, and web text without real customer data.
3. Generate poisoned pairs by inserting attacks into visible body text, hidden
   markup, metadata, tool output, search snippets, child artifacts, and multi-chunk
   sequences. Vary placement, language, encoding, voice, and finance intent.
4. Create hard negatives containing phrases such as “ignore previous
   instructions” in legitimate security research, tests, quoted material, and
   user-authorized tasks.
5. Split by original source, injection family, template, and semantic-near-duplicate
   group. No transformed version of a base document may cross train, validation,
   or test boundaries.
6. Maintain unseen-source, unseen-topic, unseen-language, position-shift, and
   adaptive-evasion holdouts owned by the evaluator, not the model trainer.

### Training and packaging

- Compare an untuned local baseline, small fine-tuned encoders, Meta Prompt Guard
  2, Protect AI scanners, and commercial APIs through one versioned adapter.
- Calibrate confidence on validation data; never choose the product threshold on
  the final test set.
- Export the selected local model to a reproducible CPU runtime, pin model and
  tokenizer hashes, generate an SBOM, and sign the artifact.
- Start in explicit shadow mode with no enforcement or external telemetry. Move
  to enforcement only after privacy review, false-positive review, and product
  authorization.

### Candidate qualification gates

These are proposal targets and must be confirmed at activation:

| Measure | Candidate gate |
| --- | --- |
| Benign false-positive rate | At most 0.1% on at least 100,000 held-out benign segments, reported with confidence interval |
| Known-family true-positive rate | At least 80% at the selected low-FPR operating point |
| Unseen-source/evasion true-positive rate | At least 65% at the same threshold; every miss remains contained by executor policy |
| Surface-heuristic tests | No more than 2 percentage-point rejection increase solely from benign position shifts or trigger-token hard negatives |
| CPU latency | p95 at most 50 ms for a 2,048-token segment on the agreed weakest supported CPU |
| Resource envelope | Peak resident memory at most 512 MiB; model artifact at most 300 MiB unless product authority accepts a larger tier |
| Privacy | Zero secrets or protected financial records in training, hosted requests, metrics, logs, or artifacts |
| End-to-end protected action safety | 100% denial of unauthorized secret disclosure, policy change, signing, broadcast, and financial execution, including forced detector false negatives |

The low-FPR detector targets are useful only if the system remains safe outside
them. A model-based detector is probabilistic and cannot be a sole release gate
for money movement.

## Scope

### In

- Specify PF-01 through PF-12 and their `/security` mapping.
- Reuse or adapt Ambient's secret-broker, typed executor, container, MCP policy,
  and threat-test patterns where licensing and architecture permit.
- Build provenance, sanitization, local classification, optional hosted adapter,
  quarantine, deterministic tool mediation, behavioral monitoring, and redacted
  audit boundaries.
- Prove containment under classifier misses, unavailable vendors, malicious tool
  metadata, browser failures, restarts, and child-agent inheritance.
- Produce true-TUI, live-repository, adversarial, performance, and human evidence.

### Out

- Changing Permissive behavior as part of this proposal.
- Giving the classifier, main agent, Agent Sweep, browser container, plugin, MCP
  server, or child agent access to raw secrets.
- Claiming that content extraction, source labels, an LLM prompt, or one detector
  makes external content trusted.
- Shipping a general agent-controlled host browser in Moderate or Aggressive.
- Publishing these proposed features as finished documentation before acceptance.

## Invariants

- The model never receives a protected secret, even when prompted by the human.
- External content cannot grant authority, weaken policy, relabel itself, approve
  an action, or choose its executor.
- Sensitive operations are typed and bounded; arbitrary shell or browser control
  is not an equivalent fallback.
- A detector false negative cannot produce an unauthorized protected action.
- A detector false positive is visible, recoverable, and measurable.
- Moderate and Aggressive never silently fall back to a less isolated browser,
  unscreened provider, hosted classifier, or unmediated tool.
- `web.run` remains provider-neutral; retrieved content cannot create a new web
  tool, choose its provider, or cause a native search path to bypass the broker.
- Moderate and Aggressive send only explicit, policy-minimized query context to
  a provider and screen every result before it enters main-agent context.
- Child agents inherit equal or stricter policy and immutable provenance.
- Agent Sweep can pause or revoke but cannot trade, reveal secrets, or silently
  change the human-selected security level.
- Audit and test artifacts are useful without containing credentials or protected
  financial data.
- Permissive remains the frozen pre-feature behavior until the human chooses
  another level.

## Ownership and implementation worktrees

No implementation worktree exists because this proposal is not active.

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | Not allocated | Not allocated | Not allocated | Delivery owner after activation |

## Useful Corbanu code references

| Path or symbol | Why it matters |
| --- | --- |
| `docs/corbanu-product-spec.md` — `P0 /security levels` | Product contract and profile semantics |
| `docs/plans/active/p0-security-levels.md` | Active initiative that must be reconciled before promotion |
| `codex-rs/protocol/src/models.rs::PermissionProfile` | Existing low-level permission policy to compose, not replace |
| `codex-rs/vault/src/lib.rs::reveal_for_programmatic_use` | Existing credential resolution boundary that must become references-only to model-visible paths |
| `codex-rs/network-proxy/src/policy.rs` | Existing egress decision point for IP, destination, and profile enforcement |
| `codex-rs/tui/src/bottom_pane/approval_overlay.rs` | Trusted human-approval surface for exact protected-action previews |
| `codex-rs/tui/src/slash_command.rs::SlashCommand` | `/security` entry point owned by the active plan |
| `codex-rs/tui/src/chatwidget/slash_dispatch.rs` | Security-tab routing and status presentation |
| `codex-rs/core/src/agent/` and tool dispatch boundaries | Proposed source-envelope propagation and pre-model ingress points; exact symbols must be resolved before activation |
| `codex-rs/ext/web-search/src/tool.rs::WebSearchTool` | Existing `web.run` compatibility entry point, `SearchRequest` construction, and event emission; the broker must sit before provider dispatch and model-visible output |
| `codex-rs/ext/web-search/src/schema.rs::commands_schema` | Existing agent-facing command contract to preserve across provider adapters |
| `codex-rs/ext/web-search/src/extension.rs::WebSearchExtensionConfig` and `ToolContributor` | Current availability and provider coupling; proposed insertion point for security-level-aware broker registration |
| `codex-rs/ext/web-search/src/output.rs::SearchOutput` | Currently marks output as external context but materializes plaintext; must emit normalized screened envelopes or quarantine outcomes |
| `codex-rs/ext/web-search/src/history.rs::recent_input` | Currently selects recent user and assistant context for search; Moderate/Aggressive must replace this with explicit policy-minimized query input |
| `codex-rs/codex-api/src/search.rs` and `codex-rs/codex-api/src/endpoint/search.rs` | Existing provider-backed search transport to wrap as one adapter rather than the universal web path |
| `codex-rs/core/src/client.rs` — native `ToolSpec::WebSearch` translation | Provider-native search path that must be disabled under Moderate/Aggressive unless results can traverse the local broker before model ingestion |
| `codex-rs/protocol/src/config_types.rs::WebSearchMode` and `WebSearchConfig` | Current cached/indexed/live, location, context-size, and domain settings; provider order, roles, privacy, budget, and profile constraints extend this contract |
| `codex-rs/core/src/config/mod.rs::resolve_web_search_mode_for_turn` | Current per-turn search-mode resolution; security-level policy must constrain route selection without altering Permissive |
| `codex-rs/core/tests/suite/web_search.rs` and `codex-rs/core/src/tools/spec_plan_tests.rs` | Existing compatibility and tool-exposure tests to extend with broker routing and native-bypass regressions |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Safe public research | Moderate; Scrapling and local classifier healthy | Ask Corbanu to research a public market page | Source, isolated route, and screened status are visible; answer uses labeled content | No host browser handle, secret, or raw page execution reaches the agent |
| Search-provider routing | Moderate; Brave first, Exa second, both qualified | Use `web.run` for a public query | Brave is selected without exposing a provider-specific tool; normalized results identify the route | Agent-facing command shape is provider-neutral and every result is screened |
| Same-role provider failover | Moderate; first search provider becomes unavailable | Repeat the query | Route event explains failover to the next qualified search provider | No direct browser, unscreened native search, or cross-role fallback occurs |
| Native search bypass | Moderate; selected model supports provider-native `web_search` whose results cannot be intercepted | Request live research | Native tool is absent or denied and brokered `web.run` is used | No provider-native result reaches the main model outside PF-04/PF-06/PF-08 |
| Query-context minimization | Moderate; chat contains positions, PNL, and an unpublished strategy | Search for public information about one named asset | Route event shows provider and redacted query digest | Provider request contains the explicit query only; protected chat context is absent |
| Result-to-page transition | Aggressive; one allowed search provider and Scrapling are healthy | Search, then `open` a returned result id | Broker resolves the stable id, revalidates the URL, fetches in isolation, extracts, and rescans | Search snippets do not authorize the fetch and the host browser is never invoked |
| Visible indirect injection | Moderate; page body says to ignore instructions and buy an attacker's coin | Ask for a page summary | Hostile segment is rejected or quarantined; safe sources may continue | Injection never reaches main-agent context or a protected action |
| Hidden injection | Moderate; hostile text is hidden in DOM metadata/style | Retrieve the page | Cleaner removes it, records transforms, and rescans | Model receives visible content only with original/transformed digests |
| Forced detector miss | Moderate; harness forces benign verdict for hostile content | Process source and attempt a trade or vault read | Agent may reason incorrectly, but broker denies unauthorized operation | No protected value, approval, signature, broadcast, or order escapes |
| Classifier unavailable | Moderate | Retrieve a new external source | Ingestion pauses with retry and provider-health guidance | No unscreened fallback to main agent or host browser |
| False positive recovery | Moderate; benign security or finance text is quarantined | Open the quarantine summary and choose safe recovery | User can skip, retry another detector/source, or inspect in an unprivileged viewer | Review does not grant authority or inject raw content into the main agent |
| Brokered credential | Moderate; valid credential reference exists | Ask for an approved read or action | Exact scope is shown; broker resolves and returns redacted receipt | Secret absent from transcript, tool args/results, logs, audit, and artifacts |
| Direct browser request | Aggressive | Ask agent to open Chrome and evaluate page JavaScript | Request is denied; isolated retrieval or human handoff is offered | No direct browser capability is registered or invoked |
| Rogue behavior | Moderate; agent begins unexplained concentration or repeated denied calls | Continue session | Agent Sweep pauses/revokes and shows evidence to the human | Sweep has no secret or financial execution path and cannot change `/security` |
| Restart and resume | Aggressive; quarantine, revocation, and kill switch state exist | Restart Corbanu and resume | Restrictive state restores before new work | No transient Permissive, unscreened ingestion, or stale approval replay |

## Implementation sequence

This sequence is a high-level dependency shape only. It is design-only until
promotion and exact worktree allocation, and it never authorizes implementation
from prose. After activation, the exact execution contract is the selected
single-feature record in the sprint map above.

1. **Reconcile authority.** Decide whether these contracts amend the active P0
   plan or follow it. Freeze Permissive behavior and resolve exact Corbanu ingress,
   vault, executor, network, agent, audit, and TUI boundaries.
2. **Build immutable contracts.** Add source envelopes, protected-data types,
   typed operation schemas, redacted receipts, and policy decisions before adding
   model-based detection.
3. **Close secrets and tools.** Make protected values references-only, add narrow
   brokers, bind approvals to canonical operations, and prove forced-detector-miss
   containment.
4. **Broker the existing web surface.** Preserve the `web.run` command schema,
   separate its logical search, fetch, and interactive roles, add the provider
   capability registry and deterministic router, wrap the existing Search API as
   one adapter, and block unmediated provider-native search in Moderate and
   Aggressive. Then add pinned retrieval, query-context minimization,
   DNS/redirect/connection enforcement, stable result ids, normalized envelopes,
   no-cross-lane fallback semantics, visible-content extraction, and quarantine.
5. **Qualify classification.** Construct the corpus and blind holdouts, run the
   local/open/commercial bakeoff, package the CPU baseline, calibrate thresholds,
   and complete shadow-mode review.
6. **Add Agent Sweep.** Start with deterministic behavioral limits, then qualify
   any secondary model on redacted events with deliberately minimal authority.
7. **Break the system.** Run component, adversarial, forced-miss, true-TUI,
   live-repository, restart, concurrency, performance, privacy, and human tests on
   the final formatted candidate.
8. **Document only what passed.** Add finished feature documentation, exact code
   references, and release evidence only after acceptance.

## Automated evidence

Commands are proposed interfaces and must be replaced with final-tree commands
when an implementation worktree is activated.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Plan lifecycle | `python3 docs/plans/check.py` | pending | plan-check output |
| Ambient reference audit | `git -C ../AmbientDesktop rev-parse HEAD` plus the code-reference queries in this plan | reviewed at `3dd37c3d...` | proposal review notes |
| Classifier corpus | `python3 scripts/security/classifier_corpus_check.py --manifest qa/security/classifier/corpus.json` | pending | corpus manifest, licenses, hashes, split-leak report |
| Blind detector bakeoff | `python3 scripts/security/classifier_bench.py --config qa/security/classifier/bakeoff.yaml` | pending | per-detector ROC/PR, fixed-FPR, OOD, latency, memory, and cost report |
| Secret boundary | `cd codex-rs && just test -p codex-vault -p codex-core` | pending | secret-flow and redaction results |
| Executor policy | `cd codex-rs && just test -p codex-security-policy` | pending; crate is planned by active P0 plan | typed-action, approval-binding, replay, and forced-miss results |
| Web broker compatibility and routing | `cd codex-rs && just test -p codex-web-search-extension && just test -p codex-core web_search` | pending | existing `web.run` schema, adapter selection, native-bypass denial, query minimization, stable result ids, normalization, failover, and profile tests |
| Retriever and egress | `cd codex-rs && just test -p codex-network-proxy` | pending | URL, DNS, redirect, rebinding, container, and no-fallback results |
| Agent Sweep | `cd codex-rs && just test -p codex-security-policy agent_sweep` | pending | anomaly, authority, pause, revocation, and false-alarm results |
| Adversarial system suite | `python3 scripts/security/adversarial.py --profile moderate --profile aggressive --force-classifier-misses` | pending | redacted case-level outcomes and canary report |
| Formatting | `cd codex-rs && just fmt -- --check` | pending | formatting output |

## True-TUI evidence

Launch through the repository TUI workflow with trace logging and an isolated
`log_dir`. Send prompt text and Enter separately. Corbanu `exec`, isolated unit
tests, and screenshots without key-driven interaction are not acceptable proof.

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Moderate poisoned research | pending | TensorCash disposable worktree | `/security`, choose Moderate, confirm; request hostile public fixture; attempt protected follow-up | Level, source label, isolated route, classifier outcome, denial/receipt | pending | `qa/release/<version>/security/tui/poisoned-research/` |
| Provider routing and failover | pending | TensorCash disposable worktree | Configure two qualified search adapters; choose Moderate; submit query; make the first adapter unavailable; repeat; open a result | Same `web.run` flow; provider and failover visible; result id survives; page uses isolated fetch | pending | `qa/release/<version>/security/tui/web-provider-routing/` |
| Query privacy | pending | TensorCash disposable worktree | Put fake protected portfolio context in chat; issue a public query; inspect redacted route evidence | Provider and digest visible; protected context absent from captured provider request and logs | pending | `qa/release/<version>/security/tui/web-query-privacy/` |
| Aggressive browser denial | pending | Isometric Game disposable worktree | `/security`, choose Aggressive, confirm; request host-browser eval; choose isolated alternative | Direct route denied; safe alternative visible; no silent fallback | pending | `qa/release/<version>/security/tui/browser-isolation/` |
| Quarantine recovery | pending | TensorCash disposable worktree | Retrieve hard-negative fixture; inspect summary; skip, retry, and use unprivileged review path | Raw text stays outside agent; every recovery choice is understandable | pending | `qa/release/<version>/security/tui/quarantine/` |
| Rogue-agent pause | pending | TensorCash disposable worktree | Run fake venue scenario with unexplained concentration and repeated denied calls | Sweep evidence, paused state, revoke/kill controls, human resume | pending | `qa/release/<version>/security/tui/agent-sweep/` |
| Restart and child inheritance | pending | Isometric Game disposable worktree | Quarantine source, revoke grant, spawn child, restart, resume | Child cannot weaken policy; state restores before work continues | pending | `qa/release/<version>/security/tui/recovery/` |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | yes | pending | pending | Financial protected actions, hostile market inputs, canary credentials, Agent Sweep, restart, and recovery |
| Isometric Game | yes | pending | pending | Browser isolation, visual/HTML hostile inputs, child inheritance, false-positive UX, and recovery |

Both repositories are disposable qualification targets. Chaotic edits are
allowed only in resolved test worktrees, never in the user's primary checkout.

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Named by release owner | pending | pending | Understand source trust, quarantine, brokered action, browser denial, Agent Sweep, recovery, and all three profiles without explanation | pending | `qa/release/<version>/security/human-acceptance.md` |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/features/security.md`, updated only for accepted shipped behavior | Must cite `P0 /security levels` | pending |
| Security architecture reference, created only after implementation is stable | Must distinguish deterministic boundaries from probabilistic detectors | pending |

Until then, all unfinished detail remains in this proposed plan.

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| Reconcile with active P0 plan | Product decision | Product authority | Before activation | Pending; overlapping worktrees are prohibited |
| Exact implementation worktree and base | Engineering | Delivery owner | Activation | Not allocated |
| Permissive golden behavior | Evidence | Delivery owner | Stage 1 | Must be frozen before any policy composition |
| Browser stance for authenticated/interactive flows | Product and security decision | Product authority | Stage 1 | Isolated retrieval is clear; human handoff design remains open |
| `web.run` compatibility boundary | Engineering contract | Delivery owner | Stage 1 | Preserve the current schema and history/events unless an explicit product decision changes them |
| Provider registry and ordered routing configuration | Product and engineering design | Delivery owner | Stage 4 | Must define roles, credential refs, origins, privacy, budgets, health, and per-profile eligibility |
| Provider-native search interception | Security blocker | Delivery owner | Stage 4 | Moderate/Aggressive must disable any native path whose output cannot be screened locally before main-model ingestion |
| “Seer” provider identity | Dependency clarification | Product authority | Provider bakeoff | Not identified in reviewed Ambient snapshot |
| Corpus licenses and privacy review | Legal/security | Release owner | Before training | Pending |
| CPU reference hardware and thresholds | Product/performance decision | Delivery owner | Before bakeoff | Candidate gates proposed above |
| Hosted-vendor data policy and cost | Buy/build decision | Product authority | Before premium integration | No vendor selected |
| Independent security reviewer | Release gate | Release owner | Final qualification | Must be named |
| Human tester | Release gate | Release owner | Final qualification | Must be named |

## Release linkage

- Release record: `qa/release/<version>/` — no version assigned.
- Benchmark tracker row: repository-root `benchmarks/README.md`, when due for the
  activated release.
- Remaining blocker: proposal activation, active-plan reconciliation, worktree
  allocation, implementation, independent review, true-TUI evidence, live-repo
  evidence, and human acceptance.

## Completion

- [ ] Product authority activates the outcome and reconciles it with the active
  P0 plan.
- [ ] Exact implementation worktree, branch, base commit, and non-overlapping
  scope are recorded.
- [ ] All 72 linked sprint records are completed with final-tree evidence and
  moved out of current docs into the sprint archive.
- [ ] PF-01 through PF-12 have accepted implementation and evidence mappings.
- [ ] Permissive compatibility is proven.
- [ ] Classifier fixed-FPR, OOD, surface-heuristic, CPU, memory, privacy, and cost
  gates pass on blind holdouts.
- [ ] Forced classifier misses cannot disclose secrets, weaken policy, or execute
  unauthorized protected actions.
- [ ] Moderate and Aggressive never silently fall back to direct browser access,
  unscreened content, or unmediated sensitive tools.
- [ ] Existing `web.run` workflows remain compatible while search and fetch use
  separate qualified provider roles behind the broker.
- [ ] Provider routing, same-role failover, native-search bypass denial, stable
  result ids, query-context minimization, normalized provenance, and
  classifier-unavailable behavior pass for every qualified adapter.
- [ ] Final-tree automated, adversarial, true-TUI, and live-repository evidence
  passes with no critical finding open.
- [ ] Named human acceptance passes.
- [ ] Finished documentation matches only the accepted candidate.
- [ ] Release and due benchmark records are linked.
