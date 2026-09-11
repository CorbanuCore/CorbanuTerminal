---
sprint_id: PF-78-S02
title: Agent Task Node profile propagation
plan_feature: PF-78
execution_order: 81
owner: Codex emergency release owner
parallel_lane: emergency-profile-repair
write_scope: codex-rs/cli/src/tasknode_cmd.rs, codex-rs/core/src/exec_env.rs, codex-rs/core/src/exec_env_tests.rs, codex-rs/core/src/tools/handlers/shell/, codex-rs/core/src/tools/handlers/shell.rs, scripts/install/, codex-rs/core/src/tools/handlers/shell_tests.rs, codex-rs/core/src/unified_exec/, codex-rs/core/tests/suite/, codex-rs/skills/src/assets/samples/tasknode-usage/
integration_gate: Codex emergency release owner runs combined profile and provider regression tests
depends_on: none
created: 2026-09-07
updated: 2026-09-10
status: completed
plan_file: docs/plans/active/p0-security-levels.md
worktree: /home/pfrpc/repos/worktrees/corbanu-release-0.1.39
branch: fix/tasknode-agent-profile-scope
base_commit: 9b71d86d7fc57b25e3b020a813a6b75dd898836a
---

# PF-78-S02 — Agent Task Node profile propagation

Renumbered from P0 PF-45-S02; see the [identity reconciliation](../../identity-reconciliation-2026-09-10.md). Historical evidence and completion state are retained.

The user reported a concrete account mismatch: the active profile tab and an agent's unscoped helper used different accounts. Restore the authorized profile boundary for all agent-side Task Node reads and writes. Product: **Campaign Tracker — LOCAL PILOT CANDIDATE**, attributable prompts and permissioned history; Task Node profile-owned identity.

Scope: core shell/unified-exec environment construction, CLI helper scope resolution, Task Node usage skill and regression tests. Preserve explicit profile authority and default/named separation; do not transfer or delete historical records.

## Done

- [x] Reproduced the source boundary: TUI resolves active user config layer, CLI defaults when no profile argument is supplied; skill preserves only home.

- [x] Inject active profile into both shell execution paths and explicit snapshot overrides.
- [x] Make helper inherit that scope and reject missing/malformed/conflicting agent scope.
- [x] Cover named/default profiles, stale overrides, two-account behavior, shell and unified exec.
- [x] Update bundled/installed usage guidance and qualify actual PTY account agreement.
- [x] Run fix, formatting and affected tests; install the repair with durable evidence.

## Remaining

None for implementation and local acceptance. Public platform publication is tracked separately in the release record.

## Execution mandate

Finish agent profile propagation and fail closed before any Task Node account lookup when scope is absent or conflicting.

## Plan linkage

Plan: [P0 security levels](../../../plans/active/p0-security-levels.md), feature PF-78. Product heading **Campaign Tracker — LOCAL PILOT CANDIDATE**: “preserve attributable user prompts and compact GLM 5.3 Flash summaries in a bounded database”.

## Code boundaries

Core shell and unified execution environment construction; CLI Task Node scope resolver; bundled Task Node usage guidance; profile regression tests.

## Preconditions

- [x] User explicitly instructed emergency repair, tests, and release on 2026-09-07.
- [x] Plan active; existing patch preserved in its recorded worktree.
- [x] Historical global sprint conflicts are disclosed; the direct emergency instruction authorizes proceeding with this repair.

## Verification

- [x] Final formatted tree passes affected profile and provider error tests.
- [x] Actual PTY verifies named/default profile agreement and conflict rejection.

## Exit evidence

- [x] Commit, test logs, installation and release recorded in qa/release/0.1.40/.
- [x] Remaining evidence gaps disclosed accurately.


## Final evidence

Code commit `d42240f40c2a84be03561288aba4e1b9c3036b9f`; candidate 0.1.40; 228 focused Rust tests and six installer/source checks passed. True-TUI named/default account agreement, cold resume, and the terminal policy-stop fixture passed. The installed entrypoint independently passed live account selection and rejection checks. See [release evidence](../../../../qa/release/0.1.40/RELEASE.md) for exact artifacts, candidate digest, repository bases, and the ongoing public build.
