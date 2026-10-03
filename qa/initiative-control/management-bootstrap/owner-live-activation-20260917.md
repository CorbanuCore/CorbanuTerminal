# Live owner recurrence: installed, firing, and deliberately not armed, 2026-09-17

The reviewed guard (received at `93f60eee8`) cleared the way for the live step,
and the independent reviewer said yes to performing it on this host. I performed
it. This records what was installed, the one real defect the attempt found, and
exactly what is and is not true now.

## What is live

`com.corbanu.initiative-owner` is bootstrapped into **`user/501`** - the
Background domain, not GUI/Aqua - at a 30 second `StartInterval`, from a
receipted installation whose plist, pins and publish state are recorded in
`installation.json`. Before the install, both `gui/501` and `user/501` answered
`launchctl print` with exit 0, and the sibling GUI domain reported the label
`absent`, so the guard's sibling check passed on a real observation rather than
on an assumption.

Firing is observed, not inferred: `runs` went 1 -> 4 over 90 seconds, one
kickstart plus three interval firings about 30 seconds apart, with
`tick.json.last_probe` advancing and `skipped` incrementing.

## The defect the attempt found: the live root cannot live on the external volume

The first install put the schedule root, runtime and logs on
`/Volumes/CorbanuDrive`. launchd fired it on schedule six times and **every run
exited 78 (`EX_CONFIG`) with both log files empty** - the program never started.
The same `owner_daemon.py` invocation, with the same arguments and the same
environment, runs correctly when I execute it by hand, so the program is not at
fault.

An identical installation staged under `~/Library/Application Support` - same
python pin, same runtime bytes, same config, only the paths moved off the
external volume - spawned on the first firing and completed a tick. That is the
discriminator: launchd will not spawn this agent with its working directory and
standard output paths on the external volume, and the failure is silent at the
process level.

The live installation therefore runs from a local private root. The coordinator
state it reads remains where it is, because that path is opened by the daemon
after it has started, and it works.

## What is deliberately NOT true yet

The kernel is **not armed**. `manager_enabled` is false, no `activation.json`
exists, and `requested_mode` is still the `off` state written by setup. Every
firing therefore records `HOLD owner_run_refused` and skips, which is exactly
what an unarmed owner should do, and the dashboard now says so: recurrence
`stalled`, reason `owner_run_refused`, with a live age.

Arming is the next step and it is not a formality. It requires a transport
section in the configuration, an `activation.json` whose authority, decision id
and revision are bound to Travis's recorded answer, and meta fields that match
its digest. That work will be reviewed like any other change; nothing about
tonight's install grants it.

## Reversal

`activate.py --owner uninstall --root <root>` performs the receipted removal and
was exercised twice tonight, on the external-volume installation and on the
local probe, before the final install. Both removals verified absence in both
domains afterwards.

## Addendum, 2026-10-03: counts, arming receipt and current state

Added by the Task Node integration worker for task
`task_cf2cc91ff71a095a3baa93df1c3d91f8`. The text above is unchanged.

- Qualification: increment D functional qualification reached **13 of 13 cases
  and 39 of 39 variants** against candidate `6f3c2d705`, recorded in commit
  `cab24b2cbc` (PF-80-S01 ledger). The fixes are `c782a340f` and `6f3c2d705`;
  the launchd guard is `93f60eee8`.
- Arming receipt: [owner-armed-20260917.md](owner-armed-20260917.md). Armed at
  generation 1, scope `fixture-only`, decision `owner-recurrence-domain-20260917`
  revision 3; `hold: null`, `firing: interval`, `health` `watchdog / ok`.
- Read-only observation, 2026-10-03T10:38Z: `launchctl print
  user/501/com.corbanu.initiative-owner` shows `domain = user/501`, `run interval
  = 30 seconds`, `runs = 35024`, `last exit code = 0`. The live `tick.json` reads
  `hold: null`, `firing: interval`, `ticks: 44228`, `consecutive_errors: 0`,
  last success 29 s before the read. The live config has `manager_enabled: false`.
- Recurrence stays **fixture-only**. `tmux-workers` scope and the manager are not
  armed by this work. Later manager-enable work is recorded separately in
  [owner-manager-enabled-20261002.md](owner-manager-enabled-20261002.md).
