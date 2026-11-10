## Protocol change: `turn.completed` usage can be `null`

- **`corbanu exec --json` reports `"usage": null` when exec receives no
  matching usage report** (#290). Unknown usage is no longer reported as an
  all-zero usage object, so consumers can distinguish "not reported" from a
  measured zero. When usage is reported, the object is unchanged.
- **A non-null usage does not prove the provider reported this turn.** On a
  resumed thread, if this turn's response reports no usage, `usage` still
  holds the thread's earlier totals. When only some of a turn's requests report
  usage, the totals include only those.
- **Usage includes observed descendant work** (#290, #1601, thanks to
  @white-guy-01). Repeated cumulative reports are counted once. Every
  history-forked descendant makes the total `null`, as do a descendant with no
  report, a failed or timed-out ancestry read, or dropped events (`Lagged`).
  The selected thread's own totals are not a difference from the start of the
  invocation, so its first report needs no inherited baseline: a resumed
  thread whose first report follows a compaction estimate keeps a known total
  (#1637). A history-forked subagent that is not saved to disk yet is now also
  recognized as a fork (the app server's `thread/read` returns its
  `forkedFromId`), so it too makes the total `null` (#1637).
  A resumed (non-fork) descendant whose first report carries in earlier usage
  (its total differs from its last request) makes the total `null` too
  (#1637, #1750, thanks to @citadelculture).
  Detached children can incur further usage after this snapshot.
- **Default multi-agent spawns copy history.** In MultiAgentV2, `spawn_agent`
  defaults to `fork_turns: "all"` when omitted, so these default multi-agent runs
  report `usage: null`. A non-null total can be larger than in earlier versions
  because it includes descendant work rather than only the root's own total.
- **Context estimates are not provider reports.** Compaction, rollback and
  full-window estimates keep the last measured report, or `null` if none exists.

### Compatibility

- **Consumers that read token counts from `turn.completed.usage` without a
  null check** will fail on turns with no reported usage and must be updated.
  The TypeScript SDK in `sdk/typescript` handles `null` from this release:
  `TurnCompletedEvent.usage` is `Usage | null`, and `Thread.run()` returns
  `usage: null` when exec receives no usage report.
