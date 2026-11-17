## Security

Command rules now also check commands run through the shell builtins `command` and `exec`, `time` (`/usr/bin/time`), `caffeinate`, `systemd-run`, `pkexec`, `bwrap`, `firejail`, and `busybox` or `toybox` applets. A forbidden rule for the inner command refuses it, and a prompt rule asks with its own reason. `command -v`/`-V` only print, so they run nothing. Unrecognized options, `systemd-run --shell` or an `Exec…=` property, and `bwrap --args` require approval, as do `firejail` commands (older firejail runs them through a shell) and `systemd-run` commands with an abbreviated option or a `$` (#2455, D670).

As with `sudo` and `unshare`, a saved allow rule for one of these launchers alone (for example `time`) no longer approves the command it runs, and such commands no longer offer an automatic “always allow” rule.
