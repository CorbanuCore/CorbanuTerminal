**Round 5 evidence check: commit `5bea9d2c9c` against `2c7c79e6d3`**

All three changes are supported by their evidence, and none of them introduces an error or contradiction elsewhere in the record. I only read files, hashed them and ran `git diff --check`, which is clean.

**Each change against its evidence**

1. **Repeats of a typed task (`README.md` lines 41–43 and `dispositions.md` lines 68–69)**
   - In every pane, only the first typed task has an origin, and it is `human`:
     - UTM T17 at 08:40:54Z;
     - Tart T17's resubmission at 16:28:27Z;
     - the Tart replay at 20:16:33Z.
   - The identical repeats show "origin not recorded next to it": UTM at 08:47:11Z, the UTM T17-utm-replay parent at 09:26:52Z, and the Tart replay at 20:20:17Z (`runs/T17-*/task-delivery-excerpt.txt`, `tart-reference/*.txt`).
   - "Repeats of a typed task" no longer seems to include the Tart T17 resubmission, which does have origin `human`. The two files now use the same wording.

2. **What the excerpt scan covers (`README.md` lines 153–154)**
   - `tools/excerpt_tasks.py` checks only `response_item` rows for the wrapper, leaving out reasoning, assistant replies and tool calls. It lists `inter_agent_communication` rows but does not scan them. So "response items … None does, agent output included" matches every excerpt's "none". The T17-cur parent's `<subagent_notification>` is a scanned user input.
   - "Stored as the message sent, before any wrapping" agrees with the product source. In `codex-rs/core/src/session/mod.rs`, `record_inter_agent_communication` (lines 4164–4200) saves the raw `InterAgentCommunication` to the session file. The prepared items go only to state and the taint step.

3. **Current policy file's hash (`README.md` line 182)**
   - `shasum -a 256 common/effective-policy.txt` gives `eb0d48b0ca60…`, which matches.
   - `common/effective-policy.as-assembled.txt` gives `51bf826a…`, the value cited in all 9 `runs/*/isolation-*.json`.
   - No other file in the record cites a stale hash.

**`effective-policy.txt` and the seal assets**
- `common/effective-policy.txt` did not change in this round; the diff stat shows no file under `common/`. Its seal-assets text was checked in round 4.
- The changes this round don't touch the seal-asset or `tart-run.sh` statements.

**`review-ledger.md` row 4** correctly lists the three fixes.

## Required corrections
None.

## Optional
None.

VERDICT: APPROVE
