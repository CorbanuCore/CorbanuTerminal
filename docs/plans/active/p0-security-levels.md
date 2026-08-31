---
title: "P0 /security levels"
status: active
change_class: product-initiative
priority: P0
owner: "Jim Ricketts"
parallel_sprint_limit: 3
integration_owner: "Codex ingress/classifier lane"
activation_authority: "Product authority defined in the product specification"
activation_basis: "P0 sequencing plus Travis Good’s 2026-08-28 decision to reconcile the complete security program into this active plan."
target_release: "TBD — candidate qualified by 2026-10-08"
deadline: 2026-10-08
created: 2026-08-23
updated: 2026-08-31
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "P0 /security levels"
  requirement_excerpt: "Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged."
implementation_worktrees:
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-protected-runtime"
    branch: "feat/p0-security-protected-runtime"
    base_commit: "43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-isolated-broker"
    branch: "feat/p0-security-isolated-broker"
    base_commit: "43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-classifier-corpus"
    branch: "feat/p0-security-classifier-corpus"
    base_commit: "9d08b15fa94676c1383ee1605b77e7cc7218dcc4"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-credential-reservations"
    branch: "feat/p0-security-credential-reservations"
    base_commit: "9d08b15fa94676c1383ee1605b77e7cc7218dcc4"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-durable-events"
    branch: "feat/p0-security-durable-events"
    base_commit: "9d08b15fa94676c1383ee1605b77e7cc7218dcc4"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-browser-retrieval"
    branch: "feat/p0-security-browser-retrieval-pf33"
    base_commit: "80a2469e401066ebaf04d95ba603ab68cb341854"
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02"
    branch: "feat/pf-13-s02-scoped-vault-resolver"
    base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
    branch: "feat/p0-security-levels"
    base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-foundation-platform"
    branch: "feat/p0-security-foundation-platform"
    base_commit: "1907d99aed9714f05a5f54fca1703658017d616c"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-ingress-classifier"
    branch: "feat/p0-security-ingress-classifier"
    base_commit: "6a35712cd5731b191d875e8c6468f1abe23eb66e"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-revocation-fence"
    branch: "feat/p0-security-revocation-fence"
    base_commit: "5521b681fff0ecb50b17c10bc1dd1356cbecc1b6"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-authoritative-state"
    branch: "feat/p0-security-authoritative-state"
    base_commit: "5521b681fff0ecb50b17c10bc1dd1356cbecc1b6"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-compatibility-drift"
    branch: "feat/p0-security-compatibility-drift"
    base_commit: "5521b681fff0ecb50b17c10bc1dd1356cbecc1b6"
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

## Reconciled planning decision — 2026-08-28

Travis Good requested the full security program, selectively reusing the cancelled
firewall decomposition and using the working-session overview/transcript and
OpenClaw implementation as primary grounding. This **planning update**, not a
product-code or release change, merges those contracts into this one active plan.

[Source reconciliation and all 72 archive dispositions](../security-source-reconciliation.md)
records the source hashes, transcript timestamps, pinned OpenClaw code, differences
from that reference and new owners. The earlier proposed firewall plan remains
historical input, not competing implementation authority.

The [OpenClaw source review](../openclaw-source-review-2026-08-28.md) pins the
2026-08-28 download at `13adff02ca3897768d80d2bca18f5acf08c55d91` and maps
implementations, callers, defaults and tests to OC-1–11. It is targeted reference
research, not OpenClaw-wide or Corbanu release qualification. Reuse persistent
memory provenance as well as turn taint; explicitly test open-channel revocation,
complete reflected-secret removal, missing-lineage handling, proxy/DNS bypasses
and migration ownership. Upstream's trusted-host defaults are not Moderate or
Aggressive guarantees.

Local source-reference input: `/Volumes/CorbanuDrive/Corbanu/.codex-work/references/openclaw-13adff02`.
It is a clean detached checkout of `https://github.com/openclaw/openclaw.git` at
the exact review pin above, with Git hooks disabled; all 42 files in the recorded
source manifest matched their SHA-256 values on 2026-08-30. Agents should use it
for local source reads without fetching, switching revisions, installing or
running OpenClaw. A pin change requires an explicit review update first.

For each adaptation, record the pinned upstream function/test, Corbanu owner and
small Codex integration hook, license notice, deliberate differences and final-tree
regression evidence. Keep the source checkout/reference outside the Corbanu runtime
dependency graph. PF-26 must track unexecuted reference cases separately from
passing candidate evidence; no source citation alone closes an acceptance item.

- **Permissive retains compatibility.** No mandatory broker, environment purge,
  ingestion classifier or migration is silently added to its shipping path.
  Existing stricter low-level restrictions remain intact.
- **Every level above Permissive requires the broker.** Moderate and Aggressive
  use separately constrained credential/financial execution, secretless agent
  launch and deterministic ingress/disclosure policy. Unsupported adapters or
  isolation fail visibly; they never fall back to raw credentials.
- **All source requirements have owners.** Earlier 68/73-record snapshots are
  historical. Sprint 13, PF-34-S04, PF-27-S03, round-two integration, and the
  completed PF-13-S06/PF-41-S03 round-three foundations preserve 24 completed
  archives and leave **52 current units**. All 72
  cancelled firewall records and seven unrelated Autoreview drafts are unchanged.
- **No evidence is retroactively accepted.** Upstream's PF-15–22 and PF-13-S01
  completions remain accepted for their original scope. The stronger review
  requirements have separate follow-ups; PF-13-S05 is completed and archived and
  PF-13-S07 owns final composed credential qualification.
- The existing October 8 deadline is unchanged, but this larger program has not
  been effort-estimated or scheduled. The owner must assess capacity and surface
  any conflict; no scope or release gate is silently waived.

## Accepted architecture refinement — 2026-08-28

Travis Good authorized the assessed recommendations after the Opus 5 / Extra
review. The [architecture appendix](../security-architecture-refinements-2026-08-28.md)
records accepted/rejected claims, platform/state and upstream-seam contracts,
action/profile usability, durable events and required regression detail.

Five draft preparation/foundation sprints make independent work explicit:
PF-27-S03 platform containment, PF-31-S04 retriever artifacts/engine fixtures,
PF-33-S03 pure destination policy, PF-34-S04 segment/verdict fixtures and
PF-41-S03 durable audit/recovery. PF-35 corpus/CPU work now follows the completed
segment contract, while real screening still depends on sanitizer and policy
integration. Browser login gains its missing screening dependency.

The plan opts into the [bounded parallel process](../../sprints/index.md#bounded-parallel-implementation)
with the exact integration owner recorded in front matter. No extra execution owner or worktree has
been allocated and no sprint is activated by this amendment. Keep PF-13-S05 and
PF-26 final qualification, full Moderate/Aggressive guarantees, independent
Permissive evidence and the existing deadline pending measured capacity review.

## Execution capacity and handoff decision — 2026-08-29

Product authority confirmed maximal LLM execution capacity, supplied platform
access for Mac, Windows and Linux, and confirmed that LLM reviewers are always
available. Sprint reviews use Computer Use to operate the logged-in Claude UI
with **Claude Opus 5.0** visibly selected and effort visibly set to **Max**. Each
review still requires immutable packet/model/effort/result evidence, verified
finding disposition and reruns. Reviewer availability does not replace the named
human final-acceptance gate.

The initial coordination packet at
`qa/security-levels/planning/parallel-handoffs-2026-08-29/README.md` historically
divided the first round into foundation/platform, browser/retrieval and
ingress/classifier legs. It is evidence, not current dispatch authority. The
round-two packet at
`qa/security-levels/planning/parallel-handoffs-2026-08-30-round-2/README.md`
owns new dispatches after PF-27-S03 integration.
PF-13-S05, PF-31-S04, PF-33-S03, and PF-27-S03 are completed and archived.
PF-27-S03 occupied the foundation/platform slot at dispatch base
`1907d99aed9714f05a5f54fca1703658017d616c`; one sprint may run at a time in
each lane. A handoff document is coordination, not execution authority: before a
draft becomes `ready`, this plan and its sprint record must name the exact owner,
CorbanuDrive worktree and branch, 40-character post-handoff `main` base, parallel
lane, literal disjoint `write_scope` and `integration_gate`, and both governance
checkers must pass.

PF-13-S05 transferred from its historical Travis worktree to the
`foundation-platform` worktree at `6a35712cd5731b191d875e8c6468f1abe23eb66e`
on 2026-08-29. Historical candidate, executable and platform evidence remains
bound to the commits and paths recorded in that evidence; the transfer does not
relabel it. The transferred sprint owns only the explicitly amended Core repair
files and final qualification artifacts recorded in its sprint record. Jim
Ricketts remains the integration owner.

Until midpoint sprint estimates exist, reserve **35% of total delivery capacity**
for integration, rebases, reviewer-finding remediation, reruns and evidence.
After estimates exist, each sprint reserves `max(0.5 day, 0.20 * midpoint)` plus
0.5 day for each applicable serialized shared surface, consumed cross-lane
contract, and all-OS/true-TUI/live-repository evidence requirement. Add one day
per cross-lane convergence gate and two days for final PF-26 convergence;
reforecast after every gate. Reviewer availability itself is not charged, but
finding remediation is. No calendar feasibility claim is made until raw sprint
estimates, retriever pins, corpus/licensing and the weakest-supported CPU exist.

The browser/retrieval worktree was reallocated on 2026-08-30 at
`1a5562738cb3d53bd4d0b6668761cfe76bd4b93e` for bounded remediation of the
post-archive tmux/Corbanu Terminal/Claude Opus 5 Max findings. PF-31-S04 and
PF-33-S03 rotate through that one lane sequentially; no protected route is
activated by this follow-up.
[PF-31-S04 remediation](../../sprints/archive/p0-security-levels/pf-31-s04-retriever-artifact-preparation.md)
completed at `c2168575695dfb2ad015bf45ef24d9e4b173b571`.
[PF-33-S03 remediation](../../sprints/archive/p0-security-levels/pf-33-s03-destination-policy-contract.md)
then completed sequentially at `80a2469e401066ebaf04d95ba603ab68cb341854`;
the browser/retrieval lane is returned after re-archive.

Credentials and machine login details remain only in the local gitignored
`AgentCredentials.md` chain and must never enter a branch, evidence bundle,
review packet or transcript. Supplied access proves reachability, not platform
containment or release qualification. Tooling, builds, caches, temporary files
and review exports for these lanes remain on CorbanuDrive.

## Upstream reconciliation — 2026-08-28

This review branch incorporates upstream `1bdc515bff48a4d9048dae7d06c6214e884265bc`.
Its runtime code, completed sprint records and qualification artifacts are retained;
this merge adds no runtime implementation or new release qualification.
[Reconciliation evidence](../security-upstream-reconciliation-2026-08-28.md)
records the two parent revisions, preservation checks and planning validation.

| Accepted foundation | New requirement owner | Integration dependency |
| --- | --- | --- |
| PF-13-S01 opaque capability/store | PF-13-S06 usage reservations and trusted metering | PF-13-S03 before live transport |
| PF-19-S01 generation/revocation types | PF-19-S02 dispatch fence and emergency restriction | PF-41-S03 / PF-22-S02, then transports and financial recovery |
| PF-20-S01 versioned config persistence | PF-20-S02 protected authoritative state | After PF-27-S03; before runtime and migration |
| PF-21-S01 frozen independent baseline | PF-21-S02 expanded surfaces and upstream drift | Before PF-22-S02 and final PF-26 |
| PF-22-S01 policy and child inheritance | PF-22-S02 protected runtime and upstream seams | After new state/fence/event/compatibility contracts |

PF-15–18 need no duplicate implementation sprint. Preserve all nine archives and
their original evidence byte-for-byte. New cases cannot be checked off from those
historical passes. The historical PF-22-S01 workspace run was **not green**:
15,788 ran, 15,617 passed, 169 failed, two timed out and 28 were skipped.
The owning repair work and final-candidate qualification remain required.

The allocated Mac worktree/base follow upstream's record; the independent
pre-feature Permissive baseline remains `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`.
Neither updating allocation nor merging upstream rewrites that oracle.

## User pain

### Sprint 13 implementation integration — 2026-08-28

The published `feat/pf-13-s02-scoped-vault-resolver` branch through
`c25e2825a2fe3fe63c8e1d58cc1e3aa82b0d1d04` is integrated into this plan.
PF-13-S02–S05 and the early PF-26-S01 harness are completed and archived;
PF-13-S05 closed after clean integrated Core, Windows component, and independent
review evidence. PF-13-S07 now owns final composed qualification after
PF-13-S06 and downstream isolation/output/migration/connection controls.

The branch's earlier PF-27 shared-contract work remains completed as PF-27-S01.
The refactored isolated broker is PF-27-S04. Earlier overlapping PF-28–30 planning
records are superseded by the canonical PF-27–41 decomposition; their code and QA
artifacts remain integrated inputs, not duplicate completed dependencies.

Canonical record links: [PF-13-S02](../../sprints/archive/p0-security-levels/pf-13-s02-scoped-vault-resolver.md),
[PF-13-S03](../../sprints/archive/p0-security-levels/pf-13-s03-openai-exact-host-proxy-substitution.md),
[PF-13-S04](../../sprints/archive/p0-security-levels/pf-13-s04-authority-lifecycle-and-raw-secret-bypass.md),
[PF-13-S05](../../sprints/archive/p0-security-levels/pf-13-s05-credential-boundary-adversarial-qualification.md),
[PF-13-S07](../../sprints/current/p0-security-levels/pf-13-s07-integrated-credential-boundary-qualification.md),
[PF-26-S01](../../sprints/archive/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md),
[PF-26-S04](../../sprints/current/p0-security-levels/pf-26-s04-final-automated-qualification.md),
[PF-27-S01](../../sprints/archive/p0-security-levels/pf-27-s01-shared-security-contracts.md), and
[PF-27-S04](../../sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md).

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
| Plan requirement excerpt | “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.” |
| Credential trust-boundary heading | **Required trust boundaries** |
| Credential requirement excerpt | “Credentials are referenced by label and resolved only inside a trusted execution boundary.” |
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
- Implement the complete PF-27–41 contracts below: process/environment containment,
  reflected-output gates, migration, durable provenance, isolated retrieval,
  screened search adapters, SSRF controls, sanitization/quarantine, local and
  optional hosted detection, browser login, financial execution/derived views,
  outbound disclosure, Agent Sweep, and runtime inspection/audit.

### Out

- Implementing external protocol servers, new trading venues, or unrelated
  integrations. Security-specific existing-search/Exa/Brave/SearXNG adapters,
  one reviewed login origin plus fixtures and the optional detector interface are in scope;
  this is not authorization for paid accounts, live trades or commercial contracts.
- Claiming conformance to AP2, OAuth, OpenID, or SPIFFE; this plan adopts
  relevant control semantics inside Corbanu.
- Adding unrelated product capabilities or downstream integrations.
- Changing the existing behavior of Permissive.
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
- **Protected modes keep managed secrets outside agent-readable paths.**
  Raw operational credentials and custody material never enter model, generic
  tools, shell environment/argv, logs, transcripts, summaries or artifacts.
  Permissive retains its baseline and makes no added secretless guarantee.
- **Independent defenses compose.** Isolation, tool policy, broker authority,
  egress checks, screening and output gates are separate controls; no classifier
  score or source wrapper grants authority.
- **Taint is durable.** Compaction, memory, a new turn or another agent cannot
  promote untrusted material. Exact human action approval does not erase taint.
- **Failures are observable.** Missing backend, broker, required local classifier,
  safe migration or audit prerequisites block affected protected work; no silent
  native-search/host-browser/raw-auth fallback.
- **Bound the claim.** The protected boundary covers managed credentials and
  classified/configured protected resources. Inventory/redaction heuristics do
  not guarantee discovery of every unknown secret in arbitrary user text, nor
  defend against a fully compromised trusted host. Unknown routes/data deny or
  quarantine; known historical exposure requires clean-context recovery.
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
| Jim Ricketts | `/Users/travisgood/Documents/ChatGPT/corbanu-security-levels` | `feat/p0-security-levels` | `7cc15ae0762664d6d01765de407329887da9f876` | Complete reconciled security program, policy/broker/ingress/financial adapters, TUI and evidence |
| Codex foundation/platform lane | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-foundation-platform` | `feat/p0-security-foundation-platform` | `1907d99aed9714f05a5f54fca1703658017d616c` | PF-27-S03 platform contract, probes, three-platform evidence, and user-authorized serialized G1 crate/workspace/Bazel/lock/CI integration |
| Codex ingress/classifier lane | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-ingress-classifier` | `feat/p0-security-ingress-classifier` | `6a35712cd5731b191d875e8c6468f1abe23eb66e` | PF-34-S04 screening contract, immutable fixtures and user-authorized serialized G1/G2 crate/workspace/Bazel/lock/CI integration |
| Codex revocation/fence lane | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-revocation-fence` | `feat/p0-security-revocation-fence` | `5521b681fff0ecb50b17c10bc1dd1356cbecc1b6` | Completed PF-19-S02 candidate `cc48f367999346bbae0c31b23f9105f229638f0d`; merged at `bff3fe02f` with exports at `1f03913ea` |
| Codex authoritative-state lane | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-authoritative-state` | `feat/p0-security-authoritative-state` | `5521b681fff0ecb50b17c10bc1dd1356cbecc1b6` | Completed PF-20-S02 candidate `cba62fbc9`; merged at `628c63b3c`, activation still blocked |
| Codex compatibility/drift lane | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-compatibility-drift` | `feat/p0-security-compatibility-drift` | `5521b681fff0ecb50b17c10bc1dd1356cbecc1b6` | Completed PF-21-S02 candidate `2555147527c47bd08a2ac9b8e98d706de7432a76`; merged at `c02568c71` and qualified 36/36 |

The PF-34-S04 creation coordinate remains the immutable base recorded above.
After allocation, the lane rebased onto `main` at
`1a5562738cb3d53bd4d0b6668761cfe76bd4b93e` to include the corrected TMUX and
provider behavior before independent review. The final lane-owned remediation
candidate is `a75efecc0a37d5544e123ad19d57867cac360a68`. On 2026-08-30 the user
transferred Jim Ricketts's unavailable G1/G2 integration responsibility to the
Codex ingress/classifier lane. The reviewed commits remain immutable. The lane
combined `main` first at `1907d99aed9714f05a5f54fca1703658017d616c`,
registered the crate at `de99c7af1774cb964f9fcf0cbbfaf2a07c1a059d`,
removed the unused dependency alias at
`279ce48a9e8d3b28ab518ff184aae770d7462d2f`, and reconciled the subsequently
advanced `main` at `3232f5e65bae60bc86122a5495ebb4c280f7c8fb` in merge
`158b9b0ebe4b06a81c98be6a58a0d1c7919a0d08` before final qualification.

Literal PF-34-S04 write scope: `codex-rs/content-security/`,
`codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, root `BUILD.bazel`, root
`MODULE.bazel.lock`, `.github/workflows/security-ingress-contract.yml`,
`qa/security-levels/ingress-contract/`, `qa/security-levels/sprints/PF-34-S04/`,
the current and archive PF-34-S04 sprint-record paths, the current security sprint
index, the global sprint index, `mkdocs.yml`, and this active-plan allocation.
Its `parallel_lane` is `ingress-classifier`. Shared registration is now serialized
inside this one lane; no concurrent sprint owns any listed path.

## Round-two parallel allocation — 2026-08-30

Round two is completed and archived. PF-19-S02 merged first, PF-20-S02 second,
and PF-21-S02 third; the integration owner then updated all compatibility CLI
callers, rebuilt the merged Corbanu binary, reran the policy/config/Core suites,
passed the 36/36 compatibility matrix, and completed a true-TMUX smoke. Protected
activation remains unavailable.

That next frontier was subsequently allocated as round three below. PF-41-S03
and PF-13-S06 completed their disjoint foundation work. Product authority also
resolved PF-35-S01's preparation decisions, but the real corpus campaign,
private blind-custodian aggregate, signed production artifact and N100
measurements remain evidence gates rather than inferred passes. The 35%
integration allowance was recalculated for round three.

After PF-34-S04 was archived, the user's 2026-08-30 integration instruction kept
the Codex ingress/classifier lane as integration owner and assigned PF-27-S03's
serialized G1 registration. That lane merged current `main`, audited the literal
scope, and registered the standalone contract crate; the foundation/platform
lane remained the candidate owner. Integration authority was not returned to
Jim Ricketts by the merge.

Literal PF-27-S03 write scope: `codex-rs/secret-broker/`,
`codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, root `BUILD.bazel`, root
`MODULE.bazel.lock`, `.github/workflows/security-platform-contract.yml`, the
three `scripts/security_platform_probe` command/implementation/test paths,
`qa/security-levels/platform/`, `qa/security-levels/sprints/PF-27-S03/`, the
current and archive PF-27-S03 sprint-record paths, both sprint indexes,
`mkdocs.yml`, and this active plan. Its `parallel_lane` is
`foundation-platform`. Shared registration is serialized at G1; no concurrent
sprint owns any listed path.

Implementation does not occur in the documentation checkout. Update this plan
before changing the implementation worktree, base, owner, or scope.

Parallel preparation lanes and integration gates are in the architecture appendix.
Before a second worker starts, assign distinct named execution owners and exact
worktrees/branches/base commits, fill each sprint's literal write_scope and
integration_gate, and record a dated capacity estimate. The existing shared
coordinates are not permission for concurrent workers in the same checkout.

## Round-three parallel allocation — 2026-08-30

Product authority resolved PF-35-S01's corpus, custody, hardware, model/runtime,
signing and distribution decisions and asked the integration owner to run the
next three dependency-complete lanes through merge and push. The immutable
dispatch base is `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`.

| Lane | Sprint | Owner | Worktree | Branch |
| --- | --- | --- | --- | --- |
| Classifier corpus | PF-35-S01 | Raman | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-classifier-corpus` | `feat/p0-security-classifier-corpus` |
| Credential reservations | PF-13-S06 | Pauli | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-credential-reservations` | `feat/p0-security-credential-reservations` |
| Durable events | PF-41-S03 | Huygens | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-durable-events` | `feat/p0-security-durable-events` |

The scopes are disjoint. PF-35 owns only `content-security` evaluation,
classifier manifests/tooling and its evidence; PF-13 owns the credential-usage
contract in `security-policy` and the existing Core capability seam; PF-41 owns
the new `security-audit` crate and its evidence. The integration owner alone
edits workspace/build registrations, shared navigation, this plan,
`humanTest.html` and `securityProgress.html`, merges each handback, and runs the
combined-tree suites and TMUX evidence.

Raw midpoint estimates are five days for PF-35-S01, three for PF-13-S06 and four
for PF-41-S03. Their formula reserves are 2.5, 1.6 and 1.8 days respectively,
including consumed contracts, serialized shared surfaces and required hardware
or fault-injection evidence. Add one day for each merge convergence gate and
reforecast after every handback. This retains at least the provisional 35%
integration posture; it does not claim calendar completion.

The checked handoff packet is
[`parallel-handoffs-2026-08-30-round-3`](../../../qa/security-levels/planning/parallel-handoffs-2026-08-30-round-3/README.md).
All three lanes require a real TMUX/Corbanu smoke and a read-only Claude Opus 5
Max review despite having no feature-level TUI contract. Those smokes are
supporting evidence and do not replace PF-26 true-TUI or human acceptance.

Round-three integration completed the PF-13-S06 credential reservation contract
and PF-41-S03 durable security-event foundation, including remediation from
independent Opus reviews, combined-tree reruns, shared export/workspace/build
registration and archival. PF-35-S01 delivered a deterministic, fail-closed
manifest/evaluator foundation and recurring CI coverage, but remains
`in_progress`: no private blind result, production signature, complete synthetic
corpus campaign or weakest-supported N100 performance result is claimed.

PF-22-S02 protected runtime integration and PF-27-S04 isolated credential broker
are now dependency-complete and may be allocated in parallel. PF-35-S01's
external qualification is the third independent lane. PF-35-S02 remains gated
until S01 is honestly completed and archived.

## Round-four rolling allocation — 2026-08-31

The user directed the integration owner to treat PF-35's externally operated
dataset/qualification campaign as independent of day-to-day engineering
capacity while preserving its honest `in_progress` evidence state. The formal
three-sprint limit remains: PF-35-S01, PF-22-S02 and PF-27-S04 occupy the three
recorded slots. PF-30-S01 receives a read-only preparation handoff now and may
enter implementation only after PF-22-S02 is integrated and archived, rolling
into that released slot.

The immutable dispatch base is
`43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4`.

| Lane | Sprint | Owner | Worktree | Branch | Raw midpoint | Integration reserve |
| --- | --- | --- | --- | --- | ---: | ---: |
| Protected runtime | PF-22-S02 | `/root/pf22_protected_runtime` | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-protected-runtime` | `feat/p0-security-protected-runtime` | 5.0 days | 5.0 days |
| Isolated broker | PF-27-S04 | `/root/pf27_isolated_broker` | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-isolated-broker` | `feat/p0-security-isolated-broker` | 12.0 days | 4.9 days |
| Source envelope preparation | PF-30-S01 | `/root/pf30_source_envelope` | Unallocated until PF-22 archive | Unallocated until PF-22 archive | 7.0 days | 4.4 days |

PF-22 consumes four cross-lane contracts and owns the first Core/manifest
convergence. PF-27 consumes the platform/event/capability contracts and requires
all-OS evidence. PF-30's forecast is provisional until the integrated PF-22
contract freezes its ingress seam. The reserve values apply the plan formula
and include one convergence day per implementation handback; reforecast after
PF-22 and PF-27 scope audits.

PF-22 exclusively owns its recorded effective-policy, protected-runtime, Core
module/manifest and lock surfaces. PF-27 owns the secret-broker, credential-
broker, Vault capability, new broker-client/config leaves and its evidence; it
does not edit `core/src/security/mod.rs`, Core/root manifests or shared locks.
The integration owner merges and archives PF-22 first, rebases PF-27, then
registers the PF-27 Core seam and reruns the combined tree. PF-30 does not write
until that PF-22 archive exists.

PF-27 construction uses the PF-27-S03 platform candidates without claiming
protected eligibility: Linux dedicated UID/service, macOS launchd/XPC helper,
and Windows service SID/AppContainer with authenticated named pipe. Final
platform acceptance remains fail closed and requires measured macOS, Linux and
Windows evidence. The user will switch tailnets when local final-tree tests and
Opus remediation are stable; no lane attempts remote qualification earlier.

The checked dispatch packet is
[`parallel-handoffs-2026-08-31-round-4`](../../../qa/security-levels/planning/parallel-handoffs-2026-08-31-round-4/README.md).
Every implementation handback requires formatting, affected tests, a supporting
real TMUX smoke and read-only Claude Opus 5 Max review through Corbanu Terminal.
PF-26 final true-TUI, live-repository and human acceptance remain separate.

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
| `codex-rs/network-proxy/src/credential_broker.rs::CredentialBroker` | Existing broker stores `real_value: String`; PF-13-S03 replaces that state on the capability route |
| `codex-rs/network-proxy/src/credential_broker/providers/openai.rs` | First exact provider fixture: HTTPS `api.openai.com:443`, `POST /v1/*`, bearer authorization |
| `codex-rs/network-proxy/src/policy.rs` | Existing egress control composed by Aggressive without overriding an existing denial |
| `codex-rs/{Cargo.toml,Cargo.lock}`, repository-root `MODULE.bazel.lock`, and crate `BUILD.bazel` files | Dependency and Cargo/Bazel parity required in the sprint that changes each crate edge |

## Sprint execution map

This map covers **52 current and 24 completed archived sprints**.
The [ordered execution index](../../sprints/current/p0-security-levels/index.md)
is dependency-correct; feature IDs are not execution order. Archive orders 1–9
stay unchanged and current orders start at 10. Existing allocation coordinates
follow upstream; the five preparation/foundation drafts and five new follow-ups
remain `UNALLOCATED` until assigned. PF-27-S03 and PF-34-S04 are completed and
archived; PF-19-S02, PF-20-S02, PF-21-S02, PF-13-S06, and PF-41-S03 are also
completed and archived. PF-35-S01 remains `in_progress` pending external
qualification evidence; other current records are `draft`, with completed
prerequisites and checked allocation required before execution. Archived links
document original scope, not new passes.

| Feature ID | Plan feature | Sprint records (completed links are archived) | State |
| --- | --- | --- | --- |
| `PF-15` | Typed security-level domain | [PF-15-S01](../../sprints/archive/p0-security-levels/pf-15-s01-security-level-domain-foundation.md) | S01 completed and archived |
| `PF-16` | Deterministic authorization request/decision | [PF-16-S01](../../sprints/archive/p0-security-levels/pf-16-s01-authorization-decision-contract.md) | S01 completed and archived |
| `PF-17` | Bounded grants and delegation | [PF-17-S01](../../sprints/archive/p0-security-levels/pf-17-s01-bounded-delegation-grants.md) | S01 completed and archived |
| `PF-18` | Human mandates and secret-free receipts | [PF-18-S01](../../sprints/archive/p0-security-levels/pf-18-s01-human-mandates-and-receipts.md) | S01 completed and archived |
| `PF-19` | Revocation and invalidation contract | [PF-19-S01](../../sprints/archive/p0-security-levels/pf-19-s01-revocation-contract.md), [PF-19-S02](../../sprints/archive/p0-security-levels/pf-19-s02-dispatch-revocation-fence.md) | S01/S02 completed and archived |
| `PF-20` | Versioned security persistence | [PF-20-S01](../../sprints/archive/p0-security-levels/pf-20-s01-versioned-security-persistence.md), [PF-20-S02](../../sprints/archive/p0-security-levels/pf-20-s02-protected-authoritative-state.md) | S01/S02 completed and archived; protected activation blocked |
| `PF-21` | Frozen Permissive compatibility | [PF-21-S01](../../sprints/archive/p0-security-levels/pf-21-s01-permissive-compatibility-baseline.md), [PF-21-S02](../../sprints/archive/p0-security-levels/pf-21-s02-expanded-compatibility-and-upstream-drift.md) | S01/S02 completed and archived |
| `PF-22` | Effective runtime policy and agent inheritance | [PF-22-S01](../../sprints/archive/p0-security-levels/pf-22-s01-runtime-policy-and-agent-inheritance.md), [PF-22-S02](../../sprints/current/p0-security-levels/pf-22-s02-protected-runtime-and-upstream-seams.md) | S01 completed; S02 follow-up draft |
| `PF-13` | Vault-backed exact-host credential boundary | [S01](../../sprints/archive/p0-security-levels/pf-13-s01-vault-backed-exact-host-credential-substitution.md), [S02](../../sprints/archive/p0-security-levels/pf-13-s02-scoped-vault-resolver.md), [S03](../../sprints/archive/p0-security-levels/pf-13-s03-openai-exact-host-proxy-substitution.md), [S04](../../sprints/archive/p0-security-levels/pf-13-s04-authority-lifecycle-and-raw-secret-bypass.md), [S05](../../sprints/archive/p0-security-levels/pf-13-s05-credential-boundary-adversarial-qualification.md), [S06](../../sprints/archive/p0-security-levels/pf-13-s06-credential-usage-reservations.md), [S07](../../sprints/current/p0-security-levels/pf-13-s07-integrated-credential-boundary-qualification.md) | S01–S06 completed; S07 draft |
| `PF-23` | Moderate/Aggressive protected-surface enforcement | [S01](../../sprints/current/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md), [S02](../../sprints/current/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md), [S03](../../sprints/current/p0-security-levels/pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) | draft |
| `PF-24` | `/security` profile selection and transition TUI | [S01](../../sprints/current/p0-security-levels/pf-24-s01-security-command-and-profile-view.md), [S02](../../sprints/current/p0-security-levels/pf-24-s02-security-confirm-cancel-and-downgrade.md) | draft |
| `PF-25` | Human grants, revocation, and kill-switch TUI | [S01](../../sprints/current/p0-security-levels/pf-25-s01-temporary-grant-tui.md), [S02](../../sprints/current/p0-security-levels/pf-25-s02-revocation-and-kill-switch-tui.md) | draft |
| `PF-26` | Harnesses, true-TUI/live-repository qualification, human acceptance, and finished docs | [S01](../../sprints/archive/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md), [S04](../../sprints/current/p0-security-levels/pf-26-s04-final-automated-qualification.md), [S02](../../sprints/current/p0-security-levels/pf-26-s02-true-tui-and-live-repository-qualification.md), [S03](../../sprints/current/p0-security-levels/pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | S01 completed; S04/S02/S03 draft |
| `PF-27` | Shared contracts, isolated credential broker and secretless launch | [S01](../../sprints/archive/p0-security-levels/pf-27-s01-shared-security-contracts.md), [S02](../../sprints/current/p0-security-levels/pf-27-s02-secretless-agent-launch.md), [S03](../../sprints/archive/p0-security-levels/pf-27-s03-platform-containment-contract.md), [S04](../../sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md) | S01/S03 completed; S02/S04 draft |
| `PF-28` | Central output and reflected-secret protection | [S01](../../sprints/current/p0-security-levels/pf-28-s01-central-secret-output-gate.md), [S02](../../sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md) | draft |
| `PF-29` | Protected-mode inventory and human migration | [S01](../../sprints/current/p0-security-levels/pf-29-s01-protected-mode-inventory.md), [S02](../../sprints/current/p0-security-levels/pf-29-s02-human-secret-migration.md) | draft |
| `PF-30` | Durable provenance and post-taint authority | [S01](../../sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md), [S02](../../sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md), [S03](../../sprints/current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md) | draft |
| `PF-31` | Isolated retrieval and download promotion | [S01](../../sprints/current/p0-security-levels/pf-31-s01-pinned-retriever-isolation.md), [S02](../../sprints/current/p0-security-levels/pf-31-s02-bounded-fetch-no-fallback.md), [S03](../../sprints/current/p0-security-levels/pf-31-s03-download-quarantine-promotion.md), [S04 completed](../../sprints/archive/p0-security-levels/pf-31-s04-retriever-artifact-preparation.md) | draft |
| `PF-32` | Screened web facade and search providers | [S01](../../sprints/current/p0-security-levels/pf-32-s01-web-facade-and-registry.md), [S02](../../sprints/current/p0-security-levels/pf-32-s02-existing-search-and-native-bypass.md), [S03](../../sprints/current/p0-security-levels/pf-32-s03-exa-search-adapter.md), [S04](../../sprints/current/p0-security-levels/pf-32-s04-brave-search-adapter.md), [S05](../../sprints/current/p0-security-levels/pf-32-s05-searxng-search-adapter.md), [S06](../../sprints/current/p0-security-levels/pf-32-s06-privacy-routing-and-failover.md) | draft |
| `PF-33` | Destination validation and connection enforcement | [S01](../../sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md), [S02](../../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md), [S03 completed](../../sprints/archive/p0-security-levels/pf-33-s03-destination-policy-contract.md) | S03 completed; S01/S02 draft |
| `PF-34` | Sanitization quarantine and safe review | [S01](../../sprints/current/p0-security-levels/pf-34-s01-render-aware-sanitization.md), [S02](../../sprints/current/p0-security-levels/pf-34-s02-quarantine-state-and-store.md), [S03](../../sprints/current/p0-security-levels/pf-34-s03-safe-quarantine-review.md), [S04 completed](../../sprints/archive/p0-security-levels/pf-34-s04-screening-contract-and-fixtures.md) | S04 completed; S01-S03 draft |
| `PF-35` | Local classifier and blind qualification | [S01](../../sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md), [S02](../../sprints/current/p0-security-levels/pf-35-s02-local-cpu-detector-artifact.md), [S03](../../sprints/current/p0-security-levels/pf-35-s03-calibration-and-ingress-gate.md) | S01 in progress; deterministic foundation merged, external qualification open; S02/S03 draft |
| `PF-36` | Optional hosted detector and safe fallback | [S01](../../sprints/current/p0-security-levels/pf-36-s01-hosted-detector-consent-contract.md), [S02](../../sprints/current/p0-security-levels/pf-36-s02-hosted-bakeoff-and-local-fallback.md) | draft |
| `PF-37` | Origin-bound browser login and human handoff | [S01](../../sprints/current/p0-security-levels/pf-37-s01-origin-bound-browser-login.md), [S02](../../sprints/current/p0-security-levels/pf-37-s02-human-auth-handoff-lifecycle.md) | draft |
| `PF-38` | Typed financial execution and exact effects | [S01](../../sprints/current/p0-security-levels/pf-38-s01-typed-financial-executor.md), [S02](../../sprints/current/p0-security-levels/pf-38-s02-full-effect-preview-and-mandate.md), [S03](../../sprints/current/p0-security-levels/pf-38-s03-sign-broadcast-and-receipts.md) | draft |
| `PF-39` | Derived financial views and disclosure control | [S01](../../sprints/current/p0-security-levels/pf-39-s01-protected-financial-derived-views.md), [S02](../../sprints/current/p0-security-levels/pf-39-s02-outbound-disclosure-controls.md) | draft |
| `PF-40` | Agent Sweep and safe recovery | [S01](../../sprints/current/p0-security-levels/pf-40-s01-sweep-events-and-rules.md), [S02](../../sprints/current/p0-security-levels/pf-40-s02-isolated-sweep-reviewer.md), [S03](../../sprints/current/p0-security-levels/pf-40-s03-sweep-alerts-and-recovery.md) | draft |
| `PF-41` | Effective security inspector and audit | [S01](../../sprints/current/p0-security-levels/pf-41-s01-effective-security-inspector.md), [S02](../../sprints/current/p0-security-levels/pf-41-s02-tamper-evident-security-audit.md), [S03](../../sprints/archive/p0-security-levels/pf-41-s03-durable-security-event-foundation.md) | S03 completed; S01/S02 draft |

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
Moderate and Aggressive. PF-27 then moves raw resolution to a separately
constrained process; PF-28 adds reflected-response/output gating, PF-29 adds
safe migration and PF-33 binds actual network connections. PF-13-S05 depends
on those completed boundaries before collecting canary/independent-review proof.
The original in-process slice alone is not the final protected-mode guarantee. PF-26-S03 updates finished vault/authentication guidance
only after candidate acceptance.

## Expanded feature contracts

These are unfinished contracts authorized for planning/execution through their
individual sprints, not claims of shipped behavior. New crate/module paths in
sprints are explicitly planned; validate actual foundation paths in the recorded
implementation worktree before readiness. Add Cargo/Bazel parity in the owning
sprint. [Source decisions](../security-source-reconciliation.md) are authoritative
for design provenance; product scope remains in the specification.

### PF-27

**Isolated credential broker and secretless launch.** One trusted broker owns raw credential resolution and a bounded authenticated IPC surface. All model/exec/child/MCP/plugin/hook/provider-pane launch paths use a secretless allowlist and OS restrictions; unsupported platform containment prevents protected activation.

### PF-28

**Central output and reflected-secret protection.** One broker-side output registry/gate covers managed values and supported encodings, short secrets, chunks, active rotations, reflected responses and every persistence/export sink. Resource pressure or unsupported response decoding denies; it never evicts protection for live credentials.

### PF-29

**Protected-mode inventory and human migration.** Inventory only scoped supported sources without leaking their values. Human preview binds migration to exact file identities; use encrypted vault/recovery storage, atomic journal, re-audit and clean-context handling for contaminated histories. Cancel is inert; failure cannot claim activation.

### PF-30

**Durable provenance and post-taint authority.** Trusted ingress assigns immutable source/authority/lineage envelopes. Propagate conservative taint through every provider serialization, summary, memory, cache/import/export, child/mailbox and resume path. Re-evaluate protected actions at use; neither a detector nor a summarizer creates authority.

### PF-31

**Isolated retrieval and download promotion.** Pin a Scrapling-class public retrieval artifact and isolate it from workspace/vault/wallet/browser profiles/IPC with bounded resources and egress. Fetch/open/click/find cannot fall back to the host browser. Downloads are sealed until exact human digest/destination promotion, which preserves taint and does not execute.

### PF-32

**Screened web facade and search providers.** Keep web.run semantics in Permissive; protected modes use one normalized screened facade. Implement the existing SearchClient adapter plus Exa, Brave and SearXNG with minimal queries, stable references, explicit capabilities, cost limits and already-authorized same-role failover. Native search that cannot be screened before model access is disabled.

### PF-33

**Destination validation and connection enforcement.** Enforce scheme/host/port/method/path and all resolved addresses at redirects, retries and actual connections; prevent rebinding, private-network/metadata access and direct socket/proxy/QUIC escape. A human-configured exact self-hosted search service is a narrow adapter exception, not arbitrary private fetch.

### PF-34

**Sanitization quarantine and safe review.** Render-aware cleaning strips hidden/non-body and unsafe control content, preserving raw/sanitized digests and immutable provenance. Encrypted bounded quarantine enforces allow/rescan/quarantine/reject before model visibility, with safe human review and restart recovery. Cleaned or released content remains untrusted.

### PF-35

**Local classifier and blind qualification.** Ship a licensed, reproducible, offline CPU detector with leakage-free evaluator-owned holdouts and profile-calibrated thresholds. Screen complete bounded inputs before exposure. Missing artifacts/timeouts pause ingestion in Moderate and Aggressive; forced false negatives still cannot authorize secrets or financial actions.

The 2026-08-30 product decisions select a synthetic-first, commercial-safe
English corpus generated through pinned vLLM on the owner-supplied RTX host.
The campaign owner searches Hugging Face using the already-provisioned token and
pins an exact commercially usable Qwen 3.8 27B-family repository and immutable
revision. An abliterated research derivative is explicitly permitted when the
standard checkpoint refuses to generate prompt-injection research fixtures;
selection requires license, provenance, architecture/vLLM compatibility and
reproducibility review, not a popularity claim. The selected repository,
revision, tokenizer, license, hashes and runtime/container identities replace
the preparation placeholder before generation evidence is accepted. A
separately custodied encrypted blind corpus and Intel N100/16 GiB/x86-64 Linux
remain the qualification boundaries. DeBERTa-v3-xsmall exported to signed INT8
ONNX Runtime is the primary detector path, with a custom lightweight classifier
only if the primary cannot meet quality and resource gates. One calibrated
score maps through two signed thresholds to allow/suspicious/hostile;
unavailable is deterministic runtime state. The offline Ed25519 root authorizes
a rotating release key, and immutable GitHub Release assets install atomically
with local verification and rollback. The initial accepted-corpus target is
approximately 250,000 training, 25,000 development/calibration and 150,000
evaluator-owned blind records. Unsupported languages fail closed. Risk-triggered
adjudication starts with complete disagreement/uncertainty review plus
stratified 1% human and separate 1% Opus audits; reassess scalability after the
first 10,000 accepted records without weakening the blind qualification gates.

### PF-36

**Optional hosted detector and safe fallback.** Build the optional hosted interface/consent and evaluation path without selecting or purchasing a vendor. Enable only a qualified named service with approved data terms and cost cap; otherwise record disabled-no-qualified-vendor. Outages use an already-qualified local detector or pause. This optional disposition cannot waive required local qualification.

### PF-37

**Origin-bound browser login and human handoff.** Use a separate origin-bound credentialed browser session, not the public retrieval worker or host profile. Broker fills only an adapter-reviewed login form; raw passwords/cookies never reach the model. Qualify a deterministic fixture and one human-selected permitted HTTPS origin with a non-production test account; record the exact origin/form contract before sprint readiness. MFA/passkeys/CAPTCHA remain human-only, sensitive keystrokes unrecorded, and cancellation/revocation destroys pending session authority. Additional origins remain denied until reviewed.

### PF-38

**Typed financial execution and exact effects.** Adapt existing wallet operations and a fake venue, not a new trading integration. Typed canonical requests and atomic limits precede complete-effect simulation/preview. Reuse PF-16–19 mandates; sign and broadcast are separate, exact-human-approved operations in Aggressive. Durable idempotency/status recovery prevents blind replay after uncertain submission.

### PF-39

**Derived financial views and disclosure control.** Account reads stay trusted; model-visible outputs are purpose/precision/expiry-limited derived values with reconstruction budgets. Apply typed disclosure checks to provider payloads, search, tools, social/email, artifacts, clipboard/export and child messages; operational credentials/custody material cannot be disclosed above Permissive.

### PF-40

**Agent Sweep and safe recovery.** Use secret-free tamper-evident behavior events and deterministic anomaly rules to pause/revoke/kill. An optional isolated reviewer receives only sanitized events and cannot grant authority. Human recovery requires fresh scoped authority and preserves taint, audit and irreversible-action status across restart.

### PF-41

**Effective security inspector and audit.** First provide shared versioned event IDs, durable commit/failure and recovery contracts in PF-41-S03; inspector/export integration stays later. Display requested versus actual level/backend/isolation/egress, broker/classifier readiness, leases/grants/expiry, taint, denials, retention and audit integrity. Degradation must be visible. Human support exports are minimized and disclosure-gated; an integrity gap cannot be hidden behind a healthy badge.

### Profile and failure contract

| Surface | Permissive | Moderate | Aggressive |
| --- | --- | --- | --- |
| Existing approvals/sandbox/tool rules | Frozen baseline | Compose existing denials with new policy | Same, never weaken |
| Credentials/agent environment | Existing supported behavior | Broker only; no raw managed secrets in agent env/context | Same plus narrow expiring credential-use grants |
| Protected financial data | Existing policy | Only scoped derived values | Derived views denied until explicit narrow grant |
| Public retrieval/search | Existing route/history/native behavior | Isolated fetch and screened authorized search | Same plus explicit provider/destination grants |
| Ingestion/provenance | Existing behavior | Sanitize, screen, durable taint | Stricter threshold; missing provenance rejects |
| Local classifier unavailable | No new control | Pause external ingestion | Pause external ingestion |
| Optional hosted detector/reviewer | No automatic activation | Explicit data/cost consent and qualification | Same plus narrow disclosure/egress grants |
| Login/challenges/downloads | Existing workflows | Exact-origin broker login; human challenges/promotion | Same plus explicit narrow access |
| Financial operations | Existing policy | Typed limits, complete effect and required mandate | Default deny; exact human approval for each sign and broadcast |
| Clipboard/export/egress | Existing policy | Typed disclosure gate; raw secrets denied | Denied until exact eligible derived-data grant |
| Broker/backend/migration failure | Existing behavior | Block activation or affected protected work | Same; never downgrade automatically |
| Revocation/restart | Existing policy | Durable stop, fresh revalidation | Same; no stale grant/session restoration |

The implementation must version the resolved profile bundle and record effective
sandbox, network, browser, protected-data, lease, ingress, inheritance, retention
and audit settings. Atomic transition previews use that effective bundle, not
only the three-level name. A partially implemented draft is not eligible to
advertise a working protected mode.

### Local classifier qualification targets

Retain the researched targets from the historical proposal as this feature's
qualification baseline, not a new repository-wide policy or a detector guarantee.
PF-35-S01 records the exact weakest supported CPU and evaluation manifests before
training; threshold/resource changes require product review before acceptance.

| Measure | Target |
| --- | --- |
| Benign false positives | ≤0.1% on ≥100,000 held-out benign segments, with confidence interval |
| Known-family detection | ≥80% at that low-FPR operating point |
| Unseen-source/evasion detection | ≥65% at the same threshold; every miss remains policy-contained |
| Benign position/trigger perturbations | ≤2 percentage-point rejection increase |
| CPU envelope | p95 ≤50 ms per 2,048-token segment; peak RSS ≤512 MiB; model ≤300 MiB |
| Privacy | No real customer secrets/protected financial records in corpus, hosted payloads, metrics or artifacts |
| Deterministic safety | All critical unauthorized-disclosure/action cases deny, including forced detector misses |

Long inputs must be bounded and screened across segment boundaries. Measure
end-to-end latency as well as per-segment speed; the model cannot receive a
streamed prefix while screening is pending. A failed required target remains
incomplete; do not substitute a wrapper heuristic or optional hosted service.

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Existing-user compatibility | Pre-feature config and representative workflows | Upgrade and continue without opening `/security` | Level is Permissive; behavior is unchanged | Policy snapshots, approval decisions, and workflow outcomes match the baseline |
| Open and cancel | Any level | Open `/security`, highlight another level, press `Esc` | Tab closes and current level remains | No config, session, child-agent, or audit state changes |
| Select Moderate | Permissive | Select Moderate; pass preflight/migration, inspect differences, confirm | Effective Moderate appears only when all required components are ready | Active/future agents enforce the complete profile; unsupported routes fail visibly and one redacted change event is recorded |
| Moderate hostile input | Moderate; untrusted page, file, or tool output contains instructions | Ask the agent to process it | Normal analysis may continue; secret, protected-data, policy-change, and protected-action requests are blocked | No protected value or unauthorized action reaches model-visible output or execution |
| Select Aggressive | Moderate | Select Aggressive and confirm | Sensitive surfaces show denied-by-default state and grant affordances | All listed sensitive paths deny until a human grants narrow access |
| Aggressive temporary grant | Aggressive | Grant one sensitive action with scope and expiry | Only that action becomes available; scope and expiry remain visible | Adjacent tool, account, destination, child agent, and post-expiry attempts fail |
| Downgrade | Aggressive with a pending grant or approval | Select Permissive and confirm the protection-removal summary | Downgrade applies and incompatible pending authority is invalidated | No old grant or approval can be replayed |
| Restart/resume | Moderate or Aggressive with kill switch or revocation active | Restart Corbanu and resume the session | Level and restrictive state are restored | No transient fallback to Permissive and no stale approval restoration |
| Agent tries policy change | Any level | Prompt or tool output asks Corbanu to weaken security | Request is treated as untrusted content | No policy mutation path is available to the agent |
| Secret migration | Permissive with synthetic legacy auth/history | Dry-run, cancel, confirm, inject failure, recover | Values never shown; protected activation waits for clean state | No plaintext rollback and no contaminated resume |
| Allowed-host reflection | Protected broker with fake provider | Send a credentialed request that reflects the credential | Redacted response or bounded denial | Canary absent from model/tool/log/stream/artifact sinks |
| Poisoned memory | Protected mode | Read hostile text, summarize, store, spawn child and resume | Original untrusted lineage persists | Forced benign detector result still cannot grant authority |
| Screened web failure | Protected web.run | Search/fetch, fail provider/worker/classifier | Safe permitted same-role retry or visible pause | No native-search or host-browser bypass |
| Quarantine/download | Suspicious page or file | Inspect, cancel, rescan or approve exact promotion | Esc is inert; content remains tainted | No active raw rendering, uncontrolled write or auto-execution |
| Brokered login | Fake exact-origin credentialed session | Approve login, handle human challenge, revoke | No password/cookie in model or recorded input | Wrong origin and stale session deny across restart |
| Financial effect | Fake wallet/venue | Preview, sign, separately approve broadcast; simulate timeout | Complete expected effect and honest uncertain status | No mutation, duplicate transfer or blind resubmission |
| Derived-data disclosure | Synthetic protected portfolio | Request view/export, cancel, grant exact permitted view | Only authorized derived data leaves trusted boundary | Reconstruction, raw portfolio and secret export deny |
| Sweep/inspector | Active protected task | Trigger anomaly; inspect runtime; recover and restart | Actual degradation, stop and fresh-authority requirements visible | Old grants remain revoked and audit is secret-free |

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

Use the [52-record current index](../../sprints/current/p0-security-levels/index.md)
for exact dependencies. Bounded parallel allocations follow the sprint process;
reading order is not a serial lock. Including 24 completed archives, the graph
has 76 nodes; exact merged-tree ordering is maintained by the checked current
index rather than inherited historical snapshot arithmetic.

1. **Prepare independent boundaries and reconcile foundations.** Allocate up to
   three checked workers: platform/foundation, browser artifact/destination policy,
   and segment/classifier preparation. Complete the new contract sprints before
   consumers; do not enable any protected path from fixtures alone.
   **Build only the remaining foundation guarantees.** Preserve PF-15–22/PF-13-S01
   completions; PF-13-S06 and PF-19/20/21/22-S02 carry the newly accepted requirements.
2. **Complete the first credential boundary.** PF-13-S02–S04 and S06 extend accepted capabilities,
   resolver and OpenAI substitution. PF-27/28/33 add process, output and connection
   guarantees. PF-24-S01 provides the initial honest profile view; PF-29 adds
   preflight/migration. PF-13-S05 then independently qualifies that complete boundary.
   PF-41-S03 supplies durable event/commit/recovery contracts before runtime,
   broker, quarantine, financial and Sweep consumers, without waiting for audit UI.
3. **Make authority durable and usable.** PF-30 adds provenance/memory/post-taint
   checks; PF-23 composes profiles, PF-24-S02 confirms safe transitions and PF-25
   provides grants/stop/recovery. Missing future adapters remain denied.
4. **Build protected research.** PF-31 isolation, PF-34 cleaning, PF-35 local
   qualification, then PF-34 quarantine/review and PF-32 screened provider adapters.
5. **Qualify optional detection.** PF-36 adds consent, a fake-service contract and
   measured hosted disposition; no commercial vendor is assumed.
6. **Complete protected workflows.** PF-37 login/human handoff, PF-38 existing-wallet
   execution and PF-39 derived-data/outbound checks reuse the same authority types.
7. **Close operator visibility.** PF-40 Sweep and PF-41 actual-state/audit inspection
   complete stop/recovery/degradation coverage.
8. **Requalify the entire final tree.** PF-26-S01 checks every source requirement,
   old-to-new mapping, standards/threat class and forced-miss regression.
   PF-26-S02 repeats all applicable true-TUI workflows in both live repositories.
9. **Accept and document.** PF-26-S03 requires named human/independent review,
   finished-only guidance and release/benchmark evidence. Planning completeness
   is not implementation, release readiness or a promised ship date.

## Automated evidence

Run fix and formatting tools before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Plan and sprint lifecycle | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | pending | governance-check output |
| Rust fix | `cd codex-rs && just fix -p <affected-project>` for every affected crate | pending; run before formatting/final tests | `qa/release/<version>/security/fix.txt` |
| Permissive compatibility | `python3 scripts/security-level-compat --baseline <commit> --upstream <upstream-commit> --candidate <binary> --output <dir>` | pending final candidate; existing PF-21-S01 harness is extended by PF-21-S02/PF-26 | `qa/release/<version>/security/compatibility/` |
| Security policy | `cd codex-rs && just test -p codex-security-policy` | pending | `qa/release/<version>/security/policy-tests.txt` |
| Config and core integration | `cd codex-rs && just test -p codex-config && just test -p codex-core` | pending | `qa/release/<version>/security/integration-tests.txt` |
| Vault and network boundaries | `cd codex-rs && just test -p codex-vault && just test -p codex-network-proxy` | pending | `qa/release/<version>/security/boundary-tests.txt` |
| Expanded broker/ingress/retriever suites | `cd codex-rs && just test -p codex-secret-broker && just test -p codex-content-security && just test -p codex-web-retriever && just test -p codex-web-search-extension` | broker and ingress contracts registered; retriever and search-extension crates remain planned | `qa/release/<version>/security/expanded-boundaries/` |
| Classifier corpus/artifact/quality | `python3 scripts/security-classifier-eval --manifest <frozen-manifest> --artifact <pinned-artifact> --output <dir>` | pending; evaluator is PF-35 work | `qa/release/<version>/security/classifier/` |
| TUI and snapshots | `cd codex-rs && just test -p codex-tui` | pending | `qa/release/<version>/security/tui-tests.txt` |
| Adversarial matrix | `python3 scripts/security-level-adversarial --candidate <binary> --output <dir>` | pending; harness is part of stage 5 | `qa/release/<version>/security/adversarial/` |
| Standards crosswalk | `python3 scripts/security-level-standards-check --manifest qa/release/<version>/security/standards-crosswalk.yaml` | pending; checker and manifest are part of stage 5 | `qa/release/<version>/security/standards-crosswalk.yaml` |
| Formatting | `cd codex-rs && just fmt`, then inspect the diff | pending; precedes final affected tests | `qa/release/<version>/security/fmt.txt` |
| Final affected tests | `cd codex-rs && just test -p <affected-project>` for each changed project; never direct `cargo test` | pending | `qa/release/<version>/security/final-tests.txt` |

## True-TUI evidence

Use the existing Rust `TmuxServer` harness and `test-tui` skill, `RUST_LOG=trace`,
and an isolated `log_dir`; do not replace the typed driver with shell automation.
PF-26-S01 supplies the local capture proxy/scanner; PF-26-S02 retains the required
Ubuntu lane, named credential-boundary test, zero-retry canary checks and cleanup.
Send
prompt text and Enter separately. `corbanu exec` is not acceptable proof.

| Flow | Test repository | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- |
| Permissive compatibility | TensorCash disposable worktree | Run the frozen baseline workflow; open `/security`; verify Permissive; repeat | Same approvals, tools, output, and persistence before and after | pending | `qa/release/<version>/security/tui/permissive/` |
| Moderate | TensorCash disposable worktree | Select Moderate; confirm; process hostile fixture; attempt protected action; cancel and retry | Level visible; normal work continues; prohibited request blocked; approval state exact | pending | `qa/release/<version>/security/tui/moderate/` |
| Aggressive | Isometric Game disposable worktree | Select Aggressive; confirm; attempt sensitive tool; grant one scoped action; spawn child; wait for expiry | Default denial; one narrow grant; child cannot weaken; expiry removes access | pending | `qa/release/<version>/security/tui/aggressive/` |
| Downgrade/recovery | Isometric Game disposable worktree | Activate kill switch; request downgrade; inspect warning; cancel once, then confirm; restart and resume | Cancel preserves level; confirmation invalidates pending authority; persisted state is coherent | pending | `qa/release/<version>/security/tui/recovery/` |

Additional PF-26-S02 cases use the same final candidate and both applicable live
repositories: migration/corrupt recovery; broker reflection/failure; multi-turn
memory taint; screened web failover/blocked classifier; quarantine and download
promotion; exact-origin login/human challenge; full-effect fake sign/broadcast
with uncertain recovery; derived-data export; Sweep and inspector/audit
degradation. Each has success, cancel/deny, failure, recovery and resume
checkpoints and safe evidence. No real funds or credentials are used.

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
| Named by release owner | pending | pending | Understand levels without explanation; preserve Permissive; use Moderate and Aggressive; cancel and downgrade safely | pending | `qa/release/<version>/security/human-acceptance.md` |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/features/security.md`, created only after acceptance | Must cite “P0 `/security` levels” and the Permissive requirement | pending |
| `docs/features/vault.md` and `docs/authentication.md`, updated only after acceptance | Must distinguish Permissive helper behavior from Moderate/Aggressive broker-only resolution and cite “Required trust boundaries” | pending |
| `docs/features/index.md` and `docs/slash_commands.md` | Must expose only candidate-verified `/security` behavior | pending |

## Dependencies, decisions, and blockers

| Item | Owner | Needed by | State |
| --- | --- | --- | --- |
| Permissive golden baseline | Jim Ricketts | PF-21-S02 / PF-26 | S01 oracle preserved byte-for-byte; S02 independent baseline/upstream/candidate plus protected-boundary controls accepted 36/36; PF-26 live qualification remains pending |
| Existing foundations | Jim Ricketts | Current consumers | PF-15–22 and PF-13-S01 completed/archived upstream; five follow-ups remain draft for added review guarantees |
| Moderate and Aggressive control matrix | Product authority | PF-23 review | Defined in the product specification; any change requires a product decision |
| Persistence and downgrade invalidation | Jim Ricketts | PF-20-S02 / PF-23 | S02 protected authoritative-state and external-anchor contracts accepted; activation remains blocked pending a qualified protected provider and PF-23 runtime transitions |
| LLM security review route | Jim Ricketts | Every sprint candidate and final qualification | Reviewer capacity supplied; use Computer Use with visibly selected Claude Opus 5.0 and Max effort, preserve immutable evidence, verify findings and repeat after accepted fixes |
| Human tester | Release owner | Final qualification | Must be named before acceptance |
| Expanded program capacity and integration allowance | Jim Ricketts / product authority | Execution scheduling | Maximal LLM capacity supplied; provisional 35% integration reserve and per-sprint/gate formula recorded above; three-active-sprint limit remains; October 8 feasibility pending measured estimates, no scope silently removed |
| Platform access and isolation capability matrix | Jim Ricketts | PF-27-S03 completion and PF-27/PF-31 integration readiness | PF-27-S03 three-platform probes are accepted; all measured platforms remain ineligible, and unsupported protected paths block visibly pending real mechanism qualification |
| Moderate workflow usability targets | Product authority / Jim Ricketts | PF-26-S02 readiness | Numeric task-completion, approval-count and latency targets with fixed workflows pending; no relaxation of protection |
| Local detector hardware/corpus/license pins | Jim Ricketts / evaluator | PF-35-S01 | Product decisions recorded for synthetic/open commercial-safe sources, independent evaluator custody, provisional N100 floor, Hugging Face selection of a pinned Qwen 3.8 27B-family generator with an abliterated research derivative permitted after license/provenance/compatibility review, DeBERTa/ONNX primary detector, signing root and immutable-release distribution; exact source revision, machine facts and measured evidence remain sprint work |
| Retriever/API/model dependency pins | Jim Ricketts | Owning adapter sprint readiness | Verify then pin current supported artifacts and APIs; historical sources are not fresh release security evidence |
| Optional hosted vendor and data terms | Product authority | PF-36-S02 real-service activation | No vendor selected; interface/fixtures and explicit disabled disposition are in scope |
| First real login origin/test account | Human owner / security reviewer | PF-37-S01 readiness | Record one permitted exact HTTPS origin, reviewed form and non-production account; missing access blocks qualification |
| Additional login/financial adapters | Product authority / security reviewer | Any expansion beyond first reviewed origin/existing wallet | Denied until separately reviewed; no new commercial venue scope |

## Release linkage

- Release record: `qa/release/<version>/` — pending target version.
- Benchmark tracker: repository-root `benchmarks/README.md`, when due for the
  target release.
- Remaining blockers: implementation, unresolved historical full-suite failures,
  final compatibility and adversarial
  evidence, independent security review, true-TUI qualification, and named
  human acceptance.

## Completion

- [ ] Accepted architecture refinements and upstream-seam register have final-tree evidence.
- [ ] Parallel allocations, scope audits and combined-tree integration evidence are complete.

- [x] Complete source scope maps to 52 current and 24 completed single-feature sprints; all 72 cancelled records have explicit dispositions.
- [ ] Every PF-27–41 contract and protected-mode readiness condition passes final-candidate evidence.
- [ ] Optional hosted/reviewer lanes are explicitly qualified-enabled or disabled with an auditable reason.
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
