---
sprint_id: "PF-27-S02"
title: "Secretless agent launch and bypass containment"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 29
owner: "broker lane worker (codex, 2026-10-06)"
parallel_lane: "broker"
write_scope: "codex-rs/core/src/security/launch_contract.rs, codex-rs/core/src/security/launch_contract_tests.rs, codex-rs/core/src/security/mod.rs, codex-rs/core/src/tools/sandboxing.rs, codex-rs/core/src/tools/sandboxing_tests.rs, codex-rs/core/src/tools/runtimes/apply_patch.rs, codex-rs/core/src/session/turn_context.rs, codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs, codex-rs/core/src/unified_exec/errors.rs, codex-rs/core/src/unified_exec/process_manager.rs, codex-rs/protocol/src/secretless_launch.rs, codex-rs/protocol/src/secretless_launch_tests.rs, codex-rs/protocol/src/lib.rs, codex-rs/protocol/src/shell_environment.rs, codex-rs/hooks/src/registry.rs, codex-rs/hooks/src/engine/command_runner.rs, codex-rs/rmcp-client/src/utils.rs, codex-rs/network-proxy/Cargo.toml, codex-rs/network-proxy/src/config.rs, codex-rs/network-proxy/src/runtime.rs, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/network-proxy/src/credential_broker/isolated/, codex-rs/network-proxy/src/credential_broker/isolated.rs, codex-rs/network-proxy/src/lib.rs, codex-rs/network-proxy/src/credential_broker/isolated_tests.rs, codex-rs/process-hardening/, codex-rs/Cargo.lock, qa/security-levels/sprints/PF-27-S02/, qa/demos/index/PF-27-S02.md, docs/sprints/current/p0-security-levels/pf-27-s02-secretless-agent-launch.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5): focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos; merged behind secretless_agent_launch. Shared files serialized by the integration owner, not reserved here: the one-line flag registration in codex-rs/features/src/lib.rs and its codex-rs/core/config.schema.json entry (also edited by PF-24-S03 and PF-30-S01), and three small hunks in codex-rs/core/src/config/mod.rs (arm the contract, login shell/snapshot off, proxy fields), a file PF-60-S03 reserves; and five new demo specs `qa/demos/specs/pf27s02-*.toml` under the directory PF-30-S02 reserves (new files only)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf27-s02-secretless-20261006"
branch: "feat/pf27-s02-secretless-20261006"
base_commit: "13cf4a2d0c07046312dc6d32c757dce90e0b14fc"
depends_on: "PF-27-S04"
created: 2026-08-28
updated: 2026-10-06
---

# PF-27-S02 — Secretless agent launch and bypass containment

**October 6:** macOS and Linux ship behind `secretless_agent_launch` (default off; Permissive unchanged).
Windows refuses protected launches with a reason; the port is [PF-27-S06](pf-27-s06-windows-broker-and-launch.md).
[Evidence, launch-boundary inventory and known limits](../../../../qa/security-levels/sprints/PF-27-S02/README.md).

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

- Contract: `core/src/security/launch_contract.rs` (sandbox, path, argv/stdin checks); allowlist in `protocol/src/secretless_launch.rs`, applied by `shell_environment::create_env`, hooks and MCP stdio.
- Launch points: `core/src/tools/sandboxing.rs` (`env_for`, `env_for_exec_server`), `tools/runtimes/shell/unix_escalation.rs`, unified-exec stdin.
- Broker: `network-proxy/src/credential_broker/isolated/` (control socket, runtime dir), `process-hardening/src/broker_containment.rs`.
- Tests: `pf_27_s02` modules in codex-protocol, codex-core, codex-network-proxy and codex-process-hardening (the planned `secret-broker/tests/containment.rs` lives in process-hardening and network-proxy instead).

## Preconditions

- [x] Active plan; PF-27-S04 completed and archived (2026-10-06).
- [x] Read root and nearest AGENTS.md; worktree recorded in the plan front matter.
- [x] Source pins and module paths confirmed; Windows has no broker or contract backend (moved to PF-27-S06).

## Done

- [x] Allowlisted launch environment once armed: provider keys, tokens, passwords, credential helpers (`SSH_AUTH_SOCK`, askpass, `GIT_CONFIG*`), start-up hooks (`BASH_ENV`, `ENV`, `ZDOTDIR`, `LD_*`, `DYLD_*`) and URL passwords are stripped; zsh gets `ZDOTDIR=/var/empty`; login shells and shell snapshots are off and top-level login shells are refused. Applies to exec, local unified exec, zsh-fork escalations, the user shell, hooks (no `-l`) and MCP stdio pass-through variables. Remote environments are refused.
- [x] Design conflict resolved: stripping runs first; the proxy then sources brokered tokens from Core's own environment, only those the user's environment policy would have passed, and adds dummies (GLM run: agent sees only a dummy, server `authorized: true`).
- [x] Raw managed values (Core's secret-looking variables at arm time) refused in argv and unified-exec stdin and removed from local launch environments.
- [x] OS denial per launch, not cached: commands must run under Seatbelt/bubblewrap; reads denied for `CODEX_HOME` credential and history stores (`secrets`, `auth.json`, `.env`, `provider_auth.json`, `config.toml`, `wallet`, `run`, snapshots, logs, sessions, `*.sqlite*`), common CLI credential files under `$HOME`, Claude's sign-in and macOS Keychains; nothing under `CODEX_HOME` is writable. In-process file tools (apply_patch, structured edits, image view) get the same profile or no file access. Unsandboxed, full-access and escalated runs are refused with a reason; Windows is refused.
- [x] Core hardening when armed: Linux non-dumpable (hides `/proc/<pid>/environ` and `mem`), macOS `PT_DENY_ATTACH`; a failed call refuses launches.
- [x] Broker hand-over items: (1) containment — Seatbelt on macOS, seccomp (+ Landlock where the kernel has it) on Linux; no exec, fork, ptrace or cross-process reads, writes only under its runtime dir; a broker that cannot confine itself is refused when armed; (2) macOS setup-pipe window — secrets never cross a Core-created pipe: stdout carries only the control-socket path, the broker accepts its parent pid only and Core checks the socket peer is its child; (3) sockets in `CODEX_HOME/run` (owner-only, agent-unreadable), else the per-user cache/runtime dir outside the sandbox's writable roots; (4) the strip/dummy conflict above.
- [x] Probes: macOS GLM 5.2 runs flag off/on (reads allowed vs denied, Core environment unreadable, refusal text); Linux `pf_27` tests in Docker (seccomp layer). Named `pf_27_s02` tests drive `env_for`/`env_for_exec_server` and the contract; Cargo lock updated.
- [x] Linux end-to-end GLM 5.2 TUI run on a Landlock host (Ubuntu, kernel 7.0, merge commit `699bd4a82f`, October 6): flag on, the agent sees only a dummy GitHub token and the server still authorizes; every protected-path probe is denied; Core's `/proc/<pid>/environ` is root-owned (unreadable even by the same user, readable with the flag off); the broker reports `landlock+seccomp` in an owner-only `CODEX_HOME/run` directory; full access is refused with the reason; the `pf_27` containment tests pass on the Landlock path. [Evidence](../../../../qa/security-levels/sprints/PF-27-S02/README.md#linux-end-to-end-landlock-host).
- [x] Behaviour note for `isolated_credential_broker` alone (PF-27-S04): the control handshake (protocol 2) and broker self-containment apply whenever the isolated broker runs; socket placement and value sourcing change only with `secretless_agent_launch`.

## Remaining

- [ ] Decision (Travis): Claude panes and external provider harnesses (`tui/src/claude_panes/`) are not under the contract. Options: block them in protected levels, or bring them under the contract. Recommendation: hand to the TUI lane, which owns the panes, to choose; until then the PF-41 inspector lists them as "not covered".
- [ ] Decision (Travis): MCP servers, hooks, the `!` user shell and app-server `command/exec` get the environment allowlist but run outside the OS sandbox. Options: refuse them in protected levels, or accept them as user-configured/user-initiated and show "not contained" in PF-41. Recommendation: accept and show "not contained" (refusing breaks every configured MCP server and hook; the user started them).
- [ ] Windows broker and contract: [PF-27-S06](pf-27-s06-windows-broker-and-launch.md).

## Verification

- [x] Linux and Bazel CI on the PR: all checks green at merge (PR #191).
- [x] `just fix -p` on every touched crate and `just fmt`; final diff inspected.
- [x] Focused: `just test -p codex-core pf_27_s02` (12) and the `pf_27` tests in protocol, network-proxy and process-hardening, all passing.
- [x] Integration: affected crate suites (722 passed; core subset 853 passed; one unrelated rmcp keyring-fixture failure recorded).
- [x] TUI applicability: GLM 5.2 TUI runs and five SOP videos ([index](../../../../qa/demos/index/PF-27-S02.md)).
- [x] Candidate, commands and outcomes recorded; synthetic credentials only.

## Exit evidence

- [x] Implementation commits and outputs under `qa/security-levels/sprints/PF-27-S02/`.
- [x] One Opus 5.5 High review (changes required, then approve with nits after fixes) dispositioned.
- [ ] Done/Remaining reflect reality; record archived when Remaining is empty or moved.
