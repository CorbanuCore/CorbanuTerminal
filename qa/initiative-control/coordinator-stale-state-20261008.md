# Coordinator stale-state cleanup (2026-10-08)

Travis approved two clean-ups after the [standby recovery](owner-manager-standby-recovery-20261008.md):
record security as resumed (hand-coordinated), and settle the stale hand claim `slack-receiver-02`.
Not changed: manager mode (standby), allocations, owner/manager launchd jobs, package, pins, Slack.

All coordinator writes used the owner CLI `coordinator_cli.py` from a byte-identical copy of the live runtime
(`python3 -B`, so the pinned runtime dir got no `__pycache__`). State: `initiative-control.oGQGyA/state/coordinator`.

## 1. Security resumed (revision 2981 → 2982)

There is no CLI operation for the workstream's text fields. `set_stream_mode` doesn't apply: the security
stream was already `mode: enabled`. The stale "paused" came from the 2026-09-13 `pause` text, the
"Owner paused" history line, and the manager's carried-over reasoning. So the change is recorded the way
earlier owner grants were (`authority:security-ownership-fable-20260914`): a meaningful `event`.

- `authority:security-resumed-hand-20261006`, kind `human_authority`, actor Travis. It says security has been
  active and hand-coordinated since the 2026-10-06 program decisions, supersedes the stored pause text, and grants
  the manager no allocation or dispatch. Release, flag removal and milestone sign-offs stay with Travis.
- The raw SQLite text was not edited.

## 2. `slack-receiver-02` settled (revision 2982 → 2983)

- Native inspection first, at 02:15Z: `/private/tmp/crecv.5GJbiB` (packet, home, tmux socket) is gone.
  No process matches session `01a09e29` or `crecv`. No Slack listener or poller is running.
- `reconcile_dispatch` (dispatcher `hand`, no agent) moved it from `running` to `failed`, with
  `owner_failure` evidence kind `owner_hand_claim_settlement`. Reason: a stale ACK-only receiver from
  2026-09-14, deadline long passed, stall already reported, session ended. No duplicate launch, no
  replacement and no Slack message.
- It has been archived to `action_history`. No action is in flight now, and the `slack-receiver` resource is free.
  Its allocation was compacted afterwards (see Follow-up).

## Manager reaction (standby, as designed)

The two meaningful events triggered one manager cycle, `0528f17a…`, at 02:16Z. It was **accepted with no
action**, prepared nothing and gave no verdicts. Its reason now reads: security is resumed but hand-coordinated
with no manager allocation, and the receiver claim is settled. It no longer says "security stays paused".

### Follow-up (02:18–02:25Z)

- **Second cycle.** Settling the claim left the `slack-receiver-02` allocation idle (its only action had failed,
  none accepted). The owner sent `owner-wake:no_prepared_work`, which started cycle `609a4fce…` at 02:18Z.
  That cycle was also no-action. It said "security and product continuation remain paused" again: the
  authority event had already been consumed, and every briefing still carries the workstream's stored 2026-09-13
  `pause` text.
- **Allocation compacted (2988 → 2989).** `compact_allocation` turned `slack-receiver-02` into the consumed stub.
  This is the same audited call routine compaction makes. The event is not meaningful, the original allocation
  stays in the audit table, and evidence is recorded. Without it, routine compaction would only run after the
  7-day failed-work grace. Until then the owner would send this wake every 12 hours.
  After compaction: no further wakes, both jobs idle with no hold or errors, owner armed at generation 11.

**Still open:** the stored security workstream text (`pause`, "Owner paused" history) cannot be changed through
any CLI operation. A raw SQLite edit was not made. So until that text is corrected, a manager briefing can
still conclude that security is paused. Fixing it needs either a small owner operation, for example to update a
workstream's text fields with evidence, or Travis's approval for a one-off audited state edit. The fix doesn't
affect dispatch: the stream is `mode: enabled`, and the manager has no security allocations.

## Dashboard

- `state/control.json`: reviewed edit removing the human-test card "3. Security — paused" (PF-83-S01, `blocked`).
  Nothing in security waits on a human test. Backup: `control.json.bak-20261008`. The `diff` shows only that
  card's lines; the parsed JSON equals the backup minus that card. SHA-256 `89053395…` → `ee7098fb…`.
  `control.json` is not part of the owner pins.
- A local render (`export.py --committed` + `control.py publish` into `/tmp`) shows no "Security — paused" card.
  The human test queue lists only the two PF-60-S03 cards.
- **Live publication is blocked.** `sync-source.sh` refused because the declared source checkout
  `worktrees/management-workstreams-20260911` is detached at `64137b7189`, not on
  `integrate/management-workstreams-20260911`. Another session owns that checkout; it was left untouched.
  The script's error trap marked the server `health.json` as sync failed (02:18:33Z). The last good snapshot
  (2026-10-06 18:55Z) still shows the old card. Re-run the sync once the checkout is back on its branch.
- Side effect fixed: the local preview export also rewrote the live `state/source.json` (export.py always
  records the collection there). It was restored byte-for-byte from the 2026-10-06 export copy, which matches the
  server's (`8d54f0c4…`).
