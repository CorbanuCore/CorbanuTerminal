# Nested-launch check in the standalone binaries

Round 4 (#219) added the nested-launch check to `corbanu`. The standalone
`codex-exec` and `codex-tui` binaries skipped it, so an agent command under
Aggressive that could run one of them got an agent outside that check.

## Fix

- The stored level and nested-launch detection moved from `codex-tui` into a
  new crate, `codex-security-level`. The code and the decisions are
  unchanged; `codex-tui` re-exports them and keeps everything that needs its
  config (the Aggressive overrides and verification).
- At start-up, after argument parsing, each standalone binary runs the same
  check:

  | Binary | Kind | Nested under `refuse` | Nested under `pass` |
  | --- | --- | --- | --- |
  | `codex-exec` | agent | refused | refused: only `corbanu exec` and `corbanu review` can hold a run to Aggressive |
  | `codex-tui` | interactive | refused | refused |
  | `codex-app-server`, `codex-mcp-server` | host | refused | refused |

  `codex-app-server` and `codex-mcp-server` had the same gap, so they are
  included. A person's own launch is unaffected.
- The `codex-linux-sandbox` alias of `codex-exec` runs before the check, so
  sandboxed commands are unaffected.
- `corbanu stdio-to-uds` now counts as a host (review finding): it reaches a
  running app server or daemon socket.

## Limits

- Other standalone helpers start no agent and are not checked. Older or
  upstream `codex` builds on PATH cannot be covered.
- A standalone binary started by a person does not apply a stored Aggressive
  level; only `corbanu` does. Its agent commands are then not sandboxed by
  Aggressive, so a launch from them is not detected as nested.
- The detection limits in [nested-launch](../nested-launch/README.md) still
  apply.
