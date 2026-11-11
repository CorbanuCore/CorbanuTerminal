## Moderate: waiting on and starting agents after untrusted content

- **An interrupted agent message no longer comes back when the agent is
  reloaded** (D485, #1952), at every security level. If an interrupt stopped
  a turn before it had recorded the message it was working on (for example a
  just-started agent's first message), the message stayed marked as handed
  over, and reloading the agent later (after it was unloaded, or on quit and
  resume) handed it over again and started a turn on it, even from a
  `send_message` that never starts one. Such a message is now cancelled when
  the turn stops. Messages the turn did record were already never replayed;
  recovery after a crash is unchanged.
