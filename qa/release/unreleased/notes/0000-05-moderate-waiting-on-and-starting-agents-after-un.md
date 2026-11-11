## Moderate: waiting on and starting agents after untrusted content

- **Waiting on an agent never asks** (D320, #1653). On Z.AI and other
  providers without namespace tools, `wait_agent_v1` used to ask (or, with
  approvals off as in `corbanu exec`, was refused) as "a tool the security
  level has not classified" once untrusted content had entered the session,
  so a parent couldn't wait for its child. It now goes ahead, as `wait_agent`
  already did.
- **Starting an agent asks, on every route.** After untrusted content under
  Moderate (and Aggressive), `spawn_agent` (and `spawn_agent_v1`,
  `spawn_agent_plaintext`) asks for your approval, and the prompt says it is
  "a request to start a new agent". With approvals off it is refused. This is
  new on the OpenAI Responses route, where `spawn_agent` used to go ahead.
- **The `_v1` agent aliases act like their usual names** (D334, #1702). On
  Z.AI, Chat Completions and Claude-style routes, `send_input_v1` and
  `close_agent_v1` now go ahead after untrusted content, as `send_input` and
  `close_agent` do; before, they asked (or were refused with approvals off) as
  unclassified tools.
- **Restarting a finished agent asks, like starting one** (D334). After
  untrusted content, `resume_agent` (and `resume_agent_v1`) asks for your
  approval on every route, and so does a MultiAgentV2 `followup_task` (and
  `followup_task_plaintext`) to an agent that isn't running, since it starts a
  new turn there; the prompt says it is "a request to restart a finished
  agent", and with approvals off it is refused. A follow-up to an agent with a
  turn running goes ahead. Before, `resume_agent` and `followup_task` went
  ahead on every route that offers them, and `resume_agent_v1` asked as an
  unclassified tool.
- **`send_input` to an agent that isn't running asks too** (D388, #1832).
  After untrusted content, V1 `send_input` (and `send_input_v1`) to a
  finished or idle agent starts a new turn there, so it now asks like
  `followup_task`; with approvals off it is refused. To a running agent it
  still goes ahead.
- **Interrupting an agent that isn't running no longer wakes it** (D404,
  #1796), at every security level. `interrupt_agent` on a finished or idle
  agent only reports its status; before, the interrupt notice started a new
  turn there, with the caller's reason as its message.
- **Interrupting a just-started agent stops its first turn again** (D435).
  For an agent `spawn_agent` just started, whose first turn hasn't begun,
  `interrupt_agent` sends the interrupt, which stops that first turn, but not
  the notice that would start another. Since D404 the interrupt was not sent
  at all, so the first turn ran, and the reply wrongly said the agent "was not
  started". A first turn held back by the agent limit, the kill switch or a
  signed-out account is not cancelled and runs later; the reply says so. The
  interrupt can also cancel that agent's MCP server startup, so a held-back
  first turn may later run without its MCP tools. An idle agent reloaded
  without a turn still only gets its status reported.
- **Known limit** (D389): a follow-up or `send_input` to an agent that is
  running when checked goes ahead; if that turn ends just before the message
  arrives, the message starts a new turn without asking. Likewise, at any
  level, if an agent's turn ends just before an interrupt reaches it, the
  interrupt notice can start a new turn.
- **The same rules apply under Aggressive** (D335).
