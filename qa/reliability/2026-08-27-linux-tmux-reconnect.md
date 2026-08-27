# Remote Linux/tmux reconnect investigation

- Reported by: Travis Good, 2026-08-27.
- Status: open; not reproduced or root-caused in this record.
- Classification: routine diagnostic/documentation record; a future fix must be
  classified from its actual behavior and boundaries before implementation.
- Product linkage: **Shipping MVP — LIVE**, “supervision, resume, and recovery”;
  **Product principles**, “Maintain continuous Codex parity without removing
  Corbanu-specific behavior.”
- Report: after returning some time later, Corbanu Terminal and Codex regularly
  show a “lost connection / reconnecting” message. The programs run in tmux on
  remote Linux machines, not in a local macOS terminal.
- Inspected fork: `12bf62444bcab7c5eea6d25b23aa301993fcb0ab`. The affected remote
  Corbanu/Codex binaries, commits, providers, and times are not yet recorded.

## Evidence and limits

The inspected Core retry handler, `codex-rs/core/src/responses_retry.rs`, emits
`Reconnecting...` on retryable stream errors. The client reuses WebSockets and
supports transport fallback. This is a candidate message path, not proof that it
generated the reported message. Capture the exact text and surrounding events.

Local macOS CLI/desktop versions and historical local logs do not identify this
remote incident. No evidence currently attributes it to browser isolation,
security policy, tmux, SSH, an upstream regression, or a specific provider.
Client sleep can interrupt SSH attachment; it does not by itself establish that
the remote process or its independent outbound provider connection stopped.

## Diagnostic matrix

Use a synthetic non-financial session. Obtain remote access/target information
before collecting evidence; do not infer a host or alter production settings.

| Comparison | Record / assertion |
| --- | --- |
| Idle but continuously attached | Remote process PID/start time, last successful response, idle interval, exact warning time and subsequent recovery |
| tmux detach and reattach, same SSH connection | Whether the process persists and whether warning predates reattachment |
| SSH disconnect/reconnect, same tmux process | Separate terminal attachment events from outbound provider disconnects |
| New process versus resumed conversation | Distinguish cached transport reuse from history reconstruction failure |
| Corbanu versus upstream Codex | Exact remote executable paths, versions/commits, provider endpoint identity, relevant redacted configuration, and comparable inputs |
| Transport, if supported by each build | Controlled WebSocket versus HTTPS runs; no global production configuration change |
| Failure before response versus after a tool result | Whether recovery duplicates work or loses events/state; never use a real financial action for this probe |

Before each run record Linux distribution/kernel, tmux version, time zone and
clock, relevant proxy/VPN configuration without secrets, process start time,
and the test session identifier. Correlate provider request IDs, application
logs, and SSH/tmux events in one timestamp window. Keep raw logs private and link
only redacted extracts; never dump tokens, the full environment, or conversations.

## Recovery acceptance for a future fix

- Separate recoverable connection churn from terminal failure; preserve an
  actionable error if bounded retry cannot recover.
- Do not hide repeated failures or increase timeouts without measured evidence.
- Preserve native cancellation, conversation position, and child lifecycle.
- In Moderate/Aggressive, preserve taint, authority epochs, revocations, and
  expired-grant denial; no temporary fallback to Permissive.
- Verify no duplicate tool execution or blind replay of financial side effects
  using fake executors and recorded request/action identities.
- Compare the fork and a pinned upstream candidate, then repeat the affected
  actual-key remote Linux/tmux workflow on the final build.

## Remaining

- [ ] Record exact remote environment, timestamp, message, and safe reproducer.
- [ ] Correlate SSH/tmux/process/provider events and identify the failing layer.
- [ ] Determine upstream-common versus Corbanu-specific behavior with evidence.
- [ ] Classify and authorize any fix; record its upstream-touch footprint.
- [ ] Qualify the fix without weakening security or replay protection.
