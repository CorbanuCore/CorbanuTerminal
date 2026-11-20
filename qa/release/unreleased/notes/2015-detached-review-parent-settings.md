## Agents and reviews

- **Reviews now start on a fork of the conversation, which keeps its current
  security settings** (#2015, D711). The app server no longer offers detached
  review delivery: clients fork the conversation with `thread/fork` and start
  the review on the fork with `turn/start`. A fork of a running conversation
  starts with the sandbox, approval policy, Windows sandbox and security level
  the conversation is running with, as detached reviews did in 0.1.49, and now
  also with its approval reviewer. That includes changes made since it started. A fork request can
  still choose its own sandbox and approval policy, but it can't set a
  security level or Windows sandbox below the conversation's. On Windows, a
  fork request that asks for a stricter Windows sandbox (elevated) than the
  app server started with keeps it instead of getting the start-up one (D721).
