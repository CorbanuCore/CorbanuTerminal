---
sprint_id: "PF-27-S02"
title: "Secretless agent launch and bypass containment"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 29
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-27-S04"
created: 2026-08-28
updated: 2026-08-28
---

# PF-27-S02 — Secretless agent launch and bypass containment

## Execution mandate

- Deliver: No raw managed secret enters agent environment, command line, mounts, process memory access, or tool output in protected modes.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Non-negotiable controls** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: No raw managed secret enters agent environment, command line, mounts, process memory access, or tool output in protected modes.
- Sources and archive disposition: [PF-27 reconciliation](../../../plans/security-source-reconciliation.md#pf-27).

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/core/src/exec_env.rs; codex-rs/config/src/shell_environment_policy.rs; codex-rs/sandboxing/src/{manager,spawn}.rs.
- Launch adapters: `codex-rs/codex-mcp/src/connection_manager/startup.rs`; `codex-rs/tui/src/claude_panes/{execution,provider}.rs`; `codex-rs/model-provider/src/auth.rs`.
- Planned: codex-rs/core/src/security/launch_contract.rs; codex-rs/secret-broker/tests/containment.rs.
- Tests: planned colocated Rust test modules prefixed `pf_27_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-27-S04 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Run all three OS containment probes against shell, plugins/MCP and descendants, including process memory/debug APIs and inherited descriptors/handles. Protect policy-store writes as well as broker secrets; stale capability results cannot enable launch.

- [ ] Inventory SDK/plugin/native-harness auth handoffs including hidden request-transport metadata; verify raw opt-out flags, ignored proxy variables and child environments cannot bypass protected mode. Separate engine availability from verified process/OS containment.

- [ ] Add a Moderate/Aggressive allowlisted launch environment; strip inherited provider keys, proxy passwords, credential helpers, startup scripts, profile sourcing, and raw-secret argv/stdin. Permissive uses its frozen path unchanged.
- [ ] Apply the contract to exec/unified-exec, children, MCP/plugins, hooks, external provider/Claude panes, containers, and browser workers; inventory each launch boundary and deny unsupported routes.
- [ ] Enforce OS-level denial of vault/auth paths, broker memory/process handles, unrestricted IPC, host sockets, and secret-bearing mounts; network env hints alone are not containment.
- [ ] Test shell initialization, nested processes, proc/debugger access, container escape routes, environment overrides, and direct auth-helper attempts with synthetic canaries.
- [ ] Publish actual backend capability probes for Linux/macOS/Windows; unsupported isolation blocks protected-mode activation with a reason, without silent downgrade.
- [ ] Add named `pf_27_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_27_s02 && just test -p codex-core pf_27_s02 && just test -p codex-sandboxing pf_27_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-27-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
