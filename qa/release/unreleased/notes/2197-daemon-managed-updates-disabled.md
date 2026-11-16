## Fixes

- The app-server daemon no longer downloads or runs the upstream OpenAI Codex
  installer. Managed self-install/update is disabled pending Corbanu installer
  support. Bootstrap keeps starting the daemon, reports automatic updates as
  disabled, and directs users to the [Corbanu installation instructions](https://github.com/CorbanuCore/CorbanuTerminal#install).
  Install/update manually and restart the daemon. Mutating lifecycle commands
  stop an updater left by an earlier build, including when the managed binary
  is missing; an old updater remains active until one of those commands runs
  or the machine reboots.
