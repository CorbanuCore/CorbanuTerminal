# The owner is armed, and the first interval-attributed ticks did work

`com.corbanu.initiative-owner` is armed at generation 1 in `user/501`, and the
schedule is no longer refusing. This is the state PF-80-S01 has been working
toward since recurrence was approved: the loop between prompts is the job's own
interval rather than me polling by hand.

## What is true, from the state rather than from intent

`tick.json` after the first two admitted intervals:

- `hold: null` - the latched `owner_run_refused` is cleared
- `ticks: 3`, `last_success` and `previous_success` both set, 30.5 s apart
- `firing: "interval"` - attributed to the schedule, not to a kickstart
- `consecutive_errors: 0`, `last_error: null`

`owner.sqlite3` agrees and is more specific: **two boots**, one per admitted
tick, each pinning the same package digest; one fixture operation with its
observation and its delivery to the coordinator; `health` shows `watchdog / ok /
0 consecutive failures`. `meta` reads `armed`, generation 1, decision
`owner-recurrence-domain-20260917` revision 3.

The dashboard now publishes `recurring-at-observation` with no reason, replacing
the honest `stalled / owner_run_refused` it has shown since the install.

## Order of operations, as the independent review required

The reviewer approved arming at **fixture-only** scope while I dispatch workers
by hand, and gave the order. I followed it: read `--activation-status` while the
coordinator was quiescent; confirmed `state off`, `next_generation 1`, scope
`fixture-only`, and both stored digests equal to the live configuration and
package; ran `--arm --dry-run` and accepted its preview of the activation file,
the meta rows and the coordinator effects; armed; confirmed the generation
matched the predicted `next_generation`; left the HOLD latched and watched one
interval bump only `skipped` and `last_probe`, exactly as predicted; then
recovered with inspection evidence naming why the hold existed.

The status read showed one in-flight action, the long-stalled
`slack-receiver-02`, already `stall_reported` with `watchdog_will_report false`,
so no armed tick would relabel anything of mine. That visibility is the reason
this step was safe to take rather than a gamble, and it exists because an
independent review refused the round that did not have it.

## What arming required first, and what it cost

The live runtime had to be restaged from the received integration commit,
because the arming entry point did not exist in the runtime installed earlier
today, and the installation receipt pins the runtime digests. So: receipted
uninstall, restage, recompute the configuration's package digest, reinstall,
verify firing, then arm. Two records were preserved rather than overwritten -
the previous installation receipt is kept under `schedule-archive-<timestamp>`,
and the previous (empty, never-armed) owner state is kept beside the new one.

## What is still NOT true

Scope is `fixture-only`: the configuration carries no transport section, so the
owner launches no workers. `tmux-workers` scope remains unarmed, because an
armed transport owner can collide with my hand dispatch - manual keys against
daemon deliveries, hand-owned claims becoming `unowned_claim` holds - and that
handoff has to be coordinated, not assumed.

**Disarm revokes admission; it does not restore watchdog history or return
consumed generations.** `--disarm --generation 1` is the reversal, and I keep it
ready.
