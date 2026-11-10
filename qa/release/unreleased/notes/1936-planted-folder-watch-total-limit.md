## Windows sandbox

- **Default (elevated) sandbox: the planted-folder watch has a total limit
  across nested `.codex` and `.agents` folders** (#1909). It already stopped
  looking through one such folder past 20,000 entries or 64 MiB of files. It
  now also stops past 100,000 entries or 256 MiB across all of them at a
  command's start or end. As with the per-folder limits, the folders it
  can't finish are flagged after every command; the command itself is never
  blocked.
