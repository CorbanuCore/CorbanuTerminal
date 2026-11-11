## A force-stopped turn still reports its file changes

- **The turn's file-change summary is sent when an interrupt force-stops a
  turn** (#1604, D282). When a turn misses the interrupt grace period and is
  stopped, it now sends the summary of the files it changed, as a turn that
  ends normally does, so the status line stays current.
- **A turn stopped while recording a search result still marks the
  conversation as using outside content** (#1536). The check now runs before
  the result is recorded instead of after it.
