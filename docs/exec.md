# Non-interactive mode

For information about non-interactive mode, see [this documentation](https://developers.openai.com/codex/noninteractive).

## JSON token-usage limits

Do not use `turn.completed.usage` from `corbanu exec --json` as a complete
billing or run-cost record:

- It copies the latest matching usage report's thread totals for the selected
  thread and turn. It does not add usage from spawned subagents or calculate a
  difference from the start of this invocation.
- If exec receives no matching usage report, the counters default to zero.
  There is no marker distinguishing unavailable usage from measured zero, so
  zero counters do not prove that the run was free.

These are limits of the existing JSON output, tracked in
[#290](https://github.com/CorbanuCore/CorbanuTerminal/issues/290).

Product basis: [Shipping MVP — LIVE](corbanu-product-spec.md#shipping-mvp--live),
“the `corbanu` command.” This section documents its current reporting limits.

## When the Corbanu home cannot be read

If the Corbanu home becomes unreadable ("Operation not permitted") during a
run, for example while a drive is briefly blocked, `corbanu exec` waits and
retries with backoff (1 s, doubling to at most 60 s) for up to an hour,
printing each attempt, and continues the turn once access returns. If session
items could not be written meanwhile (they are kept in memory), exec waits for
their folder before it saves the session at exit, within what is left of the
same window since the first failure (an interrupted run does not wait, and
Ctrl-C stops the wait). Items of a thread that has already ended, such as a
finished subagent, are not waited for: nothing writes them after it ends. If
access does not return in time, the turn fails and exec exits 1. Change the
window with `--home-wait-secs` (`0` fails at once).

A `corbanu` command started while the home is unreadable stops at once with
exit status 75 (1 when, on macOS, the process's own sandbox denies the read,
which waiting cannot fix). Private helpers that Corbanu runs inside a sandbox
(file access, apply-patch, the arg0 exec helper) skip this check and record
nothing, since that sandbox may deny the home on purpose.

Each failure is recorded, without file contents or credentials, in
`~/Library/Logs/Corbanu/home-access.log` on macOS
(`~/.local/state/corbanu/home-access.log` elsewhere): the operation, path and
error, the permissions, flags, ACL and extended-attribute names of the path
and its parents, the mount, the sandbox check, the responsible process and the
working directory.
