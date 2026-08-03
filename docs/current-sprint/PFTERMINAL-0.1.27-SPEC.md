# PF Terminal 0.1.27 Canonical Product-Preserving Codex Convergence Spec

Date: 2026-08-01

Native-agent architecture decision updated: 2026-08-02

Implementation status: **P0 product-preservation repair in progress — `/spawn` is being refactored
onto the OpenAI Codex native agent framework; the PFTQA-013 cold-crew projection defect is repaired
and live-verified, operator-created Claude-pane deletion is live-verified through PFTQA-020,
ownership-aware whole-crew removal is live-verified through PFTQA-021, and the complete feature
matrix remains a release blocker; wallet account handling is partially live-qualified and the
PFTQA-022 destructive-confirmation defect is repaired and live-verified; vault handling is
partially live-qualified after the PFTQA-023 secure-action and truthful-completion repair; and
provider qualification is partial after the PFTQA-024 catalogue/harness repair; and PFTQA-025's
OpenAI reserved-schema and encrypted-follow-up defects are repaired and live-verified, and
PFTQA-026's interrupt/reuse and cold-runtime restoration defects are repaired and live-verified,
and PFTQA-027's rapid slash-command admission defect is repaired and live-verified while the
retained Angmar/Burzum/Snaga hierarchy has completed direct and multi-level native follow-up on
the current artifact; PFTQA-028 has additionally live-qualified fresh manual DeepSeek/YOLO role
creation, direct input, multi-level reuse, and cold restoration; the complete `/spawn` matrix
remains open; PFTQA-029 has additionally live-qualified standard-crew creation, Claude Plan
Nazgul delegation, OpenAI-native Troll-to-Orc delivery, the typed OpenAI-to-OpenRouter plaintext
adapter, exact result consolidation, and retained member reuse; PFTQA-030 has repaired and
live-qualified pane-local provider projection for retained Claude Plan and OpenRouter members;
PFTQA-031 has repaired permission-contract restoration and live-qualified workspace-write/never
inheritance across Main, Nazgul, Troll, Orc, managed follow-up, and cold reuse of the same root and
Orc identities; PFTQA-032 has repaired explicit permission propagation to retained native members
and stale dynamic-context replay on non-OpenAI provider wires, and has live-qualified the
workspace-write/on-request slice across the same retained DeepSeek hierarchy, native managed
follow-up, interactive denial, and flagless cold restoration; PFTQA-033 has live-qualified the
read-only/on-request slice across every retained role, native managed follow-up, rejected
workspace writes, and flagless cold restoration; PFTQA-034 has repaired provider-valid helper
model routing and live-qualified DeepSeek automatic approval review plus the Full Access
cancel/confirm, direct-execution, and flagless-restoration workflow on the exact retained Orc;
PFTQA-035 has closed the exact literal `--yolo` fresh-user-pane compound-command cell without
approval or bubblewrap and restored the retained home to Workspace/Ask afterward;
the complete feature matrix and remaining `/spawn` and `/status` matrices remain open**

Release status: **OPERATOR-AUTHORIZED RELEASE CANDIDATE — residual matrix gaps remain documented;
final packaging and GitHub publication are the remaining shipment gates**

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

For `/spawn`, that objective specifically means refactoring PF Terminal onto the pinned OpenAI
Codex Core native multi-agent implementation already used by Codex subagents. Core is the sole
authority for agent threads, parent/child paths, message delivery, follow-up work, interruption,
waiting, restoration, and explicit closure. PF Terminal remains the authority for the released
Nazgul/Troll/Orc product: role prompts and delegation doctrine, crew composition, provider/model
and effort selection, permissions, billing disclosure, panes, commands, direct human input, and
presentation. This document uses “OpenAI agent framework” as shorthand for that pinned Codex Core
implementation; it does not mean the separate OpenAI Agents SDK.

The refactor is accepted only when a completed role remains alive and reusable on the same Core
thread and canonical path, with its persistent PF role prompt intact. Task completion must never
implicitly close, discard, bench, replace, or make a Nazgul, Troll, or Orc read-only. A provider
adapter may change the wire representation required by a non-OpenAI endpoint, but it may not
introduce a second graph, queue, mailbox, lifecycle, or prompt-derived routing system.

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

These workflows must use the pinned OpenAI Codex Core native agent framework—the same subagent
graph and collaboration boundary used by Codex `spawn_agent`, `send_message`, `followup_task`,
`wait_agent`, and related controls—as their sole native controller, registry, thread graph,
mailbox, and lifecycle. PFTerminal preserves the released product workflow by configuring that
framework and projecting it into `/spawn`, `/agent`, `/subagents`, and `/panes`; it must not
implement a competing native-agent runtime beside it.

### 5.4 Wallet, vault, GPU, Telegram, and Task Node

- wallet create, restore, backup, lock/unlock, SOL/USDC balance, exact plan purchase, receipt, and
  plan recovery;
- vault credential storage and lookup without raw-secret disclosure;
- GPU catalogue, authorization, launch, persistence, inspect, and termination;
- the sole selectable DeepSeek rental variant, the qualified TP2 recipe for
  `deepseek-ai/DeepSeek-V4-Flash-0731` at its pinned immutable revision;
- other released qualified GPU recipes, including GLM, plus preserved experimental recipes that
  remain explicitly labelled experimental;
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
- **Agents:** PF command surfaces configure and project the OpenAI Codex native agent framework
  while retaining PF role doctrine, orchestration semantics, exact routes, and durable identity.
- **Failure:** entitlement, payment, authentication, invalid-route, and transient failures stay
  typed. Non-retryable failures must not repeat paid work.

When semantic intent selects a natural-language workflow, use a small structured classifier with
deterministic outputs and a conservative fallback. Regex and literals are only defensive checks
for mechanical protocol markers.

### 6.1 Native `/spawn` architecture

`/spawn` must be refactored onto the pinned OpenAI Codex Core native multi-agent framework; this is
not an adapter layered over the old PFTerminal orchestration runtime and not a new PF imitation of
the upstream APIs. The same Core framework used by Codex subagents is the authority for every
native `/spawn` agent. “Native” means a Codex thread registered in the Core agent graph, addressed
by its Core `ThreadId` and canonical `AgentPath`, and operated through the Core collaboration APIs.
Once equivalent state has been migrated, legacy TUI-owned native dispatch, polling, report
parsing, and lifecycle paths must be removed rather than retained as a second fallback runtime.
In this specification, “OpenAI agent framework” means this pinned Codex Core agent graph and
collaboration implementation; it does not mean a new parallel runtime or an unrelated SDK.

The refactor boundary is therefore explicit: OpenAI Codex Core supplies the agent runtime, while
PFTerminal supplies the product semantics. Adopting Core does not authorize replacing a Nazgul,
Troll, or Orc with a generic subagent, weakening its role prompt, disabling direct pane input, or
adopting an upstream task-worker lifecycle in which completion implies disposal. A PFTerminal crew
member is a durable, human-addressable Core agent that remains present after each turn and accepts
later direct input or native follow-up work until the user explicitly confirms its removal.

The implementation mapping is normative:

| PF Terminal concept | OpenAI Codex Core authority | Required PF behavior |
| --- | --- | --- |
| Human-addressable Nazgul, Troll, or Orc | Native thread classified as `AgentClass::CrewMember { human_addressable: true, ... }` | The pane accepts direct user input; parent control must not disable the composer. |
| Crew/member identity and hierarchy | Core `ThreadId`, canonical `AgentPath`, agent graph, and durable crew/member metadata | Every PF view projects the same identity and parent edge; display names are not routing keys. |
| First assignment | Core native spawn with typed initial input | The literal task is admitted once through Core, never through a parallel TUI queue. |
| Input while a member is running | Core message/steering path (`send_message`) | Add context to the current turn without starting a duplicate turn. |
| New work after a turn is done | Core follow-up path (`followup_task`) | Reuse the same thread, path, role, route, and pane for a new turn. |
| Child result | Core terminal-result `AgentMessage` delivered to the parent | Deliver once, durably; do not infer completion or routing from model prose. |
| Finished current turn | Core `AgentStatus::Completed` | Keep the member registered, visible, selectable, directly addressable, and reusable. |
| Process residency | Core loaded/unloaded lifecycle | Unloading may release process resources but must preserve durable identity and allow Core resume. |
| Permanent removal | Explicit Core close/shutdown invoked by a human-confirmed, ownership-aware PF removal workflow | Never occur merely because a task completed, errored, was interrupted, a parent finished, or the TUI restarted. |

`AgentStatus::Completed`, `Interrupted`, `Errored`, and `Unloaded` are recoverable member states, not
deletion signals. Only an explicit, authorized close/removal operation may transition a retained
crew member to `Shutdown` and remove its durable graph/layout membership. A parent model may assign,
message, interrupt, wait for, and reuse a child, but it may not autonomously convert task completion
into permanent removal. A normal terminal answer
must transition the current turn to completed while leaving the member available to both direct
pane input and native follow-up work.

Core owns:

- thread creation, registration, parent/child graph, canonical path, status, and capacity;
- mailbox admission, message identity, deduplication, ordering, wake-up, and follow-up turns;
- wait, interrupt, resume, completion, explicit close, and terminal-result delivery;
- durable graph restoration and the distinction between a completed reusable member and a closed
  or unloaded agent.

PFTerminal owns only the product layer above Core:

- `CrewSpec`, standard/custom crew composition, Nazgul/Troll/Orc role selection, names, and icons;
- the persistent built-in Nazgul, Troll, and Orc base prompts and their command doctrine;
- exact provider/model/effort selection, credentials, billing disclosure, and spend attribution;
- `/spawn`, `/agent`, `/subagents`, and `/panes` presentation, navigation, task entry, and status
  projection;
- fresh per-turn application context containing only live roster, canonical paths, routing state,
  and delivered-report state.

The required end-to-end flow is:

1. `/spawn` gathers the PF role, name, hierarchy, provider, model, effort, permission, and billing
   choices and submits one typed native spawn request to Core.
2. Core creates and registers the member through the same `AgentControl`/agent-graph boundary used
   by OpenAI Codex subagents. PF crew/member identity is durable metadata on that native thread.
3. New work for an idle or completed member enters its Core mailbox through native follow-up;
   information for a running turn enters through native message delivery. No TUI queue starts a
   provider turn.
4. Core delivers the member's terminal result to its parent exactly once and leaves the member
   reusable. Turn completion is not agent shutdown.
5. `/spawn`, `/agent`, `/subagents`, and `/panes` read and control that same Core graph. Persisted PF
   layout augments presentation but cannot claim a member is addressable until Core has restored
   or materialized its thread.

Startup and restoration are a read-before-write reconciliation boundary. When resuming an owner
thread, PFTerminal must load the existing PF layout, restore or materialize its referenced Core
threads, reconcile identities and edges against the Core graph, and only then publish a new layout
snapshot. Startup must never persist an empty/default in-memory layout over an existing owner
layout before that reconciliation completes. A missing, malformed, incomplete, or temporarily
unavailable Core member must preserve the last valid layout and enter an explicit degraded state;
it must not erase the crew, clear parent edges/endpoints, bind Main as a replacement Nazgul, or
overwrite both current and recovery snapshots with the degraded projection.

Nazgul, Troll, and Orc behavior remains a PF Terminal product contract during this refactor. Each
built-in base prompt is applied as persistent role/developer instruction when Core creates or
restores that member. It is not copied into each assignment, replaced with a generic worker prompt,
or weakened by live roster injection. A task payload contains the task; live application context
contains current hierarchy and routing facts; persistent role doctrine contains the role. An agent
that finishes a task stays registered under the same identity and keeps its role prompt for later
`followup_task` work until an explicit, human-confirmed removal operation.

Prompt composition has exactly three independently testable layers:

1. **Persistent role layer:** the released Nazgul, Troll, or Orc base prompt and role-specific
   delegation doctrine, installed through Core role configuration and retained across turns.
2. **Live application-context layer:** current crew roster, canonical paths, exact routes,
   permissions, and report-delivery state; regenerated from typed state and never used to replace
   the persistent role layer.
3. **Assignment layer:** the user's or parent's actual task, delivered once as native mailbox input
   without a synthetic role restatement, regex-derived routing instructions, or taskless
   `Continue.` message.

Tests must inspect these layers before provider serialization and at the provider boundary. A
successful-looking answer is insufficient if the task was duplicated, role doctrine was omitted
or injected into the task, live context was stale, or a non-OpenAI provider received a taskless
continuation instead of the assignment.

The following are prohibited native-agent implementations:

1. A TUI-owned second task queue, lifecycle state machine, or authoritative agent registry.
2. Parsing model prose, XML tags, names, or regexes as the primary native dispatch mechanism.
3. Falling back from a failed Core mailbox operation to direct `turn/start` task admission.
4. Repeating or overriding role doctrine inside individual task payloads or dynamic roster context.
5. Treating turn completion as shutdown, unloading, benching, replacement, or loss of identity.
6. Spawning a duplicate because an existing listed crew member is idle or completed.

`/spawn` creates native members through the Core spawn boundary with `AgentClass::CrewMember` and a
stable crew/member identity. UI task submission and manager delegation use the same Core message
bus: a new assignment to an idle or completed member uses the native follow-up path; information
for a running member uses the native message path. Child completion arrives as one stable Core
terminal-result message. The completed member remains registered and accepts later work on the
same thread and canonical path until an explicit, human-confirmed removal operation occurs. Parent
completion, parent acknowledgement, process exit, compaction, and cold restoration are not removal
authorization.

Core also owns the native manager auto-processing loop breaker. A cold or newly restored manager
must queue terminal results without starting a paid turn until it receives fresh operator or
assignment input. Once active, a manager may automatically process at most three consecutive
terminal-result-only turns that each successfully dispatch more crew work. The next result remains
durably queued on the same manager without waking another paid turn. A terminal-result turn that
acknowledges or consolidates without dispatch ends the chain, and fresh operator or assignment
input resets it. This policy is keyed by Core `ThreadId`, counts successful native collaboration
actions rather than model prose, and never closes, replaces, or unloads the manager or worker.
External headless panes use the same shared protocol ceiling only in their isolated edge adapter;
they do not govern native agents.

Core history and mailbox persistence retain native typed `AgentMessage` items. At the provider
boundary, OpenAI transports that native item directly; Responses-compatible providers that do not
implement the OpenAI collaboration item receive the same plaintext assignment through the normal
external-input/user role. This is a wire adaptation only: it must not create a second mailbox,
rewrite the durable Core graph, replace the assignment with a synthetic `Continue.`, or require the
recipient model to rediscover its task by calling agent-listing tools.

The first-party OpenAI Responses route also owns a reserved `collaboration.*` tool protocol. For
OpenAI, PFTerminal must emit the pinned upstream definitions of `spawn_agent`, `send_message`,
`followup_task`, `wait_agent`, `interrupt_agent`, and `list_agents` exactly, including encrypted
argument annotations, required fields, descriptions, and output schemas where the upstream
contract defines them. PF-specific provider selection, billing, role identity, hierarchy, and
presentation metadata belong in typed Core/PF state around that protocol; they must not be added
to, removed from, or used to reinterpret a reserved OpenAI function schema. A first-party request
that receives `reserved for use by this model and must match the configured schema` is a local
protocol failure and a release blocker, not a provider-authentication or model failure.

Non-OpenAI providers may receive a capability-specific serialization of the same Core operation
when they do not implement OpenAI's encrypted argument annotations or native collaboration item.
That adapter may expose plaintext assignment content and provider-compatible function schemas, but
it must preserve the operation identity, target, stable message identity, and trigger semantics.
It cannot introduce another registry, queue, lifecycle, role prompt, routing policy, or completion
path. Provider adaptation is selected from typed provider capabilities, never from a model-name
regex or a one-off exception for an observed prompt.

An OpenAI model-issued collaboration call requires an additional typed transport boundary when its
target uses a provider that cannot consume OpenAI's encrypted assignment field. Core must resolve
the target thread and provider before mailbox admission. The reserved OpenAI call then fails closed
without admitting a message or starting the target turn and directs the model to the corresponding
ordinary function—`spawn_agent_plaintext`, `send_message_plaintext`, or
`followup_task_plaintext`. That adapter invokes the same Core handler, graph, mailbox, message
identity, and trigger semantics with a provider-neutral plaintext assignment. It is exposed in
normal and Code Mode Only OpenAI turns, rejects OpenAI targets that support the native encrypted
call, and redacts its plaintext arguments from tool logging. OpenAI ciphertext must never be stored
as third-party task text, and an adapter retry must never create a second graph, thread, or task.

`/panes` is a projection and control surface over Core state, never the source of native lifecycle
truth. Persisted PF layout may retain display preferences and crew presentation metadata, but
restoration must reconcile against the Core thread graph and reopen through Core before accepting
work. Layout persistence must be atomic and must remain gated until the initial owner-layout/Core
reconciliation succeeds. A cold target is materialized or resumed in Core and the same mailbox
message is retried; the TUI may not bypass Core.

Headless external Claude panes may retain a clearly isolated edge adapter where their external
runtime cannot participate in the Core agent protocol. That compatibility adapter must terminate
at the Core mailbox boundary when communicating with a native member and must never determine the
behavior of native Nazgul, Troll, or Orc agents.

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
3. Refactor native orchestration onto the OpenAI Codex agent framework as the single runtime and
   message bus while preserving PF crews, role prompts, routes, panes, and workflows.
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

The earlier compilation and primary PF workflow failures have been repaired in the reconstruction
worktree. On 2026-08-03 the operator explicitly authorized packaging, push, and release after live
Task Node recovery, PFTerminal integration, and a successful end-to-end `/spawn` orchestration run.
That decision accepts the residual matrix gaps recorded below; it does not convert unchecked cells
to PASS or shrink the product contract. Shipment still requires one immutable candidate commit,
successful release-mode package smokes, and the repository's cross-platform release workflow.

No P0 may be deferred. Publication, stable promotion, paid provider campaigns, GPU rental, push,
tag, and release creation are separate operator-approved actions.

## 11. Definition of done

The work is complete only when a user upgrading from PF Terminal 0.1.26 receives the updated
Codex runtime without losing PF behavior, data, routes, identity, or control—and the exact
installable artifacts prove it.

> Codex is upgraded underneath PF Terminal; PF Terminal is not transformed into Codex.

## 12. 2026-08-01 reconstruction execution ledger

This ledger records the current worktree evidence. It does not promote, publish, tag, push, or
release anything, and it does not replace the exact clean-commit artifact gate in section 9.

### 12.1 Reported implementation changes (not hands-on qualified)

The entries below describe implementation work and automated evidence available at the time. They
must not be read as feature acceptance; section 13 controls qualification.

- Upstream turn, compaction, provider, catalogue, configuration, state, app-server protocol, and
  TUI changes were reconciled without removing PF command surfaces.
- Native `/spawn` task admission now uses the OpenAI Codex agent framework as its runtime
  authority: Core mailbox delivery, canonical agent paths, stable message identities, and reusable
  `CrewMember` lifecycle. The TUI submits typed `ThreadAgentMessage` assignments and, for a cold
  target, asks Core to materialize the saved member before retrying the same stable mailbox
  message. It does not admit native `/spawn` work through direct `turn/start` or a second TUI task
  queue. This is implementation evidence only; `/spawn` remains unaccepted until the complete
  section 13.6 hands-on matrix passes.
- The active-process native manager loop passed a paid DeepSeek hierarchy exercise after its
  lifecycle accounting moved to Core task admission: three consecutive terminal-result turns
  dispatched follow-up work, the fourth result remained queued without another paid turn, fresh
  input resumed the same Troll and Orc identities, and acknowledgement ended the chain.
- Cold restoration first exposed a native graph defect: `/spawn` could persist a child
  edge beneath a lazy Main thread whose rollout had never materialized. Core now materializes and
  flushes a durable parent and child before publishing their graph edge; the TUI no longer writes
  that edge directly. Manual native hierarchy selection now makes the bound Nazgul the actual Core
  parent of its Troll. In a fresh no-task qualification, Main, Angmar, Burzum, and Snaga all had
  durable rollouts and the Core edges were Main -> Angmar -> Burzum -> Snaga before any model turn.
  One full-restart qualification made the same panes searchable/addressable; Snaga's result reached
  cold Burzum with `trigger_turn:false`, Burzum started no paid turn, and fresh input on the same
  Burzum processed the queued result and acknowledged without dispatch. That historical
  sub-workflow was superseded by the later PFTQA-013 startup-layout failure and its repaired
  qualification below; it is not independent evidence for the broader section 13.6 matrix.
- Fresh-home onboarding now reaches Main after a real DeepSeek provider selection and saved-key
  handoff without duplicating provider saves or leaving the old onboarding event receiver active.
  This is a qualified onboarding sub-workflow only; the complete provider and authentication
  matrix remains open.
- Operator-created PF user panes now persist as explicit layout members separate from managed
  crew and parent-controlled task-only workers. A full process exit restored DeepSeek/high/YOLO
  pane `Cold Restore QA` on the same child `ThreadId`, replayed its transcript, accepted direct
  input, and ran another real shell tool turn. `/agent`, `/subagents`, and `/panes` agreed on that
  child. The original empty Main was materialized under a different Main identity on explicit
  resume, so exact Main restoration and the broader pane/provider/active-turn/close matrix remain
  unaccepted.
- `/docs` now keeps discovery and search bounded to the known MkDocs page set, follows internal
  relative/site-local links and generated/explicit/HTML anchors, and retains back/forward history.
  Async errors schedule an immediate redraw. The npm package stages the existing `mkdocs.yml` and
  `docs/` tree, and a live staged-package run from `/tmp` loaded that offline copy through managed
  package provenance. Active-turn/all-pane use, every exit key/terminal-mode combination, and a
  final packaged artifact matrix remain open.
- A paid active-turn follow-up opened `/docs` while a real background shell tool was running and
  later received `DOCS_ACTIVE_OK`. A second turn opened `/panes` while its shell tool was running;
  the picker identified Main as `running`, and the untouched turn later received
  `PANES_ACTIVE_OK`. This qualifies active Main only, not every pane kind.
- A later PF user-pane matrix found that both Main and an active native child were labelled
  `(current)`: the external-pane registry deliberately maps every native Core thread to
  `codex-main`, so it was not sufficient current-thread authority. `/panes` now compares the active
  Core `ThreadId` for native rows. A focused regression and rebuilt live binary showed exactly one
  current row across Main and three native user panes. While one user pane ran a real 45-second
  shell tool, the operator switched to an idle sibling and back; the running row remained visible
  and the original thread returned exact `PF_USER_ACTIVE_OK` without losing its transcript.
- The same cold matrix exposed a remaining P0 owner-layout defect: successful TUI `thread/start`
  returned a valid ID but kept persistent Main and operator-pane threads lazy until their first
  model turn. A layout could therefore reference an ID with no rollout, and exact resume created a
  different Main. Persistent TUI threads now materialize and flush at the app-server start
  boundary; non-TUI app-server clients retain the upstream lazy behavior. In a fresh zero-turn
  live retest, Main `019fc0f3-45e5-7b43-a1cd-5af1c61f0414` and `No Task Durable`
  `019fc0f4-26ff-7ff3-b1eb-300582ee4e2e` each had a rollout before exit. Exact Main resume restored
  the same owner layout and the same zero-token child session.
- **PFTQA-013 — CLOSED, `/spawn` startup restoration destroyed the saved crew projection.** On
  candidate binary SHA-256
  `d27a927644defa57a16786506e11cc0628948d176db5fea910f66601fa9c887d`, resuming Main
  `019fc084-96de-7f42-b35d-a9e7b7f0cce4` from a copied isolated evidence home overwrote a complete
  Angmar -> Burzum -> Snaga layout during startup, before `/panes` was opened. The resulting layout
  had no `CrewSpec`, no Nazgul binding, no parent edges, and no endpoints; `/panes` showed only Main
  and implicitly treated it as the root. Both the current and recovery layout snapshots in the
  disposable clone were replaced. The four durable Core rollouts still existed, so this is a
  generalized initialization/reconciliation and persistence-order defect, not permission to
  synthesize missing agents or hard-code the observed IDs. The reader now verifies the checksum of
  the raw saved JSON before applying schema defaults, so adding a defaulted field cannot make a
  valid prior-schema layout appear corrupt. Persistence also fails closed when neither the current
  nor recovery generation verifies instead of replacing both with startup defaults. Finally,
  durable `CrewSpec` role metadata—not transient liveness metadata—is authoritative when projecting
  restored Nazgul, Troll, and Orc members into the command surfaces. On rebuilt debug binary
  `920e4a117714f8d30e7920cd74f6fc2f411c38477c3a461836f560e770a2252a`, a fresh cloned-home
  process resumed exact Main `019fc084-96de-7f42-b35d-a9e7b7f0cce4`, Angmar
  `019fc084-b0bd-7423-8567-690b06a26252`, Burzum
  `019fc084-ef07-7e41-90d0-67c964696084`, and Snaga
  `019fc085-1255-7902-8dfb-39f848acf4e1`. `/spawn`, `/agent`, `/subagents`, and `/panes` agreed on
  the exact canonical paths and hierarchy; direct human input remained available in Troll and Orc
  panes. Both persisted layout generations retained the same complete crew and checksum after
  repeated writes. This closes PFTQA-013 only; the remaining provider, permission, failure,
  lifecycle, and close matrices in section 13.6 keep the overall `/spawn` feature unaccepted.
- **PFTQA-014 — CLOSED, the restored Nazgul root was visible but not searchable in `/panes`.** The
  generated root row omitted picker search metadata while generated Troll, Orc, and Claude rows
  supplied it. The root projection now indexes its role, display title, and durable target ID—the
  same generalized picker boundary used regardless of the observed name. The restored-CrewSpec
  regression asserts that Angmar and its thread ID are indexed. On rebuilt debug binary
  `1f57870a08d3e34e6ccaa9d33b032c21a867a3806e588c56b7fa18d550fd6080`, a cold-resumed
  `/panes` search for `Angmar` returned exactly `Nazgul: Angmar [nazgul]`; selecting it opened the
  existing DeepSeek direct/high/YOLO native pane with normal human input, and `/quit` exited
  cleanly. No model task was submitted for this qualification.
- **PFTQA-015 — CLOSED, one unavailable operator pane repeatedly blocked `/panes`.** Startup
  correctly surfaced a missing-rollout failure, but every later `/panes` invocation automatically
  retried the same failed `thread/resume`; the picker stayed behind the repeated error. A failed
  operator attach is now marked unavailable for the remainder of the process, excluded from
  automatic restore candidates, and retained as a disabled searchable row with an inline reason.
  This is a liveness-state boundary, not a thread-ID exception. In live qualification, startup
  reported the failure once, `/panes` opened, a healthy `Cold Restore QA` sibling remained
  selectable and interactive, and reopening `/panes` did not retry the failure.
- **PFTQA-016 — CLOSED FOR IDLE OPERATOR-PANE ARCHIVE; ACTIVE AND OTHER PANE TYPES PENDING.** A
  successful `/archive` moved an operator pane rollout to `archived_sessions` but exited before
  removing the thread from Main's TUI-owned layout membership. Restart therefore retried the
  intentionally archived session and rendered a ghost disabled pane. Successful archive and
  delete requests now remove only operator-owned pane membership, fall active selection back to
  Main, and persist the owner layout; managed crew and parent-controlled workers remain under
  their separate native lifecycle authority. In a fresh DeepSeek home, zero-token pane `Archive
  Cleanup QA` (`019fc124-28d9-7412-8a59-0ed3e1a1195f`) archived successfully, its transcript file
  remained in `archived_sessions`, its ID disappeared from Main
  `019fc11e-f165-77b0-ba33-51dcd26939d0`'s layout, and a full restart showed Main without that pane.
  The confirmation also says `exit PFTerminal`, not `exit Codex`. Active-turn admission is covered
  separately by PFTQA-017 and operator-pane delete by PFTQA-018; operator-created Claude delete and
  whole-crew removal are defined and closed separately by PFTQA-020 and PFTQA-021.
- **PFTQA-017 — CLOSED FOR ACTIVE OPERATOR-PANE ARCHIVE.** Merely marking `/archive` available
  during a task was insufficient: when terminal input arrived before the slash popup rendered,
  Enter followed the active-turn queue path and stored `/archive` as ordinary follow-up text. The
  composer now treats every recognized slash command that declares active-task availability as an
  immediate control-plane action, independently of popup timing. Startup/configuration input still
  queues, unavailable commands retain the existing rejection path and draft, and unknown
  slash-shaped text retains deferred validation. On debug artifact
  `12d4b1736a3d24c35815b08f886fd316d4b768a1aba6817720fc350b45d97238`, `/archive` opened its
  confirmation while a real 240-second DeepSeek-backed shell tool was active. Confirmation aborted
  the tool after 17.1 seconds, recorded `turn_aborted`, archived thread
  `019fc129-7ad2-7092-ae3d-729962d89f4b`, removed it from Main's operator-pane membership, fell
  selection back to Main, and exited. Cold restart plus exact-ID `/panes` search returned no match;
  the archived rollout contains the user request and tool call but no completed marker or assistant
  completion. Delete is covered separately by PFTQA-018; operator-created Claude delete and
  whole-crew removal are defined and closed separately by PFTQA-020 and PFTQA-021.
- **PFTQA-018 — CLOSED FOR IDLE AND ACTIVE OPERATOR-PANE DELETE.** Idle `/delete` already reached
  the server, but active `/delete` was still hidden and rejected by the command-availability
  policy even though app-server `thread/delete` uses the same active-thread shutdown boundary as
  archive. Delete now declares active-task availability and therefore uses the generalized
  PFTQA-017 control-plane admission instead of a command-specific input route. On debug artifact
  `8a4b5ec474c90120b3fc0c981a64cc46950fa45f647d4581d6c6f915f0971422`, zero-turn `Delete Idle
  QA` (`019fc137-48db-7442-8a4e-9ffaadc2290f`) was permanently removed after confirmation. A
  separate `Delete Active QA` (`019fc13a-335d-7200-bed0-33cbce9c0973`) opened confirmation while
  a real 240-second DeepSeek-backed tool was active; confirmation shut down the thread and tool,
  permanently removed the rollout and state records, removed owner-layout membership, fell back
  to Main, and exited. No 240-second child remained. Cold restart plus exact-ID `/panes` search
  returned no match. This is intentionally irreversible delete evidence; the active transcript is
  absent by product contract. Operator-created Claude delete and whole-crew removal are defined and
  closed separately by PFTQA-020 and PFTQA-021.
- **PFTQA-019 — CLOSED FOR CROSS-OWNER LIFECYCLE SAFETY; REMOVAL FLOWS SUPERSEDED BY PFTQA-020 AND
  PFTQA-021.** When a
  Claude pane is selected, the active native receiver is deliberately detached. The prior
  archive/delete handler then fell back to the chat widget's last native thread—normally Main—and
  could archive or permanently delete that unrelated session after confirmation. A directly
  selected managed `/spawn` member could likewise delete a native subtree without reconciling
  durable `CrewSpec` and pane-layout ownership. Native thread lifecycle commands now accept only
  Main or operator-created native panes. External Claude panes, managed crew members, and
  parent-controlled task workers fail visibly with ownership-specific guidance before any server
  mutation. On debug artifact
  `017a95b6223efa3089b1f99cc3918e2f47f60a12fcdbe3b476cac2733ff92a1f`, a real zero-turn Opus 5
  Claude Plan pane in a disposable cloned home confirmed both `/archive` and `/delete`; each
  returned the Claude-pane ownership error, the TUI remained usable, and Main
  `019fc11e-f165-77b0-ba33-51dcd26939d0` remained in normal `sessions`. No provider inference ran.
  The 93 MiB disposable clone was removed after evidence extraction. Released 0.1.26 contained no
  Claude-pane remove action. PFTQA-020 supersedes this historical limitation for operator-created
  Claude panes, and PFTQA-021 supplies the separate verified whole-crew lifecycle.
- **PFTQA-020 — CLOSED FOR IDLE AND ACTIVE OPERATOR-CREATED CLAUDE-PANE DELETE; WHOLE-CREW REMOVAL
  CLOSED SEPARATELY BY PFTQA-021.** `/delete` now resolves the selected external pane before native
  thread lookup. For an
  operator-created Claude pane it cancels a running turn when present, removes only the verified
  `$PFTERMINAL_HOME/panes/<pane-id>` artifact directory, removes the registry/layout membership,
  falls selection back to Main, persists, and exits. A `spawn_role` guard rejects direct removal of
  independently managed Claude crew members so crew state cannot be partially destroyed. The first
  live active test exposed a separate routing regression: after a native restore failure, selected
  Claude input fell through to `No active thread is available.` External-pane routing now occurs
  before native-thread resolution, independent of whether a native thread is loaded. Five focused
  regressions pass for idle cleanup, active cancellation, app-event deletion, managed-member
  refusal, and external-before-native routing. Live idle deletion on debug artifact
  `4c704d4146fe3010c44c76345bc46c4ce476247ac0aeabe4742e5cb049ba5e36` removed
  `claude-4a99ad7a-bf73-4cce-ba2a-53fa794416b5`; its artifact directory and layout membership were
  absent and a cold-restart name search returned no match. On current debug artifact
  `d43e96dbdf33e41a4dffce60bbb4ef4be60a7bdfe042c6b7f421127f8c966647`, restored Opus 5 Claude Plan
  pane `claude-c8b9ad2d-c970-4ecb-92c0-2bbcf2da1485` accepted a real prompt despite the failed
  native sibling, showed `Claude running`, and opened `/delete` confirmation before completing.
  Confirmation terminated the Claude process, removed the complete pane directory, persisted an
  empty `claude_pane_ids` list with `codex-main` active, exited, and produced no match for `Delete
  Claude Active QA` after cold restart. Main `019fc11e-f165-77b0-ba33-51dcd26939d0` remained the
  owner. Claude `/archive` remains intentionally unsupported and fails at the ownership boundary;
  permanent operator-pane delete is the defined external-pane removal contract.
- **PFTQA-021 — CLOSED FOR EXPLICIT WHOLE-CREW REMOVAL AND EMPTY-STATE PROJECTION.** `/spawn` now
  exposes `Remove managed crew` only for a verified `CrewSpec`, requires a confirmation whose safe
  default keeps the crew, and performs one ownership-aware lifecycle operation. Native members are
  deleted in reverse specification order through the app-server lifecycle boundary; running
  managed Claude members are canceled and their verified pane artifacts removed; durable Core
  edges, `CrewSpec`, layout mappings, runtime/status/dispatch caches, navigation rows, and active
  selection are reconciled together. A user-owned Main or operator pane merely bound as Nazgul is
  preserved. Ordinary completed agents remain durable and reusable: only this explicit confirmed
  lifecycle operation shuts them down. The first disposable-clone attempt failed closed because
  its copied state still named the source home's absolute rollout paths; no member was deleted and
  the `CrewSpec` remained available for retry. After rebasing only those four copied paths, live
  removal deleted native Angmar `019fc084-b0bd-7423-8567-690b06a26252`, Burzum
  `019fc084-ef07-7e41-90d0-67c964696084`, and Snaga
  `019fc085-1255-7902-8dfb-39f848acf4e1` from crew
  `crew-019fc084-b08e-7791-bb0e-15eb9c402c27`. State then contained zero member thread rows, zero
  member graph edges, no member rollouts, and one intact Main row
  `019fc084-96de-7f42-b35d-a9e7b7f0cce4`; the persisted layout had `spawn_crew: null`, no root
  binding, empty spawn maps, and `codex-main` active. Live QA also caught and repaired a projection
  defect where the empty layout synthesized `Nazgul: PFTerminal - Main`. The current candidate now
  renders `No managed crew` in both `/panes` and `/spawn status`, disables the removal action with
  `No managed crew exists`, and retains those results after a full cold restart. A mixed
  native/running-Claude integration test separately proves Claude cancellation/artifact cleanup and
  bound-Main preservation. No provider inference was submitted for this lifecycle test.
- **PFTQA-022 — CLOSED FOR DESTRUCTIVE/BILLABLE CONFIRMATION ADMISSION; WALLET REMAINS PARTIAL.**
  Safety-sensitive confirmations now suppress numeric accelerators, default to Cancel/Back, and
  require explicit navigation plus Enter. The generalized regression proves a digit neither moves
  nor accepts the selection. Exact artifact
  `f850c7a30c501422cadf3070c5270479575c7ba8560559130acbad7eef896ff9` live-qualified cancel,
  digit-preservation, and explicit removal on a zero-balance disposable wallet without inference
  or a transaction.
- **PFTQA-023 — CLOSED FOR VAULT SECURE ACTIONS, ASYNC REDRAW, AND TRUTHFUL MUTATION COMPLETION;
  FULL VAULT GATE REMAINS PARTIAL.** The released `/vault credential reveal|export` surface still
  reached a placeholder, the credential menu lacked reveal/replace/delete, background history did
  not reliably request a frame, and a ten-second timeout wrapped un-cancellable `spawn_blocking`
  mutations. Live delete therefore reported failure and then silently completed later. Vault
  reveal now uses a host-owned transient view that zeroizes on close and never enters transcript or
  composer state. Add and replace submit redacted, zeroizing events and perform storage work off the
  TUI thread. Delete defaults to Cancel, ignores bare digit selection, and awaits the real mutation
  result rather than claiming cancellation while work continues. All local vault reads and copy,
  reveal, add, replace, and delete operations await their actual blocking-task result while leaving
  the TUI event loop responsive. `App::insert_history_cell` explicitly requests a frame, so a
  completed async status/list operation appears without another keypress.

  On debug artifact `efe4f7567291f52d3064fc1d96ca519eda8422e111ca3424c50cc54994ea48ff`,
  an isolated encrypted-file/keyring-fallback vault covered bare menu, status, list, metadata-only
  show, five credential actions, protected reveal, masked replacement, plaintext scans, process
  restart with the same label and updated timestamp, digit/Esc delete cancellation, and explicit
  deletion. That run exposed the false timeout: the UI said delete timed out, then `/vault list`
  proved the credential had been deleted. On final exact artifact
  `b162fff7bb9f108109e4d2a16da7ce86b53a9cdd1ba635320127032359677867`, a fresh masked add and
  confirmed delete produced exactly their real terminal results; delete emitted no timeout and the
  menu immediately showed zero credentials. The QA harness mistakenly entered a second copy of its
  disposable marker after the one-entry add modal had already closed; that text became an ordinary
  prompt and received a 401 from the deliberately invalid DeepSeek placeholder key. The resulting
  history/rollout copies make the zero-leak acceptance row intentionally incomplete even though the
  product's masked modal did not emit the secret. The complete disposable home and tmux session were
  removed. No valid provider inference, paid request, credential, or remote resource was used.
- **PFTQA-024 — CLOSED FOR PROVIDER OUTPUT METADATA AND BENCHMARK TRUTHFULNESS; FULL PROVIDER GATE
  REMAINS PARTIAL.** A real `vercel-anthropic-fast` / `zai/glm-5.2-fast` attempt never reached the
  provider because the selectable model lacked `max_output_tokens`, while the benchmark counted
  the resulting local `task_complete` events and falsely reported two successful turns. The
  bundled standard and Fast GLM 5.2 records now use Vercel's published 128K provider maximum; the
  focused catalogue regression asserts that exact value. The benchmark now resolves stable/debug
  homes with the same entrypoint contract as the product and counts a completed turn only when the
  same turn ID has both a provider-response event and terminal completion. Tool workloads may
  additionally require at least one persisted model function call. Re-evaluating the failed
  rollout produces zero turns and zero calls; a real DeepSeek rollout produces two turns and one
  call.

  On exact debug artifact `c9af50fed3747051eead43c390c784bcc08a83a7260f83ca71a28c34641a5a91`,
  direct Anthropic `claude-opus-5` and OAuth-backed `claude-opus-5-plan` each completed a normal
  response and a real shell-tool turn with exact provider/model events and token/cache accounting.
  On that same current artifact, direct `deepseek-v4-flash` and pinned OpenRouter
  `deepseek/deepseek-v4-flash-0731` each completed the same two-turn workload with exactly one
  persisted function call and no fallback route.
  The rebuilt Vercel route passed local serialization and reached the real Messages endpoint, but
  the only resident credential returned 401 before inference, so Vercel remains unaccepted. Kimi,
  Z.AI, PF Plan, OpenAI, Ambient, Baseten, Meta, other models/capabilities, authentication
  recovery, transactional switching, restart persistence, and billing reconciliation remain open.
- **PFTQA-025 — CLOSED FOR OPENAI RESERVED SCHEMAS AND SAME-CHILD ENCRYPTED FOLLOW-UP; FULL
  `/spawn` GATE REMAINS OPEN.** On
  exact debug artifact `c9af50fed3747051eead43c390c784bcc08a83a7260f83ca71a28c34641a5a91`,
  an isolated authenticated OpenAI Luna probe reached the first-party endpoint but failed before
  inference with HTTP 400: `collaboration.followup_task` is reserved and did not match the
  configured schema. Comparison with pinned upstream identified a generalized provider-boundary
  divergence: PFTerminal removed the upstream encrypted marker from collaboration message fields
  and also extended reserved spawn, interrupt, wait, and listing schemas with PF metadata. Those
  changes may be valid information in PF state and provider-neutral adapters, but they are invalid
  mutations of OpenAI's reserved wire contract. A first repair restored encrypted annotations and
  pinned result shapes; the live endpoint then rejected the still-extended `spawn_agent` schema.
  The final provider-boundary profile exposes only OpenAI's reserved assignment, canonical task
  name, and history-fork fields while PF route, billing, role, and presentation metadata remains in
  typed Core/PF state. Provider-neutral tools retain their plaintext-capable superset.

  The first live retry then found that encrypted OpenAI `followup_task` content was being stored as
  ordinary plaintext ciphertext. Delivery now preserves native encryption only on first-party
  OpenAI and leaves non-OpenAI plaintext delivery unchanged. On rebuilt exact debug artifact
  `3e6c5c3727477403b25102ddcdc3107e80a113db852484fb9da9c3b1aa0960f8`, a normal two-turn Luna
  probe passed, then Luna used native `spawn_agent` and `wait_agent` to create
  `/root/openai_followup_fixed`, received `FIRST_OK`, used native `followup_task` and `wait_agent`
  on that same path, and received `SECOND_OK`. The child rollout retained one thread and recorded
  two distinct completed turns; neither completion shut it down or spawned a replacement. Focused
  schema/profile tests passed 3/3 and provider-wire encryption tests passed 5/5. The complete
  Nazgul/Troll/Orc, direct-input, provider, interrupt, running-turn message, cold-resume, and pane
  matrices in section 13.6 remain release blockers.
- **PFTQA-026 — CLOSED FOR OPENAI NATIVE CONTROL AND COLD-RUNTIME CONTINUITY; FULL `/spawn` GATE
  REMAINS OPEN.** A live Luna control run first proved native `send_message` reached a running child:
  `/root/openai_control_worker` changed its terminal result from `FIRST_UNSTEERED` to
  `FIRST_STEERED`, remained the same thread/path, and accepted later follow-up work. The same run
  then exposed a protocol/runtime mismatch: OpenAI's reserved `interrupt_agent` schema correctly
  supplied only `target`, but the PF handler still required the provider-neutral `reason` field.
  The model's first exact call failed locally and succeeded only after inventing a reason. The
  handler now accepts the exact reserved call and supplies a deterministic truthful audit reason
  outside the wire contract while retaining explicit reason/superseding metadata for
  provider-neutral callers.

  On artifact `4a8a2c341bd6a2a5544ddcf0cbe6a1fa3968ef31a8f88d9f5116150a0d10198f`,
  Luna's first target-only interrupt succeeded, reported the child's prior `running` status, and
  the same child thread `019fc1ea-0c53-77c3-83cc-835ffe318493` then returned
  `RECOVERED_FIXED`; only one child was spawned. A new process next restored the parent and listed
  that child as `unloaded`, but follow-up failed because the SQLite discovery row had no model even
  though the durable child rollout contained exact `thread_settings_applied` and `turn_context`
  runtime records. The cold-load boundary now recovers model, provider, and effort from that
  durable model context when the discovery index is incomplete. The QA harness also records a
  resume baseline so historical turns/function calls can never satisfy a new cold-resume run.

  On rebuilt artifact `45cb31e3822bce7b224ef91c4f807591922ab80daaf9a1933b83bf0b223e7c6d`,
  a second new process resumed exact parent `019fc1ea-0659-7cf1-8d1d-a7106ca5a3ae`, listed the
  existing child as unloaded, and delivered `COLD_FIXED_ONE` and `COLD_FIXED_TWO` through two
  native follow-ups. Both results appended to the original child rollout and retained exact path,
  nickname, provider/model, and thread ID; the parent rollout contains exactly one spawn total.
  Focused regressions cover exact target-only interrupt parsing, provider-neutral interrupt
  metadata, and paginated cold resume with a deliberately missing indexed model. Role/pane and
  remaining provider/permission matrices remain open.
- **PFTQA-027 — CLOSED FOR RAPID SLASH-COMMAND ADMISSION; FULL PANE AND `/spawn` GATES REMAIN
  OPEN.** Live current-artifact role QA first reproduced a generalized input-boundary failure:
  rapidly entered short slash commands could remain wholly inside the paste-burst buffer, so the
  composer could not recognize slash-command context and treated the first Enter as pasted input.
  `/panes` therefore required a second Enter even though the command itself was valid. The repair
  inspects only whether pending burst text begins with the slash protocol marker and materializes
  that pending text before ordinary Enter semantics. It does not inspect command names or add a
  `/panes`- or `/docs`-specific route. A rapid-key-stream regression covers adjacent `/panes` and
  `/docs`, and the focused `burst_enter` set passes.

  On rebuilt artifact `b8d47fe89c99859ef5809c1f5cc5a08db1cdc8a74ad383a4beb59eb8ca818af5`,
  one literal `/panes` burst plus one Enter opened the live picker. One literal `/docs` burst plus
  one Enter immediately dispatched its actionable missing-project error, and
  `/docs --config /home/pfrpc/repos/PfTerminal-0.1.27-reconstruction/mkdocs.yml` plus one Enter
  opened the 50-page terminal viewer. The same current code line was also exercised immediately
  before this rebuild against the retained DeepSeek direct Angmar/Burzum/Snaga crew: each role
  accepted direct human input; Burzum used native `followup_task` on the same Snaga and returned
  `TROLL_MANAGER_OK`; Angmar used the same retained Burzum, which used the same retained Snaga, and
  the chain returned `NAZGUL_ORC_OK`, `NAZGUL_TROLL_OK`, and `NAZGUL_MANAGER_OK`. No role was
  spawned, replaced, closed, or automatically shut down. This is a protocol round trip, not the
  real planning/review workload required by section 13.6, so that broader row remains open.
- **PFTQA-028 — CLOSED FOR FRESH MANUAL DEEPSEEK/YOLO CREATION, DIRECT INPUT, NATIVE REUSE, AND
  COLD RESTORATION; FULL `/spawn` GATE REMAINS OPEN.** Final formatted artifact
  `b8d47fe89c99859ef5809c1f5cc5a08db1cdc8a74ad383a4beb59eb8ca818af5` started from isolated
  home `/tmp/pfterminal-spawn-current-qa.EgZDE8` and created, through the complete manual `/spawn`
  UI with no initial task, Main `019fc1ff-ae00-7ed0-8d75-c193cdafabc8`, Angmar
  `019fc200-298a-73f1-84bc-8fb4a4929715`, Burzum
  `019fc201-0c76-7010-81e8-d9a212dc9ee1`, and Snaga
  `019fc201-bdf5-7c82-94f5-5a0b669c4de8`. Every created role used exact DeepSeek direct
  `deepseek-v4-flash`, high reasoning, and inherited YOLO/Full Access.

  Manual role creation intentionally left control on the initiating pane. QA initially submitted
  three `NEW_*` markers there; those are Main evidence and are not counted as role input. After
  selecting each exact role through `/panes`, its first role-local input returned
  `ACTUAL_NAZGUL_DIRECT_OK`, `ACTUAL_TROLL_DIRECT_OK`, and `ACTUAL_ORC_DIRECT_OK`. Angmar then used
  native follow-up on the same Burzum, Burzum used native follow-up on the same Snaga, and the chain
  returned `FRESH_ORC_OK`, `FRESH_TROLL_OK`, and `FRESH_NAZGUL_OK`. `/spawn status` retained Burzum
  and Snaga as completed and reusable rather than shutting either down.

  After `/quit`, a new process resumed the exact Main. `/spawn status` restored the same Burzum and
  Snaga IDs as idle; `/panes` reopened Snaga's original transcript; Snaga returned
  `FRESH_COLD_ORC_OK`; and `/status` confirmed its original thread ID and Full Access. `/agent` and
  `/subagents` projected the same four-node canonical graph. The isolated QA process was stopped;
  the home and rollouts are retained as release evidence. PFTQA-029 closes standard-crew,
  three-Orc, and mixed-provider retained-follow-up coverage. Real planning/review work, remaining
  permissions, injected failures, active-turn control, and complete close/removal matrices remain
  open.
- **PFTQA-029 — CLOSED FOR STANDARD-CREW MIXED-PROVIDER DELEGATION AND RETAINED REUSE; FULL
  `/spawn` GATE REMAINS OPEN.** A fresh standard crew in isolated home
  `/tmp/pfterminal-standard-crew-qa.zGkqEn` retained Main
  `019fc20e-af8e-70f3-a523-2326a36faaec`, Claude Plan/Fable Nazgul Angmar
  `019fc20e-c5fd-7573-96fb-e66ed4aa0f97`, OpenAI/Sol Troll Burzum
  `019fc20e-c662-75a3-960a-46e9d6737d88`, OpenAI/Luna Orc Snaga
  `019fc20e-c6c8-7cd1-ab9d-371651216c8b`, OpenAI/Terra Orc Ghash
  `019fc20e-c72e-7e32-a8ea-5d9e6b07b593`, and OpenRouter/Grok Orc Krimp
  `019fc20e-c793-7562-9773-56f78d624e2a`. The first live chain exposed two generalized provider
  boundaries: Anthropic Messages cannot serialize Responses namespace tools, and OpenAI's reserved
  encrypted assignment is opaque to an OpenRouter target. The provider capability plan now
  flattens collaboration functions for Anthropic. OpenAI retains the exact reserved native schemas;
  Core resolves a cross-provider target before admission, fails the encrypted call closed, and
  exposes a typed plaintext function adapter backed by the same handler and mailbox. Code Mode Only
  also receives those adapters.

  Exact role-pane input returned `STANDARD_NAZGUL_OK`, `STANDARD_TROLL_SOL_OK`,
  `STANDARD_ORC_LUNA_OK`, `STANDARD_ORC_TERRA_OK`, and `STANDARD_ORC_GROK_OK`; those turns prove
  every standard member remained human-addressable before the management chain.

  On final formatted debug artifact
  `9f7e758a55c526a877f8ef5ba2c8b4c02f9c7779282a14df431404a0d1ac6614`, Angmar issued exactly one
  `followup_task` to the existing Burzum. Burzum's native encrypted calls reached Snaga and Ghash.
  Its native Krimp call returned the fail-closed adapter instruction with no admitted message or
  target turn; Burzum then called `followup_task_plaintext` with the same target and literal
  `Reply exactly ADAPTER2_GROK_OK.` assignment. Krimp's rollout contains that plaintext
  `InterAgentCommunication`, returned `ADAPTER2_GROK_OK`, and completed on the original thread.
  Snaga and Ghash returned `ADAPTER2_LUNA_OK` and `ADAPTER2_TERRA_OK`; Burzum returned
  `ADAPTER2_TROLL_CONSOLIDATED_OK`; Angmar returned `ADAPTER2_NAZGUL_CONSOLIDATED_OK`.
  `/spawn status` showed all five named roles completed and reusable with those exact latest
  results. No thread was spawned, replaced, closed, interrupted, or shut down during the rerun.
  The targeted `multi_agent_v2` test slice passed 87/87; focused cross-provider handler, Code Mode
  Only exposure, tool-log classification, model-plan, and provider-bound request tests also passed.
  Real planning/review, remaining permission/failure modes, active-turn mixed-provider messaging,
  and complete close/removal matrices remain open.
- **PFTQA-030 — CLOSED FOR RETAINED MIXED-PROVIDER PANE PROJECTION; FULL `/status` MATRIX PENDING.**
  In the retained Angmar pane, the header and rollout correctly identified `claude-plan` /
  `claude-fable-5-plan`, while `/status` rendered the owner pane's DeepSeek provider. Actual
  inference remained on Claude Plan, proving a false pane-local UI projection rather than a routing
  fallback. Session binding copied the selected thread's model, permissions, and cwd but omitted its
  provider; a later settings event copied only the provider ID, leaving the parent's provider
  metadata, wire API, auth flags, and runtime URL in the widget. Provider projection is now atomic:
  retained-session binding, thread-settings updates, and interactive model selection all install
  the complete provider snapshot and displayed base URL. The focused regression starts from
  DeepSeek, binds Claude Plan, applies OpenRouter thread settings, and proves `/status` reports only
  OpenRouter. On post-format artifact
  `8f8e27d698aad0d15a3caa1ecb1fb507ce3b56584849e5ff38bf93d97dad9b4f`, a new process resumed the
  original retained crew. Angmar `019fc20e-c5fd-7573-96fb-e66ed4aa0f97` reported
  `Claude Plan - https://api.anthropic.com/v1`; Krimp
  `019fc20e-c793-7562-9773-56f78d624e2a` reported
  `OpenRouter - https://openrouter.ai/api/v1`. Both retained their original roles, transcripts,
  thread IDs, completed results, and reusable idle state. No provider inference or spend occurred
  during this status-only retest. Other providers, permission profiles, account modes, and all
  remaining `/status` rows still require the section 13 matrix.
- **PFTQA-031 — CLOSED FOR WORKSPACE-WRITE/NEVER INHERITANCE AND COLD PERMISSION RESTORATION;
  FULL PERMISSION MATRIX PENDING.** A fresh isolated DeepSeek direct/high hierarchy started under
  explicit `workspace-write` plus `never`: Main `019fc246-2d5f-7d50-81bf-1f6287bd207d`, Angmar
  `019fc246-b1c3-7040-a3be-aa642c869fc2`, Burzum
  `019fc247-6d9b-79d1-9c17-1c1e1188ff41`, and Snaga
  `019fc248-0e17-7281-a37c-741db6f6b1d3`. Main and every role reported
  `Custom (workspace, never)`. Each role independently wrote its marker inside the configured
  workspace and received exit 1 / `Read-only file system` when attempting the corresponding
  write outside it. Angmar then reused the existing Burzum, which reused the existing Snaga; the
  managed chain produced the same allowed/denied behavior and returned `CHAIN_PERMISSION_GREEN`
  without spawning, replacing, interrupting, closing, or shutting down a member.

  The first flagless restart exposed a generalized resume defect: TUI supplied the current base
  config's Full Access defaults as request overrides, and app-server did not recover approval and
  sandbox state from durable thread history. Resume now treats model and permission overrides as
  independent typed decisions. Only explicit CLI/session/profile permission selections override a
  saved thread; an ordinary resume omits them, and app-server restores the latest acknowledged
  approval policy and concrete permission profile from `thread_settings_applied` or
  `turn_context`. Focused TUI tests cover raw harness flags, session/profile layers, ordinary user
  defaults, and emitted resume parameters; focused app-server unit and protocol tests cover
  durable restoration and explicit-override precedence.

  On final formatted debug artifact
  `db00c6bdd3d94a699b162f4d889498043f1af64a8b9de029f5979d2b318430ef`, explicit resume correctly
  reseeded the restricted contract. A later new process resumed the exact Main with no permission
  flags and restored `Custom (workspace, never)`. The exact original Snaga selected from that
  process reported the same contract. Both Main and Snaga then wrote distinct cold markers inside
  the workspace and received exit 1 for distinct outside writes; host checks confirmed every
  outside artifact was absent. `/spawn status` retained the original Burzum and Snaga IDs as
  completed/idle and reusable. Workspace-write/on-request is closed separately by PFTQA-032 and
  read-only/on-request by PFTQA-033; additional-root/profile, Telegram, and remaining
  provider/pane combinations remain open.
- **PFTQA-032 — CLOSED FOR WORKSPACE-WRITE/ON-REQUEST INHERITANCE, NON-OPENAI DYNAMIC-CONTEXT
  AUTHORITY, AND COLD RESTORATION; FULL PERMISSION MATRIX PENDING.** The first explicit
  `workspace-write` plus `on-request` resume updated Main but left retained native members on their
  prior `workspace-write` plus `never` contract because member attachment hard-coded
  `RestoreFromThread`. Native restored-member resume now receives the same typed permission
  decision as Main: explicit CLI/session/profile selection applies to every retained member, while
  a flagless resume continues to restore each thread's durable contract.

  That repair exposed a separate provider-boundary failure. Angmar correctly reported
  `Workspace (Ask for approval)`, and its new `thread_settings_applied` event persisted
  `on-request`, but DeepSeek obeyed an older replayed developer fragment saying approval was
  `never`. Core intentionally keeps dynamic developer updates append-only for durable audit and
  OpenAI prompt-prefix caching. Non-OpenAI Responses, Chat Completions, and Anthropic request
  builders now retain only the newest fragment for each recognized dynamic section at
  serialization time. Persistent role doctrine, user/assistant history, typed agent messages, and
  the stored rollout remain unchanged; the first-party OpenAI path remains append-only.

  The focused retained-member regression proves both ordinary restoration and explicit
  workspace/on-request override. A request-boundary regression proves a non-OpenAI Responses
  request retains current permissions, user work, and persistent Nazgul doctrine while excluding
  the superseded permission fragment; the existing OpenAI override-history integration test still
  passes. On final formatted debug artifact
  `d136f81ef970752c8d2d66ffb5b3020381ecacd170d15e41984f45e4898ed0a7`, Main, Angmar, Burzum, and
  Snaga all reported `Workspace (Ask for approval)`. The hierarchy completed read, compound Git,
  file-read, and in-workspace marker commands during this slice. On the final artifact each of the
  four roles issued its exact outside-workspace marker through a real approval dialog; every dialog
  was rejected and every forbidden artifact remained absent. Angmar reused the existing Burzum,
  which reused the existing Snaga through native `followup_task`; Snaga returned
  `ONREQ_CHAIN_OK`, both managers independently verified the artifact, and no member was spawned,
  replaced, interrupted, closed, or shut down. A new process then resumed exact Main
  `019fc246-2d5f-7d50-81bf-1f6287bd207d` with no permission flags. Main and exact Snaga
  `019fc248-0e17-7281-a37c-741db6f6b1d3` restored `Workspace (Ask for approval)`, raised and
  survived distinct rejected approval dialogs, and `/spawn status` retained original Angmar,
  Burzum `019fc247-6d9b-79d1-9c17-1c1e1188ff41`, and Snaga as the same reusable hierarchy.
  Read-only/on-request is closed separately by PFTQA-033; additional roots/profiles, Telegram, and
  remaining provider/pane combinations remain open.
- **PFTQA-033 — CLOSED FOR READ-ONLY/ON-REQUEST INHERITANCE, MANAGED FOLLOW-UP, AND COLD
  RESTORATION; FULL PERMISSION MATRIX PENDING.** Exact artifact
  `d136f81ef970752c8d2d66ffb5b3020381ecacd170d15e41984f45e4898ed0a7` resumed the retained
  DeepSeek direct/high hierarchy with explicit `read-only` plus `on-request`: Main
  `019fc246-2d5f-7d50-81bf-1f6287bd207d`, Angmar
  `019fc246-b1c3-7040-a3be-aa642c869fc2`, Burzum
  `019fc247-6d9b-79d1-9c17-1c1e1188ff41`, and Snaga
  `019fc248-0e17-7281-a37c-741db6f6b1d3`. Every role reported
  `Read Only (Ask for approval)`, ran `pwd` plus a compound Git-history read, and read the first
  workspace-manifest line with exit 0. Each role then attempted its own marker write inside the
  configured workspace. Every attempt opened a real approval dialog, every dialog was rejected,
  and host checks confirmed all four targets absent.

  Angmar then reused existing Burzum, which reused existing Snaga through native
  `followup_task`. Snaga read the manifest and returned exit 0 plus exact output `[workspace]`;
  Burzum independently ran the same read and verified both values; Angmar independently repeated
  the read and verified the complete chain. No member was spawned, replaced, interrupted, closed,
  or shut down. A new process resumed exact Main with no permission flags. Main and exact Snaga
  both restored `Read Only (Ask for approval)`, raised distinct approval dialogs for workspace
  writes, survived rejection, and left both cold targets absent. `/spawn status` retained the
  original Angmar/Burzum/Snaga hierarchy. Additional writable roots/profiles, Telegram, remaining
  provider/pane combinations, and any unfilled command-variety cells remain open.
- **PFTQA-034 — CLOSED FOR DEEPSEEK AUTOMATIC REVIEW AND THE FULL ACCESS USER FLOW; BROADER
  PROVIDER/PERMISSION MATRIX PENDING.** On the retained exact Snaga, selecting Approve for me
  initially sent the OpenAI-only hidden helper name `codex-auto-review` to the direct DeepSeek
  endpoint, which rejected it before command review. Configured providers now resolve all three
  background helper workloads—approval review, memory extraction, and consolidation—to a model
  valid for that backend. Unknown custom non-OpenAI providers fall back to the current active
  model instead of inheriting a globally configured OpenAI helper.

  The generalized provider table regression covers 16 built-in configured routes across the
  three helper selectors; focused Core regressions cover direct DeepSeek and an unknown custom
  provider. The final debug artifact
  `5e11eaf561f3c7233a42146673b37e2879a40f2c66d3d9ca198c4eeaa350ed65` launched a Guardian whose
  rollout identifies provider `deepseek` and model `deepseek-v4-flash`; it returned `allow`, and
  the reviewed outside-workspace command ran without a human approval dialog or route error. The
  Full Access warning defaulted to Cancel and preserved the prior profile. After explicit
  confirmation, an outside command ran without approval, and a flagless new process restored Full
  Access on the same Snaga `019fc248-0e17-7281-a37c-741db6f6b1d3`. Both disposable markers were
  removed, and QA reset the retained hierarchy to Workspace/Ask. No member was spawned, replaced,
  interrupted, closed, or shut down.
- **PFTQA-035 — CLOSED FOR THE EXACT FRESH `--yolo` USER-PANE COMPOUND-COMMAND CELL.** Exact
  artifact `5e11eaf561f3c7233a42146673b37e2879a40f2c66d3d9ca198c4eeaa350ed65` resumed Main under a
  literal `pfterminal-debug --yolo` invocation and reported Full Access. Through `/panes`, QA
  created and switched to a new persistent DeepSeek direct/high PFTerminal pane
  `019fc28f-d89f-7b41-92d1-b6939536ba02`; creation started no task, and the pane reported YOLO
  mode. It invoked one non-allowlisted outside-write + read + Git-history compound command through
  `exec_command`, returned exit 0, and produced no approval or bubblewrap event. The disposable
  marker was removed. A new explicit workspace-write/on-request process then restored
  Workspace/Ask on exact Main, the same new user pane, and exact retained Snaga. The user pane and
  managed hierarchy remain persistent and reusable; no managed agent was shut down.
- **PFTQA-036 — CLOSED FOR BACKGROUND-HELPER PROVIDER IDENTITY COLLISIONS.** A retained mixed-crew
  code review found that helper routing still inferred built-in identity from credential-variable
  names and one display-name check. A custom/OSS provider reusing a built-in credential variable,
  or a custom provider named `OpenAI`, could therefore inherit a hidden model from the wrong
  backend. Built-in recognition now matches the provider transport contract (name, base URL, and
  wire API), and one centralized resolver keeps approval review, memory extraction, and memory
  consolidation on the active model whenever a hidden OpenAI default is not valid for that
  transport. The two new collision regressions, the focused Guardian custom-provider regression,
  scoped fixes, formatting, and the final debug build passed. The focused memory runtime fixture
  compiled but overflowed its existing test stack, so that runtime test is not claimed green.
- **PFTQA-037 — CLOSED FOR RUNNING-PROCESS PROVIDER-CREDENTIAL CACHE COHERENCE; ASYNC COMPLETION UX
  REMAINS OPEN.** On the pre-repair binary, a real DeepSeek key added and replaced through generic
  `/vault` authenticated successfully, but deleting `provider/deepseek_api_key` left the current
  process authenticated and a subsequent real request succeeded. Generic provider-labelled vault
  add/replace/delete now use the canonical provider-auth storage functions, which update the shared
  storage revision observed by every `AuthManager` cache. On artifact
  `a4aa83739b953b87aa011f3eb6c8bbe2d7189c55d92d764321fcdf78432ab026`, a real DeepSeek request
  succeeded immediately after confirmed replace, failed in the same process with missing
  `DEEPSEEK_API_KEY` after confirmed delete, and succeeded again after confirmed generic add. The
  live run also proved an operation may dismiss its modal before its asynchronous completion toast;
  callers must currently wait for `Added`, `Replaced`, or `Deleted` before relying on the mutation.
- **PFTQA-038 — CURRENT GPU CATALOGUE/LOCAL BILLING STATE QUALIFIED WITHOUT A NEW RENTAL.** Artifact
  `a4aa83739b953b87aa011f3eb6c8bbe2d7189c55d92d764321fcdf78432ab026` opened `/gpu status` in an
  isolated clone and rendered the qualified official-weight DeepSeek 0731 TP2 recipe at revision
  `deepseek-v4-flash-0731-sglang-v0.5.15-post1-2xh200-r3`, plus the qualified GLM route and two
  distinctly labelled experimental GGUF routes. Selecting official DeepSeek opened the first
  pre-charge maximum-hourly-price step; Esc cancelled without creating a rental. Read-only state
  inspection found 33 historical rentals and zero active or potentially billable local records:
  every row is `terminated_confirmed` or `failed`, and no runtime provider remains registered.
  Direct read-only Vast and RunPod inventory calls using the old standalone key files returned 403
  and 401, respectively, so independent provider-side absence is not claimed from those stale
  files. No new rental, provider mutation, or GPU spend occurred.
- **PFTQA-039 — CURRENT TELEGRAM, TASK NODE, AND DOCTOR BOUNDARIES INSPECTED; MUTATING REMOTE
  MATRICES REMAIN OPEN.** The current artifact returned healthy read-only Telegram identity and
  authorization state for `@a666mac_bot`, one allowed chat, one allowed user, and the configured
  workspace. The production poller is still a seven-day-old process executing a now-deleted binary;
  its durable state has one thread, completed update IDs, and no pending delivery, so QA did not
  replace or interrupt that production connector merely to obtain candidate evidence. The actual
  Task Node status, balance, rewards, outstanding-task, and verification commands all failed closed
  with the bounded `terminal_login_required` result and leaked no token; linking/mutations require
  operator authentication. `doctor --json` completed 17 checks with one warning: 761 indexed thread
  rows versus 747 active rollouts, including three missing active rows and eight stale placeholder
  rows, with zero duplicate IDs, malformed filenames, archive mismatches, or scan errors. Live
  Telegram messaging/restart and the authenticated Task Node mutation matrix remain mandatory.
- **PFTQA-040 — CURRENT GOAL, MEMORY, MCP, HOOK, SESSION, AND LOCAL-UI SLICE QUALIFIED; COMPLETE
  MATRICES REMAIN OPEN.** In one disposable DeepSeek home, `/goal` created a real continuation,
  reported time/token use, paused, edited, survived process restart with the edited objective and a
  resume prompt, and cleared. The deliberately vague QA objective consumed about 40K tokens in one
  minute while scanning broadly, so continuation budgeting/prompt restraint remains a product-risk
  row. `/memories` persisted use/generate enablement, launched the real consolidation worker, marked
  its global job done, and produced valid empty-inventory `MEMORY.md`, summary, and raw-memory
  artifacts. A configured stdio MCP server was discovered with both tools; a real model turn opened
  the MCP approval boundary and the approved nested `codex` call returned `MCP_NESTED_OK`.

  `/hooks` listed every lifecycle event, detected the new command, required review, displayed its
  exact command/timeout/source, persisted trust, enabled/disabled it, and executed the trusted
  `SessionStart` marker on the first real turn. The same home completed `/side` and `/btw` isolated
  turns; text-file mention/read with a spaced filename; rename, new, name-based resume, fork, and
  divergent-fork history; raw-mode on/off; copy; empty background-process stop/clean; live theme,
  title, and status-line persistence; experimental toggle/rollback; and keymap conflict detection,
  remap, actual shortcut use, keypress inspection, and default restoration. Full goal compaction and
  resume, non-empty memory extraction, hook timeout/failure/bypass, MCP verbose/resource/elicitation
  and reconnect failures, plugin installation, authenticated apps, and the remaining command matrix
  stay open.
- **PFTQA-041 — RUNTIME IMPORT INPUT ROUTING AND PRODUCT IDENTITY REPAIRED; ADDITIONAL CORE
  COMMANDS QUALIFIED.** Live use of `/import` on the current runtime first exposed two generalized
  defects: the screen retained Codex product copy, and it created a second terminal-event stream
  while the application's permanent input drainer was still polling the shared single-consumer
  broker. The prompt rendered but ignored Escape, Ctrl-C, navigation, shortcuts, and Enter. The
  import source and selection screens now consume the application's existing drained-event
  receiver; no command-specific key or literal route was added. All import copy now names
  PFTerminal except factual compatibility paths and internal protocol identifiers. The focused
  import suite passes 23/23, including runtime-receiver regressions for both nested screens. On
  exact debug artifact `7b6574b7fff8ea780f61ce1b0f956d0c91237c803844fdbcd103603019522af0`,
  live `/import` accepted Up/Down, explicit Cancel, and Escape and returned to the composer.

  The same disposable workspace used `/init` to create and re-read a real `AGENTS.md`; `/diff` to
  render an exact uncommitted patch; `/review` to inspect that patch and identify the deliberately
  removed zero-division guard; `/compact` followed by a continuity turn returning
  `COMPACT_REVIEW_OK`; and Plan mode followed by an exact return to Default mode. `/model` showed
  direct DeepSeek and the OpenRouter catalogue including pinned
  `deepseek/deepseek-v4-flash-0731`; `/providers` exposed DeepSeek, OpenRouter, Anthropic, Meta,
  Vercel, and Baseten rather than hiding configured-key providers. `/orchestrate status` rendered
  the empty assignment state and the guided picker resolved the current durable thread. `/approve`
  and `/ide` returned bounded truthful unavailable-state messages. `/clear PFTQA Clear` created a
  new named durable session with a new ID while preserving workspace, permissions, provider, and
  instructions; `/logout` completed and exited the isolated process. Import execution itself was
  cancelled to avoid copying real external-agent data into the fixture, so imported-data fidelity,
  conflict, rollback, and partial-failure cells remain open. After recording hashes, IDs, token
  totals, and results, QA removed the 95 MiB disposable home, workspace, secret-reading launcher,
  token scratch file, and six generated `.snap.new` files; no matching process remained.
- **PFTQA-042 — LOCAL PLUGIN LIFECYCLE QUALIFIED.** In a second isolated home, `/plugins` loaded
  the real 180-entry catalogue, opened the Build Web Apps details page, disclosed source, version,
  authentication behavior, skills, hooks, and apps, and installed the local curated plugin. The
  installed count changed from zero to one; `/skills` immediately exposed Build Web Apps as a
  plugin result. Space disabled and re-enabled it with visible state changes, and the details-page
  uninstall returned the count to zero. One accidental prompt caused by retaining the `@` mention
  character was interrupted after a read-only directory listing; its persisted usage was 21,835
  input tokens, including 11,264 cached, and 235 output tokens. No plugin app authenticated and no
  external marketplace was added. QA terminated the process, removed both 95 MiB disposable homes
  created by the two command slices, and removed an empty stale `/tmp/.git` marker that had made
  onboarding incorrectly describe `/tmp` as a Git project. Remote marketplace add/update,
  authenticated plugin apps, hooks, restart persistence while installed, and failure rollback
  remain open.
- **PFTQA-043 — `/spawn` TROLL TASK, BUSY DELIVERY, AND COLD REUSE CLOSED.** On artifact
  `7b6574b7fff8ea780f61ce1b0f956d0c91237c803844fdbcd103603019522af0`, the retained DeepSeek
  Burzum accepted a role-correct management task through the `/spawn` task UI, delegated the exact
  shell probe to retained Snaga, independently verified the result, and returned
  `TROLL_TASK_UI_OK`. While Burzum was running a second task, another assignment entered the same
  native mailbox/active turn; `/spawn status` stayed responsive and projected the running task,
  then Burzum returned `BUSY_SECOND_DONE`. After a full process restart, `/spawn status` restored
  exact Burzum `019fc247-6d9b-79d1-9c17-1c1e1188ff41` and Snaga
  `019fc248-0e17-7281-a37c-741db6f6b1d3`; both accepted new work on the same threads, and Snaga's
  terminal result returned automatically to Burzum for review. Neither member was replaced,
  closed, interrupted, or shut down.
- **PFTQA-044 — PACKAGED DOCS, ENCRYPTED-FALLBACK VAULT, AND USER-PANE SURFACES QUALIFIED.** On
  artifact `01f5ab3f9d9ca7af88f1cb31f06f315545cccc87fb8d62d13c574b35da0f4bd9`, bare `/docs` from an
  unrelated workspace loaded the packaged 50-page viewer; targeted
  `/docs integrations/openrouter`, search, internal navigation, missing pages, and malformed
  options all produced bounded correct results. The debug launcher now identifies the managed
  package root and seeds an encrypted fallback key under the debug home's path-derived filename;
  it also exports the authoritative `PFTERMINAL_DEBUG_HOME`, not only the entrypoint-overridden
  `CODEX_HOME`. A fresh vault reported its actual key source, stored one masked disposable
  credential with a 0600 fallback key, listed metadata only, and recovered it after a full process
  restart. `/panes` created a DeepSeek direct/high user pane, renamed and navigated it, and a real
  tool turn returned `FINAL_PANE_TOOL_OK`. `/side` and `/btw` returned their exact markers and
  control returned to the parent; a named fork retained lineage and diverged normally.

  The same fresh-state slice exercised adjacent PF boundaries without claiming paid/remote
  success: `/wallet status`, idempotent lock, masked no-wallet unlock, and invalid usage were
  truthful; `/gpu status` exposed the qualified 0731 TP2 recipe and negative stop/terminate results;
  Telegram reported disconnected, rejected a deliberately invalid token, refused start without
  configuration, treated stop as already stopped, and preserved state when disconnect confirmation
  was cancelled. `/providers` showed both DeepSeek available from environment and OpenRouter not
  configured. No wallet funds, bot, rental, or provider-side resource was created.
- **PFTQA-045 — TASK NODE LINK/LOGOUT NO LONGER BLOCK THE TUI; AUTHENTICATED MATRIX OPEN.** The
  pre-repair `/tasknode link` ran network and encrypted-vault persistence on the UI event loop and
  froze all input. Link now performs the entire start-and-save transaction on a named worker;
  logout similarly performs revoke and encrypted deletion off the UI thread. Focused Task Node TUI
  tests pass 2/2. On artifact
  `01f5ab3f9d9ca7af88f1cb31f06f315545cccc87fb8d62d13c574b35da0f4bd9`, a deterministic local
  endpoint held the link request open: `/status` rendered immediately during the stall, then link
  failed with its bounded timeout. `/tasknode logout` likewise left `/status` interactive and
  eventually reported local session removal. No real Task Node account was linked and the
  authenticated task/context/chat/reward matrix remains open.
- **PFTQA-046 — EXACT MODEL EFFORT RESTORATION CLOSED FOR ORDINARY RESUME.** A real named fork first
  exposed that `/resume` could display `default` even though the source rollout's latest
  `thread_settings_applied` and `turn_context` both persisted DeepSeek `high`. SQLite metadata can
  lag or be overwritten by an older resume projection. App-server resume now treats the newest
  durable settings/turn context as canonical when no invocation-level model override was supplied;
  focused stale-projection and explicit-default regressions pass. On final artifact
  `ae0e967c3d81969b8769781b2b0ebb4d943c86902327219a2f2e6ae5e863523e`, a normal flagless
  `pfterminal-debug --yolo` invocation resumed exact thread
  `019fc31a-ec18-7423-b915-b5181ad48848` with DeepSeek direct/high, then a real response returned
  `RESUME_EFFORT_OK`; the newly appended turn context again records model `deepseek-v4-flash` and
  effort `high`. An explicit invocation `-m` remains an intentional current-runtime override and
  is a separate policy permutation, not ordinary resume.
- **PFTQA-047 — PACKAGED `/docs` CLOSED ACROSS EVERY ADVERTISED PANE CLASS.** The retained crew was
  cold-started on final artifact
  `ae0e967c3d81969b8769781b2b0ebb4d943c86902327219a2f2e6ae5e863523e`, preserving exact Main,
  Nazgul, Troll, Orc, and PF user-pane identities. Bare or targeted packaged docs then opened from
  Main, PF user pane, Angmar, Burzum, and Snaga. QA created a disposable Opus 5 Claude Plan
  headless pane without starting a model turn, opened the targeted OpenRouter page there, and
  permanently deleted that pane through its dedicated lifecycle. The managed crew remained
  durable and reusable. Combined with the earlier real active-Main `DOCS_ACTIVE_OK` turn, the
  advertised pane-class/active-turn docs gate is closed. No provider request or remote mutation
  occurred. The two final isolated QA homes containing copied credentials, their shared workspace,
  and the stale detached Telegram QA daemon were removed after evidence capture; the retained crew
  home remains intentionally available for subsequent crew QA.
- **PFTQA-048 — CENTRAL OPERATOR DOCS CORRECTED AND LIVE-RENDERED; HISTORICAL PAGE AUDIT OPEN.** The
  release-facing index, install, getting-started, authentication, configuration, slash-command,
  integration-index, and OpenRouter pages had stale provider/model inventories that omitted direct
  DeepSeek, pinned OpenRouter DeepSeek Flash 0731, Kimi, Meta, current Claude routes, and multiple
  preserved PF product surfaces. QA updated those existing pages rather than creating another spec
  set. The terminal viewer rendered the edited getting-started, authentication, OpenRouter, and
  slash-command pages successfully from the packaged tree. Older implementation-record pages still
  contain historical model names and must be labelled or reconciled before the content-by-content
  documentation gate can close.
- **PFTQA-049 — SLASH CONTROL-PLANE ARGUMENT FALLTHROUGH REPAIRED.** Live use exposed that
  `/panes Burzum` was recognized as a command name but, because `/panes` has no inline arguments,
  fell through into a paid agent turn. The shared slash parser now retains recognized commands with
  unsupported arguments as control-plane input; dispatch warns locally, clears the stray arguments,
  and runs the bare command. Unknown commands still fail locally and preserve the draft. Four
  focused direct/queued/external regressions passed before final format. On final artifact
  `05451548f4f4ec7da78a80fdf91a6cd6ba1f603c80e7a92a2fb3128cf4e6545a`, live `/panes Burzum`
  opened the picker with an actionable warning and `/pane` showed the canonical `/panes` suggestion;
  the source rollout remained at exactly ten turn contexts before and after both inputs.
- **PFTQA-050 — VAULT CONCURRENCY, WRONG-KEY, AND CORRUPTION BOUNDARIES CLOSED.** A synchronized
  concurrent-writer regression reproduced encrypted-store corruption twice before repair: each
  writer independently initialized or rewrote the passphrase/index state, and a subsequent vault
  operation could no longer decrypt `local.age`. Vault operations now hold one filesystem lock for
  the complete read/modify/write transaction across threads and processes. The repaired concurrent
  writers preserve every distinct credential. A vault opened with the wrong key fails without
  changing one byte of ciphertext, after which the original key still reveals the original value;
  a newly opened vault also rejects deliberately corrupted ciphertext for both list and reveal.
  The complete serial `codex-vault` suite passed 17/17 and debug artifact
  `942f608981bbb4a42257e7d876d407840a5cd06ddcf228c4693685f32527174f` built successfully.
  Remaining vault gates are the OS-keyring restart, partial-write/permission failure, clipboard
  clearing, and every inline/menu permutation.
- **PFTQA-051 — LOCAL COMMAND BREADTH EXERCISED; DEBUG APPROVAL RESOLUTION REPAIRED.** On artifact
  `942f608981bbb4a42257e7d876d407840a5cd06ddcf228c4693685f32527174f`, live `/feedback` rendered
  all five categories, disclosed every proposed attachment, cancelled without uploading, and, after
  a cold restart with `feedback.enabled=false`, rendered the explicit disabled state. `/ide` failed
  locally with actionable VS Code/Cursor guidance when no IDE was attached; `/pets` truthfully
  refused unsafe tmux rendering; `/vim` entered Normal mode and disabled again through actual Vim
  input; unsupported DeepSeek `/personality` failed locally; `/rollout` printed the exact current
  file and `/ps` accurately reported no background terminals. Live `/test-approval` then exposed a
  real routing defect: cancelling a locally generated approval was forwarded as an unowned thread
  operation and reported `Not available in TUI yet`. Approval requests now carry an explicit
  response destination, so thread-owned approvals retain the app-server path while local previews
  terminate inside the TUI. Three focused local/thread routing regressions passed. Rebuilt artifact
  `7d25e1d8e2fb64c31a379671e8d797bb4a8a2ea8426a59aa4cdc9c3b2dba94b2` returned `Local approval
  resolved: cancelled.` in live use. A subsequent `/spawn status` retained the same Angmar, Burzum,
  Snaga, and Ghash IDs as idle/reusable members; no agent was replaced or shut down.
- **PFTQA-052 — CLI COMPLETION IDENTITY AND PROFILE HELP REPAIRED.** Real generation on artifact
  `7d25e1d8e2fb64c31a379671e8d797bb4a8a2ea8426a59aa4cdc9c3b2dba94b2` found that Bash, Zsh,
  Fish, PowerShell, and Elvish completion scripts all registered `codex`; Bash contained no
  `pfterminal` token. The root profile option also instructed users to load from `$CODEX_HOME`.
  The shared completion generator now emits the PFTerminal executable identity, and nested plugin
  and marketplace command metadata/examples use PFTerminal as well. All five generated scripts on
  rebuilt artifact `13708c53ba2f50db1268c12cfabf3cab2afb9c1d87a7d304a2c715f69b9d4440`
  contain `pfterminal` with no stale command registration, and root help reports
  `$PFTERMINAL_HOME/<name>.config.toml`. The all-shell regression passed in all three CLI binary
  test targets. Actually sourcing each script and exercising completions in its shell remains open.
- **PFTQA-053 — WALLET INLINE NO-STATE PARITY REPAIRED.** On the isolated retained-crew home,
  `/wallet status` accurately rendered no wallet; inline `/wallet create` and `/wallet restore`
  opened their masked secure modals and cancelled without creating state. Pre-repair `/wallet unlock`
  nevertheless solicited a passcode, then failed only after secret entry with `no wallet exists`;
  `/wallet lock` claimed a global lock despite the same uncreated state. Unlock now checks daemon
  state before opening any secret-entry surface, and lock checks existence before reporting a state
  transition. The focused unlock/custom-duration set passed 7/7. On rebuilt artifact
  `3fcf2685e3c11c9ec103729c0754936bfd98d70799bb8a7ab608a56491846834`, live unlock requested no
  secret and directed the user to create/restore; live lock reported `No local wallet exists; there
  is nothing to lock.` The menu continued to show the two valid Create/Restore actions. Funded
  purchase, migration, and full positive inline/menu duplication remain open.
- **PFTQA-054 — `/spawn` PROVIDER-FAILURE RECOVERY RETAINS THE SAME AGENT.** The retained all-DeepSeek
  crew was cold-resumed with a deliberately invalid credential and a task was sent through the
  actual `/spawn status` task UI to Snaga `019fc248-0e17-7281-a37c-741db6f6b1d3`. The turn ended
  with a bounded, visible DeepSeek 401; `/spawn status` kept the exact Snaga identity, recorded the
  error, and continued to offer `Send task to Snaga`. After a graceful TUI restart with the valid
  direct credential, the persisted crew showed the same Snaga ID as idle and reusable. A second
  task to that same member returned exactly `SPAWN_RECOVERY_OK`, and persisted crew state returned
  to `ready`. No member was removed, replaced, interrupted, or shut down.
- **PFTQA-055 — CLI COMPLETION EXECUTION AND DOCTOR IDENTITY QUALIFIED.** The rebuilt Bash completion
  script was sourced in a clean Bash process, registered `_pfterminal` for the `pfterminal`
  executable, and completed the `re` prefix to both `review` and `resume`. Zsh, Fish, PowerShell,
  and Elvish are not installed on this host, so their execution cells remain explicitly open.
  Live `doctor --json` exposed a deeper inherited-branding defect: it reported `codexVersion`,
  `CODEX_HOME`, and PATH entries for `codex` instead of diagnosing PFTerminal. The diagnostic
  boundary now emits `pfterminalVersion`, `PFTERMINAL_HOME`, and resolves PATH entries for
  `pfterminal`; stale `codexVersion` is absent. On artifact
  `1233ac0e1f4447a91f9c080bc4df673f7669f4e3643e9fc28db3f3669472e65c`, the live report returned
  `overallStatus: ok`, the correct branch and home, and the rebuilt Bash completion again passed.
  The CLI crate run executed 849 tests: 848 passed; the unrelated cloud-sandbox integration test
  `sandbox_fetches_and_enforces_cloud_managed_permission_profile` failed after retry while waiting
  for its sandboxed version command. The doctor/completion tests passed in all three binary targets.
- **PFTQA-056 — PROVIDER SEARCH REPAIRED; LOCAL SETTINGS/EXTENSION SURFACES EXERCISED.** Live
  `/providers` correctly showed Direct DeepSeek as environment-backed, but searching for
  `OpenRouter` returned `no matches` even though the provider row existed below the viewport. The
  shared selection view intentionally filters only on each row's explicit search index, and every
  provider row had omitted that index. All account, plan, and API-key provider rows now index their
  complete visible name; this repairs the whole provider-search class rather than special-casing
  OpenRouter. On final artifact
  `bcf0a0878d7ec79c9a8e972c751ad35c44543fa8524b365503bc9366c0a3094c`, a real search returns
  `Provider: OpenRouter API Key — Not configured`, while Direct DeepSeek remains visible as
  environment-backed. The same live batch exercised `/status`, `/debug-config`, the non-Git `/diff`
  boundary, `/keymap`, `/keymap debug` with a real keypress, `/raw` on/off, theme preview/select/restore,
  title and status-line mutation persisted to config, `/experimental`, memories cancellation,
  filesystem skill discovery, the empty hooks and MCP states, ChatGPT-gated `/apps` hiding, and the
  29-row plugin catalogue without installing anything. Windows-only sandbox setup/read-root commands
  were correctly absent on Linux. Stale Codex product text in the exercised key inspector, title,
  hook, and sleep-inhibitor descriptions was corrected; the title's actual live app-name value was
  also changed from `codex` to `pfterminal` and verified in the rebuilt preview. Features tests passed
  33/33 and the new title identity regression passed 1/1. The full TUI crate run was not green:
  3,778/3,803 passed, with 25 current failures concentrated in dispatch integration, queued slash
  behavior, several snapshots, and lifecycle cases; those failures remain explicit release work.
- **PFTQA-057 — REBUILT RUNTIME, RETAINED CREW, SIDE TURNS, FILE MENTION, AND LOCAL SERVICE
  ADMISSION QUALIFIED.** Final debug artifact
  `9d90437015b51c831af2e402e065ac21df7e3fb1e858e8d84cf2555a08d8b47f` was verified through
  `/proc/<pid>/exe`, then cold-resumed the exact Main thread and the existing Angmar/Burzum/Snaga/
  Ghash/Krimp hierarchy with unchanged thread IDs. `/panes`, `/agent`, and `/subagents` agreed on
  the complete idle hierarchy. Direct DeepSeek completed an ordinary streamed response and a real
  `pwd` tool turn in Full Access; `/copy` then reported a successful copy of the exact last response.
  Both `/btw` and `/side` completed real provider turns in an ephemeral Side surface and `ctrl+c`
  returned to the intact Main transcript. `/init` created a 369-word `AGENTS.md` only in the isolated
  empty QA workspace; `/mention` found that file, attached it, and the model returned its first
  Markdown heading without a shell call. The import workflow discovered one settings item, two
  plugins, and three recent chats, displayed the exact pending mutations, and was cancelled before
  changing state. `/usage`, `/ps`, `/stop`, `/rollout`, tmux-safe `/pets`, provider-incompatible
  `/personality`, `/gpu status`, `/telegram status`, `/tasknode status`, and disposable `/goal`
  create/status/interrupt/pause/clear paths all produced bounded, truthful results. The GPU picker
  still exposes the qualified official `deepseek-ai/DeepSeek-V4-Flash-0731` TP2 route separately
  from experimental variants; Telegram reported not connected; Task Node failed closed as unlinked.
  Claude Plan catalogue descriptions now identify PFTerminal rather than a Codex harness, and live
  `/providers` search still finds the unconfigured OpenRouter row. Models-manager tests passed 60/60
  before the required fix/format/build sequence. This closes a broad local/runtime slice; it does not
  convert the credentialed Telegram, Task Node, GPU-rental, import-commit, non-tmux pet, or full
  provider matrices to green.
- **PFTQA-058 — PROVIDER-AWARE STATUS, CLI EXECUTION/REVIEW, PERMISSIONS IDENTITY, COMPACTION,
  AND PLAN MODE QUALIFIED.** Final debug artifact
  `189bd714b18c1e125dedf27ea1eb8d186a52079311f899acc6c7a34f55e193b2` cold-resumed the exact
  retained Main and managed crew before the local session matrix. A persistent Opus 5 Claude Plan
  pane completed real response and shell-tool turns, cold-restored its transcript, and `/status`
  now identifies command-backed Claude account auth instead of claiming that an API key exists.
  `pfterminal exec` help, `$PFTERMINAL_HOME`, and human-output role labels now identify PFTerminal.
  Direct DeepSeek completed real human, JSON, stdin, and tool executions; OpenAI Luna completed
  real response and tool executions; `review --uncommitted` found the intended P1 behavior defect
  in a disposable repository fixture, which was removed. The shared approval-preset boundary now
  uses PFTerminal copy in every permission UI; four focused popup tests and live `/permissions`
  passed. `/compact` completed and the next real turn returned `COMPACT_CONTINUE_OK`. The native
  upstream `/plan` workflow proposed a plan without tools, accepted a revision, explicitly stayed
  in Plan mode, then switched to Default only after explicit approval and executed the approved
  read-only commands. A QA harness initially navigated the implementation modal incorrectly and
  created a disposable probe; it was removed and is not counted as product failure evidence. Exec
  tests passed 129/129, models-manager passed 60/60, CLI passed 848/849 with the same unrelated
  cloud-sandbox failure, and the changed permission/status snapshots passed focused tests. The last
  broad TUI run remains red and must be rerun after the focused fixes; this batch does not authorize
  release or convert any remote integration matrix to green.
- **PFTQA-059 — LOCAL SANDBOX ENFORCEMENT, ACTIONABLE LIFECYCLE ERRORS, AND PLUGIN RESTART
  PERSISTENCE QUALIFIED.** Artifact
  `fb3c40bcfa445cc45546e534826febcfbbc16df1bab444b145e6915d399c5b1a` ran real commands through
  each built-in permission profile. Read-only allowed reads but rejected writes and DNS; workspace
  allowed an in-workspace write but rejected a repository-external write; Full Access reached the
  network; exit status propagated in every case. Missing archive and unarchive names failed safely.
  Missing forced delete initially collapsed its app-server cause into `failed to delete session`;
  the shared CLI report formatter now preserves nested RPC causes for the complete lifecycle class.
  Its regression passed in all three CLI binary targets, and live delete output includes the exact
  thread ID and missing-rollout cause with empty stdout and nonzero exit. The local curated
  `build-web-apps` plugin installed, survived a full process restart, contributed a visible plugin
  row and `frontend-app-builder` skill, and completed a real selected-skill DeepSeek turn. Removal
  followed by another restart removed both the plugin and its skills. Disposable sandbox and plugin
  test artifacts were removed; no plugin remains installed. Remote marketplace, plugin app/hook,
  authentication/failure rollback, managed sandbox, remote lifecycle, and platform matrices remain
  open, so this is not complete feature acceptance.
- **PFTQA-060 — NATIVE `/orchestrate` DISPATCH, STATE, RESULT HANDOFF, AND DUPLICATE-AUDIT
  BOUNDARIES QUALIFIED.** Final debug artifact
  `87ff1ba7302e03d831d78f9fd0332bced0ce3c5c689a95775419ac1d1d6bd044` retained the exact
  mixed-provider Main, Angmar, Burzum, Snaga, Ghash, and Krimp thread identities. Pre-repair live
  assignments proved three generalized failures: native managers were prompted to emit the legacy
  fenced adapter; Multi-Agent v2 `followup_task` success was not projected into assignment state;
  and a manager with multiple active assignments was correlated by holder alone through arbitrary
  `HashMap` iteration. Native assignment prompts now require `followup_task` to the durable Worker
  thread ID and reserve the fenced adapter for external panes. The state observer handles both the
  v1 completed SendInput item and the actual v2 completed Interacted activity, correlates the exact
  `(Manager, Worker)` pair, starts the execution clock, and records delivered state. For a direct
  native parent/child pair, Core's own terminal-result delivery is authoritative and PFTerminal no
  longer queues a duplicate audit mandate. The final live assignment used the already retained
  Angmar and Burzum: one native dispatch produced `ORCHESTRATE_NO_DUP_OK`, one native result reached
  Angmar, one audit emitted `WHIP_DONE`, and persisted assignment-5 is Done with `fires: 0`,
  `last_dispatch_result: delivered`, and the exact Worker output. The affected automated set passed
  33/33 before the required fix/format/build sequence. No agent was spawned, replaced, closed, or
  shut down. Historical pre-repair assignments remain preserved as failure evidence; provider,
  topology, pause/resume/extend, failure-retry, external-pane, restart-mid-assignment, and long-run
  matrices remain open, so `/orchestrate` is not globally accepted and the release remains NO-GO.
- **PFTQA-061 — MANAGED-CREW ACTIVE-TURN `/panes` SWITCHING QUALIFIED.** The same final artifact
  `87ff1ba7302e03d831d78f9fd0332bced0ce3c5c689a95775419ac1d1d6bd044` selected exact retained
  Snaga `019fc20e-c6c8-7cd1-ab9d-371651216c8b`, started one real 30-second Luna shell-tool turn,
  and opened `/panes` while it was active. The picker labelled Snaga `running` and retained its
  prior result preview. Control switched to Main without interrupting or queueing input into Snaga.
  After the background turn finished, `/panes` projected the same Snaga as `completed` with exact
  `CREW_PANE_SWITCH_OK` output. Returning to it preserved the complete command, literal output, and
  exit-0 transcript. No crew identity, role, parentage, provider route, or lifecycle state was
  replaced or shut down. Active Nazgul/Troll/other-Orc, active external Claude, concurrent approval,
  failed-turn, rename/search collision, process-restart, and multi-layout cells remain open; this is
  a managed-Orc switching slice, not global `/panes` acceptance.
- **PFTQA-062 — DIRECT MANAGED-PANE RESULT OWNERSHIP AND DELEGATED DELIVERY QUALIFIED.** PFTQA-061
  also exposed a separate P0 Core routing defect: every terminal turn in a spawned session was
  unconditionally reported to its structural parent. Direct operator work in Snaga therefore
  replayed Snaga's result into Burzum, caused Burzum to answer with stale prior orchestration output,
  and cascaded that stale output into Angmar. This was not a `/panes` rendering failure and was not
  repaired with prompt text or a command regex. Turn admission now records whether the complete
  admitted input contains triggering collaboration work; explicit operator input owns the turn
  locally and conservatively suppresses parent completion, while real `followup_task` work retains
  exactly-once Core result delivery. Focused classification, task-admission, direct-child, and
  delegated-child regressions passed, including a public mock-API integration test that spawns a V2
  child, addresses it directly, and proves its result never enters the parent rollout. Debug artifact
  `27efd8d403e18c568dab9f515e28da75eb1d9bf978dab4b06209e3af7d6d0d59` cold-resumed the exact
  retained hierarchy. Exact Snaga returned `DIRECT_CHILD_NO_CASCADE_27EFD8`; after an additional
  eight-second delay the marker existed only in Snaga's rollout, while Burzum and Angmar byte sizes
  and mtimes remained exactly unchanged. The complementary exact Angmar -> Burzum assignment used
  one `followup_task`, one stable message ID, one terminal-result envelope, and one audit result:
  `DELEGATED_PARENT_DELIVERY_27EFD8` followed by `DELEGATED_AUDIT_OK`. No agent was spawned,
  replaced, closed, or shut down. This closes the direct-operator versus delegated-turn ownership
  boundary; other hierarchy depths, simultaneous steering, interruption/error, provider, restart,
  and soak cells remain open, and the release remains NO-GO.
- **PFTQA-063 — ACTIVE TROLL SWITCHING AND PER-PANE MODEL PROJECTION QUALIFIED.** Artifact
  `5c731c3fd6f770ffd31db326e987abed8aae01dda959a2cad4f464c3bd509db5` cold-resumed the same
  retained hierarchy. Exact Sol-bound Burzum ran `sleep 20 && printf TROLL_PANE_SWITCH_27EFD8`.
  `/panes` labelled Burzum `running`; control switched to exact Main without interrupting the task;
  after completion the row showed the exact output; and returning to Burzum preserved the command,
  literal output, and exit 0. The marker exists only in Burzum's rollout, proving the PFTQA-062
  operator-result boundary at Troll depth. This workflow exposed a separate picker projection
  defect: when opened from Burzum, Main was first labelled `gpt-5.6-sol` even though Main `/status`
  proved `deepseek-v4-flash`, and an incomplete first repair rendered `model unavailable`. Main's
  row had been reading the active chat widget, then a navigation cache that is optional for the
  primary thread. It now reads Main's durable primary `ThreadSessionState`, with navigation metadata
  only as fallback; other native rows continue to use their own cached session model. The focused
  rendered snapshot passed and explicitly proves inactive DeepSeek Main beside active Sol child.
  The broader three-test `pane_picker_` filter remained 2/3 because the independent legacy fixture
  `pane_picker_separates_user_panes_from_managed_spawn_crew` no longer constructs a managed hierarchy
  but still expects a Nazgul row; live retained Nazgul rendering remains correct. That test debt is
  preserved, not called green. Active Troll switching and the observed per-thread model-projection
  boundary are closed; other providers, external panes, concurrent approvals/failures, and multi-
  layout/restart/soak cells remain open, and the release remains NO-GO.
- **PFTQA-064 — ACTIVE NAZGUL SWITCHING AND MANAGED-ROOT STATE PROJECTION QUALIFIED.** The first
  exact Angmar turn on artifact
  `5c731c3fd6f770ffd31db326e987abed8aae01dda959a2cad4f464c3bd509db5` completed
  `NAZGUL_PANE_SWITCH_5C731C` and remained local to Angmar, but exposed a separate P0 picker
  defect: the Nazgul row rendered only a root-binding explanation and could not tell the operator
  whether the managed root was running or completed. This was not repaired with task text or a
  role-specific status invention. The bound-root row now obtains native and external state through
  the same managed-pane projection used by Troll and Orc rows, while retaining its duplicate
  user-pane identity explanation. Focused native-root running/completed and cold-restored CrewSpec
  regressions passed before fix/format/build. Artifact
  `a9bc78485af7924956daf8af8fc3235f95d5642fa181f919405d4381b76fc740` cold-resumed exact
  Main `019fc20e-af8e-70f3-a523-2326a36faaec` and exact Angmar
  `019fc20e-c5fd-7573-96fb-e66ed4aa0f97`. During
  `sleep 25 && printf NAZGUL_ACTIVE_A9BC78`, `/panes` labelled Angmar `running`; control switched
  to exact DeepSeek Main without interruption; the row then changed to `completed` with the exact
  result; and returning preserved the full command, literal output, and exit 0. The marker exists
  only in Angmar's rollout. No agent was spawned, replaced, closed, or shut down. Active switching
  is now proven at retained Nazgul, Troll, and Orc depths; other providers/Orcs, external Claude,
  concurrent approvals/failures, rename/search collisions, multi-layout, restart-mid-turn, and soak
  cells remain open, and the release remains NO-GO.
- **PFTQA-065 — ACTIVE NATIVE CLAUDE PLAN USER-PANE SWITCHING QUALIFIED.** On artifact
  `a9bc78485af7924956daf8af8fc3235f95d5642fa181f919405d4381b76fc740`, exact retained
  native PFTerminal user pane `PFTerminal 1` / session
  `019fc397-e536-7ee3-8c75-bf01df5b87c1` reported Claude Opus 5 Plan, Claude Plan account
  connectivity, and Full Access. Its persisted layout identity is `codex_user_pane_ids`, not a
  headless `claude_pane_ids` entry. During
  `sleep 25 && printf CLAUDE_PANE_ACTIVE_A9BC78`, `/panes` projected the native row as
  `running`; the unfiltered picker simultaneously showed exact DeepSeek Main as idle and the
  native Claude Plan pane as current/running; control switched to Main without interruption; and
  the row returned to idle after completion. Returning to the exact pane restored the complete
  command, literal output, no-trailing-newline note, and exit 0. The marker is present in only that
  native pane's retained rollout among session files. No repair, pane creation/removal, credential
  change, or agent lifecycle operation was required. Native Claude Plan active switching is closed
  for this retained full-access slice; other native providers, headless Claude panes,
  permissions, failures, interruption, restart-mid-turn, concurrent panes, and soak remain open,
  and the release remains NO-GO.
- **PFTQA-066 — NATIVE BACKGROUND TERMINAL DISCOVERY AND EXPLICIT STOP QUALIFIED.** On artifact
  `0150070f2e8fec0f671db6926f6fdddf8e635bb71a6ff966977947885a127abd`, exact retained native
  Claude Plan pane `019fc397-e536-7ee3-8c75-bf01df5b87c1` started
  `sleep 120 && printf BACKGROUND_STOP_MUST_NOT_PRINT_015007`. Esc interrupted the model turn but,
  by the unified-exec contract, retained the yielded background terminal for explicit operator
  control. `/ps` displayed that exact command; host inspection found its Bash and `sleep` PIDs;
  `/stop` removed both; `/ps` then displayed no background terminals; a second `/stop` remained
  idempotent; and the same pane immediately returned `BACKGROUND_STOP_RECOVERY_015007`. The
  completion marker never ran. Focused headless-Claude cancellation tests also pass, including a
  detached tool process group; that supporting adapter coverage is not misrepresented as the
  native unified-exec result. Other pane/provider ownership, multiple-job selection, `/clean`,
  restart, failure, and platform cells remain open, and the release remains NO-GO.
- **PFTQA-067 — NATIVE USER-PANE NAME CREATION AND COLD RESTORATION QUALIFIED.** A retained native
  Claude Plan thread survived restart but its friendly name degraded from `PFTerminal 1` to its
  short thread ID because the ordinary creation path stored the nickname in PFTerminal metadata
  without invoking the app server's durable `thread/setName` boundary. The repair makes every
  native user-pane registration persist the accepted unique name before installing the session,
  updates the returned session state, and surfaces a visible partial-success error if durable name
  persistence fails. It does not infer names from prompts or add name-specific routing. The focused
  creation/refresh integration regression passed before fix/format/build. On artifact
  `2c1c4f7dc4c95c8d759804bee1c6d9867ef6c6a9415698084373d99dd10a1cff`, the original exact
  thread `019fc397-e536-7ee3-8c75-bf01df5b87c1` was restored as `PFTerminal 1` after cold restart.
  More importantly, the ordinary `/panes` creation flow made zero-turn Claude Plan pane
  `PFTQA Name 2c1c4f` / `019fc428-4e75-73e0-b3d1-15851ef618b5`; the backend immediately confirmed
  its name, a second cold restart found it by exact friendly-name search, and `/status` restored the
  exact name, ID, provider, Full Access profile, and zero token usage. The disposable pane was then
  explicitly deleted; its rollout is absent, while `PFTerminal 1` and every retained managed-crew
  ID remain untouched. Duplicate-name, rename collision, persistence-failure injection, and
  platform cells remain open, and the release remains NO-GO.
- **PFTQA-068 — MULTI-JOB `/CLEAN` ALIAS AND PANE RECOVERY QUALIFIED.** On artifact
  `2c1c4f7dc4c95c8d759804bee1c6d9867ef6c6a9415698084373d99dd10a1cff`, exact retained native
  Claude Plan pane `019fc397-e536-7ee3-8c75-bf01df5b87c1` independently yielded two real commands:
  `sleep 180 && printf CLEAN_JOB_A_MUST_NOT_PRINT_2C1C4F` and the corresponding distinct `B`
  command. Host inspection showed separate Bash process groups and child `sleep` PIDs, while the
  footer and `/ps` truthfully reported two background terminals and both exact commands. The
  advertised `/clean` alias removed all four processes; `/ps` returned empty; a second `/clean`
  remained idempotent; and the same pane immediately returned `CLEAN_MULTI_RECOVERY_2C1C4F`.
  Neither completion marker ran. Cross-pane ownership, unrelated-process safety injection,
  restart, partial failure, very-high job counts, and platform cells remain open, and the release
  remains NO-GO.
- **PFTQA-069 — IDLE `/QUIT` AND `/EXIT` DURABLE-STATE PRESERVATION QUALIFIED.** On artifact
  `2c1c4f7dc4c95c8d759804bee1c6d9867ef6c6a9415698084373d99dd10a1cff`, `/quit` from exact
  retained native Claude Plan pane `019fc397-e536-7ee3-8c75-bf01df5b87c1` cleanly terminated the
  TUI with no residual PFTerminal process or background terminal. A cold resume found that exact
  pane as `PFTerminal 1` with the same provider and transcript. `/exit` then cleanly terminated a
  second process from the same pane. The next cold resume retained exact Main, Angmar, Burzum,
  Snaga, Ghash, Krimp, and `PFTerminal 1` identities, names, hierarchy, and idle state; none was
  deleted, replaced, closed, or shut down. These commands ran no inference. Active turns,
  confirmation behavior, intentional background-terminal policy, non-tmux restoration, signals,
  and platform cells remain open, and the release remains NO-GO.
- **PFTQA-070 — REAL TASK NODE GITHUB LINK AND AUTHENTICATED PFTerminal USE QUALIFIED.** A real
  operator-assisted GitHub link initially failed before authentication because
  `tasknode.postfiat.org` returned no bytes before PFTerminal's 15-second request deadline. The
  same failure reproduced outside PFTerminal. Fly inspection found the single production web
  machine alternating between passing and critical health, a recent exit 134/restart, proxy
  connection resets, `/api/app-state` latency as high as 23.4 seconds, and database query read
  timeouts. Even a trivial public `/health` response took 18.46 seconds. No credential was rejected
  or exposed. After Task Node recovered, the operator completed the real GitHub link and confirmed
  PFTerminal was working with the authenticated service on artifact
  `2c1c4f7dc4c95c8d759804bee1c6d9867ef6c6a9415698084373d99dd10a1cff`.

  PFTerminal's non-streaming Task Node client now uses one bounded 45-second timeout for login,
  status, tasks, context, and other requests. Requests still run off the UI thread and are not
  automatically retried, so mutation POSTs cannot be duplicated by resilience logic. The focused
  Task Node TUI set passed 2/2 before fix/format/build; the rebuilt artifact is
  `e85c474f8880ff93bcc01cd0c04566ecd4e4dbfe52baf4471fb3c29593ae5e61`. The user-assisted real
  auth success qualifies linking and authenticated connectivity on the responsive-backend path;
  the rebuilt 45-second behavior is supporting build evidence until exercised during another slow
  response. Task listing/detail/actions, context read/edit, chat, rewards, wallet projection,
  verification, logout/re-login, token expiry/revocation, restart persistence, and the production
  backend capacity incident remain open, and the release remains NO-GO.
- **PFTQA-071 — LARGE USER-RUN MIXED-PROVIDER `/SPAWN` CREW WORK QUALIFIED.** The operator completed
  a substantial real end-to-end `/spawn` crew run on artifact
  `2c1c4f7dc4c95c8d759804bee1c6d9867ef6c6a9415698084373d99dd10a1cff` and reported that the
  complete workflow worked. Read-only durable inspection corroborated the product result without
  reproducing the task's substantive content. Main `019fc462-f69d-7b71-a45f-de75e5b872c6`
  retained exact crew `crew-019fc47a-b484-7212-97c9-bb6c60a2dabd`: Claude Fable 5 Plan/xhigh
  Angmar `019fc47a-b4b7-7fe0-a407-e1600aaa4acb`; OpenAI Sol/xhigh Burzum
  `019fc47b-a938-7b90-b195-33faeb6f3510`; OpenRouter DeepSeek V4 Flash 0731/high Snaga
  `019fc47b-e2c3-70a2-bc5a-36a9a301cc52`; OpenAI Terra/xhigh Ghash
  `019fc47c-039c-74f3-bb8e-2a783e3319ea`; and OpenAI Luna/xhigh Krimp
  `019fc47c-25c0-7070-a0f5-9e307df29df5`.

  Across those five retained role rollouts, the run produced 18 `task_complete` events, 42 durable
  inter-agent communication events, extensive real function/custom-tool activity, and real patch
  completion events in all three Orcs. There were zero recorded error or interruption events. The
  complete parent-edge map persisted and the final durable crew state returned to `ready`; no role
  was deleted, replaced, closed, or shut down. This closes a large successful mixed-provider
  spawned-crew work slice; it does not qualify the separate `/orchestrate` assignment workflow.
  Restart-mid-run, failure/retry/timeout, deliberate provider outage, concurrent independent crews,
  very-long soak, and platform cells remain open, and the release remains NO-GO.
- Compaction no longer injects a synthetic `Continue.` user message.
- Stable and debug binaries resolve isolated PF homes; the final continuity exercise used a fresh
  `PFTERMINAL_DEBUG_HOME` and did not touch the stable PF or stock Codex homes.
- CLI update guidance targets `@agticorp/pfterminal` for npm, bun, and pnpm.
- The logs client honors the exact database path supplied by `--db`, including non-UTF-8 path
  bytes, through an explicit state-layer override.
- Stable and experimental app-server schemas were regenerated from the reconciled protocol. The
  existing app-server schema writer was invoked directly because the root `just` recipe currently
  repeats the `codex-rs/` path after changing into that directory; that recipe defect remains a
  clean-candidate qualification item rather than being hidden by product-code changes.
- PF wallet, vault, GPU, Telegram, Task Node, panes, spawn, orchestration, provider, model, package,
  installer, and branding code remains present. No feature deprecation was accepted as a conflict
  resolution.

### 12.2 Automated evidence completed

| Gate | Result | Notes |
| --- | ---: | --- |
| Native/PF multi-agent preservation set | 108 passed | Includes spawn, nesting, mailbox, wait, resume, interruption, and completion projection coverage. |
| Current Core native-agent focused suite | 458 passed | Includes native role selection, provider-neutral delivery, same-member follow-up, cold resume, terminal-result classification, and the three-dispatch Core loop breaker with operator-input recovery. |
| OpenAI reserved collaboration boundary | 3 schema/profile tests, 5 provider-wire tests, and 7 spawn-description integration tests passed | The model-visible OpenAI plan retains reserved encrypted fields and exact parameter shapes regardless of PF role configuration; provider-neutral schemas remain plaintext-capable. Live PFTQA-025 supplies real endpoint spawn/wait/follow-up evidence. |
| Mixed-provider Core collaboration adapter | 87/87 targeted MultiAgentV2 tests plus focused handler/plan/request regressions passed | Anthropic receives flattened collaboration functions; OpenAI native encrypted delivery remains unchanged for OpenAI targets; incompatible targets fail before admission and use the same Core operation through a redacted plaintext adapter. Live PFTQA-029 proves OpenAI Sol -> OpenRouter Grok follow-up and full retained hierarchy consolidation. |
| Pane-local provider projection | 1 focused retained-session/settings regression passed | Starting from a DeepSeek parent snapshot, the widget binds complete Claude Plan metadata and then complete OpenRouter metadata; `/status` contains the selected OpenRouter provider and URL and no stale DeepSeek identity. Live PFTQA-030 proves the original retained Angmar and Krimp panes after a cold process restart. |
| Permission-contract resume | 3 focused TUI tests, 2 app-server unit tests, and 1 app-server protocol integration passed | TUI distinguishes explicit CLI/session/profile permission overrides from ordinary base-config defaults; app-server restores the latest durable approval and concrete permission profile only when the request has no explicit override. Live PFTQA-031 proves explicit workspace-write/never plus flagless cold restoration on the exact Main and retained Orc. |
| Non-OpenAI current dynamic-context authority | 1 provider-bound request regression and 1 contextual-adapter regression passed; existing OpenAI permission-history integration passed | Non-OpenAI Responses, Chat, and Anthropic serialization sends only the latest recognized permissions/model/mode/environment section without rewriting durable history or PF role doctrine. OpenAI retains its append-only prompt-cache behavior. Live PFTQA-032 proves DeepSeek requests approval instead of obeying a superseded `never` fragment. |
| Provider-valid background helper routing | Built-in provider table plus 2 custom-identity collision regressions; 2 focused Core approval-review regressions passed | Approval review, memory extraction, and consolidation share one transport-aware resolver. Built-ins are identified by name, base URL, and wire API rather than a reused credential variable or display name. Unknown/custom/OSS routes use the active model instead of an OpenAI-only hidden helper. The broad `codex-core` run remains environmentally non-green; the focused regressions pass. The focused memory runtime fixture compiled but stack-overflowed. Live PFTQA-034 supplies direct DeepSeek Guardian evidence and PFTQA-036 closes the custom identity collisions. |
| Core durable native graph boundary | 1 passed | A lazy durable parent and its child are materialized and flushed before Core publishes their native graph edge; the depth-limit fixture also uses a real root/parent chain rather than an orphan. |
| Current TUI focused spawn suite | 81 passed | Native task admission stays on the Core mailbox path; no direct `turn/start` fallback. The set includes the bound-Nazgul Core-parent regression and proves restored `CrewSpec` roles drive the `/spawn` and `/panes` projection even when transient liveness omits role metadata. |
| Pane-layout compatibility and preservation | 14 passed | The reader verifies raw saved JSON before schema-default migration, recovers a verified previous generation, and refuses to destroy either a sole unverified generation or two unverified current/recovery generations. Live PFTQA-013 qualification supplies the full-process evidence that component persistence tests cannot. |
| PF user-pane persistence regressions | 4 passed | Layout owner/member loading, partial-metadata identity preservation, direct-input liveness, and picker separation between operator panes and parent-controlled task workers. |
| PF user-pane current-state projection | 1 passed | When a native child is active, exactly that Core thread—not Main—is labelled current in `/panes`. |
| Current TUI pane-focused suite | 50 passed | Covers picker identity, failed-pane isolation, operator-pane lifecycle cleanup, layout compatibility, human-addressable native panes, and Claude-pane-local failure/status behavior. |
| Active-task slash control-plane admission | 5 passed | Two `/archive` tests, two `/delete` tests, and one popup-independent composer test prove recognized active-task commands dispatch immediately instead of entering the follow-up queue; an adjacent `/status` case prevents an archive/delete-only special case. |
| Rapid slash-command first-Enter admission | Focused rapid-key-stream test and `burst_enter` set passed | Pending paste-burst text beginning with the slash protocol marker is materialized before Enter semantics. Adjacent `/panes` and `/docs` assertions prevent a command-name special case; live PFTQA-027 proves both commands dispatch with one Enter. |
| Native lifecycle ownership scope | 1 passed | Main and operator-created native panes are allowed; external Claude panes, managed `/spawn` members, and parent-controlled task workers are rejected before native archive/delete mutation. Live PFTQA-019 confirms both destructive commands leave Main untouched from a selected Claude pane. |
| Claude operator-pane delete and routing | 5 passed | Idle/active artifact cleanup, active cancellation, app-event deletion, managed-Claude-member refusal, and external-before-native input routing. Live PFTQA-020 covers idle and real active Opus 5 deletion plus cold-restart absence. |
| Whole-crew lifecycle and empty projection | 4 focused regressions passed; 14 lifecycle tests passed in the final focused run | Crew-boundary registry tests cover running managed-Claude cancellation and rejection of operator panes; the mixed native/Claude integration test proves complete removal with bound-Main preservation; the empty-projection regression prevents Main from being synthesized as a Nazgul after removal. Live PFTQA-021 covers confirmation cancel/accept, native removal, persisted Core/layout cleanup, disabled empty-state UI, and cold-restart absence. |
| Vault TUI focused set | 35 passed; provider-label mapping regression passed | Covers masked add, redacted async add/replace events, secure reveal with zeroization and no history/debug leak, safe delete confirmation, menu/action routing, inline-secret recall rejection, metadata-only output, status/list/delete command behavior, and strict provider-label-to-cache-key mapping. Live PFTQA-037 proves real add/replace/delete authentication coherence in one process. |
| Encrypted vault crate | 17 passed | Covers add/update/reveal/delete, duplicate and invalid labels, persistence, encrypted-at-rest index, distinct-label isolation, missing-entry failures, serialized concurrent writers, wrong-key fail-closed preservation, and corrupt-ciphertext rejection after reopen. |
| Async history redraw | 1 passed | A background history insertion requests a frame without user input; live PFTQA-023 status/list output appeared without a wake-up key. |
| Task Node non-blocking TUI boundary | 2 passed | Link and logout completion are delivered through app events after worker execution; live PFTQA-045 proves `/status` remains usable during a deliberately stalled link and encrypted logout. |
| Resume runtime authority | 2 focused app-server regressions and 1 TUI startup regression passed | The newest durable settings/turn context overrides a stale SQLite runtime projection, including exact explicit-default clearing. PFTQA-046 proves a real flagless resume and subsequent DeepSeek turn both retain `high`. |
| Slash control-plane admission | 4 focused regressions passed | Direct unsupported arguments, adjacent unknown command, queued unknown command, and external-Claude-pane control routing all remain local. PFTQA-049 proves both the `/panes <junk>` and `/pane` live boundaries produce zero turns. |
| Local approval response routing | 3 focused regressions passed | Local cancellation resolves visibly without a thread operation; the debug slash command uses that destination; ordinary thread-owned patch approval still emits the correct app-server operation. PFTQA-051 supplies the pre/post-repair live result. |
| PFTerminal completion identity | 3 binary-target regressions passed | One all-shell regression generates Bash, Zsh, Fish, PowerShell, and Elvish scripts for each CLI binary target and rejects stale Codex registration. PFTQA-052 verifies the rebuilt user-facing outputs. |
| Wallet unlock preflight | 7 focused regressions passed | Covers no-wallet preflight without secret solicitation, failed-passcode retry, fixed/custom policies, validation, continuation preservation, and narrow-terminal guidance. PFTQA-053 supplies live no-state unlock/lock evidence. |
| Retained `/spawn` provider recovery | Live invalid-key failure plus valid-key recovery passed | The exact retained Orc ID survives a provider 401, exposes the failure in status, remains task-addressable, and successfully completes a follow-up after cold restart. PFTQA-054 supplies the persisted-session evidence. |
| CLI doctor identity and Bash completion execution | Doctor/completion tests passed in all three binary targets; live checks passed | The JSON schema and detail labels use PFTerminal identity, installation checks inspect the PFTerminal executable, and a clean Bash process sources and exercises generated completion. Four non-installed shell execution cells and one unrelated cloud-sandbox test remain open. |
| Provider-menu search and local settings breadth | Provider-row search regression passed in the full TUI run; features 33/33; title identity 1/1; live final-artifact checks passed | Every provider row has a real search index, OpenRouter is discoverable, and the exercised settings/extension menus perform visible state or truthful gated/empty behavior. The full TUI suite remains red at 3,778/3,803 and remote mutations are not inferred from local menu use. |
| Provider catalogue output limits | 1 focused test passed | Standard and Fast Vercel GLM 5.2 records retain the provider-published 128K output maximum; Opus 5/Plan records retain 128K and direct DeepSeek Flash retains 384K. |
| Native-provider benchmark truthfulness | Fixture replay passed | The repaired harness counts the two real DeepSeek provider turns and rejects the two local Vercel metadata failures as zero; it also resolves PF stable/debug homes instead of assuming stock `CODEX_HOME`. |
| Persistent TUI thread materialization | 2 passed | A no-task Main is durable before a crew/layout references it, and a no-task operator pane is durable immediately after `thread/start`. Populated-crew startup preservation is separately covered by the PFTQA-013 regression and live qualification. |
| Non-TUI app-server lazy-start compatibility | 1 passed | The standard app-server client still receives a precomputed, unmaterialized thread path; immediate durability is scoped to persistent TUI/native-agent product threads. |
| MkDocs terminal viewer focused suite | 13 passed | Dev-tree and packaged fallback discovery, targeted pages, bounded content search, structured links, anchors, traversal rejection, and actionable recovery. |
| npm offline-doc staging | 1 passed | The staged main package contains `mkdocs.yml`, `docs/index.md`, and publishes both paths in package metadata. |
| External-pane loop adapter | 2 passed | The shared three-dispatch policy pauses chained external-pane automation and fresh input resumes it; acknowledgement without dispatch terminates the chain. |
| App-server native mailbox edge | 1 passed | A cold manager's requested terminal-result trigger is conservatively downgraded to queue-only, while stable message identity and later normal assignment remain intact. |
| Focused compaction regressions | 3 passed | Reviewed snapshots contain no synthetic continuation turn. |
| TUI crate | 3,733 passed, 9 skipped | Full `codex-tui` suite; all changed snapshots reviewed and accepted. |
| GPU market crate | 72 passed | Catalogue, immutable recipes, authorization, controller, persistence, and failure boundaries. |
| Telegram crate | 119 passed | Command, bridge, notification, permission, and continuity coverage. |
| Sandboxing crate | 60 passed | Platform sandbox policy/unit coverage available on this host. |
| State crate | 190 passed | Includes state-path and persistence reconciliation. |
| App-server protocol | 286 of 287 before regeneration; 4 focused schema tests passed after regeneration | The stale schema was regenerated; the focused stable/experimental schema checks then passed. |
| App server | 785 passed, 30 skipped in the final host-compatible run | A preceding 790-test broad run passed 781 and isolated five explicit bubblewrap/user-namespace host failures. The 24-test command-exec group was separately classified against the same host boundary. |
| Wallet crate | 11 passed | No live-chain purchase or spend was performed. |
| Wallet daemon crate | 8 passed | Local daemon behavior only. |
| Installer Python suite | 17 passed | Installer behavior and PF naming coverage. |
| Package Python suite | 18 passed | Target discovery and package layout coverage. |
| CLI crate | 844 of 846 | The two remaining cases are explicit host bubblewrap/loopback restrictions. Repaired doctor, home-isolation, update-label, logs-client, and absent-state-database cases pass. |
| Remaining modified-package group | Fully accounted | Initial run passed 1,240 of 1,270. After repairing stale API fixtures and the shared OpenAI mock-provider harness: app-server-client passed 30/30 with an 8 MiB test stack, config passed its focused fixture, exec passed 128/128, and the two host-keyring cases passed directly in 97.75s and 118.13s. |
| Schema determinism | Passed | Second generation reproduced config hash `f32a56d2…` and aggregate app-schema hash `c1d57e2a…`; all 4 schema parity tests passed. |
| Feature-manifest tooling | 4 passed; 0 unresolved | Final comparison contains 3 allowlisted inline-argument improvements (`clear`, `fork`, `new`), 0 unresolved differences, and 0 invalid allowlist entries. |
| Scoped lint | Passed | Final `just fix -p` completed for core, app server, app-server client, config, rollout, rollout trace, tools, and login after the earlier PF package lint passes. |
| Formatting | Passed | Final `just fmt` completed after lint. No automated tests were run afterward, as required by repository ordering. |
| Repository hygiene | Passed locally | `git diff --check` clean and no `.snap.new` files remain. |

The full core run produced 3,174 passes out of 3,303 tests, with 128 failures and one timeout on
this heavily loaded host. The failure population is dominated by unavailable bubblewrap network
namespace operations, loopback/network isolation, shell timeouts, and proxy timeouts. One
isolated `steer_interrupts_wait_agent...` fixture still times out even though the focused native
V2 wait tests pass. This is classified evidence, not a green full-core release gate, and must be
resolved or reproduced in the clean qualification environment before release.

No full-workspace test was run because that requires operator approval. Paid provider requests are
bounded and reconciled in the hands-on spend ledger; no wallet purchase or GPU rental was made.

### 12.3 Exact rebuilt binary evidence

The current post-format product-binary build commands were:

```text
cargo build -p codex-cli --bin pfterminal-debug
cargo build -p codex-cli --bin pfterminal
```

The debug build completed successfully after the PFTQA-049 slash control-plane repair and
produced a version `0.1.27` binary. The stable binary and wallet daemon were not rebuilt by this
narrower final command, so their last recorded post-format hashes are retained separately.

| Binary | SHA-256 |
| --- | --- |
| `target/debug/pfterminal-debug` | `05451548f4f4ec7da78a80fdf91a6cd6ba1f603c80e7a92a2fb3128cf4e6545a` |
| `target/debug/pfterminal` (last recorded; not rebuilt for PFTQA-023) | `9807f38414661753b8f8eb8b7451120f7453e298bfa2efcbaa928b33a37f0953` |
| `target/debug/pfterminal-walletd` (last recorded; not rebuilt for PFTQA-023) | `f8f5263bae77d3e861018682cbb90554be350b7e9629e8b24b5314f95c35b9d9` |

These are local reconstruction artifacts, not release artifacts. The user-local
`~/.local/bin/pfterminal-debug` remains a launcher script; it now exports
`PFTERMINAL_DEBUG_HOME` (the binary entrypoint's authoritative override) and the managed package
root before executing this rebuilt binary.

### 12.4 Prior partial TUI surface exercise (not feature acceptance)

The exact rebuilt `target/debug/pfterminal-debug` ran in a PTY with working directory
`/home/pfrpc/repos/goodalexander/isometricgame`. The final session-continuity pass used the fresh
isolated debug home `/tmp/pfterminal-debug-final2-20260801` through the binary-specific
`PFTERMINAL_DEBUG_HOME` variable. A localhost mock Ollama endpoint accepted discovery requests and
intentionally rejected inference with HTTP 503, proving the request boundary while guaranteeing
zero provider spend.

Observed live surfaces:

- startup displayed `PFTerminal (v0.1.27)`, the exact isometric-game directory, and the local
  zero-spend model;
- `/gpu` displayed RunPod and Vast.ai credential setup, qualified DeepSeek TP2 and GLM recipes,
  immutable revisions, hardware requirements, and capacity-search actions; no rental action was
  confirmed;
- `/panes` displayed `PFTerminal - Main`, native PF pane creation, Claude pane creation, and the
  managed `/spawn` crew section with Nazgul bound to the root;
- `/spawn` displayed standard crew creation and distinct Nazgul, Troll, Orc, and hierarchy-status
  workflows;
- `/wallet` contacted the local wallet daemon boundary and displayed Create wallet and Restore
  wallet without creating either;
- `/vault` displayed masked credential actions and a zero-credential isolated vault without
  exposing secrets. On this headless host, credential listing also surfaced the expected missing
  Secret Service/DBus backend error without leaking a credential.

Fresh session A, `019fbdd4-a6f9-7dd0-9634-f80b27550cb8`, persisted marker
`FINAL-TUI-SESSION-A-20260801`. Fresh session B,
`019fbdd5-4036-7710-9822-36d8b2e5fede`, persisted its distinct B marker. Session B was resumed
first and session A second; each rendered only its own expected history. `/status` on each fresh
and resumed session reported the matching UUID, the isometric-game directory, provider
`gpt-oss - http://127.0.0.1:11436/v1`, and `0 total (0 input + 0 output)` tokens. All inference
attempts were rejected locally; the mock was stopped and port 11436 was closed after the exercise.

The isometric-game repository already contained extensive user work before this verification.
Its pre- and post-exercise `git status --porcelain=v1 -z` hashes both equal
`6a105404db01ebc5d2df6e24d8e29099270caa6f5e60d067564e6367df7a56c9`, with the same 529 existing
entries. The feature-surface pass created one identifiable debug-only rollout in the normal debug
home before the binary-specific isolation variable was corrected; the two-session continuity
proof and all of its state are confined to the isolated debug home. Neither pass edited the
isometric-game workspace, stable PF home, or stock Codex home.

### 12.5 Release disposition

The reconstruction worktree does not yet demonstrate the complete product-preservation workflows.
It is not ready for release qualification until section 13 is executed and all resulting defects
are repaired. Release remains **NO-GO** because hands-on acceptance is incomplete, the operator
explicitly forbade a release, and sections 9 and 10 still require a clean immutable commit,
cross-platform packages, copied-home migration, upgrade/rollback evidence, and resolution or clean
environment reproduction of the classified suite failures.

No branch was pushed, no tag or release was created, no package was published, no installed
pointer was changed, and no paid provider or GPU resource was used.

### 12.6 DeepSeek V4 Flash source audit

On 2026-08-01, the public OpenRouter model API identified
`deepseek/deepseek-v4-flash-0731` as the current pinned Flash route, with canonical revision
`deepseek/deepseek-v4-flash-20260731`, a 1,048,576-token context window, a 384,000-token provider
completion limit, and catalogue rates of $0.14/M input, $0.0028/M cached input, and $0.28/M
output. The OpenRouter catalogue exposes that pinned model alongside
`deepseek/deepseek-v4-pro`; the Pro route was preserved rather than replaced.

The public Hugging Face model API identified
`7872f01b1d1fe23eabc4c98b48bffcef5a386062` as the current immutable commit for
`deepseek-ai/DeepSeek-V4-Flash-0731`, last modified 2026-08-01. The qualified TP2 H200 recipe pins
that exact commit in both its manifest and SGLang launch command and accounts for its
166,898,661,074-byte repository footprint. Recipe validation
requires an immutable model revision and image digest and rejects a launch command that does not
consume its declared model source and revision. The qualified SGLang runtime and image remain at
the previously validated `0.5.15.post1` deployment rather than being changed solely because a newer
runtime exists; such a runtime change requires separate paid hardware qualification.

Verification completed without a provider request or GPU rental: 181 model-provider,
models-manager, and GPU-market tests passed; the complete TUI project run passed 3,732 of 3,733
tests with only the intentionally changed OpenRouter picker snapshot pending, and the accepted
snapshot then passed on its focused rerun. No credential was read, no paid endpoint was called, and
no rental was created.

After the final debug rebuild, disposable Rust fingerprints, dependency objects, build-script
outputs, incremental state, and examples were removed while the three rebuilt executables were
preserved. The target directory fell from 35 GB to 5.5 GB and available disk space rose from
126 GB to 155 GB.

## 13. Mandatory full-product hands-on acceptance specification

This section supersedes any earlier statement in this document that a feature, reconstruction,
or release candidate is complete merely because it compiles, its unit tests pass, its menu opens,
its request dispatches, or a mock server receives a request. Sections 12.2 through 12.6 remain an
engineering ledger, but they are not product acceptance. In particular, the menu-only observations
in section 12.4 do not qualify `/spawn`, `/panes`, `/wallet`, `/vault`, `/gpu`, or `/docs` as usable.

The incident that triggered this gate is concrete: a real `/spawn` Nazgul pane accepted direct
input after one repair, but inherited `Workspace (Ask for approval)` while PFTerminal Main was in
`YOLO mode`. Its first non-allowlisted command entered broken bubblewrap and then opened an approval
dialog. The prior tests had covered pane creation and input dispatch but not the complete user
workflow. This is a release-blocking failure and proof that component-level smoke tests are not an
adequate product gate.

### 13.1 Acceptance vocabulary

A feature may be marked **PASS** only when all applicable layers below pass against the same exact
candidate artifact:

1. **Discover:** the released entry point is visible where expected, accurately named, and has
   useful help.
2. **Enter:** the user can open the workflow using the real CLI, TUI, Telegram, or service entry
   point.
3. **Act:** the user performs the feature's material operation. Opening a menu is not acting.
4. **Observe:** the result is visible, accurate, and attributable to the requested route, account,
   resource, pane, wallet, or session.
5. **Persist:** expected state survives switching panes, restarting PFTerminal, and resuming the
   session or service.
6. **Fail safely:** invalid credentials, denied permissions, unavailable services, cancellation,
   timeouts, and provider errors produce bounded, actionable behavior without state corruption,
   secret leakage, silent fallback, duplicate spend, or infinite retry.
7. **Recover:** the user can retry, reconnect, unlock, resume, or clean up without manual database
   surgery.
8. **Isolate:** stable PF, debug PF, stock Codex, test fixtures, and unrelated projects do not mutate
   one another.

The following evidence is never sufficient by itself:

- compilation or lint success;
- a unit, snapshot, mocked, or dispatch-only test;
- an enum member, source file, menu item, or non-panicking screen;
- a `200` from a fake provider that did not exercise the real adapter;
- a model response that does not prove the requested provider and billing route;
- a wallet, vault, GPU, Telegram, or Task Node menu that was opened without completing an action;
- an agent pane that was created but never received input, called a tool, completed a task, and
  survived switching and resume;
- a feature tested only in the default permissions mode when it behaves differently in other modes.

### 13.2 Candidate, environment, and evidence record

Every hands-on run must record:

- candidate Git commit, dirty-tree status, binary absolute path, version, SHA-256, build profile,
  platform, architecture, terminal, and timestamp;
- PF home path and whether it is fresh, migrated, stable, or debug;
- working directory and its before/after `git status` hash;
- command or keystroke sequence, selected settings, expected result, actual result, and cleanup;
- provider ID, exact provider model ID, wire API, auth mode, billing mode, service tier, effort,
  response/request IDs where available, token counts, latency, and measured charge for paid calls;
- pane/thread/session IDs and parent hierarchy for agent tests;
- wallet network and test account identity without seed phrase or private-key material;
- GPU provider, rental ID, hardware, image digest, model revision, endpoint model identity, timing,
  charge, and termination confirmation;
- redacted terminal transcript, relevant structured logs, screenshots when rendering matters, and
  an explicit `PASS`, `FAIL`, or `BLOCKED` disposition;
- tester identity and a second reviewer for paid, destructive, credential, migration, and release
  artifact gates.

Secrets must never appear in QA reports, shell history, TUI command history, logs, screenshots,
rollouts, fixtures, or Git. Destructive wallet and credential tests use disposable test accounts.
Paid provider and GPU tests require an operator-approved budget, but lack of authorization leaves
the applicable gate **BLOCKED**, never waived or converted to PASS.

### 13.3 Required settings matrix

Each feature checklist below is run in every applicable matrix cell. A feature owner must record
why a cell is inapplicable; “already unit tested” is not a reason.

| Dimension | Required cells |
| --- | --- |
| Binary/home | `pfterminal` with stable isolated home; `pfterminal-debug --yolo` with debug isolated home; packaged candidate binary |
| Home state | fresh home; copied 0.1.26 home; current user-shaped fixture; restart and resume |
| Permissions | read-only/approval; workspace-write/on-request; danger-full-access/never (`--yolo`) |
| Work state | idle; active model turn; active tool call; completed turn; interrupted turn |
| Session | fresh; renamed; resumed; forked; compacted; archived/unarchived where applicable |
| Pane | Main; PF user pane; Nazgul; Troll; Orc; Claude pane where supported; side conversation |
| Provider route | OpenAI; Claude Plan; Anthropic API; PF Plan; Kimi; Z.AI/GLM; DeepSeek direct; OpenRouter; Vercel; Baseten; Meta/local/rented GPU when release-visible |
| Input | plain text; multiline/paste; file mention; image where route declares vision; tool-producing task |
| Terminal | supported Linux terminal and tmux; narrow and wide resize; macOS and Windows packaged qualification |
| Network | normal; transient disconnect; timeout; authentication failure; non-retryable payment/entitlement failure |

Risk-based reduction is allowed only for combinations proven equivalent by the same typed boundary.
At minimum, every release-visible provider must complete one real inference and one real tool loop;
every distinct permissions mode must complete a real local command workflow; every pane type must
accept direct input when it is user-addressable; and every stateful feature must pass restart/resume.

### 13.4 Global TUI and turn-execution gate

- [ ] Start the exact candidate through both `pfterminal` and `pfterminal-debug`; confirm PF
      branding, correct home, version, working directory, model, provider, and permissions.
- [ ] Submit a normal prompt and receive a complete real-provider response with correct streaming,
      token accounting, latency, and route identity.
- [ ] Ask the model to run one allowlisted simple command and one non-allowlisted compound command.
      Verify behavior matches the selected permission mode.
- [ ] In `--yolo`, verify the non-allowlisted command runs directly: no bubblewrap, approval dialog,
      “escalation,” or silent downgrade may occur.
- [ ] In workspace/on-request, verify allowed workspace operations run and an outside-workspace write
      requests approval once; approve and deny paths must both work.
- [ ] In read-only, verify reads work and writes are denied or approved according to the displayed
      policy without corrupting the turn.
- [ ] Interrupt during streaming and during a tool call, then submit another turn successfully.
- [ ] Exercise queued input, steering, paste, file mention, image attachment, copy, scrollback, raw
      mode, terminal resize, and mouse/keyboard navigation.
- [ ] Trigger transient provider and tool failures; confirm bounded retry, no duplicate paid action,
      actionable error, and successful manual retry.
- [ ] Restart and resume; verify transcript, route, model, effort, permissions, cwd, usage, goal,
      pane hierarchy, and pending/finished state are correct.
- [ ] Confirm no feature calls the product Codex except bounded upstream attribution.

### 13.5 Provider, model, authentication, and billing gate

For every release-visible provider/model route in the generated feature manifest:

- [ ] The route appears only when its required authentication is available and is absent or clearly
      disabled with actionable setup when unavailable.
- [ ] Add or log in with the real credential through the released workflow; restart and confirm the
      credential remains usable without being displayed.
- [ ] Select the exact route and effort in `/model`; verify the header and `/status` agree.
- [ ] Complete a real prompt and real tool-calling turn. Record provider request identity and prove
      that no fallback provider or model handled it.
- [ ] Exercise supported reasoning modes, context limits, output limits, JSON output, tools, vision,
      service tiers, caching, and plan eligibility from typed catalogue data.
- [ ] Reject an invalid key, expired login, unsupported model, unsupported effort, and insufficient
      balance before unintended spend; recovery must not require restart unless documented.
- [ ] Force a switch failure and verify displayed, persisted, sampling, and compaction routes remain
      the previously acknowledged route.
- [ ] Compare measured token counts and charge with catalogue pricing and provider records. Unknown
      pricing blocks automatic paid selection.
- [ ] Verify non-retryable authentication, entitlement, and payment failures do not retry paid work.
- [ ] Restart and resume; the exact provider/model/auth/billing route must persist.

Required named route coverage:

- [ ] OpenAI login/API route, including Sol, Luna, Terra, and supported service tiers.
- [ ] Claude Plan login route and direct Anthropic API-key route, including Opus/Fable and plan
      variants. Catalogue output limits must come from authoritative metadata, not invented caps.
- [ ] PF Terminal plan route, entitlement, usage display, exhausted-plan behavior, and renewal/reset.
- [ ] Kimi Code/direct route and current Kimi model, with runtime and cost evidence.
- [ ] Z.AI/GLM route and current GLM model.
- [ ] DeepSeek direct Responses route using `deepseek-v4-flash` / DeepSeek-V4-Flash-0731.
- [ ] OpenRouter route using exact ID `deepseek/deepseek-v4-flash-0731`, plus other release-visible
      OpenRouter models; direct and OpenRouter DeepSeek results must remain distinguishable.
- [ ] Vercel AI Gateway, Baseten, Meta, Ambient, local/OSS, and custom compatible routes that remain
      release-visible.
- [ ] Provider picker pagination/navigation: adding a key makes the provider visible without
      mislabelling another provider, losing focus, or requiring unrelated credentials.

### 13.6 `/spawn` and native agent hierarchy gate

No `/spawn` workflow passes until a human has used the resulting pane to complete real work.

Architecture acceptance is binary: the same OpenAI Codex Core agent graph and collaboration APIs
used by native Codex subagents must create, address, wake, resume, interrupt, complete, and reuse
PFTerminal native crew members. The TUI may present these operations but may not substitute its own
native queue or lifecycle. Passing component tests for thread creation or mailbox delivery does not
qualify the feature if startup can erase the PF projection of that graph.

- [x] Serialize every first-party OpenAI `collaboration.*` tool definition and compare it with the
      pinned upstream contract. Names, namespace, descriptions, encrypted annotations, parameter
      schemas, required fields, additional-property policy, and output schemas must match exactly;
      PF route, billing, role, and presentation fields must remain outside the reserved schema.
      PFTQA-025 covers the focused plan/profile tests, provider-bound request integration test, and
      live endpoint acceptance.
- [ ] On at least one non-OpenAI Responses-compatible route, inspect the capability-selected
      collaboration serialization and complete spawn, message, and follow-up work. The literal task
      must be visible to the child where encrypted native arguments are unsupported, while the same
      Core operation identity, target, trigger behavior, and durable graph remain authoritative.
      PFTQA-029 proves the cross-provider follow-up leg into OpenRouter/Grok, including the literal
      task and same Core mailbox. Provider-neutral spawn plus running-turn `send_message` on that
      route remain required.
- [x] Run an authenticated first-party OpenAI lifecycle from a clean debug home and prove the
      initial request reaches inference without a reserved-schema 400. Create a child, deliver a
      running-turn message, complete it, start a second turn through `followup_task`, list/wait for
      it, interrupt a separate disposable turn, and reuse the same child identity afterward.
      PFTQA-025 and PFTQA-026 prove the exact reserved schema, encrypted follow-up, running-turn
      steering, listing/waiting, target-only interrupt, same-child reuse, and cold-process
      continuation on authenticated OpenAI Luna.
- [x] From PFTerminal Main, open `/spawn`; create a Nazgul, a Troll under that Nazgul, and at least
      three Orcs under the Troll using distinct supported provider/model routes.
      PFTQA-028 proves fresh manual creation. PFTQA-029 proves the standard mixed-provider crew with
      Luna, Terra, and OpenRouter/Grok Orc routes and retained exact identities.
- [x] Create the standard crew and manually create each role. Both paths must produce durable,
      correctly named hierarchy entries without collisions or “different Nazgul root” corruption.
      PFTQA-028 proves the complete fresh manual path; PFTQA-029 proves current-artifact standard
      crew creation and reuse.
- [x] Immediately type directly into every newly created Nazgul, Troll, and Orc pane. No PFTerminal
      `/spawn` pane may say direct input is disabled.
      PFTQA-027 proves the retained hierarchy and PFTQA-028 proves fresh current-artifact Angmar,
      Burzum, and Snaga. QA rejected three markers mistakenly submitted on Main after background
      creation and counted only role-local responses after exact `/panes` selection.
- [ ] In every permissions mode, ask each role to run `pwd`, inspect Git history with a compound
      command, read a file, and make a disposable workspace edit. The effective permission shown by
      `/status` and the actual command behavior must match the parent/user selection.
      PFTQA-031 proves the workspace-write/never slice for Main, Nazgul, Troll, Orc, a managed
      Nazgul -> Troll -> Orc follow-up, and flagless cold reuse of the exact Main and Orc. It also
      proves outside writes are denied and leave no artifact. PFTQA-032 proves the corresponding
      workspace-write/on-request slice, including real approval dialogs, rejected-write absence,
      non-OpenAI context authority, native managed follow-up, and flagless cold restoration.
      PFTQA-033 proves read-only/on-request across every retained role, rejected workspace writes,
      native managed read-only follow-up, and flagless cold restoration. PFTQA-034 proves Approve
      for me on exact Snaga uses a DeepSeek-bound Guardian, and separately proves Full Access
      cancel/confirm, approval-free outside execution, and flagless restoration before resetting
      the live hierarchy to Workspace/Ask. Additional roots/profiles, remaining provider/pane
      combinations, and any unfilled command-variety cells keep this row from feature-PASS.
- [x] Specifically launch `pfterminal-debug --yolo`, create a new pane, and run a non-allowlisted
      compound command. It must not invoke bubblewrap or open an approval dialog. PFTQA-035 proves
      the literal launch, no-task persistent pane creation, YOLO/Full Access projection, one real
      outside-write + read + Git compound command with exit 0, marker cleanup, and explicit safe
      restoration of Main, the same user pane, and exact retained Snaga to Workspace/Ask.
- [ ] Give a Nazgul a real planning/review task, have it create or manage a Troll, have the Troll
      assign independent work to Orcs, and receive consolidated results back in the expected pane.
      PFTQA-027 proves the existing Nazgul -> Troll -> Orc native follow-up and result path without
      replacement or shutdown, but its protocol marker task is not a real planning/review task.
- [ ] Exercise `spawn_agent`, `send_message`, `followup_task`, `wait_agent`, `interrupt_agent`, and
      agent listing through the actual product hierarchy where exposed.
- [ ] Inspect the provider-bound request for assignments to Nazgul, Troll, and Orc. The user task
      must remain the literal mailbox assignment; persistent role doctrine must come from the
      built-in role configuration, and live application context must contain roster/routing state
      without a second synthetic task-role prompt. OpenAI may receive the native `AgentMessage`;
      every other provider must receive a visible external-input/user message containing that same
      assignment, never a taskless synthetic `Continue.`.
- [ ] Complete two sequential real tasks on the same Troll and the same Orc. After task one, each
      must remain registered as completed/idle at the same `ThreadId`, canonical `AgentPath`, crew
      ID, member ID, name, role, route, and permissions; task two must use native follow-up and must
      not spawn a replacement or issue shutdown.
- [ ] Put the same retained member into `Interrupted`, `Errored`, and `Unloaded` states in separate
      runs. Each state must preserve the same durable identity and pane; a later direct assignment
      or native follow-up must recover that member without creating a replacement. Only an explicit
      confirmed removal may produce `Shutdown` or erase graph/layout membership.
- [ ] While an Orc is running, use native `send_message` to add relevant information and prove it
      reaches that running turn. After it completes, use native `followup_task` on the same path and
      prove a new turn starts exactly once.
- [ ] Prove one Orc terminal answer produces exactly one Core terminal-result message at its Troll
      and one Troll terminal answer produces exactly one at its Nazgul. Neither completion may
      require TUI tag parsing, polling prose, or a TUI-owned report-delivery queue.
- [ ] Exercise the native Core manager loop breaker end to end: activate the same manager with
      fresh work, cause three consecutive terminal-result-only turns to dispatch follow-up work,
      prove the fourth result is durably queued without another paid turn, then send fresh operator
      input and prove that same `ThreadId` resumes. Also prove acknowledgement without dispatch
      terminates the chain and a cold-restored manager starts in queue-only quarantine.
- [ ] Cold-restart the process with completed Nazgul, Troll, and Orc members. Reconcile the UI from
      the Core graph, resume/materialize each through Core, and deliver a new stable mailbox message.
      Instrument the app-server boundary and prove no direct `turn/start` fallback admits the task.
      PFTQA-028 proves exact fresh-crew restoration and new work on the original Snaga; delivery to
      every restored role and a current-run boundary trace remain before this row can pass.
- [x] Reproduce the PFTQA-013 startup boundary with a populated owner layout containing a
      `CrewSpec`, Nazgul binding, parent map, member endpoints, and durable Main -> Nazgul -> Troll
      -> Orc Core threads. Before any picker or task is opened, restart and resume the exact Main.
      Prove startup reads and reconciles that state before any persistence write; current and
      recovery snapshots remain recoverable; no default/empty projection is written; and
      `/spawn`, `/agent`, `/subagents`, and `/panes` show the same exact IDs, paths, names, roles,
      routes, statuses, and edges.
- [ ] Inject a missing rollout, unreadable layout, invalid edge, and temporarily unavailable Core
      member separately during cold restore. Each case must preserve the last valid layout, expose
      a typed degraded/recovery state, and avoid destructive persistence, implicit Main-as-Nazgul
      rebinding, duplicate creation, or silent hierarchy loss.
- [ ] Force mailbox target-not-found, duplicate message ID, busy target, interrupted target, and
      capacity exhaustion. Each condition must remain bounded and typed; there must be no regex
      route, duplicate provider turn, competing TUI queue, silent task loss, or identity replacement.
- [x] Compare `/spawn`, `/agent`, `/subagents`, and `/panes` for the same live crew. Thread IDs,
      canonical paths, hierarchy, lifecycle status, and completion state must agree because all
      four surfaces project the same Core authority.
      PFTQA-013 proves this after migration/restoration; PFTQA-028 proves it again on a freshly
      created and then cold-restored current-artifact hierarchy.
- [ ] Verify explicit provider/model choices are exact for every child; no allowlist or hidden block
      may reject a configured provider unless an operator-created policy explicitly does so.
- [ ] Verify missing API key, 401, provider timeout, insufficient balance, unsupported model, and
      missing output-limit metadata fail in the affected pane without damaging siblings or Main.
- [ ] Switch away during streaming/tool use and return; output, prompt state, approvals, TPS, role,
      model, and status must belong to that pane only.
- [ ] Interrupt, retry, unload/reopen, and resume each role without removing it. Parent/child
      identity, names, persistent role prompt, route, permissions, messages, and completed work
      must persist after full process restart. Exercise permanent close only on a disposable crew
      through the separate human-confirmed removal workflow; a parent agent must not infer that
      authorization from task completion.
- [ ] Verify a parent-controlled low-level task-only `spawn_agent` worker remains distinguishable
      from a human-addressable `/spawn` pane. The internal restriction must never leak onto a
      PFTerminal-created pane.
- [ ] Confirm costs and usage are attributable per separately billable child and total correctly.
- [ ] Complete all crew turns and exit PFTerminal without closing the crew. Restart and prove every
      completed Nazgul, Troll, and Orc remains visible, directly writable, prompt-correct, and
      reusable on its original identity. Then, on a disposable crew only, explicitly confirm the
      ownership-aware removal workflow and verify that Core closure leaves no orphan worker, stuck
      turn, duplicate hierarchy, runaway retry, or unbounded log growth.

### 13.7 `/panes`, PF user panes, Claude panes, and navigation gate

- [ ] Open `/panes` from idle Main and during an active turn; list Main, PF user panes, all native
      agents, and configured Claude panes with correct labels, models, roles, status, and hierarchy.
      Search each generated pane kind by displayed name and durable identity; filtering must retain
      the exact selectable row rather than producing no match or selecting a similarly named pane.
  - [x] A cold-restored native Nazgul root is searchable by its displayed name and opens the exact
        existing human-addressable thread (PFTQA-014). The same live matrix also found Main, Troll,
        and Orc by displayed name and found Nazgul and Troll by exact durable thread ID. PF
        user-pane, Claude-pane, duplicate-name, active-turn, and remaining durable-ID cases remain
        pending.
- [ ] Create a PF user pane, select provider/model/effort, type a task, run a real tool command,
      receive a response, switch away, and return without losing input or output.
- [ ] Create a Claude pane using Claude Plan and each release-visible API-key gateway profile;
      complete a real prompt and tool command and verify exact authentication/provider route.
- [ ] Switch among Main, PF pane, Nazgul, Troll, Orc, and Claude pane by picker and keyboard
      navigation while turns are idle and active. Direct input must reach only the selected pane.
- [ ] Verify `/agent`, `/subagents`, and `/panes` views agree on thread identity and liveness.
- [ ] Rename panes, create multiple similarly named panes, resize the terminal, use tmux, and confirm
      selection never jumps to another thread.
- [x] Archive one pane with work active and one idle; verify confirmation, cleanup, parent fallback,
      transcript retention, and sibling continuity.
  - [x] A zero-token idle operator pane archived after explicit confirmation; its rollout moved to
        `archived_sessions`, its layout membership was removed, restart fell back to Main, and the
        pane did not return as a ghost row (PFTQA-016).
  - [x] A real active operator-pane tool was aborted by confirmed `/archive`; the archived rollout
        retained the interrupted turn, owner membership was removed, the process exited, restart
        returned to Main, and exact-ID search did not restore the pane (PFTQA-017).
- [x] Delete idle and active operator panes. Confirmation permanently removes rollout/state,
      interrupts active work without an orphan process, removes layout membership, falls back to
      Main, exits, and remains absent after cold restart (PFTQA-018).
- [ ] Exercise archive/delete lifecycle for Claude panes and managed crew with their correct
      ownership semantics.
  - [x] Confirmed `/archive` and `/delete` from a selected Claude pane fail before touching Main;
        managed crew and parent-controlled workers are protected by the same ownership boundary
        (PFTQA-019).
  - [x] Dedicated `/delete` permanently removes idle and active operator-created Claude panes,
        cancels an active Claude process, removes verified artifacts and layout membership, falls
        back to Main, and remains absent after cold restart. Direct deletion of a managed Claude
        crew member is rejected; Claude `/archive` is not a supported external-pane lifecycle
        operation (PFTQA-020).
  - [x] Remove a complete managed crew through one ownership-aware whole-crew action and verify
        native/Claude member shutdown, Core graph cleanup, durable `CrewSpec` reconciliation,
        sibling/Main safety, and cold-restart absence (PFTQA-021). The live source crew exercised
        native Nazgul/Troll/Orc deletion and cold absence; the mixed integration test exercised a
        running managed Claude member and preservation of a user-owned bound Main root.
- [ ] Restart the entire process and resume the layout. Every retained pane must restore its exact
      transcript, route, cwd, permissions, role, hierarchy, and input capability.
- [x] A stale or failed operator pane shows a disabled searchable row with an inline reason, is not
      retried every time `/panes` opens, and does not block switching to a healthy sibling or wedge
      the global TUI (PFTQA-015). Managed-crew and Claude-pane failure injection remain covered by
      their respective broader matrix rows.
- [x] If `/pane` existed in the released manifest as an alias, verify it; otherwise verify help
      consistently advertises the canonical `/panes` command and does not invent a dead alias.

### 13.8 `/docs` terminal documentation gate

- [x] Run bare `/docs`; the real terminal viewer must load the documentation index, not a placeholder.
- [x] Open a targeted existing page with `/docs <page>` and a nested page/path; verify title,
      headings, code blocks, lists, links, tables, and navigation.
- [x] Search, follow internal links, go back/forward, scroll, page, resize narrow/wide, and return to
      the chat without losing composer or transcript state.
- [x] Open docs while a model turn is active and from every pane type where the command is advertised.
- [x] Request a missing/ambiguous page and malformed argument; receive actionable results without
      panic, blank screen, shell escape, or state corruption.
- [x] Test with packaged/offline documentation and with the development docs tree. Missing MkDocs or
      assets must produce a clear recovery instruction.
- [x] Close with every advertised exit key and confirm terminal mode, mouse mode, alternate screen,
      clipboard, and redraw are restored.
- [ ] Verify all release-visible feature pages describe current PF behavior and branding, especially
      wallet, vault, GPU, providers, spawn, panes, Telegram, Task Node, permissions, and recovery.

### 13.9 Wallet and plan-purchase gate

Use a disposable wallet and, for an actual plan purchase, an operator-approved minimal-value
mainnet account. Merely reaching the daemon or seeing Create/Restore is not acceptance. The
released 0.1.26 floor exposes SOL and USDC balances and an exact x402 USDC plan-payment flow; it
does not expose a PFT-token balance or a general arbitrary-destination send workflow. Those
invented requirements are not 0.1.27 preservation gates.

- [x] Start with no wallet; run bare `/wallet` and `/wallet status`; verify accurate locked/uncreated
      state and daemon health.
- [x] Create a wallet through the secure modal, complete the protected backup flow, and verify the
      product does not write recovery/private material to normal command history, logs, rollouts,
      or the isolated home outside encrypted wallet state. The QA harness nevertheless captured
      one secure-view frame in operator-visible tool output; that evidence-handling error is recorded
      below and the zero-balance fixture was removed.
- [x] Lock and unlock with correct and incorrect credentials; test a one-minute timeout/autolock and
      full PFTerminal process restart.
- [x] Restore a second isolated disposable wallet from its supported recovery material and verify the
      exact expected public address.
- [x] Query SOL and USDC against Solana mainnet, reconcile the zero-balance fixture with an independent
      RPC source, and verify the UI does not invent funds.
- [x] List live PF inference plans and verify exact USDC amounts, insufficient-funds admission, and the
      no-transfer existing-plan recovery failure path.
- [x] Verify destructive and billable confirmation dialogs cannot be committed by bare number keys:
      they render no numeric accelerators and require explicit navigation plus Enter. Cancel must
      preserve the wallet; explicit remove must remove only the disposable local wallet and plan
      credential while leaving on-chain state untouched.
- [ ] Fund an operator-approved disposable wallet, purchase the minimal plan, verify transaction and
      service receipts, entitlement activation, balance/usage update, and exact billing source.
- [ ] Exercise cancellation before signing plus simulated/provider-side payment failures, RPC timeout,
      dropped transaction, idempotent retry, and post-confirmation reconciliation; no ambiguous charge
      or duplicate purchase may result.
- [ ] Restart the wallet daemon and PFTerminal after a funded purchase; verify wallet identity, lock
      state, balances, receipts, and plan entitlement persist.
- [ ] Verify backup/restore and migration from a copied 0.1.26 wallet state; originals remain untouched.
- [ ] Run every released `/wallet create|restore|unlock|lock|status` inline form and equivalent menu
      action. PFTQA-053 now covers all five inline command admissions in the no-wallet state,
      including masked create/restore cancellation and repaired unlock/lock truthfulness; positive
      create/restore/unlock/lock duplication still remains.
- [x] Confirm binary-specific debug-home isolation using `PFTERMINAL_DEBUG_HOME`; validate that stable,
      debug, and disposable test homes do not read or mutate one another's wallet files.

### 13.10 Vault and provider credential gate

- [x] Open bare `/vault`; verify current credential labels and providers without secret values.
- [x] Add a disposable credential using `/vault credential add` and the masked modal. Inline secret
      text must be rejected and removed from slash-command recall.
- [x] List, inspect metadata, use, update, copy/reveal through the explicit protected action, and
      delete the disposable credential with confirmation.
- [x] Verify the credential actually authenticates one real provider request after add/update and
      fails after deletion.
- [ ] Restart PFTerminal and the OS credential backend; verify persistence, unlock/recovery behavior,
      and correct backend selection.
- [x] Exercise Secret Service/keyring unavailable, encrypted-file fallback, wrong key, corrupt store,
      duplicate label, invalid label, timeout, cancellation, and concurrent access. PFTQA-023 and
      PFTQA-044 cover fallback/timeout/cancellation; PFTQA-050 closes wrong-key, corrupt-store, and
      concurrent read/modify/write safety; crate regressions cover duplicate and invalid labels.
- [ ] Exercise OS-keyring restart, partial-write and permission failures, recovery/rotation, and
      concurrent operations from separate live PFTerminal processes.
- [ ] Search terminal output, logs, rollouts, SQLite, history, crash reports, process arguments, and
      Git changes for the test secret; zero unintended copies are allowed.
- [ ] Verify clipboard copy is clearly disclosed and cleared where supported; no background pane or
      agent may retrieve a vault secret without the intended capability.
- [ ] Run all released inline vault actions and their menu equivalents; usage errors must not echo a
      supplied secret.

### 13.11 GPU rental and hosted-model gate

An actual bounded rental is required before the GPU feature can pass. Catalogue display or mocked
launch is not acceptance.

- [ ] Add disposable/limited RunPod and Vast.ai credentials through `/providers` or `/vault`; verify
      both valid and invalid credential behavior.
- [ ] Open `/gpu` and `/gpu status`; verify catalogue entries, provider availability, hardware,
      region, price, image digest, model source, immutable revision, tensor parallelism, context,
      output, and estimated total are accurate.
- [x] Confirm the only selectable **qualified official-weight** DeepSeek rental is
      `deepseek-ai/DeepSeek-V4-Flash-0731` pinned to revision
      `7872f01b1d1fe23eabc4c98b48bffcef5a386062`, using the qualified TP2 recipe. Experimental
      third-party GGUF routes remain visible and explicitly labelled experimental; preservation does
      not permit silently deleting them.
- [ ] Search capacity, select an operator-approved bounded rental, review charge/destructive
      confirmation, launch it, and observe every state transition through ready or a bounded failure.
- [ ] Authenticate to the resulting endpoint, verify the served model/revision independently, select
      it in `/model`, and complete real text, tool, long-context, and declared vision/JSON workflows.
- [ ] Restart PFTerminal while the rental remains active; `/gpu status` must reconcile the same rental
      without duplicate launch or lost credentials.
- [ ] Exercise provider capacity loss, image pull failure, model load failure, health timeout, stale
      endpoint, controller restart, and credential expiration with actionable cleanup.
- [ ] Run `/gpu stop <id>` and verify serving stops without unintended rental destruction where that
      distinction exists; recover serving if supported.
- [ ] Run `/gpu terminate <id>`, verify provider-side termination and billing cessation independently,
      then confirm local state reaches terminal cleanup and no secret/process/log remains.
- [ ] Test cancellation at every pre-charge stage and idempotent repeated stop/terminate.
- [ ] Record actual startup time, throughput, first-token latency, stability, total charge, and cleanup.

### 13.12 Telegram remote-control gate

- [ ] Use `/telegram connect` with a disposable bot token; validate token, authorize the intended
      chat, reject a different chat, and ensure the token is stored only in the vault.
- [ ] Start, query status, stop, restart, and disconnect through both TUI and CLI/service workflows.
- [ ] Send real Telegram text, multiline, command, attachment/image, and cancellation messages; verify
      exactly one corresponding PF turn and correctly chunked/escaped replies.
- [ ] Select an exact provider/model/effort and permissions policy locally; remote work must use and
      report the same route and policy with no silent fallback.
- [ ] Run a safe tool task remotely in each permission mode, including approval/denial where supported.
      `--yolo` must not silently become workspace-write and Telegram must not bypass a restrictive mode.
- [ ] Restart connector/PFTerminal/network during a turn; verify bounded delivery retry, deduplication,
      resume, session identity, and no cross-chat leakage.
- [ ] Exercise invalid/expired token, revoked bot, Telegram rate limit, oversized response, unsupported
      attachment, unauthorized chat, stale approval, and provider failure.
- [ ] Disconnect and prove subsequent messages cannot control PF Terminal while local TUI remains usable.

### 13.13 Task Node, goals, memory, skills, hooks, MCP, apps, and plugins gate

- [ ] `/tasknode link|status|tasks|outstanding|task <id>|verification|refused|rewarded|request|context|chat|requests|balance|rewards|logout` each completes its real read or mutation against a test account.
- [x] Keep the TUI interactive while Task Node link/logout network and encrypted-vault operations
      are pending; surface bounded completion or failure without a wake-up key (PFTQA-045).
- [ ] Task acceptance, evidence submission, verification response, refusal/retry, reward display, wallet
      binding, context edit, chat continuity, authentication failure, restart, and logout all work.
- [ ] `/goal` create/view/update/complete and long-running continuation survive compaction, pane switch,
      interrupt, restart, and resume without duplicate continuation.
- [ ] `/memories` enable/disable/use/generate behavior is observable across turns and projects; memory
      drop/update maintenance is either functional and debug-only or absent from release UI—never a stub.
- [ ] `/skills` discovers system, user, repo, and plugin skills; selects the correct skill, reads its
      instructions, invokes required tools, handles missing dependencies, and refreshes after change.
- [ ] `/hooks` lists, enables/disables, approves, executes, times out, reports failure, and respects
      trust/bypass settings without hidden command execution.
- [ ] `/mcp` and `/mcp verbose` connect to a real test MCP server, list tools/resources/auth, execute a
      tool, handle elicitation, reconnect, and isolate server failure.
- [ ] `/apps` connects, authenticates, searches/reads/writes only within granted scope, handles revoked
      auth, and does not expose unavailable connectors.
- [ ] `/plugins` lists, installs/enables/disables/updates/removes a disposable plugin; its skills, MCP,
      apps, ordering, cache refresh, restart persistence, and failure rollback work.

### 13.14 Complete slash-command acceptance inventory

Every command below requires discoverability, real invocation, visible effect, invalid-input behavior,
cancel/back behavior, restart/persistence where stateful, and use during an active turn or side
conversation exactly where the UI advertises it. Detailed gates above are additive.

- [ ] `/model`: select real provider/model/effort/service tier, complete a turn, reject bad route,
      preserve acknowledged route across restart.
- [ ] `/gpu`: complete section 13.11.
- [ ] `/ide`: attach live IDE selection/open-file context and prove it reaches the model. PFTQA-051
      closes the absent-IDE boundary with actionable VS Code/Cursor guidance and zero provider turn.
- [ ] `/permissions`: change all supported profiles, run commands proving enforcement, persist correctly.
      PFTQA-031 through PFTQA-033 prove workspace/never, workspace/on-request, and
      read-only/on-request enforcement plus flagless restoration on the retained hierarchy.
      PFTQA-034 proves direct DeepSeek Approve for me uses a provider-valid Guardian, and proves the
      Full Access cancel/confirm, approval-free execution, and flagless-restoration workflow on
      exact Snaga. Remaining providers, roles, profiles, escalation/denial, `/approve`, additional
      roots, and invalid-transition cases keep this command open.
- [ ] `/keymap` and `/keymap debug`: remap, use, detect conflict/invalid config, restore defaults.
- [ ] `/vim`: enter/normal modes, edit/submit multiline input, switch panes, persist setting. PFTQA-051
      proves live enable, Normal-mode projection, command entry through Insert mode, and disable.
- [ ] `/setup-default-sandbox`: complete supported platform setup and verify resulting enforcement;
      hide or explain accurately where unsupported.
- [ ] `/sandbox-add-read-dir <path>`: grant only the selected root, prove read/no-write boundaries,
      reject invalid paths, and persist as documented.
- [ ] `/experimental`: toggle each release-exposed experiment, verify its behavior and rollback.
- [ ] `/approve`: retry only the intended recent auto-review denial once; reject stale/wrong denial.
- [ ] `/memories`: complete memory cases in section 13.13.
- [ ] `/skills`: complete skill cases in section 13.13.
- [ ] `/import`: import setup/project/recent chats from a disposable Claude Code fixture, verify mapping,
      duplicate handling, cancellation, rollback, secret safety, and resume.
- [ ] `/hooks`: complete hook cases in section 13.13.
- [ ] `/review [args]`: review actual dirty changes, select base/commit/custom instructions, navigate
      findings, interrupt, and return to normal work.
- [ ] `/rename <name>`: rename, reject empty/invalid/duplicate edge cases, show name in picker/resume.
- [ ] `/new [name]`: create isolated session, preserve prior session, route/cwd/settings, and resume both.
- [ ] `/archive`: confirm/cancel, archive real session, remove from normal picker, unarchive via CLI.
- [ ] `/delete`: cancel first, then delete disposable session, verify removal and no unrelated deletion.
- [ ] `/resume [id|name]`: resume exact session with transcript, cwd, route, permissions, panes, and goal.
- [ ] `/fork [name]`: fork at current point, diverge histories, preserve lineage, route, tools, and state.
- [ ] `/app`: on supported platforms open/continue in PF Desktop; handle missing install and unsupported OS.
- [ ] `/init`: create/update AGENTS.md only after confirmation, preserve existing instructions, obey cwd.
- [ ] `/compact`: compact a long real conversation, preserve canonical facts/tool state/route, continue work.
- [ ] `/plan [prompt]`: enter Plan, ask questions, revise/approve plan, exit to execution without executing
      while still in Plan.
- [ ] `/goal [objective]`: complete goal cases in section 13.13.
- [ ] `/agent` and `/subagents`: list/select real threads, show liveness/role/model, navigate and return.
- [ ] `/spawn [status|nazgul|troll|orc]`: complete section 13.6.
- [ ] `/orchestrate`: attach/manage real native pane whips, dispatch work, reconcile status, recover failure.
- [ ] `/tasknode`: complete section 13.13.
- [ ] `/panes`: complete section 13.7.
- [ ] `/side [prompt]` and `/btw [prompt]`: create ephemeral fork, complete a real turn/tool call, return
      without contaminating parent history, and handle parent interruption/resume.
- [ ] `/copy`: copy exact last Markdown response including code/unicode; handle no response/headless clipboard.
- [ ] `/raw on|off`: toggle, select/copy long output, preserve input and rendering, reject invalid argument.
- [ ] `/diff`: show tracked/untracked/binary/renamed changes accurately in real dirty repo; no mutation.
- [ ] `/docs [page]`: complete section 13.8.
- [ ] `/mention`: find/select text and image files, handle spaces/large/ignored/missing files, prove attachment.
- [ ] `/status`: accurately report model/provider/cwd/permissions/account/mode/session/tokens/limits for every pane.
- [ ] `/usage [daily|weekly|cumulative]`: reconcile real usage/reset data; handle unavailable/malformed feed.
- [ ] `/debug-config`: show effective layers, sources, constraints, CLI overrides, and secrets redacted.
- [ ] `/title`: enable/reorder/disable fields, observe terminal title live, persist and restore.
- [ ] `/statusline`: enable/reorder/disable fields, observe correct per-pane live values, persist and restore.
- [ ] `/theme`: preview/select/persist themes, render code/diff/markdown, recover invalid custom theme.
- [ ] `/pets` and `/pet`: choose/hide/persist pet, verify narrow terminal and no input/render interference.
- [ ] `/mcp [verbose]`: complete MCP cases in section 13.13.
- [ ] `/apps`: complete app cases in section 13.13.
- [ ] `/plugins`: complete plugin cases in section 13.13.
- [ ] `/providers`: add/update/remove/login/logout real credentials and plans; provider visibility and routing
      must update transactionally without leaking secrets.
- [ ] `/telegram [status|connect|start|stop|disconnect]`: complete section 13.12.
- [ ] `/logout`: confirm/cancel, remove intended account auth only, preserve vault/wallet unless documented,
      prove provider is unusable and re-login works.
- [ ] `/wallet [status|create|restore|unlock|lock]`: complete section 13.9.
- [ ] `/vault`: complete section 13.10.
- [ ] `/quit` and `/exit`: handle active/idle turns, confirmation, terminal restoration, background-service policy,
      and subsequent resume.
  - [x] PFTQA-069 proves both commands exit cleanly from the same idle native Claude Plan pane and
        that successive cold resumes retain the exact user pane and complete managed crew without
        residual PFTerminal/background processes. Active-turn, confirmation, background-policy,
        non-tmux, signal, and platform cells remain open.
- [ ] `/feedback`: preview/redact logs, cancel, submit to test endpoint or approved channel, handle
      offline/error. PFTQA-051 proves category selection, proposed-attachment disclosure, cancellation
      with no upload, and explicit configuration-disabled behavior; submission remains intentionally open.
- [ ] `/rollout` (debug): print the exact existing rollout, verify permissions/redaction, hide in release.
- [ ] `/ps`: list real background terminals with command/status/cwd/thread ownership accurately.
  - [x] PFTQA-066 proves one real native Claude Plan background command remains discoverable after
        turn interruption and exposes the exact command until explicitly stopped. Cross-pane,
        multiple-command, cwd, restart, and platform cells remain open.
- [ ] `/stop` and `/clean`: stop real background terminals idempotently without killing unrelated processes.
  - [x] PFTQA-066 proves `/stop` removes both Bash and child `sleep`, leaves `/ps` empty, is
        idempotent on a second call, and leaves the owning pane usable. PFTQA-068 separately covers
        `/clean` with two jobs; unrelated-process, restart, and platform cells remain open.
  - [x] PFTQA-068 proves the `/clean` alias removes two distinct Bash process groups and both child
        `sleep` processes, leaves `/ps` empty, is idempotent on a second call, and leaves the owning
        pane usable. Cross-pane, unrelated-process, restart, partial-failure, high-count, and
        platform cells remain open.
- [ ] `/clear [name]`: start clean session/UI with expected persistence and no deletion of prior transcript.
- [ ] `/personality`: select each supported personality, observe real response behavior, persist per settings.
- [ ] `/test-approval` (debug): produce and resolve real test approval UI; remain absent from release.
      PFTQA-051 repairs and live-verifies cancellation through the local response destination; accept,
      session-accept, and release-build absence remain open.
- [ ] `/debug-m-drop` and `/debug-m-update`: if visible in debug, perform bounded real maintenance with
      confirmation and evidence; they must be absent from release and may not display a stub.

### 13.15 User-visible CLI and service acceptance inventory

- [ ] No-argument interactive launch and all documented global flags, including model, cwd, added roots,
      config overrides, search, OSS, sandbox, approvals, `--yolo`, alternate screen, and version/help.
- [ ] `exec`/`e`: real prompt, tool call, JSON/stream output modes, exit codes, stdin, files/images, interrupt.
- [ ] `review`: real repository review with supported targets and machine/human output.
- [ ] `login`, login status/methods, and `logout`: each supported auth mode, expiry, re-auth, isolation.
- [ ] `vault`: all documented internal/user-safe vault helpers with secret redaction and exit codes.
- [ ] `tasknode`: every documented nested command against a test account.
- [ ] `telegram`: foreground/background lifecycle, health, restart, signals, and exact session control.
- [ ] `claude-pane-smoke` and `claude-pane-workflow-suite`: real authenticated Claude Plan and API-key
      profiles, machine-readable report, correct nonzero failure exit.
- [ ] `mcp`, `mcp-server`, `plugin`, `app-server`, and `remote-control`: real client/server lifecycle,
      auth, protocol compatibility, reconnect, shutdown, and failure exit codes.
- [ ] `completion`: generate and source Bash/Zsh/Fish/PowerShell completions; commands/flags match help.
      PFTQA-052 generates all four plus Elvish on the rebuilt artifact, proves PFTerminal registration,
      and PFTQA-055 sources/exercises Bash successfully. Zsh, Fish, PowerShell, and Elvish are not
      installed on this host, so those source/execution cells remain open rather than inferred.
      and repairs nested plugin/marketplace identity; actual shell sourcing and interactive completion
      selection remain open.
- [ ] `update`: check, download, verify signature/checksum, install, relaunch, preserve state, rollback on failure.
- [ ] `doctor`: healthy and intentionally broken config/auth/provider/wallet/GPU/Telegram cases; actionable,
      accurate, redacted output and machine-readable mode.
- [ ] `sandbox`: run real read/write/network commands proving selected sandbox contract and exit propagation.
- [ ] `apply`/`a`: apply valid diff, reject malformed/conflicting diff, show result, never overwrite silently.
- [ ] `resume`, `fork`, `archive`, `unarchive`, and `delete`: named/ID/last/picker workflows and safe errors.
- [ ] `app` on supported platforms: launch/install/continue correct PF Desktop session.
- [ ] Hidden GPU controller, endpoint-token, Claude-token, daemon, proxy, and stdio-relay commands are
      exercised through the product workflows that consume them and have direct contract/security tests.
- [ ] Exit codes, stdout/stderr separation, non-TTY behavior, signals, terminal restoration, and PF branding
      are correct for every user-visible subcommand.

### 13.16 Persistence, migration, isolation, packaging, and operational safety

- [ ] Copy representative 0.1.24, 0.1.25, and 0.1.26 homes; launch exact candidate, migrate, use every
      stateful feature, restart, resume, and compare originals byte-for-byte unchanged.
- [ ] Validate sessions, panes, spawn hierarchy, goals, memories, providers, vault references, wallet,
      plan entitlements, GPU rentals, Telegram, Task Node, plugins, skills, hooks, themes, keymaps, title,
      statusline, and usage state after migration.
- [ ] Run stable PF, debug PF, and stock Codex concurrently; verify paths, processes, sockets, databases,
      logs, wallet daemon, credentials, updates, and cleanup are isolated.
- [ ] Force process kill, power-loss-like interruption, full disk, corrupt/truncated state, stale lock,
      unavailable keyring, unavailable network, and service crash; verify bounded recovery and backups.
- [ ] Install and upgrade actual Linux, macOS, and Windows packages; verify first launch, existing-home
      upgrade, checksums/signatures, PATH/shortcuts, uninstall preservation policy, rollback, and update.
- [ ] Run a 24-hour soak with real turns, pane switching, spawn activity, Telegram, wallet daemon, and GPU
      controller where authorized. Record CPU, memory, file descriptors, disk, database/WAL, logs, retries,
      spend, and orphan processes. No crash loop or unbounded log growth is allowed.
- [ ] Verify log rotation and service restart policy for every daemon/background process.
- [ ] Run security review for credential exposure, unauthorized remote control, command policy bypass,
      unsafe update/install, path traversal, symlink attacks, cross-home access, and dependency advisories.

### 13.17 Defect handling and release decision

Every failure found during hands-on QA must include:

- exact candidate and environment record;
- minimal reproduction plus adjacent/generalized cases;
- failed boundary classification: routing, state, intent, policy, persistence, timeout, provider,
  permissions, billing, rendering, packaging, or user workflow;
- severity, affected feature/matrix cells, logs/transcript, and cleanup state;
- generalized repair and automated regression test;
- rerun of the exact failed hands-on workflow and the affected neighboring matrix cells.

Literal-input patches, prompt stuffing, hidden allowlists, hard-coded model caps, and special-case
routes do not close a defect. The failed boundary must be repaired.

The release is **GO** only when:

- [ ] every release-visible feature and command above is PASS, not menu-observed, compile-only,
      mocked-only, skipped, assumed, or delegated to the operator;
- [ ] every required matrix cell is PASS or has a reviewed technical equivalence justification;
- [ ] every paid/destructive gate has completed under an approved bounded test account and budget;
- [ ] no open P0/P1 defect, silent fallback, secret leak, state corruption, orphaned paid resource,
      unbounded retry/log growth, or product-branding violation remains;
- [ ] the exact packaged hashes match the qualified artifacts and a second reviewer signs the evidence;
- [ ] the operator explicitly authorizes release after reviewing the consolidated report.

The original acceptance rule remains the standard for claiming complete matrix qualification.
For this release, the operator has explicitly accepted the documented residual risk and authorized
shipment. The accurate statement is therefore:

> **PF Terminal 0.1.27 is authorized for release with residual QA gaps documented in this spec and
> the hands-on ledger. Unchecked cells are not silently reclassified as PASS.**
