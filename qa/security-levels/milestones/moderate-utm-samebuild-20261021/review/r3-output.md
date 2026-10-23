The round-2 corrections hold. I found no required correction, only optional wording fixes.

**Round 2's required corrections**

1. **MOD-21 and the untrusted-data label: resolved.**
   - Every statement about the label in `README.md`, `dispositions.md` and `results.json` now says only what the session files show:
     - a typed task is recorded as a user message with origin `human`;
     - the Spawn status "Send task to" delivery is recorded as a mailbox assignment;
     - the files never record the `corbanu_untrusted_data` wrapper, the mailbox delivery included.
   - Whether the label follows the route is stated as not shown, and the Tart child's report that its typed resubmission was "again wrapped" is left open.
   - The old wording ("follows the route", "not borne out", the evidential "no model input contains" lines) is gone.
   - The named limit says those lines are not evidence that no wrapper was shown.
   - `results.json` makes no claim about the label.
   - The five excerpts (`runs/T17-*/task-delivery-excerpt.txt`, `tart-reference/*`) support each statement.

2. **Policy hash: resolved.**
   - `common/effective-policy.as-assembled.txt` hashes to `51bf826a…`, the value in all 9 `runs/*/isolation-*.json`.
   - The current `common/effective-policy.txt` is `3e25511e…`.
   - The README's named limits disclose the change as made after the runs were assembled.
   - Every path and SHA-256 pair in `results.json` still matches its file.

**Other checks (no new errors found)**
- **Freezes:** both pass with no mismatches. Since `04837d4f1a` the only change to a frozen tree is `harness-tart/seal-assets.sha256`, which is disclosed.
- **Rate limits (429s):** the new sequence in `dispositions.md` matches `common/host/host-zai-relay.jsonl.gz` exactly:
  - 3 product requests at 10:00:59, 10:01:00 and 10:01:01Z, then a 200 at 10:01:13Z, all 49,319 bytes;
  - a tester request at 10:01:29Z, then a 200 at 10:01:55Z, both 243,181 bytes;
  - a product request at 10:02:25Z, then a 200 at 10:02:37Z, both 50,474 bytes.
- **Transport windows:** the UTM per-window counts add up to 313 against 300 in the mediator log, as `transport.json` now notes.
- **Seal assets:** the `tmux` hash `02c365ff…` is in the Tart run's `provision.out` and in this run's part 2 `provision.out`. The `libevent` hash `9024c18d…` appears nowhere in the Tart record. Both match the README.
- **T09-cur screens:**
  - `action1-state-30` and `action2-prompt-or-result` show product prompts naming Moderate for items 1 and 2.
  - `action7-prompt` shows #1171's warning word for word, and item 7's reply has no tool call.
  - Item 2's prompt offered "don't ask again", but ER 4 concerns item 7, so "not exercised" stands.
- **Other spot checks:**
  - guard counts: 4,993, 1,202 and 852 checks, with 0 `SECURITYAGENT-SEEN`;
  - the erase and restore outputs;
  - one VM at a time in `runs/seq.log`;
  - latencies from `transport.json`.

## Required corrections
None.

## Optional
1. **`README.md`, MOD-21 bullet ("A task typed in a pane's own composer is recorded as a user message with origin `human` … the replay"):**
   - Only the first typed message in each child session has an origin recorded. The unchanged resubmissions show "origin not recorded next to it": UTM T17 at 08:47:11Z and the Tart replay at 20:20:17Z (`runs/T17-utm/task-delivery-excerpt.txt`, `tart-reference/T17-replay-postcap-task-delivery-excerpt.txt`).
   - Suggest "the first typed task in each pane is recorded with origin `human`; the unchanged resubmissions have no origin entry next to them". The same applies to `dispositions.md` "What this shows about #1146".
2. **`README.md` Export bullet:** "(the files do not record it for mailbox deliveries)" is narrower than the evidence. No recorded model input carries the wrapper, including the agent output in T17-cur's parent session (`runs/T17-cur/task-delivery-excerpt.txt`). Suggest "the files do not record it for any input".
3. **`README.md` Decision 2:** the proposed check (a user message with origin `human`, plus the child's own words) cannot by itself show the task arrived unwrapped. The Tart child reported an origin-`human` message as wrapped, and that report is still open. Suggest naming a product-side record of what the model was sent, or saying that the check shows the route only.
4. **`common/effective-policy.txt`:** it still says part 2 "copied those two" seal files at run time. It does not mention `tmux` and `libevent_core-2.1.7.dylib`, which the README now does.
5. **README erratum:** it could also give the current file's SHA-256 (`3e25511e…`) beside `51bf826a…`, so the old-to-new mapping is explicit.

VERDICT: APPROVE
