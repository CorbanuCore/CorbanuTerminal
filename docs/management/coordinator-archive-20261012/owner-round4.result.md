The manager is on, but its first real action is stuck on a hold. The owner is promoted to **generation 3**, routing `tmux-workers`, with `manager_enabled: true`. It ticks normally: 44,431 ticks, no hold on the ticks themselves, 0 errors. But it held its first real action, `owner-first-check-01-r2`, with `wrong_ack`. So the "no holds" check fails.

## Firewall change and restore (the same in all three test runs)
I used `neo2` with the vault `neo-vm` login at use time. The original files were moved into a root-only folder (same disk, so inode and timestamp are kept), and test copies were put in their place. The test copy of the firewall rules has the `inference` address list and its port-443 rule commented out. The test `/etc/hosts` drops 6 lines: chatgpt.com ×2, auth.openai.com, api.openai.com and api.z.ai ×2. I removed Z.AI too, because the worker runs with full access and could call any endpoint it can reach.

| File | Before | During | After |
|---|---|---|---|
| `corbanu.pf83` | `58954a3f…975d` (inode 2387569) | `b4432fbb…7ac6` | identical to before |
| `/etc/hosts` | `6be8ce8e…cba5` (inode 17599) | `c7dd0e2e…e2da` | identical to before |
| `pf.conf` | `fb9fe39d…b53b` | unchanged | unchanged |

- **Credentials:** restored exactly after every run (`db552572…`, `4ac7ce27…`, `ec7ab237…`).
- **Shared folder:** remounted after every run.
- **After the restore:** api.z.ai answered 301, api.openai.com 421 and chatgpt.com 403. github.com still doesn't resolve, so PF-83 reruns work as before.

## Connection evidence (final run)
- **Outbound traffic:** the capture on the VM's network interface recorded 39 packets, all local housekeeping traffic or my own SSH connections from the host. **Zero packets to or from any public address, and no connection started by the VM.**
- **Blocked attempts:** the firewall log has 76 blocked attempts to provider addresses. All of them came from deliberate probes, either mine or the worker's own two probe commands. None happened during the controls or the rest of the worker run.

## Run receipts
- **Run 1 (`d23da0`):** one attempt was held because the model's ACK had a trailing newline, and the retry passed. Reviewer said NOT QUALIFIED: the controls didn't go through the owner, and leftover test files hadn't been checked.
- **Run 2 (`fcfd9a`):** passed, but the test account could remount the writable shared folder, which maps to your Mac's `/Volumes/FASTDRIVE2/z23`. Reviewer said NOT QUALIFIED.
- **Run 3 (`6ef76a`):**
  - I kept the shared folder mounted as root inside the root-only folder, so the test account's mount attempt fails ("Resource busy").
  - Old PF-83 files, `~/qual`, `~/.corbanu` and neo2's home were moved out of reach or locked for the run.
  - Positive control through the owner: ACK, START and RETURN, with all 3 model responses traced through the broker.
  - Broker-down control: failed safely, the owner held the action, and zero public connections.
  - Main run: 13 ticks, ACK, START and RETURN each counted once, all 23 model responses traced through the broker.
- I also corrected the round-3 write-up the reviewer flagged, in a separate corrections file; the original is unchanged.

## Review verdict
`claude-opus-5-5-plan` (session `01a1019e…`) said **QUALIFIED**: all four fields supported, counts 1/1/1. Limits it noted:
- The shared folder is blocked from inside the VM, not removed in UTM.
- The firewall and the packet capture both run inside the VM.
- No separate person designed or ran the test cases.
- There's no written spend authorization for using your ChatGPT subscription through the broker.

## Promotion
- **Preflight:** all five items passed on a clean export of commit `888d6dd1ad`.
- **Recipe:** ran to completion. My one change sets `manager_enabled: true` and recomputes the activation's config digest, as round 1 said was needed.
- **Result:** armed, **generation 3**, `tmux-workers`. Interval ticks have run since 12:15:50Z.

## Still open
1. **The held action needs a decision.** The model sent a correct ACK, but as a non-final message with a trailing newline, and the owner's exact-match check refused it. That has happened on 2 of 7 real first replies today. Fixing the check changes the package fingerprint, so the test would need redoing. There's also no command to clear a hold, and holds block any later reconfiguration.
2. **The held worker is still running on the host** (`w-ylg59fqs`), idle, with no START sent. After 30 minutes the watchdog will report it as stalled. To revoke admission: `--disarm --generation 3`. I didn't disarm, because that wouldn't clear the hold and would use up a generation.
3. **Dashboard not republished.** The wrapper still exports this worktree, including the other session's four uncommitted files.
4. **VM hardening follow-ups:** remove the z23 shared folder in UTM, and tighten the firewall's port-22 rule.
5. `slack-receiver-02` is still unsettled.

**Commit:** `3c8736914f` (record only), pushed fast-forward to `integrate/management-workstreams-20260911`. The four uncommitted files were not staged. Evidence is in `.codex-work/owner-manager-enable-20261002/round4/`.