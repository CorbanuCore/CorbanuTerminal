I've merged the timeout fix, but I couldn't fix the login dependency. It needs a credential decision from Travis. I didn't requalify, reinstall or repin anything, so both live jobs still run the old package. Both are ticking with no holds.

**What shipped: PR #268, merged** (merge commit `362007413c`, all checks passed)
- **Timeout fix:** if a subprocess times out before any manager cycle starts, the manager job now counts it as an error and keeps going, like the owner job. The likely source is the 5-second interpreter check at the start of each tick.
- **Still stops:** three such timeouts in a row latch a hold named `manager_pre_cycle_timeouts`. A timeout after a cycle has started, or any other error, still stops the job at once. `--recover` resets the count.
- **Tests:** the manager-lane tests pass (25/25) and the manager-cycle tests pass (30/30). The full suite ran 971 tests with 2 failures, and both fail the same way on untouched `main`.
- **Review:** the first Opus 5.5 High review asked for changes. The fix had missed the only real pre-cycle timeout, the interpreter check, so I moved it. The second review approved, and I applied its three minor notes.
- **Record:** `qa/initiative-control/management-bootstrap/manager-lane-timeout-and-logout-20261008.md`, linked from the PF-80-S01 sprint file. Evidence is in `.codex-work/owner-manager-enable-20261002/round8/`.

**Why the login fix is blocked**
- The manager job runs in the GUI session because each cycle reads the Claude login from the Corbanu vault, and that vault's key is in Travis's login keychain.
- From the background domain (`user/501`, where the owner job runs) that read fails with "User interaction is not allowed". The round-7 test showed this.
- After a logout the login keychain is locked whatever domain the job is in, root LaunchDaemon included.
- So just moving the job would turn every cycle into a credential failure and a hold.

**Decision for Travis: how should the manager get its Claude login without his session?**
1. **Separate manager vault (recommended if he still wants this fix).**
   - A new vault under `corbanu-owner-live/private/` holds only the Claude subscription token. Its key sits in a file only he can read, not in the keychain.
   - The job then moves to the background domain, and the code change is small.
   - It needs Travis to provision the token, or to approve me copying it without displaying it.
   - Cost: any process running as Travis could read the token. That partly reverses his Oct 3 rule that the manager reads its login from the vault and never from a file.
2. **Keep it as it is.** Screen lock doesn't stop the manager; only logging out or rebooting does. It's on standby with no allocations, so little is lost.

After he decides, one requalification and repin will cover both changes. If he picks 2, I requalify for the timeout fix alone.

**Live state and other findings**
- Both jobs show no holds and no errors. The manager ran one cycle on its own (a "no prepared work" wake-up) and it ended with no action, so standby is working.
- Owner ticks are slow right now (load around 15) but they finish.
- **Reboot:** neither job is in `~/Library/LaunchAgents`, so neither restarts after a reboot. Whether the owner job survives a full logout has never been tested. I'd suggest deciding on a reboot-safe install at the same time.
- I removed my own worktree, and nothing the live settings point to was touched.