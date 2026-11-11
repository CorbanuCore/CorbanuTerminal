## Moderate: waiting on and starting agents after untrusted content

- **An agent message that arrives while a turn is stopping is no longer
  lost** (#2010). A turn that was being stopped (interrupt, kill switch,
  guardian, a replacing turn, or quit) could still take a queued agent
  message in its last moments without noting it, so the message was neither
  delivered nor cancelled. It now stays queued for the next turn, or for
  recovery after quit and resume.
