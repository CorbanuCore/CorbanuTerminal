---
sprint_id: "PF-23-S01"
title: "Moderate ingress and disclosure enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 40
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-13-S05, PF-22-S02, PF-30-S03"
created: 2026-08-24
updated: 2026-10-06
---

# PF-23-S01 — Moderate ingress and disclosure enforcement

## Execution mandate

- Deliver: Moderate deterministically blocks untrusted requests for secrets, protected financial data, policy changes, or protected actions.
- Excludes: Aggressive defaults, financial signing implementation, classifier training, TUI, and browser isolation.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: normal analysis continues while hostile instructions cannot gain authority or protected data.

## Code boundaries

- Existing: `codex-rs/core/src/tools/{router,registry}.rs`; `core/src/mcp_tool_call.rs`; `core/src/exec.rs`
- Planned: `codex-rs/core/src/security/{protected_surface,protected_surface_tests}.rs`
- Tests: affected tool, MCP, exec, context, vault, and policy suites

## Preconditions

- [ ] PF-13-S05, PF-22-S02, PF-30-S03 are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-23.

## Remaining

- [ ] Implement the plan's action/profile usability matrix with conservative data and control-flow ancestry; a model selecting clean-looking human values after hostile content is not trusted reconstruction. No runtime Moderate activation until all required subsystems qualify.

- [ ] Integrate PF-30 durable provenance and post-taint checks; detector output never supplies authority, and a new turn or summary never clears taint.
- [ ] Moved from PF-30-S02 (2026-10-06): positive protected memory extraction. Stage one still denies under
  Moderate/Aggressive (PF-30-S04); allowing it needs labelled, lineage-bound rollout input and the
  [stage-one handoff](../../../../qa/security-levels/sprints/PF-30-S01-typed-source-envelope/memory-stage-one-follow-up.md) matrix.
- [ ] Moved from PF-30-S03 (2026-10-06), routes that never reach the shared approval seam: MCP tool calls,
  `write_stdin` into running processes, and code mode. Give each a typed protected resource and the same
  post-taint check: fresh human approval bound to taint and policy, refused under `never`.
- [ ] Moved from PF-30-S03: what a command-text classifier cannot see, at run time or by the OS: strings
  built at run time (`chr()`, environment lookups, names read from files), build tools that run arbitrary
  code (`make`, `npm run`, build scripts), hard links, and reads of a home through an unclassified route.
  Typed resources plus sandbox-level denial of home and credential reads replace the lexical net. Known
  lexical gaps from the PF-30-S03 review: `cd` inside a substitution or subshell moving the classifier's folder,
  code passed through positional parameters, functions or `set --`, command-string wrappers (`su -c`,
  `runuser -c`, `script -c`, `ssh host cmd`), name references and `${!x}`, `cd -P`/`||` approximations, loop
  stdin from a process substitution, `awk system()`/`sed e`, and symlink hops into automounts during lookups.
- [ ] Moved from PF-30-S03: outbound disclosure requests after taint, and quoted malicious trades (value
  transfer proposed from tainted content).
- [ ] Register required protected-mode subsystems and deny unsupported/unready routes; final activation requires the full plan readiness matrix, not this dispatch slice alone.

- [ ] Classify protected surfaces by typed resource/action at the shared Core dispatch boundary.
- [ ] Treat project text, tool/MCP output, hooks, plugins, connectors, and external content as non-authoritative inputs.
- [ ] Deny vault enumeration/extraction, protected-financial-data disclosure, policy mutation, approval bypass, and value transfer without matching authority.
- [ ] Preserve non-protected analysis and existing Permissive behavior.
- [ ] Add paraphrase and adjacent-case regressions; literal prompt matching is not the primary router.
- [ ] Emit stable secret-free decisions and audit metadata.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused tests: `cd codex-rs && just test -p codex-core protected_surface`.
- [ ] Boundary regressions: `cd codex-rs && just test -p codex-core tools:: && just test -p codex-core mcp_tool_call`.
- [ ] TUI applicability: none; PF-26-S02 owns interactive proof.

## Exit evidence

- [ ] Commit, typed surface matrix, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-23-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
