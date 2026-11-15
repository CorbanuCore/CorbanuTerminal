## Agent tools act only on the caller's own agents

- V1 `close_agent` and `send_input` check the saved ownership of registered,
  unloaded agents before closing or loading them. A loaded thread's actual
  parent is checked again before input or interruption.
- V2 `list_agents` no longer mistakes another session's `/root` entry for the
  caller. V2 `wait_agent` includes the caller's unloaded agents in its summary,
  as `list_agents` does.
- Agent shutdown leaves hidden internal threads with their lifecycle owner.
