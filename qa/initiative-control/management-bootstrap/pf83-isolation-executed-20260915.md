# PF-83 guest isolation — executed and verified

**Executed by:** Fable, 2026-09-15, using the administrator credential Travis
placed in the vault under `neo-vm`. **Guest:** macOS 26.2 arm64 at
`192.168.64.3`. **Host:** `192.168.64.1`.

This closes the network blocker that made 173 of 175 preflight controls
UNPROVEN. It does not qualify anything else: no case has run, and all 380
verdicts remain `not_reached` until the preflight is re-run and a separate
manager decision authorises case execution.

## What was done

1. **Safety net first.** A dead-man script reverted `pfctl` after 900 seconds
   unless `/var/run/pf83.confirmed` existed. `/etc/pf.conf` was backed up to
   `/etc/pf.conf.pf83.bak`. The marker was only created after host SSH was
   re-verified through the loaded ruleset.
2. **Key-based admin access installed.** The manager's existing public key was
   added to `neo2`'s `authorized_keys`, so the vault password can be rotated
   without breaking access. The password was never written to any file, never
   printed, and was fetched from the vault at each point of use.
3. **Tailscale removed as a network path.** `tailscale down` then
   `tailscale logout`, which discards the node key, then `scutil --nc stop` on
   the `io.tailscale.ipn.macsys` service. `networksetup -removenetworkservice`
   refused with *"there aren't any other network services on IPv4"* and was not
   forced; the service is present but **Disconnected** and logged out, so it has
   no credential to reconnect with, and the packet filter blocks its underlay
   regardless.
4. **Default-deny packet filter.** `/etc/pf.anchors/corbanu.pf83`, hooked into
   `/etc/pf.conf` by appending an `anchor`/`load anchor` pair without touching
   Apple's existing anchors.
5. **Persistence.** `/Library/LaunchDaemons/com.corbanu.pf83.plist` runs
   `pfctl -E -f /etc/pf.conf` at boot.

## The ruleset as loaded

```
pass in  quick on en0 inet proto tcp from 192.168.64.1 to (en0) port = 22 keep state
pass out quick on en0 inet proto tcp from (en0) port = 22 to 192.168.64.1 keep state
pass out quick on en0 inet proto udp from any port = 68 to 192.168.64.1 port = 67 keep state
pass in  quick on en0 inet proto icmp from 192.168.64.1 to (en0) icmp-type echoreq keep state
block drop log quick on en0 all
```

Rule 2 exists because an already-open SSH session has no state under a freshly
loaded ruleset, so its replies would be dropped and the session killed mid-load.

## The mediated pinhole is deliberately absent

The brief specified one outbound pinhole to a host mediator on port 8111. It was
installed, and verification found the guest could reach it — because **Docker on
the host was already listening on 8111**. The rule was pointing the guest at an
unrelated published container port rather than at a mediator. It was removed and
the guest is now fully sealed. The rule returns only when a real mediator exists,
on a port verified unused beforehand. Sealed is the correct state until then.

## Verification

As the unprivileged `agent` account, which is who the harness runs as. Every
result below was reproduced **after a reboot**.

| Check | Required | Observed |
| --- | --- | --- |
| `curl https://github.com` | fails | HTTP code `000` |
| `nc 192.168.64.1 22` | fails | denied |
| `nc 1.1.1.1 443` | fails | denied |
| `host github.com` | fails | denied |
| `nc 192.168.64.1 8111` | fails | denied |
| host to guest SSH | succeeds | succeeds |

Interface and routing state after reboot: a single default route via `en0`, no
`inet 100.x` and no `fd7a:` address on any interface, Tailscale service
**Disconnected**.

**Positive denial evidence, not merely absence of connectivity.** `pflog0` does
not exist on this macOS build, so packet logging was unavailable. Rule counters
were used instead, and they are stronger: after the reboot the block rule shows

```
block drop log quick on en0 all   [ Evaluations: 839  Packets: 839  Bytes: 84981  States: 0 ]
```

839 packets were actually matched and dropped by that rule. A timeout proves a
connection did not complete; a counter proves the filter is what stopped it.

## What this does and does not establish

Established: the guest cannot reach the internet, cannot reach the host except
for inbound SSH, cannot resolve DNS, and the filter survives reboot.

Not established: any PF-83 case result, the correctness of the harness, or the
integrity of the guest against an adversary with local admin. `neo`, `neo2` and
`root` remain admin accounts on the guest and can undo all of this; the control
assumes the guest is not hostile, only that the software under test is confined.

## Follow-ups

- Rotate the `neo-vm` password. Its length matches the one posted in Slack
  earlier today, so it is very likely the exposed value. Key access is installed,
  so rotation will not lock anyone out.
- Build the host mediator, verify its port is unused, then restore rule 4.
- Re-run the authoritative preflight and record the new control scores.
