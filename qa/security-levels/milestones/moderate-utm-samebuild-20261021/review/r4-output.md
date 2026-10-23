**Round 4 evidence check: commit `2c7c79e6d3` against `8d6c780779`**

All five changes from round 3 are supported by the evidence they cite, and none of them contradicts anything else in the record. I checked them by reading files and hashing them; nothing was run that writes.

**Each change against its evidence**
- **README MOD-21 bullet and dispositions.md "What this shows about #1146", first typed task versus resubmissions:**
  - In each child pane, only the first typed task has an origin, and it is `human`:
    - UTM T17 at 08:40:54Z (`runs/T17-utm/task-delivery-excerpt.txt`);
    - the Tart replay at 20:16:33Z (`tart-reference/T17-replay-postcap-task-delivery-excerpt.txt`);
    - Tart T17's resubmission at 16:28:27Z, which is the first typed task in that pane (`tart-reference/T17-task-delivery-excerpt.txt`).
  - The identical repeats show "origin not recorded next to it": UTM at 08:47:11Z and the Tart replay at 20:20:17Z.
  - "No origin entry next to them" matches what `tools/excerpt_tasks.py` checks: it looks only at the next two rows for a `source_origin` entry.
  - dispositions.md line 58, "(origin `human`)", agrees.
- **Export bullet, agent output:** the T17-cur parent's `<subagent_notification>` is a user-role input that the script scans, and the result is "none". This is supported. For the mailbox part of this bullet, see optional item 1.
- **Decision 2:** the claim that such a run shows the route only follows from the record. The Tart child's "again wrapped" report concerns an origin-`human` typed task (dispositions.md lines 57–58).
- **`common/effective-policy.txt`, seal assets:**
  - `../moderate-tart-parity-20261020/harness/tart-run.sh` lines 60–62 copy `seal-guest.sh` and `dns-refuse.py` from `scripts/qa/tart/`. They copy `tmux` and `libevent_core-2.1.7.dylib` from host paths.
  - `harness-tart/tart-run.sh` has the same lines. Its differences from the Tart copy are only the run paths, VM name and lock holder.
  - `harness-tart/seal-assets.sha256` lists all four files. "Same SHA-256 as the Tart record's copies" is still claimed only for the two scripts, which is right.
- **README policy erratum:** git log shows `common/effective-policy.txt` changed in `3269d3cd3a` (round 1 fixes) and in `2c7c79e6d3` (round 3 fixes) only. The round 2 commit added only `effective-policy.as-assembled.txt`.
  - That file hashes to `51bf826a…`, the value in all 9 `runs/*/isolation-*.json`.
  - Nothing else in the record hashes the current policy file, so no path and SHA-256 pair is now stale.
- `git diff --check` is clean.

## Required corrections
None.

## Optional
1. **`README.md` line 153 (Export bullet), "mailbox deliveries … included":**
   - The excerpt script checks only `response_item` rows for the wrapper. It skips `inter_agent_communication` rows (`tools/excerpt_tasks.py` lines 27–31), so the excerpts' "none" does not cover the mailbox delivery.
   - The claim itself still holds, from the product code. `record_inter_agent_communication` writes the raw message to the session file, not the prepared model input (`codex-rs/core/src/session/mod.rs`, about lines 4182–4200).
   - Suggest saying that the scan covers response items, and that the session file stores the mailbox message as sent, before any wrapping.
2. **`README.md` lines 41–43:** "the unchanged resubmissions have no origin entry" could be read to include "the T17 resubmission" on Tart, which does have origin `human`. Suggest "the unchanged repeats of a typed task".
3. **`README.md` line 182:** round 3's optional item 5 was applied only in part. The current file is now `eb0d48b0…`, not `3e25511e…`. Giving that hash would make the old-to-new mapping explicit, and the round 3 summary in `review-ledger.md` would then be fully accurate.

VERDICT: APPROVE
