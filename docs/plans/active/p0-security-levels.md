---
title: "P0 /security levels"
status: active
change_class: product-initiative
priority: P0
owner: "Jim Ricketts"
max_active_sprints: 3
integration_owner: "Jim Ricketts"
activation_authority: "Product authority defined in the product specification"
activation_basis: "Accountable sequencing item 1 defines /security as P0 and immediate."
target_release: "TBD — candidate qualified by 2026-10-08"
deadline: 2026-10-08
created: 2026-08-23
updated: 2026-08-27
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "P0 /security levels"
  requirement_excerpt: "Permissive preserves the shipping behavior and does not silently change existing policies."
implementation_worktrees:
  - path: "/home/pfrpc/repos/CorbanuTerminal-pf13-s02"
    branch: "feat/pf-13-s02-scoped-vault-resolver"
    base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02"
    branch: "feat/pf-13-s02-scoped-vault-resolver"
    base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01"
    branch: "codex/pf-27-shared-security-contracts"
    base_commit: "ea7d4bec720098f6e0994fcfcc59e272108f7e70"
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-pf26-s01"
    branch: "codex/pf-26-security-harnesses"
    base_commit: "cb808c30c0058c101597ab2ada3da16238565c5e"
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01"
    branch: "codex/pf-30-isolated-runtime"
    base_commit: "9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c"
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-pf29-s01"
    branch: "codex/pf-29-untrusted-ingress"
    base_commit: "9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c"
---

# P0 `/security` levels

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | **Active** |
| Active-plan slot | **1 of 2** |
| Product authority | Defined once in the product specification |
| Authoritative decision | “Accountable sequencing,” item 1: `/security` is P0 and begins immediately |
| Delivery owner | Jim Ricketts |
| Deadline | **2026-10-08** |
| Scope amendment | Travis Good, 2026-08-27: preserve Permissive, adopt stronger guarantees only for Moderate/Aggressive, separately scope browser isolation, and permit bounded independent sprint concurrency |
| Integration amendment | Travis Good, 2026-08-27: require upstream separation and compatibility evidence, explicitly schedule the browser lane in parallel, and tighten injection/task-integrity coverage; no new runtime sprint activated by this documentation update |
| PF-27 activation | Travis Good, 2026-08-27: “Please start work on PF-27”; isolated contracts lane allocated below, leaving PF-13 qualification unchanged |
| PF-27 completion | Travis Good, 2026-08-27: “Got it, please finish PF-27”; full shared-contract sprint completed with evidence below; no consumer activation or release authorization |
| PF-26 activation | Travis Good, 2026-08-27: ensure PF-27 is pushed, then complete PF-26; activate dependency-ready S01 in the isolated harness lane. S04/S02/S03 remain gated by their recorded dependencies and human acceptance. |
| Browser/content activation | Travis Good, 2026-08-27: approve pushing PF-26-S01 and starting PF-30/PF-29 in parallel; allocate the two worktrees below. PF-13 qualification retains the third slot. |
| Runtime and acceptance decision | Travis Good, 2026-08-27: reuse Podman/Docker; prefer Podman for new installs; guide setup and recover only owned Scrapling services in stronger modes; require all three OSes, Mac/Linux first. Human tester is Travis Good (acceptance pending). |
| S01 implementation and reviewer | Travis Good, 2026-08-27: implement PF-30-S01 end to end; reviewer is Fable High (`claude-fable-5`, high effort). No fallback reviewer; Windows instructions follow Mac/Linux qualification. |

The scope/integration amendments approved product-initiative planning plus routine
process/validator work; the later PF-27 activation starts only its allocated sprint,
not release acceptance. The design
input is [Security Comparative Analysis](https://github.com/CorbanuCore/CorbanuTerminal/blob/549c18f0b63b8e5c4fedf60b18932d1d48adb56f/research/2026-08-23-product-security-session/SecurityComparativeAnalysis.html),
dated 2026-08-23; its comparative scores are not certification evidence.

## User pain

Corbanu already has approvals, sandboxing, a vault, scoped wallet actions, and
tool permissions. Those controls are spread across the product. A user cannot
answer one simple question: “How locked down is my agent right now?”

Security settings should be understandable without learning internal policy
types. They should also be optional: introducing the feature must not silently
change the behavior of an existing Corbanu installation.

## Product intent and ideal flow

The user types `/security` and sees one focused tab with three choices:

| Level | User promise |
| --- | --- |
| **Permissive** | Corbanu behaves exactly as it does today. Existing policies are not changed. |
| **Moderate** | Corbanu adds strong protection around untrusted content, secrets, sensitive data, credentials, and protected actions while preserving normal agent workflows. |
| **Aggressive** | Sensitive access is denied by default. The user opens narrow, expiring permissions explicitly, and every sign or broadcast requires exact human approval. |

The current level is obvious. Moving to another level shows the policy
difference before confirmation. `Esc` cancels without changing anything.

A confirmed change takes effect immediately for the current session and child
agents, persists across restart, invalidates incompatible pending approvals,
and creates a secret-free audit event. Only the human can change or downgrade
the level.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | **P0 `/security` levels** |
| Plan requirement excerpt | “Permissive preserves the shipping behavior and does not silently change existing policies.” |
| PF-13 trust-boundary heading | **Required trust boundaries** |
| PF-13 requirement excerpt | “Credentials are referenced by label and resolved only inside a trusted execution boundary.” |
| Expanded feature heading | **Moderate/Aggressive isolation and content provenance** |
| Expanded requirement excerpt | “Browser isolation is a separately scoped feature within the security initiative.” |
| Product outcome advanced | One understandable control for agent security posture |
| North-star criterion advanced | External content cannot silently gain sensitive access or change security policy |

## Scope

### In

- Add `/security` as a first-class slash command and TUI tab.
- Define a standards-derived authorization contract for protected actions,
  grants, delegation, revocation, and adversarial coverage.
- Define one persisted `SecurityLevel` value: Permissive, Moderate, or
  Aggressive.
- Default existing and upgraded installations to Permissive.
- Prove Permissive uses the current policy resolution paths without adding,
  removing, or rewriting a policy.
- Make Moderate enforce the product-spec controls for untrusted content,
  model-visible secrets and protected data, trusted credential resolution,
  protected-action previews and approval binding, redacted audit events,
  revocation, and kill switch.
- Make Aggressive include Moderate and then default sensitive tools, account
  access, credential use, protected-data disclosure, financial actions,
  arbitrary egress, and clipboard/export paths to denied.
- In Aggressive, allow only narrow human grants with visible scope and expiry;
  require exact human approval for every sign or broadcast.
- Make active and newly spawned agents inherit the selected level and prevent
  agents, tools, project content, hooks, plugins, connectors, and MCP servers
  from weakening it.
- Show the current level and a concise protection summary in the tab and
  session status.
- Require explicit human confirmation for a downgrade; cancel or restart must
  not apply an unconfirmed change.
- Add compatibility, policy, adversarial, snapshot, and true-TUI evidence.
- Add explicit shared security contracts (PF-27), cross-surface confidentiality
  (PF-28), external-content provenance/taint (PF-29), and isolated public-web
  acquisition (PF-30) for Moderate/Aggressive only.
- Expose effective enforcement and independent browser/content-firewall health
  in the security inspector; unavailable required controls fail closed.

### Out

- Implementing external protocols or integrations.
- Claiming conformance to AP2, OAuth, OpenID, or SPIFFE; this plan adopts
  relevant control semantics inside Corbanu.
- Adding unrelated product capabilities or downstream integrations.
- Changing the existing behavior of Permissive.
- Authenticated browser login, new search providers, classifier training or
  hosted detection services, and the proposed Agent Sweep behavior monitor.
- Automatically migrating/deleting user plaintext credentials. Stronger modes
  detect unsupported legacy credential paths and block or quarantine their use;
  human-directed migration is a later separately authorized workflow.
- Replacing the existing `/permissions` surface; `/security` composes the
  relevant existing controls into a user-facing posture.
- Allowing a model or model-based reviewer to choose, change, or downgrade the
  security level.
- Publishing a finished-feature page before the accepted candidate passes the
  required evidence.

## Invariants

- **Permissive is current behavior.** Its policy snapshot and representative
  workflows must match the pre-feature baseline.
- **Only a human changes the level.** There is no agent tool, prompt command,
  config instruction, or project-file mechanism that can do so.
- **Moderate and Aggressive are deterministic.** Model judgment can warn but
  cannot grant authority.
- **Unknown state fails visibly.** A corrupt or unknown stored level cannot
  silently become Permissive.
- **Downgrades are explicit.** They show what protection is being removed and
  invalidate incompatible pending authority.
- **Inheritance never weakens.** Child agents receive the same or a stricter
  effective level.
- **Moderate/Aggressive secrets stay out of model-visible and audit paths.**
- **Sanitized content is still untrusted.** Provenance is host-assigned and
  derived taint cannot be cleared by summaries, compaction, memory, or children.
- **Isolation and content policy are separate controls.** Neither an available
  browser backend nor a classifier result grants protected-action authority.
- **Aggressive grants are narrow and expire.**
- **Kill switch and revocation override pending work and survive restart.**

## Standards-derived control profile

This profile borrows mature control shapes without turning `/security` into a
standards-integration project. The implementation remains local and
provider-independent. A standard is a design source only where the plan names
the exact behavior adopted.

| Source | Behavior adopted by `/security` | Boundary |
| --- | --- | --- |
| [OpenID AuthZEN Authorization API 1.0](https://openid.net/specs/authorization-api-1_0-final.html) | Separate the policy enforcement point from the deterministic policy decision point; represent each check as subject, resource, action, context, and an allow/deny decision | Adopt the decision shape; do not require a networked AuthZEN service or claim conformance |
| [OAuth Rich Authorization Requests, RFC 9396](https://www.rfc-editor.org/rfc/rfc9396.html) and [OAuth Token Exchange, RFC 8693](https://www.rfc-editor.org/rfc/rfc8693.html) | Give every temporary grant explicit action, resource, destination, limits, actor chain, and expiry; delegation must narrow authority and preserve the acting agent | Adopt authorization and delegation semantics; do not build an OAuth server |
| [OpenID Shared Signals and CAEP](https://openid.net/three-shared-signals-final-specifications-approved/) | Model downgrade, revocation, kill switch, and risk changes as durable events that invalidate cached decisions, grants, and pending approvals | Adopt invalidation semantics locally; cross-system signal exchange is out of scope |
| [Agent Payments Protocol 0.2](https://ap2-protocol.org/ap2/specification/) | Keep approval on a non-agentic trusted surface; bind human approval to the exact canonical action preview; return a secret-free receipt; reject mutation, replay, or mismatch | Adopt the mandate pattern for protected actions; payment and commerce support is out of scope |
| [OWASP Top 10 for Agentic Applications 2026](https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/) | Make every applicable agentic risk category traceable to at least one prevention control and one adversarial test | Use as the release threat-coverage baseline, not as a certification claim |

The release evidence must include a versioned crosswalk from each adopted
behavior to its code boundary, automated test, adversarial case, and true-TUI
flow. Standards drift after the 2026-08-23 review date requires an explicit plan
update; it cannot silently change Permissive or an accepted security level.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | `/home/pfrpc/repos/CorbanuTerminal-pf13-s02` | `feat/pf-13-s02-scoped-vault-resolver` | `1bdc515bff48a4d9048dae7d06c6214e884265bc` | Security-level model, persistence, policy composition, TUI, tests, and evidence |
| Jim Ricketts | `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02` | `feat/pf-13-s02-scoped-vault-resolver` | `1bdc515bff48a4d9048dae7d06c6214e884265bc` | macOS qualification, complete Core regression, and evidence reconciliation |
| Jim Ricketts | `/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01` | `codex/pf-27-shared-security-contracts` | `ea7d4bec720098f6e0994fcfcc59e272108f7e70` | PF-27 shared security contracts; Codex implementation, separate from PF-13 candidate |
| Jim Ricketts | `/Users/travisgood/Documents/ChatGPT/corbanu-pf26-s01` | `codex/pf-26-security-harnesses` | `cb808c30c0058c101597ab2ada3da16238565c5e` | PF-26-S01 Python harnesses, fixtures and evidence only; no native runtime or PF-13 edits |
| Jim Ricketts | `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01` | `codex/pf-30-isolated-runtime` | `9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c` | PF-30-S01 browser backend; shared planning updates serialized here |
| Jim Ricketts | `/Users/travisgood/Documents/ChatGPT/corbanu-pf29-s01` | `codex/pf-29-untrusted-ingress` | `9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c` | PF-29-S01 source ingress and bounded context; no browser or shared manifest edits |

Implementation does not occur in the documentation checkout. Update this plan
before changing the implementation worktree, base, owner, or scope.

New lanes are deliberately `UNALLOCATED` until a worker and a dependency-complete
base are selected. Do not reuse the stale `corbanu-security-levels` coordinates.
Record each actual lane worktree/branch/base here and in its sprint before
activation. Jim Ricketts integrates shared files and records merge commits;
concurrent workers use distinct branches and non-overlapping write scopes.

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `codex-rs/tui/src/slash_command.rs::SlashCommand` | Adds `/security` to the command surface |
| `codex-rs/tui/src/chatwidget/slash_dispatch.rs` | Routes `/security` to its TUI |
| `codex-rs/tui/src/chatwidget/permission_popups.rs` | Existing permission UX patterns; security levels must remain distinct and understandable |
| `codex-rs/tui/src/app/config_persistence.rs` | Persists a confirmed level and updates active state |
| `codex-rs/config/src/config_toml.rs::ConfigToml` | Typed configuration and schema boundary |
| `codex-rs/protocol/src/models.rs::PermissionProfile` | Existing low-level permission policy that `/security` composes without redefining |
| `codex-rs/tui/src/bottom_pane/approval_overlay.rs` | Existing approval UI used for protected-action confirmation |
| `codex-rs/security-policy/src/lib.rs` | Existing feature-worktree crate exporting levels, actor chains, authorization requests, bounded grants, mandates, receipts, and revocation state |
| `codex-rs/core/src/agent/{control,registry}.rs` | Human/agent/session/task identity and child inheritance supplied by the Core policy adapter |
| `codex-rs/vault/src/lib.rs::reveal_for_programmatic_use` | Existing raw-secret helper boundary that PF-13-S04 must gate in Moderate and Aggressive |
| `codex-rs/cli/src/main.rs::run_vault_auth_helper` | Existing supported CLI escape path preserved only for Permissive |
| `codex-rs/network-proxy/src/credential_broker.rs::CredentialBroker` | Scoped OpenAI route stores only secret-free capability state; legacy broker values zeroize on drop |
| `codex-rs/network-proxy/src/credential_broker/providers/openai.rs` | First exact provider fixture: HTTPS `api.openai.com:443`, `POST /v1/*`, bearer authorization |
| `codex-rs/network-proxy/src/policy.rs` | Existing egress control composed by Aggressive without overriding an existing denial |
| `codex-rs/{Cargo.toml,Cargo.lock}`, repository-root `MODULE.bazel.lock`, and crate `BUILD.bazel` files | Dependency and Cargo/Bazel parity required in the sprint that changes each crate edge |

## Sprint execution map

This map covers every implementation and qualification unit currently required
by the plan. PF-15 through PF-22 and PF-13-S01 through PF-13-S04 are
completed and archived with final-tree evidence. PF-13-S05 continues qualification;
PF-27-S01 and PF-26-S01 are completed and archived from their separate contracts/harness allocations. Other drafts become executable
only after dependencies, allocation, and concurrency checks pass.

| Feature ID | Plan feature | Current sprint records | State |
| --- | --- | --- | --- |
| `PF-15` | Typed security-level domain | `qa/security-levels/sprints/PF-15-S01/evidence.md` | completed |
| `PF-16` | Deterministic authorization request/decision | `qa/security-levels/sprints/PF-16-S01/evidence.md` | completed |
| `PF-17` | Bounded grants and delegation | `qa/security-levels/sprints/PF-17-S01/evidence.md` | completed |
| `PF-18` | Human mandates and secret-free receipts | `qa/security-levels/sprints/PF-18-S01/evidence.md` | completed |
| `PF-19` | Revocation and invalidation contract | `qa/security-levels/sprints/PF-19-S01/evidence.md` | completed |
| `PF-20` | Versioned security persistence | `qa/security-levels/sprints/PF-20-S01/evidence.md` | completed |
| `PF-21` | Frozen Permissive compatibility | `qa/security-levels/sprints/PF-21-S01/evidence.md` | completed |
| `PF-22` | Effective runtime policy and agent inheritance | `qa/security-levels/sprints/PF-22-S01/evidence.md` | completed |
| `PF-13` | Vault-backed exact-host credential boundary | S01: `qa/security-levels/sprints/PF-13-S01/evidence.md`; S02: `qa/security-levels/sprints/PF-13-S02/evidence.md`; S03: `qa/security-levels/sprints/PF-13-S03/evidence.md`; S04: `qa/security-levels/sprints/PF-13-S04/evidence.md`; [S05](../../sprints/current/p0-security-levels/pf-13-s05-credential-boundary-adversarial-qualification.md) | in progress; S05 in progress |
| `PF-23` | Moderate/Aggressive protected-surface enforcement | [S01](../../sprints/current/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md), [S02](../../sprints/current/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md), [S03](../../sprints/current/p0-security-levels/pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) | draft |
| `PF-24` | `/security` profile selection and transition TUI | [S01](../../sprints/current/p0-security-levels/pf-24-s01-security-command-and-profile-view.md), [S02](../../sprints/current/p0-security-levels/pf-24-s02-security-confirm-cancel-and-downgrade.md) | draft |
| `PF-25` | Human grants, revocation, and kill-switch TUI | [S01](../../sprints/current/p0-security-levels/pf-25-s01-temporary-grant-tui.md), [S02](../../sprints/current/p0-security-levels/pf-25-s02-revocation-and-kill-switch-tui.md) | draft |
| `PF-26` | Early harnesses, final automated qualification, true-TUI/live-repository proof, and acceptance | [S01 completion evidence](https://github.com/CorbanuCore/CorbanuTerminal/blob/9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c/qa/security-levels/sprints/PF-26-S01/evidence.md), [S04](../../sprints/current/p0-security-levels/pf-26-s04-final-automated-qualification.md), [S02](../../sprints/current/p0-security-levels/pf-26-s02-true-tui-and-live-repository-qualification.md), [S03](../../sprints/current/p0-security-levels/pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | S01 completed; S04/S02/S03 dependency-gated drafts; feature not complete |
| `PF-27` | Shared security integration contracts | [S01 completion evidence](https://github.com/CorbanuCore/CorbanuTerminal/blob/cb808c30c0058c101597ab2ada3da16238565c5e/qa/security-levels/sprints/PF-27-S01/evidence.md) | completed; native consumers remain separately gated |
| `PF-28` | Cross-surface confidentiality | [S01](../../sprints/current/p0-security-levels/pf-28-s01-confidentiality-and-safe-environments.md) | draft |
| `PF-29` | External Content Firewall | [S01](../../sprints/current/p0-security-levels/pf-29-s01-source-envelopes-and-ingress.md), [S02](../../sprints/current/p0-security-levels/pf-29-s02-derived-taint-and-action-context.md) | S01 allocated; readiness inventory pending; S02 draft |
| `PF-30` | Browser Isolation (separate feature) | [S01](../../sprints/current/p0-security-levels/pf-30-s01-isolated-acquisition-runtime.md), [S02](../../sprints/current/p0-security-levels/pf-30-s02-acquisition-integration-and-recovery.md), [S03](../../sprints/current/p0-security-levels/pf-30-s03-runtime-setup-and-recovery-tui.md) | S01 in progress; S02/S03 dependency-gated drafts |

### Adopted feature contracts

- **PF-27:** Extend existing policy/protocol types with versioned requested/
  effective enforcement facts, host-assigned source envelopes, derived taint,
  authority epochs, and trusted UI events. Register shared module/event seams
  once; downstream sprints implement consumers, not competing policy systems.
- **PF-28:** Inventory every model/output/diagnostic/export sink and child
  environment. Reuse PF-13's trusted broker, centralize exact-value and pattern
  redaction, allowlist child environments, and block unsupported credential
  paths in stronger modes. Redaction is defense in depth, never permission to
  hand plaintext to a model or generic child. Protected financial values use
  explicit narrow derived views; raw balances/positions/PNL are not receipts.
- **PF-29:** Cover repository/files, retrieved web/browser/document text,
  tool/MCP/connector/email results, prior memory, and delegated output through
  existing adapters. Authenticate origin outside content, normalize markup/
  control-token spoofing, preserve taint through every derivation and resume,
  and supply action context for deterministic post-read authorization. Unknown
  provenance denies protected use. No classifier is required for safety.
- **PF-30:** Isolate eligible unauthenticated acquisition in an ephemeral
  backend with pinned runtime, disposable profile, restricted mounts, no host
  IPC/vault/credentials, resource limits, redirect/DNS/IP-aware destination
  enforcement, quarantine, and explicit file promotion. Preserve the existing
  web-tool facade. Disable unsupported/native-search or host-browser bypasses
  in stronger modes. Return PF-29 source envelopes, not a privileged browser
  handle. Missing backend visibly denies acquisition; never fall back. Backend
  selection and supported-platform matrix must be recorded before S01 is ready.

### Browser runtime lifecycle decision

Implements **Moderate/Aggressive isolation and content provenance**, “Reuse an
installed Podman or Docker runtime without replacing it or changing its global
configuration.” The installation, consent, and effective-health flow is PF-30,
not a new security level or an extension of PF-14 arbitrary-model review.

- Preserve a previously selected runtime. With one installed engine, use it;
  with both, prefer a usable Podman unless the user already chose Docker. An
  installed but unavailable engine needs recovery, not silent replacement.
- Pin the Scrapling OCI digest and platform manifests, not `latest`; pin new
  installers and verify their hashes/signatures. Do not upgrade existing engines
  automatically. Verify capabilities and record the actual engine version.
- Check before stronger-mode acquisition and after resume. Pull missing images,
  start stopped owned services, and retry stalled owned services only within a
  bounded recovery budget. Revalidate ownership, image/configuration identity,
  containment, and a real acquisition probe before reporting healthy.
- Installation, VM creation, elevation, and download costs require a trusted
  user-facing consent flow. Do not pipe shell installers from the network or
  accept passwords in application-owned input, captured PTYs, models, or logs.
  OS authentication cancellation is ordinary failure, never permission to bypass.
- No global Docker/Podman context changes, daemon restart, unrelated container
  removal, host mounts, or registry credentials copied into the worker. A
  running container is not sufficient proof of isolation or content-firewall health.
- S01 implements internal lifecycle/containment; S02 joins the web facade and
  ingress; S03 adds native setup/recovery UX after S02 and PF-24-S01. PF-24-S02
  consumes that setup contract before completing level transitions. PF-26-S04
  explicitly waits for S03. Public support for all three OSes remains pending.

Pinned inputs, preflight findings, commands, platform matrix, and limits:
repository file `qa/security-levels/sprints/PF-30-S01/runtime-selection.md`.

S01 implementation allocation: keep containment/lifecycle/worker code in the
new `codex-rs/browser-isolation/` crate and a thin adapter in the already-reserved
Core browser module. The network-proxy browser policy composes existing policy
and adds connection-pinned public-address validation. The worker has no network
interface except loopback; all acquisition HTTP flows cross a bounded stdio
broker. Browser content cannot acquire host sockets or runtime credentials.

The pinned upstream image requires a fixed local derivative recipe to expose
packaged browser binaries to an unprivileged UID; no package installation occurs
in that build. Pin the recipe hash/base digest, record the resulting platform
image ID, and run only that ID. No registry publication or floating-tag trust.
Serial shared-file owner is S01 for `codex-rs/Cargo.toml`, `Cargo.lock`,
`core/Cargo.toml`, and `MODULE.bazel.lock`; PF-29 stays draft during registration.
Record all files and reviewable implementation stages in S01 before edits.

### Injection methods and limits

OpenClaw is a design/fixture source, not a runtime dependency or certification.
PF-29-S01 records source, license, and pinned revision before adapting its
source-labelled wrappers, randomized delimiters, normalization, and model-control
token handling. Reference: [external-content.ts at 6ce272c2](https://github.com/openclaw/openclaw/blob/6ce272c2a662f81b7779507335d91de4d61c589b/src/security/external-content.ts).
Host-issued typed provenance, not the marker text, establishes source identity.
No detector result grants authority; forced misses must still preserve the
deterministic confidentiality and protected-action boundaries.

PF-29 owns a source inventory with concrete adapter, support/denial state, owner,
fixture ID, and evidence for each file/web/document/MCP/connector/email/memory/
delegation path. PF-26 measures task hijacking (including test weakening and
misleading review output) separately from exfiltration. Benign quotations and
legitimate instructions provide false-positive controls. Semantic task failure
must be reported even when no secret leaks; no promise of perfect model obedience
is made. Browser containment does not confer trust on retrieved prose.

### Requirement traceability

All stronger-mode rows below implement **Required trust boundaries** (“External
content enters as untrusted data”) and **Non-negotiable controls** (“Classify
instruction intent and provenance before external content can influence tools
or financial actions”), plus the expanded heading cited above.

| Design requirement | Owner sprint(s) | Coverage state | Acceptance evidence |
| --- | --- | --- | --- |
| Existing Permissive unchanged | PF-21-S01; PF-26-S04 | baseline completed; final comparison pending | Frozen baseline and final compatibility run |
| Exact brokered credential use | PF-13-S01–S05 | S01–S04 completed; S05 pending | Canary, platform evidence, independent review |
| Requested versus effective policy, no widening | PF-22-S01; PF-27-S01; PF-23 | foundation completed; integration pending | Unknown/degraded states, inherited denials, epoch races |
| Secret-free sinks, reflected errors, safe child env | PF-28-S01 | pending; PF-13 evidence reused, not generalized | Canary absent from model requests, tool results, logs, exports, artifacts, environments, and unbound network |
| Provenance and sanitization without trust elevation | PF-29-S01 | pending | Forged markers and supported-source coverage matrix |
| Sticky taint and post-read action checks | PF-29-S02; PF-23-S01 | pending | Compaction, memory, child, resume, unknown-origin and action regressions |
| Browser containment, egress, quarantine, recovery | PF-30-S01–S03 | pending; separate feature | Platform backend matrix, escape/bypass/redirect tests and true-PTY recovery |
| Existing runtime reuse, installation consent, password-free application handling | PF-30-S01/S03; PF-24-S02 | pending | Mac/Linux/Windows installed/missing/stopped/stalled/cancel/elevation/verification flows; unchanged Permissive |
| Inspector and trusted controls | PF-24; PF-25 | pending | Effective facts, separate health, exact grants, revoke/kill, actual-key proof |
| Early attacks plus final integrated qualification | PF-26-S01; S04; S02; S03 | pending | Source/sink crosswalk, full tests, two live repos, independent and human acceptance |
| Complete ingress inventory and non-secret task hijacking | PF-29-S01; PF-26-S01/S04; PF-23-S01 | pending | Every adapter supported or denied; separate task-integrity and authority assertions, benign controls, forced detector misses |
| Upstream separation and compatibility | PF-27-S01; each consumer; PF-26-S04/S02 | pending | Verified upstream baseline, touch record, adapter contracts, integrated Core/platform/TUI evidence |
| Browser login or additional credential providers | No current sprint | deferred | Separate product decision; no implied support |
| Classifier training, hosted detection, Agent Sweep | Proposed firewall plan only | excluded | Not prerequisites and not claimed shipped |

The proposed firewall plan remains non-executable. Only the contracts explicitly
adopted here authorize new drafts; its cancelled sprint catalog is not revived.

### Dependency graph and parallel lanes

The sprint front matter is authoritative; the [current index](../../sprints/current/p0-security-levels/index.md)
lists every edge. Display order is not a waterfall. Up to three eligible lanes
may run together under the sprint-process rules, never all rows automatically.

| Lane | Work | Hard handoff / allowed overlap |
| --- | --- | --- |
| qualification | PF-13-S05 | Mac triage and independent review may run together against pinned evidence; Windows follow-up completed at `ea7d4bec72` |
| contracts | PF-27-S01 | Completed and archived; shared contracts available to eligible consumers, PF-13 qualification unchanged |
| harness | PF-26-S01 | Completed and archived; frozen fixtures/checkers available, native product evidence pending; PF-13 unchanged |
| confidentiality | PF-28-S01 | After PF-13-S05, PF-27, and early harness |
| content | PF-29-S01 then S02 | After PF-27 and early harness; independent of browser backend construction |
| browser | PF-30-S01 then S02 then S03 | S01 after PF-27/harness; S02 joins ingress; S03 joins PF-24-S01 inspector for setup UX |
| enforcement | PF-23-S01 then S02 | S01 joins confidentiality, derived taint, browser integration, and shared contracts |
| lifecycle | PF-23-S03 | After PF-23-S01; can overlap S02 using the completed epoch/dispatch interface |
| inspector | PF-24-S01 then S02 | S01 after PF-27; S02 waits for both enforcement variants and lifecycle |
| grant-ui / revoke-ui | PF-25-S01 / S02 | Parallel after PF-24-S02 shared event/overlay registration; separate view files |
| qualification | PF-26-S04 then S02 then S03 | Final integrated automated proof, actual-key/live-repo proof, then human/docs/release |

Each sprint supplies its intended literal write scopes. Owners and exact
worktrees must be allocated before readiness; any discovered shared-file need
is serialized through a contract/integration prerequisite. No implementation
may start under `ready`, and blocked work retains its slot. Final integration
records dependency commits and reruns all affected proof; prior lane success is
supporting evidence, not a substitute for final-tree acceptance.

Approved scheduling intent: prioritize PF-30-S01 as a parallel browser lane
alongside PF-29 content and PF-28 confidentiality once all three are eligible.
PF-30-S01 does not wait for PF-13-S05 or completion of PF-29. If PF-13 remains
active, it consumes one of the three slots; browser plus content can use the
other two, with confidentiality queued. PF-30-S02 waits for its ingress contract
and serializes any shared facade/registration changes. Jim Ricketts allocates
actual independent worktrees/branches and backend support before readiness.
The 2026-08-27 allocation above starts PF-30-S01; PF-29-S01's native-adapter
inventory must finish before readiness. Its allocated branch does not by itself
authorize code. Shared planning changes originate in the browser worktree and
are integrated into the content worktree before its activation.

PF-30-S01 has an internal backend checkpoint with 272 passing focused Rust tests
and six worker tests on each of Mac/Linux. Its latest evidence is
`qa/security-levels/sprints/PF-30-S01/platform-fixes.md`. S01 remains `in_progress`:
the P1 explicit-denial/DNS ordering and P2 Podman image-ID findings are fixed,
and Fable 5 High returned a clean scoped fix-cycle review via Computer Use.
Its initial trailing-dot finding was withdrawn after validation. The integration owner must
resolve the unchanged native allowlist/DNS ordering before DNS qualification,
allocating any additional write scope first. Real platform qualification
(including Windows) is still incomplete. No dependent sprint is made executable
by this draft.

The historical failed platform attempt is recorded in
`qa/security-levels/sprints/PF-30-S01/platform-qualification-2026-08-27.md`:
with explicit user authorization, Mac's shared Docker engine was recovered and
rootless Podman installed on Linux. Both committed backend smokes failed: Docker
rejects the PID/UTS arguments and Podman's capability inspection is incompatible.
The Mac diagnostic also exposed disabled seccomp that readiness did not detect.
The four-file repair now passes real Mac Docker/Linux Podman backend smokes and
negative seccomp/capability checks; Fable 5 High's scoped re-review is clean.
The three existing Ambient Docker containers are one browser plus egress/DNS
helpers and were preserved. Full adversarial/lifecycle qualification, legacy
engine rejection checks and the native DNS residual remain open. The supplied
Windows endpoint is unreachable from both hosts. No full platform qualification
pass is claimed; see the latest evidence for source hashes and exact coverage.

### Upstream-touch record

Owner: Jim Ricketts. Follow the [upstream integration contract](../upstream-integration.md).
Canonical upstream: `https://github.com/openai/codex.git`. Verified upstream SHA:
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`, fetched from that repository on
2026-08-27. Fork merge `45a60f03d2f6c041d284b41cc3f33c416d9eeed1` incorporates
that exact parent; ancestry to PF-27 base `ea7d4bec720098f6e0994fcfcc59e272108f7e70`
was verified after deepening the shallow checkout. This pins the inherited
baseline, not an upstream upgrade or compatibility pass.

Paths below are under `codex-rs/`; exact files, commands, and artifacts are
resolved in each sprint before readiness. These are planned boundaries, not
claims that every named adapter already exists.

| Feature / sprint | Upstream touch and native seam | Product boundary / reason | Contract proof and upgrade disposition |
| --- | --- | --- | --- |
| PF-13-S05 | Existing broker/provider request adapters; inventory from S01–S04 commits | `vault/`, `security-policy/`; trusted exact credential use | Preserve pinned canary evidence; Windows follow-up passed; full-Core triage and independent review remain pending; inventory before upstream acceptance |
| PF-27-S01 | `protocol/src/lib.rs`, Core/TUI/network module registration and existing Core policy snapshot | Typed security integration contracts; one registration owner | Completed contract/epoch/native-inheritance tests on macOS; [retained/adapted seam decisions](https://github.com/CorbanuCore/CorbanuTerminal/blob/cb808c30c0058c101597ab2ada3da16238565c5e/qa/security-levels/sprints/PF-27-S01/consumer-contracts.md); native consumer qualification remains pending |
| PF-28-S01 | Provider/output serialization and child environments; enumerate all literal hooks | Core confidentiality module consuming vault broker | Exact outgoing-byte, reflected-error, log/export/environment canaries; no duplicate scanner policy |
| PF-29-S01 | `core/src/mcp_tool_call.rs`, `core/src/session/inject.rs`; remaining native adapter inventory pending | `core/src/security/ingress/`; native ingestion adapters | The old `core/src/tools/handlers/read_file.rs` is absent at this baseline; resolve concrete tool/file/context hooks before readiness, not a placeholder implementation |
| PF-29-S02 | `core/src/compact.rs`, `core/src/compact_remote.rs`, `core/src/memories/`, `core/src/agent/control.rs`, `core/src/rollout/`, `core/src/session/rollout_reconstruction.rs` | `core/src/security/taint/`; lineage at native persistence/child seams | Compaction, delegation, memory, restart/resume retain taint and current authority |
| PF-30-S01/S02 | `network-proxy/src/browser_policy.rs`, Core browser adapter and Cargo dependency; S02 owns web facade/tool registration | `browser-isolation/` crate owns worker/host broker/lifecycle; Core remains thin | Real platform containment/egress and cancel/resume/independent-health tests; S01 serially owns workspace/Cargo/Bazel registration before PF-29 activation |
| PF-30-S03 | Native TUI security view/event adapters, exact registrations allocated before readiness | Browser-owned runtime setup coordinator; OS authentication remains outside application input | Setup/cancel/retry/resume/elevation and existing-runtime preservation; no provider/history schema changes; serialize shared TUI registration |
| PF-23-S01/S02/S03 | `core/src/tools/router.rs` and recorded lifecycle hooks | Security protected-surface policy; one deterministic decision path | Native dispatch/post-read epochs/grants/revocation/reconnect; no transport-specific bypass |
| PF-24/PF-25 | Native TUI command/event/overlay registration | Separate security views; no TUI-owned authorization | Wire/UI compatibility, cancel/downgrade/revoke, true-PTY evidence |
| PF-26 | Existing adapters are read-only; fixture and evidence paths in sprint scopes | Independent test harness, not another policy engine | Exact final candidate, complete Core and affected suites, platform matrix, two live-repo TUI flows |

PF-27 resolves shared manifest/lockfile and test-registration ownership before
consumers become ready; add exact paths to its write scope if needed. Each
consumer links its row, fills exact contract commands, and records patch
disposition on upstream changes. PF-26-S04 audits all rows on the integrated
candidate. Passing plan/sprint structure checks is not upstream qualification.

#### PF-26-S01 execution contract

S01 is completed at code commit `bed9c5bfeece2414cbf7e3f54af09fcb646959ed`, with
39 harness tests, six existing credential-canary tests and a clean Autoreview.
[Evidence and handoff](https://github.com/CorbanuCore/CorbanuTerminal/blob/9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c/qa/security-levels/sprints/PF-26-S01/evidence.md)
retain pending product results; the remaining three PF-26 sprints are not complete.

The harness base is the pushed PF-27 completion commit
`cb808c30c0058c101597ab2ada3da16238565c5e`; its inherited upstream ancestry is
unchanged. Pin the accepted PF-21 baseline bytes and PF-27 adapter definitions,
and validate those pins before fixture preparation. Do not rewrite historical
evidence or create a Python policy engine. Native adapters are read-only; their
eventual owners supply host-recorded observations, never model-authored verdicts.

The lane owns `scripts/security_level_{compat,evidence,adversarial,capture,standards_check}.py`,
the three `scripts/security-level-*` entrypoints named in S01, corresponding
`scripts/test_security_level_*.py` tests, `qa/security-levels/fixtures/`, and
`qa/security-levels/sprints/PF-26-S01/`. Shared plan/sprint metadata is updated
serially. The capture fixture is loopback-only, synthetic, and never forwards to
a provider; it supports PF-13's future transport harness, not a shipping proxy.

Contract checks: immutable Permissive probe hashes; all seven pinned PF-27
adapter definitions and their recorded source/test mappings; complete ingress,
sink, control and ownership inventories; strict evidence identity/digest checks.
Run `python3 -m unittest discover -s scripts -p 'test_security_level_*.py'`,
`python3 docs/plans/check.py`, and `python3 docs/sprints/check.py` after formatting.
Python-only work does not require another Rust build or an actual-key TUI run.
Fixture self-tests may pass while product observations remain pending; no
synthetic report is accepted as final candidate qualification. PF-26-S04 supplies
the integrated candidate and platform evidence, S02 supplies tmux/live-repository
proof, and S03 supplies named human acceptance and release evidence.

#### PF-27 execution contract

Jim Ricketts owns shared registration; PF-27 landed a reviewable policy-only
slice in `security-policy/src/integration.rs` and `integration_tests.rs`, registered
in `security-policy/src/lib.rs`. Reuse `SecurityLevel`/`SecuritySettings`,
`AuthorizationRequest`, `BoundedGrant`, and `RevocationState`; do not introduce a
second policy, persistence, or agent-lifecycle implementation. Dependency direction
is runtime/inspector adapters → protocol contracts → security-policy primitives,
never policy → Core/TUI/provider implementations.

The first slice had no new dependencies, manifests, installation, or runtime
activation. Core/TUI/protocol registrations subsequently landed serially in PF-27,
not in consumer work. Expand literal write scope before any further manifest/lockfile or extra
registration file is changed. PF-23 consumes policy/action epochs; PF-24/25 consume
inspector facts and trusted requests; PF-28 consumes confidentiality health;
PF-29 owns source/taint producers; PF-30 owns browser-health producers. Their native
footprints are the rows above; no consumer is activated by this allocation.

Commands from `codex-rs`: `just fix -p codex-security-policy`, `just fmt`, then
`just test -p codex-security-policy`. Before later adapter handoff also run
`just test -p codex-protocol`, `just test -p codex-core security::`, and
`just test -p codex-tui security::` after the corresponding scoped fixes/formatting;
new fixture names and broader affected tests are recorded with those adapters.
Final command results, code candidate, source hashes, and consumer fixture definitions
are recorded in `qa/security-levels/sprints/PF-27-S01/evidence.md`. PF-27 is completed
and archived; dependent sprints still need their other prerequisites and allocations.

PF-27 completion allocation: the contracts lane also owns
`security-policy/src/{provenance,action_context}.rs` and sibling tests,
`protocol/{Cargo.toml,src/security.rs,src/security_tests.rs}`, Core
`src/security/{mod,effective_policy,integration,integration_tests,trusted_requests}.rs`,
TUI `src/lib.rs` and `src/security/`, `codex-rs/Cargo.lock`, and
`MODULE.bazel.lock`. Shared edits are serial within this lane. The only new crate
edge is protocol → existing security-policy; no browser/runtime dependency is
selected here. Core adds a fresh runtime incarnation to its existing policy epoch,
not a second persisted authority system. No model/provider tool schema or live
security command is activated. PF-29/PF-23/PF-24/PF-25 consume the seams later.
Run scoped Clippy and `just fmt`, then `just test -p codex-security-policy`,
`just test -p codex-protocol`, `just test -p codex-core --lib security::`,
`just test -p codex-core --lib security_inheritance`, and
`just test -p codex-tui --lib security::`, using `--cargo-profile ci-test` for
bounded build storage. Register each concrete conformance fixture and downstream
owner in the sprint evidence; definitions are not native-adapter qualification.
PF-27 also registers empty downstream module files in Core `src/security/`
(`confidentiality`, `ingress`, `taint`, `browser_isolation`, `protected_surface`,
`aggressive`, `transition`, `recovery`, `ui_events`), TUI `src/security/view.rs`
and `src/bottom_pane/{mod,security_view}.rs`, and network-proxy
`src/{lib,browser_policy}.rs`. These reserve disjoint consumer-owned files, not
working controls. Include `just fix -p codex-network-proxy --profile dev-small`
and `just test -p codex-network-proxy --cargo-profile ci-test` in final checks.

PF-14 remains proposed. Its packet, provider-readiness, and child-runtime work
must reuse these contracts; shared Core files serialize across plans. The remote
Linux/tmux reconnect investigation is separate routine diagnosis, tracked in
`qa/reliability/2026-08-27-linux-tmux-reconnect.md`; no transport fix or timeout
change is hidden inside this security plan.

### PF-13 integration contract

PF-13 uses existing security primitives instead of defining a parallel
capability system. `codex-security-policy` owns secret-free request, actor,
grant, revocation, and receipt types. Core owns human/agent/session/task identity,
policy composition, and the bounded capability lifecycle. Vault resolves an
approved label only inside a zeroizing callback. The network proxy validates the
transport and injects the credential only for `POST https://api.openai.com/v1/*`
using the existing OpenAI bearer header; redirects and adjacent hosts fail.

Permissive retains the shipping `vault auth-helper` and broker behavior.
PF-13-S04 makes that raw-secret helper unavailable to agent execution under
Moderate and Aggressive, which must use the brokered path. PF-13-S05 supplies
canary and independent-review evidence before PF-23 composes the boundary into
the security profiles. PF-26-S03 updates finished vault/authentication guidance
only after candidate acceptance.

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Existing-user compatibility | Pre-feature config and representative workflows | Upgrade and continue without opening `/security` | Level is Permissive; behavior is unchanged | Policy snapshots, approval decisions, and workflow outcomes match the baseline |
| Open and cancel | Any level | Open `/security`, highlight another level, press `Esc` | Tab closes and current level remains | No config, session, child-agent, or audit state changes |
| Select Moderate | Permissive | Select Moderate, inspect differences, confirm | Moderate becomes visible immediately | Active and future agents enforce Moderate and one redacted change event is recorded |
| Moderate hostile input | Moderate; untrusted page, file, or tool output contains instructions | Ask the agent to process it | Normal analysis may continue; secret, protected-data, policy-change, and protected-action requests are blocked | No protected value or unauthorized action reaches model-visible output or execution |
| Select Aggressive | Moderate | Select Aggressive and confirm | Sensitive surfaces show denied-by-default state and grant affordances | All listed sensitive paths deny until a human grants narrow access |
| Aggressive temporary grant | Aggressive | Grant one sensitive action with scope and expiry | Only that action becomes available; scope and expiry remain visible | Adjacent tool, account, destination, child agent, and post-expiry attempts fail |
| Downgrade | Aggressive with a pending grant or approval | Select Permissive and confirm the protection-removal summary | Downgrade applies and incompatible pending authority is invalidated | No old grant or approval can be replayed |
| Restart/resume | Moderate or Aggressive with kill switch or revocation active | Restart Corbanu and resume the session | Level and restrictive state are restored | No transient fallback to Permissive and no stale approval restoration |
| Agent tries policy change | Any level | Prompt or tool output asks Corbanu to weaken security | Request is treated as untrusted content | No policy mutation path is available to the agent |
| Isolated web acquisition | Moderate/Aggressive; qualified backend | Read hostile public content, cancel a request, then retry | Content is labeled untrusted; cancellation cleans up isolated state | No host profile, credential, IPC, workspace, or forbidden network access |
| Isolation unavailable and recovery | Moderate/Aggressive; missing/crashed backend | Request acquisition, restore backend, retry | Visible denial followed by isolated recovery | No host-browser fallback; health and audit facts agree |
| Derived hostile content | Moderate/Aggressive | Summarize, compact, delegate, store memory, restart, then request protected action | Provenance persists and protected policy rechecks | No source-derived authority or protected disclosure |

## Standards-derived acceptance contract

| Contract | Required evidence |
| --- | --- |
| Deterministic decision | Every protected operation produces a typed subject/resource/action/context request and allow/deny result outside the model; malformed or incomplete requests deny visibly |
| Narrow delegation | A child agent receives no more authority than its parent; the record preserves both the human principal and acting-agent chain |
| Bounded grant | A temporary grant names its action, resource, destination, quantitative limits, and expiry; changing any bound field or using it for an adjacent operation fails |
| Exact human intent | The trusted TUI presents a canonical action preview and binds approval to it; mutation, replay, duplicate submission, and stale approval fail and produce secret-free receipts |
| Immediate invalidation | Downgrade, revocation, kill switch, and risk events invalidate cached decisions, active grants, and pending approvals before another protected operation can start and remain effective after restart |
| Threat coverage | A versioned OWASP crosswalk maps every applicable agentic risk to a prevention control, automated or adversarial case, true-TUI flow where interactive, result, and artifact |

## Implementation sequence

1. **Preserve accepted foundations; finish PF-13 qualification.** Complete Mac
   Core failure triage and independent review. The Windows follow-up passed at
   `ea7d4bec72`. No completed foundation is reopened merely to rename its scope.
2. **Land shared contracts and early harnesses.** PF-27 defines integration
   seams; PF-26-S01 supplies hostile-source fixtures and test runners. The
   read-only PF-24-S01 inspector can proceed against those status contracts.
3. **Build independent boundaries.** Schedule confidentiality, external-content,
   and browser work subject to slots and literal write scopes; join browser
   integration with the ingress contract before enabling acquisition.
4. **Compose enforcement and lifecycle.** PF-23-S01 joins the boundaries;
   Aggressive enforcement and transition/recovery can then run independently.
5. **Wire trusted controls.** PF-24-S02 lands shared event plumbing; grant and
   revoke/kill views can run in parallel. Each interactive sprint records actual
   keys before completion, not just snapshots.
6. **Join on a frozen candidate.** PF-26-S04 runs full automated/security/platform
   qualification; PF-26-S02 repeats integrated true-TUI flows in both live repos.
   Isolated platform/repository runs may execute concurrently at the same commit.
   PF-26-S03 obtains human acceptance, finished docs, and release/benchmark proof.

## Automated evidence

Run fix and formatting tools before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Plan and sprint lifecycle | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | pending | governance-check output |
| Rust fix | `cd codex-rs && just fix -p <affected-project>` for every affected crate | pending; run before formatting/final tests | `qa/release/<version>/security/fix.txt` |
| Permissive compatibility | `python3 scripts/security-level-compat --baseline <commit> --candidate <binary> --output <dir>` | pending; harness is PF-21/PF-26 work | `qa/release/<version>/security/compatibility/` |
| Security policy | `cd codex-rs && just test -p codex-security-policy` | pending | `qa/release/<version>/security/policy-tests.txt` |
| Config and core integration | `cd codex-rs && just test -p codex-config && just test -p codex-core` | pending | `qa/release/<version>/security/integration-tests.txt` |
| Vault and network boundaries | `cd codex-rs && just test -p codex-vault && just test -p codex-network-proxy` | pending | `qa/release/<version>/security/boundary-tests.txt` |
| TUI and snapshots | `cd codex-rs && just test -p codex-tui` | pending | `qa/release/<version>/security/tui-tests.txt` |
| Adversarial matrix | `python3 scripts/security-level-adversarial --bundle <prepared-dir> --observations <host-run.json> --candidate <binary> --source-commit <sha> --platform <platform> --not-before <UTC> --output <dir>` | pending; S01 constructs, S04 qualifies final candidate | `qa/release/<version>/security/adversarial/` |
| Standards crosswalk | `python3 scripts/security-level-standards-check --manifest <crosswalk.json> --candidate <binary> --source-commit <sha> --platform <platform> --not-before <UTC>` | pending; S01 constructs, S04 closes coverage; preparation is not qualification | `qa/release/<version>/security/standards-crosswalk.json` |
| Formatting | `cd codex-rs && just fmt`, then inspect the diff | pending; precedes final affected tests | `qa/release/<version>/security/fmt.txt` |
| Final affected tests | `cd codex-rs && just test -p <affected-project>` for each changed project; never direct `cargo test` | pending | `qa/release/<version>/security/final-tests.txt` |

## True-TUI evidence

Use the `test-tui` skill, `RUST_LOG=trace`, and an isolated `log_dir`. Send
prompt text and Enter separately. `corbanu exec` is not acceptable proof.

| Flow | Test repository | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- |
| Permissive compatibility | TensorCash disposable worktree | Run the frozen baseline workflow; open `/security`; verify Permissive; repeat | Same approvals, tools, output, and persistence before and after | pending | `qa/release/<version>/security/tui/permissive/` |
| Moderate | TensorCash disposable worktree | Select Moderate; confirm; process hostile fixture; attempt protected action; cancel and retry | Level visible; normal work continues; prohibited request blocked; approval state exact | pending | `qa/release/<version>/security/tui/moderate/` |
| Aggressive | Isometric Game disposable worktree | Select Aggressive; confirm; attempt sensitive tool; grant one scoped action; spawn child; wait for expiry | Default denial; one narrow grant; child cannot weaken; expiry removes access | pending | `qa/release/<version>/security/tui/aggressive/` |
| Downgrade/recovery | Isometric Game disposable worktree | Activate kill switch; request downgrade; inspect warning; cancel once, then confirm; restart and resume | Cancel preserves level; confirmation invalidates pending authority; persisted state is coherent | pending | `qa/release/<version>/security/tui/recovery/` |
| Browser/content boundary | Both disposable repositories | Inspect separate health; acquire hostile content; cancel; disable backend; recover and resume | Source taint persists; unavailable backend denies visibly; no fallback or secret exposure | pending | `qa/release/<version>/security/tui/browser-content/` |

## Live-repository applicability

| Repository | Applicable? | Qualification record |
| --- | --- | --- |
| TensorCash | yes | Permissive compatibility and Moderate protected-action workflow |
| Isometric Game | yes | Aggressive TUI, inheritance, expiry, downgrade, and recovery workflow |

Resolve exact disposable worktree paths and base commits in the release evidence
before qualification.

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Travis Good | pending | pending | Understand levels; preserve Permissive; set up/recover isolation; use Moderate/Aggressive; cancel and downgrade safely | pending; tester named, not signed off | `qa/release/<version>/security/human-acceptance.md` |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/features/security.md`, created only after acceptance | Must cite “P0 `/security` levels” and the Permissive requirement | pending |
| `docs/features/vault.md` and `docs/authentication.md`, updated only after acceptance | Must distinguish Permissive helper behavior from Moderate/Aggressive broker-only resolution and cite “Required trust boundaries” | pending |
| `docs/features/index.md` and `docs/slash_commands.md` | Must expose only candidate-verified `/security` behavior | pending |

## Dependencies, decisions, and blockers

| Item | Owner | Needed by | State |
| --- | --- | --- | --- |
| Permissive golden baseline | Jim Ricketts | PF-26-S04 | PF-21-S01 completed; final-candidate compatibility remains pending |
| Existing security-policy commits | Jim Ricketts | Downstream integration | PF-15 through PF-22 completed and archived; preserve evidence |
| Moderate and Aggressive control matrix | Product authority | PF-23 review | Defined in the product specification; any change requires a product decision |
| Persistence and downgrade invalidation | Jim Ricketts | PF-20/PF-23 | Persistence code is present; transition and final evidence remain pending |
| Independent security reviewer | Travis Good / release owner | PF-13-S05 and final qualification | Fable High: `claude-fable-5`, high effort, selected 2026-08-27. Review result pending; no silent substitution or PF-14 activation |
| Human tester | Travis Good | Final qualification | Named 2026-08-27; final-candidate acceptance pending |
| Browser backend/platform matrix | Jim Ricketts | PF-30-S01/S03 | Podman preferred, existing Docker preserved; pinned Scrapling inputs and all-platform pending matrix in S01 record; Mac/Linux before Windows |
| Lane allocation and shared files | Jim Ricketts | Each sprint readiness | Draft coordinates are UNALLOCATED; serialize shared-file changes and check the three-slot limit |
| Scope reconciliation | Travis Good | This amendment | Approved 2026-08-27; stronger guarantees only in Moderate/Aggressive; browser isolation is PF-30 |

## Release linkage

- Release record: `qa/release/<version>/` — pending target version.
- Benchmark tracker: repository-root `benchmarks/README.md`, when due for the
  target release.
- Remaining blockers: implementation, final compatibility and adversarial
  evidence, independent security review, true-TUI qualification, and named
  human acceptance.

## Completion

- [x] Every currently required implementation unit is represented by a valid single-feature sprint.
- [ ] Permissive compatibility is proven against the frozen pre-feature baseline.
- [ ] Moderate and Aggressive match the product-spec control matrix.
- [ ] Every adopted standards behavior is mapped to passing code, test, and TUI
  evidence without an unsupported conformance claim.
- [ ] Required final-tree automated and adversarial evidence passes.
- [ ] True-TUI and both live-repository workflows pass.
- [ ] No critical security finding remains open.
- [ ] Named human acceptance passes.
- [ ] Finished documentation matches the accepted candidate.
- [ ] Release and due benchmark records are linked.
