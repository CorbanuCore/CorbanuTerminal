## Agents and reviews

- **A detached review runs with its conversation's current security
  settings** (#2015). It used to start from the settings the app server
  loaded when it started, so it could miss changes made since: a Windows
  sandbox set up later, a sandbox or approval policy chosen for the
  conversation or changed during it, or a stricter `/security` level confirmed
  during the session. It now uses the sandbox, approval policy, Windows
  sandbox and security level the conversation is running with, as an inline
  review already did.
