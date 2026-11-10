## Windows sandbox

- **Non-admin sandbox git watch reads a global git config on a network
  share** (D517, #1877). When your home folder, and so your `.gitconfig`,
  is on a network share or mapped network drive, the hooks folders and
  scripts it names are now watched too. The sandbox waits for that config
  at most about 2 seconds per check; if the share doesn't answer in time,
  or the read fails, the configs count as changed after every command
  instead of holding the command up. Include paths such a config names on
  a share are never opened. Configs on this machine are read as before.
