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
updated: 2026-08-26
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
| Plan requirement excerpt | “Permissive preserves the shipping behavior and does not silently change existing policies.” |
| PF-13 trust-boundary heading | **Required trust boundaries** |
| PF-13 requirement excerpt | “Credentials are referenced by label and resolved only inside a trusted execution boundary.” |
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
| Jim Ricketts | `/home/pfrpc/repos/CorbanuTerminal-pf13-s02` | `feat/pf-13-s02-scoped-vault-resolver` | `1bdc515bff48a4d9048dae7d06c6214e884265bc` | Security-level model, persistence, policy composition, TUI, tests, and evidence |
| Jim Ricketts | `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02` | `feat/pf-13-s02-scoped-vault-resolver` | `1bdc515bff48a4d9048dae7d06c6214e884265bc` | macOS qualification, complete Core regression, and evidence reconciliation |

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
completed and archived with final-tree evidence. PF-13-S05 is the sole `in_progress`
sprint; later records remain `draft` until their dependencies are completed and
archived.

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
| `PF-26` | Harnesses, true-TUI/live-repository qualification, human acceptance, and finished docs | [S01](../../sprints/current/p0-security-levels/pf-26-s01-security-harnesses-and-standards-crosswalk.md), [S02](../../sprints/current/p0-security-levels/pf-26-s02-true-tui-and-live-repository-qualification.md), [S03](../../sprints/current/p0-security-levels/pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | draft |

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

1. **Reconcile the existing foundation.** Execute PF-15 through PF-21 in
   dependency order. Review the seven existing commits, correct them if needed,
   collect final-tree evidence, and archive each record; code presence alone is
   not acceptance.
2. **Compose runtime authority.** PF-22 makes Core the source of effective policy
   and child inheritance. PF-13 then builds the credential capability, scoped
   vault resolver, exact OpenAI proxy path, bypass closure, and adversarial proof.
3. **Connect protected surfaces.** PF-23 applies Moderate/Aggressive decisions at
   content, vault, permission, approval, network, tool, and agent boundaries
   without changing Permissive or overriding an existing denial.
4. **Build the trusted TUI.** PF-24 implements profile view and transitions;
   PF-25 implements narrow grants, revocation, kill switch, and recovery with
   snapshots. Agents have no route to these human-origin events.
5. **Qualify the final candidate.** PF-26-S01 produces deterministic harnesses
   and the standards crosswalk. PF-26-S02 performs true-TUI success, failure,
   recovery, and resume workflows in disposable TensorCash and Isometric Game
   worktrees with actual keys sent.
6. **Accept, document, and link release evidence.** PF-26-S03 obtains named human
   acceptance, updates only finished security/vault/authentication guidance, and
   records release and benchmark state before plan completion.

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
| Adversarial matrix | `python3 scripts/security-level-adversarial --candidate <binary> --output <dir>` | pending; harness is part of stage 5 | `qa/release/<version>/security/adversarial/` |
| Standards crosswalk | `python3 scripts/security-level-standards-check --manifest qa/release/<version>/security/standards-crosswalk.yaml` | pending; checker and manifest are part of stage 5 | `qa/release/<version>/security/standards-crosswalk.yaml` |
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
| Permissive golden baseline | Jim Ricketts | PF-21-S01 | Commit `220af8dae8` contains an initial manifest/tests; final reconciliation evidence is pending |
| Existing security-policy commits | Jim Ricketts | PF-15 through PF-21 | Seven commits are present; none is accepted until its current sprint completes and archives |
| Moderate and Aggressive control matrix | Product authority | PF-23 review | Defined in the product specification; any change requires a product decision |
| Persistence and downgrade invalidation | Jim Ricketts | PF-20/PF-23 | Persistence code is present; transition and final evidence remain pending |
| Independent security reviewer | Release owner | PF-13-S05 and final qualification | Must be named before either review completes |
| Human tester | Release owner | Final qualification | Must be named before acceptance |

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
