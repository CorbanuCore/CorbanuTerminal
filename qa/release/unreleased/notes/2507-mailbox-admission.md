## Agents and reviews

- Slow durable-mailbox writes for an unloaded agent no longer block unrelated agents from loading or unloading. Reload still waits for earlier mail admission and preserves its delivery notification, including when the sender is cancelled (#2507).
