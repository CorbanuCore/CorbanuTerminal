---
title: "Unified provider onboarding and management"
status: active
change_class: product-initiative
priority: P1
owner: "Codex primary agent"
parallel_sprint_limit: 1
integration_owner: "Codex primary agent"
activation_authority: "Final product authority — user decision"
activation_basis: "The user's 2026-09-01 P1 decision to supersede the remaining Claude-auth planning slot with one provider-auth initiative, preserve the merged Claude implementation evidence, unify onboarding and /providers, support multi-provider setup and deferred Corbanu Plan onboarding, and make every configured provider active by default."
target_release: "TBD"
deadline: "TBD"
created: 2026-08-30
updated: 2026-09-01
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Shipping MVP — LIVE"
  requirement_excerpt: "Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat."
implementation_worktrees:
  - path: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
    branch: "feat/unified-provider-auth"
    base_commit: "f7356a94e032234022a462d65b576a7de2854859"
---

# Unified provider onboarding and management

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | **Active** |
| Active-plan slot | **2 of 2**, superseding the remaining Claude-auth slot |
| Priority | **P1** — current drift is disrupting co-founders and users |
| Product authority | Final product authority — user decision |
| Authoritative decision | Unify onboarding and `/providers` behind one provider catalog, status model, authentication controller, and persisted eligibility policy; retain the merged Claude-auth foundation and its evidence. |
| Accountable owner | Codex primary agent |
| Implementation owner | GPT-5.6 Sol high implementation agent |
| Target release | TBD |
| Deadline | TBD |

This plan continues the active record previously titled **Reliable Claude
subscription authentication**. PF-42 through PF-47 landed on `origin/main` in
`f7356a94e032234022a462d65b576a7de2854859`; their completed sprint records
remain historical evidence. The expanded plan does not reopen those delivered
contracts, represent their unclosed release gates as passed, or consume a third
active-plan slot.

## User pain

First-run onboarding and `/providers` currently maintain separate provider
lists, status rules, persistence decisions, event routes, and completion
behavior. A provider can appear configured in one interface and absent or
unusable in the other. New providers and custom providers must be wired twice,
and the repaired Claude subscription path can regress when one host bypasses it.

This creates repeated authentication work, ambiguous provider state, and direct
productivity loss for co-founders and users.

## Product intent and ideal flow

On a fresh profile, onboarding shows the same provider catalog and metadata-only
status later shown by `/providers`. The user may configure any number of
providers, returning to the list after each setup. Every successfully configured
provider is active by default. **Done** ends the provider phase only when at
least one usable provider exists or a queued Corbanu Plan flow can produce one.

Choosing Corbanu Plan during the provider phase only queues its onboarding and
returns immediately to the list. After **Done**, the wallet/Plan flow runs with
Escape and cancel support. Success makes Corbanu configured and active without
overriding an already usable current provider. Cancellation continues when
another provider is usable; if Corbanu was the only path, it returns to the
provider screen.

On a fresh install, the first successfully configured provider supplies the
initial default model. An existing usable model/provider selection is preserved.
Later configuration expands the eligible model set without silently switching
the current model.

On return visits, `/providers` uses the same catalog, status resolver, and auth
controller. It additionally allows configured providers to be deactivated or
reactivated without deleting credentials. Credential removal is a separate,
clearly labeled action. Deactivating the provider behind the current model
requires the user to choose a usable replacement first.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.” |
| Related shipping capability | “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” |
| Product outcome advanced | The live multi-provider product has one dependable setup, status, eligibility, and recovery contract. |
| North-star criterion advanced | Users can choose inference without exposing credentials or repeatedly repairing provider state. |

## Scope

### In

- One typed provider catalog derived from runtime provider configuration plus
  product-owned account and Corbanu Plan entries.
- Automatic inclusion in both hosts of custom providers that declare an API-key
  environment variable.
- Metadata-only configured, active/inactive, current, checking, unavailable, and
  recovery-required status from one resolver.
- Persisted provider eligibility: configured providers default active; explicit
  deactivation survives restart and does not delete credentials.
- One renderer-independent authentication flow/controller contract with typed
  events, cancellation, stale-result rejection, and renderer adapters.
- Shared API-key, OpenAI account, and merged Claude subscription backends.
- Multi-provider onboarding with an explicit **Done** action.
- Deferred Corbanu Plan onboarding after provider selection, with success,
  cancel, return, and only-provider recovery behavior.
- `/providers` activation controls and replacement-before-deactivation for the
  current provider.
- Startup gating and current-model selection derived from the shared status and
  eligibility state rather than host-specific heuristics.
- Focused automated, snapshot, integration, adversarial redaction, substantial
  typed-TMUX, live-repository, human, documentation, and release evidence.

### Out

- Changing provider wire protocols, pricing, subscriptions, or model catalogs.
- Automatically importing, copying, or deleting provider-owned credentials.
- Interactive enrollment for a custom command-auth provider without a supported
  typed setup adapter; it remains visible with metadata/status and recovery copy.
- Making credential removal implicit in provider deactivation.
- Automatically switching the current model when a later provider is configured
  or when Corbanu Plan completes.
- Replacing the wallet, x402, or Corbanu Plan purchase implementation.
- Weakening vault, keyring, P0 credential-broker, or provider privacy controls.
- Releasing without a named human test or the repository release gates.

## Invariants

- Authentication state, provider eligibility, and the singular current
  model/provider selection are distinct typed concepts.
- Every successfully configured provider is active unless the user explicitly
  deactivates it; existing configured providers migrate to active by default.
- A provider is never marked configured merely because its catalog entry exists
  or its setup was queued.
- Raw credentials never enter model context, chat, rollout history, telemetry,
  logs, snapshots, TMUX artifacts, ordinary config, or status descriptions.
- Onboarding and `/providers` consume the same catalog and status service and
  dispatch the same typed auth-flow controller actions.
- Host renderers may differ, but authentication success, failure, cancellation,
  recovery, timeout, stale-result rejection, and persistence semantics may not.
- Environment-backed credentials are inspected only as metadata; removal UI
  cannot claim to delete an environment variable.
- Corbanu Plan is configured and active only after its deferred flow succeeds.
- Cancelling any flow is inert with respect to credentials, eligibility, current
  model, unrelated providers, and wallet/Plan state.
- Deactivating the current provider cannot silently select a replacement.
- A current usable provider/model is preserved. A fresh profile chooses the
  first successfully configured provider's default model.
- Unsupported or ambiguous auth sources fail visibly with recovery guidance;
  there is no heuristic account or billing-context fallback.
- No regular expression may inspect, route, transform, approve, repair, score,
  or gate any LLM prompt, model output, tool call, or provider response.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| GPT-5.6 Sol high implementation agent | `/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0` | `feat/unified-provider-auth` | `f7356a94e032234022a462d65b576a7de2854859` | Serial implementation of PF-48 through PF-56 only. |
| Codex primary agent | same receiving worktree | same branch | same base | Feature completeness, integration decisions, scope control, review budget, final-tree verification, and TMUX acceptance. |

`parallel_sprint_limit: 1` is intentional. Shared manifests, config schemas,
provider state, event routing, and TUI hosts make parallel writes unsafe. The
implementation agent is not alone in the repository and must preserve unrelated
user work and integrate rather than revert concurrent changes.

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `codex-rs/model-provider-info/src/lib.rs` | Runtime provider definitions, auth metadata, built-in and custom provider catalog inputs. |
| `codex-rs/login/src/auth/manager.rs::provider_api_key_from_auth_storage` | Runtime credential resolution, legacy/tombstone behavior, and startup parity boundary. |
| `codex-rs/tui/src/onboarding/auth.rs` | Onboarding-owned account/API-key/Claude state machine that must become a host adapter. |
| `codex-rs/tui/src/onboarding/onboarding_screen.rs` | First-run provider sequencing, completion, cancellation, and startup gating. |
| `codex-rs/tui/src/chatwidget/provider_credentials.rs` | Static `/providers` catalog, direct status reads, API-key entry, and later management host. |
| `codex-rs/tui/src/config_update.rs` | Current onboarding provider/model persistence seam. |
| `codex-rs/tui/src/chatwidget/claude_code_login.rs` | Merged Claude auth backend and recovery behavior to preserve. |
| `codex-rs/tui/src/chatwidget/wallet_menu.rs` | Existing Corbanu wallet/Plan setup, cancellation, and completion flow to invoke after **Done**. |
| `codex-rs/tui/src/app.rs` | Startup auth gating and onboarding handoff. |
| `codex-rs/tui/tests/support/tmux.rs` | Required PTY harness and artifact/canary support. |

## Upstream-touch record

| Field | Record |
| --- | --- |
| Baseline | Canonical upstream is `https://github.com/openai/codex.git`. Reverified from local `upstream/main` on 2026-09-01: upstream tip `ba6cf9c69277caec51a4c12c5b7401a9920930e0`, merge-base `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`; fork base for this initiative is `f7356a94e032234022a462d65b576a7de2854859`. Record the final candidate SHA at qualification. |
| Footprint | Product-owned provider-auth catalog/state/controller code, model-provider/config adapters, onboarding, `/providers`, app events/startup, tests, docs, and QA. Each sprint owns the literal subset in its record. |
| Boundary | Centralize Corbanu policy in a focused provider-auth module/crate. Keep upstream TUI, app-server login, config, and model-provider edits as thin typed adapters. TUI renderers never own raw storage or provider selection policy. |
| Compatibility | Preserve provider wire headers, account identity, credential stores, existing Claude source choice, app-server login cancellation, current model, config layering, startup, restart, resume, child-agent use, and native panes. |
| Verification | Contract and migration tests, affected crate suites, snapshots, config schema checks, redaction/canary checks, typed-TMUX flows, both default live repositories, physical platform evidence where required, and release records. |
| Upgrade handling | At final qualification, record upstream candidate, classify each adapter as retained/adapted/removed, resolve conflicts in a disposable worktree, and rerun combined-tree tests and TUI qualification. |

## Sprint execution map

| Feature ID | Plan feature | Sprint record | State |
| --- | --- | --- | --- |
| `PF-42` | Claude typed source/selection contract | [PF-42-S01](../../sprints/archive/claude-subscription-auth/pf-42-s01-auth-source-contract.md) | completed; evidence preserved |
| `PF-43` | Claude managed-token lifecycle | [PF-43-S01](../../sprints/archive/claude-subscription-auth/pf-43-s01-managed-token-lifecycle.md) | completed; evidence preserved |
| `PF-44` | Claude platform-auth resolution | [PF-44-S01](../../sprints/archive/claude-subscription-auth/pf-44-s01-platform-auth-resolution.md) | completed; evidence preserved |
| `PF-45` | Claude auth choice and recovery | [PF-45-S01](../../sprints/archive/claude-subscription-auth/pf-45-s01-auth-choice-and-recovery.md) | completed; evidence preserved |
| `PF-46` | Claude automated qualification | [PF-46-S01](../../sprints/archive/claude-subscription-auth/pf-46-s01-final-qualification.md) | completed; open plan-level gates carried forward |
| `PF-47` | First-run Anthropic account entry | [PF-47-S01](../../sprints/archive/claude-subscription-auth/pf-47-s01-first-run-anthropic-account.md) | completed; merged in `f7356a94e0` |
| `PF-48` | Typed provider catalog and capability contract | [PF-48-S01](../../sprints/archive/unified-provider-auth/pf-48-s01-provider-catalog-contract.md) | completed at `7936d83859` |
| `PF-49` | Shared metadata status and persisted eligibility | [PF-49-S01](../../sprints/archive/unified-provider-auth/pf-49-s01-status-and-eligibility.md) | completed at `5fcde1c1d9` |
| `PF-50` | Renderer-independent API-key authentication controller | [PF-50-S01](../../sprints/archive/unified-provider-auth/pf-50-s01-api-key-flow-controller.md) | completed at `6f90e89792` |
| `PF-51` | OpenAI account adapter on the shared controller | [PF-51-S01](../../sprints/archive/unified-provider-auth/pf-51-s01-openai-account-adapter.md) | completed at `13dcc188fb` |
| `PF-52` | Claude subscription adapter on the shared controller | [PF-52-S01](../../sprints/archive/unified-provider-auth/pf-52-s01-claude-auth-adapter.md) | completed at `a723374834` |
| `PF-53` | Multi-provider onboarding and deferred Corbanu Plan flow | [PF-53-S01](../../sprints/archive/unified-provider-auth/pf-53-s01-multi-provider-onboarding.md) | completed at `30b595034b` |
| `PF-54` | Unified `/providers` management and eligibility controls | [PF-54-S01](../../sprints/current/unified-provider-auth/pf-54-s01-provider-management.md) | in progress |
| `PF-55` | Startup, current-model, and custom-provider convergence | [PF-55-S01](../../sprints/current/unified-provider-auth/pf-55-s01-startup-provider-convergence.md) | draft |
| `PF-56` | Integrated qualification, review, docs, and release evidence | [PF-56-S01](../../sprints/current/unified-provider-auth/pf-56-s01-final-qualification.md) | draft |

## Hard dependency graph

```text
PF-42..PF-47 merged Claude foundation
  -> PF-48 catalog contract
    -> PF-49 status and eligibility
      -> PF-50 API-key flow controller
        -> PF-51 OpenAI account adapter
          -> PF-52 Claude adapter
            -> PF-53 multi-provider onboarding + deferred Corbanu
              -> PF-54 /providers management
                -> PF-55 startup/current/custom convergence
                  -> PF-56 integrated qualification
```

One sprint is executable at a time. A dependent sprint remains draft until its
predecessor is completed and archived.

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Multi-provider success | Fresh profile | Configure two or more providers, return after each, then choose **Done** | All successful providers show configured and active; first successful provider supplies the initial model | Restart preserves eligibility and current model; each provider can make a real request |
| Deferred Corbanu success | At least one configured provider; Corbanu unconfigured | Queue Corbanu, configure another provider, choose **Done**, complete wallet/Plan flow | Provider screen returns immediately after queueing; Corbanu flow starts only after **Done** | Corbanu becomes configured/active and the existing current provider remains selected |
| Deferred Corbanu cancel | Corbanu queued | Escape or cancel deferred flow | No wallet, credential, eligibility, or current-model partial state | Continue with another usable provider; return to provider screen if none exists |
| Shared recovery | Missing, rejected, stale, or conflicting credential | Enter from either host, cancel/retry/replace through the same flow | Same status labels, actions, failure, and recovery semantics | No host-specific fallback or stale completion changes state |
| Deactivate non-current | Configured active provider not serving current model | Deactivate in `/providers` | Provider becomes inactive and credentials remain | Restart preserves inactive state; reactivation requires no reauthentication |
| Deactivate current | Current provider is active | Request deactivation | Replacement picker appears before mutation | Cancel is inert; choosing a usable replacement switches explicitly then deactivates |
| Custom API provider | Runtime custom provider declares `env_key` | Open either host and configure/use it | Provider appears automatically with shared status and masked setup | Both hosts and startup agree; raw key never appears |
| Existing install migration | Existing configured providers and usable selection | Upgrade and launch | Configured providers default active; current selection remains | No forced reauthentication or model switch |

## Implementation sequence

1. Freeze provider identity, setup capability, catalog ordering, custom-provider,
   and host-presentation contracts in PF-48.
2. Centralize metadata-only status plus configured/eligibility persistence and
   migration in PF-49.
3. Build the renderer-independent typed controller and API-key adapter in PF-50.
4. Adapt OpenAI account login in PF-51 and the merged Claude flow in PF-52.
5. Replace onboarding's single-provider completion with the multi-provider and
   deferred Corbanu workflow in PF-53. Its shared host effect executor also maps
   OpenAI `ApiKeyStorage::OpenAiAuth` into the existing app-server API-key
   persistence request; PF-54 reuses that executor.
6. Rebuild `/providers` as a manager host with eligibility and safe replacement
   controls in PF-54.
7. Remove startup/catalog/status heuristics and converge custom-provider/current
   model behavior in PF-55.
8. Format, test, perform substantial true-TMUX qualification, run the bounded
   review program, document, and collect release evidence in PF-56.

## Review and TMUX policy

At most four formal review passes are planned across the initiative:

1. implementation-owner boundary/self-check after the shared contracts;
2. primary-agent integration review after both hosts are migrated;
3. primary-agent final-tree completeness/security review;
4. one external **Claude Fable 5 high** review spawned and controlled through
   the TMUX harness against the final candidate.

No additional autoreview or external review is scheduled. The accountable owner
may exceed four only when a review uncovers a major issue; the finding, reason,
remediation, and additional review must be recorded.

TMUX proof is continuous, not deferred to a token final smoke. PF-53, PF-54, and
PF-55 add focused typed-TMUX scenarios; PF-56 reruns the full matrix on the
formatted integrated tree with real keys, visible checkpoints, cancellation,
recovery, restart/resume, and secret canaries.

## Automated evidence

Run fix and formatting tools before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Governance | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | pending | pending |
| Provider contracts | `CARGO_INCREMENTAL=0 just test -p codex-provider-auth -j 1 --retries 0` or the final approved crate target | pending | pending |
| Login/provider resolution | `CARGO_INCREMENTAL=0 just test -p codex-login -j 1 --retries 0`; affected model-provider tests | pending | pending |
| TUI focused | `CARGO_INCREMENTAL=0 just test -p codex-tui onboarding -j 1 --retries 0`; provider-management and Claude filters | pending | pending |
| Config/schema | affected config edit, parse, migration, and schema checks | pending | pending |
| Adversarial/redaction | cancel races, stale results, timeout, malformed sources, canary scan, and no-regex LLM-path guard | pending | pending |
| Combined tree | formatting/fixes, `git diff --check`, affected integration suites, and final build | pending | pending |

## True-TUI evidence

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Fresh multi-provider | pending | isolated profile plus TensorCash | Configure multiple providers; return after each; **Done**; send prompt and Enter separately | shared status, active defaults, initial model, real response | pending | pending |
| Deferred Corbanu | pending | isolated profile plus Isometric Game | Queue Plan; configure another provider; **Done**; success and separate Escape/cancel runs | deferral order, no override, safe cancel/return | pending | pending |
| Account/API recovery | pending | both applicable repositories | OpenAI, Claude token/login, API-key failure, retry, replace, cancel | identical host semantics, no stale mutation | pending | pending |
| Eligibility/current model | pending | isolated restart fixtures | deactivate/reactivate; current-provider replacement; cancel; restart/resume | credentials preserved, explicit replacement, state durable | pending | pending |
| Custom provider | pending | custom `env_key` and command-auth fixtures | inspect both hosts, configure supported custom provider, start request | automatic catalog inclusion and status parity | pending | pending |
| External review | pending | final candidate | Spawn Claude Fable 5 high through TMUX; supply review mandate; collect findings and exit | exact runtime visible, bounded review, disposition recorded | pending | pending |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | yes | pending environment resolution | pending | Exercises real provider setup, request, restart, and substantial systems work. |
| Isometric Game | yes | pending environment resolution | pending | Exercises interactive list clarity, cancellation, model switching, and resumed work. |

Disposable worktrees and exact base commits must be recorded before PF-56 runs.
Historical Claude artifacts do not substitute for final integrated-tree runs.

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Named tester pending | pending | pending | Multi-provider onboarding, deferred Corbanu success/cancel, `/providers` activation, OpenAI/Claude/API-key recovery, restart | pending | pending |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/features/model-providers.md` | required — **Shipping MVP — LIVE** multi-provider and vault requirements | pending |
| `docs/authentication.md` | required — **Shipping MVP — LIVE** vault requirement | pending |
| `docs/features/claude-plan-authentication.md` | retain citation; update only if final behavior changes the shipped flow | pending |
| Corbanu Plan onboarding guide at its existing canonical page | retain citation; update only after accepted candidate | pending |

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| Claude auth foundation | dependency | implementation owner | PF-48 | merged at `f7356a94e0`; preserve PF-42–PF-47 behavior and evidence |
| Active-plan slot | product | final product authority | activation | Claude slot explicitly superseded on 2026-09-01; P0 plan remains untouched |
| Provider terminology | product | final product authority | PF-49 | configured providers are active by default; current provider is separate |
| Multi-provider onboarding | product | final product authority | PF-53 | configure any number, explicit **Done**, first success supplies fresh-install default |
| Deferred Corbanu | product | final product authority | PF-53 | queue then return; run after **Done**; cancel/escape; no current-provider override |
| Deactivation | product | final product authority | PF-54 | `/providers`; preserve credential; require replacement before deactivating current |
| Review budget | delivery | Codex primary agent | PF-56 | four formal passes maximum unless a major finding is documented |
| External reviewer | delivery | Codex primary agent | PF-56 | Claude Fable 5 high, spawned and controlled via TMUX |
| Upstream baseline/candidate | integration | Codex primary agent | PF-48/PF-56 | local `upstream/main` reverified at `ba6cf9c69277caec51a4c12c5b7401a9920930e0`; reverify final candidate at qualification |
| Live accounts and named human | release | release owner | PF-56/release | pending; cannot be fabricated |
| Physical Linux/Windows evidence | release | release owner | release | prior Claude plan left this unclosed; final applicability must be recorded |
| Target version and benchmark state | release | release owner | release | pending |

## Release linkage

- Release record: `qa/release/<version>/` — target version pending.
- Benchmark tracker row: `benchmarks/README.md` when due.
- Carried blockers from the earlier Claude plan: live eligible account evidence,
  named human acceptance, TensorCash and Isometric Game runs, physical
  Linux/Windows confirmation where required, target release/tag/merge decision,
  release ledger, and due benchmark evidence.
- New blockers: every PF-48–PF-56 sprint must complete and the final integrated
  candidate must pass the stated TMUX and review program.

## Completion

- [x] Product linkage, P1 priority, scope, invariants, ownership, and worktree are current.
- [x] Merged PF-42–PF-47 Claude evidence is preserved without claiming open gates.
- [x] Every new implementation unit has one stable feature ID and one sprint record.
- [ ] PF-48 through PF-56 are completed and archived.
- [ ] Required final-tree automated and adversarial evidence passes.
- [ ] Required true-TUI and both live-repository workflows pass.
- [ ] Named human acceptance passes.
- [ ] Finished documentation matches the accepted candidate.
- [ ] Upstream disposition, release record, and due benchmark state are linked.
- [ ] No hard release gate remains pending.
