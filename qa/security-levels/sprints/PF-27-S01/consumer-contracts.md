# PF-27 consumer handoff

This is an internal contract, not shipped user guidance or backend qualification.
Product authority: **P0 `/security` levels** — “Permissive preserves the shipping
behavior and does not silently change existing policies.” The active P0 plan
owns scope; each consumer still needs its own executable sprint.

## One authority and lifecycle

Dependency direction is Core/TUI adapters → protocol → security-policy. The only
new Cargo edge is protocol → the existing security-policy crate. There is no
new third-party dependency, browser installation, agent scheduler, persisted
security format, or provider tool schema. Core retains its existing policy lock,
trusted controller, actor bindings, and revocation ledger. Native agent creation,
cancellation, compaction, and resume remain upstream-owned.

- Inspector facts distinguish committed requested level, effective inherited
  floor, and four independent control-health values. Every unimplemented control
  starts `Unavailable`; no consumer may infer protection from the selected level.
- Wire requests and snapshots validate shape, not sender authority. Never accept
  model/tool snapshots as host observations. Requests contain no human credential
  or confirmation flag. Only the trusted Core controller mints a non-serializable,
  single-consumption confirmation; neither receipt nor consumption mutates policy.
- PF-23/25 must recheck the expected epoch atomically with mutation/dispatch.
  `consume_security_confirmation` returns intent, **not an execution permit**.
  Recheck native identity, current taint, expiry, revocation, and existing denials
  at the actual operation. A grant match cannot override an existing deny.
- Authority epochs bind a fresh per-runtime nonce, policy revision, and existing
  revocation generation. New runtime/resume invalidates pending authority even if
  stored counters match; idempotent initialization within one runtime does not.
- Only host adapters issue source identity/classification. Source envelopes bind
  exact bytes by digest and have no Deserialize implementation. They do not store
  raw content or credentials. Sanitized text remains derived from the old source.
- Taint derivation is a sticky union, bounded at 64 identities; overflow and
  unverifiable ancestry become unknown. Unknown origin denies protected use.
  `trusted_input()` is exclusively for host-verified direct human input, not text
  saying it is human. Taint deserialization is only for authenticated host-owned
  checkpoints. A valid JSON shape, empty source list, or stored grant is not proof
  of trust; unauthenticated/legacy checkpoints restore unknown. PF-29 owns this
  storage boundary and every native summary/memory/delegation/resume join.
- `EpochBoundGrant::bind` is a trusted issuance seam, not a way to refresh old or
  deserialized grants. A restarted runtime must obtain fresh authorization.
- Raw serde/backend errors are not safe display/audit text. PF-28 must scrub
  protected data across all output paths; contract errors use fixed categories.
- New stronger-control consumers run only for Moderate/Aggressive and compose
  restrictively. PF-27 does not activate any new restriction in Permissive.

## Reserved ownership

Paths below are relative to `codex-rs/`. Placeholder modules are empty ownership
reservations, not implementations. PF-27 owns their shared registration lines;
consumers edit their reserved files after dependency/archive gates pass.

| Consumer | Reserved product-owned files | Contract and native handoff |
| --- | --- | --- |
| PF-23-S01 | `core/src/security/protected_surface.rs` | Post-read action binding and existing deny composition at native tool dispatch |
| PF-23-S02 | `core/src/security/aggressive.rs` | Aggressive restrictions using the same policy/epoch, no second evaluator |
| PF-23-S03 | `core/src/security/{transition,recovery}.rs` | Atomic transitions/revocation, cancel/recovery/resume, fresh authority |
| PF-24-S01 | `tui/src/security/view.rs`, `tui/src/bottom_pane/security_view.rs` | Render independent inspector facts; use observational state in `security/mod.rs` |
| PF-24-S02 | `core/src/security/ui_events.rs` | Trusted Core observations into the native TUI event flow |
| PF-25 | Its sprint must allocate trusted confirmation handlers before readiness | Reuse protocol intents and Core controller; never deserialize confirmation capability |
| PF-28 | `core/src/security/confidentiality/` | Safe output/diagnostics; separate confidentiality-health producer |
| PF-29-S01 | `core/src/security/ingress/` | Source issuance at file/web/tool/MCP/connector ingress; native hook list in plan |
| PF-29-S02 | `core/src/security/taint/` | Sticky lineage at native summary, compaction, memory, child and persistence joins |
| PF-30 | `core/src/security/browser_isolation/`, `network-proxy/src/browser_policy.rs` | Browser containment/egress adapter and independent browser-health producer |
| PF-26-S01 | Its fixture/harness scope | Consume the catalogue below; no writes to native policy or lifecycle owners |

All further shared manifests, lockfiles, provider/tool registrations, and common
Core/TUI files require explicit serial allocation before editing. PF-27 cannot
reserve an unknown backend dependency in advance. PF-30's selected installation
and version-lock contract remains its plan's responsibility. PF-14 is proposed,
not activated; it must reuse native subagents and these contracts if approved.

## Conformance and upstream disposition

[adapter-conformance.json](adapter-conformance.json) gives synthetic inputs,
required assertions, existing contract-test selectors, and downstream owners.
PF-27 proves the shared primitives and thin adapter seams. The catalogue does
**not** mark native dispatch/ingress/browser/UI integrations complete. PF-26-S01
turns these cases into its runner; consumers provide actual native-hook evidence,
then PF-26-S04 audits the integrated upstream patch inventory.

Retain policy contracts and product-owned modules across upgrades; adapt only
thin native hooks. Retain native operation/provider schemas without adding a
security model tool. Adapt the existing Core policy snapshot/confirmation to add
runtime-incarnation binding. Shared module exports and the Cargo edge are small
retained registration patches. No upstream version is upgraded by this sprint.
Rerun protocol, policy, Core security/inheritance, TUI state and network suites
against each candidate; structural plan checks cannot qualify an upstream rebase.
