## Agents and reviews

- **Reviews now start on a fork of the conversation, which keeps its current
  security settings** (#2015, D711). The app server no longer offers detached
  review delivery: clients fork the conversation with `thread/fork` and start
  the review on the fork with `turn/start`. As detached reviews did in 0.1.49,
  a fork of a running conversation starts with the sandbox, approval policy,
  approval reviewer, Windows sandbox and security level the conversation is
  running with, including changes made since it started. A fork request can
  still choose its own sandbox and approval policy, but it can't set a
  security level or Windows sandbox below the conversation's. On Windows, a
  fork request that asks for a stricter Windows sandbox (elevated) than the
  app server started with keeps it instead of getting the start-up one (D721).
