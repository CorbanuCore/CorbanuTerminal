## Agent tools act only on the caller's own agents

- **A sub-agent's `list_agents` shows only itself and its own agents**
  (D320, #1756, #1767, thanks to @citadelculture). With MultiAgentV2,
  `list_agents` used to list every live agent, including the caller's parent
  and siblings and other root threads' agents; it now lists the caller and
  the agents it started (and theirs), loaded or not. This is the intended
  tightening of D320. `close_agent` and `send_input` (and their `_v1`
  aliases) likewise refuse an agent the caller didn't start, directly or
  through its own agents.
- **`wait_agent` waits only for the caller's own agents** (#1744, #1701,
  thanks to @citadelculture). `wait_agent` (and `wait_agent_v1`) could wait
  for any loaded agent, including another root thread's, and return its final
  status. It now refuses an agent the caller didn't start, directly or
  through its own agents, with "wait_agent can only wait for agents spawned
  by this agent or by its sub-agents". With MultiAgentV2, `wait_agent` used
  to report every live agent with its last result; it now reports only the
  caller's agents.
