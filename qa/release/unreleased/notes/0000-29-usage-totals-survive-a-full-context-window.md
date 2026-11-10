## Usage totals survive a full context window

- **A full context window no longer resets the thread's usage totals**
  (#1583, #1577, thanks to @bradsbots). When a request or a compaction hit the
  context window, the thread's input, cached-input, cache-write, output and
  reasoning counters were reset to zero, so `corbanu exec --json` usage, the
  TUI's `/status` and the app server's `thread/tokenUsage/updated` totals then
  counted only usage after the fill. They now keep the earlier reports. The
  total token count is still set to the full window at a fill, and the context
  indicator is unchanged.
