The manager is **not** on. The required safety check that must pass before the owner is disarmed (the round-94 preflight) refused four of its five items, so I stopped before step 3. The owner is still **armed at generation 1, scope `fixture-only`** and ticks normally. Nothing was disarmed, reinstalled or re-armed, so there is no new generation.

Separately, even a successful switch would not make it "dispatch work by itself." In tmux-workers mode the owner only runs prepared actions that have been explicitly handed to it. New actions go to hand dispatch by default. `manager_enabled` only changes a label on fixture-only ticks. The owner never creates work. Travis should know this before re-answering.

**State before and after (unchanged)**
- **Owner:** armed, generation 1, `fixture-only`, decision `owner-recurrence-domain-20260917` rev 3. Config digest `cd07b20b…d6f5`.
- **Package digests:** installed `84c09899…5625`; the 704650ccd8 code is `8021fc53…2065`.
- **Holds:** no unresolved holds and no recovery required.
- **Coordinator:** revision 2911, enabled, no manager cycle, no handoff yet.
- **Actions:** 4 accepted, 5 failed, 1 running (`slack-receiver-02`, stalled, already reported). **None are prepared**, so there is nothing for the owner to run.
- **Ticks after the refusal:** 42,967 total, `hold: null`, `firing: interval`, 0 consecutive errors, launchd last exit 0.

**Preflight (exit 2)**

| Item | Result |
| --- | --- |
| 1. Manager prepares actions through `freeze_worker_inputs` | Not met: not adopted |
| 2. A fresh prepared action to hand over | Not met: none exist |
| 3. No unfinished publication write | Passed |
| 4. Real worker test, or Travis's named limitation | Not met: neither exists |
| 5. Hand dispatch and publishers quiet | Not met: needs item 2, and hand workers are running in `workers-20261002` |

Travis's "yes" covers the activation decision, but not items 1, 2 or 4. The procedure says a limitation record I write in his name grants nothing.

**Commands that changed state**
- **Binary:** copied to `/Volumes/CorbanuDrive/Corbanu/.codex-work/owner-worker-bin/704650ccd8/corbanu` (read-only). sha256 `7b8c77a613e48214e9e6b29d70941d634d05222175140f7e57d5cfcba36dd77c`, the same as the source; the copy's signature checks out and it runs as corbanu 0.1.48.
- **runs_dir:** created `/Users/Neo/.local/state/corbanu-owner-runs` (0700, local disk, not /tmp).
- **Transport:** the old file is kept as `private/transport-archive-20260917.json`. I wrote a new `private/transport.json` (0600, sha256 `6520870d…b2b1`) with the new binary and runs_dir, and the same tmux and auth_link. It passes the transport check, and the auth file was never opened. The live config does not use it yet.
- **Dashboard:** ran `sync-source.sh`, exit 0.
- **Record:** wrote `qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md`, uncommitted.

Raw evidence (status JSONs, preflight result, authority record, sync log) is in `.codex-work/owner-manager-enable-20261002/`.

**Dashboard result**
- `health.json` now shows `collected_at` `2026-10-02T23:46:02Z`, previously Sept 27.
- **The sync did not refuse the dirty checkout. It published the uncommitted working-tree versions of the 4 files in `scripts/initiative_control/`, labeled as commit 704650ccd8.** I checked their hashes; the checkout itself was not touched.
- The decision feed is still dated 2026-09-20 and shows the owner's recurrence as `unknown / observation-stale`. Republishing does not refresh it.

**Left open**
1. Change the manager so it prepares actions through `freeze_worker_inputs`, with explicit provider and `--yolo` authority.
2. Get at least one fresh prepared action with an accepted manager decision.
3. Either run a real isolated worker test (ACK/START/RETURN) on binary `7b8c77a6…` with package `8021fc53…` and an independent review, or get Travis's own named-limitation acceptance.
4. Stop hand dispatch and raw tmux sends, with publishers quiet, within 300 s of the check.
5. Settle `slack-receiver-02`.

After that, run the promotion recipe as written. The new activation will be generation 3. The replacement config must also set `manager_enabled: true`, because the preflight copies the current `false` value.