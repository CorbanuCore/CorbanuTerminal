## Windows sandbox

- Sandbox setup now waits for confirmation before reporting “Sandbox ready”
  for the running conversation. A failed or unconfirmed switch gives `/new`
  recovery guidance, and an error is no longer hidden by an unconfirmed retry.
  The message clarifies that already-running work keeps its earlier sandbox.
- Selecting a weaker sandbox explains that it applies from the next session,
  without attributing the restriction to an unspecified configuration source.
- Detached reviews refresh the missing-sandbox notice for the security level
  inherited from their conversation.
