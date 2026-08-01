# PF Terminal 0.1.27 Canonical Product-Preserving Codex Convergence Spec

Date: 2026-08-01

Status: **P0 / NO-GO**

Product: **PF Terminal**

Release branch: `release/0.1.27-reconstruction`

Released floor: `pfterminal-v0.1.26-pre-convergence` / `d9e2a383ab02550ba26c525c6c99794dd99ae13a`

Pinned upstream Codex: `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`

Clean convergence base: `45a60f03d2f6c041d284b41cc3f33c416d9eeed1`

## 1. Authority and objective

This is the only active implementation and release specification for PF Terminal 0.1.27.
Earlier convergence plans, readiness documents, incident notes, and recovery manifests are
historical evidence only. If they conflict with this document, this document controls.

The objective is:

> Upgrade the Codex foundation underneath PF Terminal while preserving the complete released
> PF Terminal product and its identity.

PF Terminal is not being replaced by Codex, reduced to Codex, or redefined as a branded Codex
build. Upstream supplies runtime mechanics. PF Terminal remains the product and retains its
commands, providers, models, plans, orchestration, wallet, vault, GPU, Telegram, Task Node,
persistence, installers, documentation, and branding.

No deprecation or removal is authorized for 0.1.27. A future deprecation requires a separate
operator-approved decision naming the behavior, user impact, migration, compatibility period,
and target release. Source deletion, an upstream alternative, or a claim that a feature is
obsolete does not constitute approval.

## 2. Non-negotiable product contract

Released PF Terminal 0.1.26 is the compatibility floor. A feature is preserved only when a user
can still discover and complete its released workflow with compatible inputs, results,
permissions, billing, state, failure behavior, and recovery behavior.

Keeping a source file, enum member, alias, menu label, or stub is not preservation. Replacing an
implementation is allowed only when automated and artifact-level tests prove the complete
released workflow.

The following are release blockers:

1. A released entry point is absent, hidden, unbound, or reaches a placeholder.
2. A released workflow loses a material operation, provider, model, plan, state transition,
   failure guarantee, or recovery path.
3. PF state is deleted, rewritten incompatibly, read from the wrong home, or collided with stock
   Codex state.
4. An explicit model/provider request is silently substituted, including substitution that can
   spend on another route.
5. A PF Terminal screen, prompt, error, help entry, installer, package, or document identifies
   Codex as the product.
6. A conflict is resolved by deleting PF behavior without a compatibility test for the released
   workflow.
7. The candidate comes from an unreconciled dirty tree or is not reproducible from one clean
   commit.
8. A generated manifest, schema, package, or snapshot differs without review and qualification.

## 3. Source-of-truth hierarchy

Every convergence decision is evaluated against three sources:

1. **PF Terminal 0.1.26** defines the released product contract.
2. **Pinned upstream Codex** defines upstream runtime behavior and fixes.
3. **The 0.1.27 candidate** must combine both without losing the product contract.

When sources disagree:

- adopt upstream runtime mechanics at typed integration boundaries;
- retain PF behavior and user entry points;
- adapt provider, model, billing, capability, persistence, and presentation differences;
- never resolve a semantic conflict by deleting one side wholesale;
- test the general behavior class, not only an incident's literal example;
- record a necessary lasting divergence in `FORK_POLICY.md` with owner, reason, tests, and
  removal condition.

## 4. Upstream behavior to adopt

PF Terminal converges on the pinned upstream implementation for:

- turn execution and completion;
- streaming and retry mechanics;
- context projection and compaction;
- tool protocols and tool loops;
- permissions and approvals;
- app-server protocol evolution;
- native agent lifecycle, mailbox, wait, resume, cancel, and close mechanics;
- terminal rendering and input mechanics;
- upstream security and correctness fixes.

Adoption is not blind copying. Upstream behavior passes through PF's typed route, capability,
billing, state, and presentation boundaries. Upstream lacks authority to remove PF functionality
for which it has no equivalent.

## 5. PF behavior to preserve

The generated 0.1.26 feature manifest is authoritative. This section is an explicit minimum,
not a replacement for that manifest.

### 5.1 Entry points

- `pfterminal` and `pfterminal-debug` binaries;
- PF stable and debug homes, isolated from each other and from stock Codex;
- released CLI help, aliases, flags, environment behavior, updates, and exit behavior;
- `/model`, `/providers`, `/vault`, `/wallet`, and `/gpu`;
- `/agent`, `/spawn`, `/orchestrate`, and `/panes`;
- `/docs`, including the viewer and targeted pages;
- `/telegram`, `/tasknode`, `/goal`, `/skills`, `/permissions`, and `/usage`;
- every other released command discovered by the feature manifest.

### 5.2 Providers, models, plans, and billing

- OpenAI, Anthropic/Claude, PF plans, Kimi, GLM, Grok, OpenRouter, Vercel, Z.AI, and supported
  custom/local routes;
- released model records, aliases, context/output limits, reasoning options, visibility, plan
  eligibility, and provider bindings;
- Opus, Fable, Kimi K3, GLM 5.2, Grok, DeepSeek, and distinct Sol, Terra, and Luna tiers;
- GPT-5.5 remains ineligible for automatic selection;
- exact provider/model selection with transactional switching and no silent paid fallback;
- provider authentication, API-key, plan-login, entitlement, service-tier, and billing modes.

Route identity is structured data containing at least provider, exact provider model ID, wire
protocol, authentication mode, billing mode, effort, service tier, capabilities, and accepted
reported aliases. Routing, capability, and billing policy must not come from model-name
substrings or one-off regexes.

An explicit route either runs exactly or fails before inference and spend. A model switch commits
only after provider acknowledgement; on failure, displayed, persisted, sampling, and compaction
routes remain unchanged.

### 5.3 Orchestration and agents

- model-aware `/spawn`, `/orchestrate`, `/agent`, and `/panes` workflows;
- exact child routes and conservative typed automatic selection;
- durable child identity, route, capabilities, vision, plan/billing source, and selection reason;
- nested spawn, mailbox, wait, resume, cancel, close, inspect, and result delivery;
- visible accounting and warnings for separately billable child work;
- released pane inspection and control behavior.

These workflows may share one upstream-native controller, registry, mailbox, and lifecycle. That
internal consolidation is valid only if each released entry point and workflow remains complete.

### 5.4 Wallet, vault, GPU, Telegram, and Task Node

- wallet create, import, backup, unlock, balance, plan purchase, receipt, and recovery;
- vault credential storage and lookup without raw-secret disclosure;
- GPU catalogue, authorization, launch, persistence, inspect, and termination;
- the sole selectable DeepSeek rental recipe
  `deepseek-ai/DeepSeek-V4-Flash-0731` at its pinned revision;
- Telegram setup, control, exact-route execution, permissions, and resume;
- Task Node setup, commands, authentication, and state continuity;
- required services, including the wallet daemon.

### 5.5 State, distribution, and identity

- byte-compatible released migrations through PF migration 0045;
- migration and resume from representative 0.1.24, 0.1.25, and 0.1.26 homes;
- no test mutation of production homes;
- Linux, macOS, and Windows packages, installers, updates, checksums, upgrades, rollback, and
  platform launch behavior;
- PF Terminal names, paths, banners, confirmations, errors, help, snapshots, docs, and packages.

`Codex` may be user-visible only for factual upstream attribution or an exact external
product/protocol name. Internal crate and protocol identifiers can remain when users cannot see
them and renaming harms compatibility. They must map to PF Terminal language at the UI boundary.

## 6. Integration architecture

Repair boundaries instead of accumulating special cases:

- **Catalogue:** one typed catalogue owns routes, capabilities, context, output, reasoning, plan
  eligibility, billing, and accepted aliases.
- **Selection:** explicit selections are exact; automatic selection is structured,
  capability-aware, billing-aware, explainable, and conservative.
- **Provider:** provider-specific wire behavior adapts without forking the general turn loop.
- **State:** released PF namespaces and migrations remain stable; upstream improvements begin
  after that compatibility floor.
- **Presentation:** internal identifiers translate to PF Terminal product language.
- **Agents:** PF command surfaces use the native lifecycle while retaining PF orchestration
  semantics and durable identity.
- **Failure:** entitlement, payment, authentication, invalid-route, and transient failures stay
  typed. Non-retryable failures must not repeat paid work.

When semantic intent selects a natural-language workflow, use a small structured classifier with
deterministic outputs and a conservative fallback. Regex and literals are only defensive checks
for mechanical protocol markers.

## 7. Reconstruction procedure

Release work occurs on `release/0.1.27-reconstruction` in the clean reconstruction worktree. The
damaged integration tree is evidence and salvage material, not a release source.

Preserved incident state:

- dirty salvage branch: `quarantine/0.1.27-dirty-salvage-20260801T0245Z`;
- dirty salvage commit: `843216c17740cab0ecf7908139c855e9480b65c7`;
- clean reconstruction base: `45a60f03d2f6c041d284b41cc3f33c416d9eeed1`.

Rules:

1. Never merge, reset onto, or copy the dirty salvage tree wholesale.
2. Compare released PF, pinned upstream, clean base, and salvage at each failed boundary.
3. Port small coherent slices whose commit names the protected feature and upstream need.
4. Add generalized regression tests in the same slice.
5. Compile and test a slice before dependent work.
6. Do not push, tag, publish, alter installed stable pointers, spend on paid provider tests, or
   launch paid GPU work without explicit operator authorization.

Implementation order:

1. Restore a compiling workspace and reconcile protocol, API, state, and app-server boundaries.
2. Restore catalogue, provider, authentication, plan, vault, wallet, and GPU boundaries.
3. Converge orchestration onto the native lifecycle while preserving PF workflows.
4. Restore commands, panes, docs, presentation, and branding.
5. Qualify migrations, home isolation, installers, packaging, and exact artifacts.

## 8. Required automated proof

### 8.1 Product-contract guardrail

Compare machine-readable feature manifests for 0.1.26 and the candidate, covering:

- binaries, CLI commands, flags, and exit behavior;
- slash commands, aliases, descriptions, availability, and dispatch;
- providers, models, routes, capabilities, plans, auth, and billing;
- configuration keys, defaults, profiles, and schemas;
- app-server methods and generated protocol types;
- databases, migration hashes, homes, services, platform artifacts, protected implementation
  paths, and the source path inventory.

The comparison fails on missing or changed released contract items. An intentional structural
difference needs an allowlist entry linked to an existing acceptance test; prose cannot waive it.

### 8.2 Workflow tests

Automated tests prove:

- every protected CLI and slash entry dispatches to working behavior;
- model switches are exact and transactional;
- paid routes are not substituted or retried after non-retryable payment failures;
- capabilities, context, output, reasoning, and billing come from typed records;
- child identity and route survive nesting, compaction, resume, cancellation, and reopen;
- `/panes` retains inspect/control behavior on the native agent implementation;
- wallet, vault, GPU, Telegram, and Task Node workflows remain complete;
- released configs and copied homes migrate and resume without collision or mutation;
- stable PF, debug PF, and stock Codex homes cannot mutate one another;
- branding audits cover static and runtime-composed strings;
- long and image-heavy histories compact without corrupting canonical state;
- retry, overload, 413, interruption, compaction, and switching cannot loop infinitely or
  duplicate paid work.

Semantic failures require adjacent cases and paraphrases, not only the incident's literal input.

### 8.3 Repository gates

- run changed-crate tests with `just test -p <crate>`;
- run complete TUI, core, state, API, app-server, protocol, CLI, catalogue/provider, wallet,
  vault, GPU, Telegram, installer, and packaging suites;
- regenerate schemas and prove a clean second generation;
- review every changed snapshot and golden output;
- run scoped `just fix -p <crate>` and final `just fmt` under repository rules;
- run full-workspace `just test` only with operator approval;
- require `git diff --check`, no pending snapshots, and a zero-hit secret scan.

## 9. Exact artifact qualification

Source tests do not qualify a release. One immutable clean candidate commit must produce
`pfterminal` and `pfterminal-debug`, with source, locks, toolchain, build command, and hashes.

The exact artifacts must pass:

1. startup identity, CLI help, and clean exit;
2. every protected PF command group;
3. provider setup and exact model switching;
4. spawn, orchestrate, agent inspection, panes, and docs;
5. wallet, vault, GPU, Telegram, and Task Node smokes;
6. clean installation on each shipped platform;
7. upgrade from copied released state and representative session resume;
8. documented rollback with recoverable user data;
9. checksum/signature validation and secret scanning.

Any source or dependency change creates a new candidate and invalidates earlier artifact evidence.
An older local binary cannot qualify a rebuilt artifact.

## 10. Release decision

PF Terminal 0.1.27 remains **NO-GO** until:

- the feature comparison has no unexplained regression;
- all protected workflows pass automated and exact-artifact tests;
- branding has zero unclassified user-visible violations;
- migration, resume, isolation, install, upgrade, and rollback evidence is complete;
- schemas and packages reproduce from one clean commit;
- logs, snapshot review, hashes, and classifications live under `qa/release/0.1.27/`;
- the operator approves promotion of the qualified candidate.

Known blockers include released-contract differences in binaries, CLI, configuration,
app-server methods, slash behavior, and the model catalogue, plus compilation failures across
convergence boundaries. These are implementation failures to repair, not reasons to shrink the
product contract.

No P0 may be deferred. Publication, stable promotion, paid provider campaigns, GPU rental, push,
tag, and release creation are separate operator-approved actions.

## 11. Definition of done

The work is complete only when a user upgrading from PF Terminal 0.1.26 receives the updated
Codex runtime without losing PF behavior, data, routes, identity, or control—and the exact
installable artifacts prove it.

> Codex is upgraded underneath PF Terminal; PF Terminal is not transformed into Codex.
