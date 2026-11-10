## Telegram `/status`: token estimates are not a thread total

- **`/status` no longer shows unknown usage as a free thread.** After
  compaction or rollback on a thread whose provider never reported usage, it
  said `Thread total: 0 tokens (0 in, 0 out)`. It now says
  `Thread total: not reported by the provider`, and `Context` shows the token
  estimate (#1570, #1586, thanks to @sendoeth).
- **A full context window is shown as context, not as the thread's cost.**
  When the window is exceeded, `Context` shows the full window and
  `Thread total` stays the provider-reported usage, computed from the
  provider's counters, which the fill now keeps (#1583; #1586, thanks to
  @sendoeth). When the first update the bridge sees for a thread is a fill
  (for example after a bridge restart), `Context` shows the full window if
  the thread had already reached it (#1638), and otherwise what the fill
  added.
  After compaction, `Context` shows core's token estimate, and `/new` starts
  the new thread without the previous thread's total.
