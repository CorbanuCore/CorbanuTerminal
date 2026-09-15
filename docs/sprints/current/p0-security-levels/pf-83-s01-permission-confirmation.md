---
sprint_id: "PF-83-S01"
title: "Request-correlated permission confirmation"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-83"
execution_order: 84
owner: "Astra High permission-confirmation worker"
parallel_lane: "permission-confirmation"
write_scope: "codex-rs/app-server/src/request_processors/turn_processor.rs, codex-rs/app-server/src/thread_state.rs, codex-rs/app-server/src/bespoke_event_handling.rs, codex-rs/app-server/src/connection_cleanup.rs, codex-rs/app-server/src/lib.rs, codex-rs/app-server/src/settings_confirmation.rs, codex-rs/app-server/src/settings_confirmation_tests.rs, codex-rs/app-server/tests/suite/v2/thread_settings_update.rs, codex-rs/app-server/README.md, codex-rs/app-server-protocol/src/protocol/v2/thread.rs, codex-rs/app-server-protocol/schema/, codex-rs/tui/src/app.rs, codex-rs/tui/src/app_server_session.rs, codex-rs/tui/src/app/config_persistence.rs, codex-rs/tui/src/app/thread_settings.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app/thread_routing.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/app/permission_confirmation.rs, codex-rs/tui/src/app/permission_confirmation_tests.rs, codex-rs/tui/src/chatwidget/permission_popups.rs, codex-rs/tui/src/chatwidget/settings.rs, codex-rs/tui/src/chatwidget/input_restore.rs, codex-rs/tui/src/chatwidget/tests/permissions.rs, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_after_mode_switch.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default@windows.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default.snap, codex-rs/tui/src/app/tests.rs, codex-rs/tui/src/app/test_support.rs, codex-rs/tui/src/app/snapshots/, qa/reliability/live-permission-transition-20260913/, docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md"
integration_gate: "Repair coordinator receives isolated candidate, audits exact scope and reruns app-server/protocol/TUI and relevant unchanged Core regressions; fresh Fable review and enforced independent functional execution/evidence required. Canonical integration remains existing Fable-owner coordinated; no merge/push/install or live session change in this allocation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913"
branch: "fix/live-permission-transition-20260913"
base_commit: "005cc644f59b1e762e5497b329e106c67925d4ed"
depends_on: "none"
created: 2026-09-14
updated: 2026-09-14
---

# PF-83-S01 — Request-correlated permission confirmation

## Execution mandate

- Deliver one app-server/TUI confirmation path with honest next-turn command permission scope.
- Exclude authorization-policy changes, live-turn hot swap, global quiescence, automatic approval/replay, Core lifecycle changes, dependency changes and app installation.
- Product authority: Travis's explicit expanded-scope approval; security owner's explicit PF27 reservation handoff.

## Plan linkage

- Plan: [P0 security and permission initiative](../../../plans/active/p0-security-levels.md#current-permission-confirmation-allocation--september-14).
- Feature: PF-83. Product heading **Permission selection confirmation — TO BUILD**: "A submitted selection is not a confirmed change."
- No unfinished security dependency: existing Core settings events and native lifecycle are inputs, not newly enabled protected controls.

## Code boundaries

- Exact allowed files/prefixes are front matter; pending-outcome leaf modules are planned.
- App-server processor/listener/connection state may correlate existing Core submission IDs; no Core source writes.
- v2 settings request/response types and their generated schema fixtures may support narrow opt-in confirmation; no unrelated wire changes.
- TUI permission selection, native client/response/error routing, state and tests only. Snapshots/generated changes must be listed exactly in handoff.
- Manager-reviewed cycle2 allowance: <=750 non-test and <=1750 total hand-authored source/test changed lines; generated fixtures separately mechanical. Extra100 test lines cover Fable review02 unsupported-server routing regression only; cycle1 fixes unchanged. No new paths or product scope. Stop at measured overage for staging.

## Preconditions

- [x] Active plan amended for the explicit user-approved confirmation contract.
- [x] Existing security owner returned paused PF27 reservation; no broker resumption.
- [x] Exact worktree/base recorded; no unfinished sprint dependency.
- [x] Selected scope disjoint from accounting and Task Node recorded allocations.
- [x] Both governance checkers passed before source dispatch: three active plans,116 current/126 archived sprints; PF27 note compacted to its100-line limit without dropping evidence.

## Done

- [x] Recorded source ACK insufficiency and preserved previous rejected broad proposal.
- [x] Frozen independent original F01-F11 retained; no execution/acceptance claim.
- [x] Registered request-specific confirmation before Core submission; exact Core success/no-op/error completion, independent of snapshot deduplication.
- [x] Added end-to-end deadline and listener/connection/shutdown cleanup; single deferred reply owner, ordered bounded enqueue, no uncertain-effect retry.
- [x] Preserved literal legacy `{}` acceptance; new TUI requires explicit Applied and treats acceptance-only/unsupported responses as unconfirmed.
- [x] Implemented native asynchronous requested/outcome UI, stale-result rejection, confirmed-only reviewer persistence and Core-owned next-turn settings.
- [x] Added pending-turn saved-draft deferral without automatic queue replay; no Core authorization/current-turn/MCP source changes.
- [x] Documented API and regenerated affected schema fixtures; retained all failed attempts and formatter-restoration diagnosis in repair QA.

- [x] **Guest network isolation executed and verified, 2026-09-15.** Tailscale was found connected on the guest, a second default route that a filter on `en0` alone would not have closed; it is logged out and disconnected. A default-deny packet filter now leaves only inbound SSH from the host, its reply path, DHCP with the host and host ICMP echo. The mediated inference pinhole is deliberately absent: it was opened on port 8111 and verification found Docker already listening there on the host, so it pointed the guest at an unrelated published port. The guest is sealed until a real mediator exists on a verified-unused port. All deny checks fail as required and host SSH works, reproduced after a reboot, with the filter restored at boot by a LaunchDaemon. `pflog0` does not exist on this build, so denial is evidenced by rule counters instead, which are stronger than timeouts: the block rule matched and dropped 839 packets after the reboot. [Receipt](../../../../qa/initiative-control/management-bootstrap/pf83-isolation-executed-20260915.md).
## Remaining
- [ ] **The 175 denial controls cannot be proven by network isolation, and the preflight now shows why.** Increment-07 completed in 9.76 seconds against the sealed guest: 2 valid, 173 UNPROVEN, `not_reached`. For both bridge roles `internet_http` went from HTTP 200 to a resolution failure and `lan_tcp` from connected to a timeout, so the isolation is working and observable. The validator requires `return_value == -1` with `errno` in `EPERM`, `EACCES`, `EROFS` — operating-system permission denials. A packet filter produces `ETIMEDOUT`, `ENETUNREACH` or `EAI_NONAME`, never `EPERM`. `macos_preflight.py` states the same limitation itself: `partial account/VM separation; no per-run filesystem/process/IPC/network sandbox`, and `boundary-policy.txt` names the prerequisite as a disposable VM **plus OS enforcement**. We have the VM and the account; the OS-enforcement half does not exist. Next unit: a per-run seatbelt profile for the bridge and executor side so denied operations return a qualifying errno. The target must not simply be wrapped in a second sandbox, because its own permission behaviour is what PF-83 exists to qualify. [Finding](../../../../qa/initiative-control/management-bootstrap/pf83-preflight-finding-20260915.md).
- [ ] Inference in the guest is enabled and proven by owner decision: the subscription credential was copied to the guest at Travis's instruction rather than mediated, and `pf` now permits outbound 443 to six pinned model-endpoint addresses with `/etc/hosts` mapping those names. A live smoke test through the pinned package returned the expected token. GitHub, raw.githubusercontent.com and codeload.github.com remain denied, so the anti-cheat property is intact. The harness must still stage the credential into each run profile; it does not today.

- [ ] Verify both next-turn directions, unchanged MCP refresh and pending approval semantics with focused native fixtures.
- [ ] Freeze implementation; obtain fresh Fable code review and correct substantive in-scope findings.
- [ ] Provision independent code-blind executor with enforced filesystem/process/IPC/network boundaries and negative controls; execute original cases with explicit approved-scope amendment.

## Verification

- [x] Pinned Rust1.95, isolated cache, locked/offline dependencies; formatting before final tests.
- [x] Focused app-server settings/lifecycle/public JSON-RPC cases via `just test`: final8 passed.
- [x] App-server-protocol unit/schema parity via `just test`: final8 passed.
- [x] Focused TUI permission/client/snapshot/pending-control cases via `just test`: final8 passed.
- [x] Focused unchanged Core next-turn/MCP tests via `just test`: final8 passed, no workspace-wide test.
- [ ] Real-key exact-package PTY success/cancel/failure/recovery/continuation in both disposable live repositories.
- [ ] Independent evidence check and schema-2 isolation receipts; original F01-F11 mapped, no unaccepted waiver.

## Exit evidence

- Final8 support run: 350/350 passed; exact command, frozen patch/manifest hashes, 517 non-test/1250 total changed Rust lines, retained failures and limits: `qa/reliability/live-permission-transition-20260913/implementation-checkpoint.md`.
- [ ] Exact candidate/tree, changed files, commands/results and independent reviews retained in repair QA.
- [ ] Receiving scope audit/combined-tree evidence; canonical integration writer coordinated separately.
- [ ] Human acceptance and release/benchmark status honestly recorded; no installed/runtime-fixed claim beforehand.
- [ ] Done/Remaining reflect results; archive only after all mandatory acceptance evidence is complete.
