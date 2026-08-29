# OpenClaw security source review — 2026-08-28

This review grounds Corbanu's security sprints in specific implementations, callers,
defaults, bypasses and tests. It is planning evidence, not a certification of
OpenClaw or Corbanu.

## Snapshot and scope

- Origin: [openclaw/openclaw](https://github.com/openclaw/openclaw).
- Default branch at download: `main`; exact revision:
  `13adff02ca3897768d80d2bca18f5acf08c55d91`.
- Commit: `fix(openshell): serialize symlinked mirror workspaces (#131917)`;
  committed `2026-08-28T17:06:08-04:00`; package version `2026.8.1`.
- Comparison baseline: `6ce272c2a662f81b7779507335d91de4d61c589b`, the
  historical SecurityComparativeAnalysis reference. Historical citations stay
  historical; the current adoption reference is the new revision.
- Downloaded a shallow source checkout, not an installed/running Gateway.
  No provider credentials, live financial accounts or system CA changes were used.
- [MIT license](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/LICENSE): preserve applicable copyright/license notices
  for future adaptation. This update copies no upstream implementation into Corbanu.
- Change class: routine planning/evidence refinement of the already-authorized
  active plan. Product citation: **Non-negotiable controls** — “Permit agents to
  reference credentials only by label; resolve them solely inside the trusted
  execution boundary.” **Reconciled security scope — TO BUILD** owns the wider
  feature set. No feature, dependency, readiness status or release gate changes.

The review is **targeted, not repository-wide**. Complete small security primitives
and the named portions of larger call chains were read directly. Tests below are
separated into executed, inspected and follow-up coverage. Native platform
containment, all plugins/providers, full approval flows, browser navigation/CDP
integration, financial integrations and all upstream suites were not qualified.
A path appearing in a search result is not counted as an audited implementation.

The [upstream trust model](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/SECURITY.md#L126) treats the Gateway,
installed plugins and host environment as trusted operator infrastructure; it
does not promise hostile multi-tenant isolation. Corbanu's Moderate/Aggressive
broker and secretless-process requirements are stronger. The differences below
are adoption constraints, not blanket vulnerability claims about OpenClaw.

## Changes from the earlier source map

| Area | Verified update |
| --- | --- |
| Egress proxy | New optional traffic allowlist, with per-run secret-bound hosts and blind-tunnel hosts also admitted; omission still permits non-secret traffic. Token validation now uses canonical random-token bytes. |
| External-content bounds | Truncation preserves complete sanitized markers/useful text before a later clipped marker; new regression cases cover this. |
| Guarded fetch | Release now aborts the request and cancels unread bodies before releasing a pooled dispatcher; caller `init.signal` is retained when no explicit signal is supplied. |
| Secret migration/runtime | Apply records committed auth ownership before runtime publication; snapshot activation checks credential revisions and rollback ownership. |
| Taint/memory | The earlier map was incomplete: persistent memory provenance and transcript-to-memory-flush taint exist. The tiny turn helper alone does not describe the full system. |

These are selected semantic differences or coverage corrections, not an exhaustive
change log. The sentinel, exact-redaction registry and small turn-state helper are
unchanged between the two pins.

<a id="oc-1"></a>

## OC-1 — Secret references and provider handoff

Source: [src/secrets/sentinel.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/sentinel.ts) (`mintSecretSentinel`,
`resolveSecretSentinel`), [src/agents/provider-secret-egress.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/provider-secret-egress.ts),
and [provider stream handoff](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/provider-stream.ts#L80).

The sentinel uses authenticated process-local encryption; minting also registers
the raw value for redaction. Resolution is a callable same-process operation,
not OS-peer authorization. `OPENCLAW_SECRET_SENTINELS=off/0/false` returns the raw
value. Provider handoff unwraps API keys, headers and symbol-attached request
transport metadata before invoking the provider transport.

Reuse: opaque references, tamper rejection and a single final transport handoff.
Do not transplant the unrestricted resolver, raw opt-out or in-process key into
agent-accessible Corbanu code. Inventory hidden transport metadata and plugin/SDK
handoffs as well as obvious HTTP headers.

Evidence: sentinel test cases inspected; synthetic round-trip, tamper and raw
opt-out probes executed. Full provider/SDK suites not run.
Owners: PF-13-S02/S03, PF-27-S04/S02, PF-29-S01.

<a id="oc-2"></a>

## OC-2 — Proxy authorization, lifecycle and reflected responses

Source: [src/secrets/egress-proxy/proxy-server.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/egress-proxy/proxy-server.ts),
[src/secrets/egress-proxy/stream-substitution.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/egress-proxy/stream-substitution.ts),
[src/secrets/egress-proxy/registry.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/egress-proxy/registry.ts),
[src/secrets/egress-proxy/runtime.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/egress-proxy/runtime.ts).
Callers: [Gateway startup](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/gateway/server-core-runtime.ts#L159),
[Gateway exec environment](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/bash-tools.exec-run.ts#L435),
[authority-close callback](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/gateway/server-core-runtime.ts#L325).
Contract: [proxy limitations](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/docs/gateway/secrets.md#L313).

- Enabled only when configured true. Gateway-hosted exec receives proxy variables
  and sentinels; sandbox/node and provider-native harness subprocesses do not use
  this proxy. The Gateway owns the proxy and its decryption process.
- Authentication binds a random token to run/instance; substitution also checks
  that run's sentinel and exact normalized hostname. The traffic allowlist is a
  different control from each secret's binding.
- Binding does not include method/path/port or validated DNS peer. The parser
  accepts ports 1–65535 even though non-443 substitution is not a documented
  compatibility target. Do not infer a port refusal from an unused reason enum.
- `bypassHosts` uses blind CONNECT. Proxy variables are cooperative routing,
  not OS-enforced outbound containment.
- Upstream response headers/body are forwarded without this proxy scrubbing
  reflected credentials. This limitation is explicitly documented upstream.
- Stream substitution keeps a bounded carry and rejects unresolved references,
  but can emit an ordinary prefix before a later reference fails. That is not
  all-or-nothing request authorization.

**Open-channel revocation is an unexecuted source-level concern.**
`revokeRun` deletes the token-map entry; an accepted CONNECT/TLS handler retains
its `RegisteredRun` object. The inner forwarding path does not recheck membership.
The inspected [revocation test](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/egress-proxy/proxy-server.test.ts#L552)
opens a *new* CONNECT after revocation. It does not prove established-channel,
in-flight-body or same-ID re-registration revocation. No live proxy exploit or
upstream security report was performed in this review.

Corbanu acceptance must cover revoking an already-open keep-alive/TLS channel,
revoking during body upload, run replacement/re-registration, and cancellation
while unrelated runs continue. A rejected new connection alone is insufficient.
No unauthorized side effect may rely on discovering a malformed reference late
in a stream. Full effect authorization belongs before dispatch.

Evidence: proxy implementation and selected test bodies inspected; split-sentinel
and late-refusal stream probes executed. Proxy network tests not executed.
Owners: PF-13-S03/S04/S05, PF-27-S04, PF-28-S02, PF-33-S01/S02.

<a id="oc-3"></a>

## OC-3 — Exact redaction is not a central output gate

Source: [src/logging/secret-redaction-registry.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/logging/secret-redaction-registry.ts),
[maskToken](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/logging/redact.ts#L231) and
[redactSensitiveText](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/logging/redact.ts#L919).

The registry stores raw, percent-encoded and JSON-escaped forms in a module-local
map, ignores values shorter than six characters, and evicts above 512 entries
(representations, not necessarily 512 secrets). It matches whole supplied strings;
separate calls have no chunk carry. The diagnostic `maskToken` preserves a prefix
and suffix for values of at least 18 characters. Exact registration still runs
when general logging redaction is off.

Reuse: exact/encoded matching and independent exact-value registration.
Corbanu requires broker-owned raw state, complete secret removal at protected
sinks, short-value handling, encoding/chunk tests, and capacity exhaustion that
does not silently unprotect live credentials. Logs, model input, transcript,
clipboard, exports and support bundles need explicit sink coverage.

Evidence: registry matching and limits are exercised by the synthetic probes; logging
mask behavior is source-inspected, not an executed whole-logging assertion.
Owners: PF-28-S01/S02, PF-39-S02, PF-41-S02.

<a id="oc-4"></a>

## OC-4 — Wrappers, metadata and unsafe hook bypass

Source: [src/security/external-content.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/security/external-content.ts) and
[src/security/external-content-source.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/security/external-content-source.ts).
Caller: [hook preparation](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/cron/isolated-agent/run-prepare.ts#L506).

`wrapExternalContent` adds randomized boundaries; sanitization neutralizes forged
boundaries, selected Unicode variants and model special tokens. Metadata is
sanitized and newline-folded. `truncateSanitizedExternalContent` bounds expansion
and preserves the retained raw-prefix count. `detectSuspiciousPatterns` is a
heuristic signal, not a deny/classifier decision. The hook caller logs detections
and can omit wrapping through explicit `allowUnsafeExternalContent` settings.

Reuse: wrapper/metadata/Unicode/truncation fixtures, including a complete marker
before a clipped one. Corbanu assigns immutable source types at ingress, keeps
host-created authorization notices distinct from external text, and must not let
external role labels or “safe” classifier output mint authority. No hook bypass
may silently disable required protected-mode screening.

Evidence: all 85 tests in [src/security/external-content.test.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/security/external-content.test.ts) executed
under the minimal reference harness. This is not proof that every ingress uses
the helper. Owners: PF-30-S01/S03, PF-34-S01, PF-35-S01/S03.

<a id="oc-5"></a>

## OC-5 — Turn taint and maintenance inheritance

Source: [src/agents/embedded-agent-runner/run/turn-taint-state.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/embedded-agent-runner/run/turn-taint-state.ts).
Callers: [run-loop observer](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/embedded-agent-runner/run-loop.ts#L225),
[CLI transcript metadata](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/embedded-agent-runner/cli-backend-dispatch-transcript.ts#L109),
[active-turn scan](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/auto-reply/reply/agent-runner-memory.ts#L477) and
[memory-flush initial taint](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/auto-reply/reply/agent-runner-memory.ts#L1533).

Non-presentation network results make the helper sticky through retries in that
run. CLI transcript records carry network/tainted metadata. Memory maintenance
reads active-turn transcript metadata and seeds initial taint; read errors or
insufficient bounded history can conservatively taint it. The scan stops at a
user-message boundary. A newly created helper defaults clean unless seeded.

Reuse those handoffs, not a claim that “OpenClaw has no persistence.”
Corbanu still needs ancestry union across turns, compaction, derived summaries,
cache, imports, child messages and restart. A user message or one-action approval
must not erase already-derived data's provenance.

Evidence: two turn-state tests and a fresh-helper probe executed; transcript and
maintenance callers source-inspected only. Owners: PF-30-S02/S03; see OC-11.

<a id="oc-6"></a>

## OC-6 — Inventory, apply and stale-state ownership

Source: [runSecretsAudit](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/audit.ts#L636),
[preflight/apply/restore](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/apply.ts#L734),
[snapshot activation and rollback](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/secrets/runtime-state.ts#L949).

Audit enumerates config, auth stores, model configuration, dotenv and legacy-source
findings. Exec-provider resolution is skipped without consent and reports whether
resolution was complete. Apply defaults to dry-run, validates projected state,
requires exec consent, uses atomic individual writes and best-effort restoration;
auth writes carry ownership into runtime publication/rollback. Runtime activation
checks both snapshot and credential revisions. It is not a durable encrypted
multi-file crash transaction. Upstream deliberately avoids plaintext backup files.

Corbanu should reuse completeness flags, consent, compare-and-activate and
ownership-aware rollback. Inventory cannot report ready after skipped required
checks. Migration needs encrypted recovery, crash/power-loss cases, stale preview
rejection and preservation of unrelated concurrent credential updates. Never
equate “atomic file replace” with whole-migration atomicity.

Evidence: key audit/apply/runtime function ranges and selected apply tests
(preflight/no-backup and publication-failure rollback) inspected. Audit/apply/
runtime suites not executed.
Owners: PF-29-S01/S02, PF-20-S02, PF-41-S01.

<a id="oc-7"></a>

## OC-7 — Effective explanation versus observed health

Source: [sandboxExplainCommand](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/commands/sandbox-explain.ts#L147),
[resolveSandboxRuntimeStatus](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/runtime-status.ts#L77).

The command resolves session/agent policy and inherited paths, not just a global
configuration flag. Runtime classification also reads immutable creator-required
sandbox state. This is useful inspector plumbing, but a policy classification is
not a live attestation of engine, broker, mounts, DNS or audit health.

Owners: PF-41-S01/S02. Show configured, resolved and observed states separately;
stale/unavailable probes must not produce a green protected-mode claim.
The command's policy-resolution path was inspected; its complete output/platform
integration and test suites were not run.

<a id="oc-8"></a>

## OC-8 — Sandbox, tool policy and browser plane

Source: [Docker/browser defaults](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/config.ts#L80),
[create arguments](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/docker.ts#L329),
[required sandbox session](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/context.ts#L154),
[src/agents/sandbox/runtime-status.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/sandbox/runtime-status.ts),
[browser SSRF policy](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/extensions/browser/src/browser/config.ts#L241),
[extensions/browser/src/browser/ssrf-policy-helpers.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/extensions/browser/src/browser/ssrf-policy-helpers.ts).
Contract: [docs/gateway/sandbox-vs-tool-policy-vs-elevated.md](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/docs/gateway/sandbox-vs-tool-policy-vs-elevated.md).

Ordinary sandbox mode defaults off; configured Docker defaults include read-only
root, network none and dropped capabilities. Browser networking/binds have
separate defaults and override handling. Host-control and browser sandbox
enablement are separate switches. Creator-required sandbox sessions override
mode off, cap writable shared workspace exposure and require a creator principal.
Do not describe every upstream sandbox failure as a host fallback.

Browser SSRF policy defaults to an explicit strict object; operator private-network
exceptions and CDP-control exceptions need separate treatment from public content.
Container names, healthy engines and denied tool names alone do not establish
containment. Corbanu must verify the actual image digest, user, mounts, resource/
network policy, ownership and recovery path; no host/elevated fallback above
Permissive. Public retrieval and credentialed login remain separate planes.

Evidence: relevant resolution/creation paths inspected; no container/browser,
CDP or three-OS qualification run. Owners: PF-27-S02, PF-31-S01/S02/S03,
PF-37-S01/S02, PF-22-S02.

<a id="oc-9"></a>

## OC-9 — SSRF, redirects, DNS pinning and transport release

Source: [policy and pinning](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/infra/net/ssrf.ts#L223),
[redirect and dispatch loop](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/infra/net/fetch-guard.ts#L376),
[src/agents/tools/web-guarded-fetch.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/tools/web-guarded-fetch.ts).

The direct strict path validates host/IP plus every DNS answer, creates a pinned
lookup/dispatcher and re-evaluates redirects. Cross-origin redirects strip
sensitive headers and normally remove unsafe bodies; explicit authorization
retention and unsafe-replay options exist. Trusted environment/explicit/managed
proxy paths may delegate final DNS resolution; `pinDns:false` does not bind the
validated lookup to the connection. A mocked fetch can intentionally skip DNS.

`hostnameAllowlist` restricts hosts; `allowedHostnames` can grant private-address
trust. They are not interchangeable. Exact-origin exceptions are reconsidered at
each hop. The self-hosted web wrapper explicitly permits private networks.
Corbanu's self-hosted adapter exception must not leak into public retrieval.

Reuse explicit hop ownership and release cancellation. Add real loopback-network
tests (not only mocked fetch), mixed DNS answers, mapped IPv6, changed ports,
redirect credential/body policy, authority-aware connection pooling, abort after
headers, unread/error bodies and sibling-request preservation. Protected policy
must enforce actual outbound routes even when a subprocess ignores proxy env.

Evidence: policy/dispatcher and redirect-loop ranges inspected, latest
`fetch-guard.release.test.ts` lifecycle fixture inspected; network suites not run.
Owners: PF-33-S01/S02, PF-31-S02, PF-32-S05/S06.

<a id="oc-10"></a>

## OC-10 — Fetch output, metadata, cache and spill

Source: [output bounds and spill](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/tools/web-fetch.ts#L358),
[output construction and cache](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/tools/web-fetch.ts#L558),
[guarded fetch and release](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/tools/web-fetch.ts#L753).

The fetch path marks results as network-sourced, wraps page prose and metadata,
budgets sanitizer expansion, separates protocol fields, partitions cache by
request configuration and supports overflow spill. Provider fallback, cached
content, errors and spilled artifacts are additional ingress routes, not exemptions.
Corbanu must retain provenance and screening on each, cap decompressed/decoded
content as well as transport bytes, and put raw downloads/spills through the
quarantine/promotion contract. No ordinary workspace file may become an unscreened
spill route.

Evidence: targeted output/cache/fetch/error/finally sections inspected.
`web-fetch.output-contract.test.ts` was identified as an implementation-time
test reference, not executed here.
Owners: PF-31-S02/S03, PF-32-S01/S02/S06, PF-34-S01/S02/S03.

<a id="oc-11"></a>

## OC-11 — Persistent memory provenance is a real reuse candidate

Source: [memory origin observer](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/agent-tools.ts#L527),
[src/agents/memory-write-provenance.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/memory-write-provenance.ts),
[src/memory/memory-artifact-provenance.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/memory/memory-artifact-provenance.ts),
[extensions/memory-core/src/memory/memory-path-provenance.ts](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/extensions/memory-core/src/memory/memory-path-provenance.ts).

Tool construction derives memory-write origin from sender ownership/current taint.
The store uses canonical workspace/path identity, content hashes, sticky
least-trusted origin, reservation-owned rollback and reject-new capacity behavior.
The write observer records provenance before mutation and rolls it back on failed
writes. Memory indexing consumes stored classification. This materially improves
the adoption plan beyond the old turn-helper citation.

Limits: coverage is selected workspace Markdown paths, not arbitrary source
envelopes; unrecorded workspace memory defaults to agent-origin and dreaming paths
receive system classification under upstream's trusted-workspace model.
Corbanu cannot use missing records, filenames or model-generated summaries to
promote authority. Bind provenance to content identity at read time, retain
ancestry and reject/quarantine missing or corrupt protected-mode lineage.

Evidence: writer/store/index-classification modules and all five artifact-store
test bodies inspected; persistent-store suites not executed.
Owners: PF-30-S02/S03, PF-34-S02, PF-40-S01.

## Adoption and upstream-upgrade discipline

Use this pin as a design/fixture reference, not a new runtime dependency on
OpenClaw. Keep Corbanu policy/broker/provenance/retriever logic in its designated
modules; keep Codex integration hooks small and explicit. For each adaptation,
record upstream file/function + commit + license, Corbanu owner + hook, adapted
tests, deliberate differences and final-tree evidence. Upstream changes require
semantic reinspection, not mechanically replacing the hash.

PF-35/36 classifiers, PF-37 exact-origin login, PF-38/39 financial/disclosure
contracts, PF-40 advisory Sweep and PF-41 tamper-evident audit remain Corbanu
requirements. The inspected OpenClaw code does not establish classifier quality,
secure arbitrary-model review, financial safety or complete audit integrity.
Do not mark these features implemented merely because a nearby upstream utility
exists. OpenClaw's `sessions_spawn` naming is not evidence about Codex's native
subagent protocol or deprecation status; the separate Autoreview plan is unchanged.

## Evidence and remaining qualification

Reproduction record and machine-readable outputs:
`qa/security-levels/planning/openclaw-2026-08-28/README.md`.
87 upstream helper tests passed (85 external-content, 2 turn-state), plus 10
Corbanu-authored synthetic observation probes. Node 24.15.0, Vitest 4.1.11,
Vite 8.2.2 and tsx 4.23.12; isolated minimal runner, workspace source aliases,
no upstream global test setup. Passing an observation probe may confirm a
limitation; it does not mean that limitation meets Corbanu's security contract.

Pending: full upstream integration suites, established-tunnel reproduction,
native container/browser/OS proof, each Corbanu regression port, final candidate
tests/TUI/live repositories and named human acceptance. PF-26 owns cross-feature
qualification. No old evidence was relabeled, and no sprint was completed or
activated by this review.
