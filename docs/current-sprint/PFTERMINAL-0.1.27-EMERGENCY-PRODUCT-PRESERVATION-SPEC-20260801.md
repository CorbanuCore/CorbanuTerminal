# PF Terminal 0.1.27 Emergency Product-Preservation Specification

Date: 2026-08-01  
Status: **P0 / NO-GO**  
Owner: PF Terminal release engineering  
Applies to: upstream Codex convergence and every PF Terminal 0.1.27 release candidate  
Released compatibility floor: `pfterminal-v0.1.26-pre-convergence` (`68807bc8e`)  
Convergence base: `45a60f03d`  

This specification overrides any release plan, migration note, or code-review judgment that
allows a released PF Terminal feature to disappear, become unreachable, silently change
meaning, or acquire Codex product branding.

## 1. Emergency directive

PF Terminal is the product. Codex is its upstream open-source foundation.

The upstream integration may improve PF Terminal's runtime. It may not turn PF Terminal into
a lightly renamed Codex build. It may not delete PF Terminal behavior because upstream lacks
an equivalent. It may not call a feature “replaced” merely because a nearby upstream mechanism
exists.

The following are absolute release blockers:

1. A command, workflow, provider, plan, wallet function, orchestration function, persistence
   path, installer, or other user-observable behavior present in released PF Terminal 0.1.26 is
   missing or materially degraded.
2. A PF Terminal screen, prompt, error, confirmation, help entry, installer, or generated
   artifact presents Codex as the product.
3. A merge conflict is resolved by deleting PF behavior without a feature-level compatibility
   test and explicit operator approval.
4. A release candidate is built from the current unreconciled dirty worktree.
5. “Equivalent,” “legacy,” “duplicate,” “obsolete,” or “unused” is used as justification for a
   deletion without proving the released user workflow still passes.

No deprecation is authorized by this specification. Any future deprecation requires a separate
written decision identifying the feature, migration path, user impact, and release version.

## 2. Incident evidence

At the start of this emergency specification:

- the active branch was `integrate/upstream-20260730` at `45a60f03d` with hundreds of modified,
  deleted, or untracked paths;
- the released 0.1.26 slash-command enum contained `/panes` and `/docs`;
- convergence removed both commands before they were restored in the dirty worktree;
- the restored commands have not yet passed the complete TUI and artifact acceptance gates;
- a current user-facing archive confirmation still says “exit Codex” in
  `codex-rs/tui/src/chatwidget/slash_dispatch.rs`;
- prior review language incorrectly treated released UI as replaceable implementation detail.

These are examples of a general boundary failure: upstream adoption was evaluated at the file
and implementation level instead of against a frozen PF Terminal product contract. The repair
must therefore create and enforce that product contract. Fixing only `/docs`, `/panes`, or one
branding string is insufficient.

## 3. Definitions

### 3.1 Preserved feature

A feature is preserved only when all applicable conditions remain true:

- the user can discover it through the same command, help, menu, or documented entry point;
- the entry point reaches a working implementation rather than a placeholder or dead end;
- its inputs, output, persisted state, permissions, billing behavior, and failure behavior remain
  compatible with the released version;
- existing user state can be resumed or migrated without destructive mutation;
- its acceptance test passes against the exact packaged release candidate.

Keeping an enum variant, alias, source file, or description alone does not preserve a feature.

### 3.2 Compatible implementation replacement

Internal code may be replaced by an upstream implementation only if the released PF workflow
continues to satisfy the full acceptance contract. For example, `/panes` may enter one native
agent graph instead of maintaining a second pane engine, but it must remain discoverable and
must show, select, inspect, and control the agents users could access through the released
workflow. A redirect that loses functionality fails this requirement.

### 3.3 Product branding

User-facing product identity includes terminal chrome, prompts, confirmations, errors, help,
slash-command descriptions, CLI help, installers, update messages, telemetry consent text,
generated snapshots, and public documentation.

Those surfaces must identify the product as **PF Terminal** or **PFTerminal** according to the
local style. “Codex” is allowed only in these bounded contexts:

- factual upstream attribution, such as “PF Terminal is built on OpenAI Codex”;
- an explicitly named third-party authentication product, protocol, package, or compatibility
  mode whose real external name contains Codex;
- internal Rust crate/module/type names and stable protocol or telemetry identifiers that users
  do not see and whose renaming would break upstream compatibility.

An internal Codex identifier must never be copied directly into user-visible text. When an
upstream error or label crosses the UI boundary, PF Terminal must map it to PF Terminal product
language while preserving technical details needed to diagnose the problem.

## 4. Required source strategy

The current dirty tree is evidence and salvage material, not a release branch.

- [ ] Allow every currently running non-destructive test to finish and save its full log.
- [ ] Run the repository secret scanner over tracked and untracked salvage content.
- [ ] Record `git status`, the full path manifest, diff statistics, source refs, toolchain,
      migration hashes, and generated-file state.
- [ ] Commit the complete safe salvage state to a local quarantine branch. Do not publish it and
      do not rewrite or delete the existing recovery ref.
- [ ] Create a separate clean worktree and release branch from `45a60f03d`.
- [ ] Port changes into the clean branch as small reviewed slices. Never merge or copy the dirty
      tree wholesale.
- [ ] For every slice, state which released PF feature or upstream parity requirement it serves.
- [ ] Compile and test each slice before starting the next dependent slice.
- [ ] Keep the clean release branch reviewable: product logic, tests, generated artifacts, and
      release evidence must be attributable to coherent commits.

Recommended slices:

1. upstream runtime convergence with no PF product deletion;
2. PF provider, catalogue, authentication, plan, vault, wallet, and GPU boundaries;
3. model-aware native orchestration and exact route identity;
4. PF TUI commands, documentation viewer, panes/agent compatibility, and branding;
5. state migration, debug-home isolation, installers, packaging, and release evidence.

## 5. Frozen PF Terminal feature manifest

### 5.1 Manifest generation

The release process must generate a machine-readable manifest from the 0.1.26 tag and compare it
with the candidate. The manifest must cover behavior, not merely file names.

- [ ] Add `scripts/release/build_pf_feature_manifest.py`.
- [ ] Generate `qa/release/0.1.27/pf-feature-manifest-0.1.26.json` from
      `pfterminal-v0.1.26-pre-convergence`.
- [ ] Generate the same manifest for the clean release candidate.
- [ ] Add a comparator that fails on a missing entry point, missing implementation binding,
      changed persistence namespace, removed configuration key, or lost platform artifact.
- [ ] Require every intentional structural difference to link to an acceptance test. An allowlist
      entry without a test is invalid.

At minimum, the manifest must inventory:

- CLI binaries, subcommands, flags, environment variables, and exit behavior;
- TUI slash commands, descriptions, availability rules, and dispatch targets;
- provider IDs, model routes, authentication methods, billing classes, and selectable plans;
- configuration fields, defaults, profile behavior, and schema exports;
- SQLite databases, migrations, rollout/session records, wallet/vault files, and home paths;
- app-server methods and generated protocol types used by PF clients;
- Linux, macOS, and Windows packages, installers, update assets, and checksums;
- background services and integrations including Telegram, wallet daemon, Task Node, and GPU
  rental;
- orchestration entry points, child identity, resume/cancel/wait behavior, and model selection.

### 5.2 Mandatory released user entry points

The following PF surfaces are explicitly protected. This list is a minimum; absence from this
human-written list does not override the generated 0.1.26 manifest.

- [ ] `pfterminal` and `pfterminal-debug` binaries.
- [ ] PF stable and debug homes remain isolated from each other and from stock Codex.
- [ ] `/model`, `/providers`, `/vault`, `/wallet`, and `/gpu`.
- [ ] `/agent`, `/spawn`, `/orchestrate`, and `/panes`.
- [ ] `/docs`, including bare viewer launch and targeted-page arguments.
- [ ] `/telegram`, `/tasknode`, `/goal`, `/skills`, `/permissions`, and `/usage`.
- [ ] Provider setup for OpenAI, Anthropic/Claude plans, PF plans, Kimi, GLM, Grok, OpenRouter,
      Vercel, Z.AI, and supported custom/local routes.
- [ ] Exact model/provider selection, transactional switching, and no silent paid fallback.
- [ ] Model-aware child spawning with durable model, provider, capability, vision, plan/billing,
      and selection-reason evidence.
- [ ] Wallet creation/import/backup/unlock/balance/plan-purchase behavior.
- [ ] Vault credential storage and lookup without secret disclosure.
- [ ] GPU selection, authorization, launch, persistence, inspection, and termination. The sole
      selectable DeepSeek recipe is `deepseek-ai/DeepSeek-V4-Flash-0731` at Hugging Face commit
      `7872f01b1d1fe23eabc4c98b48bffcef5a386062`; OpenRouter uses the exact model ID
      `deepseek/deepseek-v4-flash-0731`. Unversioned DeepSeek V4 Flash aliases are not release
      pins.
- [ ] Telegram control and resume using the selected permissions and exact route.
- [ ] Session resume from PF 0.1.26 state without database collision or foreign migration.
- [ ] Linux shell installer, macOS DMG, Windows package/installer, checksum verification, upgrade,
      and rollback.

## 6. Branding firewall

Brand preservation must be enforced in code and CI rather than by visual memory.

- [ ] Add a branding audit script that scans renderable strings, CLI help, installer text,
      snapshots, release metadata, and public PF documentation.
- [ ] Classify each `Codex` occurrence as internal identifier, upstream attribution, external
      proper noun, or violation.
- [ ] Store the bounded non-user-facing exceptions in a reviewed data file with path, context,
      owner, and rationale. Do not suppress an entire directory.
- [ ] Fail CI when a new unclassified occurrence appears.
- [ ] Fail CI when product chrome, confirmation text, help, errors, or release assets call PF
      Terminal “Codex.”
- [ ] Replace the current “archive ... and exit Codex” confirmation with PF Terminal language.
- [ ] Review every changed TUI snapshot and CLI golden output for product identity.
- [ ] Assert that the startup banner, `/status`, `/help`, `/providers`, `/model`, archive/quit
      confirmations, crash guidance, doctor output, and updater all identify PF Terminal.

The audit must distinguish attribution from identity. README text explaining the Codex foundation
is valid. A prompt telling a PF Terminal user that they are exiting Codex is a P0 violation.

## 7. Upstream convergence rules

Every conflict must be evaluated using three sources:

1. released PF Terminal 0.1.26 for the product contract;
2. pinned upstream Codex for runtime behavior and bug fixes;
3. the candidate implementation for the combined result.

Conflict resolution rules:

- [ ] Prefer upstream turn execution, context projection, compaction, tools, permissions,
      streaming, retries, TUI mechanics, app-server evolution, and native agent lifecycle.
- [ ] Preserve PF provider adapters, catalogue, plans, credentials, wallet, GPU, Telegram,
      Task Node, orchestration entry points, packaging, home isolation, and branding.
- [ ] Adapt PF behavior at typed boundaries; do not fork upstream runtime logic when a provider,
      model, billing, capability, or presentation adapter can express the difference.
- [ ] Remove PF-only runtime policy only after proving its removal does not remove a released
      feature. Arbitrary caps and regex heuristics are runtime policy, not protected features.
- [ ] Never infer product behavior from model-name substrings or one-off regexes. Use the typed
      catalogue and structured route/capability state.
- [ ] Record each retained upstream divergence in `FORK_POLICY.md` with owner, reason, tests, and
      removal condition.
- [ ] Record the exact upstream commit. Updating that pin requires rerunning the entire manifest,
      branding, migration, and compatibility gates.

## 8. Required automated tests

### 8.1 Product-contract tests

- [ ] A golden test compares the candidate slash-command inventory with the released inventory.
- [ ] Dispatch tests prove every protected slash command reaches its intended behavior.
- [ ] `/docs` tests cover viewer launch, targeted page, success, missing page, and viewer close.
- [ ] `/panes` tests prove the compatibility entry opens the native agent view and retains the
      released inspect/control workflow.
- [ ] CLI golden tests cover PF names, help, aliases, debug binary, and all protected subcommands.
- [ ] Branding tests cover static text and runtime strings assembled from multiple fragments.
- [ ] Configuration/schema tests load a representative 0.1.26 config without losing fields.
- [ ] Migration tests open a copied 0.1.26 PF home, resume sessions, preserve credentials/wallet
      references, and leave the original fixture unchanged.
- [ ] Collision tests prove PF stable, PF debug, and stock Codex homes cannot mutate one another.

### 8.2 Provider and model tests

- [ ] Route identity is structured as provider, exact provider model ID, wire protocol, auth and
      billing mode, effort, service tier, and accepted reported aliases.
- [ ] Explicit route requests either run exactly or fail before inference. No substitution test may
      observe spend on a different route.
- [ ] Model switches commit only after provider acknowledgement; failed switches retain the old
      displayed, persisted, compaction, and sampling route.
- [ ] Billing is known for every spawn-eligible paid route through verified catalogue pricing or a
      typed provider billing feed. Unpriced routes are ineligible for automatic paid spawning.
- [ ] Capability selection uses typed vision, tools, image generation, context, output, reasoning,
      and plan eligibility rather than names.
- [ ] GPT-5.5 is never selected automatically; Sol, Terra, and Luna retain their distinct intended
      tiers.
- [ ] Opus, Fable, Kimi K3, GLM 5.2, Grok, and DeepSeek paths have provider-contract tests.

### 8.3 Orchestration tests

- [ ] A manager can query the catalogue and explain each selected child's exact model, provider,
      capability fit, billing source, and plan preference before dispatch.
- [ ] Explicit Kimi, GLM, Opus/Fable, Sol/Terra/Luna, and Grok selections are honored or refused
      accurately.
- [ ] Child identity survives nested spawn, resume, compaction, cancellation, completion, and TUI
      reopen.
- [ ] `/spawn`, `/orchestrate`, `/agent`, and `/panes` use one native controller, registry, mailbox,
      and accounting path while preserving each released workflow.
- [ ] A child never inherits an incompatible full-history route silently. The UI explains a refusal
      using the actual structured constraint.
- [ ] Agent output cannot leak orchestration control blocks into an ordinary user transcript.

### 8.4 Long-session and upstream parity tests

- [ ] Anthropic long histories compact before provider request-size failure.
- [ ] Signed and redacted thinking blocks replay byte-for-byte as required by the provider.
- [ ] Image-heavy histories use provider-aware projection without destroying the canonical stored
      conversation.
- [ ] Output limits come from typed provider/model capability, not a PF-wide arbitrary cap.
- [ ] Interrupted streams, overloads, 413 responses, compaction, retry, and model switching cannot
      enter infinite or duplicate-work loops.
- [ ] OpenAI cache behavior matches pinned upstream Codex within the declared performance gate on
      equal task counts.
- [ ] Generic code review, repository repair, and the isometric-game visual endurance scenario run
      through the exact debug/RC binary without false route warnings, loss of control, or context
      implosion.

### 8.5 Required crate and workspace gates

- [ ] Run changed-crate tests through `just test -p <crate>`.
- [ ] Run complete `codex-tui` tests and review every pending snapshot.
- [ ] Run complete `codex-core`, app-server, protocol, API, CLI, provider/catalogue, state, wallet,
      vault, GPU, Telegram, installer, and packaging suites.
- [ ] Classify any environment-only sandbox failure with an isolated reproducer and require the
      corresponding supported-host CI gate to pass. Do not simply relabel a failure as a flake.
- [ ] Regenerate configuration and app-server schemas and prove a clean second generation.
- [ ] With operator approval, run the complete workspace `just test` gate.
- [ ] Run scoped `just fix` and final `just fmt` after tests, following repository rules.
- [ ] `git diff --check` passes and no `.snap.new`, `.pending-snap`, temporary log, secret, or build
      output is part of the candidate.

## 9. Exact artifact acceptance

Source tests alone cannot approve the release.

- [ ] Build `pfterminal` and `pfterminal-debug` from one clean candidate commit.
- [ ] Record source commit, dependency locks, compiler/toolchain, binary hashes, and build command.
- [ ] Prove `pfterminal-debug --yolo` launches that exact local debug binary with the debug home.
- [ ] Run scripted TUI smoke tests covering startup identity, help, all protected PF command groups,
      model switching, agent inspection, docs, panes, provider setup, wallet, vault, and clean exit.
- [ ] Install each packaged platform artifact into a clean environment and repeat the smoke test.
- [ ] Upgrade a copy of a released 0.1.26 home and resume representative long, orchestrated, wallet,
      provider, and Telegram sessions.
- [ ] Roll back using documented steps and prove user data remains recoverable.
- [ ] Verify package signatures/checksums and reject modified artifacts.
- [ ] Run a zero-hit secret scan over source, logs, evidence, and packages.

No test run against an earlier build qualifies a later rebuild. Any source or dependency change
invalidates the artifact qualification and requires a new candidate number.

## 10. Release decision record

The release remains **NO-GO** until every P0 item is checked with linked evidence.

The final evidence packet must include:

- [ ] 0.1.26 and candidate feature manifests plus the comparator report;
- [ ] branding audit with zero violations;
- [ ] clean source history and final diff against both 0.1.26 and pinned upstream;
- [ ] test commands, full logs, failure classifications, snapshots, and generated-file checks;
- [ ] migration, resume, home-isolation, installer, upgrade, and rollback evidence;
- [ ] exact artifact hashes and smoke transcripts;
- [ ] exact-route and orchestration evidence for the supported model/provider classes;
- [ ] performance parity evidence with equal run counts;
- [ ] an operator-signed list of any deferred P1 item. No P0 may be deferred.

Only after that packet is complete may release engineering ask the operator to approve a private
or public release candidate. Stable publication, paid provider campaigns, GPU rental, pushing,
tagging, and GitHub release creation remain separate operator-approved actions.

## 11. Immediate next actions

- [ ] Preserve the dirty tree without resetting or discarding it.
- [ ] Finish and archive the active test log.
- [ ] Create the safe local quarantine snapshot.
- [ ] Create the clean reconstruction worktree at `45a60f03d`.
- [ ] Generate the released feature manifest before porting more code.
- [ ] Port and test PF features in the five reviewable slices in Section 4.
- [ ] Remove all user-facing Codex identity, beginning with the known archive confirmation.
- [ ] Produce a debug-ready clean candidate and evidence packet.
- [ ] Stop before publication and request explicit release approval.
