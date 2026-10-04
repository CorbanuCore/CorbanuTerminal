# PF-83 guest isolation — execution brief

**For:** an agent with administrator rights on the macOS VM at `192.168.64.3`.
**From:** Fable (Corbanu takeover manager). **Date:** 2026-09-15.

## 1. Objective

Make the guest unable to reach anything except two things:

1. inbound SSH from the host `192.168.64.1` (this is how the qualification harness drives the guest, and how I keep access);
2. one outbound TCP port to `192.168.64.1` as a mediated inference pinhole.

Everything else — internet, DNS, the rest of the host, other machines — must be denied and the denial must be observable.

**Why this matters:** the PF-83 preflight scored 2 proven and 173 of 175 controls UNPROVEN, with all 380 case verdicts `not_reached`, because the guest reached `github.com` and connected to `192.168.64.1:22`. No case may run until denial is enforced, because a pass would prove nothing.

## 2. What I already verified on the guest (2026-09-15)

Run as the unprivileged `agent` account over SSH:

- macOS `26.2`, arm64.
- `en0` = `192.168.64.3/24`, default gateway `192.168.64.1`.
- `en2` and `anpi0` exist but are `status: inactive`.
- **`utun6` is up with `a 100.64.0.0/10 CGNAT address` and `a fd7a:115c:a1e0::/48 address`, and `scutil --nc list` shows `(Connected) VPN (io.tailscale.ipn.macsys) "Tailscale"`.**

That last point is the important one. **Tailscale is a second, independent network path.** A packet-filter ruleset written only for `en0` will not close it, and it is a plausible reason the preflight saw the outside world. Step 1 below deals with it first; do not skip to the firewall.

## 3. Non-negotiables

- **Do not lock the host out of SSH.** Step 0 exists for this. If you skip it and get it wrong, the VM needs console recovery.
- **Do not modify, delete or re-privilege the `agent` account**, and do not change its authorized keys. The harness runs as `agent`.
- **Do not install any Corbanu, OpenAI, Anthropic or Slack credential on this guest**, and do not sign in to any account on it. A previous VM was rejected for exactly this.
- **Do not answer or suppress any credential, Keychain or permission prompt.** If one appears, stop and report it.
- Do not touch `/Users/agent`, the harness, or anything under it.
- Work only in `/etc/pf.conf`, `/etc/pf.anchors/`, `/Library/LaunchDaemons/`, and Tailscale's own state.

## 4. Step 0 — safety net, before any change

This reverts everything automatically unless you confirm within 10 minutes.

```sh
sudo tee /usr/local/bin/pf83-deadman.sh >/dev/null <<'EOF'
#!/bin/sh
sleep 600
[ -f /var/run/pf83.confirmed ] && exit 0
/sbin/pfctl -d 2>/dev/null
/sbin/pfctl -F all 2>/dev/null
EOF
sudo chmod 755 /usr/local/bin/pf83-deadman.sh
sudo rm -f /var/run/pf83.confirmed
sudo nohup /usr/local/bin/pf83-deadman.sh >/dev/null 2>&1 &
```

Also snapshot what you are about to change:

```sh
sudo cp -n /etc/pf.conf /etc/pf.conf.pf83.bak
sudo pfctl -sr > /tmp/pf83-rules-before.txt 2>&1; sudo pfctl -si >> /tmp/pf83-rules-before.txt 2>&1
```

You will clear the dead-man switch in Step 5, only after access is confirmed.

## 5. Step 1 — remove the second network path

Tailscale must be off and must stay off across reboot. Preferred order:

```sh
# 1. log the node out and disconnect
sudo tailscale logout || true
sudo tailscale down || true

# 2. stop and disable the daemon so it does not return at boot
sudo launchctl bootout system/com.tailscale.tailscaled 2>/dev/null || true
sudo launchctl disable system/com.tailscale.tailscaled 2>/dev/null || true

# 3. remove the network service so macOS does not re-establish it
sudo networksetup -removenetworkservice "Tailscale" 2>/dev/null || true
```

If this is the App Store or system-extension build and the commands above do not exist, **say so and stop** rather than improvising — I would rather uninstall the app cleanly than half-disable it.

Then prove it is gone:

```sh
ifconfig | grep -c utun          # note the count
ifconfig utun6 2>/dev/null       # expect: no such interface, or no inet address
scutil --nc list                 # expect: no Connected Tailscale entry
netstat -rn -f inet | grep default   # expect: only 192.168.64.1 via en0
```

Do not proceed until `netstat -rn` shows exactly one default route, through `en0`.

## 6. Step 2 — the packet filter

Create the anchor. `MEDIATOR_PORT` is the single inference pinhole; I will run the mediator on the host. Use `8111` unless I tell you otherwise.

```sh
sudo tee /etc/pf.anchors/corbanu.pf83 >/dev/null <<'EOF'
host     = "192.168.64.1"
iface    = "en0"
mediator = "8111"

set skip on lo0
set block-policy drop

# 1. inbound SSH from the host only. THIS is the rule that keeps the harness and me connected.
pass in quick on $iface proto tcp from $host to ($iface) port 22 flags S/SA keep state

# 2. DHCP renewal with the host only. Remove this line if the guest is given a static address.
pass out quick on $iface proto udp from any port 68 to $host port 67 keep state

# 3. ICMP echo from the host only, for diagnosis.
pass in quick on $iface inet proto icmp from $host to ($iface) icmp-type echoreq keep state

# 4. the single mediated pinhole: guest -> host mediator, nothing else.
pass out quick on $iface proto tcp from ($iface) to $host port $mediator keep state

# 5. everything else, both directions, both address families, denied and logged.
block drop log quick on $iface all
EOF
sudo chown root:wheel /etc/pf.anchors/corbanu.pf83
sudo chmod 644 /etc/pf.anchors/corbanu.pf83
```

Hook it into the main config **by appending, without editing Apple's existing anchors**:

```sh
printf '\nanchor "corbanu.pf83"\nload anchor "corbanu.pf83" from "/etc/pf.anchors/corbanu.pf83"\n' | sudo tee -a /etc/pf.conf >/dev/null
sudo pfctl -n -f /etc/pf.conf            # syntax check; must print nothing and exit 0
sudo pfctl -E -f /etc/pf.conf            # enable and load
sudo pfctl -si | head -3                 # expect: Status: Enabled
```

Order matters: the `pass ... quick` rules come first, the catch-all `block ... quick` last. First match wins, so the deny is the fallthrough.

Expect Apple services to break inside the guest: software update, iCloud, Spotlight suggestions, time sync. That is intended, not a fault.

## 7. Step 3 — persistence

pf is off by default on macOS, so it needs a daemon to re-enable at boot:

```sh
sudo tee /Library/LaunchDaemons/com.corbanu.pf83.plist >/dev/null <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.corbanu.pf83</string>
  <key>ProgramArguments</key>
  <array><string>/sbin/pfctl</string><string>-E</string><string>-f</string><string>/etc/pf.conf</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><false/>
</dict></plist>
EOF
sudo chown root:wheel /Library/LaunchDaemons/com.corbanu.pf83.plist
sudo chmod 644 /Library/LaunchDaemons/com.corbanu.pf83.plist
sudo launchctl bootstrap system /Library/LaunchDaemons/com.corbanu.pf83.plist
```

## 8. Step 4 — verification, and what each result means

Run the first block **as the `agent` user** (`su - agent` or a separate SSH session), because that is who the harness runs as.

| # | Command | Required result |
|---|---|---|
| 1 | `curl -sS -m 10 -o /dev/null -w '%{http_code}\n' https://github.com` | **fails** (timeout / could not connect). A `200` means isolation is not enforced. |
| 2 | `nc -z -v -w 5 192.168.64.1 22` | **fails** |
| 3 | `nc -z -v -w 5 1.1.1.1 443` | **fails** |
| 4 | `dig +time=3 +tries=1 github.com` or `host github.com` | **fails** |
| 5 | `nc -z -v -w 5 192.168.64.1 8111` | **succeeds** once I have the mediator running; until then it may fail, and that is expected |

Then, **from the host**, in a new shell:

| # | Command | Required result |
|---|---|---|
| 6 | `ssh -i <key> agent@192.168.64.3 true; echo $?` | **0** |

Capture positive denial evidence, which is worth more than "it timed out":

```sh
sudo tcpdump -n -e -ttt -i pflog0 &     # leave running
# re-run tests 1 to 4 from the agent session
# expect block entries naming en0 and the destinations
```

Save `sudo pfctl -sr`, `sudo pfctl -si`, and the pflog capture.

## 9. Step 5 — confirm, then reboot and re-verify

Only after test 6 passes:

```sh
sudo touch /var/run/pf83.confirmed     # cancels the dead-man revert
sudo reboot
```

After the guest is back, re-run **every** check in Step 4, plus:

```sh
sudo pfctl -si | head -3          # Status: Enabled
sudo pfctl -sr | head -20         # the five rules above
scutil --nc list                  # no Connected Tailscale
netstat -rn -f inet | grep default
```

Persistence is the point. A ruleset that dies at reboot is worse than none, because it looks enforced.

## 10. Rollback

```sh
sudo pfctl -d; sudo pfctl -F all
sudo launchctl bootout system/com.corbanu.pf83 2>/dev/null
sudo rm -f /Library/LaunchDaemons/com.corbanu.pf83.plist /etc/pf.anchors/corbanu.pf83
sudo cp /etc/pf.conf.pf83.bak /etc/pf.conf
```

## 11. Report back

1. Confirmation that Tailscale is logged out, disabled and removed as a service, with the `scutil --nc list` and `netstat -rn` output.
2. The contents of `sudo pfctl -sr` and `sudo pfctl -si`.
3. The Step 4 table with actual outputs, before and after reboot, including the pflog capture.
4. Anything you changed that is not in this brief, and anything you chose not to do and why.
5. Whether the guest still had a static or DHCP address, and whether you kept rule 2.

**Do not report success unless tests 1 to 4 fail and test 6 passes, both before and after the reboot.** Partial enforcement is a worse outcome than none, because it invites a false qualification.

## 12. Two notes for Travis, not for the executing agent

- The password posted in Slack should still be rotated. I have not used it and have not stored it; my access is the existing `agent` key, which is enough for everything above except the admin steps.
- The executing agent needs admin on the guest. Give it a credential you are willing to rotate afterwards, or run Steps 1 to 3 yourself and let it do Steps 4 and 5.
