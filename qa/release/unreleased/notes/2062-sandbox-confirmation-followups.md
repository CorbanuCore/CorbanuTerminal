## Windows sandbox

- Sandbox setup now waits for confirmation before reporting “Sandbox ready”
  for the running conversation. A failed or unconfirmed switch gives `/new`
  recovery guidance, and an error is no longer hidden by an unconfirmed retry.
  The message clarifies that already-running work keeps its earlier sandbox.
  A disabled sandbox is described as confirmed settings, without claiming readiness,
  and rejected updates show the server's reason.
- Selecting a weaker sandbox explains that it applies from the next session,
  without attributing the restriction to an unspecified configuration source.
- A conversation forked from a running one (`thread/fork`, which replaces
  detached reviews) refreshes the missing-sandbox notice for the security
  level it inherits.
