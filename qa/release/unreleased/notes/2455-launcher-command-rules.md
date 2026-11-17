## Security

Command rules now also check commands run through `unshare` (for example `unshare -r chroot / …`), `nsenter`, `setpriv`, `timeout`, `nice`, `nohup`, `stdbuf`, `setsid`, `ionice`, `chrt`, `taskset`, `flock`, `doas`, and `runuser -u`, alone or chained with `sudo`, `chroot`, and `env`. A forbidden rule for the inner command now refuses it before any approval is offered, including the Moderate user-namespace approval, and a prompt rule's reason is shown in that approval. Unrecognized launcher options, and forms that run a shell string (`flock … -c`, `runuser` without `-u`), require approval (#2455, D670).

As with `sudo`, a saved allow rule for one of these launchers no longer approves the command it runs, and these commands no longer offer an automatic “always allow” rule.
