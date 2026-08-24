---
title: "P0 /security levels"
status: active
change_class: product-initiative
priority: P0
owner: "Jim Ricketts"
activation_authority: "Product authority defined in the product specification"
activation_basis: "Accountable sequencing item 1 defines /security as P0 and immediate."
target_release: "TBD — candidate qualified by 2026-10-08"
deadline: 2026-10-08
created: 2026-08-23
updated: 2026-08-24
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "P0 /security levels"
  requirement_excerpt: "Permissive preserves the shipping behavior and does not silently change existing policies."
implementation_worktrees:
  - path: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
    branch: "feat/p0-security-levels"
    base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
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
| Requirement excerpt | “Permissive preserves the shipping behavior and does not silently change existing policies.” |
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

### Out

- Implementing external protocols or integrations.
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
- **Secrets stay out of model-visible and audit paths.**
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
| Jim Ricketts | `/home/pfrpc/repos/CorbanuTerminal-security-levels` | `feat/p0-security-levels` | `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb` | Security-level model, persistence, policy composition, TUI, tests, and evidence |

Implementation does not occur in the documentation checkout. Update this plan
before changing the implementation worktree, base, owner, or scope.

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
| `codex-rs/vault/src/lib.rs::reveal_for_programmatic_use` | Credential-resolution boundary affected by Moderate and Aggressive |
| `codex-rs/network-proxy/src/policy.rs` | Egress control used by Aggressive |
| `codex-rs/security-policy/` (planned) | Small crate for level semantics and deterministic policy composition rather than adding the concept to `codex-core` |

## Sprint execution map

The first execution unit establishes the secret non-disclosure boundary before
security-level UI or probabilistic content classification. It remains a draft
until the delivery owner allocates an exact implementation worktree.

| Feature ID | Plan feature | Current sprint mandate | Execution state |
| --- | --- | --- | --- |
| `PF-13` | Vault-backed egress capability boundary | [PF-13-S01 — Vault-backed exact-host credential substitution](../../sprints/current/p0-security-levels/pf-13-s01-vault-backed-exact-host-credential-substitution.md) | Draft pending worktree allocation |

`PF-13` proves that Corbanu can use a vault credential without placing its raw
value in model-visible state. It is intentionally narrower than a general
capability platform: the first slice binds one broker-supported HTTP credential
to an exact actor, purpose, operation, destination, and lifetime, then resolves
it only at the network transport boundary.

Before implementation begins, the delivery owner must assign the exact
worktree/branch/base commit to PF-13-S01, move it to `ready` or `in_progress`,
and pass `python3 docs/sprints/check.py`. Plan prose does not substitute for the
sprint record.

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

1. **Freeze Permissive compatibility.** Capture representative current policy
   snapshots and TUI workflows before adding level logic. These are the golden
   baseline, not a reinterpreted expectation.
2. **Add the security-level domain.** Introduce the typed enum, versioned
   persistence, effective-policy composition, downgrade invalidation, child
   inheritance, and audit event contract in a small dedicated crate. Define
   typed authorization requests, decisions, bounded grants, actor chains,
   protected-action mandates, receipts, and revocation events from the control
   profile above.
3. **Connect enforcement.** Place deterministic decision and enforcement points
   at the existing content, vault, permission, approval, network, tool, and
   agent-spawn boundaries. Reuse existing policy implementations rather than
   duplicate them.
4. **Build the TUI.** Add `/security`, the three-option tab, concise differences,
   current-level display, confirmation/cancel, downgrade warning, temporary
   grants, expiry, revocation, and kill-switch state with snapshots.
5. **Qualify.** Produce the standards crosswalk, then run compatibility,
   adversarial, mutation/replay, restart, concurrency, inheritance, true-TUI,
   live-repository, and human acceptance on the final formatted candidate.
6. **Document and release.** Publish finished user guidance only after
   acceptance and link the release evidence before closing the plan.

## Automated evidence

Run fix and formatting tools before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Plan lifecycle | `python3 docs/plans/check.py` | pending | plan-check output |
| Permissive compatibility | `python3 scripts/security-level-compat --baseline <commit> --candidate <binary> --output <dir>` | pending; harness is part of stage 1 | `qa/release/<version>/security/compatibility/` |
| Security policy | `cd codex-rs && just test -p codex-security-policy` | pending | `qa/release/<version>/security/policy-tests.txt` |
| Config and core integration | `cd codex-rs && just test -p codex-config && just test -p codex-core` | pending | `qa/release/<version>/security/integration-tests.txt` |
| Vault and network boundaries | `cd codex-rs && just test -p codex-vault && just test -p codex-network-proxy` | pending | `qa/release/<version>/security/boundary-tests.txt` |
| TUI and snapshots | `cd codex-rs && just test -p codex-tui` | pending | `qa/release/<version>/security/tui-tests.txt` |
| Adversarial matrix | `python3 scripts/security-level-adversarial --candidate <binary> --output <dir>` | pending; harness is part of stage 5 | `qa/release/<version>/security/adversarial/` |
| Standards crosswalk | `python3 scripts/security-level-standards-check --manifest qa/release/<version>/security/standards-crosswalk.yaml` | pending; checker and manifest are part of stage 5 | `qa/release/<version>/security/standards-crosswalk.yaml` |
| Formatting | `cd codex-rs && just fmt -- --check` | pending | `qa/release/<version>/security/fmt.txt` |

## True-TUI evidence

Use the `test-tui` skill, `RUST_LOG=trace`, and an isolated `log_dir`. Send
prompt text and Enter separately. `corbanu exec` is not acceptable proof.

| Flow | Test repository | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- |
| Permissive compatibility | TensorCash disposable worktree | Run the frozen baseline workflow; open `/security`; verify Permissive; repeat | Same approvals, tools, output, and persistence before and after | pending | `qa/release/<version>/security/tui/permissive/` |
| Moderate | TensorCash disposable worktree | Select Moderate; confirm; process hostile fixture; attempt protected action; cancel and retry | Level visible; normal work continues; prohibited request blocked; approval state exact | pending | `qa/release/<version>/security/tui/moderate/` |
| Aggressive | Isometric Game disposable worktree | Select Aggressive; confirm; attempt sensitive tool; grant one scoped action; spawn child; wait for expiry | Default denial; one narrow grant; child cannot weaken; expiry removes access | pending | `qa/release/<version>/security/tui/aggressive/` |
| Downgrade/recovery | Isometric Game disposable worktree | Activate kill switch; request downgrade; inspect warning; cancel once, then confirm; restart and resume | Cancel preserves level; confirmation invalidates pending authority; persisted state is coherent | pending | `qa/release/<version>/security/tui/recovery/` |

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

## Dependencies, decisions, and blockers

| Item | Owner | Needed by | State |
| --- | --- | --- | --- |
| Permissive golden baseline | Jim Ricketts | Stage 1 | Must be captured before implementation |
| Moderate and Aggressive control matrix | Product authority | Stage 1 review | Defined in the product specification; any change requires a product decision |
| Persistence and downgrade invalidation design | Jim Ricketts | Stage 2 | Pending implementation design |
| Independent security reviewer | Release owner | Final qualification | Must be named before review |
| Human tester | Release owner | Final qualification | Must be named before acceptance |

## Release linkage

- Release record: `qa/release/<version>/` — pending target version.
- Benchmark tracker: repository-root `benchmarks/README.md`, when due for the
  target release.
- Remaining blockers: implementation, final compatibility and adversarial
  evidence, independent security review, true-TUI qualification, and named
  human acceptance.

## Completion

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
