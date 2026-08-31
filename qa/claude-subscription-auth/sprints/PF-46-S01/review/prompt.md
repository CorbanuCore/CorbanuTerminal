# Independent final review: reliable Claude subscription authentication

Review the complete branch diff from frozen base
`8ae13e168817445205321bae410740cbc3e919b7` through the current candidate in
`/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated`.
Use read-only inspection only. Do not edit files, invoke another reviewer, or
expose any credential value.

The user-requested product behavior is:

- `/providers` offers an explicit two-method choice for Claude Plan. The
  official `claude setup-token` long-lived subscription token is first,
  selected by default, and recommended with concise eligibility/lifetime copy.
- The compatibility choice uses Claude Code's ordinary rotating login state.
- The exact selected source is persisted. Runtime resolution must never
  silently fall back to a different source, identity, account, or billing path.
  Native Claude Plan panes must bind that same selection at execution without
  persisting credential material in their plans, settings, artifacts, or audits.
- Managed tokens use the established encrypted vault with masked entry and
  metadata-only inspection. Raw values must not enter config, chat, rollout,
  logs, snapshots, errors, debug output, test artifacts, or generic vault
  reveal/programmatic paths.
- macOS Keychain is authoritative for current Claude Code login state; Linux
  and Windows use the credentials file. Legacy/name drift, missing/blank
  refresh tokens, refresh races, conflicts, cancel, failure, retry, recovery,
  and restart/resume must be deterministic and preserve unrelated credentials.

Inspect implementation, tests, snapshots, docs, and QA harnesses. Pay special
attention to transaction rollback, lock/error paths, zeroization and process
output, refresh/401 concurrency, platform source IDs and service names,
subprocess cancellation/timeouts, TUI state transitions, persisted-selection
compatibility, and whether the tests actually prove the claimed no-disclosure
and no-fallback behavior.

The diff intentionally includes coordination-only planning/sprint metadata
commit `f36c28770`, selectively mirroring the completed/archive state of
PF-13-S06 and PF-41-S03 so global lifecycle checkers pass. It does not import
their product code, manifests, workflows, MkDocs, or QA evidence. Do not treat
that metadata as Claude-auth implementation, but report any actual consistency
problem it creates.

For every actionable finding, report:

1. severity `P0`, `P1`, `P2`, or `P3`;
2. exact file and tight line range;
3. a concrete failing scenario;
4. why existing tests do not catch it; and
5. the smallest safe fix.

Do not report style preferences, speculative hardening, or issues outside this
branch diff. End with exactly one standalone verdict line:

`CLEAN`

or

`CHANGES REQUIRED`
