## Agents and reviews

- **A review started from a conversation runs with its current security
  settings** (#2015). The app server no longer offers detached review
  delivery: clients now fork the conversation with `thread/fork` and start
  the review on the fork with `turn/start`. A fork of a running conversation
  starts with the sandbox, approval policy, approval reviewer, Windows sandbox
  and security level the conversation is running with, including changes made
  since it started: a Windows sandbox set up later, an approval policy or
  sandbox changed during the conversation, or a stricter `/security` level
  confirmed during the session. It no longer falls back to the settings the
  app server loaded when it started. A fork request can still choose its own
  sandbox and approval policy, but it can't set a security level or Windows
  sandbox below the conversation's. On Windows, a fork request that asks for
  a stricter Windows sandbox (elevated) than the app server started with now
  keeps it instead of getting the start-up one (D721).
